/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{Arc, Weak, atomic::AtomicU32};

// Third Party Imports
use prost::Message;
use tokio::sync::oneshot::{self, Receiver};
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;

// Local Imports
use crate::{
    Error, Result,
    dispatch::Dispatch,
    execution_manager::{Active, ExecutionManager},
    host::lock,
    run_control::RunControl,
    streams::{ExecutionTime, SampleScope},
};
use dokime_operations::Resources;
use dokime_protocol::host::execution_outcome::Attribution;
use dokime_protocol::host::executions::ExecutionCommandPhase;
use dokime_protocol::host::{EvaluatedOutcome, ExecutionMode, ExecutionOutcome, ResultStatus, UnevaluatedOutcome};
use dokime_protocol::model::origin::Kind as OriginKind;
use dokime_protocol::model::{Origin, Suites, Termination};
use dokime_protocol::plugin::{
    CriteriaEvaluateRequest, CriteriaEvaluateResponse, ExecutionsCleanupRequest, ExecutionsCleanupResponse, ExecutionsInitializeRequest,
    ExecutionsInitializeResponse, StepsExecuteRequest, StepsExecuteResponse,
};
use dokime_storage::{Execution, ExecutionSnapshot, OutcomeCounts};
use harmos_protocol::host::PluginEntry;
use harmos_runtime::plugin::grpc::{Error as BindingError, ScopedBinding};
use harmos_runtime::streams::StreamPublicationError;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Cycle {
    pub identifier: String,
    pub executions: Vec<String>,
    store: dokime_storage::Store,
    result: Receiver<Result<Completed>>,
}
/// What a Finished Cycle Task Reports; Evidence Is Read Only When a Caller Waits for It.
struct Completed {
    executions: Vec<String>,
    states: Vec<RunState>,
    counts: OutcomeCounts,
}
#[derive(Debug)]
pub struct CycleResult {
    pub executions: Vec<ExecutionSnapshot>,
    pub states: Vec<RunState>,
    pub counts: OutcomeCounts,
}
struct ActiveRun {
    binding: Option<ScopedBinding>,
    release: Option<Release>,
}
// Releases the Run's Scope, Failures and Prompts When Dropped.
struct Release {
    host: ExecutionManager,
    execution: String,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunState {
    Queued,
    Initializing,
    Stepping,
    Evaluating,
    CleaningUp,
    Finished,
    Cancelled,
    Interrupted,
}

/* === Traits === */

/* === Implementations === */

impl Cycle {
    /// Await the Cycle and Read Each Execution's Sealed Evidence, Including Every Sample.
    ///
    /// The run itself never materializes its samples; only this caller-requested read does.
    pub async fn wait(self) -> Result<CycleResult> {
        let completed = self.result.await.map_err(|_| Error::Invalid("Wait for Cycle: Inspect the Failed Execution Task".into()))??;
        let executions = completed.executions.iter().map(|execution| Ok(self.store.read(execution)?)).collect::<Result<Vec<_>>>()?;
        Ok(CycleResult {
            executions,
            states: completed.states,
            counts: completed.counts,
        })
    }
}
impl ActiveRun {
    // Stop Scoped Calls and Await Accepted Ingestion; the Scope Stays Until Drop.
    async fn close(&mut self) -> Result<()> {
        let Some(binding) = self.binding.take() else { return Ok(()) };
        binding.close().await.map_err(|source| settlement("Close Execution Binding", &source))
    }
}
impl Drop for ActiveRun {
    fn drop(&mut self) {
        // A Panic or Dropped Run Hands Its Binding to a Host Task That Drains Before Release.
        // A Task That Never Runs Still Drops Both, Which Signals Close and Releases.
        let (Some(binding), Some(release)) = (self.binding.take(), self.release.take()) else { return };
        let host = release.host.clone();
        let _ = host.spawn(async move {
            if let Err(source) = binding.close().await {
                let source = settlement("Close Abandoned Execution Binding", &source);
                let execution = &release.execution;
                tracing::error!(%execution, %source, "Close Abandoned Execution Binding; Inspect the Interrupted Evidence");
            }
            drop(release);
        });
    }
}
impl Drop for Release {
    fn drop(&mut self) {
        let (host, execution) = (&self.host, self.execution.as_str());
        if let Ok(mut active) = host.inner.active.lock() {
            active.remove(execution);
        }
        if let Ok(mut scopes) = host.inner.sample_scopes.lock() {
            scopes.remove(execution);
        }
        if let Ok(mut failures) = host.inner.sample_failures.lock() {
            failures.remove(execution);
        }
        host.inner.prompts.clear(execution);
        host.signal_change();
    }
}

/* === Public Functions === */

pub(crate) async fn start(host: ExecutionManager, origins: Vec<Origin>) -> Result<Cycle> {
    if origins.is_empty() {
        return Err(Error::Invalid("Materialize at Least One Test Case Before Starting a Cycle".into()));
    }
    let identifier = uuid::Uuid::now_v7().to_string();
    let mut prepared = Vec::new();
    for mut origin in origins {
        let Some(OriginKind::PluginRun(mut run)) = origin.kind.take() else {
            return Err(Error::Invalid("Start Cycles with PluginRun Origins; Import Evidence Separately".into()));
        };
        run.validate().map_err(Error::Invalid)?;
        let entry = host.entry(&run.plugin).await?;
        let digest = &entry
            .description
            .as_ref()
            .unwrap()
            .declarations
            .as_ref()
            .ok_or_else(|| Error::Invalid("Reconnect the Plugin Catalog".into()))?
            .digest;
        if &run.digest != digest || run.started_at != entry.description.as_ref().unwrap().started_at {
            origin = host.materialize(&run.plugin, &run.template, run.test_case.take().unwrap().parameters).await?;
        } else {
            origin.kind = Some(OriginKind::PluginRun(run));
        }
        prepared.push((uuid::Uuid::now_v7().to_string(), origin, entry));
    }
    let admitted: Vec<_> = prepared
        .iter()
        .map(|(execution, origin, entry)| Execution {
            identifier: execution.clone(),
            cycle: identifier.clone(),
            origin: Some(origin.clone()),
            catalog: entry.description.as_ref().unwrap().declarations.as_ref().unwrap().encode_to_vec(),
            ..Default::default()
        })
        .collect();
    host.register_cycle(&identifier)?;
    host.inner.store.create_executions(&admitted)?;
    let executions = admitted.iter().map(|execution| execution.identifier.clone()).collect();
    {
        let mut active = lock(&host.inner.active)?;
        for (execution, _, entry) in &prepared {
            active.insert(
                execution.clone(),
                Active {
                    plugin: entry.name.clone(),
                    cancellation: CancellationToken::new(),
                    control: Arc::new(RunControl::default()),
                },
            );
        }
    }
    launch(host, identifier, executions, prepared)
}

pub(crate) async fn start_existing_mode(host: ExecutionManager, execution: &str, mode: ExecutionMode) -> Result<Cycle> {
    let cut = host.inner.store.cut(execution)?;
    if cut.state != dokime_storage::ExecutionState::Open || cut.frontier != 0 {
        return Err(Error::Invalid("Start Execution: Select Unstarted Open PluginRun Evidence; Rerun Sealed Evidence Separately".into()));
    }
    let origin = cut.execution.origin.ok_or_else(|| Error::Invalid("Start Execution: Supply a PluginRun Origin".into()))?;
    let Some(OriginKind::PluginRun(run)) = origin.kind.as_ref() else {
        return Err(Error::Invalid("Start Execution: Imports Cannot Enter the Run Queue".into()));
    };
    run.validate().map_err(Error::Invalid)?;
    let entry = host.entry(&run.plugin).await?;
    if entry.description.as_ref().unwrap().declarations.as_ref().is_none_or(|catalog| catalog.digest != run.digest)
        || entry.description.as_ref().unwrap().started_at != run.started_at
    {
        return Err(Error::Invalid("Start Execution: Rematerialize the Changed Plugin Catalog into New Execution Evidence".into()));
    }
    host.register_cycle(&cut.execution.cycle)?;
    {
        let mut active = lock(&host.inner.active)?;
        if active.contains_key(execution) {
            return Err(Error::Invalid("Start Execution: The Execution Already Has an Active Run".into()));
        }
        active.insert(
            execution.into(),
            Active {
                plugin: run.plugin.clone(),
                cancellation: CancellationToken::new(),
                control: Arc::new(if mode == ExecutionMode::Interactive { RunControl::interactive() } else { RunControl::default() }),
            },
        );
    }
    launch(host, cut.execution.cycle, vec![execution.into()], vec![(execution.into(), origin, entry)])
}

/* === Private Functions === */

/// Name the Binding Action That Failed and What Resolves It.
fn settlement(action: &str, source: &BindingError) -> Error {
    match source {
        BindingError::Deadline { .. } => Error::Invalid(format!(
            "{action}: The Plugin Did Not Settle Its Samples Within stop_grace; Rebuild It on an SDK That Ends Samples When the Host \
             Closes Demand, Then Rerun the Execution"
        )),
        source => Error::Invalid(format!("{action}: {source}")),
    }
}

/// Name the Execution's Streams and the Action When Their Samples Could Not Settle.
fn samples_settlement(action: &str, source: &BindingError, streams: &[String]) -> Error {
    if matches!(source, BindingError::Deadline { .. }) {
        return settlement(action, source);
    }
    // The Library Wraps a Failed Window Answer; Its Causes Say Why the Window Failed.
    let mut causes = vec![source.to_string()];
    let mut cause = std::error::Error::source(source);
    while let Some(next) = cause {
        causes.push(next.to_string());
        cause = next.source();
    }
    let streams = streams.join(", ");
    if causes.iter().any(|cause| cause.contains(&StreamPublicationError::Capacity.to_string())) {
        return Error::Invalid(format!(
            "{action}: Streams {streams} Answered a Window Whose Rows Did Not Fit the Shared Plugin Publication Pool; Shorten Their \
             Windows (the Observer's Window Setting or interval_window_ms), Lower Their Rate or Stream Count, Then Rerun the Execution"
        ));
    }
    Error::Invalid(format!("{action}: Streams {streams}: {}; Rerun the Execution", causes.join(": ")))
}

fn launch(
    host: ExecutionManager, identifier: String, executions: Vec<String>, prepared: Vec<(String, Origin, PluginEntry)>,
) -> Result<Cycle> {
    host.track(&executions)?;
    let (send, result) = oneshot::channel();
    let store = host.inner.store.clone();
    let owner = host.clone();
    let observed = executions.clone();
    owner.spawn(async move {
        let result = async {
            let mut workers = JoinSet::new();
            for (index, (execution, origin, entry)) in prepared.into_iter().enumerate() {
                let host = host.clone();
                workers.spawn(async move {
                    let result = drive(host.clone(), execution.clone(), origin, entry).await;
                    if result.is_err() {
                        host.settled(&execution).await?;
                        let cut = host.inner.store.cut(&execution)?;
                        if cut.state == dokime_storage::ExecutionState::Open {
                            host.inner.store.close_streams(&execution)?;
                            host.inner.store.close(
                                &execution,
                                Termination::Interrupted,
                                ExecutionOutcome {
                                    attribution: Some(Attribution::NotEvaluated(UnevaluatedOutcome {
                                        termination: Termination::Interrupted as i32,
                                        reason: "Execution Infrastructure Failed; Inspect the Host Diagnostic".into(),
                                    })),
                                },
                            )?;
                            host.inner.store.seal(&execution)?;
                        }
                    }
                    host.finish_progress(&execution, result.as_ref().err().map(ToString::to_string))?;
                    Ok::<_, Error>((index, result))
                });
            }
            let mut finished = Vec::new();
            let mut failure = None;
            while let Some(result) = workers.join_next().await {
                match result {
                    Ok(Ok((index, result))) => match result {
                        Ok(result) => finished.push((index, result)),
                        Err(error) => {
                            failure.get_or_insert(error);
                        }
                    },
                    Ok(Err(error)) => {
                        failure.get_or_insert(error);
                    }
                    Err(error) => {
                        failure.get_or_insert(Error::Join(error));
                    }
                }
            }
            if let Some(error) = failure {
                return Err(error);
            }
            finished.sort_by_key(|(index, _)| *index);
            let mut executions = Vec::new();
            let mut outcomes = Vec::new();
            let mut states = Vec::new();
            for (_, (execution, execution_states)) in finished {
                outcomes.extend(host.inner.store.cut(&execution)?.execution.outcome);
                executions.push(execution);
                states.extend(execution_states);
            }
            let counts = OutcomeCounts::from_outcomes(outcomes.iter());
            Ok(Completed { executions, states, counts })
        }
        .await;
        // Settle Every Admitted Execution Even If a Worker Failed to Join
        let mut result = result;
        for execution in observed {
            // A Failed Worker's Binding Drains Before Its Streams Close.
            if let Err(error) = host.settled(&execution).await {
                result = Err(error);
            }
            let settled = (|| -> Result<()> {
                let cut = host.inner.store.cut(&execution)?;
                if cut.state == dokime_storage::ExecutionState::Open {
                    host.inner.store.close_streams(&execution)?;
                    host.inner.store.close(
                        &execution,
                        Termination::Interrupted,
                        ExecutionOutcome {
                            attribution: Some(Attribution::NotEvaluated(UnevaluatedOutcome {
                                termination: Termination::Interrupted as i32,
                                reason: "Execution Worker Failed; Inspect the Host Diagnostic".into(),
                            })),
                        },
                    )?;
                }
                if host.inner.store.cut(&execution)?.state == dokime_storage::ExecutionState::Closed {
                    host.inner.store.seal(&execution)?;
                }
                Ok(())
            })();
            if let Err(error) = settled {
                result = Err(error);
            }
            if let Err(error) = host.finish_progress(&execution, result.as_ref().err().map(ToString::to_string)) {
                result = Err(error);
            }
        }
        let _ = send.send(result);
    })?;
    Ok(Cycle { identifier, executions, store, result })
}

async fn drive(host: ExecutionManager, execution: String, origin: Origin, entry: PluginEntry) -> Result<(String, Vec<RunState>)> {
    let mut states = Vec::new();
    let mut active = ActiveRun {
        binding: None,
        release: Some(Release {
            host: host.clone(),
            execution: execution.clone(),
        }),
    };
    let cancellation = lock(&host.inner.active)?.get(&execution).unwrap().cancellation.clone();
    let control = lock(&host.inner.active)?.get(&execution).unwrap().control.clone();
    tracing::info!(execution = %execution, "Queued Execution");
    states.push(RunState::Queued);
    let Some(OriginKind::PluginRun(run)) = origin.kind.as_ref() else { unreachable!() };
    let case = run.test_case.as_ref().unwrap();
    let description = entry.description.as_ref().ok_or_else(|| Error::Invalid("Start Execution: Discover the Plugin".into()))?;
    let catalog = description.declarations.as_ref().ok_or_else(|| Error::Invalid("Start Execution: Discover Its Catalog".into()))?;
    let suites = catalog
        .sections
        .iter()
        .find(|section| section.r#type == "dokime.model.v1.Suites")
        .ok_or_else(|| Error::Invalid("Start Execution: Declare Its Suites".into()))?;
    let suites = Suites::decode(suites.payload.as_slice())?;
    let template = suites
        .suites
        .iter()
        .flat_map(|suite| &suite.templates)
        .find(|template| template.metadata.as_ref().is_some_and(|metadata| metadata.identifier == run.template))
        .ok_or_else(|| Error::Invalid("Start Execution: Select Its Declared Template".into()))?;
    let runtime =
        host.inner.runtime.get().and_then(Weak::upgrade).ok_or_else(|| Error::Invalid("Start Execution: Assemble the Runtime".into()))?;
    let plugins = host.inner.library_plugins.get().ok_or_else(|| Error::Invalid("Start Execution: Assemble Plugins".into()))?;
    let activation = plugins.activation(&entry.name).await.map_err(|source| Error::ComposedStartup(Box::new(source)))?;
    let (time, controller) = ExecutionTime::open(host.inner.settings.clock.as_ref());
    for stream in &case.streams {
        host.inner.store.open_stream(&execution, stream)?;
    }
    let scope = SampleScope::new(catalog, &case.streams)?;
    lock(&host.inner.sample_scopes)?.insert(execution.clone(), scope);
    let binding =
        runtime.bind_grpc(&activation, &execution, time.clock.clone()).await.map_err(|source| Error::ComposedStartup(Box::new(source)))?;
    // Close the Binding on Every Exit So Accepted Samples Drain Before the Scope Is Removed.
    let binding = &*active.binding.insert(binding);
    let prepared: Result<(Resources, Termination, Vec<bool>, Result<()>)> = async {
        let resources = Resources {
            plugin: Arc::new(entry.clone()),
            context: host
                .inner
                .context
                .get()
                .and_then(Weak::upgrade)
                .ok_or_else(|| Error::Invalid("Start Execution: Assemble the Host Context Before Running".into()))?,
            store: host.inner.store.clone(),
            execution: execution.clone(),
            step: String::new(),
            operation: String::new(),
            attempt: Arc::new(AtomicU32::new(0)),
            clock: time.clone(),
            prompts: host.inner.prompts.clone(),
        };
        let mut dispatch = Dispatch {
            time: time.clone(),
            sample_failures: host.inner.sample_failures.clone(),
            binding,
            resources,
            cancellation,
            controller: controller.map(Arc::new),
        };
        states.push(RunState::Initializing);
        host.set_run_phase(&execution, "initializing")?;
        let mut initialized = template.initialize.is_none();
        let mut sampling = false;
        let mut evaluated = Vec::new();
        let result: Result<()> = async {
            if dispatch.cancellation.is_cancelled() {
                return Err(Error::Cancelled);
            }
            if let Some(operation) = &template.initialize {
                dispatch
                    .invoke::<_, ExecutionsInitializeResponse>(
                        operation,
                        ExecutionsInitializeRequest {
                            execution: execution.clone(),
                            test_case: Some(case.clone()),
                            origin: Some(origin.clone()),
                        },
                    )
                    .await?;
                initialized = true;
            }
            // The Plugin Has Registered the Unit, So the Host May Now Grant Its Stream Windows.
            binding.start_samples().await.map_err(|source| samples_settlement("Start Execution Samples", &source, &case.streams))?;
            sampling = true;
            states.push(RunState::Stepping);
            host.set_run_phase(&execution, "stepping")?;
            while let Some((index, mut command)) = control.next(case, &dispatch.cancellation).await? {
                let step = &case.steps[index];
                host.signal_change();
                let identifier =
                    step.metadata.as_ref().ok_or_else(|| Error::Invalid("Materialize Steps with Identifiers".into()))?.identifier.clone();
                dispatch.resources.step = identifier.clone();
                let position = dispatch.resources.evidence("step:start", Vec::new(), None)?;
                if let Some(command) = &mut command {
                    command.phase = ExecutionCommandPhase::Running as i32;
                    command.attempt_id = Some(format!("{execution}/{position}"));
                    let now = host.now_ns();
                    command.updated_at = Some(prost_types::Timestamp {
                        seconds: now.div_euclid(1_000_000_000),
                        nanos: now.rem_euclid(1_000_000_000) as i32,
                    });
                    host.inner.store.save_command(command)?;
                }
                host.signal_change();
                let result = dispatch
                    .invoke::<_, StepsExecuteResponse>(
                        &step.operation,
                        StepsExecuteRequest {
                            execution: execution.clone(),
                            step: identifier,
                            test_case: Some(case.clone()),
                            origin: Some(origin.clone()),
                        },
                    )
                    .await;
                dispatch.resources.evidence("step:end", Vec::new(), None)?;
                if result.is_err() {
                    dispatch.resources.evidence("step:failure", Vec::new(), None)?;
                }
                if let Some(mut command) = command {
                    command.phase = if result.is_ok() { ExecutionCommandPhase::Completed } else { ExecutionCommandPhase::Failed } as i32;
                    command.detail = result.as_ref().err().map(ToString::to_string);
                    let now = host.now_ns();
                    command.updated_at = Some(prost_types::Timestamp {
                        seconds: now.div_euclid(1_000_000_000),
                        nanos: now.rem_euclid(1_000_000_000) as i32,
                    });
                    host.inner.store.save_command(&command)?;
                }
                control.complete_step(index)?;
                host.signal_change();
                result?;
            }
            dispatch.resources.step.clear();
            // Criteria Read Settled Streams: Every Granted Window Is Committed Before Evaluation.
            sampling = false;
            binding.stop_samples().await.map_err(|source| samples_settlement("Stop Execution Samples", &source, &case.streams))?;
            states.push(RunState::Evaluating);
            host.set_run_phase(&execution, "evaluating")?;
            for criterion in &case.criteria {
                let identifier = criterion
                    .metadata
                    .as_ref()
                    .ok_or_else(|| Error::Invalid("Materialize Criteria with Identifiers".into()))?
                    .identifier
                    .clone();
                let response: CriteriaEvaluateResponse = dispatch
                    .invoke(
                        &criterion.operation,
                        CriteriaEvaluateRequest {
                            execution: execution.clone(),
                            criterion: identifier.clone(),
                            test_case: Some(case.clone()),
                            origin: Some(origin.clone()),
                        },
                    )
                    .await?;
                dispatch.resources.evidence(&format!("criterion:{identifier}"), response.encode_to_vec(), None)?;
                evaluated.push(response.passed);
            }
            Ok(())
        }
        .await;
        if let Some(mut command) = control.take_pending()? {
            command.phase = if result.is_ok() { ExecutionCommandPhase::Completed } else { ExecutionCommandPhase::Interrupted } as i32;
            host.inner.store.save_command(&command)?;
        }
        let mut termination = match &result {
            Ok(()) => Termination::Completed,
            Err(Error::Cancelled) => Termination::Cancelled,
            Err(Error::Plugin(_)) => Termination::Failed,
            Err(_) => Termination::Interrupted,
        };
        states.push(RunState::CleaningUp);
        host.set_run_phase(&execution, "cleaning_up")?;
        dispatch.cancellation = CancellationToken::new();
        dispatch.resources.step.clear();
        // A Run That Stopped Early Still Ends Its Demand Before Cleanup Ends the Unit.
        if sampling && let Err(source) = binding.stop_samples().await {
            termination = Termination::Interrupted;
            let error = samples_settlement("Stop Execution Samples", &source, &case.streams);
            dispatch.resources.evidence("streams:failure", error.to_string().into_bytes(), None)?;
        }
        if initialized && let Some(operation) = &template.cleanup {
            let cleanup = dispatch
                .invoke::<_, ExecutionsCleanupResponse>(
                    operation,
                    ExecutionsCleanupRequest {
                        execution: execution.clone(),
                        termination: termination as i32,
                        test_case: Some(case.clone()),
                        origin: Some(origin.clone()),
                    },
                )
                .await;
            if let Err(error) = cleanup {
                termination = Termination::Interrupted;
                dispatch.resources.evidence("cleanup:failure", error.to_string().into_bytes(), None)?;
            }
        }
        if let Err(error) = &result {
            dispatch.resources.evidence("execution:failure", error.to_string().into_bytes(), None)?;
        }
        Ok((dispatch.resources, termination, evaluated, result))
    }
    .await;
    let closed = active.close().await;
    host.settle_captures(&execution).await;
    // Keep the Run's Failure First and Attach a Failed Close.
    let (resources, mut termination, evaluated, result) = match prepared {
        Ok(prepared) => prepared,
        Err(error) => {
            return Err(match closed {
                Ok(()) => error,
                Err(close) => Error::Unsettled {
                    run: Box::new(error),
                    close: Box::new(close),
                },
            });
        }
    };
    // A Stream Failure Found After the Last Invocation Still Names Its Stream and Action in the Outcome.
    let mut interruption = None;
    if let Err(error) = closed {
        termination = Termination::Interrupted;
        resources.evidence("streams:failure", error.to_string().into_bytes(), None)?;
        interruption = Some(error.to_string());
    }
    if let Some(source) = lock(&host.inner.sample_failures)?.remove(&execution) {
        termination = Termination::Interrupted;
        resources.evidence("streams:failure", source.clone().into_bytes(), None)?;
        interruption.get_or_insert(source);
    }
    host.inner.store.close_streams(&execution)?;
    let attribution = if termination == Termination::Failed {
        Attribution::Evaluated(EvaluatedOutcome {
            status: ResultStatus::Failed as i32,
            reason: result.as_ref().err().map_or_else(String::new, ToString::to_string),
        })
    } else if termination == Termination::Completed && !evaluated.is_empty() {
        Attribution::Evaluated(EvaluatedOutcome {
            status: if evaluated.iter().all(|passed| *passed) { ResultStatus::Passed } else { ResultStatus::Failed } as i32,
            reason: String::new(),
        })
    } else {
        Attribution::NotEvaluated(UnevaluatedOutcome {
            termination: termination as i32,
            reason: result.as_ref().err().map(ToString::to_string).or(interruption).unwrap_or_else(|| "No Evaluated Criteria".into()),
        })
    };
    host.inner.store.close(&execution, termination, ExecutionOutcome { attribution: Some(attribution) })?;
    host.inner.store.seal(&execution)?;
    states.push(match termination {
        Termination::Cancelled => RunState::Cancelled,
        Termination::Interrupted => RunState::Interrupted,
        _ => RunState::Finished,
    });
    tracing::info!(execution = %execution, ?termination, "Sealed Execution");
    Ok((execution, states))
}

/* === Tests === */
