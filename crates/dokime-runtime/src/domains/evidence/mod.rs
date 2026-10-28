/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(crate) mod artifacts;
pub(crate) mod health;
pub mod notes;
pub(crate) mod publish;
mod selection;
mod summary;

pub use crate::artifacts::ArtifactUploadInput;
pub use dokime_domain::model::execution::notes::OperatorNoteRequest;
pub use publish::{PublishReceipt, PublishStatus, RecordPublishRequest};
pub(crate) use publish::{retain_running_samples, validation_rejection};
pub(crate) use selection::scopes_correlate;
pub use selection::{RecordSelection, ScopeSelection, TelemetrySelection};
pub use summary::{EvidenceSummaries, RecordSummary, TelemetryStreamSummary};

use serde::{Deserialize, Serialize};

use crate::artifacts::ArtifactRecord;
use crate::prompts::PromptInstance;
use crate::records::RecordContract;
use crate::telemetry::{TelemetryContract, TelemetrySample};
use dokime_domain::model::execution::health::HealthItem;
use dokime_domain::model::execution::notes::OperatorNote;
use dokime_domain::model::records::evidence::RecordEntry;
use dokime_domain::model::traceability::TraceabilitySnapshot;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct EvidenceSnapshot {
    pub record_contracts: Vec<RecordContract>,
    pub telemetry_contracts: Vec<TelemetryContract>,
    pub summaries: EvidenceSummaries,
    pub records: crate::storage::ReadRows<RecordEntry>,
    pub telemetry: Vec<TelemetrySample>,
    pub prompts: Vec<PromptInstance>,
    pub artifacts: Vec<ArtifactRecord>,
    pub health: Vec<HealthItem>,
    pub traceability: Vec<TraceabilitySnapshot>,
    pub notes: Vec<OperatorNote>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceCounts {
    pub records: usize,
    pub telemetry: usize,
    pub prompts: usize,
    pub artifacts: usize,
    pub health: usize,
    pub traceability: usize,
    pub notes: usize,
}

impl EvidenceSnapshot {
    pub fn counts(&self) -> EvidenceCounts {
        EvidenceCounts {
            records: self.records.len(),
            telemetry: self.telemetry.len(),
            prompts: self.prompts.len(),
            artifacts: self.artifacts.len(),
            health: self.health.len(),
            traceability: self.traceability.len(),
            notes: self.notes.len(),
        }
    }
}
