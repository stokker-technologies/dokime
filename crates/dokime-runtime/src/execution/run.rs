/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

// Third Party Imports
// Third-Party Imports
use tokio::sync::watch;
use tracing::Instrument;

// Local Imports
use super::Environment;
use super::ExecutionEvidence;
use super::template::SidecarTemplate;
use super::unwind::caught;
use crate::artifacts::Artifacts;
use crate::definition::TestCaseTemplate;
use crate::diagnostics::Diagnostic;
use crate::parameters::FromParameters;
use crate::records::Deliveries;
use crate::telemetry::{SampleSink, Streams};
use crate::transport::ExecutionTransport;
use crate::transport::buffer::{BufferedFrame, Executions, Tasks, lock};
use crate::transport::frame::{ExecutionSampleSink, FrameArtifactSink, FrameRecordSource, FrameTransport};
use crate::transport::{SidecarCriterionOutcome, SidecarCriterionStatus, SidecarExecutionOutcome};
use dokime_domain::model::execution::termination::TerminationReason;
use dokime_domain::model::values::Value;

/* === Constants === */

const CRITERION_BUDGET: Duration = Duration::from_secs(1);
const INITIALIZATION: &str = "initialization";
const TRACEABILITY: &str = "traceability";
const PARAMETERS: &str = "resolving parameters";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Inputs {
    pub controls: Arc<crate::transport::control::Controls>,
    pub callbacks: super::Callbacks,
    pub parameters: BTreeMap<String, Value>,
    pub bindings: crate::plugin::Configuration,
    pub caller: super::Caller,
    pub binding: Option<Arc<dyn std::any::Any + Send + Sync>>,
    pub observers: Vec<crate::execution::DeclaredObserver>,
    pub records: Vec<crate::records::RecordContract>,
    pub streams: Vec<crate::telemetry::TelemetryContract>,
    pub host: Option<crate::execution::SampleOutput>,
    pub publisher: Option<super::template::SamplePublisher>,
}

/* === Enums === */

enum Ended {
    Completed,
    Cancelled { step: String },
    Failed { step: String, diagnostic: Box<Diagnostic> },
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run<T: SidecarTemplate>(
    executions: Executions, token: String, source: String, declaration: TestCaseTemplate, inputs: Inputs, cancel: watch::Receiver<bool>,
    observer_tasks: Tasks,
) {
    let span = tracing::info_span!(
        "execution",
        execution = %token,
        template = %declaration.metadata.identifier,
        plugin = %source,
    );

    execute::<T>(executions, token, source, declaration, inputs, cancel, observer_tasks).instrument(span).await;
}

/* === Private Functions === */

async fn execute<T: SidecarTemplate>(
    executions: Executions, token: String, source: String, declaration: TestCaseTemplate, inputs: Inputs,
    mut cancel: watch::Receiver<bool>, observer_tasks: Tasks,
) {
    let Inputs {
        controls,
        callbacks,
        parameters,
        bindings,
        caller,
        binding,
        observers: declared_observers,
        records,
        streams,
        host,
        publisher,
    } = inputs;
    let host_managed_subscriptions = host.is_some() || lock(&executions).get(&token).is_some_and(|state| state.local_inputs);
    let mut transport = FrameTransport::new(Arc::clone(&executions), token.clone());
    let artifacts = Artifacts::new(token.clone(), Arc::new(FrameArtifactSink::new(Arc::clone(&executions), token.clone())));
    // Watching starts before the first stage does, because the hook that asks
    // for a directory is itself a stage, and it ends after cleanup, because a
    // template that writes its last evidence while releasing what it held is
    // exactly the case a sweep at the last step would miss.
    let watched = artifacts.clone();
    let scanning = super::scanning::start(artifacts.clone(), &executions, &token, &observer_tasks);
    let mut evidence = ExecutionEvidence::new();
    transport.capture_phase(dokime_domain::model::execution::CapturePhase::Initialization);
    // The inputs are decoded once, here, before anything runs against them: a
    // case whose values are not what the template declared is refused by the
    // field that does not fit, rather than run until whichever step reads it
    // first fails.
    let typed = match T::Parameters::from_parameters(&parameters) {
        Ok(typed) => typed,
        Err(refusal) => {
            super::scanning::settle(scanning, &watched, &executions, &token).await;
            if let Err(diagnostic) =
                finalize_record_inputs(&executions, &token, &mut cancel, &mut evidence, host_managed_subscriptions).await
            {
                transport.log(super::log::Level::Error, diagnostic.redacted().to_string());
            }
            let outcome = conclude::<T>(
                &declaration,
                None,
                &transport,
                &evidence,
                Ended::Failed { step: PARAMETERS.into(), diagnostic: Box::new(Diagnostic::of(refusal.to_string())) },
                None,
                false,
            )
            .await;
            report(&executions, &token, outcome);
            return;
        }
    };
    // Shared rather than borrowed from this frame: an observer runs on a task of
    // its own and rebuilds the environment over exactly these values, so what a
    // step reads and what an observer reads are one decoding of one case.
    let typed = Arc::new(typed);
    let parameters = Arc::new(parameters);
    let bindings = Arc::new(bindings);
    // One grant for every row this execution produces, shared by the steps that
    // push and by the observers that measure, so both are counted, sequenced and
    // seen by whatever is listening in the same place.
    let sink = Arc::new(ExecutionSampleSink::<T>::new(Arc::clone(&executions), token.clone(), source.clone(), host, publisher));
    let listening = lock(&executions).get(&token).expect("a running execution retains its state").listening.clone();
    let streams = Streams::new(Arc::clone(&sink) as Arc<dyn SampleSink>).with_listening(listening).restricted(
        declaration.metadata.identifier.as_str(),
        streams
            .iter()
            .map(|contract| contract.stream.to_string())
            // An observer's identifier is its own name, never a stream, so
            // the streams it may publish on come off the contracts it
            // declared beside it.
            .chain(declared_observers.iter().flat_map(|observer| observer.streams.iter().map(|contract| contract.stream.to_string()))),
    );
    // What the host files into this run's reach, read by a step that is waiting
    // for something this run does not produce itself.
    let deliveries = Deliveries::new(Arc::new(FrameRecordSource::new(Arc::clone(&executions), token.clone())));
    // One index per attempt, over the document this template declared: every
    // stage and every observer tick decides "already there" against the same
    // value, so a section one of them added is one none of them can add again.
    let document = Arc::new(std::sync::Mutex::new(super::document::DocumentIndex::new(declaration.document.clone())));
    // Who this run is, for the processes its steps start.
    let identity = lock(&executions)
        .get(&token)
        .map(|state| super::ExecutionIdentity {
            execution_id: state.request.test_execution_id.clone(),
            cycle_id: state.request.test_cycle_id.clone(),
            template_id: state.request.template_identifier.clone(),
            plugin_id: source.clone(),
        })
        .unwrap_or_default();
    let environment =
        Environment::new(typed.as_ref(), parameters.as_ref(), bindings.as_ref(), artifacts.clone(), streams.clone(), deliveries.clone())
            .identified(identity)
            .calling(caller.clone())
            .declaring(Arc::clone(&document))
            .with_callbacks(callbacks);
    let environment = match binding.clone() {
        Some(binding) => environment.bound(binding),
        None => environment,
    };

    let (mut template, own) = match initialize::<T>(&environment, &mut transport, &mut cancel).await {
        Ok(template) => template,
        Err(ended) => {
            let cleanup_error = settle_callbacks(&environment, &transport).await;
            environment.streams().flush().await;
            super::scanning::settle(scanning, &watched, &executions, &token).await;
            if let Err(diagnostic) =
                finalize_record_inputs(&executions, &token, &mut cancel, &mut evidence, host_managed_subscriptions).await
            {
                transport.log(super::log::Level::Error, diagnostic.redacted().to_string());
            }
            let outcome = conclude::<T>(&declaration, None, &transport, &evidence, ended, cleanup_error, false).await;
            report(&executions, &token, outcome);
            return;
        }
    };

    let own = Arc::new(own);
    let mut environment = environment.holding(Arc::clone(&own));

    // From here on the attempt holds handles, so every ending runs cleanup —
    // including a traceability failure, which ends the attempt before its first
    // step after initialization constructed the handles cleanup has to close.
    let stated = state::<T>(&mut template, &environment, &mut transport, &mut cancel).await;
    let mut ended = match stated {
        Err(ended) => ended,
        Ok(()) => {
            let observers = super::observing::start::<T>(
                super::observing::Scope {
                    executions: Arc::clone(&executions),
                    token: token.clone(),
                    source: source.clone(),
                    caller: caller.clone(),
                    binding: binding.clone(),
                    own: Arc::clone(&own),
                    parameters: Arc::clone(&typed),
                    raw: Arc::clone(&parameters),
                    bindings: Arc::clone(&bindings),
                    artifacts: artifacts.clone(),
                    streams: streams.clone(),
                    deliveries: deliveries.clone(),
                    document: Arc::clone(&document),
                },
                declared_observers,
                &records,
                &observer_tasks,
            );
            let ended = walk::<T>(&mut template, &declaration, &environment, &mut transport, &mut evidence, &mut cancel, &controls).await;
            controls.stop_accepting();
            // Observing stops before the evidence is read, so a criterion
            // decides over a set of facts that is not still growing underneath
            // it — and before cleanup, so no observer outlives the handle it
            // reads.
            super::observing::settle(observers).await;
            environment.streams().flush().await;
            ended
        }
    };

    if let Some(diagnostic) = settle_callbacks(&environment, &transport).await
        && matches!(ended, Ended::Completed)
    {
        ended = Ended::Failed { step: "settling callbacks".into(), diagnostic: Box::new(diagnostic) };
    }

    let reason = match &ended {
        Ended::Completed => TerminationReason::Completed,
        Ended::Failed { .. } => TerminationReason::Failed,
        Ended::Cancelled { .. } => TerminationReason::Cancelled,
    };
    transport.capture_phase(dokime_domain::model::execution::CapturePhase::Cleanup);
    let cleanup_error = clean_up(&mut template, &environment, &mut transport, reason).await;
    transport.capture_phase(dokime_domain::model::execution::CapturePhase::Default);
    let callback_error = settle_callbacks(&environment, &transport).await;
    let cleanup_error = cleanup_error.or(callback_error);
    // Whatever cleanup pushed reaches the host before the run reports.
    environment.streams().flush().await;

    // The watch stops here, after cleanup has written whatever it was going to
    // write and before the run reports: an artifact reaches the host complete,
    // or it does not reach it at all, and either way it is settled before the
    // outcome that says the run is over.
    super::scanning::settle(scanning, &watched, &executions, &token).await;
    // Static criteria need no instance; release ordinary owned resources too.
    drop(template);
    // The attempt's handles close here, after cleanup returned and after every
    // observer joined, and before a single criterion is decided.
    environment.release();
    drop(own);
    if matches!(ended, Ended::Cancelled { .. }) && T::cancellation_is_completion(&parameters) {
        ended = Ended::Completed;
    }

    if let Some(diagnostic) = environment.streams().failure() {
        ended = Ended::Failed { step: "publishing stream evidence".into(), diagnostic: Box::new(diagnostic) };
    }

    // Gaps join the evidence where the observers' rows did: after everything
    // that could still be measured has stopped, and before the first criterion
    // decides anything. A criterion reading `ctx.gaps` is reading a settled
    // fact about the run, not a running count.
    let inputs_settled = match finalize_record_inputs(&executions, &token, &mut cancel, &mut evidence, host_managed_subscriptions).await {
        Ok(()) => true,
        Err(diagnostic) => {
            ended = Ended::Failed { step: "settling record subscriptions".into(), diagnostic: Box::new(diagnostic) };
            false
        }
    };
    evidence.record_gaps(transport.gaps());
    let outcome =
        conclude::<T>(&declaration, inputs_settled.then_some(&environment), &transport, &evidence, ended, cleanup_error, !inputs_settled)
            .await;
    report(&executions, &token, outcome);
}

async fn finalize_record_inputs(
    executions: &Executions, token: &str, cancel: &mut watch::Receiver<bool>, evidence: &mut ExecutionEvidence, host_managed: bool,
) -> Result<(), Diagnostic> {
    let result = if host_managed { settle_record_inputs(executions, token, cancel).await } else { Ok(()) };
    if let Some(state) = lock(executions).get_mut(token) {
        evidence.subscribed_records = state.captured_records();
        for sample in state.take_staged() {
            evidence.record_sample(sample);
        }
    }
    result
}

async fn settle_record_inputs(executions: &Executions, token: &str, cancel: &mut watch::Receiver<bool>) -> Result<(), Diagnostic> {
    if let Some(state) = lock(executions).get_mut(token) {
        state.begin_record_settle();
    }
    let was_cancelled = *cancel.borrow();
    let bound = if was_cancelled { Duration::from_secs(1) } else { Duration::from_secs(35) };
    // The host settles by handing captured records and samples back. Bound
    // idle time between new inputs or their first settlement confirmation,
    // rather than total replay duration.
    let mut advanced = tokio::time::Instant::now();
    let mut deadline = advanced + bound;
    tracing::debug!(
        execution_token = token,
        cancelled = was_cancelled,
        idle_bound_ms = bound.as_millis(),
        "execution input settlement started"
    );
    let mut staged = None;
    loop {
        let progress = match lock(executions).get(token) {
            None => return Ok(()),
            Some(state) if state.record_subscriptions_settled() => return Ok(()),
            Some(state) => state.settlement_progress(),
        };
        if !was_cancelled && staged.is_some_and(|staged| progress > staged) {
            let now = tokio::time::Instant::now();
            tracing::debug!(
                execution_token = token,
                progress,
                idle_ms = now.duration_since(advanced).as_millis(),
                "execution input settlement advanced"
            );
            advanced = now;
            deadline = now + bound;
        }
        staged = Some(progress);
        if tokio::time::Instant::now() >= deadline {
            tracing::debug!(
                execution_token = token,
                progress,
                idle_ms = advanced.elapsed().as_millis(),
                "execution input settlement expired"
            );
            return Err(Diagnostic::of(format!("host did not settle execution inputs before the deadline; input progress: {progress}")));
        }
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_millis(5)) => {},
            _ = cancel.changed(), if !was_cancelled => {
                return Err(Diagnostic::of("record subscription settlement was interrupted"));
            }
        }
    }
}

async fn initialize<T: SidecarTemplate>(
    environment: &Environment<'_, T, T::Own, T::Parameters>, transport: &mut dyn ExecutionTransport, cancel: &mut watch::Receiver<bool>,
) -> Result<(T, T::Own), Ended> {
    let mut ctx = environment.initialize(transport);
    ctx.info("execution initialization started");
    let initialized = tokio::select! {
        biased;
        _ = cancel.wait_for(|cancelled| *cancelled) => Err(Ended::Cancelled {
            step: INITIALIZATION.into(),
        }),
        result = caught(T::initialize(&mut ctx)) => result.map_err(|diagnostic| Ended::Failed {
            step: INITIALIZATION.into(), diagnostic: Box::new(diagnostic),
        }),
    }?;
    ctx.info("execution initialization completed");

    Ok(initialized)
}

async fn state<T: SidecarTemplate>(
    template: &mut T, environment: &Environment<'_, T, T::Own, T::Parameters>, transport: &mut dyn ExecutionTransport,
    cancel: &mut watch::Receiver<bool>,
) -> Result<(), Ended> {
    let sections = {
        let ctx = environment.traceability(transport);
        tokio::select! {
            biased;
            _ = cancel.wait_for(|cancelled| *cancelled) => Err(Ended::Cancelled {
                step: TRACEABILITY.into(),
            }),
            result = caught(template.traceability(&ctx)) => result.map_err(|diagnostic| Ended::Failed {
                step: TRACEABILITY.into(), diagnostic: Box::new(diagnostic),
            }),
        }?
    };
    if !sections.is_empty() {
        transport.traceability(sections);
    }

    Ok(())
}

async fn walk<T: SidecarTemplate>(
    template: &mut T, declaration: &TestCaseTemplate, environment: &Environment<'_, T, T::Own, T::Parameters>,
    transport: &mut dyn ExecutionTransport, evidence: &mut ExecutionEvidence, cancel: &mut watch::Receiver<bool>,
    controls: &crate::transport::control::Controls,
) -> Ended {
    use crate::transport::control::Action;
    let interactive = controls.interactive();
    let mut automatic = 0;
    loop {
        transport.capture_phase(dokime_domain::model::execution::CapturePhase::Default);
        let command = if interactive {
            loop {
                if *cancel.borrow() {
                    return Ended::Cancelled { step: "waiting for command".into() };
                }
                if let Some(command) = controls.take() {
                    break Some(command);
                }
                tokio::select! {
                    biased;
                    _ = cancel.wait_for(|cancelled| *cancelled) => return Ended::Cancelled { step: "waiting for command".into() },
                    diagnostic = environment.streams().refused() => return Ended::Failed {
                        step: "waiting for command".into(), diagnostic: Box::new(diagnostic),
                    },
                    _ = controls.changed.notified() => {},
                }
            }
        } else {
            None
        };
        let declared = match command.as_ref().map(|command| &command.action) {
            Some(Action::Finish) => return Ended::Completed,
            Some(Action::Step(identifier)) => declaration.phases.iter().find(|step| &step.metadata.identifier == identifier),
            Some(Action::Next) => {
                let selected = dokime_domain::model::execution::result::effective_attempts(declaration, &evidence.attempts, None);
                declaration.phases.iter().find(|step| !selected.contains_key(&step.step_id))
            }
            None => {
                let step = declaration.phases.get(automatic);
                automatic += 1;
                step
            }
        };
        let Some(declared) = declared else {
            if let Some(command) = &command {
                controls.complete(&command.id, None, Some("no declared step is selected by this command".into()));
                continue;
            }
            return Ended::Completed;
        };
        if let Err(message) =
            dokime_domain::model::execution::result::validate_invocation(declaration, &evidence.attempts, &declared.step_id)
        {
            if let Some(command) = &command {
                controls.complete(&command.id, None, Some(message.into()));
                continue;
            }
            return Ended::Failed { step: declared.metadata.identifier.to_string(), diagnostic: Box::new(Diagnostic::of(message)) };
        }
        let step = declared.metadata.identifier.clone();
        // Progress is recorded here, before the step's own code can run, so a
        // step that started is a fact about the run rather than a claim the
        // template makes about itself.
        let attempt_id = crate::model::identity::new_id::<dokime_domain::model::identity::StepAttemptId>();
        let started_at = crate::model::time::now_utc();
        let start = dokime_domain::model::execution::result::StepAttemptStart {
            attempt_id: attempt_id.clone(),
            step_id: declared.step_id.clone(),
            started_at: started_at.clone(),
        };
        let admitted = tokio::select! {
            biased;
            _ = cancel.wait_for(|cancelled| *cancelled) => return Ended::Cancelled { step: step.to_string() },
            diagnostic = environment.streams().refused() => Err(diagnostic),
            answer = transport.begin_step(step.clone(), start) => answer,
        };
        if let Err(diagnostic) = admitted {
            if let Some(command) = &command {
                controls.stop_accepting();
                controls.complete(&command.id, None, Some(diagnostic.redacted().to_string()));
            }
            if *cancel.borrow() {
                return Ended::Cancelled { step: step.to_string() };
            }
            return Ended::Failed { step: step.to_string(), diagnostic: Box::new(diagnostic) };
        }
        evidence.record_step(step.clone());
        environment.handles().begin(super::StepIdentity {
            step: step.to_string(),
            step_id: declared.step_id.to_string(),
            attempt: attempt_id.to_string(),
        });
        let mut ctx = environment.step(transport).calling_stage(format!("step/{}", declared.step_id));

        let stepped = {
            let work = caught(template.run_step(step.as_str(), &mut ctx)).instrument(tracing::info_span!("step", step = %step));
            tokio::select! {
                biased;
                _ = cancel.wait_for(|cancelled| *cancelled) => None,
                diagnostic = environment.streams().refused() => Some(Err(diagnostic)),
                result = work => Some(result),
            }
        };
        // What the step left running is cancelled before its attempt ends,
        // so a process's exit and a watch's last rows still belong to it.
        drop(ctx);
        for warning in environment.handles().sweep().await {
            transport.emit(crate::records::DeclaredRecord::request(warning));
        }
        // The step's buffered rows reach the host before its attempt ends,
        // whichever way it ended, and a refused flush fails the step.
        environment.streams().flush().await;

        let stepped = match stepped {
            Some(Ok(())) => Some(environment.streams().failure().map_or(Ok(()), Err)),
            other => other,
        };
        let (termination, detail) = match &stepped {
            None => (TerminationReason::Cancelled, None),
            Some(Err(diagnostic)) => (TerminationReason::Failed, Some(diagnostic.redacted())),
            Some(Ok(())) => (TerminationReason::Completed, None),
        };
        let attempt = dokime_domain::model::execution::result::TestStepAttempt {
            attempt_id: attempt_id.clone(),
            step_id: declared.step_id.clone(),
            started_at,
            finished_at: Some(crate::model::time::now_utc()),
            termination,
            detail,
        };
        evidence.attempts.push(attempt.clone());
        transport.finish_step(attempt);
        if let Some(command) = &command {
            let error = match &stepped {
                None => Some("step was cancelled".into()),
                Some(Err(diagnostic)) => Some(diagnostic.redacted().to_string()),
                Some(Ok(())) => None,
            };
            if error.is_some() {
                controls.stop_accepting();
            }
            controls.complete(&command.id, Some(&attempt_id), error);
        }
        match stepped {
            None => {
                return Ended::Cancelled { step: step.to_string() };
            }
            Some(Err(diagnostic)) => {
                return Ended::Failed { step: step.to_string(), diagnostic: Box::new(diagnostic) };
            }
            Some(Ok(())) => {}
        }
    }
}

async fn clean_up<T: SidecarTemplate>(
    template: &mut T, environment: &Environment<'_, T, T::Own, T::Parameters>, transport: &mut dyn ExecutionTransport,
    reason: TerminationReason,
) -> Option<Diagnostic> {
    let mut ctx = environment.cleanup(transport, reason);
    let work = caught(template.cleanup(&mut ctx)).instrument(tracing::info_span!("cleanup"));
    work.await.err()
}

async fn conclude<T: SidecarTemplate>(
    declaration: &TestCaseTemplate, environment: Option<&Environment<'_, T, T::Own, T::Parameters>>, transport: &dyn ExecutionTransport,
    evidence: &ExecutionEvidence, ended: Ended, cleanup_error: Option<Diagnostic>, inputs_unsettled: bool,
) -> SidecarExecutionOutcome {
    let initialized = environment.is_some();
    let selected_attempts = Arc::new(
        dokime_domain::model::execution::result::effective_attempts(declaration, &evidence.attempts, None)
            .into_values()
            .map(|attempt| attempt.attempt_id.clone())
            .collect(),
    );
    let mut criteria = Vec::with_capacity(declaration.criteria.len());
    for criterion in &declaration.criteria {
        let identifier = criterion.metadata.identifier.to_string();
        let label = criterion.metadata.label.clone();
        let decided = if let Some(environment) = environment {
            let ctx = environment.criterion(evidence).selecting(Arc::clone(&selected_attempts)).deciding(transport, &identifier);
            let work = caught(T::evaluate(&identifier, &ctx)).instrument(tracing::info_span!("criterion", criterion = %identifier));
            let result = match tokio::time::timeout(CRITERION_BUDGET, work).await {
                Ok(result) => result.transpose(),
                Err(_) => Some(Err(Diagnostic::of("criterion evaluation exceeded one second"))),
            };
            let callback_error = settle_callbacks(environment, transport).await;

            match (result, callback_error) {
                (Some(Err(error)), _) | (_, Some(error)) => Some(Err(error)),
                (result, None) => result,
            }
        } else {
            None
        };
        let (status, message) = match decided {
            Some(Ok(true)) => (SidecarCriterionStatus::Passed, None),
            Some(Ok(false)) => {
                (SidecarCriterionStatus::Failed, Some(format!("`{label}` did not hold for the evidence this execution produced")))
            }
            // A criterion answered nothing about the run, so the run says so
            // in the criterion's own words. The whole diagnostic travels on
            // the outcome's detail, which carries one failure, not five.
            Some(Err(diagnostic)) => (SidecarCriterionStatus::Indeterminate, Some(diagnostic.message)),
            None => (
                SidecarCriterionStatus::Indeterminate,
                Some(if inputs_unsettled {
                    "execution inputs did not settle; captured measurements are incomplete".into()
                } else if initialized {
                    format!("criterion `{identifier}` is declared with no measurement bound to it")
                } else {
                    "execution did not initialize; no measurements were collected".into()
                }),
            ),
        };

        criteria.push(SidecarCriterionOutcome { identifier, label, status: status as i32, message });
    }

    let passed = criteria.iter().filter(|criterion| criterion.status == SidecarCriterionStatus::Passed as i32).count();
    let template = declaration.metadata.identifier.as_str();
    // One failure carries the detail. A run that both failed a step and failed
    // to clean up is reported by the step: the cleanup failure is a consequence
    // of it, and the step is where the operator has to look.
    let detail = match &ended {
        Ended::Failed { step, diagnostic } => Some(diagnostic.as_ref().clone().context(format!("{} failed in {template}", staged(step)))),
        _ => cleanup_error.clone().map(|diagnostic| diagnostic.context(format!("cleanup failed in {template}"))),
    };
    // The outcome carries one primary failure, but the execution log must tell
    // the whole story even when a failed step is followed by failed cleanup.
    // Use the durable, redacted rendering before this crosses the log wire.
    if let Some(diagnostic) = &detail {
        transport.log(super::log::Level::Error, diagnostic.redacted().to_string());
    }
    if matches!(ended, Ended::Failed { .. })
        && let Some(diagnostic) = &cleanup_error
    {
        transport.log(super::log::Level::Error, diagnostic.clone().context(format!("cleanup failed in {template}")).redacted().to_string());
    }

    let mut message = match &ended {
        Ended::Completed => "execution completed".to_owned(),
        Ended::Cancelled { step } => format!("execution cancelled during phase `{step}`"),
        Ended::Failed { step, diagnostic } => {
            format!("execution failed during `{step}`: {}", diagnostic.message)
        }
    };
    if !criteria.is_empty() {
        message.push_str(&format!("; {passed} of {} criteria passed", criteria.len()));
    }
    if let Some(error) = &cleanup_error {
        message.push_str(&format!("; cleanup failed: {}", error.message));
    }

    SidecarExecutionOutcome {
        passed: matches!(ended, Ended::Completed) && cleanup_error.is_none() && passed == criteria.len(),
        message: Some(message),
        criteria,
        detail_json: detail.as_ref().map(encoded),
        failed_in: match &ended {
            Ended::Failed { step, .. } => Some(step.clone()),
            _ => None,
        },
    }
}

async fn settle_callbacks<P, O, I>(environment: &Environment<'_, P, O, I>, transport: &dyn ExecutionTransport) -> Option<Diagnostic> {
    let failures = environment.callbacks().settle().await;
    for failure in &failures {
        transport.log(super::log::Level::Error, failure.redacted().to_string());
    }

    failures.into_iter().next()
}

fn staged(step: &str) -> String {
    match step {
        PARAMETERS | INITIALIZATION | TRACEABILITY => step.to_owned(),
        step => format!("step `{step}`"),
    }
}

fn encoded(detail: &Diagnostic) -> String {
    serde_json::to_string(detail).expect("a diagnostic is five owned strings and two string vectors")
}

fn report(executions: &Executions, token: &str, outcome: SidecarExecutionOutcome) {
    if let Some(state) = lock(executions).get_mut(token) {
        state.controls.close();
        state.push(BufferedFrame::terminal(outcome));
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::buffer::ExecutionState;

    #[tokio::test(start_paused = true)]
    async fn settlement_live_sample_replay_extends_idle_deadline() {
        let (cancel, mut cancelled) = watch::channel(false);
        let state = ExecutionState::new(Default::default(), cancel);
        let stream = dokime_domain::model::identity::Identifier::new("reading").unwrap();
        let sample =
            crate::telemetry::TelemetrySample::new(crate::model::identity::new_id(), stream, 1, crate::model::time::now_utc(), "fixture");
        let states = std::collections::BTreeMap::from([("fixture".into(), state)]);
        let executions = Arc::new(std::sync::Mutex::new(states));
        let waiting = Arc::clone(&executions);
        let settlement = tokio::spawn(async move { settle_record_inputs(&waiting, "fixture", &mut cancelled).await });
        tokio::task::yield_now().await;

        tokio::time::advance(Duration::from_secs(20)).await;
        assert!(lock(&executions).get_mut("fixture").unwrap().receive(sample.clone()).unwrap());
        tokio::time::advance(Duration::from_millis(10)).await;
        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_secs(20)).await;
        assert!(!lock(&executions).get_mut("fixture").unwrap().receive(sample).unwrap());
        tokio::time::advance(Duration::from_millis(10)).await;
        tokio::task::yield_now().await;
        tokio::time::advance(Duration::from_secs(20)).await;
        tokio::task::yield_now().await;
        assert!(!settlement.is_finished(), "first confirmation keeps a live-delivered sample's settlement active");
        lock(&executions).get_mut("fixture").unwrap().settle_record_subscriptions();
        assert!(settlement.await.unwrap().is_ok());
    }

    #[tokio::test(start_paused = true)]
    async fn repeated_replay_preserves_settlement_deadline() {
        let (cancel, mut cancelled) = watch::channel(false);
        let state = ExecutionState::new(Default::default(), cancel);
        let stream = dokime_domain::model::identity::Identifier::new("reading").unwrap();
        let sample =
            crate::telemetry::TelemetrySample::new(crate::model::identity::new_id(), stream, 1, crate::model::time::now_utc(), "fixture");
        let states = std::collections::BTreeMap::from([("fixture".into(), state)]);
        let executions = Arc::new(std::sync::Mutex::new(states));
        let waiting = Arc::clone(&executions);
        let settlement = tokio::spawn(async move { settle_record_inputs(&waiting, "fixture", &mut cancelled).await });
        tokio::task::yield_now().await;
        assert!(lock(&executions).get_mut("fixture").unwrap().receive(sample.clone()).unwrap());
        assert!(!lock(&executions).get_mut("fixture").unwrap().receive(sample.clone()).unwrap());
        tokio::time::advance(Duration::from_millis(10)).await;
        tokio::task::yield_now().await;

        for _ in 0..3 {
            tokio::time::advance(Duration::from_secs(10)).await;
            assert!(!lock(&executions).get_mut("fixture").unwrap().receive(sample.clone()).unwrap());
            tokio::time::advance(Duration::from_millis(10)).await;
            tokio::task::yield_now().await;
            assert!(!settlement.is_finished());
        }
        tokio::time::advance(Duration::from_secs(5)).await;
        tokio::task::yield_now().await;
        assert!(settlement.is_finished());
        assert!(settlement.await.unwrap().is_err(), "duplicate replay cannot extend the 35-second idle bound");
    }
}
