/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

#[cfg(feature = "native")]
pub(crate) mod buffer;

#[cfg(feature = "native")]
pub(crate) mod control;

#[cfg(feature = "native")]
pub(crate) mod frame;
pub mod protocol;
pub mod provider;
pub mod recording;

pub(crate) use protocol::templated;
pub use protocol::{
    ArtifactChunk, CommandExecutionReply, CommandExecutionRequest, DeliverRecordRequest, DeliverSamplesRequest, EvidenceBatch,
    EvidenceReceipt, ExecutionSignal, LeaseAsk, LeaseResolution, MAX_ARTIFACT_BYTES, MAX_ARTIFACT_CHUNK_BYTES, OperatorAnswer,
    OperatorQuestion, RUN_BOUND, RunExecution, SIGNAL_BOUND, SidecarCriterionOutcome, SidecarCriterionStatus, SidecarExecutionOutcome,
    SignalReply, StepAdmission,
};
pub use provider::{
    ProviderCriterionOutcome, ProviderCycleOutcome, ProviderDependencyProbe, ProviderExecutionErrorDetail, ProviderExecutionFailure,
    ProviderExecutionOutcome, ProviderPhaseOutcome, ProviderRecordOutcome, ProviderStatusOutcome, ProviderTelemetryOutcome,
    ProviderTelemetryRow,
};

use std::future::Future;
use std::pin::Pin;

use crate::execution::lease::LeaseHold;
use crate::prompts::{PromptRequest, PromptResponse};
use crate::telemetry::{LeaseGap, LeaseRequest};
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::records::publication::RecordPublishRequest;

pub type TransportFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

// An owned handle on the execution's log and records, for work that outlives
// the borrow of its stage: a process's output, a watch's markers.
pub trait Journal: Send + Sync {
    fn log(&self, level: crate::execution::log::Level, message: String);

    fn record(&self, record: RecordPublishRequest);
}

pub trait ExecutionTransport: Send + Sync {
    fn execution(&self) -> &str;

    fn progress(&self) -> std::sync::Arc<dyn crate::execution::ProgressSink>;

    fn prompt<'a>(&'a mut self, request: PromptRequest) -> TransportFuture<'a, crate::Result<PromptResponse>>;

    fn lease<'a>(&'a mut self, request: LeaseRequest) -> TransportFuture<'a, crate::Result<LeaseHold>>;

    fn gaps(&self) -> Vec<LeaseGap>;

    // Exact Gap Count Per Stream; `gaps` May Retain Only Bounded Examples Of A Long Run's Gaps.
    fn gap_counts(&self) -> std::collections::BTreeMap<String, u64> {
        let mut counts = std::collections::BTreeMap::new();
        for gap in self.gaps() {
            *counts.entry(gap.stream.to_string()).or_default() += 1;
        }
        counts
    }

    fn emit(&mut self, record: RecordPublishRequest);

    fn log(&self, level: crate::execution::log::Level, message: String);

    fn journal(&self) -> std::sync::Arc<dyn Journal>;

    fn traceability(&mut self, sections: Vec<dokime_domain::model::traceability::TraceabilitySection>);

    fn capture_phase(&mut self, phase: dokime_domain::model::execution::CapturePhase);

    fn begin_step<'a>(
        &'a mut self, step: Identifier, start: dokime_domain::model::execution::result::StepAttemptStart,
    ) -> TransportFuture<'a, Result<(), crate::diagnostics::Diagnostic>>;

    fn finish_step(&mut self, attempt: dokime_domain::model::execution::result::TestStepAttempt);

    fn document(&self, addition: crate::document::DocumentAddition);
}
