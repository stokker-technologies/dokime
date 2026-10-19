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
pub(crate) const MAX_UNAVAILABLE_REASON_BYTES: usize = 512;

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
impl harmos::MemorySize for DocumentPayload {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.document_id), std::mem::size_of_val(&self.document_id)),
            (harmos::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos::MemorySize::memory_size(&self.selection), std::mem::size_of_val(&self.selection)),
            (harmos::MemorySize::memory_size(&self.cards), std::mem::size_of_val(&self.cards)),
            (harmos::MemorySize::memory_size(&self.layout), std::mem::size_of_val(&self.layout)),
            (harmos::MemorySize::memory_size(&self.plot_descriptors), std::mem::size_of_val(&self.plot_descriptors)),
            (harmos::MemorySize::memory_size(&self.plot_inputs), std::mem::size_of_val(&self.plot_inputs)),
            (harmos::MemorySize::memory_size(&self.record_markers), std::mem::size_of_val(&self.record_markers)),
            (harmos::MemorySize::memory_size(&self.artifact_lists), std::mem::size_of_val(&self.artifact_lists)),
            (harmos::MemorySize::memory_size(&self.warnings), std::mem::size_of_val(&self.warnings)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos::MemorySize for DocumentCardPlacementPayload {
    fn memory_size(&self) -> usize {
        let fields = [(harmos::MemorySize::memory_size(&self.card_id), std::mem::size_of_val(&self.card_id))];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos::MemorySize for DocumentCardPayload {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos::MemorySize::memory_size(&self.body), std::mem::size_of_val(&self.body)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos::MemorySize for TelemetrySummaryGroupPayload {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos::MemorySize::memory_size(&self.tiles), std::mem::size_of_val(&self.tiles)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl TelemetrySummaryTilePayload {
    pub fn valid_result(&self) -> bool {
        self.unavailable.as_ref().is_none_or(|reason| valid_unavailable_reason(reason) && self.value.is_none() && self.state.is_none())
    }
}

impl harmos::MemorySize for TelemetrySummaryTilePayload {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.unavailable), std::mem::size_of_val(&self.unavailable)),
            (harmos::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos::MemorySize::memory_size(&self.value), std::mem::size_of_val(&self.value)),
            (harmos::MemorySize::memory_size(&self.unit), std::mem::size_of_val(&self.unit)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos::MemorySize for ArtifactListDescriptor {
    fn memory_size(&self) -> usize {
        let fields = [(harmos::MemorySize::memory_size(&self.scope), std::mem::size_of_val(&self.scope))];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos::MemorySize for DocumentWarning {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos::MemorySize::memory_size(&self.code), std::mem::size_of_val(&self.code)),
            (harmos::MemorySize::memory_size(&self.message), std::mem::size_of_val(&self.message)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
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

impl harmos::MemorySize for DocumentCardBodyPayload {
    fn memory_size(&self) -> usize {
        let heap = match self {
            Self::Text { markdown } => markdown.capacity(),
            Self::TelemetrySummary { groups } => owned_heap(groups.memory_size(), std::mem::size_of_val(groups)),
            Self::Plot { descriptor } => descriptor.as_ref().memory_size(),
            Self::Table { rows } => owned_heap(rows.memory_size(), std::mem::size_of_val(rows)),
            Self::Artifacts { descriptor } | Self::Media { descriptor } => descriptor.as_ref().memory_size(),
            Self::Steps | Self::Criteria | Self::Traceability | Self::Notes | Self::Records | Self::Health | Self::TelemetryContext => 0,
        };
        std::mem::size_of::<Self>().saturating_add(heap)
    }
}

impl harmos::MemorySize for DocumentLayoutPayload {
    fn memory_size(&self) -> usize {
        let nodes = nodes_memory(&self.nodes, 0);
        std::mem::size_of::<Self>().saturating_add(owned_heap(nodes, std::mem::size_of_val(&self.nodes)))
    }
}

impl harmos::MemorySize for DocumentNodePayload {
    fn memory_size(&self) -> usize {
        node_memory(self, 0)
    }
}

impl harmos::MemorySize for DocumentSectionNodePayload {
    fn memory_size(&self) -> usize {
        section_memory(self, 0)
    }
}

/* === Public Functions === */

/* === Private Functions === */
pub(crate) fn valid_unavailable_reason(reason: &str) -> bool {
    !reason.trim().is_empty() && reason.len() <= MAX_UNAVAILABLE_REASON_BYTES
}

fn owned_heap(owned: usize, inline: usize) -> usize {
    if owned == usize::MAX { usize::MAX } else { owned.saturating_sub(inline) }
}

fn nodes_memory(nodes: &Vec<DocumentNodePayload>, depth: usize) -> usize {
    let mut bytes =
        std::mem::size_of_val(nodes).saturating_add(nodes.capacity().saturating_mul(std::mem::size_of::<DocumentNodePayload>()));
    for node in nodes {
        bytes = bytes.saturating_add(owned_heap(node_memory(node, depth), std::mem::size_of::<DocumentNodePayload>()));
    }
    bytes
}

fn node_memory(node: &DocumentNodePayload, depth: usize) -> usize {
    use harmos::MemorySize;
    // Match The Authored Document Forest's Existing Depth Bound
    if depth > 64 {
        return usize::MAX;
    }
    let heap = match node {
        DocumentNodePayload::Section(section) => owned_heap(section_memory(section, depth), std::mem::size_of_val(section)),
        DocumentNodePayload::Card(card) => owned_heap(card.memory_size(), std::mem::size_of_val(card)),
    };
    std::mem::size_of_val(node).saturating_add(heap)
}

fn section_memory(section: &DocumentSectionNodePayload, depth: usize) -> usize {
    use harmos::MemorySize;
    if depth > 64 {
        return usize::MAX;
    }
    let fields = [
        (section.node_id.memory_size(), std::mem::size_of_val(&section.node_id)),
        (section.metadata.memory_size(), std::mem::size_of_val(&section.metadata)),
        (nodes_memory(&section.children, depth + 1), std::mem::size_of_val(&section.children)),
    ];
    fields.into_iter().fold(std::mem::size_of_val(section), |bytes, (owned, inline)| bytes.saturating_add(owned_heap(owned, inline)))
}

/* === Tests === */
