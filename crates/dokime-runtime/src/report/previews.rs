/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Report Presentation Uses Only Durable Envelopes And Exact Numeric Summaries.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports

// Local Imports
use crate::document::capture::{Capture, CardBindingResult};
use crate::document::{DocumentCardBody, DocumentWarning};
use crate::expressions::{DataSource, DataTransform};
use crate::plot::{PlotPayload, PlotPayloadRequest, PlotPoint, PlotSeries, PlotSeriesSummary};
use crate::runtime::Result;
use crate::storage::sqlite::data::telemetry::{aggregates::NumericSummary, previews::FieldPreview};
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(in crate::report) fn indices(count: usize, limit: usize) -> impl Iterator<Item = usize> {
    (0..count.min(limit)).chain(count.saturating_sub(limit).max(limit)..count)
}

pub(crate) fn plot(
    request: &PlotPayloadRequest, fields: &[FieldPreview], summaries: &[NumericSummary],
    source: &crate::execution::context::captured::source::CapturedSource,
    execution: &dokime_domain::model::execution::result::TestExecution,
) -> Result<(super::value::Value, Option<String>)> {
    let maximum =
        request.max_points.unwrap_or(source.report_defaults.plot_points as usize).min(source.report_defaults.plot_points as usize);
    let mut keys = request.y_keys.clone();
    let simple = request.y_expressions.iter().all(|expression| {
        if let DataSource::Telemetry { stream, field } = &expression.source
            && crate::plot::stream_matches_alias(&request.stream, stream)
            && expression.transforms.is_empty()
        {
            keys.push(field.clone());
            true
        } else {
            false
        }
    });
    let mean = request
        .analysis
        .as_ref()
        .is_some_and(|selection| selection.id == "dokime.analysis.mean" && selection.version == 1 && selection.parameters.is_empty());
    let supported = simple
        && maximum >= 4
        && (request.analysis.is_none() || mean)
        && request.filters.is_empty()
        && request.series_by.is_none()
        && request.series_by_expression.is_none()
        && request.z_expression.is_none()
        && request.z_stream.is_none()
        && request.z_field.is_none()
        && request.x_expression.is_none()
        && request.x_key.as_deref().is_none_or(|key| key == "timestamp")
        && request.summary_window.is_none()
        && request.scope.as_ref().is_none_or(|scope| scope.step_id.is_none() && scope.criterion_id.is_none())
        && matches!(request.plot_type, None | Some(crate::document::PlotType::TimeSeries));
    let mut warning = (!supported).then(|| format!("Plot {} requires history-dependent analysis, a transform, grouping, filtering, a partial scope, a non-time axis, or fewer than four display points. It is unavailable in the bounded report; full data remains in execution {} evidence.", request.plot_id.as_deref().unwrap_or(&request.stream), execution.test_execution_id));
    if supported && !request.marker_record_types.is_empty() {
        warning =
            Some("Record markers are omitted from the bounded plot; inspect the first/last record preview and complete evidence.".into());
    }
    let mut payload = PlotPayload {
        computed: None,
        stream: request.stream.clone(),
        x_key: "timestamp".into(),
        series_by: None,
        context: BTreeMap::new(),
        series: Vec::new(),
        cartesian: None,
        analysis: None,
        markers: Vec::new(),
        max_points: maximum,
        window_s: request.window_s,
        generated_at: source.captured_at.to_string(),
        x_format: request.x_format.clone(),
        run_started_at: execution.started_at.as_ref().map(ToString::to_string),
        cycle_started_at: source.context.cycle.cycle.started_at.as_ref().map(ToString::to_string),
        summaries: Vec::new(),
    };
    if !supported && let Some(message) = &warning {
        payload.analysis = Some(crate::plot::PlotAnalysis::Error { message: message.clone() });
    }
    let mut computed = Vec::new();
    if supported {
        // Series Follow The Authored Key Order, Not The Preview Catalog's Field Order.
        let mut ordered = Vec::new();
        for key in &keys {
            if ordered.contains(&key) {
                continue;
            }
            ordered.push(key);
        }
        let selected = ordered.into_iter().flat_map(|key| {
            fields.iter().filter(move |field| crate::plot::stream_matches_alias(&field.stream, &request.stream) && field.field == *key)
        });
        for field in selected {
            let mut envelope = field.envelope.clone();
            envelope.fit(maximum / 4)?;
            let count = envelope.buckets.values().map(|bucket| bucket.count).sum::<u64>() as usize;
            let mut points = Vec::new();
            for bucket in envelope.buckets.values() {
                for point in bucket.points() {
                    let time = time::OffsetDateTime::from_unix_timestamp_nanos(point.time)
                        .map_err(|_| super::query::invalid("invalid envelope time"))?;
                    let stamp = time.format(&time::format_description::well_known::Rfc3339).map_err(crate::storage::Error::from)?;
                    points.push(PlotPoint {
                        x: serde_json::Value::String(stamp),
                        y: Some(point.value),
                        z: None,
                    });
                }
            }
            let summary = summaries.iter().find(|summary| summary.stream == field.stream && summary.field == field.field);
            let stat = |name| summary.and_then(|summary| summary.statistics.iter().find(|(key, _)| *key == name).map(|(_, value)| *value));
            let first_time = field.envelope.buckets.values().next().map(|bucket| bucket.first.time).unwrap_or_default();
            let last_time = field.envelope.buckets.values().next_back().map(|bucket| bucket.last.time).unwrap_or(first_time);
            let label = request
                .y_expressions
                .iter()
                .find_map(|expression| {
                    let DataSource::Telemetry { field: key, .. } = &expression.source else { return None };
                    (key == &field.field).then_some(expression.label.as_ref()).flatten().cloned()
                })
                .unwrap_or_else(|| field.field.clone());
            payload.summaries.push(PlotSeriesSummary {
                series_id: field.field.clone(),
                count,
                min: stat("min"),
                max: stat("max"),
                mean: stat("mean"),
                median: stat("median"),
                std: stat("std"),
                rms: stat("rms"),
                window: Some(crate::plot::PlotWindow {
                    from: first_time as f64 / 1e9,
                    to: last_time as f64 / 1e9,
                }),
            });
            let unit = source
                .telemetry_contracts
                .iter()
                .find(|(stream, _)| stream.as_str() == field.stream)
                .and_then(|(_, versions)| versions.values().next())
                .and_then(|contract| contract.columns.iter().find(|column| column.metadata.identifier.as_str() == field.field))
                .and_then(|column| column.unit.as_ref())
                .map(|unit| unit.symbol.clone());
            if mean && let Some(value) = stat("mean") {
                let first = field.envelope.buckets.values().next().map(|bucket| bucket.first.time).unwrap_or_default();
                let last = field.envelope.buckets.values().next_back().map(|bucket| bucket.last.time).unwrap_or(first);
                computed.push(crate::plot::PlotComputedSeries {
                    key: field.field.clone(),
                    label: label.clone(),
                    origin: None,
                    input_samples: count,
                    first_axis: first as f64 / 1e9,
                    last_axis: last as f64 / 1e9,
                    output: crate::analysis::Response {
                        outputs: vec![crate::analysis::Values {
                            key: "value".into(),
                            unit: unit.clone().unwrap_or_default(),
                            values: vec![value],
                            ..Default::default()
                        }],
                    },
                    display_indices: BTreeMap::new(),
                });
            }
            let downsampled = points.len() < count;
            payload.series.push(PlotSeries {
                key: field.field.clone(),
                label,
                unit,
                points,
                raw_points: count,
                downsampled,
                format: request.y_format.clone(),
            });
        }
    }
    if supported && mean {
        let selection = request.analysis.as_ref().expect("mean selection").clone();
        let definition = crate::plot::builtins::definitions()
            .map_err(|_| super::query::invalid("builtin analysis catalog is invalid"))?
            .into_iter()
            .find(|definition| definition.id == selection.id)
            .ok_or_else(|| super::query::invalid("mean definition is missing"))?;
        payload.computed = Some(crate::plot::PlotComputation {
            definition,
            selection,
            source: crate::plot::PlotEvidence {
                stream: request.stream.clone(),
                scope: request.scope.clone(),
                position: source.evidence_through.clone(),
                window_s: None,
                fields: keys,
                x_key: "timestamp".into(),
            },
            series: computed,
        });
        payload.series.clear();
    }
    Ok((super::value::Value::from_serializable(&payload, source.report_defaults.value_depth as usize)?, warning))
}

pub(crate) fn summaries(
    capture: &mut Capture, exact: &[NumericSummary], fields: &[FieldPreview], context: crate::document::input::Context<'_>,
) {
    for binding in &mut capture.cards {
        let CardBindingResult::Summary { groups } = &mut binding.result else { continue };
        let definition = capture
            .definition
            .cards
            .iter()
            .map(|card| (&card.metadata, &card.filters, &card.body))
            .chain(
                capture
                    .definition
                    .sections
                    .iter()
                    .flat_map(|section| section.cards.iter().map(|card| (&card.metadata, &card.filters, &card.body))),
            )
            .find(|(metadata, _, _)| metadata.identifier == binding.source_id);
        let Some((_, filters, body)) = definition else { continue };
        let DocumentCardBody::TelemetrySummary { summary } = body else { continue };
        for (group, source) in groups.iter_mut().zip(&summary.groups) {
            for (tile, source) in group.tiles.iter_mut().zip(&source.tiles) {
                let needs_history = |expression: &crate::expressions::DataExpression| {
                    crate::document::binding::expression_needs_telemetry(expression, context)
                        || matches!(expression.source, DataSource::Record { .. })
                };
                let history =
                    needs_history(&source.value) || filters.iter().chain(&source.filters).any(|filter| needs_history(&filter.expression));
                if !history {
                    continue;
                }
                let result = (|| {
                    if !filters.is_empty() || !source.filters.is_empty() {
                        return None;
                    }
                    let DataSource::Telemetry { stream, field } = &source.value.source else { return None };
                    let name = match source.value.transforms.as_slice() {
                        [] => {
                            return fields
                                .iter()
                                .find(|preview| crate::plot::stream_matches_alias(&preview.stream, stream) && preview.field == *field)
                                .and_then(|preview| preview.envelope.buckets.values().next_back())
                                .map(|bucket| Value::Number(bucket.last.value));
                        }
                        [DataTransform::Aggregate(aggregate)] => aggregate.name().to_owned(),
                        _ => return None,
                    };
                    let summary = exact
                        .iter()
                        .find(|summary| crate::plot::stream_matches_alias(&summary.stream, stream) && summary.field == *field)?;
                    summary.statistics.iter().find(|(key, _)| *key == name).map(|(_, value)| Value::Number(*value))
                })();
                tile.value = result;
                tile.unavailable = tile
                    .value
                    .is_none()
                    .then(|| "This history-dependent summary has no exact incremental index; full evidence is retained.".into());
                tile.state =
                    tile.value.as_ref().and_then(|value| crate::document::resolve::matching_summary_state(value, &source.conditions));
            }
        }
    }
}

pub(crate) fn bound_definition(definition: &mut crate::document::DocumentDefinition, limit: usize) -> bool {
    fn sections(nodes: &mut Vec<crate::document::DocumentNode>) -> bool {
        let mut removed = false;
        nodes.retain_mut(|node| {
            let crate::document::DocumentNode::Section(section) = node else { return true };
            if section.repeat.is_some() {
                removed = true;
                return false;
            }
            removed |= sections(&mut section.children);
            true
        });
        removed
    }
    let mut removed = sections(&mut definition.layout.nodes);
    definition.sections.retain(|section| {
        if section.repeat.is_some() {
            removed = true;
            return false;
        }
        true
    });
    for body in definition
        .cards
        .iter_mut()
        .map(|card| &mut card.body)
        .chain(definition.sections.iter_mut().flat_map(|section| section.cards.iter_mut().map(|card| &mut card.body)))
    {
        if let crate::document::DocumentCardBody::Table { rows } = body
            && rows.len() > limit * 2
        {
            rows.drain(limit..rows.len() - limit);
        }
    }
    removed
}

pub(crate) fn warning(message: String) -> DocumentWarning {
    DocumentWarning {
        code: Identifier::new("report.bounded_evidence").expect("static identifier"),
        message,
    }
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::sqlite::data::telemetry::previews::{Envelope, Point};

    #[test]
    fn table_previews() {
        assert_eq!(indices(0, 3).collect::<Vec<_>>(), Vec::<usize>::new());
        assert_eq!(indices(5, 3).collect::<Vec<_>>(), vec![0, 1, 2, 3, 4]);
        assert_eq!(indices(1000, 3).collect::<Vec<_>>(), vec![0, 1, 2, 997, 998, 999]);
    }

    #[test]
    fn report_envelope() {
        let captured: crate::report::CapturedExecutionReportInput =
            serde_json::from_str(include_str!("fixtures/execution-report-input.json")).unwrap();
        let source = captured.source(&[]);
        let execution = captured.execution().unwrap();
        let mut envelope = Envelope {
            origin: 0,
            width: 1,
            buckets: BTreeMap::new(),
        };
        let mut fixture = Vec::new();
        for position in 0..4096 {
            let value = match position {
                25 => 1_000_000.0,
                45 => -1_000_000.0,
                _ => (position % 97) as f64,
            };
            fixture.push(value);
            envelope.push(Point { time: position as i128, position, value }, 256).unwrap();
        }
        // Reference Each Of The Eight Final Time Intervals By Scanning The Fixture.
        let expected: Vec<_> = fixture
            .chunks(512)
            .flat_map(|bucket| {
                let min = bucket.iter().enumerate().min_by(|(_, left), (_, right)| left.total_cmp(right)).unwrap().0;
                let max = bucket.iter().copied().fold(f64::NEG_INFINITY, f64::max);
                let max = bucket.iter().position(|value| *value == max).unwrap();
                let mut positions = vec![0, min, max, bucket.len() - 1];
                positions.sort_unstable();
                positions.dedup();
                positions.into_iter().map(|position| bucket[position]).collect::<Vec<_>>()
            })
            .collect();
        let fields = [FieldPreview {
            stream: "tracking".into(),
            field: "distance_m".into(),
            envelope,
        }];
        let request = PlotPayloadRequest {
            stream: "tracking".into(),
            y_keys: vec!["distance_m".into()],
            max_points: Some(32),
            plot_type: Some(crate::document::PlotType::TimeSeries),
            ..Default::default()
        };
        let (payload, warning) = plot(&request, &fields, &[], &source, execution).unwrap();
        assert!(warning.is_none());
        let payload = serde_json::to_value(payload).unwrap();
        let points: Vec<_> = payload["series"][0]["points"].as_array().unwrap().iter().map(|point| point["y"].as_f64().unwrap()).collect();
        assert_eq!(points, expected);
        assert!(points.len() <= 32);
        assert!(points.contains(&1_000_000.0));
        assert!(points.contains(&-1_000_000.0));
        assert_eq!(payload["series"][0]["raw_points"], 4096);
        let mean_request = PlotPayloadRequest {
            analysis: Some(crate::analysis::Selection {
                id: "dokime.analysis.mean".into(),
                version: 1,
                parameters: Default::default(),
            }),
            ..request.clone()
        };
        let summaries = [NumericSummary {
            stream: "tracking".into(),
            field: "distance_m".into(),
            statistics: vec![("mean".into(), 12.125)],
        }];
        let (mean, warning) = plot(&mean_request, &fields, &summaries, &source, execution).unwrap();
        assert!(warning.is_none());
        let mean = serde_json::to_value(mean).unwrap();
        assert_eq!(mean["computed"]["series"][0]["output"]["outputs"][0]["values"][0], 12.125);
        assert_eq!(mean["computed"]["series"][0]["input_samples"], 4096);
        assert!(mean["series"].as_array().unwrap().is_empty());

        let request = PlotPayloadRequest {
            x_key: Some("distance_m".into()),
            ..request
        };
        let (payload, warning) = plot(&request, &fields, &[], &source, execution).unwrap();
        assert!(warning.is_some());
        assert!(serde_json::to_value(payload).unwrap()["series"].as_array().unwrap().is_empty());
    }
}
