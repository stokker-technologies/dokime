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
use std::pin::Pin;
use std::time::Duration;
// Third Party Imports

// Local Imports
use crate::core::Value;
use crate::definition::TestCaseTemplate;
use crate::execution::{
    CleanupContext, CriterionContext, InitializationContext, ObserverContext, StepContext,
    TraceabilityContext,
};
use crate::parameters::FromParameters;
use crate::plugin::Diagnostic;
use crate::traceability::TraceabilitySection;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

// Publishes one stream's rows in order as one batch; rows before a refused
// row stay published and the refused row and the rest are missed.
pub type SamplePublisher = for<'a> fn(
    &'a harmos::sidecar::Supervisor,
    &'a str,
    &'a crate::core::Identifier,
    &'a [StampedRow],
) -> crate::transport::TransportFuture<'a, Result<(), String>>;

pub type StepFuture<'a> = Pin<Box<dyn Future<Output = Result<(), Diagnostic>> + Send + 'a>>;

pub type InitializationFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<T, Diagnostic>> + Send + 'a>>;

pub type TraceabilityFuture<'a> =
    Pin<Box<dyn Future<Output = Result<Vec<TraceabilitySection>, Diagnostic>> + Send + 'a>>;

pub type EvaluationFuture<'a, T> =
    Pin<Box<dyn Future<Output = Result<Option<T>, Diagnostic>> + Send + 'a>>;

/* === Structs === */
#[derive(Clone)]
pub struct TemplateDefinition {
    pub template: TestCaseTemplate,
    pub records: Vec<crate::records::RecordContract>,
    pub streams: Vec<crate::telemetry::TelemetryContract>,
}

// A row waiting for publication with the instant it was produced, in Unix
// microseconds on the publication clock.
#[derive(Clone, Debug)]
pub struct StampedRow {
    pub values: BTreeMap<String, Value>,
    pub published_unix_micros: i64,
}

#[derive(Clone, Debug)]
pub struct DeclaredObserver {
    pub identifier: String,
    pub label: String,
    pub interval: Duration,
    pub streams: Vec<crate::telemetry::TelemetryContract>,
    pub records: Vec<crate::records::RecordContract>,
}

/* === Enums === */

/* === Traits === */
pub trait SidecarTemplate: Send + Sized + 'static {
    const RUN: &'static str;

    const SIGNAL: &'static str;

    const REQUIRED: &'static [harmos::catalog::Node<harmos::operations::WireDeclaration>] = &[];

    /// The native streams this template and its observers publish; the plugin
    /// serving its suite registers them.
    const STREAMS: &'static [harmos::catalog::Node<crate::catalog::Publication>] = &[];

    const OPERATIONS: &'static [harmos::catalog::Node<harmos::operations::WireDeclaration>] = &[
        harmos::catalog::Node::Leaf(harmos::operations::WireDeclaration::of::<super::Run<Self>>()),
        harmos::catalog::Node::Leaf(
            harmos::operations::WireDeclaration::of::<super::Signal<Self>>(),
        ),
    ];

    type Own: Send + Sync + 'static;

    type Parameters: FromParameters;

    fn definition() -> TemplateDefinition;

    fn initialize<'a>(
        ctx: &'a mut InitializationContext<'_, Self, Self::Own, Self::Parameters>,
    ) -> InitializationFuture<'a, (Self, Self::Own)>;

    fn traceability<'a>(
        &'a self,
        _ctx: &'a TraceabilityContext<'_, Self, Self::Own, Self::Parameters>,
    ) -> TraceabilityFuture<'a> {
        Box::pin(async { Ok(Vec::new()) })
    }

    fn declared_observers() -> Vec<DeclaredObserver> {
        Vec::new()
    }

    fn observe<'a>(
        _observer: &'a str,
        _ctx: &'a mut ObserverContext<'_, Self, Self::Own, Self::Parameters>,
    ) -> StepFuture<'a> {
        Box::pin(async { Ok(()) })
    }

    fn publish_sample<'a, P: harmos::plugin::Declared>(
        _host: &'a harmos::sidecar::Supervisor,
        _token: &'a str,
        stream: &'a crate::core::Identifier,
        _rows: &'a [StampedRow],
    ) -> crate::transport::TransportFuture<'a, Result<(), String>> {
        Box::pin(async move {
            Err(format!(
                "template has no native publisher for stream {stream}"
            ))
        })
    }

    fn cancellation_is_completion(_parameters: &BTreeMap<String, Value>) -> bool {
        false
    }

    fn run_step<'a>(
        &'a mut self,
        step: &'a str,
        ctx: &'a mut StepContext<'_, Self, Self::Own, Self::Parameters>,
    ) -> StepFuture<'a>;

    fn cleanup<'a>(
        &'a mut self,
        _ctx: &'a mut CleanupContext<'_, Self, Self::Own, Self::Parameters>,
    ) -> StepFuture<'a> {
        Box::pin(async { Ok(()) })
    }

    fn evaluate<'a>(
        criterion: &'a str,
        ctx: &'a CriterionContext<'_, Self, Self::Parameters>,
    ) -> EvaluationFuture<'a, bool>;
}

/* === Implementations === */
impl DeclaredObserver {
    pub fn new(identifier: impl Into<String>, label: impl Into<String>, interval_ms: u64) -> Self {
        Self {
            identifier: identifier.into(),
            label: label.into(),
            interval: Duration::from_millis(interval_ms),
            streams: Vec::new(),
            records: Vec::new(),
        }
    }

    #[must_use]
    pub fn publishing(
        mut self,
        streams: Vec<crate::telemetry::TelemetryContract>,
        records: Vec<crate::records::RecordContract>,
    ) -> Self {
        self.streams = streams;
        self.records = records;

        self
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
