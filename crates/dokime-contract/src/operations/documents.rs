/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod capture_current_document;

/* === Re-Exports === */
pub use capture_current_document::CaptureCurrentDocument;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::common::Metadata;
#[allow(unused_imports)]
use super::common::*;
#[allow(unused_imports)]
use super::plots::*;
#[allow(unused_imports)]
use super::schema::*;
use super::value::Value;
use super::{Access, Endpoint, TRANSFER};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentScope {
    pub cycle_id: String,
    pub execution_id: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureCurrentDocumentRequest {
    pub scope: Option<DocumentScope>,
    pub document_id: String,
}

// Sources are captured separately; this is not an atomic latest/durable snapshot.
// Missing frontiers mean the source was not needed by this capture.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentFrontiers {
    pub process_epoch: String,
    pub state_revision: u64,
    pub telemetry: Option<DocumentTelemetryCut>,
    // SHA-256 of this captured artifact/report descriptor input, not a journal
    // position or published checkpoint. These owners have no shared revision.
    #[row(
        description = "SHA-256 of this captured artifact/report descriptor input, not a journal position or published checkpoint. These owners have no shared revision."
    )]
    pub artifacts_sha256: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentTelemetryCut {
    pub delivered_cursor: u64,
    pub source: Option<DocumentTelemetryCutSource>,
    pub series_ordinal: Option<u64>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentHeader {
    pub document_id: String,
    pub metadata: Option<Metadata>,
    pub scope: Option<EvidenceScope>,
    pub frontiers: Option<DocumentFrontiers>,
}

// Flat pre-order layout nodes retain ordered parent/child identity without an
// unbounded recursive protobuf. A top-level node has no parent; nested
// sections/placements do.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentLayoutNode {
    pub ordinal: u64,
    pub parent_ordinal: Option<u64>,
    pub node: Option<DocumentLayoutNodeNode>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSection {
    pub node_id: String,
    pub metadata: Option<Metadata>,
    pub repeat: Option<DocumentRepeat>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentRepeat {
    pub value: Option<DataExpression>,
    pub label_template: String,
    pub id_template: Option<String>,
    pub order_by: Option<DataExpression>,
    pub filters: Vec<PlotFilter>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentPlacement {
    pub card_id: String,
    pub column_span: Option<DocumentSpan>,
    pub row_span: u32,
    pub order: i32,
    pub density: DocumentDensity,
    pub rail: DocumentRail,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSpan {
    pub base: u32,
    pub sm: Option<u32>,
    pub md: Option<u32>,
    pub lg: Option<u32>,
    pub xl: Option<u32>,
}

// Authored Cards Are Captured Once; Bindings Carry Instance Results Separately.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentCard {
    pub metadata: Option<Metadata>,
    pub filters: Vec<PlotFilter>,
    pub body: Option<DocumentCardBody>,
}

// Built-in live content remains owned by its domain API. Plot references select
// a plot within this authored document; they do not contain client-authored input.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentOwnerReference {
    pub kind: DocumentCardKind,
    pub plot_id: Option<String>,
    // Plot owners carry the authored input and presentation alongside the
    // stable owner identity. The request has no cursor; callers add one only
    // when opening a live plot stream.
    #[row(
        description = "Plot owners carry the authored input and presentation alongside the stable owner identity. The request has no cursor; callers add one only when opening a live plot stream."
    )]
    pub plot: Option<DocumentPlotDefinition>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentPlotDefinition {
    pub metadata: Option<Metadata>,
    pub request: Option<PlotRequest>,
    pub presentation: Option<PlotPresentation>,
    // Authored z-axis data selection. This belongs to the document definition
    // and is kept separate from PlotRequest because the runtime query currently
    // evaluates only the request's lowered fields.
    #[row(
        description = "Authored z-axis data selection. This belongs to the document definition and is kept separate from PlotRequest because the runtime query currently evaluates only the request's lowered fields."
    )]
    pub z_expression: Option<DataExpression>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotPresentation {
    pub group: String,
    pub style: PlotStyle,
    pub fft: Option<PlotFftOptions>,
    pub interval: Option<PlotIntervalOptions>,
    pub histogram: Option<PlotHistogramOptions>,
    pub reference_lines: Vec<PlotReferenceLine>,
    pub reference_bands: Vec<PlotReferenceBand>,
    pub live_window_s: Option<f64>,
    pub x_format: Option<ValueFormat>,
    pub y_format: Option<ValueFormat>,
    // What the x axis measures. Unspecified is wall time, so a payload written
    // before this was declared still reads correctly.
    #[row(
        description = "What the x axis measures. Unspecified is wall time, so a payload written before this was declared still reads correctly."
    )]
    pub x_axis: PlotAxisKind,
    // The unit an elapsed x axis counts in, as the column declared it: "s",
    // "ms", "us", "min", "h". Absent for a wall-time axis.
    #[row(
        description = "The unit an elapsed x axis counts in, as the column declared it: \"s\", \"ms\", \"us\", \"min\", \"h\". Absent for a wall-time axis."
    )]
    pub x_axis_unit: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotReferenceLine {
    pub axis: ReferenceAxis,
    pub value: Option<f64>,
    pub source: Option<DataExpression>,
    pub source_scale: f64,
    pub label: Option<String>,
    pub color: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotReferenceBand {
    pub axis: ReferenceAxis,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub label: Option<String>,
    pub color: String,
    pub center: Option<DataExpression>,
    pub tolerance: Option<DataExpression>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentTable {
    pub row_count: u64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentTableRow {
    pub card_id: String,
    pub ordinal: u64,
    pub values: Option<Value>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSummary {
    pub group_count: u64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSummaryGroup {
    pub card_id: String,
    pub ordinal: u64,
    pub metadata: Option<Metadata>,
    pub tile_count: u64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSummaryTile {
    pub card_id: String,
    pub group_ordinal: u64,
    pub ordinal: u64,
    pub metadata: Option<Metadata>,
    pub value: Option<Value>,
    pub stat: DocumentSummaryStat,
    pub format: DocumentSummaryFormat,
    pub presentation: DocumentSummaryPresentation,
    pub unit: Option<String>,
    pub state: Option<DocumentSummaryState>,
    // Evaluation failure; value and state must be absent. Missing evidence alone
    // remains an absent value, not an unavailable result.
    #[row(
        description = "Evaluation failure; value and state must be absent. Missing evidence alone remains an absent value, not an unavailable result."
    )]
    pub unavailable: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentWarning {
    pub code: String,
    pub message: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSummaryGroupDefinition {
    pub card_id: String,
    pub ordinal: u64,
    pub metadata: Option<Metadata>,
    pub tile_count: u64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSummaryTileDefinition {
    pub card_id: String,
    pub group_ordinal: u64,
    pub ordinal: u64,
    pub metadata: Option<Metadata>,
    pub value: Option<DataExpression>,
    pub filters: Vec<PlotFilter>,
    pub stat: DocumentSummaryStat,
    pub format: DocumentSummaryFormat,
    pub presentation: DocumentSummaryPresentation,
    pub unit: Option<String>,
    pub conditions: Vec<DocumentSummaryCondition>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSummaryCondition {
    pub operator: FilterOperator,
    pub value: Option<Value>,
    pub state: DocumentSummaryState,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentBinding {
    pub source_id: String,
    pub metadata: Option<Metadata>,
    pub result: Option<DocumentBindingResult>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentRepeatSelection {
    pub node_id: String,
    pub instance_count: u64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentRepeatInstance {
    pub selection_id: String,
    pub ordinal: u64,
    pub suffix: String,
    pub node_id: String,
    pub filter: Option<PlotFilter>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentSectionIdentity {
    pub node_id: String,
    pub metadata: Option<Metadata>,
    // An unevaluable repeat retains its original section identity with no instances.
    #[row(
        description = "An unevaluable repeat retains its original section identity with no instances."
    )]
    pub unavailable: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentCompletion {
    pub frame_count: u64,
    pub card_count: u64,
    pub layout_node_count: u64,
    pub binding_count: u64,
    pub repeat_count: u64,
    pub section_count: u64,
}

// Header Is First; Completion Is Last. Apply Only After Clean gRPC EOF.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentFrame {
    pub frame: Option<DocumentFrameFrame>,
}

/* === Enums === */
#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentCardKind {
    Unspecified,
    Text,
    TelemetrySummary,
    Plot,
    Table,
    Steps,
    Criteria,
    Traceability,
    Notes,
    Records,
    Artifacts,
    Media,
    Health,
    TelemetryContext,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentDensity {
    Unspecified,
    Compact,
    Normal,
    Spacious,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentRail {
    Unspecified,
    Main,
    Right,
    Left,
}

// What a plot's x axis measures, so a client never infers it from a name.
#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlotAxisKind {
    Unspecified,
    // An absolute instant: the axis draws a date and a clock face.
    WallTime,
    // A duration counted from the start of the run, in `x_axis_unit`.
    Elapsed,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlotStyle {
    Unspecified,
    Line,
    Points,
    LinePoints,
    Step,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferenceAxis {
    Unspecified,
    X,
    Y,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentSummaryStat {
    Unspecified,
    Latest,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentSummaryFormat {
    Unspecified,
    Auto,
    Number,
    Percent,
    Duration,
    Distance,
    Status,
    Text,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentSummaryPresentation {
    Unspecified,
    Value,
    Progress,
    Badge,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DocumentSummaryState {
    Unspecified,
    Ok,
    Warning,
    Critical,
    Muted,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum DocumentTelemetryCutSource {
    SeriesEpoch(u64),
    RetainedWindow(bool),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum DocumentLayoutNodeNode {
    Section(DocumentSection),
    Placement(DocumentPlacement),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum DocumentCardBody {
    Markdown(String),
    Table(DocumentTable),
    Summary(DocumentSummary),
    Owner(DocumentOwnerReference),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum DocumentBindingResult {
    Literal(bool),
    Summary(DocumentSummary),
    Plot(DocumentPlotDefinition),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum DocumentFrameFrame {
    Header(DocumentHeader),
    LayoutNode(DocumentLayoutNode),
    Card(DocumentCard),
    TableRow(DocumentTableRow),
    SummaryGroup(DocumentSummaryGroup),
    SummaryTile(DocumentSummaryTile),
    Warning(DocumentWarning),
    Completion(DocumentCompletion),
    GroupDefinition(DocumentSummaryGroupDefinition),
    TileDefinition(DocumentSummaryTileDefinition),
    Binding(DocumentBinding),
    RepeatSelection(DocumentRepeatSelection),
    RepeatInstance(DocumentRepeatInstance),
    SectionIdentity(DocumentSectionIdentity),
}

#[harmos::catalog]
pub enum Operations {
    CaptureCurrentDocument(CaptureCurrentDocument),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for CaptureCurrentDocument {
    const PATH: &'static str = "/dokime.v1.Documents/CaptureCurrentDocument";
    const ACCESS: Access = Access::Read;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
