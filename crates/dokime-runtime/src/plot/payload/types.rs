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
pub use crate::document::DownsampleTechnique;

/* === Imports === */
// Standard Library Imports
use std::collections::BTreeMap;
// Third Party Imports
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
// Local Imports
use crate::document::{AxisScale, DocumentFilter, FftOptions, HistogramOptions, PlotType, SigmaIntervalOptions};
use crate::expressions::DataExpression;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::values::Value;
use dokime_domain::model::values::ValueFormat;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PlotPayloadRequest {
    #[serde(default)]
    pub analysis: Option<crate::analysis::Selection>,
    #[serde(default)]
    pub plot_id: Option<String>,
    #[serde(default)]
    pub plot_type: Option<PlotType>,
    #[serde(default)]
    pub x_scale: AxisScale,
    #[serde(default)]
    pub y_scale: AxisScale,
    pub stream: String,
    #[serde(default)]
    pub y_keys: Vec<String>,
    #[serde(default)]
    pub marker_record_types: Vec<String>,
    #[serde(default)]
    pub filters: Vec<DocumentFilter>,
    pub x_key: Option<String>,
    pub series_by: Option<String>,
    #[serde(default)]
    pub x_expression: Option<DataExpression>,
    #[serde(default)]
    pub y_expressions: Vec<DataExpression>,
    #[serde(default)]
    pub series_by_expression: Option<DataExpression>,
    #[serde(default)]
    pub z_expression: Option<DataExpression>,
    #[serde(default)]
    pub fft: FftOptions,
    #[serde(default)]
    pub interval: SigmaIntervalOptions,
    #[serde(default)]
    pub histogram: HistogramOptions,
    pub z_stream: Option<String>,
    pub z_field: Option<String>,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub x_format: Option<ValueFormat>,
    #[serde(default)]
    pub y_format: Option<ValueFormat>,
    pub max_points: Option<usize>,
    #[serde(default, deserialize_with = "deserialize_window_s")]
    // Validation preserves the nullable JSON number representation.
    pub window_s: Option<f64>,
    pub scope: Option<RecordScope>,
    #[serde(default)]
    pub cursor: Option<PlotPayloadCursor>,
    #[serde(default)]
    pub technique: DownsampleTechnique,
    #[serde(default)]
    pub summary_window: Option<PlotWindow>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PlotPayloadCursor {
    #[serde(default)]
    pub telemetry_streams: BTreeMap<String, u64>,
    #[serde(default)]
    pub record_sequence: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PlotPayloadEnvelope {
    pub plot_id: String,
    pub revision: u64,
    pub cursor: PlotPayloadCursor,
    pub payload: PlotPayload,
    pub diagnostics: PlotPayloadDiagnostics,
    pub warnings: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PlotPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub computed: Option<PlotComputation>,
    pub stream: String,
    pub x_key: String,
    pub series_by: Option<String>,
    pub context: BTreeMap<String, Value>,
    pub series: Vec<PlotSeries>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cartesian: Option<PlotCartesianFrame>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analysis: Option<super::analyses::PlotAnalysis>,
    pub markers: Vec<PlotMarker>,
    pub max_points: usize,
    pub window_s: Option<f64>,
    pub generated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub x_format: Option<ValueFormat>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run_started_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cycle_started_at: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub summaries: Vec<PlotSeriesSummary>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PlotSeriesSummary {
    pub series_id: String,
    pub count: usize,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub mean: Option<f64>,
    pub median: Option<f64>,
    pub std: Option<f64>,
    pub rms: Option<f64>,
    pub window: Option<PlotWindow>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct PlotWindow {
    pub from: f64,
    pub to: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlotComputation {
    pub definition: crate::analysis::Definition,
    pub selection: crate::analysis::Selection,
    pub source: PlotEvidence,
    pub series: Vec<PlotComputedSeries>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlotEvidence {
    pub epoch: String,
    pub stream: String,
    pub scope: Option<RecordScope>,
    #[serde(serialize_with = "serialize_analysis_cursor")]
    pub cursor: PlotPayloadCursor,
    pub window_s: Option<f64>,
    pub fields: Vec<String>,
    pub x_key: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct PlotComputedSeries {
    pub key: String,
    pub label: String,
    pub origin: Option<String>,
    pub input_samples: usize,
    pub first_axis: f64,
    pub last_axis: f64,
    pub output: crate::analysis::Response,
    pub display_indices: BTreeMap<String, Vec<u32>>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PlotCartesianFrame {
    pub mode: PlotCartesianMode,
    pub series: Vec<PlotCartesianSeriesFrame>,
    pub x_domain: Option<PlotDomain>,
    pub y_domain: Option<PlotDomain>,
    pub point_count: usize,
    pub signature: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PlotCartesianSeriesFrame {
    pub x: Vec<f64>,
    pub y: Vec<Option<f64>>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PlotDomain {
    pub min: f64,
    pub max: f64,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PlotPayloadDiagnostics {
    pub series_count: usize,
    pub point_count: usize,
    pub raw_point_count: usize,
    pub downsampled: bool,
    pub delta: bool,
    pub max_points: usize,
    pub window_s: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PlotSeries {
    pub key: String,
    pub label: String,
    pub unit: Option<String>,
    pub points: Vec<PlotPoint>,
    pub raw_points: usize,
    pub downsampled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub format: Option<ValueFormat>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PlotPoint {
    // Both native value types encode the same recursive JSON vocabulary.
    pub x: JsonValue,
    pub y: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub z: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PlotMarker {
    // Both native value types encode the same recursive JSON vocabulary.
    pub x: JsonValue,
    pub label: String,
    pub record_type: String,
    pub color: String,
}

/* === Enums === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlotPayloadStreamKind {
    Snapshot,
    Append,
    Replace,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlotCartesianMode {
    Aligned,
    Faceted,
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
/* === Functions === */

fn deserialize_window_s<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let window_s = Option::<f64>::deserialize(deserializer)?;
    if let Some(window_s) = window_s
        && super::plot_window_duration(window_s).is_none()
    {
        return Err(serde::de::Error::custom("window_s must be a finite, non-negative duration"));
    }
    Ok(window_s)
}

/* === Tests === */

fn serialize_analysis_cursor<S: serde::Serializer>(cursor: &PlotPayloadCursor, serializer: S) -> Result<S::Ok, S::Error> {
    #[derive(Serialize)]
    struct Cursor {
        telemetry_streams: BTreeMap<String, String>,
        record_sequence: Option<String>,
    }
    Cursor {
        telemetry_streams: cursor.telemetry_streams.iter().map(|(key, value)| (key.clone(), value.to_string())).collect(),
        record_sequence: cursor.record_sequence.map(|value| value.to_string()),
    }
    .serialize(serializer)
}
