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
use std::future::Future;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, PoisonError};
// Third Party Imports
// Third-Party Imports
use harmos::sidecar::Refusal;
use tokio::sync::watch;
// Local Imports
use super::template::SidecarTemplate;
use dokime_domain::model::values::Value;
use crate::definition::TestCaseTemplate;
use crate::prompts::PromptResponse;
use crate::transport::buffer::{ExecutionState, Executions, Tasks, lock};
use crate::transport::{
    CommandExecutionReply, CommandExecutionRequest, EvidenceBatch, RunExecutionRequest,
    StepAdmission,
};

/* === Constants === */
const MAX_RETAINED_EXECUTIONS: usize = 32;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Workflow<T: SidecarTemplate> {
    workflow: String,
    pub(super) source: String,
    declaration: TestCaseTemplate,
    records: Vec<crate::records::RecordContract>,
    streams: Vec<crate::telemetry::TelemetryContract>,
    pub(super) executions: Executions,
    runs: Tasks,
    observers: Tasks,
    template: PhantomData<fn() -> T>,
    publisher: Option<super::template::SamplePublisher>,
    watched: Vec<crate::execution::DeclaredObserver>,
    binding: Option<Arc<dyn std::any::Any + Send + Sync>>,
}

#[derive(Clone)]
pub enum SampleOutput {
    Host(harmos::sidecar::Supervisor),
    Embedded(Arc<dyn EmbeddedPublisher>),
}

/* === Enums === */

/* === Traits === */
pub trait EmbeddedPublisher: Send + Sync {
    fn publish(
        &self,
        execution: &str,
        stream: &dokime_domain::model::identity::Identifier,
        values: BTreeMap<String, Value>,
        storage_payload: Vec<u8>,
    ) -> Result<(), String>;
}

/* === Implementations === */
impl<T: SidecarTemplate> Workflow<T> {
    pub fn new(workflow: impl Into<String>, source: impl Into<String>) -> Self {
        Self::declaring(workflow, source, T::definition(), T::declared_observers())
    }

    pub fn declaring(
        workflow: impl Into<String>,
        source: impl Into<String>,
        definition: crate::execution::TemplateDefinition,
        watched: Vec<crate::execution::DeclaredObserver>,
    ) -> Self {
        Self {
            workflow: workflow.into(),
            source: source.into(),
            declaration: definition.template,
            records: definition.records,
            streams: definition.streams,
            executions: Arc::new(Mutex::new(BTreeMap::new())),
            runs: Arc::new(Mutex::new(Vec::new())),
            observers: Arc::new(Mutex::new(Vec::new())),
            template: PhantomData,
            publisher: None,
            watched,
            binding: None,
        }
    }

    pub fn with_publisher(mut self, publisher: super::template::SamplePublisher) -> Self {
        self.publisher = Some(publisher);
        self
    }

    #[must_use]
    pub fn bound(mut self, binding: Arc<dyn std::any::Any + Send + Sync>) -> Self {
        self.binding = Some(binding);

        self
    }

    pub fn with_records(mut self, records: &[crate::records::RecordContract]) -> Self {
        self.records = records
            .iter()
            .filter(|record| {
                record
                    .template_id
                    .as_ref()
                    .is_none_or(|template| template == &self.declaration.test_case_template_id)
            })
            .cloned()
            .collect();
        self
    }

    pub fn start(&self, request: RunExecutionRequest) -> Result<(), Refusal> {
        self.start_on(request, None, super::Caller::default())
    }

    fn accepts(&self, request: &RunExecutionRequest) -> Result<(), Refusal> {
        if request.workflow != self.workflow {
            return Err(Refusal::of(format!(
                "unsupported sidecar execution workflow {:?}",
                request.workflow
            )));
        }
        if request.template_identifier != self.declaration.metadata.identifier.as_str() {
            return Err(Refusal::of(format!(
                "unsupported sidecar execution template {:?}",
                request.template_identifier
            )));
        }

        Ok(())
    }
}

impl<T: SidecarTemplate> Workflow<T> {
    pub fn workflow(&self) -> &str {
        &self.workflow
    }

    pub fn template_identifier(&self) -> &str {
        self.declaration.metadata.identifier.as_str()
    }

    pub fn owns(&self, token: &str) -> bool {
        lock(&self.executions).contains_key(token)
    }

    pub fn take_local_samples(
        &self,
        token: &str,
    ) -> Result<Vec<crate::telemetry::TelemetrySample>, Refusal> {
        lock(&self.executions)
            .get_mut(token)
            .map(|state| state.take_local_samples())
            .ok_or_else(|| Refusal::of(format!("execution token {token:?} is not active")))
    }

    pub fn start_on(
        &self,
        request: RunExecutionRequest,
        host: Option<SampleOutput>,
        caller: super::Caller,
    ) -> Result<(), Refusal> {
        if request.execution_token.trim().is_empty() {
            return Err(Refusal::of("execution token must not be empty"));
        }
        {
            let executions = lock(&self.executions);
            if let Some(existing) = executions.get(&request.execution_token) {
                return if existing.request == request {
                    Ok(())
                } else {
                    Err(reused(&request.execution_token))
                };
            }
        }
        self.accepts(&request)?;
        if !matches!(request.mode.as_str(), "automatic" | "interactive") {
            return Err(Refusal::of(
                "execution mode must be automatic or interactive",
            ));
        }
        let parameters = resolved(&request.parameters_json)?;
        let secrets = serde_json::from_str(&request.secrets_json)
            .map_err(|_| Refusal::of("invalid execution secret bindings"))?;
        let bindings = crate::plugin::Configuration {
            secrets,
            ..Default::default()
        };
        let mut runs = self.runs.lock().unwrap_or_else(PoisonError::into_inner);
        runs.retain(|run| !run.is_finished());

        let token = request.execution_token.clone();
        let (cancel, cancelled) = watch::channel(false);
        let (callbacks, controls) = {
            let mut executions = lock(&self.executions);
            if let Some(existing) = executions.get(&token) {
                if existing.request == request {
                    return Ok(());
                }
                return Err(reused(&token));
            }
            retain_capacity(&mut executions)?;
            let mut state = ExecutionState::new(request, cancel);
            state.local_inputs = host.is_none();
            state.subscribe(&self.declaration.subscriptions);
            let callbacks = state.callbacks.clone();
            let controls = Arc::clone(&state.controls);
            executions.insert(token.clone(), state);

            (callbacks, controls)
        };

        runs.push(tokio::spawn(super::run::run::<T>(
            Arc::clone(&self.executions),
            token,
            self.source.clone(),
            self.declaration.clone(),
            super::run::Inputs {
                controls,
                callbacks,
                parameters,
                bindings,
                caller,
                binding: self.binding.clone(),
                observers: self.watched.clone(),
                records: self.records.clone(),
                streams: self.streams.clone(),
                host,
                publisher: self.publisher,
            },
            cancelled,
            Arc::clone(&self.observers),
        )));
        Ok(())
    }

    pub fn command(
        &self,
        request: CommandExecutionRequest,
    ) -> Result<CommandExecutionReply, Refusal> {
        let command = crate::transport::control::Command::parse(&request)?;
        let controls = lock(&self.executions)
            .get(&request.execution_token)
            .map(|state| Arc::clone(&state.controls))
            .ok_or_else(|| Refusal::of("execution token is not active"))?;
        controls.admit(command)
    }

    pub fn drain(&self, token: &str, cursor: u64) -> Result<EvidenceBatch, Refusal> {
        let mut executions = lock(&self.executions);
        let state = executions.get_mut(token).ok_or_else(|| {
            Refusal::of(format!(
                "execution token {token:?} is not active in this plugin"
            ))
        })?;
        let reply = state.drain_after(cursor)?;
        state.admit_local_inputs(
            reply.cursor,
            &self.source,
            &self.records,
            &crate::model::time::now_utc(),
        )?;
        if state.local_inputs && reply.record_subscriptions_settling {
            state.settle_record_subscriptions();
        }
        Ok(reply)
    }

    /// Wait for the next output page, input-settlement request or completed cleanup.
    ///
    /// This is the waitable form of `drain`, for the execution's single output
    /// consumer. Registering before the cursor read preserves changes arriving
    /// between the read and suspension. Dropping the future stops waiting.
    pub async fn next(&self, token: &str, cursor: u64) -> Result<EvidenceBatch, Refusal> {
        let changed = lock(&self.executions)
            .get(token)
            .map(|state| Arc::clone(&state.changed))
            .ok_or_else(|| Refusal::of("execution token is not active in this plugin"))?;
        loop {
            let notified = changed.notified();
            tokio::pin!(notified);
            notified.as_mut().enable();
            let batch = self.drain(token, cursor)?;
            if batch.cursor > cursor || batch.record_subscriptions_settling || batch.cleanup_settled {
                return Ok(batch);
            }
            notified.await;
        }
    }

    pub fn cancel(&self, token: &str) {
        let callbacks = {
            let mut executions = lock(&self.executions);
            executions.get_mut(token).map(|state| {
                state.cancel();
                state.callbacks.clone()
            })
        };
        if let Some(callbacks) = callbacks {
            callbacks.cancel();
        }
    }

    pub fn answer(&self, token: &str, prompt_id: &str, response_json: &str) -> Result<(), Refusal> {
        let response: PromptResponse = serde_json::from_str(response_json)
            .map_err(|error| Refusal::of(format!("invalid operator answer JSON: {error}")))?;
        if let Some(state) = lock(&self.executions).get_mut(token) {
            state.answer(prompt_id, response);
        }

        Ok(())
    }

    pub fn deliver_captured(
        &self,
        record: &crate::records::Record,
        owner: &str,
        scopes: &[dokime_domain::model::execution::RecordScope],
    ) -> usize {
        let mut taken = 0;
        for state in lock(&self.executions).values_mut() {
            if let Some(scope) = scopes.iter().find(|scope| {
                scope
                    .test_execution_id
                    .as_ref()
                    .is_some_and(|id| id.as_str() == state.request.test_execution_id)
            }) && state.deliver_captured(record, owner, Some(scope))
            {
                taken += 1;
            }
        }
        taken
    }

    pub fn deliver_samples(
        &self,
        scope: &dokime_domain::model::execution::RecordScope,
        samples: &[crate::telemetry::TelemetrySample],
        gaps: &[crate::telemetry::LeaseGap],
    ) -> Result<usize, Refusal> {
        let mut taken = 0;
        let mut deliveries = Vec::new();
        let mut executions = lock(&self.executions);
        for state in executions
            .values()
            .filter(|state| state.addressed_by(Some(scope)))
        {
            for sample in samples {
                state.validate_sample(sample).map_err(Refusal::of)?;
            }
        }
        for state in executions.values_mut() {
            if !state.addressed_by(Some(scope)) {
                continue;
            }
            let mut took = false;
            for sample in samples {
                if state.receive(sample.clone()).map_err(Refusal::of)? {
                    took = true;
                    if state.listening.receiver_count() > 0 {
                        deliveries.push((
                            state.listening.clone(),
                            sample.stream.clone(),
                            sample.values.clone(),
                        ));
                    }
                }
            }
            for gap in gaps {
                state.interrupted(gap.clone());
                took = true;
            }
            if took {
                taken += 1;
            }
        }
        drop(executions);
        // Deliver only to each addressed execution's existing subscription bus,
        // after releasing the state lock. Never broadcast across execution scopes.
        for (listening, stream, values) in deliveries {
            let _ = listening.send((stream, values));
        }

        Ok(taken)
    }

    pub fn admit(&self, token: &str, admission: &StepAdmission) {
        if let Some(state) = lock(&self.executions).get_mut(token) {
            state.acknowledge_step(admission);
        }
    }

    pub fn resolve_lease(&self, token: &str, lease_token: &str, granted: bool, message: &str) {
        if let Some(state) = lock(&self.executions).get_mut(token) {
            state.resolve_lease(lease_token, granted, message);
        }
    }

    pub fn settle_records(&self, token: &str) {
        if let Some(state) = lock(&self.executions).get_mut(token) {
            state.settle_record_subscriptions();
        }
    }

    pub fn interrupt(&self) {
        let callbacks: Vec<_> = lock(&self.executions)
            .values_mut()
            .map(|state| {
                state.cancel();
                state.callbacks.clone()
            })
            .collect();
        for callbacks in callbacks {
            callbacks.cancel();
        }
    }

    pub fn shutdown(&self) -> std::pin::Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            self.interrupt();
            let runs = {
                let mut runs = self.runs.lock().unwrap_or_else(PoisonError::into_inner);
                std::mem::take(&mut *runs)
            };
            let mut refused = std::collections::BTreeSet::new();
            for mut run in runs {
                loop {
                    // Finalization owns local input admission after the driver stops polling.
                    for (token, state) in lock(&self.executions).iter_mut() {
                        if !state.local_inputs || refused.contains(token) {
                            continue;
                        }
                        if let Err(error) = state.settle_local_inputs(
                            &self.source,
                            &self.records,
                            &crate::model::time::now_utc(),
                        ) {
                            state.push(crate::transport::buffer::BufferedFrame::log(
                                super::log::Level::Error,
                                error.to_string(),
                            ));
                            refused.insert(token.clone());
                        }
                    }
                    tokio::select! {
                        _ = &mut run => break,
                        _ = tokio::time::sleep(crate::transport::buffer::PROMPT_POLL_INTERVAL) => {}
                    }
                }
            }
            // A panicked run cannot join its children. Retain their handles at
            // the workflow boundary and settle them before resources finalize.
            let observers = {
                let mut observers = self
                    .observers
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner);
                std::mem::take(&mut *observers)
            };
            for observer in &observers {
                observer.abort();
            }
            for observer in observers {
                let _ = observer.await;
            }

            let callbacks: Vec<_> = lock(&self.executions)
                .iter()
                .map(|(token, state)| (token.clone(), state.callbacks.clone()))
                .collect();
            for (token, callbacks) in callbacks {
                let failures = callbacks.settle().await;
                if let Some(state) = lock(&self.executions).get_mut(&token) {
                    for failure in failures {
                        state.push(crate::transport::buffer::BufferedFrame::log(
                            super::log::Level::Error,
                            failure.redacted().to_string(),
                        ));
                    }
                }
            }
        })
    }
}

impl<T: SidecarTemplate> Clone for Workflow<T> {
    fn clone(&self) -> Self {
        Self {
            workflow: self.workflow.clone(),
            source: self.source.clone(),
            declaration: self.declaration.clone(),
            records: self.records.clone(),
            streams: self.streams.clone(),
            executions: Arc::clone(&self.executions),
            runs: Arc::clone(&self.runs),
            observers: Arc::clone(&self.observers),
            template: PhantomData,
            publisher: None,
            watched: self.watched.clone(),
            binding: self.binding.clone(),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn resolved(parameters_json: &str) -> Result<BTreeMap<String, Value>, Refusal> {
    serde_json::from_str(parameters_json)
        .map_err(|error| Refusal::of(format!("invalid resolved parameters JSON: {error}")))
}

fn reused(token: &str) -> Refusal {
    Refusal::of(format!(
        "execution token {token:?} was reused with different inputs"
    ))
}

fn retain_capacity(executions: &mut BTreeMap<String, ExecutionState>) -> Result<(), Refusal> {
    if executions.len() < MAX_RETAINED_EXECUTIONS {
        return Ok(());
    }
    let finished = executions
        .iter()
        .find_map(|(token, state)| state.terminal().then(|| token.clone()));
    match finished {
        Some(token) => {
            executions.remove(&token);
            Ok(())
        }
        None => Err(Refusal::of(format!(
            "sidecar already has {MAX_RETAINED_EXECUTIONS} active executions"
        ))),
    }
}

/* === Tests === */
