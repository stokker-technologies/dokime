/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Query Plot Snapshots

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Error, Result, client::Context};
use dokime_protocol::v1;
use dokime_storage::NumericWindow;
use prost::Message;
use prost_reflect::DescriptorPool;
use sha2::{Digest, Sha256};
use std::num::NonZeroUsize;

/* === Constants === */

pub const ID: &str = "dokime.plots.query_snapshots";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn declaration(settings: &super::ClientSettings) -> Result<harmos_protocol::v1::OperationContract> {
    super::declaration::<v1::PlotsQuerySnapshotsRequest, v1::PlotSnapshotsReply>(ID, settings.read_timeout)
}
pub async fn handle(context: &Context, request: v1::PlotsQuerySnapshotsRequest) -> Result<v1::PlotSnapshotsReply> {
    super::complete(ID,context.settings.read_timeout,async {
        let mut reply=v1::PlotSnapshotsReply::default();
        for request in request.requests {
            reply.envelopes.push(query_one(context,request).await?);
            if reply.encoded_len()>context.settings.message_bytes {return Err(Error::Invalid("Query Plot Snapshots: Lower the Resolution or Split the Batch to Fit One Message; Use Watch for Visible-First Delivery".into()));}
        }
        Ok(reply)
    }).await
}
pub(super) async fn query_one(context: &Context, request: v1::PlotRequest) -> Result<v1::PlotEnvelope> {
    let (store, _cancel_on_drop) = context.host.store().read_session()?;
    let context = context.clone();
    tokio::task::spawn_blocking(move || {
        let execution = super::projection::execution_scope(request.scope.as_ref())?;
        let mut cut = store.metadata(execution)?;
        if let Some(at) = &request.at {
            let history = v1::StoreHistory {
                store: context.host.identity().backend_id,
                epoch: execution.into(),
            };
            if at.history.as_ref() != Some(&history) || at.sequence > cut.frontier {
                return Err(Error::Invalid("Query Plot: Refresh the Document's Evidence Cut for This Backend".into()));
            }
            cut.frontier = at.sequence;
        }
        snapshot_at(&store, &cut, &request, context.settings.page_items, context.host.now_ns(), &context.host.identity().epoch)
    })
    .await
    .map_err(|error| Error::Invalid(format!("Query Plot: Retry After the Query Worker Failed: {error}")))?
}
pub fn snapshot_at(
    store: &dokime_storage::Store, cut: &dokime_storage::ExecutionCut, request: &v1::PlotRequest, maximum: usize, now_ns: i64, epoch: &str,
) -> Result<v1::PlotEnvelope> {
    let execution = cut.execution.identifier.as_str();
    let original = request;
    let selection_key = Sha256::digest(request.encode_to_vec()).iter().map(|byte| format!("{byte:02x}")).collect();
    let selected = store.plot_selection(cut, request)?;
    let request = &selected.request;
    let catalog = harmos_protocol::v1::Catalog::decode(cut.execution.catalog.as_slice())?;
    let pool = DescriptorPool::from_file_descriptor_set(
        catalog.types.clone().ok_or_else(|| Error::Invalid("Query Plot: Restore the Frozen Stream Descriptors".into()))?,
    )
    .map_err(|error| Error::Invalid(error.to_string()))?;
    let stream = catalog
        .streams
        .iter()
        .find(|stream| stream.id == request.stream)
        .ok_or_else(|| Error::Invalid(format!("Query Plot: Select a Declared Stream {}", request.stream)))?;
    let descriptor =
        pool.get_message_by_name(&stream.row).ok_or_else(|| Error::Invalid("Query Plot: Restore the Frozen Row Descriptor".into()))?;
    let mut fields = request.y_keys.clone();
    let mut aggregate = None;
    for expression in &request.y_expressions {
        match expression.kind.as_ref() {
            Some(v1::data_expression::Kind::Stream(reference)) if reference.identifier == stream.id => fields.push(reference.field.clone()),
            Some(v1::data_expression::Kind::Aggregate(expression)) => {
                let Some(v1::data_expression::Kind::Stream(reference)) = expression.input.as_ref().and_then(|input| input.kind.as_ref())
                else {
                    return Err(Error::Invalid("Query Plot: Aggregate a Numeric Stream Column".into()));
                };
                if reference.identifier != stream.id {
                    return Err(Error::Invalid("Query Plot: Select Columns from One Stream".into()));
                }
                fields.push(reference.field.clone());
                aggregate = v1::Aggregate::try_from(expression.kind).ok();
            }
            _ => return Err(Error::Invalid("Query Plot: Select Numeric Columns from the Requested Stream".into())),
        }
    }
    if fields.is_empty() {
        fields.extend(
            descriptor
                .fields()
                .filter(|field| !field.is_list() && !field.is_map() && numeric(&field.kind()))
                .map(|field| field.name().to_owned()),
        );
    }
    fields.sort();
    fields.dedup();
    for field in &fields {
        if !selected.projected.contains(field)
            && descriptor.get_field_by_name(field).is_none_or(|field| field.is_list() || field.is_map() || !numeric(&field.kind()))
        {
            return Err(Error::Invalid(format!("Query Plot: Select a Declared Numeric Scalar Field {}.{field}", stream.id)));
        }
    }
    if fields.is_empty() {
        return Err(Error::Invalid("Query Plot: Select at Least One Numeric Field".into()));
    }

    let technique = v1::Downsample::try_from(request.technique)
        .map_err(|_| Error::Invalid("Query Plot: Select a Declared Downsampling Technique".into()))?;
    if matches!(technique, v1::Downsample::Raw | v1::Downsample::Lttb) {
        return Err(Error::Invalid("Query Plot: Select Auto or MinMax Until Raw and LTTB Windows Are Connected".into()));
    }
    if request.max_points == Some(0) {
        return Err(Error::Invalid("Query Plot: Select a Positive Display Resolution".into()));
    }
    if !request.filters.is_empty() {
        return Err(Error::Invalid("Query Plot: Select an Unfiltered Stream Window".into()));
    }
    if request.analysis.is_some() {
        return Err(Error::Invalid("Query Plot: Use a Built-In Reduction Until Plugin Analysis Invocation Is Connected".into()));
    }
    if request.z_stream.as_ref().is_some_and(|selected| selected != &stream.id) {
        return Err(Error::Invalid("Query Plot: Select Coordinate Columns from One Stream".into()));
    }
    if aggregate.is_some_and(|kind| {
        !matches!(kind, v1::Aggregate::Count | v1::Aggregate::Sum | v1::Aggregate::Min | v1::Aggregate::Max | v1::Aggregate::Mean)
    }) {
        return Err(Error::Invalid("Query Category Plot: Select Count, Sum, Min, Max or Mean".into()));
    }
    if request.panel_width == Some(0) || request.panel_height == Some(0) {
        return Err(Error::Invalid("Query Plot: Supply Positive Panel Dimensions".into()));
    }
    let kind = v1::PlotKind::try_from(request.plot_type).map_err(|_| Error::Invalid("Query Plot: Select a Declared Plot Kind".into()))?;
    let field_reference = |expression: Option<&v1::DataExpression>, fallback: Option<&str>| -> Result<Option<String>> {
        match expression.and_then(|expression| expression.kind.as_ref()) {
            Some(v1::data_expression::Kind::Stream(reference)) if reference.identifier == stream.id => Ok(Some(reference.field.clone())),
            None => Ok(fallback.map(str::to_owned)),
            _ => Err(Error::Invalid("Query Plot: Select Coordinate Columns from the Requested Stream".into())),
        }
    };
    let x = field_reference(request.x_expression.as_ref(), request.x_key.as_deref())?;
    let group = field_reference(request.series_by_expression.as_ref(), request.series_by.as_deref())?;
    let z = field_reference(request.z_expression.as_ref(), request.z_field.as_deref())?;
    if request.hole.is_some_and(|hole| !hole.is_finite() || !(0.0..1.0).contains(&hole)) {
        return Err(Error::Invalid("Query Pie: Choose a Hole Fraction in [0, 1)".into()));
    }
    if request.pareto && (kind != v1::PlotKind::Bar || request.arrangement.is_some() || group.is_some()) {
        return Err(Error::Invalid("Query Pareto: Select One Ungrouped Bar Series".into()));
    }
    if request.arrangement.is_some() && (kind != v1::PlotKind::Bar || group.is_none()) {
        return Err(Error::Invalid("Query Grouped Bars: Select a Series Column".into()));
    }
    if request.xy_interval.is_some() && (kind != v1::PlotKind::Xy || request.xy_interval != Some(v1::XyInterval::Sigma as i32)) {
        return Err(Error::Invalid("Query XY Intervals: Select Sigma on an XY Plot".into()));
    }
    if fields.len() != 1 && (matches!(kind, v1::PlotKind::Pie | v1::PlotKind::Contour | v1::PlotKind::GridChart) || request.pareto) {
        return Err(Error::Invalid("Query Plot: Select Exactly One Value Column for This Kind".into()));
    }
    let requested = request.max_points.unwrap_or(maximum as u64).max(4).min(maximum as u64) as usize;
    let display = request.panel_width.map(|width| width as usize * 4).unwrap_or(requested).min(requested).max(4);
    let bins = NonZeroUsize::new((display / 4).max(1)).unwrap();
    let (after, through) = store.step_range(
        execution,
        request.scope.as_ref().and_then(|scope| scope.step_id.as_deref()).unwrap_or_default(),
        cut.frontier,
        NonZeroUsize::new(maximum).unwrap(),
    )?;
    let (start_ns, end_ns) = if let Some(viewport) =
        request.viewport.as_ref().filter(|_| matches!(kind, v1::PlotKind::TimeSeries | v1::PlotKind::Unspecified))
    {
        (ns(viewport.from)?, ns(viewport.to)?)
    } else {
        let bounds = store.numeric_bounds(execution, &stream.id, &fields[0], through)?.unwrap_or((0, 0));
        if let Some(window_s) = request.window_s {
            if !window_s.is_finite() || window_s < 0.0 {
                return Err(Error::Invalid("Query Plot: Choose a Finite Nonnegative Time Window".into()));
            }
            (bounds.0.max(bounds.1.saturating_sub(ns(window_s)?)), bounds.1)
        } else {
            bounds
        }
    };
    if start_ns > end_ns {
        return Err(Error::Invalid("Query Plot: Choose Increasing Viewport Bounds".into()));
    }
    let resolution = ((i128::from(end_ns) - i128::from(start_ns) + 1 + bins.get() as i128 - 1) / bins.get() as i128).max(1) as u64;
    let mut payload = v1::PlotPayload {
        stream: stream.id.clone(),
        x_key: x.clone().unwrap_or_else(|| "timestamp".into()),
        series_by: group.clone(),
        max_points: display as u64,
        window_s: request.window_s,
        generated_at: super::projection::timestamp(now_ns).to_string(),
        bucket_width_s: Some(resolution as f64 / 1e9),
        bucket_window: Some(v1::PlotWindow {
            from: start_ns as f64 / 1e9,
            to: end_ns as f64 / 1e9,
        }),
        ..Default::default()
    };
    let mut frame = v1::PlotFrame {
        mode: Some(v1::PlotCartesianMode::Faceted as i32),
        ..Default::default()
    };
    let mut examined = 0;
    let mut histograms = Vec::new();
    let mut boxes = Vec::new();
    for field in fields {
        let window = NumericWindow {
            execution,
            stream: &stream.id,
            field: &field,
            after,
            through,
            start_ns,
            end_ns,
            bins,
        };

        let mut column_groups = None;
        let count = if let Some(group_field) = group.as_deref().filter(|_| {
            matches!(kind, v1::PlotKind::TimeSeries | v1::PlotKind::Unspecified | v1::PlotKind::Histogram | v1::PlotKind::BoxPlot)
        }) {
            let groups = store.plot_group_values(window, group_field, (display / 4).max(1))?;
            let mut total = 0;
            let mut counts = Vec::new();
            let group_display = (display / groups.len().max(1)).max(4);
            for group in groups {
                store.select_plot_group(Some(&group))?;
                let label = group.label.clone();
                let group_window = NumericWindow {
                    bins: NonZeroUsize::new((group_display / 4).max(1)).unwrap(),
                    ..window
                };
                let count = if kind == v1::PlotKind::Histogram {
                    let mut histogram = store.histogram_window(
                        group_window,
                        request.histogram.as_ref().map_or(group_display, |options| options.bins as usize).clamp(1, group_display),
                    )?;
                    histogram.key = format!("{field}/{label}");
                    let count = histogram.count;
                    histograms.push(histogram);
                    count
                } else if kind == v1::PlotKind::BoxPlot {
                    let mut summary = store.box_window(group_window, group_display.saturating_sub(1))?;
                    summary.key = format!("{field}/{label}");
                    summary.label = label.clone();
                    let count = summary.count;
                    boxes.push(summary);
                    count
                } else {
                    let (column, mut summary, count) = store.time_series_window(group_window)?;
                    summary.series_id = format!("{field}/{label}");
                    summary.window = payload.bucket_window;
                    payload.summaries.push(summary);
                    frame.point_count += column.y.len() as u64;
                    frame.series.push(column);
                    count
                };
                counts.push((label, count));
                total += count;
            }
            store.select_plot_group(None)?;
            column_groups = Some(counts);
            total
        } else if kind == v1::PlotKind::Histogram {
            let count = request
                .histogram
                .as_ref()
                .map(|options| options.bins as usize)
                .filter(|bins| *bins != 0)
                .unwrap_or(bins.get())
                .min(display)
                .max(1);
            let mut histogram = store.histogram_window(window, count)?;
            histogram.key = field.clone();
            let count = histogram.count;
            histograms.push(histogram);
            count
        } else if kind == v1::PlotKind::BoxPlot {
            let mut summary = store.box_window(window, display.saturating_sub(1))?;
            summary.key = field.clone();
            summary.label = field.clone();
            let count = summary.count;
            boxes.push(summary);
            count
        } else if matches!(
            kind,
            v1::PlotKind::Xy | v1::PlotKind::Trajectory | v1::PlotKind::Bar | v1::PlotKind::Pie | v1::PlotKind::Contour
        ) {
            let x = x.as_deref().ok_or_else(|| Error::Invalid("Query Plot: Select an X Coordinate or Category Column".into()))?;
            let settings = dokime_storage::KindWindow {
                window,
                x: Some(x),
                z: z.as_deref(),
                group: group.as_deref(),
                limit: if kind == v1::PlotKind::Contour {
                    display.min(bins.get().saturating_mul(request.panel_height.unwrap_or(display as u32) as usize))
                } else {
                    display
                },
            };
            if request.xy_interval.is_some() {
                let options = request.interval.unwrap_or(v1::PlotIntervalOptions {
                    bin_width: 1.0,
                    sigma: 1.0,
                    clip_to_observed: false,
                });
                let (analysis, count) = store.sigma_window(settings, options)?;
                payload.analysis = Some(v1::PlotAnalysis {
                    result: Some(v1::PlotAnalysisResult {
                        kind: Some(v1::plot_analysis_result::Kind::SigmaInterval(analysis)),
                    }),
                });
                count
            } else if kind == v1::PlotKind::Contour {
                if z.is_none() {
                    return Err(Error::Invalid("Query Contour: Select a Numeric Z Column".into()));
                }
                let (grid, count) = store.contour_scaled_window(
                    settings,
                    request.viewport.as_ref(),
                    request.y_viewport.as_ref(),
                    request.reduction.and_then(|value| v1::Reduction::try_from(value).ok()).unwrap_or(v1::Reduction::Mean),
                    request.x_scale == v1::AxisScale::Log as i32,
                    request.y_scale == v1::AxisScale::Log as i32,
                )?;
                payload.analysis = Some(v1::PlotAnalysis {
                    result: Some(v1::PlotAnalysisResult {
                        kind: Some(v1::plot_analysis_result::Kind::Contour(grid)),
                    }),
                });
                count
            } else {
                let (mut series, count, groups) = if matches!(kind, v1::PlotKind::Bar | v1::PlotKind::Pie) {
                    store.category_window(settings, aggregate, kind == v1::PlotKind::Pie || request.pareto)?
                } else {
                    store.cartesian_window(settings)?
                };
                if request.pareto {
                    for column in &mut series.series {
                        let mut indices = (0..column.y.len()).collect::<Vec<_>>();
                        indices.sort_by(|left, right| {
                            column.y[*right].value.unwrap_or_default().total_cmp(&column.y[*left].value.unwrap_or_default())
                        });
                        column.x = indices.iter().map(|index| column.x[*index].clone()).collect();
                        column.y = indices.iter().map(|index| column.y[*index]).collect();
                    }
                }
                column_groups = Some(groups);
                frame.point_count += series.point_count;
                frame.series.extend(series.series);
                count
            }
        } else if kind == v1::PlotKind::GridChart {
            let grid =
                request.grid.as_ref().ok_or_else(|| Error::Invalid("Query Grid Chart: Supply Row, Column and Frame Bindings".into()))?;
            let row = field_reference(grid.row.as_ref(), None)?;
            let column = field_reference(grid.column.as_ref(), None)?;
            let instant = field_reference(grid.frame.as_ref(), None)?;
            let reduction = request
                .reduction
                .and_then(|value| v1::Reduction::try_from(value).ok())
                .ok_or_else(|| Error::Invalid("Query Grid Chart: Select an Explicit Reducer".into()))?;
            let (frames, count) = store.grid_window(
                dokime_storage::KindWindow {
                    window,
                    x: row.as_deref(),
                    z: column.as_deref(),
                    group: instant.as_deref(),
                    limit: display,
                },
                grid,
                reduction,
            )?;
            payload.analysis = Some(v1::PlotAnalysis {
                result: Some(v1::PlotAnalysisResult {
                    kind: Some(v1::plot_analysis_result::Kind::GridChart(frames)),
                }),
            });
            count
        } else if kind == v1::PlotKind::FrequencySeries {
            let options = request.fft.unwrap_or_default();
            let (column, count) = store.spectrum_window(
                dokime_storage::KindWindow {
                    window,
                    x: None,
                    z: None,
                    group: None,
                    limit: maximum,
                },
                &options,
            )?;
            frame.point_count += column.y.len() as u64;
            frame.series.push(column);
            payload.x_key = "frequency".into();
            payload.bucket_width_s = None;
            count
        } else {
            let (column, mut summary, count) = store.time_series_window(window)?;
            summary.series_id = field.clone();
            summary.window = payload.bucket_window;
            payload.summaries.push(summary);
            frame.point_count += column.y.len() as u64;
            frame.series.push(column);
            count
        };
        if let Some(summary_window) = request.summary_window.as_ref().filter(|_| group.is_none()) {
            let from = ns(summary_window.from)?;
            let to = ns(summary_window.to)?;
            if from > to {
                return Err(Error::Invalid("Query Plot Summary: Choose Increasing Source-Time Bounds".into()));
            }
            let summary_window = NumericWindow { start_ns: from, end_ns: to, ..window };
            let stats = store.numeric_statistics(&summary_window)?;
            payload.summaries.retain(|summary| summary.series_id != field);
            payload.summaries.push(v1::PlotSeriesSummary {
                series_id: field.clone(),
                count: stats.count,
                min: (stats.count > 0).then_some(stats.min),
                max: (stats.count > 0).then_some(stats.max),
                mean: (stats.count > 0).then_some(stats.mean),
                median: (stats.count > 0).then_some(stats.median),
                std: (stats.count > 0).then_some(stats.stddev),
                rms: (stats.count > 0).then_some((stats.stddev * stats.stddev + stats.mean * stats.mean).sqrt()),
                window: Some(*request.summary_window.as_ref().unwrap()),
            });
        }
        examined += count;
        if let Some(groups) = column_groups {
            for (group, raw_points) in groups {
                payload.series.push(v1::PlotSeries {
                    key: if group.is_empty() { field.clone() } else { format!("{field}/{group}") },
                    label: if group.is_empty() { field.clone() } else { group },
                    unit: request.unit.clone(),
                    raw_points,
                    downsampled: raw_points > display as u64,
                    ..Default::default()
                });
            }
        } else {
            payload.series.push(v1::PlotSeries {
                key: field.clone(),
                label: field,
                unit: request.unit.clone(),
                raw_points: count,
                downsampled: count > display as u64,
                ..Default::default()
            });
        }
    }
    if kind == v1::PlotKind::Histogram {
        payload.analysis = Some(v1::PlotAnalysis {
            result: Some(v1::PlotAnalysisResult {
                kind: Some(v1::plot_analysis_result::Kind::Histogram(v1::PlotHistogramAnalysis { series: histograms })),
            }),
        });
    } else if kind == v1::PlotKind::BoxPlot {
        payload.analysis = Some(v1::PlotAnalysis {
            result: Some(v1::PlotAnalysisResult {
                kind: Some(v1::plot_analysis_result::Kind::BoxPlot(v1::PlotBoxSummaryAnalysis { series: boxes, total: examined })),
            }),
        });
    } else if payload.analysis.is_none() {
        payload.frame = Some(frame.clone());
    }
    if let Some(frame) = &mut payload.frame {
        frame.signature = Sha256::digest(frame.encode_to_vec()).iter().map(|byte| format!("{byte:02x}")).collect();
    }
    if !request.marker_record_types.is_empty() {
        let mut markers = std::collections::BTreeMap::<u64, (v1::PlotMarker, u64)>::new();
        let mut cursor = after;
        loop {
            let page = store.evidence_page(execution, cursor, through, NonZeroUsize::new(maximum).unwrap())?;
            if page.is_empty() {
                break;
            }
            for evidence in page {
                cursor = evidence.position;
                let Some(kind) = evidence.kind.strip_prefix("record:") else {
                    continue;
                };
                if !request.marker_record_types.iter().any(|pattern| super::evidence_selection::matches(pattern, kind))
                    || evidence.source_ns < start_ns
                    || evidence.source_ns > end_ns
                {
                    continue;
                }
                let bucket = ((evidence.source_ns as i128 - start_ns as i128).max(0) as u128 / resolution.max(1) as u128) as u64;
                let marker = v1::PlotMarker {
                    x: Some(v1::Value {
                        kind: Some(v1::value::Kind::Number(evidence.source_ns as f64 / 1e9)),
                    }),
                    label: kind.into(),
                    record_type: kind.into(),
                    color: String::new(),
                };
                let entry = markers.entry(bucket).or_insert((marker, 0));
                entry.1 += 1;
            }
        }
        payload.markers = markers
            .into_values()
            .take(display)
            .map(|(mut marker, count)| {
                if count > 1 {
                    marker.label = format!("{} ({count})", marker.label);
                }
                marker
            })
            .collect();
    }
    let point_count = if let Some(analysis) =
        payload.analysis.as_ref().and_then(|analysis| analysis.result.as_ref()).and_then(|result| result.kind.as_ref())
    {
        match analysis {
            v1::plot_analysis_result::Kind::Histogram(histogram) => histogram.series.iter().map(|series| series.bins.len() as u64).sum(),
            v1::plot_analysis_result::Kind::BoxPlot(boxes) => boxes.series.iter().map(|series| 1 + series.outliers.len() as u64).sum(),
            v1::plot_analysis_result::Kind::Contour(grid) => grid.values.len() as u64,
            v1::plot_analysis_result::Kind::GridChart(grid) => grid.frames.iter().map(|frame| frame.values.len() as u64).sum(),
            v1::plot_analysis_result::Kind::SigmaInterval(sigma) => sigma.series.iter().map(|series| series.bins.len() as u64).sum(),
            _ => 0,
        }
    } else {
        frame.point_count
    };
    let temporal = matches!(kind, v1::PlotKind::Unspecified | v1::PlotKind::TimeSeries | v1::PlotKind::Xy | v1::PlotKind::Trajectory);
    if !temporal {
        payload.bucket_width_s = None;
        payload.bucket_window = None;
    }
    let label = |key: &str| {
        let (field, suffix) = key.split_once('/').map_or((key, None), |(field, group)| (field, Some(group)));
        let label = selected.labels.get(field).map_or(field, |label| label.as_str());
        suffix.map_or_else(|| label.to_owned(), |group| format!("{label}/{group}"))
    };
    for series in &mut payload.series {
        series.key = label(&series.key);
        series.label = label(&series.label);
    }
    for summary in &mut payload.summaries {
        summary.series_id = label(&summary.series_id);
    }
    if let Some(analysis) = payload.analysis.as_mut().and_then(|analysis| analysis.result.as_mut()).and_then(|result| result.kind.as_mut())
    {
        match analysis {
            v1::plot_analysis_result::Kind::Histogram(histogram) => {
                for series in &mut histogram.series {
                    series.key = label(&series.key);
                }
            }
            v1::plot_analysis_result::Kind::BoxPlot(boxes) => {
                for series in &mut boxes.series {
                    series.key = label(&series.key);
                    series.label = label(&series.label);
                }
            }
            v1::plot_analysis_result::Kind::SigmaInterval(interval) => {
                for series in &mut interval.series {
                    series.key = label(&series.key);
                }
            }
            _ => (),
        }
    }
    if selected.projected.contains(&payload.x_key) {
        payload.x_key = original
            .x_expression
            .as_ref()
            .filter(|expression| !expression.label.is_empty())
            .map_or("X".into(), |expression| expression.label.clone());
    }
    if payload.series_by.as_ref().is_some_and(|group| selected.projected.contains(group)) {
        payload.series_by = Some(
            original
                .series_by_expression
                .as_ref()
                .filter(|expression| !expression.label.is_empty())
                .map_or("Group".into(), |expression| expression.label.clone()),
        );
    }
    Ok(v1::PlotEnvelope {
        plot_id: request.plot_id.clone().unwrap_or_else(|| stream.id.clone()),
        revision: cut.frontier,
        epoch: epoch.into(),
        selection: selection_key,
        diagnostics: Some(v1::PlotPayloadDiagnostics {
            series_count: payload.series.len() as u64,
            point_count,
            raw_point_count: examined,
            downsampled: examined > point_count,
            max_points: display as u64,
            window_s: request.window_s,
            examined_rows: examined,
            resolution_nanos: if temporal { resolution } else { 0 },
        }),
        payload: Some(payload),
        warnings: Vec::new(),
    })
}

/* === Private Functions === */

fn numeric(kind: &prost_reflect::Kind) -> bool {
    matches!(
        kind,
        prost_reflect::Kind::Double
            | prost_reflect::Kind::Float
            | prost_reflect::Kind::Int32
            | prost_reflect::Kind::Sint32
            | prost_reflect::Kind::Sfixed32
            | prost_reflect::Kind::Int64
            | prost_reflect::Kind::Sint64
            | prost_reflect::Kind::Sfixed64
            | prost_reflect::Kind::Uint32
            | prost_reflect::Kind::Fixed32
            | prost_reflect::Kind::Uint64
            | prost_reflect::Kind::Fixed64
            | prost_reflect::Kind::Bool
    )
}
fn ns(seconds: f64) -> Result<i64> {
    let nanos = seconds * 1e9;
    if !nanos.is_finite() || nanos < i64::MIN as f64 || nanos >= i64::MAX as f64 {
        return Err(Error::Invalid("Query Plot: Choose Finite Source-Time Bounds Within Range".into()));
    }
    Ok(nanos.round() as i64)
}

/* === Tests === */
