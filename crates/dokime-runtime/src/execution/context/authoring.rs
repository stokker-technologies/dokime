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
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

// Third Party Imports
// Third-Party Imports
use serde::de::DeserializeOwned;

// Local Imports
use crate::artifacts::Artifacts;
use crate::execution::environment::Environment;
use crate::execution::evidence::{ExecutionEvidence, Samples};
use crate::execution::log::Level;
use crate::prompts::Prompt;
use crate::publication::{Publication, Publishable};
use crate::records::DeclaredRecord;
use crate::telemetry::{LeaseGap, StreamSelection, TelemetryRow, TelemetryStream};
use crate::transport::ExecutionTransport;
use dokime_domain::model::execution::CapturePhase;
use dokime_domain::model::execution::EvidenceOrigin;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::termination::TerminationReason;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::StepAttemptId;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

macro_rules! own_resources {
    ($($stage:ty),+ $(,)?) => {
        $(impl<'a, C, O, I> Context<'a, C, O, I, $stage> {
                                                                                                pub fn resources(&self) -> &'a O {
                self.environment.resources()
            }
        })+
    };
}

own_resources!(Step, Cleanup);

#[cfg(feature = "native")]
own_resources!(Observing);

macro_rules! own_document {
    ($($stage:ty),+ $(,)?) => {
        $(impl<C, O, I> Context<'_, C, O, I, $stage> {
                                                                                                                                                                                                                        pub fn document(&self) -> crate::execution::document::DocumentHandle<'_> {
                crate::execution::document::DocumentHandle::new(self.environment.document(), self.transport)
            }
        })+
    };
}

own_document!(Initialize, Step, Cleanup);

#[cfg(feature = "native")]
own_document!(Observing);

/* === Type Aliases === */

pub type InitializationContext<'a, C = (), O = (), I = ()> = Context<'a, C, O, I, Initialize>;
pub type StepContext<'a, C = (), O = (), I = ()> = Context<'a, C, O, I, Step>;

#[cfg(feature = "native")]
pub type ObserverContext<'a, C = (), O = (), I = ()> = Context<'a, C, O, I, Observing>;
pub type CleanupContext<'a, C = (), O = (), I = ()> = Context<'a, C, O, I, Cleanup>;

/* === Structs === */

pub struct Context<'a, C, O, I, Stage> {
    environment: &'a Environment<'a, C, O, I>,
    transport: &'a mut dyn ExecutionTransport,
    stage: Stage,
    call_stage: String,
}

pub struct Initialize;

pub struct Step;

#[cfg(feature = "native")]
pub struct Observing {
    pub(in crate::execution) tick: crate::execution::tick::Tick,
    pub(in crate::execution) publisher: std::sync::Arc<crate::execution::observing::ExecutionPublisher>,
}

pub struct Cleanup {
    pub reason: TerminationReason,
}

pub struct CriterionContext<'a, C = (), I = ()> {
    calls: std::marker::PhantomData<fn() -> C>,

    #[cfg(feature = "native")]
    caller: crate::execution::Caller,

    #[cfg(feature = "native")]
    binding: Option<std::sync::Arc<dyn std::any::Any + Send + Sync>>,

    parameters: &'a I,
    raw: &'a BTreeMap<String, Value>,
    evidence: &'a ExecutionEvidence,
    selected_attempts: Arc<BTreeSet<StepAttemptId>>,
    log: Option<(&'a dyn ExecutionTransport, &'a str)>,

    #[cfg(feature = "native")]
    callbacks: crate::execution::Callbacks,
}

/* === Enums === */

/* === Traits === */

pub trait Required {
    type Operations;
}

// The bound sits on the operation, so a one-member `requires` catalog never
// lets inference pick the operation before it reads the request.
#[diagnostic::on_unimplemented(
    message = "`{T}` does not list `{Self}` in its `requires` catalog",
    label = "add this operation to the template's `requires = ...` catalog"
)]
pub trait Called<T> {}

/* === Implementations === */

impl<T: Required, O> Called<T> for O where T::Operations: crate::catalog::Contains<O> {}

impl<'a, C, O, I, S> Context<'a, C, O, I, S> {
    pub(in crate::execution) fn new(
        environment: &'a Environment<'a, C, O, I>, transport: &'a mut dyn ExecutionTransport, stage: S,
    ) -> Self {
        Self {
            environment,
            transport,
            stage,
            call_stage: String::new(),
        }
    }

    pub(in crate::execution) fn calling_stage(mut self, stage: impl Into<String>) -> Self {
        self.call_stage = stage.into();
        self
    }

    pub fn environment(&self) -> &'a Environment<'a, C, O, I> {
        self.environment
    }

    pub fn artifacts(&self) -> &'a Artifacts {
        self.environment.artifacts()
    }

    pub fn parameters(&self) -> &'a I {
        self.environment.parameters()
    }

    pub fn raw_parameters(&self) -> &'a BTreeMap<String, Value> {
        self.environment.raw_parameters()
    }

    pub fn parameter<T: DeserializeOwned>(&self, id: &str) -> crate::Result<T> {
        self.environment.parameter(id)
    }

    pub fn secret(&self, id: &str) -> crate::Result<crate::plugin::Secret> {
        self.environment.secret(id)
    }

    pub fn trace(&self, message: impl Into<String>) {
        self.transport.log(Level::Trace, message.into());
    }

    pub fn debug(&self, message: impl Into<String>) {
        self.transport.log(Level::Debug, message.into());
    }

    pub fn info(&self, message: impl Into<String>) {
        self.transport.log(Level::Info, message.into());
    }

    pub fn warn(&self, message: impl Into<String>) {
        self.transport.log(Level::Warn, message.into());
    }

    pub fn error(&self, message: impl Into<String>) {
        self.transport.log(Level::Error, message.into());
    }
}

#[cfg(feature = "native")]
impl<C, O, I, S> Context<'_, C, O, I, S> {
    pub async fn call<Op>(&self, request: Op) -> crate::Result<Op::Response>
    where
        Op: harmos::operations::OperationDeclaration<Request = Op> + harmos::Portable + Called<C>,
    {
        self.environment.caller().call_at(&self.call_stage, request).await
    }

    // The host surface for this stage: `call` and `submit` take the request,
    // which is the operation declaration.
    pub fn host(&self) -> crate::execution::StepHost {
        let binding = crate::execution::builtins::Binding {
            artifacts: self.environment.artifacts().clone(),
            streams: self.environment.streams().clone(),
            journal: self.transport.journal(),
            identity: self.environment.identity(),
            handles: self.environment.handles().clone(),
        };
        self.environment.caller().step_host(&self.call_stage, binding)
    }
}

impl<'a, C, O, I> Context<'a, C, O, I, Step> {
    pub fn traceability(&self) -> crate::execution::traceability::TraceabilityHandle<'_> {
        crate::execution::traceability::TraceabilityHandle::new(self.environment.traceability_index(), self.transport)
    }

    pub fn progress<T>(&self, phase: impl Into<String>, items: T) -> crate::execution::Progress<T::IntoIter>
    where
        T: IntoIterator,
        T::IntoIter: ExactSizeIterator,
    {
        crate::execution::Progress::new(phase, items.into_iter(), self.transport.progress(), 3)
    }

    pub async fn ask<R: Prompt>(&mut self, prompt: R) -> crate::Result<R::Response> {
        let issues = crate::prompts::validate_definition(&prompt.definition());
        if !issues.is_empty() {
            return Err(crate::Error::PromptRefused(
                issues.into_iter().map(|issue| format!("{}: {}", issue.path, issue.message)).collect::<Vec<_>>().join("; "),
            ));
        }
        let response = self.transport.prompt(prompt.request()).await?;
        prompt.decode(response)
    }

    #[cfg(feature = "native")]
    pub async fn lease<R: crate::telemetry::TelemetryStream>(&mut self) -> crate::Result<crate::execution::lease::Lease<'a, R>> {
        self.lease_stream(R::stream_id()).await
    }

    #[cfg(feature = "native")]
    pub async fn lease_stream<R>(
        &mut self, stream: dokime_domain::model::identity::Identifier,
    ) -> crate::Result<crate::execution::lease::Lease<'a, R>> {
        let request = crate::telemetry::LeaseRequest::new(stream.clone());
        let hold = self.transport.lease(request).await?;

        Ok(crate::execution::lease::Lease::new(stream, hold, self.environment.streams().clone()))
    }

    pub async fn publish<V: Publishable>(&mut self, value: V) {
        match value.publication().into() {
            Publication::Row { stream, values, payload } => {
                self.environment.streams().published_as(&stream, values, Some(payload), std::any::type_name::<V>()).await;
            }
            Publication::Record(request) => {
                // A record that follows rows reaches the host after them.
                self.environment.streams().flush().await;
                let request = *request;
                self.transport.emit(request);
            }
        }
    }

    pub fn stream<R: crate::telemetry::TelemetryRow>(&self) -> StreamSelection<'a, R> {
        self.environment.streams().selecting()
    }

    #[cfg(feature = "native")]
    pub fn listen<T: crate::execution::listen::Listenable>(&self) -> T::Listening {
        T::listening(self.environment.streams(), self.environment.deliveries())
    }
}

#[cfg(feature = "native")]
impl<'a, C, O, I> Context<'a, C, O, I, Observing> {
    pub fn tick(&self) -> &crate::execution::tick::Tick {
        &self.stage.tick
    }

    pub async fn publish<V: Publishable>(&mut self, value: V) {
        match value.publication().into() {
            Publication::Row { stream, values, payload } => {
                self.environment.streams().published_as(&stream, values, Some(payload), std::any::type_name::<V>()).await;
            }
            Publication::Record(request) => {
                if let Err(refusal) = self.stage.publisher.record(*request) {
                    self.transport.log(Level::Error, format!("an observer's record was not filed: {refusal}"));
                }
            }
        }
    }

    pub async fn publish_row(&mut self, stream: &Identifier, values: BTreeMap<String, Value>, payload: Vec<u8>) {
        self.environment.streams().published(stream, values, payload).await;
    }

    pub fn stream<R: TelemetryRow>(&self) -> StreamSelection<'a, R> {
        self.environment.streams().selecting()
    }
}

impl<C, O, I> Context<'_, C, O, I, Cleanup> {
    pub fn reason(&self) -> TerminationReason {
        self.stage.reason
    }
}

impl<'a, C, I> CriterionContext<'a, C, I> {
    pub fn new(parameters: &'a I, raw: &'a BTreeMap<String, Value>, evidence: &'a ExecutionEvidence) -> Self {
        Self {
            calls: std::marker::PhantomData,
            #[cfg(feature = "native")]
            caller: crate::execution::Caller::default(),
            #[cfg(feature = "native")]
            binding: None,
            parameters,
            raw,
            evidence,
            selected_attempts: Arc::default(),
            log: None,
            #[cfg(feature = "native")]
            callbacks: crate::execution::Callbacks::default(),
        }
    }

    #[cfg(feature = "native")]
    #[must_use]
    pub fn calling(mut self, caller: crate::execution::Caller) -> Self {
        self.caller = caller;

        self
    }

    #[cfg(feature = "native")]
    #[must_use]
    pub fn bound(mut self, binding: Option<std::sync::Arc<dyn std::any::Any + Send + Sync>>) -> Self {
        self.binding = binding;

        self
    }

    #[cfg(feature = "native")]
    pub fn binding<B: 'static>(&self) -> &B {
        self.binding
            .as_deref()
            .and_then(|binding| binding.downcast_ref::<B>())
            .expect("a bound workflow is decided against the binding it was built from")
    }

    #[cfg(feature = "native")]
    pub async fn call<Op>(&self, request: Op) -> crate::Result<Op::Response>
    where
        Op: harmos::operations::OperationDeclaration<Request = Op> + harmos::Portable + Called<C>,
    {
        let stage = self.log.map_or_else(String::new, |(_, criterion)| format!("criterion/{criterion}"));
        self.caller.call_at(&stage, request).await
    }

    #[cfg(feature = "native")]
    pub(in crate::execution) fn with_callbacks(mut self, callbacks: crate::execution::Callbacks) -> Self {
        self.callbacks = callbacks;

        self
    }

    #[cfg(feature = "native")]
    pub fn callbacks(&self) -> &crate::execution::Callbacks {
        &self.callbacks
    }

    pub fn deciding(mut self, transport: &'a dyn ExecutionTransport, criterion: &'a str) -> Self {
        self.log = Some((transport, criterion));

        self
    }

    pub fn trace(&self, message: impl Into<String>) {
        self.record(Level::Trace, message.into());
    }

    pub fn debug(&self, message: impl Into<String>) {
        self.record(Level::Debug, message.into());
    }

    pub fn info(&self, message: impl Into<String>) {
        self.record(Level::Info, message.into());
    }

    pub fn warn(&self, message: impl Into<String>) {
        self.record(Level::Warn, message.into());
    }

    pub fn error(&self, message: impl Into<String>) {
        self.record(Level::Error, message.into());
    }

    fn record(&self, level: Level, message: String) {
        if let Some((transport, criterion)) = self.log {
            transport.log(level, format!("criterion `{criterion}`: {message}"));
        }
    }

    pub fn parameter<T: DeserializeOwned>(&self, id: &str) -> crate::Result<T> {
        crate::execution::environment::parameter(self.raw, id)
    }

    pub fn parameters(&self) -> &'a I {
        self.parameters
    }

    pub fn raw_parameters(&self) -> &'a BTreeMap<String, Value> {
        self.raw
    }

    pub fn evidence(&self) -> ExecutionEvidence {
        self.selected_evidence(None)
    }

    pub fn evidence_from(&self, phase: CapturePhase) -> ExecutionEvidence {
        self.selected_evidence(Some(phase))
    }

    fn selected_evidence(&self, phase: Option<CapturePhase>) -> ExecutionEvidence {
        ExecutionEvidence {
            samples: self.evidence.samples.selected(|sample| self.selected(sample.scope.as_ref(), phase)),
            subscribed_records: self.evidence.subscribed_records.selected(|record| self.selected(record.scope.as_ref(), phase)),
            attempts: self
                .evidence
                .attempts
                .iter()
                .filter(|attempt| self.selected_attempts.contains(&attempt.attempt_id))
                .cloned()
                .collect(),
            steps: self.evidence.steps.clone(),
            gaps: self.evidence.gaps.clone(),
            gap_counts: self.evidence.gap_counts.clone(),
        }
    }

    #[cfg(feature = "native")]
    pub(crate) fn selecting(mut self, attempts: Arc<BTreeSet<StepAttemptId>>) -> Self {
        self.selected_attempts = attempts;
        self
    }

    fn selected(&self, scope: Option<&RecordScope>, phase: Option<CapturePhase>) -> bool {
        let Some(scope) = scope else {
            return false;
        };
        match (&scope.origin, phase) {
            (EvidenceOrigin::Step { attempt_id }, None) => self.selected_attempts.contains(attempt_id),
            (EvidenceOrigin::Default, Some(CapturePhase::Default))
            | (EvidenceOrigin::Initialization, Some(CapturePhase::Initialization))
            | (EvidenceOrigin::Cleanup, Some(CapturePhase::Cleanup)) => true,
            _ => false,
        }
    }

    pub fn samples<S: TelemetryStream>(&self) -> Samples<S::Row> {
        self.selected_samples::<S>(None)
    }

    pub fn samples_from<S: TelemetryStream>(&self, phase: CapturePhase) -> Samples<S::Row> {
        self.selected_samples::<S>(Some(phase))
    }

    fn selected_samples<S: TelemetryStream>(&self, phase: Option<CapturePhase>) -> Samples<S::Row> {
        let rows = self.evidence.samples.selected(|sample| sample.stream.as_str() == S::ID && self.selected(sample.scope.as_ref(), phase));
        Samples::new(rows, S::Row::from_row)
    }

    pub fn records<R: DeclaredRecord>(&self) -> crate::Result<Vec<R>> {
        self.selected_records::<R>(None)
    }

    pub fn records_from<R: DeclaredRecord>(&self, phase: CapturePhase) -> crate::Result<Vec<R>> {
        self.selected_records::<R>(Some(phase))
    }

    fn selected_records<R: DeclaredRecord>(&self, phase: Option<CapturePhase>) -> crate::Result<Vec<R>> {
        let mut rows = Vec::new();
        let mut bytes = 0usize;
        for record in self
            .evidence
            .subscribed_records
            .iter()
            .filter(|record| record.record_type.as_str() == R::RECORD_TYPE && self.selected(record.scope.as_ref(), phase))
        {
            bytes = bytes
                .checked_add(crate::execution::evidence_rows::encoded_bytes(&record))
                .filter(|bytes| *bytes <= 64 * 1024 * 1024)
                .expect("criterion record collection exceeds bounded workspace; use record_rows");
            rows.push(R::from_details(&record.details).map_err(|error| crate::Error::record_field(R::RECORD_TYPE, error))?);
        }
        Ok(rows)
    }

    pub fn record_rows<R: DeclaredRecord>(&self) -> impl Iterator<Item = crate::Result<R>> + '_ {
        self.evidence
            .subscribed_records
            .iter()
            .filter(|record| record.record_type.as_str() == R::RECORD_TYPE && self.selected(record.scope.as_ref(), None))
            .map(|record| R::from_details(&record.details).map_err(|error| crate::Error::record_field(R::RECORD_TYPE, error)))
    }

    pub fn steps_reached(&self) -> &'a [Identifier] {
        &self.evidence.steps
    }

    pub fn gaps<R: TelemetryStream>(&self) -> Vec<LeaseGap> {
        let stream = R::ID;
        self.evidence.gaps.iter().filter(|gap| gap.stream.as_str() == stream).cloned().collect()
    }

    // Every Gap The Stream Had; `gaps` Returns Bounded Examples Of A Long Run's Gaps.
    pub fn gap_count<R: TelemetryStream>(&self) -> u64 {
        self.evidence.gap_counts.get(R::ID).copied().unwrap_or_default()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
