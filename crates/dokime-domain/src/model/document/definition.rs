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

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use crate::model::execution::RecordScope;
use crate::model::expressions::DataExpression;
use crate::model::expressions::DataSource;
use crate::model::expressions::RecordMarker;
use crate::model::identity::DocumentId;
use crate::model::identity::Identifier;
use crate::model::identity::Metadata;
use crate::model::telemetry::TelemetryColumn;
use crate::model::telemetry::TelemetryContract;
use crate::model::values::Value;
use crate::model::values::ValueFormat;
use crate::model::values::ValueType;
use crate::model::values::format::ValueFormatKind;
use crate::model::values::format::validate_value_format_with;

/* === Constants === */

pub const DURATION_UNIT_SYMBOLS: &[&str] = &["s", "ms", "us", "\u{b5}s", "min", "h"];

/* === Macros === */

/* === Type Aliases === */

pub type Document = DocumentDefinition;
pub type Section = DocumentSection;
pub type Card = DocumentCard;

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentDefinition {
    pub document_id: DocumentId,
    pub metadata: Metadata,

    #[serde(default)]
    pub cards: Vec<DocumentCardDefinition>,

    #[serde(default)]
    pub layout: DocumentLayout,

    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sections: Vec<DocumentSection>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Default)]
pub struct DocumentLayout {
    #[serde(default)]
    pub nodes: Vec<DocumentNode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentSectionNode {
    pub node_id: Identifier,
    pub metadata: Metadata,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat: Option<DocumentSectionRepeat>,

    #[serde(default)]
    pub children: Vec<DocumentNode>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentCardPlacement {
    pub card_id: Identifier,

    #[serde(default)]
    pub layout: DocumentCardLayout,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentCardDefinition {
    pub metadata: Metadata,

    #[serde(default)]
    pub filters: Vec<DocumentFilter>,

    #[serde(flatten)]
    pub body: DocumentCardBody,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentSection {
    pub metadata: Metadata,

    #[serde(default)]
    pub cards: Vec<DocumentCard>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repeat: Option<DocumentSectionRepeat>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentCard {
    pub metadata: Metadata,

    #[serde(default)]
    pub layout: DocumentCardLayout,

    #[serde(default)]
    pub filters: Vec<DocumentFilter>,

    #[serde(flatten)]
    pub body: DocumentCardBody,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentCardLayout {
    #[serde(default)]
    pub col_span: ResponsiveSpan,

    #[serde(default = "default_row_span")]
    pub row_span: u8,

    #[serde(default)]
    pub order: i32,

    #[serde(default)]
    pub density: DocumentCardDensity,

    #[serde(default)]
    pub rail: DocumentCardRail,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponsiveSpan {
    #[serde(default = "default_base_span")]
    pub base: u8,

    #[serde(default)]
    pub sm: Option<u8>,

    #[serde(default)]
    pub md: Option<u8>,

    #[serde(default)]
    pub lg: Option<u8>,

    #[serde(default)]
    pub xl: Option<u8>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentSectionRepeat {
    pub value: DataExpression,
    pub label_template: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id_template: Option<String>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub order_by: Option<DataExpression>,

    #[serde(default)]
    pub filters: Vec<DocumentFilter>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelemetrySummaryDefinition {
    #[serde(default)]
    pub groups: Vec<TelemetrySummaryGroupDefinition>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelemetrySummaryGroupDefinition {
    pub metadata: Metadata,

    #[serde(default)]
    pub tiles: Vec<TelemetrySummaryTileDefinition>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelemetrySummaryTileDefinition {
    pub metadata: Metadata,
    pub value: DataExpression,

    #[serde(default)]
    pub filters: Vec<DocumentFilter>,

    #[serde(default)]
    pub stat: TelemetrySummaryStat,

    #[serde(default)]
    pub format: TelemetrySummaryFormat,

    #[serde(default)]
    pub presentation: TelemetrySummaryPresentation,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,

    #[serde(default)]
    pub conditions: Vec<TelemetrySummaryCondition>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TelemetrySummaryCondition {
    pub operator: FilterOperator,
    pub value: Value,
    pub state: TelemetrySummaryState,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlotDefinition {
    #[serde(skip)]
    pub automatic_id: bool,

    pub metadata: Metadata,
    pub plot_type: PlotType,

    #[serde(default)]
    pub stream: Option<String>,

    #[serde(default)]
    pub x: Option<String>,

    #[serde(default)]
    pub y: Vec<String>,

    #[serde(default)]
    pub z_stream: Option<String>,

    #[serde(default)]
    pub z_field: Option<String>,

    #[serde(default)]
    pub series_by: Option<String>,

    #[serde(default)]
    pub x_expression: Option<DataExpression>,

    #[serde(default)]
    pub y_expressions: Vec<DataExpression>,

    #[serde(default)]
    pub z_expression: Option<DataExpression>,

    #[serde(default)]
    pub series_by_expression: Option<DataExpression>,

    #[serde(default)]
    pub filters: Vec<DocumentFilter>,

    #[serde(default)]
    pub unit: Option<String>,

    #[serde(default)]
    pub style: PlotStyle,

    #[serde(default)]
    pub x_scale: AxisScale,

    #[serde(default)]
    pub y_scale: AxisScale,

    #[serde(default)]
    pub fft: FftOptions,

    #[serde(default)]
    pub analysis: Option<crate::model::analysis::Selection>,

    #[serde(default)]
    pub interval: SigmaIntervalOptions,

    #[serde(default)]
    pub histogram: HistogramOptions,

    #[serde(default)]
    pub marker_record_types: Vec<String>,

    #[serde(default)]
    pub reference_lines: Vec<ReferenceLine>,

    #[serde(default)]
    pub reference_bands: Vec<ReferenceBand>,

    #[serde(default = "default_live_window_s")]
    pub live_window_s: Option<f64>,

    #[serde(default)]
    pub x_format: Option<ValueFormat>,

    #[serde(default)]
    pub y_format: Option<ValueFormat>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_axis: Option<PlotAxisKind>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_points: Option<u32>,

    #[serde(default)]
    pub downsample: DownsampleTechnique,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentFilter {
    pub expression: DataExpression,
    pub operator: FilterOperator,

    #[serde(default)]
    pub value: Option<Value>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct FftOptions {
    #[serde(default)]
    pub sample_rate_hz: Option<f64>,

    #[serde(default)]
    pub window: FftWindow,

    #[serde(default)]
    pub detrend: FftDetrend,

    #[serde(default)]
    pub level_unit: FrequencyLevelUnit,

    #[serde(default)]
    pub include_dc: bool,

    #[serde(default)]
    pub min_frequency_hz: Option<f64>,

    #[serde(default)]
    pub max_frequency_hz: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SigmaIntervalOptions {
    #[serde(default = "default_interval_bin_width")]
    pub bin_width: f64,

    #[serde(default = "default_interval_sigma")]
    pub sigma: f64,

    #[serde(default = "default_true")]
    pub clip_to_observed: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HistogramOptions {
    #[serde(default = "default_histogram_bins")]
    pub bins: usize,

    #[serde(default)]
    pub unit: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReferenceLine {
    pub axis: ReferenceAxis,

    #[serde(default)]
    pub value: Option<f64>,

    #[serde(default)]
    pub source: Option<DataExpression>,

    #[serde(default = "default_reference_source_scale")]
    pub source_scale: f64,

    #[serde(default)]
    pub label: Option<String>,

    #[serde(default = "default_reference_color")]
    pub color: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReferenceBand {
    pub axis: ReferenceAxis,

    #[serde(default)]
    pub min: Option<f64>,

    #[serde(default)]
    pub max: Option<f64>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub center: Option<DataExpression>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tolerance: Option<DataExpression>,

    #[serde(default)]
    pub label: Option<String>,

    #[serde(default = "default_reference_color")]
    pub color: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Band {
    min: Option<f64>,
    max: Option<f64>,
    center: Option<DataExpression>,
    tolerance: Option<DataExpression>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    value: Option<f64>,
    source: Option<DataExpression>,
    scale: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlotDescriptor {
    pub plot_id: Identifier,
    pub metadata: Metadata,
    pub plot_type: PlotType,
    pub input: PlotInput,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlotInput {
    pub plot_id: Identifier,
    pub scope: RecordScope,
    pub plot_type: PlotType,
    pub x_field: Option<String>,
    pub series_fields: Vec<String>,
    pub z_stream: Option<String>,
    pub z_field: Option<String>,
    pub stream: Option<String>,
    pub series_by: Option<String>,

    #[serde(default)]
    pub x_expression: Option<DataExpression>,

    #[serde(default)]
    pub y_expressions: Vec<DataExpression>,

    #[serde(default)]
    pub z_expression: Option<DataExpression>,

    #[serde(default)]
    pub series_by_expression: Option<DataExpression>,

    #[serde(default)]
    pub filters: Vec<DocumentFilter>,

    pub unit: Option<String>,
    pub style: PlotStyle,
    pub x_scale: AxisScale,
    pub y_scale: AxisScale,
    pub fft: FftOptions,

    #[serde(default)]
    pub analysis: Option<crate::model::analysis::Selection>,

    pub interval: SigmaIntervalOptions,
    pub histogram: HistogramOptions,
    pub marker_record_types: Vec<String>,
    pub reference_lines: Vec<ReferenceLine>,
    pub reference_bands: Vec<ReferenceBand>,
    pub live_window_s: Option<f64>,

    #[serde(default)]
    pub x_format: Option<ValueFormat>,

    #[serde(default)]
    pub y_format: Option<ValueFormat>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_axis: Option<PlotAxisKind>,

    pub available_streams: Vec<Identifier>,
    pub marker_count: usize,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_points: Option<u32>,

    #[serde(default)]
    pub downsample: DownsampleTechnique,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentDataset {
    pub grain: DocumentDatasetGrain,

    #[serde(default)]
    pub rows: Vec<BTreeMap<String, Value>>,

    #[serde(default)]
    pub warnings: Vec<DocumentDatasetWarning>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentDatasetWarning {
    pub code: Identifier,
    pub message: String,

    #[serde(default)]
    pub count: u64,
}

// DSL Constructors

pub struct Plot;

/* === Enums === */

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DownsampleTechnique {
    Unspecified,
    Raw,
    MinMax,
    Lttb,

    #[default]
    Auto,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum DocumentNode {
    Section(Box<DocumentSectionNode>),
    Card(DocumentCardPlacement),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum DocumentCardBody {
    Text { markdown: String },
    TelemetrySummary { summary: TelemetrySummaryDefinition },
    Plot { plot: Box<PlotDefinition> },
    Table { rows: Vec<BTreeMap<String, Value>> },
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentCardBodyKind {
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentCardDensity {
    Compact,

    #[default]
    Normal,

    Spacious,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentCardRail {
    #[default]
    Main,

    Right,
    Left,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlotType {
    TimeSeries,
    Bar,
    GroupedBar,
    StackedBar,
    Pie,
    Donut,
    Pareto,
    XyScatter,
    XyLine,
    XyInterval,
    Trajectory,
    Heatmap,
    XyValueHeatmap,
    WifiChannelOccupancy,
    FrequencySeries,
    Histogram,
    BoxSummary,
    AggregateTable,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlotStyle {
    #[default]
    Line,

    Points,
    LinePoints,
    Step,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AxisScale {
    #[default]
    Linear,

    Log,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PlotAxisKind {
    #[default]
    WallTime,

    Elapsed {
        unit: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    Contains,
    Exists,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelemetrySummaryStat {
    #[default]
    Latest,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelemetrySummaryFormat {
    #[default]
    Auto,

    Number,
    Percent,
    Duration,
    Distance,
    Status,
    Text,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelemetrySummaryPresentation {
    #[default]
    Value,

    Progress,
    Badge,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TelemetrySummaryState {
    Ok,
    Warning,
    Critical,
    Muted,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DocumentDatasetGrain {
    CycleView,
    TestCase,
    TestExecution,
    TelemetrySample,
    Record,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FrequencyLevelUnit {
    #[default]
    Linear,

    #[serde(rename = "dB")]
    Db,

    #[serde(rename = "dBu")]
    Dbu,

    #[serde(rename = "dBm")]
    Dbm,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FftDetrend {
    None,

    #[default]
    Mean,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FftWindow {
    #[default]
    Hann,

    Rectangular,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceAxis {
    X,
    Y,
}

/* === Traits === */

pub trait DocumentTagged {
    fn metadata_mut(&mut self) -> &mut Metadata;

    fn tag(&mut self, tag: impl Into<String>) {
        self.metadata_mut().tags.push(tag.into());
    }

    fn tags<I, S>(&mut self, tags: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.metadata_mut().tags.extend(tags.into_iter().map(Into::into));
    }
}

/* === Implementations === */

impl harmos_domain::MemorySize for PlotDescriptor {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.plot_id), std::mem::size_of_val(&self.plot_id)),
            (harmos_domain::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos_domain::MemorySize::memory_size(&self.input), std::mem::size_of_val(&self.input)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos_domain::MemorySize for PlotInput {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.analysis), std::mem::size_of_val(&self.analysis)),
            (harmos_domain::MemorySize::memory_size(&self.plot_id), std::mem::size_of_val(&self.plot_id)),
            (harmos_domain::MemorySize::memory_size(&self.scope), std::mem::size_of_val(&self.scope)),
            (harmos_domain::MemorySize::memory_size(&self.x_field), std::mem::size_of_val(&self.x_field)),
            (harmos_domain::MemorySize::memory_size(&self.series_fields), std::mem::size_of_val(&self.series_fields)),
            (harmos_domain::MemorySize::memory_size(&self.z_stream), std::mem::size_of_val(&self.z_stream)),
            (harmos_domain::MemorySize::memory_size(&self.z_field), std::mem::size_of_val(&self.z_field)),
            (harmos_domain::MemorySize::memory_size(&self.stream), std::mem::size_of_val(&self.stream)),
            (harmos_domain::MemorySize::memory_size(&self.series_by), std::mem::size_of_val(&self.series_by)),
            (harmos_domain::MemorySize::memory_size(&self.x_expression), std::mem::size_of_val(&self.x_expression)),
            (harmos_domain::MemorySize::memory_size(&self.y_expressions), std::mem::size_of_val(&self.y_expressions)),
            (harmos_domain::MemorySize::memory_size(&self.z_expression), std::mem::size_of_val(&self.z_expression)),
            (harmos_domain::MemorySize::memory_size(&self.series_by_expression), std::mem::size_of_val(&self.series_by_expression)),
            (harmos_domain::MemorySize::memory_size(&self.filters), std::mem::size_of_val(&self.filters)),
            (harmos_domain::MemorySize::memory_size(&self.unit), std::mem::size_of_val(&self.unit)),
            (harmos_domain::MemorySize::memory_size(&self.histogram), std::mem::size_of_val(&self.histogram)),
            (harmos_domain::MemorySize::memory_size(&self.marker_record_types), std::mem::size_of_val(&self.marker_record_types)),
            (harmos_domain::MemorySize::memory_size(&self.reference_lines), std::mem::size_of_val(&self.reference_lines)),
            (harmos_domain::MemorySize::memory_size(&self.reference_bands), std::mem::size_of_val(&self.reference_bands)),
            (harmos_domain::MemorySize::memory_size(&self.x_format), std::mem::size_of_val(&self.x_format)),
            (harmos_domain::MemorySize::memory_size(&self.y_format), std::mem::size_of_val(&self.y_format)),
            (harmos_domain::MemorySize::memory_size(&self.x_axis), std::mem::size_of_val(&self.x_axis)),
            (harmos_domain::MemorySize::memory_size(&self.available_streams), std::mem::size_of_val(&self.available_streams)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos_domain::MemorySize for DocumentDefinition {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.document_id), std::mem::size_of_val(&self.document_id)),
            (harmos_domain::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos_domain::MemorySize::memory_size(&self.cards), std::mem::size_of_val(&self.cards)),
            (harmos_domain::MemorySize::memory_size(&self.layout), std::mem::size_of_val(&self.layout)),
            (harmos_domain::MemorySize::memory_size(&self.sections), std::mem::size_of_val(&self.sections)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos_domain::MemorySize for DocumentCardDefinition {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos_domain::MemorySize::memory_size(&self.filters), std::mem::size_of_val(&self.filters)),
            (harmos_domain::MemorySize::memory_size(&self.body), std::mem::size_of_val(&self.body)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos_domain::MemorySize for DocumentCard {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos_domain::MemorySize::memory_size(&self.filters), std::mem::size_of_val(&self.filters)),
            (harmos_domain::MemorySize::memory_size(&self.body), std::mem::size_of_val(&self.body)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos_domain::MemorySize for DocumentSection {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos_domain::MemorySize::memory_size(&self.cards), std::mem::size_of_val(&self.cards)),
            (harmos_domain::MemorySize::memory_size(&self.repeat), std::mem::size_of_val(&self.repeat)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos_domain::MemorySize for DocumentCardBody {
    fn memory_size(&self) -> usize {
        let heap = match self {
            Self::Text { markdown } => markdown.capacity(),
            Self::TelemetrySummary { summary } => {
                let owned = harmos_domain::MemorySize::memory_size(summary);
                if owned == usize::MAX {
                    return usize::MAX;
                }
                owned.saturating_sub(std::mem::size_of_val(summary))
            }
            Self::Plot { plot } => harmos_domain::MemorySize::memory_size(plot.as_ref()),
            Self::Table { rows } => {
                let owned = harmos_domain::MemorySize::memory_size(rows);
                if owned == usize::MAX {
                    return usize::MAX;
                }
                owned.saturating_sub(std::mem::size_of_val(rows))
            }
            Self::Steps
            | Self::Criteria
            | Self::Traceability
            | Self::Notes
            | Self::Records
            | Self::Artifacts
            | Self::Media
            | Self::Health
            | Self::TelemetryContext => 0,
        };
        std::mem::size_of::<Self>().saturating_add(heap)
    }
}

impl harmos_domain::MemorySize for PlotDefinition {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos_domain::MemorySize::memory_size(&self.stream), std::mem::size_of_val(&self.stream)),
            (harmos_domain::MemorySize::memory_size(&self.x), std::mem::size_of_val(&self.x)),
            (harmos_domain::MemorySize::memory_size(&self.y), std::mem::size_of_val(&self.y)),
            (harmos_domain::MemorySize::memory_size(&self.z_stream), std::mem::size_of_val(&self.z_stream)),
            (harmos_domain::MemorySize::memory_size(&self.z_field), std::mem::size_of_val(&self.z_field)),
            (harmos_domain::MemorySize::memory_size(&self.series_by), std::mem::size_of_val(&self.series_by)),
            (harmos_domain::MemorySize::memory_size(&self.x_expression), std::mem::size_of_val(&self.x_expression)),
            (harmos_domain::MemorySize::memory_size(&self.y_expressions), std::mem::size_of_val(&self.y_expressions)),
            (harmos_domain::MemorySize::memory_size(&self.z_expression), std::mem::size_of_val(&self.z_expression)),
            (harmos_domain::MemorySize::memory_size(&self.series_by_expression), std::mem::size_of_val(&self.series_by_expression)),
            (harmos_domain::MemorySize::memory_size(&self.filters), std::mem::size_of_val(&self.filters)),
            (harmos_domain::MemorySize::memory_size(&self.unit), std::mem::size_of_val(&self.unit)),
            (harmos_domain::MemorySize::memory_size(&self.histogram), std::mem::size_of_val(&self.histogram)),
            (harmos_domain::MemorySize::memory_size(&self.marker_record_types), std::mem::size_of_val(&self.marker_record_types)),
            (harmos_domain::MemorySize::memory_size(&self.reference_lines), std::mem::size_of_val(&self.reference_lines)),
            (harmos_domain::MemorySize::memory_size(&self.reference_bands), std::mem::size_of_val(&self.reference_bands)),
            (harmos_domain::MemorySize::memory_size(&self.x_format), std::mem::size_of_val(&self.x_format)),
            (harmos_domain::MemorySize::memory_size(&self.y_format), std::mem::size_of_val(&self.y_format)),
            (harmos_domain::MemorySize::memory_size(&self.x_axis), std::mem::size_of_val(&self.x_axis)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos_domain::MemorySize for HistogramOptions {
    fn memory_size(&self) -> usize {
        let fields = [(harmos_domain::MemorySize::memory_size(&self.unit), std::mem::size_of_val(&self.unit))];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos_domain::MemorySize for ReferenceLine {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.source), std::mem::size_of_val(&self.source)),
            (harmos_domain::MemorySize::memory_size(&self.label), std::mem::size_of_val(&self.label)),
            (harmos_domain::MemorySize::memory_size(&self.color), std::mem::size_of_val(&self.color)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos_domain::MemorySize for ReferenceBand {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.center), std::mem::size_of_val(&self.center)),
            (harmos_domain::MemorySize::memory_size(&self.tolerance), std::mem::size_of_val(&self.tolerance)),
            (harmos_domain::MemorySize::memory_size(&self.label), std::mem::size_of_val(&self.label)),
            (harmos_domain::MemorySize::memory_size(&self.color), std::mem::size_of_val(&self.color)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos_domain::MemorySize for FftOptions {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

impl harmos_domain::MemorySize for SigmaIntervalOptions {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

impl harmos_domain::MemorySize for PlotAxisKind {
    fn memory_size(&self) -> usize {
        let heap = match self {
            Self::WallTime => 0,
            Self::Elapsed { unit } => unit.capacity(),
        };
        std::mem::size_of::<Self>().saturating_add(heap)
    }
}

impl harmos_domain::MemorySize for TelemetrySummaryDefinition {
    fn memory_size(&self) -> usize {
        let fields = [(harmos_domain::MemorySize::memory_size(&self.groups), std::mem::size_of_val(&self.groups))];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos_domain::MemorySize for TelemetrySummaryGroupDefinition {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos_domain::MemorySize::memory_size(&self.tiles), std::mem::size_of_val(&self.tiles)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos_domain::MemorySize for TelemetrySummaryTileDefinition {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.metadata), std::mem::size_of_val(&self.metadata)),
            (harmos_domain::MemorySize::memory_size(&self.value), std::mem::size_of_val(&self.value)),
            (harmos_domain::MemorySize::memory_size(&self.filters), std::mem::size_of_val(&self.filters)),
            (harmos_domain::MemorySize::memory_size(&self.unit), std::mem::size_of_val(&self.unit)),
            (harmos_domain::MemorySize::memory_size(&self.conditions), std::mem::size_of_val(&self.conditions)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos_domain::MemorySize for TelemetrySummaryCondition {
    fn memory_size(&self) -> usize {
        let fields = [(harmos_domain::MemorySize::memory_size(&self.value), std::mem::size_of_val(&self.value))];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos_domain::MemorySize for DocumentLayout {
    fn memory_size(&self) -> usize {
        let nodes = document_nodes_memory(&self.nodes, 0);
        if nodes == usize::MAX {
            return usize::MAX;
        }
        std::mem::size_of::<Self>().saturating_add(nodes.saturating_sub(std::mem::size_of_val(&self.nodes)))
    }
}

impl harmos_domain::MemorySize for DocumentNode {
    fn memory_size(&self) -> usize {
        document_node_memory(self, 0)
    }
}

impl harmos_domain::MemorySize for DocumentSectionNode {
    fn memory_size(&self) -> usize {
        document_section_memory(self, 0)
    }
}

impl harmos_domain::MemorySize for DocumentFilter {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.expression), std::mem::size_of_val(&self.expression)),
            (harmos_domain::MemorySize::memory_size(&self.value), std::mem::size_of_val(&self.value)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos_domain::MemorySize for DocumentSectionRepeat {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.value), std::mem::size_of_val(&self.value)),
            (harmos_domain::MemorySize::memory_size(&self.label_template), std::mem::size_of_val(&self.label_template)),
            (harmos_domain::MemorySize::memory_size(&self.id_template), std::mem::size_of_val(&self.id_template)),
            (harmos_domain::MemorySize::memory_size(&self.order_by), std::mem::size_of_val(&self.order_by)),
            (harmos_domain::MemorySize::memory_size(&self.filters), std::mem::size_of_val(&self.filters)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos_domain::MemorySize for DocumentCardPlacement {
    fn memory_size(&self) -> usize {
        let fields = [(harmos_domain::MemorySize::memory_size(&self.card_id), std::mem::size_of_val(&self.card_id))];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl Default for DocumentCardLayout {
    fn default() -> Self {
        Self {
            col_span: ResponsiveSpan::default(),
            row_span: default_row_span(),
            order: 0,
            density: DocumentCardDensity::Normal,
            rail: DocumentCardRail::Main,
        }
    }
}

impl Default for ResponsiveSpan {
    fn default() -> Self {
        Self { base: default_base_span(), sm: None, md: None, lg: None, xl: None }
    }
}

impl Default for SigmaIntervalOptions {
    fn default() -> Self {
        Self { bin_width: default_interval_bin_width(), sigma: default_interval_sigma(), clip_to_observed: true }
    }
}

impl Default for HistogramOptions {
    fn default() -> Self {
        Self { bins: default_histogram_bins(), unit: None }
    }
}

impl DocumentDefinition {
    pub fn new(identifier: impl AsRef<str>, label: impl Into<String>) -> Self {
        let identifier = identifier.as_ref();
        Self {
            document_id: DocumentId::from_identifier(identifier).unwrap_or_else(|_| DocumentId::from_seed(identifier)),
            metadata: metadata(identifier, label.into(), None),
            cards: Vec::new(),
            layout: DocumentLayout::default(),
            sections: Vec::new(),
        }
    }

    pub fn section(&mut self, section: DocumentSection) {
        self.insert_section_node(section, None);
    }

    pub fn upsert_section(&mut self, section: DocumentSection) {
        let section_id = section.metadata.identifier.clone();
        let existing_index = first_section_node_index(&self.layout, &section_id);
        remove_section_node_from_layout(&mut self.layout, &section_id);
        self.insert_section_node(section, existing_index);
    }

    pub fn upsert_card(&mut self, section_id: &Identifier, section_label: impl Into<String>, card: DocumentCard) {
        self.upsert_card_definition(DocumentCardDefinition::from(card.clone()));
        let placement =
            DocumentNode::Card(DocumentCardPlacement { card_id: card.metadata.identifier.clone(), layout: card.layout.clone() });
        let section = self.layout.nodes.iter_mut().find_map(|node| section_node_mut(node, section_id));

        match section {
            Some(section) => upsert_card_placement(&mut section.children, placement),
            None => {
                let mut created = DocumentSection::new(section_label);
                created.id(section_id.as_str());
                created.card(card);
                self.insert_section_node(created, None);
            }
        }
    }

    pub fn prepend_sections(&mut self, sections: impl IntoIterator<Item = DocumentSection>) {
        for (index, section) in sections.into_iter().enumerate() {
            remove_section_node_from_layout(&mut self.layout, &section.metadata.identifier);
            self.insert_section_node(section, Some(index));
        }
    }

    pub fn section_count(&self) -> usize {
        self.normalized_parts().1.section_count()
    }

    pub fn card_count(&self) -> usize {
        self.normalized_parts().0.len()
    }

    pub fn merge_content_from(&mut self, source: &DocumentDefinition) {
        let (cards, layout) = source.normalized_parts();
        for card in cards {
            self.upsert_card_definition(card);
        }
        self.merge_layout(layout);
    }

    pub fn normalized_parts(&self) -> (Vec<DocumentCardDefinition>, DocumentLayout) {
        let card_capacity = self
            .sections
            .iter()
            .try_fold(self.cards.len(), |count, section| count.checked_add(section.cards.len()))
            .expect("normalized card capacity overflows");
        let node_capacity = self.layout.nodes.len().checked_add(self.sections.len()).expect("normalized node capacity overflows");
        let mut cards = Vec::with_capacity(card_capacity);
        cards.extend(self.cards.iter().cloned());
        let mut layout = DocumentLayout { nodes: Vec::with_capacity(node_capacity) };
        layout.nodes.extend(self.layout.nodes.iter().cloned());

        for section in &self.sections {
            for card in &section.cards {
                upsert_card_definition(&mut cards, DocumentCardDefinition::from(card.clone()));
            }
            let mut children = Vec::with_capacity(section.cards.len());
            for card in &section.cards {
                children.push(DocumentNode::Card(DocumentCardPlacement {
                    card_id: card.metadata.identifier.clone(),
                    layout: card.layout.clone(),
                }));
            }
            layout.nodes.push(DocumentNode::Section(Box::new(DocumentSectionNode {
                node_id: section.metadata.identifier.clone(),
                metadata: section.metadata.clone(),
                repeat: section.repeat.clone(),
                children,
            })));
        }

        if layout.nodes.is_empty() && !cards.is_empty() {
            layout.nodes = Vec::with_capacity(cards.len());
            for card in &cards {
                layout.nodes.push(DocumentNode::Card(DocumentCardPlacement {
                    card_id: card.metadata.identifier.clone(),
                    layout: DocumentCardLayout::default(),
                }));
            }
        }

        (cards, layout)
    }

    fn upsert_card_definition(&mut self, card: DocumentCardDefinition) {
        if let Some(existing) = self.cards.iter_mut().find(|existing| existing.metadata.identifier == card.metadata.identifier) {
            *existing = card;
        } else {
            self.cards.push(card);
        }
    }

    fn insert_section_node(&mut self, section: DocumentSection, insertion_index: Option<usize>) {
        for card in &section.cards {
            self.upsert_card_definition(DocumentCardDefinition::from(card.clone()));
        }

        let node = DocumentNode::Section(Box::new(DocumentSectionNode::from(section)));
        match insertion_index {
            Some(index) if index <= self.layout.nodes.len() => {
                self.layout.nodes.insert(index, node);
            }
            _ => self.layout.nodes.push(node),
        }
    }

    fn merge_layout(&mut self, layout: DocumentLayout) {
        merge_layout_children(&mut self.layout.nodes, layout.nodes);
    }

    pub(crate) fn fold_authored_sections(&mut self) {
        if self.sections.is_empty() {
            return;
        }

        for section in std::mem::take(&mut self.sections) {
            self.insert_section_node(section, None);
        }
    }

    pub(crate) fn insert_added_section(&mut self, section: DocumentSection) {
        self.insert_section_node(section, None);
    }

    pub(crate) fn append_card(&mut self, section_id: &Identifier, card: DocumentCard) {
        let placement =
            DocumentNode::Card(DocumentCardPlacement { card_id: card.metadata.identifier.clone(), layout: card.layout.clone() });
        self.upsert_card_definition(DocumentCardDefinition::from(card));
        if let Some(section) = self.layout.nodes.iter_mut().find_map(|node| section_node_mut(node, section_id)) {
            section.children.push(placement);
        }
    }
}

impl DocumentLayout {
    pub fn section_count(&self) -> usize {
        nodes_section_count(&self.nodes)
    }
}

impl From<DocumentCard> for DocumentCardDefinition {
    fn from(card: DocumentCard) -> Self {
        Self { metadata: card.metadata, filters: card.filters, body: card.body }
    }
}

impl From<DocumentSection> for DocumentSectionNode {
    fn from(section: DocumentSection) -> Self {
        Self {
            node_id: section.metadata.identifier.clone(),
            metadata: section.metadata,
            repeat: section.repeat,
            children: section
                .cards
                .into_iter()
                .map(|card| DocumentNode::Card(DocumentCardPlacement { card_id: card.metadata.identifier, layout: card.layout }))
                .collect(),
        }
    }
}

impl DocumentSection {
    pub fn new(label: impl Into<String>) -> Self {
        let label = label.into();
        let identifier = identifier_from_label(&label);

        Self { metadata: Metadata::new(identifier, label, None), cards: Vec::new(), repeat: None }
    }

    pub fn id(&mut self, identifier: impl AsRef<str>) {
        self.metadata.identifier = Identifier::new(identifier.as_ref()).unwrap_or_else(|_| identifier_from_label(identifier.as_ref()));
    }

    pub fn card(&mut self, card: DocumentCard) {
        self.cards.push(card);
    }

    pub fn plot(&mut self, plot: PlotDefinition) {
        self.card(DocumentCard::plot(plot));
    }

    pub fn note(&mut self, text: impl Into<String>) {
        let identifier = format!("{}.note.{}", self.metadata.identifier.as_str(), self.cards.len());
        self.card(DocumentCard::text(identifier, "Note", text));
    }

    pub fn each(&mut self, value: impl Into<DataExpression>, label_template: impl Into<String>) -> &mut DocumentSectionRepeat {
        self.repeat.insert(DocumentSectionRepeat::new(value, label_template))
    }

    pub fn compact(&mut self) {
        for card in &mut self.cards {
            card.layout.density = DocumentCardDensity::Compact;
        }
    }
}

impl DocumentCard {
    pub fn telemetry_summary(summary: TelemetrySummaryDefinition) -> Self {
        let metadata = summary
            .groups
            .first()
            .map(|group| group.metadata.clone())
            .unwrap_or_else(|| metadata("telemetry.summary", "Telemetry Summary".to_owned(), None));
        Self { metadata, layout: DocumentCardLayout::default(), filters: Vec::new(), body: DocumentCardBody::TelemetrySummary { summary } }
    }

    pub fn plot(mut plot: PlotDefinition) -> Self {
        plot.refresh_identifier();

        Self {
            metadata: plot.metadata.clone(),
            layout: DocumentCardLayout::default(),
            filters: Vec::new(),
            body: DocumentCardBody::Plot { plot: Box::new(plot) },
        }
    }

    pub fn text(identifier: impl AsRef<str>, label: impl Into<String>, markdown: impl Into<String>) -> Self {
        Self {
            metadata: metadata(identifier.as_ref(), label.into(), None),
            layout: DocumentCardLayout::default(),
            filters: Vec::new(),
            body: DocumentCardBody::Text { markdown: markdown.into() },
        }
    }

    pub fn only(&mut self, filter: DocumentFilter) {
        self.filters.push(filter);
    }
}

impl DocumentCardBody {
    pub fn kind(&self) -> DocumentCardBodyKind {
        match self {
            DocumentCardBody::Text { .. } => DocumentCardBodyKind::Text,
            DocumentCardBody::TelemetrySummary { .. } => DocumentCardBodyKind::TelemetrySummary,
            DocumentCardBody::Plot { .. } => DocumentCardBodyKind::Plot,
            DocumentCardBody::Table { .. } => DocumentCardBodyKind::Table,
            DocumentCardBody::Steps => DocumentCardBodyKind::Steps,
            DocumentCardBody::Criteria => DocumentCardBodyKind::Criteria,
            DocumentCardBody::Traceability => DocumentCardBodyKind::Traceability,
            DocumentCardBody::Notes => DocumentCardBodyKind::Notes,
            DocumentCardBody::Records => DocumentCardBodyKind::Records,
            DocumentCardBody::Artifacts => DocumentCardBodyKind::Artifacts,
            DocumentCardBody::Media => DocumentCardBodyKind::Media,
            DocumentCardBody::Health => DocumentCardBodyKind::Health,
            DocumentCardBody::TelemetryContext => DocumentCardBodyKind::TelemetryContext,
        }
    }
}

impl DocumentSectionRepeat {
    fn new(value: impl Into<DataExpression>, label_template: impl Into<String>) -> Self {
        Self { value: value.into(), label_template: label_template.into(), id_template: None, order_by: None, filters: Vec::new() }
    }

    pub fn id(&mut self, template: impl Into<String>) -> &mut Self {
        self.id_template = Some(template.into());
        self
    }

    pub fn order_by(&mut self, order_by: impl Into<DataExpression>) -> &mut Self {
        self.order_by = Some(order_by.into());
        self
    }

    pub fn only(&mut self, filter: DocumentFilter) -> &mut Self {
        self.filters.push(filter);
        self
    }
}

// Comparisons Build The Filters `only` Takes, One Per Operator.
impl DataExpression {
    pub fn eq(self, value: impl Into<Value>) -> DocumentFilter {
        DocumentFilter::compared(self, FilterOperator::Eq, value)
    }

    pub fn ne(self, value: impl Into<Value>) -> DocumentFilter {
        DocumentFilter::compared(self, FilterOperator::Ne, value)
    }

    pub fn gt(self, value: impl Into<Value>) -> DocumentFilter {
        DocumentFilter::compared(self, FilterOperator::Gt, value)
    }

    pub fn gte(self, value: impl Into<Value>) -> DocumentFilter {
        DocumentFilter::compared(self, FilterOperator::Gte, value)
    }

    pub fn lt(self, value: impl Into<Value>) -> DocumentFilter {
        DocumentFilter::compared(self, FilterOperator::Lt, value)
    }

    pub fn lte(self, value: impl Into<Value>) -> DocumentFilter {
        DocumentFilter::compared(self, FilterOperator::Lte, value)
    }

    pub fn contains(self, value: impl Into<Value>) -> DocumentFilter {
        DocumentFilter::compared(self, FilterOperator::Contains, value)
    }

    pub fn exists(self) -> DocumentFilter {
        DocumentFilter { expression: self, operator: FilterOperator::Exists, value: None }
    }
}

impl DocumentFilter {
    fn compared(expression: DataExpression, operator: FilterOperator, value: impl Into<Value>) -> Self {
        Self { expression, operator, value: Some(value.into()) }
    }
}

impl TelemetrySummaryDefinition {
    pub fn new() -> Self {
        Self { groups: Vec::new() }
    }

    pub fn group(&mut self, group: TelemetrySummaryGroupDefinition) {
        self.groups.push(group);
    }
}

impl Default for TelemetrySummaryDefinition {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetrySummaryGroupDefinition {
    pub fn new(identifier: impl AsRef<str>, label: impl Into<String>) -> Self {
        Self { metadata: metadata(identifier.as_ref(), label.into(), None), tiles: Vec::new() }
    }

    pub fn tile(&mut self, tile: TelemetrySummaryTileDefinition) {
        self.tiles.push(tile);
    }
}

impl TelemetrySummaryTileDefinition {
    pub fn new(label: impl Into<String>, value: DataExpression) -> Self {
        let label = label.into();
        Self {
            metadata: Metadata::new(identifier_from_label(&label), label, None),
            value,
            filters: Vec::new(),
            stat: TelemetrySummaryStat::Latest,
            format: TelemetrySummaryFormat::Auto,
            presentation: TelemetrySummaryPresentation::Value,
            unit: None,
            conditions: Vec::new(),
        }
    }

    pub fn id(&mut self, identifier: impl AsRef<str>) {
        self.metadata.identifier = Identifier::new(identifier.as_ref()).unwrap_or_else(|_| identifier_from_label(identifier.as_ref()));
    }

    pub fn unit(&mut self, unit: impl Into<String>) {
        self.unit = Some(unit.into());
    }

    pub fn format(&mut self, format: TelemetrySummaryFormat) {
        self.format = format;
    }

    pub fn presentation(&mut self, presentation: TelemetrySummaryPresentation) {
        self.presentation = presentation;
    }

    pub fn only(&mut self, filter: DocumentFilter) {
        self.filters.push(filter);
    }

    pub fn condition(&mut self, operator: FilterOperator, value: impl Into<Value>, state: TelemetrySummaryState) {
        self.conditions.push(TelemetrySummaryCondition { operator, value: value.into(), state });
    }
}

impl PlotAxisKind {
    pub fn elapsed(unit: impl Into<String>) -> Self {
        Self::Elapsed { unit: unit.into() }
    }

    pub fn for_column(column: &TelemetryColumn) -> Option<Self> {
        if let Some(symbol) = column.unit.as_ref().map(|unit| unit.symbol.as_str()).filter(|symbol| DURATION_UNIT_SYMBOLS.contains(symbol))
        {
            return Some(Self::elapsed(symbol));
        }

        match column.value_type {
            ValueType::Datetime => Some(Self::WallTime),
            _ => None,
        }
    }

    pub fn elapsed_unit(&self) -> Option<&str> {
        match self {
            Self::WallTime => None,
            Self::Elapsed { unit } => Some(unit.as_str()),
        }
    }
}

impl PlotDefinition {
    pub fn with_type(label: impl Into<String>, plot_type: PlotType) -> Self {
        let label = label.into();
        let identifier = identifier_from_label(&label);

        Self {
            automatic_id: true,
            metadata: Metadata::new(identifier, label, None),
            plot_type,
            stream: None,
            x: None,
            y: Vec::new(),
            z_stream: None,
            z_field: None,
            series_by: None,
            x_expression: None,
            y_expressions: Vec::new(),
            z_expression: None,
            series_by_expression: None,
            filters: Vec::new(),
            unit: None,
            style: PlotStyle::Line,
            x_scale: AxisScale::Linear,
            y_scale: AxisScale::Linear,
            fft: FftOptions::default(),
            analysis: None,
            interval: SigmaIntervalOptions::default(),
            histogram: HistogramOptions::default(),
            marker_record_types: Vec::new(),
            reference_lines: Vec::new(),
            reference_bands: Vec::new(),
            live_window_s: default_live_window_s(),
            x_format: None,
            y_format: None,
            x_axis: None,
            display_points: None,
            downsample: DownsampleTechnique::Auto,
        }
    }

    fn refresh_identifier(&mut self) {
        if !self.automatic_id {
            return;
        }

        let mut labels = vec![plot_type_label(&self.plot_type)];
        if let Some(expression) = &self.x_expression {
            labels.push(expression.axis_label());
        } else if let Some(field) = self.x.as_deref() {
            labels.push(field);
        } else if self.plot_type == PlotType::TimeSeries {
            labels.push("time");
        }

        labels.extend(self.y_expressions.iter().map(DataExpression::axis_label));
        labels.extend(self.y.iter().map(String::as_str));
        if self.y_expressions.is_empty() && self.y.is_empty() {
            // An unfinished plot retains its label-derived ID.
            return;
        }

        self.metadata.identifier = identifier_from_label(&labels.join("."));
    }

    pub fn id(&mut self, identifier: impl AsRef<str>) {
        self.automatic_id = false;
        self.metadata.identifier = Identifier::new(identifier.as_ref()).unwrap_or_else(|_| identifier_from_label(identifier.as_ref()));
    }

    pub fn unit(&mut self, unit: impl Into<String>) {
        self.unit = Some(unit.into());
    }

    pub fn stream(&mut self, stream: impl Into<String>) {
        self.stream = Some(stream.into());
    }

    pub fn x(&mut self, expression: impl Into<DataExpression>) {
        self.x_expression = Some(expression.into());
        self.refresh_identifier();
    }

    pub fn y(&mut self, expression: impl Into<DataExpression>) {
        self.y_expressions.push(expression.into());
        self.refresh_identifier();
    }

    pub fn value(&mut self, expression: impl Into<DataExpression>) {
        self.y(expression);
    }

    pub fn x_format(&mut self, format: ValueFormat) {
        self.x_format = Some(format);
    }

    pub fn x_axis(&mut self, axis: PlotAxisKind) {
        self.x_axis = Some(axis);
    }

    pub fn resolve_x_axis(&mut self, contracts: &[TelemetryContract]) {
        if self.x_axis.is_some() {
            return;
        }
        let Some((stream, field)) = self.x_axis_reference() else {
            return;
        };
        let Some(column) = contracts
            .iter()
            .filter(|contract| stream.is_none_or(|stream| contract.stream.as_str() == stream))
            .find_map(|contract| contract.columns.iter().find(|column| column.metadata.identifier.as_str() == field))
        else {
            return;
        };

        self.x_axis = PlotAxisKind::for_column(column);
    }

    fn x_axis_reference(&self) -> Option<(Option<&str>, &str)> {
        if let Some(expression) = &self.x_expression {
            if !expression.transforms.is_empty() {
                return None;
            }
            if let DataSource::Telemetry { stream, field } = &expression.source {
                return Some((Some(stream.as_str()), field.as_str()));
            }
            return None;
        }

        Some((self.stream.as_deref(), self.x.as_deref()?))
    }

    pub fn y_format(&mut self, format: ValueFormat) {
        self.y_format = Some(format);
    }

    pub fn z(&mut self, expression: impl Into<DataExpression>) {
        self.z_expression = Some(expression.into());
    }

    pub fn series_by(&mut self, expression: impl Into<DataExpression>) {
        self.series_by_expression = Some(expression.into());
    }

    pub fn only(&mut self, filter: DocumentFilter) {
        self.filters.push(filter);
    }

    pub fn style(&mut self, style: PlotStyle) {
        self.style = style;
    }

    pub fn bins(&mut self, bins: usize) {
        self.histogram.bins = bins;
    }

    pub fn marker_record(&mut self, record: impl Into<RecordMarker>) {
        self.marker_record_types.push(record.into().into_record_type());
    }

    pub fn marker_records<I, R>(&mut self, records: I)
    where
        I: IntoIterator<Item = R>,
        R: Into<RecordMarker>,
    {
        self.marker_record_types.extend(records.into_iter().map(|record| record.into().into_record_type()));
    }

    pub fn reference_line(&mut self, label: impl Into<String>, quantity: impl Into<DataExpression>, line: Line) {
        let axis = self.axis_of(quantity.into());
        self.reference_lines.push(ReferenceLine {
            axis,
            value: line.value,
            source: line.source,
            source_scale: line.scale,
            label: Some(label.into()),
            color: default_reference_color(),
        });
    }

    pub fn reference_band(&mut self, label: impl Into<String>, quantity: impl Into<DataExpression>, band: Band) {
        let axis = self.axis_of(quantity.into());
        self.reference_bands.push(ReferenceBand {
            axis,
            min: band.min,
            max: band.max,
            center: band.center,
            tolerance: band.tolerance,
            label: Some(label.into()),
            color: default_reference_color(),
        });
    }

    fn axis_of(&self, quantity: DataExpression) -> ReferenceAxis {
        match self.x_expression.as_ref().is_some_and(|x| x.source == quantity.source && x.transforms == quantity.transforms) {
            true => ReferenceAxis::X,
            false => ReferenceAxis::Y,
        }
    }
}

impl Plot {
    pub fn xy_value_heatmap(label: impl Into<String>) -> PlotDefinition {
        PlotDefinition::with_type(label, PlotType::XyValueHeatmap)
    }

    pub fn wifi_channel_occupancy(label: impl Into<String>) -> PlotDefinition {
        let mut plot = PlotDefinition::with_type(label, PlotType::WifiChannelOccupancy);
        plot.stream = Some("host.wifi.channel_occupancy".to_owned());
        plot.x = Some("timestamp".to_owned());
        plot.y = vec!["network_count".to_owned()];
        plot.series_by = Some("wifi_channel".to_owned());
        plot.z_field = Some("wifi_network_congestion_score".to_owned());
        plot.style = PlotStyle::Points;

        plot
    }

    pub fn time_series(label: impl Into<String>) -> PlotDefinition {
        PlotDefinition::with_type(label, PlotType::TimeSeries)
    }

    pub fn bar(label: impl Into<String>) -> PlotDefinition {
        PlotDefinition::with_type(label, PlotType::Bar)
    }

    pub fn xy_scatter(label: impl Into<String>) -> PlotDefinition {
        PlotDefinition::with_type(label, PlotType::XyScatter)
    }

    pub fn xy_line(label: impl Into<String>) -> PlotDefinition {
        PlotDefinition::with_type(label, PlotType::XyLine)
    }

    pub fn sigma_interval(label: impl Into<String>) -> PlotDefinition {
        PlotDefinition::with_type(label, PlotType::XyInterval)
    }

    pub fn fft(label: impl Into<String>) -> PlotDefinition {
        PlotDefinition::with_type(label, PlotType::FrequencySeries)
    }

    pub fn histogram(label: impl Into<String>) -> PlotDefinition {
        PlotDefinition::with_type(label, PlotType::Histogram)
    }

    pub fn box_summary(label: impl Into<String>) -> PlotDefinition {
        PlotDefinition::with_type(label, PlotType::BoxSummary)
    }
}

impl Band {
    pub fn between(min: f64, max: f64) -> Self {
        Self { min: Some(min), max: Some(max), center: None, tolerance: None }
    }

    pub fn parameters(center: impl Into<String>, tolerance: impl Into<String>) -> Self {
        Self::around(crate::model::expressions::parameter(center), crate::model::expressions::parameter(tolerance))
    }

    pub fn around(center: impl Into<DataExpression>, tolerance: impl Into<DataExpression>) -> Self {
        Self { min: None, max: None, center: Some(center.into()), tolerance: Some(tolerance.into()) }
    }
}

impl Line {
    pub fn at(value: f64) -> Self {
        Self { value: Some(value), source: None, scale: default_reference_source_scale() }
    }

    pub fn parameter(name: impl Into<String>) -> Self {
        Self::expression(crate::model::expressions::parameter(name))
    }

    pub fn expression(source: impl Into<DataExpression>) -> Self {
        Self { value: None, source: Some(source.into()), scale: default_reference_source_scale() }
    }
}

impl DocumentTagged for DocumentDefinition {
    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.metadata
    }
}

impl DocumentTagged for DocumentSection {
    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.metadata
    }
}

impl DocumentTagged for DocumentCard {
    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.metadata
    }
}

impl DocumentTagged for PlotDefinition {
    fn metadata_mut(&mut self) -> &mut Metadata {
        &mut self.metadata
    }
}

/* === Public Functions === */

pub fn resolve_document_plot_axes(document: &mut DocumentDefinition, contracts: &[TelemetryContract]) {
    for card in &mut document.cards {
        resolve_card_body_plot_axis(&mut card.body, contracts);
    }
    for section in &mut document.sections {
        for card in &mut section.cards {
            resolve_card_body_plot_axis(&mut card.body, contracts);
        }
    }
}

pub fn validate_plot_definition(plot: &PlotDefinition) -> Result<(), String> {
    validate_plot_definition_with(plot, &crate::model::validation::diagnostic::text)
}

pub(crate) fn plot_validation_issue_bytes(plot: &PlotDefinition) -> Option<usize> {
    validate_plot_definition_with(plot, &crate::model::validation::diagnostic::size).err()
}

fn validate_plot_definition_with<E>(plot: &PlotDefinition, error: &impl Fn(std::fmt::Arguments<'_>) -> E) -> Result<(), E> {
    if matches!(
        plot.plot_type,
        PlotType::GroupedBar | PlotType::StackedBar | PlotType::Pie | PlotType::Donut | PlotType::Pareto | PlotType::AggregateTable
    ) {
        return Err(error(format_args!("unsupported document plot type {}", plot_type_label(&plot.plot_type))));
    }

    if plot.plot_type == PlotType::TimeSeries
        && (plot.x_expression.is_some() || plot.x.as_deref().is_some_and(|field| field != "timestamp"))
    {
        return Err(error(format_args!(
            "`time_series` plots draw against the row timestamp; remove `.x(…)`, or use \
             `Plot::xy_line` to choose both axes"
        )));
    }

    if plot.z_expression.as_ref().is_some_and(|expression| !expression.transforms.is_empty()) {
        return Err(error(format_args!("z expressions must select a row value without transforms")));
    }

    if let Some(x_format) = &plot.x_format {
        validate_value_format_with(x_format, error)?;
        if plot.plot_type == PlotType::TimeSeries && x_format.kind() == ValueFormatKind::Number {
            return Err(error(format_args!(
                "`x_format` on a time_series plot takes a time format: format::clock(…), \
                 format::elapsed_seconds(), format::elapsed_ms()"
            )));
        }
        if plot_has_number_x_axis(&plot.plot_type) && x_format.kind() == ValueFormatKind::Time {
            return Err(error(format_args!(
                "`x_format` on a {} plot takes a number format: format::normal(), \
                 format::fixed(…), format::engineering()",
                plot_type_label(&plot.plot_type)
            )));
        }
    }

    if let Some(y_format) = &plot.y_format {
        if plot.plot_type == PlotType::Histogram {
            return Err(error(format_args!(
                "`y_format` on a histogram plot has nothing to format: its y axis is the count \
                 the host computes; format the binned value with `x_format`"
            )));
        }
        validate_value_format_with(y_format, error)?;
        if y_format.kind() == ValueFormatKind::Time {
            return Err(error(format_args!("`y_format` takes a number format")));
        }
    }

    match plot.plot_type {
        PlotType::Bar => {
            if plot.x.is_none() && plot.x_expression.is_none() {
                return Err(error(format_args!("bar plots must declare an x field or x expression")));
            }
            if plot.y.is_empty() && plot.y_expressions.is_empty() {
                return Err(error(format_args!("bar plots must declare at least one y field or y expression")));
            }
        }
        PlotType::GroupedBar | PlotType::StackedBar | PlotType::Pareto | PlotType::XyScatter | PlotType::XyLine | PlotType::XyInterval => {
            validate_xy_axes(plot, error)?
        }
        PlotType::Trajectory => validate_xy_axes(plot, error)?,
        PlotType::Heatmap | PlotType::XyValueHeatmap => {
            validate_xy_axes(plot, error)?;
            if plot.z_field.as_ref().is_none_or(|field| field.is_empty()) && plot.z_expression.is_none() {
                return Err(error(format_args!("{} plots must declare a z field or z expression", plot_type_label(&plot.plot_type))));
            }
        }
        PlotType::WifiChannelOccupancy => validate_wifi_channel_occupancy(plot, error)?,
        PlotType::Histogram => validate_histogram_axes(plot, error)?,
        PlotType::BoxSummary | PlotType::Pie | PlotType::Donut => {
            if plot.y.len() != 1 && plot.y_expressions.len() != 1 {
                return Err(error(format_args!(
                    "{} plots must declare exactly one y field or y expression",
                    plot_type_label(&plot.plot_type)
                )));
            }
        }
        PlotType::TimeSeries | PlotType::FrequencySeries => {
            if plot.y.is_empty() && plot.y_expressions.is_empty() {
                return Err(error(format_args!(
                    "{} plots must declare at least one y field or y expression",
                    plot_type_label(&plot.plot_type)
                )));
            }
        }
        PlotType::AggregateTable => {}
    }
    if plot.histogram.bins < 2 {
        return Err(error(format_args!("histograms must declare at least two bins")));
    }
    if !plot.interval.bin_width.is_finite() || plot.interval.bin_width <= 0.0 {
        return Err(error(format_args!("interval bin_width must be greater than zero")));
    }
    if !plot.interval.sigma.is_finite() || plot.interval.sigma <= 0.0 {
        return Err(error(format_args!("interval sigma must be greater than zero")));
    }
    if plot.fft.sample_rate_hz.is_some_and(|value| !value.is_finite() || value <= 0.0) {
        return Err(error(format_args!("FFT sample_rate_hz must be greater than zero")));
    }
    if plot.fft.min_frequency_hz.is_some_and(|value| !value.is_finite() || value < 0.0)
        || plot.fft.max_frequency_hz.is_some_and(|value| !value.is_finite() || value < 0.0)
    {
        return Err(error(format_args!("FFT frequency bounds must be greater than or equal to zero")));
    }
    if let (Some(min), Some(max)) = (plot.fft.min_frequency_hz, plot.fft.max_frequency_hz)
        && min > max
    {
        return Err(error(format_args!("FFT min_frequency_hz must be less than or equal to max_frequency_hz")));
    }
    for band in &plot.reference_bands {
        if let (Some(min), Some(max)) = (band.min, band.max)
            && min > max
        {
            return Err(error(format_args!("reference band min must be less than or equal to max")));
        }
    }

    Ok(())
}

pub(crate) fn plot_type_label(plot_type: &PlotType) -> &'static str {
    match plot_type {
        PlotType::TimeSeries => "time_series",
        PlotType::Bar => "bar",
        PlotType::GroupedBar => "grouped_bar",
        PlotType::StackedBar => "stacked_bar",
        PlotType::Pie => "pie",
        PlotType::Donut => "donut",
        PlotType::Pareto => "pareto",
        PlotType::XyScatter => "xy_scatter",
        PlotType::XyLine => "xy_line",
        PlotType::XyInterval => "xy_interval",
        PlotType::Trajectory => "trajectory",
        PlotType::Heatmap => "heatmap",
        PlotType::XyValueHeatmap => "xy_value_heatmap",
        PlotType::WifiChannelOccupancy => "wifi_channel_occupancy",
        PlotType::FrequencySeries => "frequency_series",
        PlotType::Histogram => "histogram",
        PlotType::BoxSummary => "box_summary",
        PlotType::AggregateTable => "aggregate_table",
    }
}

/* === Private Functions === */

fn document_nodes_memory(nodes: &Vec<DocumentNode>, depth: usize) -> usize {
    if depth > 64 {
        return usize::MAX;
    }
    let mut bytes = std::mem::size_of_val(nodes).saturating_add(nodes.capacity().saturating_mul(std::mem::size_of::<DocumentNode>()));
    for node in nodes {
        let owned = document_node_memory(node, depth);
        if owned == usize::MAX {
            return usize::MAX;
        }
        bytes = bytes.saturating_add(owned.saturating_sub(std::mem::size_of::<DocumentNode>()));
    }
    bytes
}

fn document_node_memory(node: &DocumentNode, depth: usize) -> usize {
    if depth > 64 {
        return usize::MAX;
    }
    let heap = match node {
        DocumentNode::Section(section) => document_section_memory(section, depth),
        DocumentNode::Card(card) => {
            let owned = harmos_domain::MemorySize::memory_size(card);
            if owned == usize::MAX {
                return usize::MAX;
            }
            owned.saturating_sub(std::mem::size_of_val(card))
        }
    };
    std::mem::size_of::<DocumentNode>().saturating_add(heap)
}

fn document_section_memory(section: &DocumentSectionNode, depth: usize) -> usize {
    use harmos_domain::MemorySize;
    if depth > 64 {
        return usize::MAX;
    }
    let fields = [
        (section.node_id.memory_size(), std::mem::size_of_val(&section.node_id)),
        (section.metadata.memory_size(), std::mem::size_of_val(&section.metadata)),
        (section.repeat.memory_size(), std::mem::size_of_val(&section.repeat)),
        (document_nodes_memory(&section.children, depth + 1), std::mem::size_of_val(&section.children)),
    ];
    fields.into_iter().fold(std::mem::size_of::<DocumentSectionNode>(), |bytes, (owned, inline)| {
        if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
    })
}

fn resolve_card_body_plot_axis(body: &mut DocumentCardBody, contracts: &[TelemetryContract]) {
    if let DocumentCardBody::Plot { plot } = body {
        plot.resolve_x_axis(contracts);
    }
}

fn plot_has_number_x_axis(plot_type: &PlotType) -> bool {
    matches!(
        plot_type,
        PlotType::Bar
            | PlotType::Histogram
            | PlotType::GroupedBar
            | PlotType::StackedBar
            | PlotType::Pareto
            | PlotType::XyScatter
            | PlotType::XyLine
            | PlotType::XyInterval
            | PlotType::Trajectory
            | PlotType::Heatmap
            | PlotType::XyValueHeatmap
            | PlotType::FrequencySeries
    )
}

fn metadata(identifier_value: &str, label: String, description: Option<String>) -> Metadata {
    Metadata::new(Identifier::new(identifier_value).unwrap_or_else(|_| identifier_from_label(identifier_value)), label, description)
}

fn upsert_card_definition(cards: &mut Vec<DocumentCardDefinition>, card: DocumentCardDefinition) {
    if let Some(existing) = cards.iter_mut().find(|existing| existing.metadata.identifier == card.metadata.identifier) {
        *existing = card;
    } else {
        cards.push(card);
    }
}

fn first_section_node_index(layout: &DocumentLayout, section_id: &Identifier) -> Option<usize> {
    layout.nodes.iter().position(|node| matches!(node, DocumentNode::Section(section) if &section.node_id == section_id))
}

fn remove_section_node_from_layout(layout: &mut DocumentLayout, section_id: &Identifier) -> bool {
    remove_section_node(&mut layout.nodes, section_id)
}

fn remove_section_node(nodes: &mut Vec<DocumentNode>, section_id: &Identifier) -> bool {
    let before = nodes.len();
    nodes.retain(|node| !matches!(node, DocumentNode::Section(section) if &section.node_id == section_id));
    let mut removed = before != nodes.len();
    for node in nodes {
        if let DocumentNode::Section(section) = node {
            removed |= remove_section_node(&mut section.children, section_id);
        }
    }
    removed
}

fn section_node_mut<'a>(node: &'a mut DocumentNode, section_id: &Identifier) -> Option<&'a mut DocumentSectionNode> {
    match node {
        DocumentNode::Section(section) => {
            if &section.node_id == section_id {
                return Some(section);
            }
            section.children.iter_mut().find_map(|node| section_node_mut(node, section_id))
        }
        DocumentNode::Card(_) => None,
    }
}

fn upsert_card_placement(nodes: &mut Vec<DocumentNode>, placement: DocumentNode) {
    let DocumentNode::Card(new_card) = &placement else {
        return;
    };
    if let Some(existing) = nodes.iter_mut().find_map(|node| match node {
        DocumentNode::Card(card) if card.card_id == new_card.card_id => Some(card),
        _ => None,
    }) {
        *existing = new_card.clone();
    } else {
        nodes.push(placement);
    }
}

fn merge_layout_children(target: &mut Vec<DocumentNode>, source: Vec<DocumentNode>) {
    for child in source {
        match &child {
            DocumentNode::Section(section) => {
                if let Some(existing) = target.iter_mut().find_map(|node| match node {
                    DocumentNode::Section(existing) if existing.node_id == section.node_id => Some(existing),
                    _ => None,
                }) {
                    *existing = section.clone();
                } else {
                    target.push(child);
                }
            }
            DocumentNode::Card(card) => {
                if let Some(existing) = target.iter_mut().find_map(|node| match node {
                    DocumentNode::Card(existing) if existing.card_id == card.card_id => Some(existing),
                    _ => None,
                }) {
                    *existing = card.clone();
                } else {
                    target.push(child);
                }
            }
        }
    }
}

fn nodes_section_count(nodes: &[DocumentNode]) -> usize {
    nodes
        .iter()
        .map(|node| match node {
            DocumentNode::Section(section) => 1 + nodes_section_count(&section.children),
            DocumentNode::Card(_) => 0,
        })
        .sum()
}

fn identifier_from_label(value: &str) -> Identifier {
    let mut normalized = String::with_capacity(value.len());
    let mut last_was_separator = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            normalized.push(character);
            last_was_separator = false;
        } else if !last_was_separator {
            normalized.push('.');
            last_was_separator = true;
        }
    }
    let normalized = normalized.trim_matches('.');
    let identifier = if normalized.is_empty() { "document.item" } else { normalized };

    Identifier::new(identifier).expect("normalized document identifier is valid")
}

fn default_base_span() -> u8 {
    12
}

fn default_row_span() -> u8 {
    1
}

fn default_live_window_s() -> Option<f64> {
    Some(120.0)
}

fn default_interval_bin_width() -> f64 {
    1.0
}

fn default_interval_sigma() -> f64 {
    3.0
}

fn default_histogram_bins() -> usize {
    12
}

fn default_reference_color() -> String {
    "#60706a".to_owned()
}

fn default_reference_source_scale() -> f64 {
    1.0
}

fn default_true() -> bool {
    true
}

fn validate_xy_axes<E>(plot: &PlotDefinition, error: &impl Fn(std::fmt::Arguments<'_>) -> E) -> Result<(), E> {
    if plot.x.is_none() && plot.x_expression.is_none() {
        return Err(error(format_args!("XY plots must declare an x field or x expression")));
    }
    if plot.y.len() != 1 && plot.y_expressions.len() != 1 {
        return Err(error(format_args!("XY plots must declare exactly one y field or y expression")));
    }

    Ok(())
}

fn validate_histogram_axes<E>(plot: &PlotDefinition, error: &impl Fn(std::fmt::Arguments<'_>) -> E) -> Result<(), E> {
    if !plot.y.is_empty() || !plot.y_expressions.is_empty() {
        return Err(error(format_args!(
            "histogram plots take their value on the x axis and the host counts it on y; \
             replace `.y(…)` with `.x(…)`"
        )));
    }
    let x_field = plot.x.as_deref().filter(|field| *field != "timestamp");
    if usize::from(x_field.is_some()) + usize::from(plot.x_expression.is_some()) != 1 {
        return Err(error(format_args!("histogram plots must declare exactly one x field or x expression: the value they bin")));
    }

    Ok(())
}

fn validate_wifi_channel_occupancy<E>(plot: &PlotDefinition, error: &impl Fn(std::fmt::Arguments<'_>) -> E) -> Result<(), E> {
    if plot.x_expression.is_some() || !plot.y_expressions.is_empty() || plot.z_expression.is_some() || plot.series_by_expression.is_some() {
        return Err(error(format_args!("wifi_channel_occupancy uses fixed host columns; remove axis and grouping expressions")));
    }

    if plot.stream.as_deref() != Some("host.wifi.channel_occupancy") {
        return Err(error(format_args!("wifi_channel_occupancy plots must use stream host.wifi.channel_occupancy")));
    }
    if plot.x.as_deref() != Some("timestamp") {
        return Err(error(format_args!("wifi_channel_occupancy plots must use timestamp as x field")));
    }
    if plot.y.len() != 1 || plot.y[0] != "network_count" {
        return Err(error(format_args!("wifi_channel_occupancy plots must use network_count as y field")));
    }
    if plot.series_by.as_deref() != Some("wifi_channel") {
        return Err(error(format_args!("wifi_channel_occupancy plots must group by wifi_channel")));
    }

    Ok(())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_identifier_override() {
        let mut section = DocumentSection::new("Battery Charging");
        assert_eq!(section.metadata.identifier.as_str(), "battery.charging");

        section.id("obc.charger.charging");
        assert_eq!(section.metadata.identifier.as_str(), "obc.charger.charging");
        assert_eq!(section.metadata.label, "Battery Charging");
    }

    #[test]
    fn each_value_defaults() {
        let mut section = DocumentSection::new("Channels");
        let repeat = section.each(crate::model::expressions::telemetry("wifi", "channel"), "Channel {value}");
        assert_eq!(repeat.label_template, "Channel {value}");

        let repeat = section.repeat.as_ref().expect("each sets the repeat");
        assert_eq!(repeat.value, crate::model::expressions::telemetry("wifi", "channel"));
        assert_eq!(repeat.order_by, None);
        assert_eq!(repeat.id_template, None);
        assert!(repeat.filters.is_empty());
    }

    #[test]
    fn each_filter_chain() {
        let channel = crate::model::expressions::telemetry("wifi", "channel");
        let strength = crate::model::expressions::telemetry("wifi", "strength");

        let mut section = DocumentSection::new("Channels");
        section.each(channel.clone(), "Channel {value}").order_by(strength.clone()).only(channel.clone().ne(13)).id("wifi.channel.{value}");

        let repeat = section.repeat.expect("each sets the repeat");
        assert_eq!(repeat.order_by, Some(strength));
        assert_eq!(repeat.filters, vec![channel.ne(13)]);
        assert_eq!(repeat.id_template.as_deref(), Some("wifi.channel.{value}"));
    }

    #[test]
    fn comparison_filter_mapping() {
        let point = crate::model::expressions::telemetry("charger", "point");
        let cases = [
            (point.clone().eq("CP-03"), FilterOperator::Eq),
            (point.clone().ne("CP-03"), FilterOperator::Ne),
            (point.clone().gt(1), FilterOperator::Gt),
            (point.clone().gte(1), FilterOperator::Gte),
            (point.clone().lt(1), FilterOperator::Lt),
            (point.clone().lte(1), FilterOperator::Lte),
            (point.clone().contains("CP"), FilterOperator::Contains),
        ];

        for (filter, operator) in cases {
            assert_eq!(filter.expression, point);
            assert_eq!(filter.operator, operator);
            assert!(filter.value.is_some());
        }
        assert_eq!(point.clone().eq("CP-03").value, Some(Value::from("CP-03")));

        let exists = point.clone().exists();
        assert_eq!(exists.expression, point);
        assert_eq!(exists.operator, FilterOperator::Exists);
        assert_eq!(exists.value, None);
    }

    #[test]
    fn only_narrows_elements() {
        let point = crate::model::expressions::telemetry("charger", "point");
        let filter = point.clone().eq("CP-03");

        let mut plot = Plot::time_series("Current");
        plot.only(filter.clone());
        assert_eq!(plot.filters, vec![filter.clone()]);

        let mut card = DocumentCard::text("charger.note", "Note", "Charging");
        card.only(filter.clone());
        assert_eq!(card.filters, vec![filter.clone()]);

        let mut tile = TelemetrySummaryTileDefinition::new("Point", point);
        tile.only(filter.clone());
        assert_eq!(tile.filters, vec![filter]);
    }
}
