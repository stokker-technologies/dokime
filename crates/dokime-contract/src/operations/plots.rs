/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod list_analyses;
mod query_snapshots;
mod watch;

/* === Re-Exports === */
pub use list_analyses::ListAnalyses;
pub use query_snapshots::QuerySnapshots;
pub use watch::Watch;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
#[allow(unused_imports)]
use super::catalog::*;
#[allow(unused_imports)]
use super::common::*;
#[allow(unused_imports)]
use super::schema::*;
use super::value::Value;
use super::{Access, Endpoint, FOLLOW, ORDINARY};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// The native bridge and server reject a request with more than 32 plots. The
// reply contains repeated envelopes in one bounded message; callers must
// lower max_points or split a batch on exhaustion.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotSnapshotsRequest {
    pub requests: Vec<PlotRequest>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotSnapshotsReply {
    pub envelopes: Vec<PlotEnvelope>,
}

// Watch is server-streaming. To change the request set, cancel this stream and
// open another one. Requests without a cursor receive an initial Snapshot
// before the watch begins; later frames are coalesced by the server at 100 ms
// only after a runtime state or telemetry watch advances.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotWatchRequest {
    pub requests: Vec<PlotRequest>,
}

#[harmos::message]
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
    pub cursor: Option<PlotCursor>,
    pub technique: DownsampleTechnique,
    pub z_expression: Option<DataExpression>,
    pub fft: Option<PlotFftOptions>,
    pub interval: Option<PlotIntervalOptions>,
    pub histogram: Option<PlotHistogramOptions>,
    pub analysis: Option<AnalysisSelection>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotFilter {
    pub expression: Option<DataExpression>,
    pub operator: FilterOperator,
    // Absent means the Exists operator. A present Value may explicitly be null.
    #[row(
        description = "Absent means the Exists operator. A present Value may explicitly be null."
    )]
    pub value: Option<Value>,
}

// Data expressions are deliberately structural. They cover the expression
// forms accepted by document plot descriptors without embedding JSON or a
// language-specific serialized AST.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DataExpression {
    pub source: Option<DataExpressionSource>,
    pub transforms: Vec<DataTransform>,
    pub label: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryDataSource {
    pub stream: String,
    pub field: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ParameterDataSource {
    pub name: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DimensionDataSource {
    pub name: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordDataSource {
    pub record_type: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactDataSource {
    pub artifact: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DataTransform {
    pub kind: Option<DataTransformKind>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct EmptyTransform {}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DetailTransform {
    pub name: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TimeBetweenTransform {
    pub previous_record_type: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryAtOrBeforeTransform {
    pub stream: String,
    pub field: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotCursorTelemetryStreamsEntry {
    pub key: String,
    pub value: u64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotCursor {
    pub telemetry_streams: Vec<PlotCursorTelemetryStreamsEntry>,
    pub record_sequence: Option<u64>,
    // Cursors are process-local. The backend rejects a cursor from another
    // runtime epoch or another plot selection instead of silently resuming it.
    #[row(
        description = "Cursors are process-local. The backend rejects a cursor from another runtime epoch or another plot selection instead of silently resuming it."
    )]
    pub epoch: String,
    pub selection: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotEnvelope {
    pub plot_id: String,
    pub revision: u64,
    pub cursor: Option<PlotCursor>,
    pub payload: Option<PlotPayload>,
    pub diagnostics: Option<PlotPayloadDiagnostics>,
    pub warnings: Vec<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotStreamFrame {
    pub kind: PlotStreamKind,
    pub envelope: Option<PlotEnvelope>,
}

#[harmos::message]
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
    // RFC3339, the origin `elapsed_since_run` counts from. Absent when this
    // payload was not built for one execution.
    #[row(
        description = "RFC3339, the origin `elapsed_since_run` counts from. Absent when this payload was not built for one execution."
    )]
    pub run_started_at: Option<String>,
    // RFC3339, the origin `elapsed_since_cycle` counts from.
    #[row(description = "RFC3339, the origin `elapsed_since_cycle` counts from.")]
    pub cycle_started_at: Option<String>,
    pub analysis: Option<PlotAnalysis>,
    pub computed: Option<AnalysisComputation>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotCartesianFrame {
    pub mode: PlotCartesianMode,
    pub series: Vec<PlotCartesianSeriesFrame>,
    pub x_domain: Option<PlotDomain>,
    pub y_domain: Option<PlotDomain>,
    pub point_count: u64,
    pub signature: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotCartesianSeriesFrame {
    pub x: Vec<f64>,
    // A message wrapper preserves null y values without a NaN sentinel.
    #[row(description = "A message wrapper preserves null y values without a NaN sentinel.")]
    pub y: Vec<NullableNumber>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct NullableNumber {
    pub value: Option<f64>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotDomain {
    pub min: f64,
    pub max: f64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotPayloadDiagnostics {
    pub series_count: u64,
    pub point_count: u64,
    pub raw_point_count: u64,
    pub downsampled: bool,
    pub delta: bool,
    pub max_points: u64,
    pub window_s: Option<f64>,
}

#[harmos::message]
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

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotPoint {
    pub x: Option<Value>,
    pub y: Option<f64>,
    pub z: Option<f64>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotMarker {
    pub x: Option<Value>,
    pub label: String,
    pub record_type: String,
    pub color: String,
}

#[harmos::message]
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

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotIntervalOptions {
    pub bin_width: f64,
    pub sigma: f64,
    pub clip_to_observed: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotHistogramOptions {
    pub bins: u64,
    pub unit: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotAnalysis {
    pub result: Option<PlotAnalysisResult>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotAnalysisError {
    pub message: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotHistogramAnalysis {
    pub series: Vec<PlotHistogramSeries>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotHistogramSeries {
    pub key: String,
    pub bins: Vec<PlotHistogramBin>,
    pub count: u64,
    pub statistics: Option<PlotHistogramStatistics>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotHistogramStatistics {
    pub mean: f64,
    pub median: f64,
    pub std: f64,
    pub rms: f64,
    pub min: f64,
    pub max: f64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotHistogramBin {
    pub lower: f64,
    pub upper: f64,
    pub count: u64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotBoxSummaryAnalysis {
    pub series: Vec<PlotBoxSummary>,
}

#[harmos::message]
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

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotSigmaIntervalAnalysis {
    pub series: Vec<PlotSigmaSeries>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotSigmaSeries {
    pub key: String,
    pub bins: Vec<PlotSigmaBin>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PlotSigmaBin {
    pub x: f64,
    pub count: u64,
    pub mean: f64,
    pub lower: f64,
    pub upper: f64,
}

// Analysis IDs Are Open; Each Contribution Declares Its Parameters And Outputs.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListAnalysesRequest {
    pub limit: u32,
    pub after_id: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListAnalysesReply {
    pub definitions: Vec<AnalysisDefinition>,
    pub next_after_id: Option<String>,
}

#[harmos::message]
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

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisOutput {
    pub key: String,
    pub label: String,
    pub unit: Option<String>,
    pub axis: Option<AnalysisAxis>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisAxis {
    pub key: String,
    pub unit: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisSelection {
    pub id: String,
    pub version: u32,
    pub parameters: Option<Value>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisComputation {
    pub definition: Option<AnalysisDefinition>,
    pub selection: Option<AnalysisSelection>,
    pub source: Option<AnalysisEvidence>,
    pub series: Vec<AnalysisSeries>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisEvidenceTelemetryStreamsEntry {
    pub key: String,
    pub value: u64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisEvidence {
    pub epoch: String,
    pub stream: String,
    pub scope: Option<EvidenceScope>,
    pub telemetry_streams: Vec<AnalysisEvidenceTelemetryStreamsEntry>,
    pub record_sequence: Option<u64>,
    pub window_s: Option<f64>,
    pub fields: Vec<String>,
    pub x_key: String,
}

#[harmos::message]
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
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AnalysisValues {
    pub key: String,
    pub unit: String,
    pub axis_unit: String,
    pub axis: Vec<f64>,
    pub values: Vec<f64>,
    // Optional display selection into the complete scientific arrays.
    #[row(description = "Optional display selection into the complete scientific arrays.")]
    pub display_indices: Vec<u32>,
}

/* === Enums === */
#[harmos::message]
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

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AxisScale {
    Unspecified,
    Linear,
    Log,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DownsampleTechnique {
    Unspecified,
    Raw,
    MinMax,
    Lttb,
}

#[harmos::message]
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

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlotStreamKind {
    Unspecified,
    Snapshot,
    Append,
    Replace,
    Error,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlotCartesianMode {
    Unspecified,
    Aligned,
    Faceted,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FftWindow {
    Unspecified,
    Hann,
    Rectangular,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FftDetrend {
    Unspecified,
    None,
    Mean,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrequencyLevelUnit {
    Unspecified,
    Linear,
    Db,
    Dbu,
    Dbm,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum DataExpressionSource {
    Telemetry(TelemetryDataSource),
    Parameter(ParameterDataSource),
    Dimension(DimensionDataSource),
    Record(RecordDataSource),
    Artifact(ArtifactDataSource),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum DataTransformKind {
    Count(EmptyTransform),
    Mean(EmptyTransform),
    Median(EmptyTransform),
    Min(EmptyTransform),
    Max(EmptyTransform),
    TimeBetween(TimeBetweenTransform),
    TelemetryAtOrBefore(TelemetryAtOrBeforeTransform),
    Detail(DetailTransform),
    PerCase(EmptyTransform),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum PlotAnalysisResult {
    Histogram(PlotHistogramAnalysis),
    BoxSummary(PlotBoxSummaryAnalysis),
    SigmaInterval(PlotSigmaIntervalAnalysis),
    Error(PlotAnalysisError),
}

#[harmos::catalog]
pub enum Operations {
    ListAnalyses(ListAnalyses),
    QuerySnapshots(QuerySnapshots),
    Watch(Watch),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for ListAnalyses {
    const PATH: &'static str = "/dokime.v1.Plots/ListAnalyses";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for QuerySnapshots {
    const PATH: &'static str = "/dokime.v1.Plots/QuerySnapshots";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for Watch {
    const PATH: &'static str = "/dokime.v1.Plots/Watch";
    const ACCESS: Access = Access::Read;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
