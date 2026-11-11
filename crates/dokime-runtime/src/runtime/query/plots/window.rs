/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Window Queries Reuse Closed Time Buckets And Read Uncovered Evidence.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};

// Third Party Imports

// Local Imports
use super::{context::RuntimePlotEnvelopeContext, request::scope_selection};
use crate::evidence_runtime::TelemetrySelection;
use crate::plot::{PlotPayloadEnvelope, PlotPayloadRequest, PlotPoint, PlotSeries, PlotSeriesSummary, PlotWindow};
use crate::runtime::storage::telemetry::{ReadOptions, Sequence, visit_saved};
use crate::runtime::{Result, handle::RuntimeHandle};
use crate::storage::sqlite::data::telemetry::previews::{Bucket, Point, window_buckets};
use dokime_domain::model::time::Timestamp;
use harmos::storage::{ReadBatch, StorageCursor};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn select_viewport(selection: &mut TelemetrySelection, request: &PlotPayloadRequest) -> Result<()> {
    if let Some(window) =
        request.viewport.filter(|_| request.x_key.as_deref().is_none_or(|key| key == "timestamp") && request.x_expression.is_none())
    {
        selection.start = Some(timestamp((window.from * 1e9) as i128)?);
        let end = ((window.to * 1e9) as i128)
            .checked_add(1_000)
            .ok_or_else(|| crate::runtime::Error::Operation("plot viewport timestamp overflows".into()))?;
        selection.end = Some(timestamp(end)?);
    }
    Ok(())
}

#[tracing::instrument(level = "debug", name = "plot.window", skip_all)]
pub(super) async fn query(
    context: &RuntimePlotEnvelopeContext, runtime: &RuntimeHandle, request: &PlotPayloadRequest, at: &StorageCursor,
    codecs: &[crate::storage::telemetry::TypedCodec],
) -> Result<Option<ReadBatch<PlotPayloadEnvelope>>> {
    let Some(execution) = request.scope.as_ref().and_then(|scope| scope.test_execution_id.as_ref()) else { return Ok(None) };
    let plain = request.analysis.is_none()
        && request.filters.is_empty()
        && request.marker_record_types.is_empty()
        && request.series_by.is_none()
        && request.series_by_expression.is_none()
        && request.x_expression.is_none()
        && request.x_key.as_deref().is_none_or(|key| key == "timestamp")
        && request.z_expression.is_none()
        && request.z_stream.is_none()
        && request.z_field.is_none()
        && request.scope.as_ref().is_none_or(|scope| scope.step_id.is_none() && scope.criterion_id.is_none() && scope.session_id.is_none())
        && matches!(request.plot_type, None | Some(crate::document::PlotType::TimeSeries))
        && request.y_expressions.iter().all(|expression| {
            expression.transforms.is_empty()
                && matches!(&expression.source, crate::expressions::DataSource::Telemetry { stream, .. } if stream == &request.stream)
        });
    if !plain || (request.window_s.is_none() && request.viewport.is_none()) || request.max_points == Some(0) {
        return Ok(None);
    }
    let mut matching =
        codecs.iter().filter(|codec| codec.schema().id.as_str().strip_prefix("dokime.telemetry.") == Some(request.stream.as_str()));
    let Some(codec) = matching.next() else { return Ok(None) };
    if matching.next().is_some() {
        return Ok(None);
    }
    let mut normalized = request.clone();
    if !request.y_expressions.is_empty() {
        normalized.y_keys = request
            .y_expressions
            .iter()
            .filter_map(|expression| {
                if let crate::expressions::DataSource::Telemetry { field, .. } = &expression.source { Some(field.clone()) } else { None }
            })
            .collect();
        normalized.y_expressions.clear();
    }
    if super::telemetry_contract(context, &request.stream).is_some_and(|contract| {
        contract
            .columns
            .iter()
            .any(|column| normalized.y_keys.iter().any(|key| key == column.metadata.identifier.as_str()) && column.nullable)
    }) {
        return Ok(None);
    }
    if !crate::plot::stored::supports_codec(&normalized, codec) {
        return Ok(None);
    }
    let selection = TelemetrySelection {
        streams: vec![
            dokime_domain::model::identity::IdentifierPattern::new(request.stream.clone())
                .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?,
        ],
        scope: scope_selection(request),
        ..Default::default()
    };
    let owner = execution.to_string();
    let stream = request.stream.clone();
    let keys = normalized.y_keys.clone();
    let cut = at.clone();
    let (saved, latest) = runtime
        .local_storage
        .read(move |connection| {
            Ok((|| -> crate::storage::Result<_> {
                let head = crate::storage::sqlite::data::saved(connection)?;
                head.history.check(&cut.history)?;
                let saved = StorageCursor {
                    history: head.history,
                    sequence: head.sequence.min(cut.sequence),
                };
                let latest =
                    crate::storage::sqlite::data::telemetry::previews::latest_time(connection, &owner, &stream, &keys, saved.sequence)?;
                Ok((saved, latest))
            })())
        })
        .await??;
    let tail_samples = crate::runtime::storage::telemetry::collect_saved(
        runtime.storage(),
        at,
        &selection,
        runtime.schema_settings(),
        ReadOptions {
            codecs: Some(codecs),
            after: saved.sequence,
            sequence: Sequence::StoragePosition,
            ..Default::default()
        },
        &runtime.local_storage,
    )
    .await?;
    let latest = tail_samples
        .iter()
        .filter_map(|sample| crate::plot::parse_timestamp(sample.timestamp.as_str()).map(|time| time.unix_timestamp_nanos()))
        .chain(latest)
        .max();
    let Some(end) = latest else { return Ok(None) };
    let (from, to) = if let Some(window) = request.viewport {
        ((window.from * 1e9) as i128, (window.to * 1e9) as i128)
    } else {
        let from = end
            .checked_sub((request.window_s.unwrap_or_default() * 1e9) as i128)
            .ok_or_else(|| crate::runtime::Error::Operation("plot window timestamp overflows".into()))?;
        (from, end)
    };
    timestamp(from)?;
    timestamp(to)?;
    let maximum = request.max_points.unwrap_or(runtime.read_application_settings().await?.settings.report.plot_points as usize);
    let columns = request.panel_width.map(|width| width as usize).unwrap_or((maximum / 4).max(1)).min((maximum / 4).max(1));
    let base = runtime.local_storage.preview_settings().bucket_interval.as_nanos() as i128;
    let width = bucket_width(from, to, columns, base);
    let owner = execution.to_string();
    let stream = request.stream.clone();
    let keys = normalized.y_keys.clone();
    let cut = saved.clone();
    let (_, stored) = runtime
        .local_storage
        .read(move |connection| {
            Ok((|| -> crate::storage::Result<_> {
                let mut saved = crate::storage::sqlite::data::saved(connection)?;
                saved.history.check(&cut.history)?;
                saved.sequence = saved.sequence.min(cut.sequence);
                let mut fields = Vec::new();
                for key in keys {
                    let mut buckets = window_buckets(connection, &owner, &stream, &key, from, to, saved.sequence)?;
                    buckets.retain(|(start, end, _)| (end - start) <= width && start.div_euclid(width) == (end - 1).div_euclid(width));
                    let ranges = crate::storage::sqlite::data::telemetry::previews::window_raw_ranges(
                        connection,
                        &owner,
                        &stream,
                        &key,
                        from..=to,
                        saved.sequence,
                        width,
                    )?;
                    fields.push((key, buckets, ranges));
                }
                Ok((
                    StorageCursor {
                        history: saved.history,
                        sequence: saved.sequence,
                    },
                    fields,
                ))
            })())
        })
        .await??;
    let mut series = Vec::new();
    let mut summaries = Vec::new();
    let mut bucket_counts = BTreeMap::new();
    for (key, stored, mut ranges) in stored {
        let mut buckets = BTreeMap::<i128, Bucket>::new();
        let mut coverage = Vec::new();
        let mut stored = stored;
        stored.sort_by_key(|(start, end, _)| (*start, *end));
        for (start, end, bucket) in stored {
            // A Changed Interval Configuration Can Leave Overlapping Resolutions.
            if coverage.last().is_some_and(|(_, previous)| *previous > start) {
                continue;
            }
            coverage.push((start, end));
            merge(&mut buckets, start.div_euclid(width), bucket)?;
        }
        let mut seen = BTreeSet::new();
        let mut append = |sample: crate::telemetry::TelemetrySample| -> Result<()> {
            if !seen.insert(sample.sequence) {
                return Ok(());
            }
            let Some(value) =
                sample.values.get(&key).and_then(dokime_domain::model::values::numeric_value).filter(|value| value.is_finite())
            else {
                return Ok(());
            };
            let Some(time) = crate::plot::parse_timestamp(sample.timestamp.as_str()) else { return Ok(()) };
            let time = time.unix_timestamp_nanos();
            if time < from || time > to {
                return Ok(());
            }
            let point = Point { time, position: sample.sequence, value };
            merge(&mut buckets, time.div_euclid(width), Bucket::new(point))
        };
        ranges.sort_unstable();
        let mut reads: Vec<(i128, i128)> = Vec::new();
        for (mut start, end) in ranges {
            for (left, right) in &coverage {
                if *right <= start || *left >= end {
                    continue;
                }
                if start < *left {
                    reads.push((start, *left));
                }
                start = start.max(*right);
            }
            if start < end {
                reads.push((start, end));
            }
        }
        reads.sort_unstable();
        let mut merged: Vec<(i128, i128)> = Vec::new();
        for (start, end) in reads {
            if let Some(last) = merged.last_mut().filter(|last| last.1 >= start) {
                last.1 = last.1.max(end);
            } else {
                merged.push((start, end));
            }
        }
        for (start, end) in merged {
            let mut gap = selection.clone();
            gap.start = Some(timestamp(start)?);
            // Candidate Bounds Must Survive SystemTime's Platform Precision.
            let candidate_end =
                end.checked_add(1_000).ok_or_else(|| crate::runtime::Error::Operation("plot window timestamp overflows".into()))?;
            gap.end = Some(timestamp(candidate_end)?);
            visit_saved(
                runtime.storage(),
                &saved,
                &gap,
                runtime.schema_settings(),
                ReadOptions {
                    codecs: Some(codecs),
                    sequence: Sequence::StoragePosition,
                    ..Default::default()
                },
                &runtime.local_storage,
                &mut |sample: crate::telemetry::TelemetrySample| {
                    let Some(time) = crate::plot::parse_timestamp(sample.timestamp.as_str()) else { return Ok(()) };
                    let time = time.unix_timestamp_nanos();
                    if time < start || time >= end {
                        return Ok(());
                    }
                    append(sample)
                },
            )
            .await?;
        }
        // Tail Rows Can Be Late And Therefore Belong To Any Previously Closed Bucket.
        for sample in tail_samples.iter() {
            append(sample.clone())?;
        }
        let reduction = tracing::debug_span!("plot.window.reduce").entered();
        for bucket in buckets.values() {
            let key = (bucket.first.time as f64 / 1e9 / (width as f64 / 1e9)).floor() as i128;
            *bucket_counts.entry(key).or_insert(0) += bucket.count;
        }
        let moments = buckets.values().cloned().reduce(|mut left, right| {
            left.merge(right).expect("bounded evidence count");
            left
        });
        let count = moments.as_ref().map_or(0, |bucket| bucket.count as usize);
        let points = buckets
            .values()
            .flat_map(Bucket::points)
            .map(|point| {
                Ok(PlotPoint {
                    instant: None,
                    x: serde_json::Value::String(timestamp(point.time)?.to_string()),
                    y: Some(point.value),
                    z: None,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let label = request
            .y_expressions
            .iter()
            .find_map(|expression| {
                matches!(&expression.source, crate::expressions::DataSource::Telemetry { field, .. } if field == &key)
                    .then(|| expression.label.clone())
                    .flatten()
            })
            .unwrap_or_else(|| key.clone());
        summaries.push(summary(
            &key,
            moments.as_ref(),
            PlotWindow {
                from: from as f64 / 1e9,
                to: to as f64 / 1e9,
            },
        ));
        series.push(PlotSeries {
            key,
            label,
            unit: request.unit.clone(),
            raw_points: count,
            downsampled: points.len() < count,
            points,
            format: request.y_format.clone(),
        });
        drop(reduction);
    }
    let (run, cycle) = context.run_and_cycle_start(request.scope.as_ref());
    let format = super::plot_payload_context(request, super::telemetry_contract(context, &request.stream), run, cycle, maximum);
    let mut payload =
        crate::plot::project_retained_plot_payload(&[], Default::default(), request, &context.generated_at, &format)?.as_ref().clone();
    payload.bucket_width_s = Some(width as f64 / 1e9);
    payload.bucket_window = Some(PlotWindow {
        from: from as f64 / 1e9,
        to: to as f64 / 1e9,
    });
    payload.series = series;
    payload.bucket_counts = bucket_counts;
    payload.summaries = if request.defer_statistics { Vec::new() } else { summaries };
    payload.max_points = columns.saturating_add(2).saturating_mul(4);
    if let Some(contract) = super::telemetry_contract(context, &request.stream) {
        crate::plot::preview::index::decorate(&mut payload, std::slice::from_ref(contract), request);
    }
    let mut frame_request = request.clone();
    frame_request.panel_width = None;
    payload = tracing::debug_span!("plot.window.frame").in_scope(|| crate::plot::finish_plot_frame(payload, &frame_request))?;
    let diagnostics = crate::plot::plot_payload_diagnostics(&payload);
    Ok(Some(ReadBatch::new(PlotPayloadEnvelope {
        plot_id: request.plot_id.as_deref().unwrap_or(&request.stream).to_owned(),
        revision: at.sequence,
        payload,
        diagnostics,
        warnings: Vec::new(),
    })))
}

/* === Private Functions === */

fn timestamp(nanos: i128) -> Result<Timestamp> {
    let time =
        time::OffsetDateTime::from_unix_timestamp_nanos(nanos).map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
    let text =
        time.format(&time::format_description::well_known::Rfc3339).map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
    Ok(Timestamp::new(text))
}

fn bucket_width(from: i128, to: i128, columns: usize, base: i128) -> i128 {
    let desired = ((to - from).max(1) / columns.max(1) as i128).max(1);
    if desired < base {
        return desired;
    }
    let mut width = base;
    while width < desired {
        width *= 2;
    }
    width
}

fn merge(buckets: &mut BTreeMap<i128, Bucket>, key: i128, bucket: Bucket) -> Result<()> {
    if let Some(current) = buckets.get_mut(&key) {
        current.merge(bucket)?;
    } else {
        buckets.insert(key, bucket);
    }
    Ok(())
}

fn summary(key: &str, bucket: Option<&Bucket>, window: PlotWindow) -> PlotSeriesSummary {
    let mean = bucket.map(|bucket| bucket.sum / bucket.count as f64);
    PlotSeriesSummary {
        series_id: key.to_owned(),
        count: bucket.map_or(0, |bucket| bucket.count as usize),
        min: bucket.map(|bucket| bucket.min.value),
        max: bucket.map(|bucket| bucket.max.value),
        mean: bucket.zip(mean).map(|(bucket, mean)| bucket.scale * mean),
        median: None,
        rms: bucket.map(|bucket| bucket.scale * (bucket.squares / bucket.count as f64).sqrt()),
        std: bucket.zip(mean).map(|(bucket, mean)| bucket.scale * (bucket.squares / bucket.count as f64 - mean * mean).max(0.0).sqrt()),
        window: Some(window),
    }
}

/* === Tests === */
