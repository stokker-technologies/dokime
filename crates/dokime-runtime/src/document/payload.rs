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

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::artifacts::ArtifactStatus;
use crate::document::{
    definition::PlotDescriptor,
    definition::{DocumentCardLayout, TelemetrySummaryFormat, TelemetrySummaryPresentation, TelemetrySummaryStat, TelemetrySummaryState},
    input::DocumentSelection,
};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::DocumentId;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::records::evidence::RecordMarker;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentPayload {
    pub document_id: DocumentId,
    pub metadata: Metadata,
    pub selection: DocumentSelection,

    #[serde(default)]
    pub cards: Vec<DocumentCardPayload>,

    #[serde(default)]
    pub layout: DocumentLayoutPayload,

    pub plot_descriptors: Vec<PlotDescriptor>,
    pub plot_inputs: Vec<crate::document::definition::PlotInput>,
    pub record_markers: Vec<RecordMarker>,
    pub artifact_lists: Vec<ArtifactListDescriptor>,
    pub warnings: Vec<DocumentWarning>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct DocumentLayoutPayload {
    #[serde(default)]
    pub nodes: Vec<DocumentNodePayload>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentSectionNodePayload {
    pub node_id: Identifier,
    pub metadata: Metadata,

    #[serde(default)]
    pub children: Vec<DocumentNodePayload>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentCardPlacementPayload {
    pub card_id: Identifier,

    #[serde(default)]
    pub layout: DocumentCardLayout,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentCardPayload {
    pub metadata: Metadata,

    #[serde(flatten)]
    pub body: DocumentCardBodyPayload,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelemetrySummaryGroupPayload {
    pub metadata: Metadata,

    #[serde(default)]
    pub tiles: Vec<TelemetrySummaryTilePayload>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelemetrySummaryTilePayload {
    pub metadata: Metadata,
    pub value: Option<Value>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unavailable: Option<String>,

    #[serde(default)]
    pub stat: TelemetrySummaryStat,

    #[serde(default)]
    pub format: TelemetrySummaryFormat,

    #[serde(default)]
    pub presentation: TelemetrySummaryPresentation,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<TelemetrySummaryState>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactListDescriptor {
    pub scope: RecordScope,
    pub artifact_count: usize,
    pub completed_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentWarning {
    pub code: Identifier,
    pub message: String,
}

/* === Enums === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum DocumentNodePayload {
    Section(DocumentSectionNodePayload),
    Card(DocumentCardPlacementPayload),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum DocumentCardBodyPayload {
    Text { markdown: String },
    TelemetrySummary { groups: Vec<TelemetrySummaryGroupPayload> },
    Plot { descriptor: Box<PlotDescriptor> },
    Table { rows: Vec<std::collections::BTreeMap<String, Value>> },
    Steps,
    Criteria,
    Traceability,
    Notes,
    Records,
    Artifacts { descriptor: Box<ArtifactListDescriptor> },
    Media { descriptor: Box<ArtifactListDescriptor> },
    Health,
    TelemetryContext,
}

/* === Traits === */

/* === Implementations === */

impl TelemetrySummaryTilePayload {
    pub fn valid_result(&self) -> bool {
        self.unavailable.as_ref().is_none_or(|reason| valid_unavailable_reason(reason) && self.value.is_none() && self.state.is_none())
    }
}

impl ArtifactListDescriptor {
    pub fn from_statuses(scope: RecordScope, statuses: impl IntoIterator<Item = ArtifactStatus>) -> Self {
        let mut artifact_count = 0;
        let mut completed_count = 0;

        for status in statuses {
            artifact_count += 1;
            if status == ArtifactStatus::Complete {
                completed_count += 1;
            }
        }

        Self { scope, artifact_count, completed_count }
    }
}

/* === Public Functions === */

pub(crate) fn valid_unavailable_reason(reason: &str) -> bool {
    !reason.trim().is_empty()
}

/* === Private Functions === */

/* === Tests === */
