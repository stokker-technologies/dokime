/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

//! A plot built from evidence read one page at a time.
//!
//! Each admitted sample is reduced on arrival to the few values the plot reads
//! from it, so a plot over a long capture holds compact rows rather than the
//! samples. The rows then pass through exactly the series construction and
//! declared reduction the slice-based builder applies.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::collections::BTreeMap;
// Third Party Imports
use serde_json::Value as JsonValue;
use time::OffsetDateTime;
// Local Imports
use super::{
    DEFAULT_PLOT_MAX_POINTS, PlotPayload, PlotPayloadContext, PlotPayloadRequest, PlotPoint,
    PlotSeries, ZIndex, compare_series_group, expression_field_name, expression_group_value,
    expression_numeric_value, expression_x_value, finish_plot_payload, finish_plot_series,
    group_value, is_row_expression, numeric_value, plot_markers, retain_window,
    sample_matches_filters, sample_matches_scope, sample_value, stream_matches_alias, timestamp,
    x_sort_value, x_value,
};
use crate::document::PlotType;
use crate::records::RecordEntry;
use dokime::core::Timestamp;
use dokime::expressions::DataSource;
use dokime::telemetry::TelemetrySample;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// A plot whose series are accumulated from evidence pages.
pub(crate) struct StreamedPlot {
    request: PlotPayloadRequest,
    x_key: String,
    z_index: ZIndex,
    series: Series,
    rows: Vec<Row>,
    bytes: usize,
}

/// What one admitted sample contributes to the plot.
struct Row {
    timestamp: Timestamp,
    time: Option<OffsetDateTime>,
    sort: f64,
    x: JsonValue,
    ys: Vec<Option<f64>>,
    group: Option<String>,
    z: Option<f64>,
}

/* === Enums === */
/// Which per-sample series family the request selects.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Series {
    Keys,
    Expressions,
}

/* === Traits === */

/* === Implementations === */
impl StreamedPlot {
    /// Accepts the request when every series it draws is computed per sample
    /// from its own stream.
    pub(crate) fn new(request: PlotPayloadRequest) -> Option<Self> {
        let whole_capture = request.analysis.is_some()
            || matches!(
                request.plot_type,
                Some(
                    PlotType::FrequencySeries
                        | PlotType::Trajectory
                        | PlotType::WifiChannelOccupancy
                )
            );
        let series = if request.y_expressions.is_empty() {
            Series::Keys
        } else if request.y_expressions.iter().all(is_row_expression) {
            Series::Expressions
        } else {
            return None;
        };
        let z_index = per_sample_z(&request)?;
        if whole_capture {
            return None;
        }
        let x_key = request
            .x_key
            .clone()
            .unwrap_or_else(|| "timestamp".to_owned());
        Some(Self {
            request,
            x_key,
            z_index,
            series,
            rows: Vec::new(),
            bytes: std::mem::size_of::<Self>(),
        })
    }

    pub(crate) fn push(&mut self, sample: &TelemetrySample) {
        let request = &self.request;
        if sample.stream.as_str() != request.stream
            || !sample_matches_scope(sample.scope.as_ref(), request.scope.as_ref())
            || !sample_matches_filters(sample, &request.filters)
        {
            return;
        }
        let (x, ys, group) = match self.series {
            Series::Keys => (
                x_value(sample, &self.x_key),
                request
                    .y_keys
                    .iter()
                    .map(|key| sample.values.get(key).and_then(numeric_value))
                    .collect::<Vec<_>>(),
                request
                    .series_by
                    .as_deref()
                    .and_then(|series_by| sample_value(sample, series_by).and_then(group_value)),
            ),
            Series::Expressions => (
                expression_x_value(sample, request.x_expression.as_ref(), &self.x_key),
                request
                    .y_expressions
                    .iter()
                    .map(|expression| expression_numeric_value(sample, expression))
                    .collect::<Vec<_>>(),
                request
                    .series_by_expression
                    .as_ref()
                    .and_then(|series_by| expression_group_value(sample, series_by)),
            ),
        };
        let row = Row {
            timestamp: sample.timestamp.clone(),
            time: timestamp(sample),
            sort: x_sort_value(sample, &self.x_key),
            x,
            ys,
            group,
            z: self.z_index.resolve(sample),
        };
        self.bytes = self.bytes.saturating_add(row.memory());
        self.rows.push(row);
    }

    /// Bytes the accumulated rows hold, charged against the capture allowance.
    pub(crate) fn memory_size(&self) -> usize {
        self.bytes.saturating_add(
            self.rows
                .capacity()
                .saturating_sub(self.rows.len())
                .saturating_mul(std::mem::size_of::<Row>()),
        )
    }

    pub(crate) fn finish(
        self,
        records: &[RecordEntry],
        generated_at: &str,
        format_context: &PlotPayloadContext,
    ) -> PlotPayload {
        let Self {
            request,
            x_key,
            series,
            mut rows,
            ..
        } = self;
        // The same order and window the slice builder applies to its samples.
        rows.sort_by(|left, right| left.timestamp.cmp(&right.timestamp));
        retain_window(&mut rows, request.window_s, |row| row.time);
        if x_key != "timestamp" {
            rows.sort_by(|left, right| left.sort.total_cmp(&right.sort));
        }
        let technique = request.technique;
        let unit = request.unit.as_deref();
        let mut series = match series {
            Series::Keys => match request.series_by.as_ref() {
                None => key_series(&rows, &request.y_keys, unit, technique),
                Some(_) => grouped_key_series(&rows, &request.y_keys, unit, technique),
            },
            Series::Expressions => expression_series(&rows, &request, unit, technique),
        };
        for entry in &mut series {
            entry.format = format_context.series_format.clone();
        }
        let payload = PlotPayload {
            computed: None,
            stream: request.stream.clone(),
            x_key,
            series_by: request.series_by.clone(),
            context: BTreeMap::new(),
            series,
            cartesian: None,
            analysis: None,
            markers: plot_markers(
                records,
                &request.marker_record_types,
                request.scope.as_ref(),
            ),
            max_points: request.max_points.unwrap_or(DEFAULT_PLOT_MAX_POINTS),
            window_s: request.window_s,
            generated_at: generated_at.to_owned(),
            x_format: format_context.x_format.clone(),
            run_started_at: format_context.run_started_at.clone(),
            cycle_started_at: format_context.cycle_started_at.clone(),
            summaries: Vec::new(),
        };
        finish_plot_payload(payload, &request)
    }
}

impl Row {
    fn memory(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(self.timestamp.as_str().len())
            .saturating_add(json_memory(&self.x))
            .saturating_add(
                self.ys
                    .capacity()
                    .saturating_mul(std::mem::size_of::<Option<f64>>()),
            )
            .saturating_add(self.group.as_ref().map_or(0, String::capacity))
    }
}

/* === Public Functions === */

/* === Private Functions === */
// A z value read from the plotted sample itself, or `None` when it needs
// another stream's history.
fn per_sample_z(request: &PlotPayloadRequest) -> Option<ZIndex> {
    if let Some(expression) = &request.z_expression {
        return match &expression.source {
            DataSource::Telemetry { stream, field } => {
                stream_matches_alias(&request.stream, stream).then(|| ZIndex::Primary {
                    field: field.clone(),
                })
            }
            _ => Some(ZIndex::Expression(expression.clone())),
        };
    }
    let Some(field) = &request.z_field else {
        return Some(ZIndex::None);
    };
    let stream = request.z_stream.as_deref().unwrap_or(&request.stream);
    stream_matches_alias(&request.stream, stream).then(|| ZIndex::Primary {
        field: field.clone(),
    })
}

fn key_series(
    rows: &[Row],
    y_keys: &[String],
    unit: Option<&str>,
    technique: super::DownsampleTechnique,
) -> Vec<PlotSeries> {
    y_keys
        .iter()
        .enumerate()
        .map(|(index, key)| {
            let points = rows.iter().map(|row| row.point(index)).collect();
            finish_plot_series(
                key.clone(),
                key.clone(),
                unit.map(ToOwned::to_owned),
                points,
                0,
                technique,
            )
        })
        .collect()
}

fn grouped_key_series(
    rows: &[Row],
    y_keys: &[String],
    unit: Option<&str>,
    technique: super::DownsampleTechnique,
) -> Vec<PlotSeries> {
    let mut groups: BTreeMap<&str, Vec<&Row>> = BTreeMap::new();
    for row in rows {
        if let Some(group) = &row.group {
            groups.entry(group).or_default().push(row);
        }
    }
    let mut groups = groups.into_iter().collect::<Vec<_>>();
    groups.sort_by(|(left, _), (right, _)| compare_series_group(left, right));
    y_keys
        .iter()
        .enumerate()
        .flat_map(|(index, key)| {
            groups.iter().map(move |(group, rows)| {
                let (series_key, label) = if y_keys.len() == 1 {
                    ((*group).to_owned(), (*group).to_owned())
                } else {
                    (format!("{key}:{group}"), format!("{key} {group}"))
                };
                let points = rows.iter().map(|row| row.point(index)).collect();
                finish_plot_series(
                    series_key,
                    label,
                    unit.map(ToOwned::to_owned),
                    points,
                    0,
                    technique,
                )
            })
        })
        .collect()
}

fn expression_series(
    rows: &[Row],
    request: &PlotPayloadRequest,
    unit: Option<&str>,
    technique: super::DownsampleTechnique,
) -> Vec<PlotSeries> {
    request
        .y_expressions
        .iter()
        .enumerate()
        .flat_map(|(index, expression)| {
            let label = expression_field_name(expression)
                .unwrap_or("value")
                .to_owned();
            if request.series_by_expression.is_some() {
                let mut grouped = BTreeMap::<&str, Vec<PlotPoint>>::new();
                for row in rows {
                    let (Some(group), Some(_)) = (&row.group, row.ys[index]) else {
                        continue;
                    };
                    grouped.entry(group).or_default().push(row.point(index));
                }
                grouped
                    .into_iter()
                    .map(|(group, points)| {
                        finish_plot_series(
                            format!("{label}:{group}"),
                            group.to_owned(),
                            unit.map(ToOwned::to_owned),
                            points,
                            0,
                            technique,
                        )
                    })
                    .collect::<Vec<_>>()
            } else {
                let points = rows
                    .iter()
                    .filter(|row| row.ys[index].is_some())
                    .map(|row| row.point(index))
                    .collect();
                vec![finish_plot_series(
                    label.clone(),
                    label,
                    unit.map(ToOwned::to_owned),
                    points,
                    0,
                    technique,
                )]
            }
        })
        .collect()
}

impl Row {
    fn point(&self, index: usize) -> PlotPoint {
        PlotPoint {
            x: self.x.clone(),
            y: self.ys[index],
            z: self.z,
        }
    }
}

fn json_memory(value: &JsonValue) -> usize {
    match value {
        JsonValue::String(text) => text.capacity(),
        JsonValue::Array(items) => items
            .iter()
            .map(|item| std::mem::size_of::<JsonValue>().saturating_add(json_memory(item)))
            .sum(),
        JsonValue::Object(fields) => fields
            .iter()
            .map(|(key, item)| {
                key.capacity()
                    .saturating_add(std::mem::size_of::<JsonValue>())
                    .saturating_add(json_memory(item))
            })
            .sum(),
        _ => 0,
    }
}

/* === Tests === */
#[cfg(test)]
mod tests {
    use super::*;
    use crate::plot::payload::{DownsampleTechnique, build_plot_payload};
    use dokime::core::{Identifier, Value};

    fn sample(index: usize) -> TelemetrySample {
        let seconds = 1_700_000_000 + index as i64;
        let mut sample = TelemetrySample::new(
            Identifier::new("fixture.pulse").unwrap(),
            1,
            Timestamp::from_datetime(OffsetDateTime::from_unix_timestamp(seconds).unwrap()),
            "fixture",
        );
        let wave = ((index as f64) / 7.0).sin() * 10.0 + (index % 13) as f64;
        sample.values.insert("value".into(), Value::Number(wave));
        sample
            .values
            .insert("index".into(), Value::Integer(index as i64));
        sample.values.insert(
            "phase".into(),
            Value::Text(["a", "b", "c"][index % 3].to_owned()),
        );
        sample
    }

    fn requests() -> Vec<PlotPayloadRequest> {
        let base = PlotPayloadRequest {
            stream: "fixture.pulse".into(),
            y_keys: vec!["value".into()],
            ..PlotPayloadRequest::default()
        };
        let telemetry = |field: &str| dokime::expressions::telemetry("fixture.pulse", field);
        let mut requests = Vec::new();
        for technique in [
            DownsampleTechnique::Raw,
            DownsampleTechnique::MinMax,
            DownsampleTechnique::Lttb,
            DownsampleTechnique::Auto,
        ] {
            for max_points in [None, Some(64), Some(500), Some(0)] {
                requests.push(PlotPayloadRequest {
                    technique,
                    max_points,
                    ..base.clone()
                });
                requests.push(PlotPayloadRequest {
                    technique,
                    max_points,
                    series_by: Some("phase".into()),
                    y_keys: vec!["value".into(), "index".into()],
                    ..base.clone()
                });
                requests.push(PlotPayloadRequest {
                    technique,
                    max_points,
                    x_key: Some("index".into()),
                    window_s: Some(900.0),
                    ..base.clone()
                });
                requests.push(PlotPayloadRequest {
                    technique,
                    max_points,
                    y_keys: Vec::new(),
                    y_expressions: vec![telemetry("value")],
                    x_expression: Some(telemetry("index")),
                    ..base.clone()
                });
                requests.push(PlotPayloadRequest {
                    technique,
                    max_points,
                    y_keys: Vec::new(),
                    y_expressions: vec![telemetry("value"), telemetry("index")],
                    series_by_expression: Some(telemetry("phase")),
                    z_field: Some("index".into()),
                    ..base.clone()
                });
            }
        }
        requests
    }

    // streamed plots equal the slice builder for every series family and reduction.
    #[test]
    fn streamed_matches_slice() {
        for length in [0, 1, 7, 499, 2_500] {
            let mut samples = (0..length).map(sample).collect::<Vec<_>>();
            // Capture order is not timestamp order; both builders sort.
            samples.reverse();
            for request in requests() {
                let context = PlotPayloadContext::default();
                let expected = build_plot_payload(&samples, &[], request.clone(), "now", &context);
                let mut streamed = StreamedPlot::new(request.clone())
                    .unwrap_or_else(|| panic!("streamable request: {request:?}"));
                for page in samples.chunks(64) {
                    for sample in page {
                        streamed.push(sample);
                    }
                }
                let actual = streamed.finish(&[], "now", &context);
                assert_eq!(
                    serde_json::to_value(&actual).unwrap(),
                    serde_json::to_value(&expected).unwrap(),
                    "length={length} request={request:?}"
                );
            }
        }
    }

    // requests that need the whole capture are handed back.
    #[test]
    fn whole_capture_refused() {
        let base = PlotPayloadRequest {
            stream: "fixture.pulse".into(),
            y_keys: vec!["value".into()],
            ..PlotPayloadRequest::default()
        };
        for request in [
            PlotPayloadRequest {
                plot_type: Some(PlotType::FrequencySeries),
                ..base.clone()
            },
            PlotPayloadRequest {
                z_field: Some("value".into()),
                z_stream: Some("fixture.other".into()),
                ..base.clone()
            },
        ] {
            assert!(StreamedPlot::new(request).is_none());
        }
    }
}
