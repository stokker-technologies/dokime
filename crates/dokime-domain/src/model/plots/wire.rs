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
use serde::Deserialize;
use serde::Serialize;

// Local Imports
#[allow(unused_imports)]
use crate::model::catalog::wire::*;

#[allow(unused_imports)]
use crate::model::definition::wire::*;
use crate::model::document::DownsampleTechnique;

#[allow(unused_imports)]
use crate::model::identity::wire::*;
use crate::model::values::wire::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotSnapshotsReply {
    pub envelopes: Vec<PlotEnvelope>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotRequest {
    pub plot_id: Option<String>,
    pub plot_type: PlotType,
    pub x_scale: AxisScale,
    pub y_scale: AxisScale,
    pub stream: String,
    pub y_keys: Vec<String>,
    pub marker_record_types: Vec<String>,
    pub filters: Vec<PlotFilter>,
    pub x_key: Option<String>,
    pub series_by: Option<String>,
    pub x_expression: Option<DataExpression>,
    pub y_expressions: Vec<DataExpression>,
    pub series_by_expression: Option<DataExpression>,
    pub z_stream: Option<String>,
    pub z_field: Option<String>,
    pub unit: Option<String>,
    pub max_points: Option<u64>,
    pub window_s: Option<f64>,
    pub scope: Option<EvidenceScope>,
    pub technique: DownsampleTechnique,
    pub z_expression: Option<DataExpression>,
    pub fft: Option<PlotFftOptions>,
    pub interval: Option<PlotIntervalOptions>,
    pub histogram: Option<PlotHistogramOptions>,
    pub analysis: Option<AnalysisSelection>,

    #[row(
        description = "The x range a time series or scatter plot shows after a zoom; its series summaries cover only the rows inside it."
    )]
    pub summary_window: Option<PlotWindow>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotFilter {
    pub expression: Option<DataExpression>,
    pub operator: FilterOperator,

    #[row(description = "Absent means the Exists operator. A present Value may explicitly be null.")]
    pub value: Option<Value>,
}

// Data expressions are deliberately structural. They cover the expression
// forms accepted by document plot descriptors without embedding JSON or a
// language-specific serialized AST.
#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DataExpression {
    pub source: Option<DataExpressionSource>,
    pub transforms: Vec<DataTransform>,
    pub label: Option<String>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryDataSource {
    pub stream: String,
    pub field: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterDataSource {
    pub name: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionDataSource {
    pub name: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordDataSource {
    pub record_type: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactDataSource {
    pub artifact: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DataTransform {
    pub kind: Option<DataTransformKind>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct EmptyTransform {}

// One member of the Harmos aggregate vocabulary as its enum message; a
// percentile rank above 100 is refused.
#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AggregateTransform {
    pub aggregate: harmos_domain::Aggregate,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DetailTransform {
    pub name: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeBetweenTransform {
    pub previous_record_type: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryAtOrBeforeTransform {
    pub stream: String,
    pub field: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotEnvelope {
    pub plot_id: String,
    pub revision: u64,
    pub epoch: String,
    pub selection: String,
    pub payload: Option<PlotPayload>,
    pub diagnostics: Option<PlotPayloadDiagnostics>,
    pub warnings: Vec<String>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotStreamFrame {
    pub kind: PlotStreamKind,
    pub envelope: Option<PlotEnvelope>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotPayload {
    pub stream: String,
    pub x_key: String,
    pub series_by: Option<String>,
    pub context: Option<Value>,
    pub series: Vec<PlotSeries>,
    pub cartesian: Option<PlotCartesianFrame>,
    pub markers: Vec<PlotMarker>,
    pub max_points: u64,
    pub window_s: Option<f64>,
    pub generated_at: String,
    pub x_format: Option<ValueFormat>,

    #[row(description = "RFC3339, the origin `elapsed_since_run` counts from. Absent when this payload was not built for one execution.")]
    pub run_started_at: Option<String>,

    #[row(description = "RFC3339, the origin `elapsed_since_cycle` counts from.")]
    pub cycle_started_at: Option<String>,

    pub analysis: Option<PlotAnalysis>,
    pub computed: Option<AnalysisComputation>,

    #[row(
        description = "One summary per series, in series order, for time series and scatter plots: the statistics of the exact rows in the shown window."
    )]
    pub summaries: Vec<PlotSeriesSummary>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotSeriesSummary {
    pub series_id: String,
    pub count: u64,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub mean: Option<f64>,
    pub median: Option<f64>,
    pub std: Option<f64>,
    pub rms: Option<f64>,
    pub window: Option<PlotWindow>,
}

// An inclusive x range in the cartesian x space: epoch seconds on a time
// axis, x values on a scatter.
#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotWindow {
    pub from: f64,
    pub to: f64,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotCartesianFrame {
    pub mode: PlotCartesianMode,
    pub series: Vec<PlotCartesianSeriesFrame>,
    pub x_domain: Option<PlotDomain>,
    pub y_domain: Option<PlotDomain>,
    pub point_count: u64,
    pub signature: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotCartesianSeriesFrame {
    pub x: Vec<f64>,

    #[row(description = "A message wrapper preserves null y values without a NaN sentinel.")]
    pub y: Vec<NullableNumber>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct NullableNumber {
    pub value: Option<f64>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotDomain {
    pub min: f64,
    pub max: f64,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotPayloadDiagnostics {
    pub series_count: u64,
    pub point_count: u64,
    pub raw_point_count: u64,
    pub downsampled: bool,
    pub max_points: u64,
    pub window_s: Option<f64>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotSeries {
    pub key: String,
    pub label: String,
    pub unit: Option<String>,
    pub points: Vec<PlotPoint>,
    pub raw_points: u64,
    pub downsampled: bool,
    pub format: Option<ValueFormat>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotPoint {
    pub x: Option<Value>,
    pub y: Option<f64>,
    pub z: Option<f64>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotMarker {
    pub x: Option<Value>,
    pub label: String,
    pub record_type: String,
    pub color: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotFftOptions {
    pub sample_rate_hz: Option<f64>,
    pub window: FftWindow,
    pub detrend: FftDetrend,
    pub level_unit: FrequencyLevelUnit,
    pub include_dc: bool,
    pub min_frequency_hz: Option<f64>,
    pub max_frequency_hz: Option<f64>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotIntervalOptions {
    pub bin_width: f64,
    pub sigma: f64,
    pub clip_to_observed: bool,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotHistogramOptions {
    pub bins: u64,
    pub unit: Option<String>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotAnalysis {
    pub result: Option<PlotAnalysisResult>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotAnalysisError {
    pub message: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotHistogramAnalysis {
    pub series: Vec<PlotHistogramSeries>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotHistogramSeries {
    pub key: String,
    pub bins: Vec<PlotHistogramBin>,
    pub count: u64,
    pub statistics: Option<PlotHistogramStatistics>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotHistogramStatistics {
    pub mean: f64,
    pub median: f64,
    pub std: f64,
    pub rms: f64,
    pub min: f64,
    pub max: f64,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotHistogramBin {
    pub lower: f64,
    pub upper: f64,
    pub count: u64,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotBoxSummaryAnalysis {
    pub series: Vec<PlotBoxSummary>,

    #[row(description = "Samples summarized across every box.")]
    pub total: u64,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotBoxSummary {
    pub key: String,
    pub count: u64,
    pub min: f64,
    pub q1: f64,
    pub median: f64,
    pub q3: f64,
    pub max: f64,
    pub lower_whisker: f64,
    pub upper_whisker: f64,
    pub outliers: Vec<f64>,
    pub label: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotSigmaIntervalAnalysis {
    pub series: Vec<PlotSigmaSeries>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotSigmaSeries {
    pub key: String,
    pub bins: Vec<PlotSigmaBin>,

    #[row(description = "Samples the bins summarize: every row with a finite x and y.")]
    pub raw_points: u64,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotSigmaBin {
    pub x: f64,
    pub count: u64,
    pub mean: f64,
    pub lower: f64,
    pub upper: f64,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotWifiOccupancyAnalysis {
    #[row(description = "Every channel a series reported, ascending.")]
    pub channels: Vec<i64>,

    #[row(description = "Channels that held a network in any scan, ascending.")]
    pub occupied_channels: Vec<i64>,

    pub bands: Vec<PlotWifiBandWindow>,

    #[row(description = "Scans in time order.")]
    pub scans: Vec<PlotWifiScan>,
}

// Channels of one band that held a network in any scan of the window.
#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotWifiBandWindow {
    pub band: String,
    pub occupied_channels: u64,
}

// One scan: the readings sharing a whole second, a channel keeping its
// largest reading.
#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotWifiScan {
    pub time: String,
    pub channels: Vec<PlotWifiChannelCount>,
    pub bands: Vec<PlotWifiBandScan>,
    pub occupied_channels: u64,
    pub total_networks: u64,
    pub peak_count: u64,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotWifiChannelCount {
    pub channel: i64,
    pub count: u64,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotWifiBandScan {
    pub band: String,
    pub occupied_channels: u64,
    pub total_networks: u64,
    pub peak_count: u64,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListAnalysesReply {
    pub definitions: Vec<AnalysisDefinition>,
    pub next_after_id: Option<String>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisDefinition {
    pub id: String,
    pub version: u32,
    pub label: String,
    pub description: String,
    pub parameters: Vec<ParameterSchema>,
    pub outputs: Vec<AnalysisOutput>,
    pub max_samples: u32,
    pub max_output_values: u32,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisOutput {
    pub key: String,
    pub label: String,
    pub unit: Option<String>,
    pub axis: Option<AnalysisAxis>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisAxis {
    pub key: String,
    pub unit: Option<String>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisSelection {
    pub id: String,
    pub version: u32,
    pub parameters: Option<Value>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisComputation {
    pub definition: Option<AnalysisDefinition>,
    pub selection: Option<AnalysisSelection>,
    pub source: Option<AnalysisEvidence>,
    pub series: Vec<AnalysisSeries>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisEvidence {
    pub store: String,
    pub epoch: String,
    pub position: u64,
    pub stream: String,
    pub scope: Option<EvidenceScope>,
    pub window_s: Option<f64>,
    pub fields: Vec<String>,
    pub x_key: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisSeries {
    pub key: String,
    pub label: String,
    pub origin: Option<String>,
    pub input_samples: u64,
    pub first_axis: f64,
    pub last_axis: f64,
    pub outputs: Vec<AnalysisValues>,
}

// Numeric Payloads Stay Numeric On The Wire, Including Custom Results.
#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisValues {
    pub key: String,
    pub unit: String,
    pub axis_unit: String,
    pub axis: Vec<f64>,
    pub values: Vec<f64>,

    #[row(description = "Optional display selection into the complete scientific arrays.")]
    pub display_indices: Vec<u32>,
}

/* === Enums === */

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlotType {
    Unspecified,
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

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AxisScale {
    Unspecified,
    Linear,
    Log,
}

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterOperator {
    Unspecified,
    Eq,
    Ne,
    Gt,
    Gte,
    Lt,
    Lte,
    Contains,
    Exists,
}

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlotStreamKind {
    Unspecified,
    Snapshot,
    Replace,
    Error,
}

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlotCartesianMode {
    Unspecified,
    Aligned,
    Faceted,
}

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FftWindow {
    Unspecified,
    Hann,
    Rectangular,
}

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FftDetrend {
    Unspecified,
    None,
    Mean,
}

#[harmos_domain::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrequencyLevelUnit {
    Unspecified,
    Linear,
    Db,
    Dbu,
    Dbm,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum DataExpressionSource {
    Telemetry(TelemetryDataSource),
    Parameter(ParameterDataSource),
    Dimension(DimensionDataSource),
    Record(RecordDataSource),
    Artifact(ArtifactDataSource),
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum DataTransformKind {
    Aggregate(AggregateTransform),
    TimeBetween(TimeBetweenTransform),
    TelemetryAtOrBefore(TelemetryAtOrBeforeTransform),
    Detail(DetailTransform),
    PerCase(EmptyTransform),
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum PlotAnalysisResult {
    Histogram(PlotHistogramAnalysis),
    BoxSummary(PlotBoxSummaryAnalysis),
    SigmaInterval(PlotSigmaIntervalAnalysis),
    Error(PlotAnalysisError),
    WifiOccupancy(PlotWifiOccupancyAnalysis),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
