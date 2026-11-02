/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Incremental Plot Projections

/* === Modules === */

pub(crate) mod index;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::{PlotPayload, PlotPayloadContext, PlotPayloadRequest, PlotPoint, PlotSeries};
use crate::document::PlotType;
use crate::expressions::DataTransform;
use crate::runtime::Result;
use crate::storage::sqlite::data::telemetry::previews::{Envelope, Point};
use crate::telemetry::TelemetrySample;
use dokime_domain::model::records::evidence::RecordEntry;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub(crate) struct Limits {
    pub exact_points: usize,
    pub grid_columns: usize,
    pub grid_rows: usize,
    pub groups: usize,
    pub buckets: usize,
    pub interval_nanos: i128,
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Preview {
    samples: Vec<TelemetrySample>,
    records: Vec<RecordEntry>,
    reduced: bool,
    latest: BTreeMap<String, TelemetrySample>,
    previous: BTreeMap<String, RecordEntry>,
    previous_samples: BTreeMap<String, Vec<TelemetrySample>>,
    series: BTreeMap<String, Trace>,
    pub context: BTreeMap<String, Value>,
    pub reason: Option<String>,

    marker_records: Vec<RecordEntry>,

    marker_count: usize,

    marker_stride: usize,

    last_marker: Option<RecordEntry>,
}

#[derive(Serialize, Deserialize)]
struct Trace {
    template: PlotSeries,
    exact: Vec<PlotPoint>,
    envelope: Envelope,
    grid: Grid,
    count: usize,
    moments: Moments,
    groups: BTreeMap<String, (serde_json::Value, Moments)>,
}

#[derive(Default, Serialize, Deserialize)]
struct Grid {
    extent: Option<[f64; 4]>,
    cells: BTreeMap<usize, Cell>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
struct Cell {
    x: f64,
    y: f64,
    count: u64,
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct Moments {
    count: usize,
    scale: f64,
    sum: f64,
    squares: f64,
    min: f64,
    max: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl From<crate::storage::sqlite::PreviewSettings> for Limits {
    fn from(settings: crate::storage::sqlite::PreviewSettings) -> Self {
        Self {
            exact_points: settings.exact_points.get(),
            grid_columns: settings.grid_columns.get(),
            grid_rows: settings.grid_rows.get(),
            groups: settings.groups.get(),
            buckets: settings.bucket_count.get(),
            interval_nanos: settings.bucket_interval.as_nanos() as i128,
        }
    }
}

impl Preview {
    pub(crate) fn merge(&mut self, other: Self, request: &PlotPayloadRequest, limits: Limits) -> Result<()> {
        self.marker_records.extend(self.last_marker.take());
        self.marker_records.extend(other.marker_records);
        self.marker_records.extend(other.last_marker);
        self.marker_records.sort_by(|left, right| left.timestamp.cmp(&right.timestamp).then(left.sequence.cmp(&right.sequence)));
        self.marker_records.dedup_by(|left, right| left.record_id == right.record_id);
        self.last_marker = self.marker_records.last().cloned();
        if self.marker_records.len() > limits.exact_points {
            let mut selected = super::payload::marker_indices(self.marker_records.len(), limits.exact_points).into_iter().peekable();
            let mut index = 0;
            self.marker_records.retain(|_| {
                let keep = selected.peek() == Some(&index);
                if keep {
                    selected.next();
                }
                index += 1;
                keep
            });
        }
        self.reduced |= other.reduced;
        self.samples.extend(other.samples);
        self.records.extend(other.records);
        self.bound(limits);
        if self.reduced {
            self.samples.clear();
            self.records.clear();
        }
        self.context.extend(other.context);
        self.reason = self.reason.take().or(other.reason);
        for (key, other) in other.series {
            let Some(trace) = self.series.get_mut(&key) else {
                if self.series.len() >= limits.groups {
                    self.reason = Some("The plot has more series than the configured preview group limit.".into());
                    break;
                }
                self.series.insert(key, other);
                continue;
            };
            if matches!(request.plot_type, Some(PlotType::XyScatter | PlotType::Trajectory | PlotType::XyValueHeatmap))
                && trace.count + other.count > limits.exact_points
            {
                for point in trace.exact.iter().chain(&other.exact) {
                    if let (Some(x), Some(y)) = (axis(point), point.y) {
                        trace.grid.push(Cell { x, y, count: 1 }, limits);
                    }
                }
            }
            for cell in other.grid.cells.values() {
                trace.grid.push(*cell, limits);
            }
            for bucket in other.envelope.buckets.values() {
                for mut point in bucket.points() {
                    point.position = point.position.saturating_add(trace.count as u64);
                    trace.envelope.push(point, limits.buckets)?;
                }
            }
            trace.count += other.count;
            trace.moments.merge(other.moments);
            trace.exact.extend(other.exact);
            if trace.count > limits.exact_points {
                trace.exact.clear();
            }
            for (key, (x, moments)) in other.groups {
                if !trace.groups.contains_key(&key) && trace.groups.len() >= limits.groups {
                    self.reason = Some("The plot has more count groups than the configured preview group limit.".into());
                    break;
                }
                trace.groups.entry(key).and_modify(|(_, current)| current.merge(moments.clone())).or_insert((x, moments));
            }
        }
        Ok(())
    }

    pub(crate) fn sample(&mut self, mut sample: TelemetrySample, request: &PlotPayloadRequest, limits: Limits) -> Result<()> {
        if !super::sample_matches_scope(&sample, request.scope.as_ref()) {
            return Ok(());
        }
        if !record_plot(request)
            && super::stream_matches_alias(sample.stream.as_str(), &request.stream)
            && !super::sample_matches_filters(&sample, &request.filters)
        {
            return Ok(());
        }
        sample.storage_payload = None;
        let key = if record_plot(request) {
            format!("{}:{}", sample.stream, serde_json::to_string(&sample.scope).map_err(crate::storage::Error::from)?)
        } else {
            sample.stream.to_string()
        };
        if !self.latest.contains_key(&key) && self.latest.len() >= limits.groups {
            self.reason = Some("The plot has more correlation scopes than the configured preview limit.".into());
            return Ok(());
        }
        if super::stream_matches_alias(sample.stream.as_str(), &request.stream) {
            self.context.extend(
                sample.values.iter().filter(|(_, value)| !matches!(value, Value::Null)).map(|(key, value)| (key.clone(), value.clone())),
            );
        }
        self.latest.insert(key, sample.clone());
        if !self.reduced {
            self.samples.push(sample.clone());
            self.bound(limits);
        }
        if !super::stream_matches_alias(sample.stream.as_str(), &request.stream) || record_plot(request) {
            return Ok(());
        }
        self.project(std::slice::from_ref(&sample), &[], request, limits)
    }

    pub(crate) fn record(&mut self, record: RecordEntry, request: &PlotPayloadRequest, limits: Limits) -> Result<()> {
        let relevant = request.y_expressions.iter().chain(request.x_expression.iter()).any(|expression| {
            matches!(&expression.source, crate::expressions::DataSource::Record { record_type } if record_type == record.record_type.as_str())
                || expression.transforms.iter().any(|transform| matches!(transform, DataTransform::TimeBetween { previous_record_type } if previous_record_type == record.record_type.as_str()))
        });
        if !relevant && !request.marker_record_types.iter().any(|kind| kind == record.record_type.as_str()) {
            return Ok(());
        }
        if request.scope.as_ref().is_some_and(|scope| !record.is_visible_in(scope)) {
            return Ok(());
        }
        if request.marker_record_types.iter().any(|kind| kind == record.record_type.as_str()) {
            self.marker(record.clone(), limits);
        }
        if !relevant {
            return Ok(());
        }
        if !self.reduced {
            self.records.push(record.clone());
            self.bound(limits);
        }
        if record_plot(request) {
            let mut records: Vec<_> = self.previous.values().filter(|previous| {
                previous.record_type != record.record_type || request.y_expressions.iter().flat_map(|expression| &expression.transforms)
                    .any(|transform| matches!(transform, DataTransform::TimeBetween { previous_record_type } if previous_record_type == previous.record_type.as_str()))
            }).cloned().collect();
            records.push(record.clone());
            records.sort_by_key(|record| record.sequence);
            // Project Only The New End Record; Prior End Records Are Correlation State.
            let mut current = request.clone();
            current.y_expressions.retain(|expression| {
                matches!(&expression.source, crate::expressions::DataSource::Record { record_type } if record_type == record.record_type.as_str())
            });
            if !current.y_expressions.is_empty() {
                let mut samples: Vec<_> = self.previous_samples.values().flatten().cloned().collect();
                samples.extend(self.latest.values().cloned());
                samples.sort_by(|left, right| left.timestamp.cmp(&right.timestamp).then(left.sequence.cmp(&right.sequence)));
                samples.dedup_by(|left, right| left.sample_id == right.sample_id);
                self.project(&samples, &records, &current, limits)?;
            }
        }
        let key = format!("{}:{}", record.record_type, serde_json::to_string(&record.scope).map_err(crate::storage::Error::from)?);
        if !self.previous.contains_key(&key) && self.previous.len() >= limits.groups {
            self.reason = Some("The plot has more record correlation scopes than the configured preview limit.".into());
            return Ok(());
        }
        self.previous_samples.insert(key.clone(), self.latest.values().cloned().collect());
        self.previous.insert(key, record);
        Ok(())
    }

    fn marker(&mut self, record: RecordEntry, limits: Limits) {
        self.marker_stride = self.marker_stride.max(1);
        if self.marker_count.is_multiple_of(self.marker_stride) {
            self.marker_records.push(record.clone());
        }
        self.marker_count = self.marker_count.saturating_add(1);
        self.last_marker = Some(record);
        if self.marker_records.len() > limits.exact_points.max(1) {
            let mut index = 0;
            self.marker_records.retain(|_| {
                let keep = index % 2 == 0;
                index += 1;
                keep
            });
            self.marker_stride = self.marker_stride.saturating_mul(2);
        }
    }

    fn bound(&mut self, limits: Limits) {
        if self.samples.len().saturating_add(self.records.len()) > limits.exact_points {
            self.samples.clear();
            self.records.clear();
            self.reduced = true;
        }
    }

    fn project(
        &mut self, samples: &[TelemetrySample], records: &[RecordEntry], request: &PlotPayloadRequest, limits: Limits,
    ) -> Result<()> {
        let mut raw = request.clone();
        let aggregate = aggregate(request);
        if aggregate.is_some() {
            for expression in &mut raw.y_expressions {
                expression.transforms.retain(|transform| !matches!(transform, DataTransform::Aggregate(_)));
            }
        }
        let context = PlotPayloadContext {
            default_max_points: limits.exact_points,
            x_format: request.x_format.as_ref(),
            series_format: request.y_format.as_ref(),
            run_started_at: None,
            cycle_started_at: None,
        };
        let payload = super::project_retained_plot_payload(samples, records.into(), &raw, "", &context)?;
        self.context.extend(payload.context.clone());
        let scatter = matches!(request.plot_type, Some(PlotType::XyScatter | PlotType::Trajectory | PlotType::XyValueHeatmap));
        for series in &payload.series {
            let aggregate = aggregate_for(request, &series.key);
            if !self.series.contains_key(&series.key) && self.series.len() >= limits.groups {
                self.reason = Some("The plot has more series than the configured preview group limit.".into());
                break;
            }
            let trace = self.series.entry(series.key.clone()).or_insert_with(|| Trace {
                template: PlotSeries { points: Vec::new(), ..series.clone() },
                exact: Vec::new(),
                envelope: Envelope {
                    origin: 0,
                    width: limits.interval_nanos,
                    buckets: BTreeMap::new(),
                },
                grid: Grid::default(),
                count: 0,
                moments: Moments::default(),
                groups: BTreeMap::new(),
            });
            for point in &series.points {
                let Some(value) = point.y.filter(|value| value.is_finite()) else { continue };
                trace.count += 1;
                trace.moments.push(value);
                if let Some(method) = aggregate {
                    let key = group_key(&point.x);
                    if !trace.groups.contains_key(&key) && trace.groups.len() >= limits.groups {
                        self.reason = Some("The plot has more count groups than the configured preview group limit.".into());
                        break;
                    }
                    let (_, moments) = trace.groups.entry(key).or_insert_with(|| (point.x.clone(), Moments::default()));
                    moments.push(value);
                    if moments.value(method).is_none() && self.reduced {
                        self.reason = Some(format!("{} needs more exact values than the configured preview limit.", method.name()));
                    }
                    continue;
                }
                if trace.count <= limits.exact_points {
                    trace.exact.push(point.clone());
                } else if trace.count == limits.exact_points + 1 {
                    if scatter {
                        for point in &trace.exact {
                            if let (Some(x), Some(y)) = (axis(point), point.y) {
                                trace.grid.push(Cell { x, y, count: 1 }, limits);
                            }
                        }
                    }
                    trace.exact.clear();
                }
                if scatter && trace.count > limits.exact_points {
                    if let Some(x) = axis(point) {
                        trace.grid.push(Cell { x, y: value, count: 1 }, limits);
                    }
                } else if let Some(x) = axis(point) {
                    let time = point
                        .x
                        .as_str()
                        .and_then(super::parse_timestamp)
                        .map(|time| time.unix_timestamp_nanos())
                        .unwrap_or((x * 1e9) as i128);
                    trace.envelope.push(
                        Point {
                            time,
                            position: trace.count as u64,
                            value,
                        },
                        limits.buckets,
                    )?;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn payload(
        &self, request: &PlotPayloadRequest, generated_at: &str, context: &PlotPayloadContext, limits: Limits,
    ) -> Result<PlotPayload> {
        let raw = if self.reduced { self.latest.values().cloned().collect::<Vec<_>>() } else { self.samples.clone() };
        let records = if self.reduced { &[][..] } else { &self.records[..] };
        let mut payload = super::project_retained_plot_payload(&raw, records.into(), request, generated_at, context)?.as_ref().clone();
        let mut markers = self.marker_records.clone();
        if let Some(last) = &self.last_marker
            && markers.last().is_none_or(|record| record.record_id != last.record_id)
        {
            markers.push(last.clone());
        }
        if !markers.is_empty() {
            payload.markers = super::payload::plot_markers(
                markers.as_slice().into(),
                &request.marker_record_types,
                request.scope.as_ref(),
                payload.max_points,
            );
        }
        if self.reduced {
            payload.series.clear();
            for trace in self.series.values() {
                let points = if let Some(method) = aggregate_for(request, &trace.template.key) {
                    trace
                        .groups
                        .values()
                        .map(|(x, moments)| PlotPoint {
                            x: x.clone(),
                            y: moments.value(method),
                            z: None,
                        })
                        .collect()
                } else if trace.count <= limits.exact_points {
                    trace.exact.clone()
                } else if !trace.grid.cells.is_empty() {
                    let maximum = payload.max_points.max(1);
                    let mut display = limits;
                    display.grid_columns = display.grid_columns.min(maximum);
                    display.grid_rows = display.grid_rows.min(maximum / display.grid_columns);
                    let mut grid = Grid {
                        extent: trace.grid.extent,
                        cells: BTreeMap::new(),
                    };
                    for cell in trace.grid.cells.values() {
                        grid.insert(*cell, display);
                    }
                    grid.cells
                        .values()
                        .map(|cell| PlotPoint {
                            x: serde_json::json!(cell.x),
                            y: Some(cell.y),
                            z: Some(cell.count as f64),
                        })
                        .collect()
                } else {
                    let timestamp = time_axis(request);
                    let mut envelope = trace.envelope.clone();
                    envelope.fit((payload.max_points / 4).max(1))?;
                    envelope
                        .buckets
                        .values()
                        .flat_map(|bucket| bucket.points())
                        .map(|point| {
                            let x = point.time as f64 / 1e9;
                            let x = if timestamp {
                                let time = time::OffsetDateTime::from_unix_timestamp_nanos(point.time).expect("ingested timestamp");
                                serde_json::Value::String(
                                    time.format(&time::format_description::well_known::Rfc3339).expect("valid timestamp"),
                                )
                            } else {
                                serde_json::json!(x)
                            };
                            PlotPoint { x, y: Some(point.value), z: None }
                        })
                        .collect()
                };
                payload.series.push(PlotSeries {
                    downsampled: points.len() < trace.count,
                    raw_points: trace.count,
                    points,
                    ..trace.template.clone()
                });
            }
        }
        if request.series_by.is_some() || request.series_by_expression.is_some() {
            payload.series.sort_by(|left, right| super::payload::compare_series_group(&left.label, &right.label));
        }
        payload.context.extend(self.context.clone());
        if let Some(selection) = &request.analysis {
            if self.reduced {
                payload.analysis = Some(super::PlotAnalysis::Error {
                    message: format!("{} needs more exact samples than the configured preview limit.", selection.id),
                });
                return super::finish_plot_frame(payload, request);
            }
            return Ok(payload);
        }
        let mut payload = super::finish_retained_plot_payload(harmos::storage::ReadBatch::new(payload), request)?.as_ref().clone();
        if !self.reduced
            && request.series_by.is_none()
            && request.series_by_expression.is_none()
            && request.window_s.is_none()
            && request.summary_window.is_none()
        {
            self.exact_summaries(&mut payload, request)?;
        }
        if self.reduced && aggregate(request).is_none() && request.summary_window.is_none() {
            for summary in &mut payload.summaries {
                if let Some(trace) = self.series.get(&summary.series_id) {
                    summary.count = trace.count;
                    summary.min = Some(trace.moments.min);
                    summary.max = Some(trace.moments.max);
                    summary.mean = trace.moments.value(harmos::Aggregate::Mean);
                    summary.std = trace.moments.value(harmos::Aggregate::Std);
                    summary.rms = trace.moments.value(harmos::Aggregate::Rms);
                    summary.median = None;
                }
            }
        }
        if self.reduced && request.summary_window.is_some() {
            // A Whole-Capture Envelope Cannot Supply Exact Statistics For A New Subrange.
            payload.summaries.clear();
        }
        if let Some(reason) = &self.reason {
            payload.analysis = Some(super::PlotAnalysis::Error { message: reason.clone() });
        } else if self.reduced
            && matches!(
                request.plot_type,
                Some(PlotType::FrequencySeries | PlotType::Histogram | PlotType::BoxSummary | PlotType::XyInterval)
            )
        {
            payload.analysis = Some(super::PlotAnalysis::Error {
                message: "This analysis needs more exact samples than the configured preview limit.".into(),
            });
        }
        Ok(payload)
    }

    fn exact_summaries(&self, payload: &mut PlotPayload, request: &PlotPayloadRequest) -> Result<()> {
        use harmos::storage::query::{
            Accumulator,
            scalar::{Agg, ScalarValue},
        };
        for summary in &mut payload.summaries {
            let field = request
                .y_expressions
                .iter()
                .find_map(|expression| {
                    let crate::expressions::DataSource::Telemetry { field, .. } = &expression.source else { return None };
                    (expression.transforms.is_empty()
                        && (field == &summary.series_id || expression.label.as_ref() == Some(&summary.series_id)))
                    .then_some(field.as_str())
                })
                .or_else(|| request.y_keys.iter().find(|field| *field == &summary.series_id).map(String::as_str));
            let Some(field) = field else { continue };
            let values = self
                .samples
                .iter()
                .filter(|sample| super::stream_matches_alias(sample.stream.as_str(), &request.stream))
                .filter_map(|sample| sample.values.get(field))
                .filter_map(|value| match value {
                    Value::Integer(value) => Some(ScalarValue::Signed(*value)),
                    Value::Unsigned(value) => Some(ScalarValue::Unsigned(*value)),
                    Value::Number(value) if value.is_finite() => Some(ScalarValue::Float(*value)),
                    _ => None,
                })
                .collect::<Vec<_>>();
            if values.is_empty() {
                continue;
            }
            let statistic = |method| -> Result<Option<f64>> {
                let mut accumulator = Accumulator::new(Agg::Of(method));
                for (index, value) in values.iter().enumerate() {
                    accumulator.push(index as u64, value)?;
                }
                Ok(match accumulator.finish(&mut || Ok(()))? {
                    ScalarValue::Float(value) => Some(value),
                    ScalarValue::Signed(value) => Some(value as f64),
                    ScalarValue::Unsigned(value) => Some(value as f64),
                    ScalarValue::Exact(value) => {
                        Some((if value.negative { -1.0 } else { 1.0 }) * value.numerator as f64 / value.denominator.get() as f64)
                    }
                    _ => None,
                })
            };
            summary.count = values.len();
            summary.min = statistic(harmos::Aggregate::Min)?;
            summary.max = statistic(harmos::Aggregate::Max)?;
            summary.mean = statistic(harmos::Aggregate::Mean)?;
            summary.median = statistic(harmos::Aggregate::Median)?;
            summary.std = statistic(harmos::Aggregate::Std)?;
            summary.rms = statistic(harmos::Aggregate::Rms)?;
        }
        Ok(())
    }
}

impl Moments {
    fn merge(&mut self, other: Self) {
        if other.count == 0 {
            return;
        }
        if self.count == 0 {
            *self = other;
            return;
        }
        let scale = self.scale.max(other.scale);
        let factor = |value| if scale == 0.0 { 0.0 } else { value / scale };
        self.sum = self.sum * factor(self.scale) + other.sum * factor(other.scale);
        self.squares = self.squares * factor(self.scale).powi(2) + other.squares * factor(other.scale).powi(2);
        self.scale = scale;
        self.count += other.count;
        self.min = self.min.min(other.min);
        self.max = self.max.max(other.max);
    }

    fn push(&mut self, value: f64) {
        self.count += 1;
        let scale = self.scale.max(value.abs());
        if scale > self.scale {
            let factor = self.scale / scale;
            self.sum *= factor;
            self.squares *= factor * factor;
            self.scale = scale;
        }
        let scaled = if self.scale == 0.0 { 0.0 } else { value / self.scale };
        self.sum += scaled;
        self.squares += scaled * scaled;
        if self.count == 1 {
            self.min = value;
            self.max = value;
        }
        self.min = self.min.min(value);
        self.max = self.max.max(value);
    }

    fn value(&self, method: harmos::Aggregate) -> Option<f64> {
        use harmos::Aggregate;
        let count = self.count as f64;
        let mean = self.sum / count;
        let value = match method {
            Aggregate::Count => count,
            Aggregate::Sum => self.scale * self.sum,
            Aggregate::Mean => self.scale * mean,
            Aggregate::Min => self.min,
            Aggregate::Max => self.max,
            Aggregate::Std => self.scale * (self.squares / count - mean * mean).max(0.0).sqrt(),
            Aggregate::Rms => self.scale * (self.squares / count).sqrt(),
            _ => return None,
        };
        value.is_finite().then_some(value)
    }
}

impl Grid {
    fn push(&mut self, cell: Cell, limits: Limits) {
        let bounds = self.extent.unwrap_or([cell.x, cell.x, cell.y, cell.y]);
        let next = [bounds[0].min(cell.x), bounds[1].max(cell.x), bounds[2].min(cell.y), bounds[3].max(cell.y)];
        if self.extent != Some(next) {
            self.extent = Some(next);
            for existing in std::mem::take(&mut self.cells).into_values() {
                self.insert(existing, limits);
            }
        }
        self.insert(cell, limits);
    }

    fn insert(&mut self, cell: Cell, limits: Limits) {
        let [left, right, bottom, top] = self.extent.expect("grid bounds");
        let bin = |value: f64, min: f64, max: f64, size: usize| {
            if max <= min {
                return 0;
            }
            (((value - min) / (max - min) * size as f64) as usize).min(size - 1)
        };
        let x = bin(cell.x, left, right, limits.grid_columns);
        let y = bin(cell.y, bottom, top, limits.grid_rows);
        self.cells
            .entry(y * limits.grid_columns + x)
            .and_modify(|existing| {
                let count = existing.count + cell.count;
                let weight = cell.count as f64 / count as f64;
                existing.x = existing.x * (1.0 - weight) + cell.x * weight;
                existing.y = existing.y * (1.0 - weight) + cell.y * weight;
                existing.count = count;
            })
            .or_insert(cell);
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn group_key(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(value) => value.clone(),
        serde_json::Value::Number(value) => value
            .as_i64()
            .map(|value| value.to_string())
            .or_else(|| value.as_u64().map(|value| value.to_string()))
            .or_else(|| value.as_f64().map(|value| value.to_string()))
            .unwrap_or_default(),
        _ => value.to_string(),
    }
}

fn aggregate(request: &PlotPayloadRequest) -> Option<harmos::Aggregate> {
    request
        .y_expressions
        .iter()
        .flat_map(|expression| &expression.transforms)
        .find_map(|transform| if let DataTransform::Aggregate(method) = transform { Some(*method) } else { None })
}

fn aggregate_for(request: &PlotPayloadRequest, key: &str) -> Option<harmos::Aggregate> {
    let expression = request.y_expressions.iter().find(|expression| match &expression.source {
        crate::expressions::DataSource::Telemetry { field, .. } => {
            key == field || key.starts_with(&format!("{field}:")) || expression.label.as_deref() == Some(key)
        }
        crate::expressions::DataSource::Record { record_type } => key.ends_with(record_type),
        _ => false,
    });
    expression
        .into_iter()
        .flat_map(|expression| &expression.transforms)
        .find_map(|transform| if let DataTransform::Aggregate(method) = transform { Some(*method) } else { None })
}

fn record_plot(request: &PlotPayloadRequest) -> bool {
    request.y_expressions.iter().any(|expression| matches!(expression.source, crate::expressions::DataSource::Record { .. }))
}

fn time_axis(request: &PlotPayloadRequest) -> bool {
    request.x_key.as_deref().is_none_or(|key| key == "timestamp")
        && request.x_expression.as_ref().is_none_or(
            |expression| matches!(&expression.source, crate::expressions::DataSource::Telemetry { field, .. } if field == "timestamp"),
        )
}

fn axis(point: &PlotPoint) -> Option<f64> {
    point
        .x
        .as_f64()
        .or_else(|| {
            point.x.as_str().and_then(|value| {
                super::parse_timestamp(value).map(|time| time.unix_timestamp_nanos() as f64 / 1e9).or_else(|| value.parse().ok())
            })
        })
        .filter(|value| value.is_finite())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expressions::{record, telemetry};
    use dokime_domain::model::{
        execution::RecordScope,
        identity::{Identifier, RecordId, TelemetrySampleId},
        time::Timestamp,
    };

    fn limits() -> Limits {
        Limits {
            exact_points: 4,
            grid_columns: 3,
            grid_rows: 2,
            groups: 32,
            buckets: 4,
            interval_nanos: 1_000_000,
        }
    }

    fn sample(position: u64, x: f64, y: f64) -> TelemetrySample {
        let time = time::OffsetDateTime::UNIX_EPOCH + time::Duration::seconds(position as i64);
        let mut sample = TelemetrySample::new(
            TelemetrySampleId::from_seed(position.to_string()),
            Identifier::new("tracking").unwrap(),
            position,
            Timestamp::new(time.format(&time::format_description::well_known::Rfc3339).unwrap()),
            "fixture",
        );
        sample.values = BTreeMap::from([("easting".into(), Value::Number(x)), ("northing".into(), Value::Number(y))]);
        sample
    }

    fn payload(preview: &Preview, request: &PlotPayloadRequest) -> PlotPayload {
        preview
            .payload(
                request,
                "1970-01-01T00:00:00Z",
                &PlotPayloadContext {
                    default_max_points: 32,
                    x_format: None,
                    series_format: None,
                    run_started_at: None,
                    cycle_started_at: None,
                },
                limits(),
            )
            .unwrap()
    }

    #[test]
    fn exact_analysis() {
        for plot_type in [PlotType::Histogram, PlotType::BoxSummary] {
            let request = PlotPayloadRequest {
                stream: "tracking".into(),
                plot_type: Some(plot_type),
                y_keys: vec!["northing".into()],
                max_points: Some(3),
                ..Default::default()
            };
            let mut preview = Preview::default();
            for index in 0..4 {
                preview.sample(sample(index, index as f64, index as f64), &request, limits()).unwrap();
            }
            assert!(!matches!(payload(&preview, &request).analysis, Some(super::super::PlotAnalysis::Error { .. })));
            preview.sample(sample(4, 4.0, 4.0), &request, limits()).unwrap();
            assert!(matches!(payload(&preview, &request).analysis, Some(super::super::PlotAnalysis::Error { .. })));
            let mut selected = request.clone();
            selected.max_points = Some(1);
            selected.analysis = Some(crate::analysis::Selection {
                id: "dokime.analysis.mean".into(),
                version: 1,
                parameters: Default::default(),
            });
            let refused = payload(&preview, &selected);
            assert!(matches!(refused.analysis, Some(super::super::PlotAnalysis::Error { .. })));
            assert!(refused.series.iter().all(|series| series.points.len() <= selected.max_points.unwrap()));
        }
    }

    #[test]
    fn marker_budget() {
        let mut preview = Preview::default();
        let mut request = PlotPayloadRequest {
            marker_record_types: vec!["connected".into()],
            ..Default::default()
        };
        for index in 0..123 {
            preview.record(event(index, index as i64, "connected"), &request, limits()).unwrap();
        }
        assert!(preview.marker_records.len() <= limits().exact_points);
        for maximum in [1, 2, 3, 17] {
            request.max_points = Some(maximum);
            let result = payload(&preview, &request);
            assert!(!result.markers.is_empty());
            assert!(result.markers.len() <= maximum);
            if maximum > 1 {
                assert_eq!(result.markers.first().unwrap().x, serde_json::json!("1970-01-01T00:00:00Z"));
                assert_eq!(result.markers.last().unwrap().x, serde_json::json!("1970-01-01T00:02:02Z"));
            }
        }
    }

    #[test]
    fn merged_history() {
        let request = PlotPayloadRequest {
            stream: "tracking".into(),
            y_keys: vec!["northing".into()],
            plot_type: Some(PlotType::TimeSeries),
            max_points: Some(3),
            ..Default::default()
        };
        let mut left = Preview::default();
        let mut right = Preview::default();
        for index in 0..20 {
            let target = if index < 10 { &mut left } else { &mut right };
            target.sample(sample(index, index as f64, index as f64), &request, limits()).unwrap();
        }
        left.merge(right, &request, limits()).unwrap();
        let result = payload(&left, &request);
        assert_eq!(result.series[0].raw_points, 20);
        assert!(result.series[0].points.len() <= 3);
        assert!((result.summaries[0].mean.unwrap() - 9.5).abs() < 1e-12);
        assert_eq!(result.summaries[0].min, Some(0.0));
        assert_eq!(result.summaries[0].max, Some(19.0));
        assert!(result.series[0].points.iter().all(|point| point.x.is_string()));
    }

    #[test]
    fn scatter_density() {
        let request = PlotPayloadRequest {
            stream: "tracking".into(),
            plot_type: Some(PlotType::XyScatter),
            x_key: Some("easting".into()),
            y_keys: vec!["northing".into()],
            ..Default::default()
        };
        let mut preview = Preview::default();
        let fixture = [(2.0, 7.0), (-2.0, -7.0), (2.0, -7.0), (-2.0, 7.0)];
        for (index, (x, y)) in fixture.into_iter().enumerate() {
            preview.sample(sample(index as u64, x, y), &request, limits()).unwrap();
        }
        let exact = payload(&preview, &request);
        let mut pairs = exact.series[0].points.iter().map(|point| (point.x.as_f64().unwrap(), point.y.unwrap())).collect::<Vec<_>>();
        pairs.sort_by(|left, right| left.0.total_cmp(&right.0).then(left.1.total_cmp(&right.1)));
        assert_eq!(pairs, vec![(-2.0, -7.0), (-2.0, 7.0), (2.0, -7.0), (2.0, 7.0)]);
        for index in 4..40 {
            preview.sample(sample(index, index as f64 * 3.0, -(index as f64)), &request, limits()).unwrap();
            preview = serde_json::from_slice(&serde_json::to_vec(&preview).unwrap()).unwrap();
        }
        let density = payload(&preview, &request);
        assert_eq!(density.series[0].raw_points, 40);
        assert_eq!(density.series[0].points.iter().map(|point| point.z.unwrap() as u64).sum::<u64>(), 40);
        assert!(density.series[0].points.len() <= limits().grid_rows * limits().grid_columns);
        assert_eq!(density.cartesian.unwrap().mode, super::super::PlotCartesianMode::Faceted);
        assert!(preview.reason.is_none());
        let mut small = request.clone();
        small.max_points = Some(2);
        let compressed = payload(&preview, &small);
        assert!(compressed.series[0].points.len() <= 2);
        assert_eq!(compressed.series[0].points.iter().map(|point| point.z.unwrap() as u64).sum::<u64>(), 40);
    }

    #[test]
    fn grouped_counts() {
        let request = PlotPayloadRequest {
            stream: "tracking".into(),
            plot_type: Some(PlotType::Bar),
            x_expression: Some(telemetry("tracking", "easting")),
            y_expressions: vec![telemetry("tracking", "northing").count()],
            filters: vec![crate::document::DocumentFilter {
                expression: telemetry("tracking", "northing"),
                operator: crate::document::FilterOperator::Gt,
                value: Some(Value::Number(0.0)),
            }],
            ..Default::default()
        };
        let mut preview = Preview::default();
        let values = [(1.0, 1.0), (6.0, 2.0), (1.0, -3.0), (1.0, 4.0), (6.0, 5.0), (6.0, 6.0), (1.0, 7.0), (6.0, 8.0)];
        for (index, (x, y)) in values.into_iter().enumerate() {
            preview.sample(sample(index as u64, x, y), &request, limits()).unwrap();
            preview = serde_json::from_slice(&serde_json::to_vec(&preview).unwrap()).unwrap();
        }
        let result = payload(&preview, &request);
        let counts: BTreeMap<_, _> = result.series[0]
            .points
            .iter()
            .map(|point| {
                (
                    point.x.as_f64().or_else(|| point.x.as_str().and_then(|value| value.parse().ok())).unwrap() as i64,
                    point.y.unwrap() as usize,
                )
            })
            .collect();
        assert_eq!(counts, BTreeMap::from([(1, 3), (6, 4)]));
        assert!(preview.reason.is_none());
    }

    #[test]
    fn channel_envelopes() {
        let request = PlotPayloadRequest {
            stream: "tracking".into(),
            plot_type: Some(PlotType::WifiChannelOccupancy),
            y_keys: vec!["northing".into()],
            series_by: Some("easting".into()),
            ..Default::default()
        };
        let mut preview = Preview::default();
        for scan in 0..20 {
            for (offset, channel) in [1.0, 6.0].into_iter().enumerate() {
                let count = if scan == 19 { channel } else { (scan % 4 + 1) as f64 };
                let mut sample = sample(scan * 2 + offset as u64, channel, count);
                let time = time::OffsetDateTime::UNIX_EPOCH + time::Duration::seconds(scan as i64);
                sample.timestamp = Timestamp::new(time.format(&time::format_description::well_known::Rfc3339).unwrap());
                preview.sample(sample, &request, limits()).unwrap();
            }
            preview = serde_json::from_slice(&serde_json::to_vec(&preview).unwrap()).unwrap();
        }
        let result = payload(&preview, &request);
        assert_eq!(
            result.series.iter().map(|series| (series.key.as_str(), series.raw_points)).collect::<Vec<_>>(),
            vec![("1", 20), ("6", 20)]
        );
        let Some(super::super::PlotAnalysis::WifiOccupancy { scans, channels, .. }) = result.analysis else {
            panic!("occupancy analysis missing")
        };
        assert_eq!(channels, vec![1, 6]);
        let last = scans.last().unwrap();
        assert_eq!(last.channels.iter().map(|channel| (channel.channel, channel.count)).collect::<Vec<_>>(), vec![(1, 1), (6, 6)]);
        assert_eq!(last.total_networks, 7);
    }

    fn event(sequence: u64, seconds: i64, kind: &str) -> RecordEntry {
        use dokime_domain::model::records::{RecordSeverity, publication::RecordPublishRequest};
        let time = time::OffsetDateTime::UNIX_EPOCH + time::Duration::seconds(seconds);
        let record = RecordPublishRequest::new(Identifier::new(kind).unwrap(), RecordSeverity::Info).into_record(
            RecordId::from_seed(sequence.to_string()),
            1,
            Timestamp::new(time.format(&time::format_description::well_known::Rfc3339).unwrap()),
            "fixture",
        );
        RecordEntry::from_record(sequence, record)
    }

    #[test]
    fn reconnect_durations() {
        let request = PlotPayloadRequest {
            stream: "tracking".into(),
            plot_type: Some(PlotType::TimeSeries),
            y_expressions: vec![record("connected").time_between("disconnected")],
            ..Default::default()
        };
        let mut preview = Preview::default();
        for index in 0..20 {
            preview.record(event(index * 2, index as i64 * 10, "disconnected"), &request, limits()).unwrap();
            preview = serde_json::from_slice(&serde_json::to_vec(&preview).unwrap()).unwrap();
            preview.record(event(index * 2 + 1, index as i64 * 10 + (index % 5 + 1) as i64, "connected"), &request, limits()).unwrap();
        }
        let result = payload(&preview, &request);
        assert_eq!(result.series[0].raw_points, 20);
        assert_eq!(result.series[0].unit.as_deref(), Some("s"));
        assert!(result.series[0].points.iter().all(|point| (1.0..=5.0).contains(&point.y.unwrap())));
        assert_eq!(result.summaries[0].count, 20);
        assert!((result.summaries[0].mean.unwrap() - 3.0).abs() < 1e-12);
        assert!(result.cartesian.is_some());
    }

    #[test]
    fn integer_statistics() {
        let request = PlotPayloadRequest {
            stream: "tracking".into(),
            plot_type: Some(PlotType::TimeSeries),
            y_keys: vec!["northing".into()],
            ..Default::default()
        };
        let mut preview = Preview::default();
        for (index, value) in [9_007_199_254_740_993, 9_007_199_254_740_994].into_iter().enumerate() {
            let mut row = sample(index as u64, 0.0, 0.0);
            row.values.insert("northing".into(), Value::Integer(value));
            preview.sample(row, &request, limits()).unwrap();
        }
        preview = serde_json::from_slice(&serde_json::to_vec(&preview).unwrap()).unwrap();
        let result = payload(&preview, &request);
        assert_eq!(result.summaries[0].count, 2);
        assert_eq!(result.summaries[0].mean, Some(9_007_199_254_740_994.0));
        assert_eq!(result.summaries[0].median, Some(9_007_199_254_740_994.0));
    }

    #[test]
    fn scoped_points() {
        let scope = RecordScope::default().with_step(dokime_domain::model::identity::StepId::from_seed("measurement"));
        let request = PlotPayloadRequest {
            stream: "tracking".into(),
            plot_type: Some(PlotType::TimeSeries),
            y_keys: vec!["northing".into()],
            max_points: Some(2),
            scope: Some(scope.clone()),
            ..Default::default()
        };
        let mut preview = Preview::default();
        for index in 0..3 {
            let mut sample = sample(index, 0.0, index as f64);
            sample.scope = Some(if index == 1 {
                RecordScope::default().with_step(dokime_domain::model::identity::StepId::from_seed("other"))
            } else {
                scope.clone()
            });
            preview.sample(sample, &request, limits()).unwrap();
        }
        let result = payload(&preview, &request);
        assert_eq!(result.series[0].points.iter().map(|point| point.y.unwrap()).collect::<Vec<_>>(), vec![0.0, 2.0]);
        assert_eq!(result.cartesian.unwrap().point_count, 2);
    }
}
