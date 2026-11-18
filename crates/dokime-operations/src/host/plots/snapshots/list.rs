/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plots Snapshots List

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroU128;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

// Third Party Imports
use harmos_protocol::model::Declarations;
use harmos_protocol::model::History;
use harmos_runtime::storage::query::scalar::{ExactNumber, ScalarValue};
use prost::Message;
use prost_reflect::DescriptorPool;
use prost_reflect::Kind;
use sha2::{Digest, Sha256};
use tokio::task::JoinSet;
use tracing::dispatcher;

// Local Imports
use crate::{Error, Result, host::Context};
use dokime_protocol::host::plots::PlotAnalysis;
use dokime_protocol::host::plots::PlotAnalysisResult;
use dokime_protocol::host::plots::PlotBoxSummaryAnalysis;
use dokime_protocol::host::plots::PlotCartesianMode;
use dokime_protocol::host::plots::PlotColumn;
use dokime_protocol::host::plots::PlotEnvelope;
use dokime_protocol::host::plots::PlotFrame;
use dokime_protocol::host::plots::PlotHistogramAnalysis;
use dokime_protocol::host::plots::PlotMarker;
use dokime_protocol::host::plots::PlotPayload;
use dokime_protocol::host::plots::PlotPayloadDiagnostics;
use dokime_protocol::host::plots::PlotRequest;
use dokime_protocol::host::plots::PlotSeries;
use dokime_protocol::host::plots::PlotSeriesSummary;
use dokime_protocol::host::plots::PlotWindow;
use dokime_protocol::host::plots::SnapshotsListRequest;
use dokime_protocol::host::plots::SnapshotsListResponse;
use dokime_protocol::host::plots::plot_analysis_result;
use dokime_protocol::model::Aggregate;
use dokime_protocol::model::AxisScale;
use dokime_protocol::model::DataExpression;
use dokime_protocol::model::Downsample;
use dokime_protocol::model::PlotIntervalOptions;
use dokime_protocol::model::PlotKind;
use dokime_protocol::model::Reduction;
use dokime_protocol::model::Value;
use dokime_protocol::model::XyInterval;
use dokime_protocol::model::data_expression;
use dokime_protocol::model::value;
use dokime_storage::persisted::Scope;
use dokime_storage::{ExecutionCut, NumericWindow, Store};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.plots.Snapshots/List";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct PlotSource {
    cut: ExecutionCut,
    catalog: Declarations,
    pool: DescriptorPool,
}

struct PlotRead<'a> {
    store: &'a Store,
    cut: &'a ExecutionCut,
    maximum: usize,
    now_ns: i64,
    epoch: &'a str,
    catalog: &'a Declarations,
    pool: &'a DescriptorPool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

#[tracing::instrument(name = "plot.operation", skip_all)]
pub async fn handle(context: &Context, request: SnapshotsListRequest) -> Result<SnapshotsListResponse> {
    crate::host::complete(ID, context.settings.read_timeout, async {
        crate::host::views::continuation(request.page.as_ref(), request.at.as_ref())?;
        let key = format!(
            "plots/{}",
            crate::host::views::hex(&crate::host::paging::bytes(&SnapshotsListRequest {
                requests: request.requests.clone(),
                ..Default::default()
            }))
        );
        let executions = request
            .requests
            .iter()
            .map(|request| crate::host::projection::execution_scope(request.scope.as_ref()))
            .collect::<Result<BTreeSet<_>>>()?;
        let scope = if executions.len() == 1 {
            Scope::Execution(executions.first().unwrap()).identifier()?
        } else {
            Scope::Root(&context.host.identity().backend_id).identifier()?
        };
        let (saved, cut) = crate::host::views::read(context, scope, &key, request.at.as_ref(), async {
            let reply = SnapshotsListResponse {
                envelopes: query_batch(context, request.requests).await?,
                ..Default::default()
            };
            Ok(reply)
        })
        .await?;
        let (envelopes, next_page_token) = crate::host::views::page(
            saved.envelopes,
            &cut,
            request.page.as_ref(),
            context.settings.page_items,
            context.settings.message_bytes,
        )?;
        crate::host::bounded_reply(
            context,
            SnapshotsListResponse {
                envelopes,
                resolved_at: Some(cut),
                next_page_token,
            },
        )
    })
    .await
}

/// Capture Complete Plot Responses at a Fixed Cut Without Live Reply Coarsening.
pub async fn snapshots_at(
    store: &Store, cut: &ExecutionCut, requests: Vec<PlotRequest>, maximum: usize, now_ns: i64, epoch: &str,
) -> Result<Vec<PlotEnvelope>> {
    batch(store, requests, Some(cut), maximum, now_ns, epoch, None).await
}

/* === Private Functions === */

async fn query_batch(context: &Context, requests: Vec<PlotRequest>) -> Result<Vec<PlotEnvelope>> {
    let (store, _cancellation) = context.host.store().read_session()?;
    batch(
        &store,
        requests,
        None,
        context.settings.page_items,
        context.host.now_ns(),
        &context.host.identity().epoch,
        Some(context.settings.message_bytes),
    )
    .await
}

async fn batch(
    store: &Store, requests: Vec<PlotRequest>, fixed: Option<&ExecutionCut>, maximum: usize, now_ns: i64, epoch: &str,
    message_bytes: Option<usize>,
) -> Result<Vec<PlotEnvelope>> {
    let mut sources = BTreeMap::new();
    let mut selections = BTreeMap::new();
    let mut work = Vec::new();
    let mut placements = Vec::new();
    for request in &requests {
        let execution = crate::host::projection::execution_scope(request.scope.as_ref())?;
        if !sources.contains_key(execution) {
            let cut = match fixed {
                Some(cut) if cut.execution.identifier == execution => cut.clone(),
                Some(_) => return Err(Error::Invalid("Capture Plots: Keep Every Plot Within Its Execution Cut".into())),
                None => store.metadata(execution)?,
            };
            let catalog = Declarations::decode(cut.execution.catalog.as_slice())?;
            let pool = {
                let _span = tracing::debug_span!("plot.schema").entered();
                DescriptorPool::from_file_descriptor_set(
                    catalog.types.clone().ok_or_else(|| Error::Invalid("Query Plot: Restore the Frozen Stream Descriptors".into()))?,
                )
                .map_err(|source| Error::Invalid(source.to_string()))?
            };
            sources.insert(execution.to_owned(), Arc::new(PlotSource { cut, catalog, pool }));
        }
        let source = sources[execution].clone();
        if let Some(at) = &request.at {
            let history = History {
                store: store.identity().into(),
                epoch: execution.into(),
            };
            if at.positions.len() != 1
                || !at.revisions.is_empty()
                || at.positions[0].history.as_ref() != Some(&history)
                || at.positions[0].sequence > source.cut.frontier
            {
                return Err(Error::Invalid("Query Plot: Refresh the Document's Evidence Cut for This Backend".into()));
            }
        }
        let mut shape = request.clone();
        shape.plot_id = None;
        let key = shape.encode_to_vec();
        let index = *selections.entry(key).or_insert_with(|| {
            let index = work.len();
            work.push((request.clone(), source));
            index
        });
        placements.push(index);
    }
    let capacity = thread::available_parallelism().map_or(1, |threads| threads.get()).min(work.len());
    let capturing = fixed.is_some();
    let mut output = vec![None; work.len()];
    let mut pending = work.into_iter().enumerate();
    let mut workers = JoinSet::new();
    loop {
        while workers.len() < capacity {
            let Some((index, (mut request, source))) = pending.next() else { break };
            let reader = store.independent_projection();
            let epoch = epoch.to_owned();
            let span = tracing::Span::current();
            let dispatch = dispatcher::get_default(Clone::clone);
            workers.spawn_blocking(move || -> Result<_> {
                let _dispatch = dispatcher::set_default(&dispatch);
                let _entered = span.enter();
                let started = Instant::now();
                let query =
                    tracing::debug_span!("plot.query", kind = request.plot_type, plot = ?request.plot_id, samples = tracing::field::Empty);
                let _query = query.enter();
                let mut cut = source.cut.clone();
                if let Some(at) = &request.at {
                    cut.frontier = at.positions[0].sequence;
                }
                loop {
                    let envelope = snapshot_prepared(
                        PlotRead {
                            store: &reader,
                            cut: &cut,
                            maximum,
                            now_ns,
                            epoch: &epoch,
                            catalog: &source.catalog,
                            pool: &source.pool,
                        },
                        &request,
                    )?;
                    if message_bytes.is_none_or(|maximum| envelope.encoded_len().saturating_add(32) <= maximum) {
                        query.record("samples", envelope.diagnostics.as_ref().map_or(0, |value| value.raw_point_count));
                        if capturing {
                            tracing::debug!(
                                elapsed_ms = started.elapsed().as_secs_f64() * 1000.0,
                                samples = envelope.diagnostics.as_ref().map_or(0, |value| value.raw_point_count),
                                stream = request.stream,
                                kind = request.plot_type,
                                plot = ?request.plot_id,
                                "Report Plot"
                            );
                        }
                        return Ok((index, envelope));
                    }
                    let points = request.max_points.unwrap_or(maximum as u64).min(maximum as u64);
                    if points <= 4 || request.technique == Downsample::Raw as i32 {
                        return Err(Error::Invalid(
                            "Query Plot: Narrow the Raw Window or Split the Batch to Fit Its Irreducible Message Budget".into(),
                        ));
                    }
                    request.max_points = Some((points / 2).max(4));
                }
            });
        }
        let Some(result) = workers.join_next().await else { break };
        let (index, envelope) =
            result.map_err(|source| Error::Invalid(format!("Query Plot: Restart After the Query Worker Failed: {source}")))??;
        output[index] = Some(envelope);
    }
    requests
        .into_iter()
        .zip(placements)
        .map(|(request, index)| {
            let mut envelope = output[index].as_ref().unwrap().clone();
            envelope.plot_id = request.plot_id.clone().unwrap_or_else(|| request.stream.clone());
            envelope.selection = Sha256::digest(request.encode_to_vec()).iter().map(|byte| format!("{byte:02x}")).collect();
            Ok(envelope)
        })
        .collect()
}

fn snapshot_prepared(read: PlotRead<'_>, request: &PlotRequest) -> Result<PlotEnvelope> {
    let PlotRead {
        store,
        cut,
        maximum,
        now_ns,
        epoch,
        catalog,
        pool,
    } = read;
    let execution = cut.execution.identifier.as_str();
    let original = request;
    let selection_key = Sha256::digest(request.encode_to_vec()).iter().map(|byte| format!("{byte:02x}")).collect();
    let selected = store.plot_selection(cut, request)?;
    let request = &selected.request;
    let stream = catalog
        .streams
        .iter()
        .find(|stream| stream.id == request.stream)
        .ok_or_else(|| Error::Invalid(format!("Query Plot: Select a Declared Stream {}", request.stream)))?;
    let descriptor =
        pool.get_message_by_name(&stream.row).ok_or_else(|| Error::Invalid("Query Plot: Restore the Frozen Row Descriptor".into()))?;
    let mut fields = if request.y_expressions.is_empty() { request.y_keys.clone() } else { Vec::new() };
    let mut aggregate = None;
    for expression in &request.y_expressions {
        match expression.kind.as_ref() {
            Some(data_expression::Kind::Stream(reference)) if reference.identifier == stream.id => fields.push(reference.field.clone()),
            Some(data_expression::Kind::Aggregate(expression)) => {
                let Some(data_expression::Kind::Stream(reference)) = expression.input.as_ref().and_then(|input| input.kind.as_ref()) else {
                    return Err(Error::Invalid("Query Plot: Aggregate a Numeric Stream Column".into()));
                };
                if reference.identifier != stream.id {
                    return Err(Error::Invalid("Query Plot: Select Columns from One Stream".into()));
                }
                fields.push(reference.field.clone());
                let reducer = Aggregate::try_from(expression.kind)
                    .map_err(|_| Error::Invalid("Query Plot Aggregate: Select a Declared Reducer".into()))?;
                if aggregate.is_some_and(|previous| previous != reducer) {
                    return Err(Error::Invalid("Query Plot Aggregate: Use One Reducer per Plot or Split the Series".into()));
                }
                aggregate = Some(reducer);
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

    let technique = Downsample::try_from(request.technique)
        .map_err(|_| Error::Invalid("Query Plot: Select a Declared Downsampling Technique".into()))?;
    if request.max_points == Some(0) {
        return Err(Error::Invalid("Query Plot: Select a Positive Display Resolution".into()));
    }
    if !request.filters.is_empty() {
        return Err(Error::Invalid("Query Plot: Select an Unfiltered Stream Window".into()));
    }
    if request.z_stream.as_ref().is_some_and(|selected| selected != &stream.id) {
        return Err(Error::Invalid("Query Plot: Select Coordinate Columns from One Stream".into()));
    }
    if aggregate.is_some_and(|kind| !matches!(kind, Aggregate::Count | Aggregate::Sum | Aggregate::Min | Aggregate::Max | Aggregate::Mean))
    {
        return Err(Error::Invalid("Query Category Plot: Select Count, Sum, Min, Max or Mean".into()));
    }
    if request.panel_width == Some(0) || request.panel_height == Some(0) {
        return Err(Error::Invalid("Query Plot: Supply Positive Panel Dimensions".into()));
    }
    let kind = PlotKind::try_from(request.plot_type).map_err(|_| Error::Invalid("Query Plot: Select a Declared Plot Kind".into()))?;
    if aggregate.is_some()
        && (!matches!(kind, PlotKind::Bar | PlotKind::Pie)
            || request.y_expressions.iter().any(|expression| !matches!(expression.kind, Some(data_expression::Kind::Aggregate(_)))))
    {
        return Err(Error::Invalid(
            "Query Plot Aggregate: Select Only Aggregate Expressions in a Bar or Pie Plot Until Other Aggregate Variants Are Served".into(),
        ));
    }
    let field_reference = |expression: Option<&DataExpression>, fallback: Option<&str>| -> Result<Option<String>> {
        match expression.and_then(|expression| expression.kind.as_ref()) {
            Some(data_expression::Kind::Stream(reference)) if reference.identifier == stream.id => Ok(Some(reference.field.clone())),
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
    if request.pareto && (kind != PlotKind::Bar || request.arrangement.is_some() || group.is_some()) {
        return Err(Error::Invalid("Query Pareto: Select One Ungrouped Bar Series".into()));
    }
    if request.arrangement.is_some() && (kind != PlotKind::Bar || group.is_none()) {
        return Err(Error::Invalid("Query Grouped Bars: Select a Series Column".into()));
    }
    if request.xy_interval.is_some() && (kind != PlotKind::Xy || request.xy_interval != Some(XyInterval::Sigma as i32)) {
        return Err(Error::Invalid("Query XY Intervals: Select Sigma on an XY Plot".into()));
    }
    if fields.len() != 1 && (matches!(kind, PlotKind::Pie | PlotKind::Contour | PlotKind::GridChart) || request.pareto) {
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
    let (start_ns, end_ns) =
        if let Some(viewport) = request.viewport.as_ref().filter(|_| matches!(kind, PlotKind::TimeSeries | PlotKind::Unspecified)) {
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
    let mut payload = PlotPayload {
        stream: stream.id.clone(),
        x_key: x.clone().unwrap_or_else(|| "timestamp".into()),
        series_by: group.clone(),
        max_points: display as u64,
        window_s: request.window_s,
        generated_at: crate::host::projection::timestamp(now_ns).to_string(),
        bucket_width_s: Some(resolution as f64 / 1e9),
        bucket_window: Some(PlotWindow {
            from: start_ns as f64 / 1e9,
            to: end_ns as f64 / 1e9,
        }),
        ..Default::default()
    };
    let mut frame = PlotFrame {
        mode: Some(PlotCartesianMode::Faceted as i32),
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
        let count = if let Some(group_field) = group
            .as_deref()
            .filter(|_| matches!(kind, PlotKind::TimeSeries | PlotKind::Unspecified | PlotKind::Histogram | PlotKind::BoxPlot))
        {
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
                let count = if kind == PlotKind::Histogram {
                    let mut histogram = store.histogram_window(
                        group_window,
                        request.histogram.as_ref().map_or(group_display, |options| options.bins as usize).clamp(1, group_display),
                    )?;
                    histogram.key = format!("{field}/{label}");
                    let count = histogram.count;
                    histograms.push(histogram);
                    count
                } else if kind == PlotKind::BoxPlot {
                    let mut summary = store.box_window(group_window, group_display.saturating_sub(1))?;
                    summary.key = format!("{field}/{label}");
                    summary.label = label.clone();
                    let count = summary.count;
                    boxes.push(summary);
                    count
                } else {
                    let (column, mut summary, count) = store.sampled_time_series_window(group_window, technique, group_display)?;
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
        } else if kind == PlotKind::Histogram {
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
        } else if kind == PlotKind::BoxPlot {
            let mut summary = store.box_window(window, display.saturating_sub(1))?;
            summary.key = field.clone();
            summary.label = field.clone();
            let count = summary.count;
            boxes.push(summary);
            count
        } else if matches!(kind, PlotKind::Xy | PlotKind::Trajectory | PlotKind::Bar | PlotKind::Pie | PlotKind::Contour) {
            let x = x.as_deref().ok_or_else(|| Error::Invalid("Query Plot: Select an X Coordinate or Category Column".into()))?;
            let settings = dokime_storage::KindWindow {
                window,
                x: Some(x),
                z: z.as_deref(),
                group: group.as_deref(),
                limit: if kind == PlotKind::Contour {
                    display.min(bins.get().saturating_mul(request.panel_height.unwrap_or(display as u32) as usize))
                } else {
                    display
                },
            };
            if request.xy_interval.is_some() {
                let options = request.interval.unwrap_or(PlotIntervalOptions {
                    bin_width: 1.0,
                    sigma: 1.0,
                    clip_to_observed: false,
                });
                let (analysis, count) = store.sigma_window(settings, options)?;
                payload.analysis = Some(PlotAnalysis {
                    result: Some(PlotAnalysisResult {
                        kind: Some(plot_analysis_result::Kind::SigmaInterval(analysis)),
                    }),
                });
                count
            } else if kind == PlotKind::Contour {
                if z.is_none() {
                    return Err(Error::Invalid("Query Contour: Select a Numeric Z Column".into()));
                }
                let (grid, count) = store.contour_scaled_window(
                    settings,
                    request.viewport.as_ref(),
                    request.y_viewport.as_ref(),
                    request.reduction.and_then(|value| Reduction::try_from(value).ok()).unwrap_or(Reduction::Mean),
                    request.x_scale == AxisScale::Log as i32,
                    request.y_scale == AxisScale::Log as i32,
                )?;
                payload.analysis = Some(PlotAnalysis {
                    result: Some(PlotAnalysisResult {
                        kind: Some(plot_analysis_result::Kind::Contour(grid)),
                    }),
                });
                count
            } else {
                let (mut series, count, groups) = if matches!(kind, PlotKind::Bar | PlotKind::Pie) {
                    store.category_window(settings, aggregate, kind == PlotKind::Pie || request.pareto)?
                } else {
                    store.cartesian_window(settings)?
                };
                if request.pareto {
                    for column in &mut series.series {
                        order_categories(column)?;
                    }
                }
                column_groups = Some(groups);
                frame.point_count += series.point_count;
                frame.series.extend(series.series);
                count
            }
        } else if kind == PlotKind::GridChart {
            let grid =
                request.grid.as_ref().ok_or_else(|| Error::Invalid("Query Grid Chart: Supply Row, Column and Frame Bindings".into()))?;
            let row = field_reference(grid.row.as_ref(), None)?;
            let column = field_reference(grid.column.as_ref(), None)?;
            let instant = field_reference(grid.frame.as_ref(), None)?;
            let reduction = request
                .reduction
                .and_then(|value| Reduction::try_from(value).ok())
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
            payload.analysis = Some(PlotAnalysis {
                result: Some(PlotAnalysisResult {
                    kind: Some(plot_analysis_result::Kind::GridChart(frames)),
                }),
            });
            count
        } else if kind == PlotKind::FrequencySeries {
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
            let (column, mut summary, count) = store.sampled_time_series_window(window, technique, display)?;
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
            payload.summaries.push(PlotSeriesSummary {
                series_id: field.clone(),
                count: stats.count,
                min: stats.min,
                max: stats.max,
                mean: stats.mean,
                median: stats.median,
                std: (stats.count > 0).then_some(stats.stddev),
                rms: Some(store.numeric_rms(&summary_window)?),
                window: Some(*request.summary_window.as_ref().unwrap()),
            });
        }
        examined += count;
        if let Some(groups) = column_groups {
            for (group, raw_points) in groups {
                payload.series.push(PlotSeries {
                    key: if group.is_empty() { field.clone() } else { format!("{field}/{group}") },
                    label: if group.is_empty() { field.clone() } else { group },
                    unit: request.unit.clone(),
                    raw_points,
                    downsampled: raw_points > display as u64,
                    ..Default::default()
                });
            }
        } else {
            payload.series.push(PlotSeries {
                key: field.clone(),
                label: field,
                unit: request.unit.clone(),
                raw_points: count,
                downsampled: count > display as u64,
                ..Default::default()
            });
        }
    }
    if kind == PlotKind::Histogram {
        payload.analysis = Some(PlotAnalysis {
            result: Some(PlotAnalysisResult {
                kind: Some(plot_analysis_result::Kind::Histogram(PlotHistogramAnalysis { series: histograms })),
            }),
        });
    } else if kind == PlotKind::BoxPlot {
        payload.analysis = Some(PlotAnalysis {
            result: Some(PlotAnalysisResult {
                kind: Some(plot_analysis_result::Kind::BoxPlot(PlotBoxSummaryAnalysis { series: boxes, total: examined })),
            }),
        });
    } else if payload.analysis.is_none() {
        if matches!(kind, PlotKind::TimeSeries | PlotKind::Unspecified)
            && let Some(first) = frame.series.first()
            && frame.series.iter().all(|column| column.x == first.x)
        {
            frame.time = first
                .x
                .iter()
                .map(|value| match value.kind {
                    Some(value::Kind::Number(value)) => Ok(value),
                    _ => Err(Error::Invalid("Read Time Series: Restore Its Numeric Source-Time Axis".into())),
                })
                .collect::<Result<Vec<_>>>()?;
            frame.mode = Some(PlotCartesianMode::Aligned as i32);
            for column in &mut frame.series {
                column.x.clear();
            }
        }
        payload.frame = Some(frame.clone());
    }
    if let Some(frame) = &mut payload.frame {
        frame.signature = Sha256::digest(frame.encode_to_vec()).iter().map(|byte| format!("{byte:02x}")).collect();
    }
    if !request.marker_record_types.is_empty() {
        let mut markers = BTreeMap::<u64, (PlotMarker, u64)>::new();
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
                if !request.marker_record_types.iter().any(|pattern| crate::host::evidence::selection::matches(pattern, kind))
                    || evidence.source_ns < start_ns
                    || evidence.source_ns > end_ns
                {
                    continue;
                }
                let bucket = ((evidence.source_ns as i128 - start_ns as i128).max(0) as u128 / resolution.max(1) as u128) as u64;
                let marker = PlotMarker {
                    x: Some(Value {
                        kind: Some(value::Kind::Number(evidence.source_ns as f64 / 1e9)),
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
            plot_analysis_result::Kind::Histogram(histogram) => histogram.series.iter().map(|series| series.bins.len() as u64).sum(),
            plot_analysis_result::Kind::BoxPlot(boxes) => boxes.series.iter().map(|series| 1 + series.outliers.len() as u64).sum(),
            plot_analysis_result::Kind::Contour(grid) => grid.values.len() as u64,
            plot_analysis_result::Kind::GridChart(grid) => grid.frames.iter().map(|frame| frame.values.len() as u64).sum(),
            plot_analysis_result::Kind::SigmaInterval(sigma) => sigma.series.iter().map(|series| series.bins.len() as u64).sum(),
            _ => 0,
        }
    } else {
        frame.point_count
    };
    let temporal = matches!(kind, PlotKind::Unspecified | PlotKind::TimeSeries | PlotKind::Xy | PlotKind::Trajectory);
    if !temporal || matches!(technique, Downsample::Raw | Downsample::Lttb) {
        payload.bucket_width_s = None;
    }
    if !temporal {
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
            plot_analysis_result::Kind::Histogram(histogram) => {
                for series in &mut histogram.series {
                    series.key = label(&series.key);
                }
            }
            plot_analysis_result::Kind::BoxPlot(boxes) => {
                for series in &mut boxes.series {
                    series.key = label(&series.key);
                    series.label = label(&series.label);
                }
            }
            plot_analysis_result::Kind::SigmaInterval(interval) => {
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
    Ok(PlotEnvelope {
        plot_id: request.plot_id.clone().unwrap_or_else(|| stream.id.clone()),
        revision: cut.frontier,
        epoch: epoch.into(),
        selection: selection_key,
        diagnostics: Some(PlotPayloadDiagnostics {
            series_count: payload.series.len() as u64,
            point_count,
            raw_point_count: examined,
            downsampled: examined > point_count,
            max_points: display as u64,
            window_s: request.window_s,
            examined_rows: examined,
            resolution_nanos: if temporal && !matches!(technique, Downsample::Raw | Downsample::Lttb) { resolution } else { 0 },
        }),
        payload: Some(payload),
        warnings: Vec::new(),
    })
}

/* === Private Functions === */

fn order_categories(column: &mut PlotColumn) -> Result<()> {
    if column.totals.len() != column.y.len() || column.x.len() != column.y.len() {
        return Err(Error::Invalid("Order Pareto Categories: Retain One Exact Total per Category".into()));
    }
    let totals = column
        .totals
        .iter()
        .map(|value| {
            let total = match value.kind.as_ref() {
                Some(value::Kind::Integer(value)) => ScalarValue::Signed(*value),
                Some(value::Kind::UnsignedInteger(value)) => ScalarValue::Unsigned(*value),
                Some(value::Kind::Number(value)) => ScalarValue::Float(*value),
                Some(value::Kind::Exact(value)) => {
                    let numerator = value
                        .numerator
                        .parse()
                        .map_err(|source| Error::Invalid(format!("Order Pareto Categories: Read the Exact Numerator: {source}")))?;
                    let denominator = value
                        .denominator
                        .parse::<u128>()
                        .ok()
                        .and_then(NonZeroU128::new)
                        .ok_or_else(|| Error::Invalid("Order Pareto Categories: Supply a Positive Exact Denominator".into()))?;
                    ScalarValue::Exact(ExactNumber::new(value.negative, numerator, denominator))
                }
                _ => return Err(Error::Invalid("Order Pareto Categories: Retain Numeric Totals".into())),
            };
            total.check().map_err(|source| Error::Invalid(format!("Order Pareto Categories: {source}")))?;
            Ok(total)
        })
        .collect::<Result<Vec<_>>>()?;
    if let Some(first) = totals.first() {
        for total in &totals {
            first.compare(total).map_err(|source| Error::Invalid(format!("Order Pareto Categories: {source}")))?;
        }
    }
    let mut indices = (0..totals.len()).collect::<Vec<_>>();
    indices.sort_by(|left, right| totals[*right].compare(&totals[*left]).unwrap());
    column.x = indices.iter().map(|index| column.x[*index].clone()).collect();
    column.y = indices.iter().map(|index| column.y[*index]).collect();
    column.totals = indices.iter().map(|index| column.totals[*index].clone()).collect();
    Ok(())
}

fn numeric(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::Double
            | Kind::Float
            | Kind::Int32
            | Kind::Sint32
            | Kind::Sfixed32
            | Kind::Int64
            | Kind::Sint64
            | Kind::Sfixed64
            | Kind::Uint32
            | Kind::Fixed32
            | Kind::Uint64
            | Kind::Fixed64
            | Kind::Bool
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

#[cfg(test)]
mod tests {
    use std::thread;

    use harmos_protocol::model::{Declarations, StreamDeclaration};
    use prost::Message;
    use prost_types::{DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet, field_descriptor_proto::Type};

    use super::{PlotColumn, Value, order_categories, snapshots_at, value};
    use dokime_protocol::host::plots::NullableNumber;
    use dokime_protocol::host::plots::PlotRequest;
    use dokime_protocol::model::{Downsample, EvidenceScope, Import, Origin, PlotKind, origin};
    use dokime_storage::{Execution, Sample, Store};

    #[tokio::test]
    async fn batch_larger_than_worker_bound_preserves_every_envelope_in_request_order() {
        let root = tempfile::tempdir().unwrap();
        let store = Store::open(root.path()).unwrap();
        let catalog = Declarations {
            types: Some(FileDescriptorSet {
                file: vec![FileDescriptorProto {
                    name: Some("plots.proto".into()),
                    package: Some("plots".into()),
                    syntax: Some("proto3".into()),
                    message_type: vec![DescriptorProto {
                        name: Some("Row".into()),
                        field: vec![FieldDescriptorProto {
                            name: Some("value".into()),
                            number: Some(1),
                            r#type: Some(Type::Double as i32),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
            }),
            streams: vec![StreamDeclaration {
                id: "readings".into(),
                row: "plots.Row".into(),
                ..Default::default()
            }],
            ..Default::default()
        };
        store
            .create(&Execution {
                identifier: "execution".into(),
                cycle: "cycle".into(),
                origin: Some(Origin {
                    kind: Some(origin::Kind::Import(Import::default())),
                }),
                catalog: catalog.encode_to_vec(),
                ..Default::default()
            })
            .unwrap();
        store.open_stream("execution", "readings").unwrap();
        let samples = (0..6)
            .map(|index| Sample {
                stream: "readings".into(),
                source_ns: index,
                sequence: index as u64,
                ..Default::default()
            })
            .collect::<Vec<_>>();
        store.sample_batch("execution", &samples).unwrap();
        store.flush_samples().unwrap();
        let cut = store.cut("execution").unwrap();
        let (reader, _cancellation) = store.read_session().unwrap();
        let count = thread::available_parallelism().map_or(1, |threads| threads.get()) + 2;
        let requests = (0..count)
            .rev()
            .map(|index| PlotRequest {
                plot_id: Some(format!("plot-{index}")),
                scope: Some(EvidenceScope {
                    execution_id: Some("execution".into()),
                    ..Default::default()
                }),
                stream: "readings".into(),
                y_keys: vec!["value".into()],
                plot_type: PlotKind::TimeSeries as i32,
                technique: Downsample::Lttb as i32,
                max_points: Some((index + 4) as u64),
                ..Default::default()
            })
            .collect::<Vec<_>>();
        let mut expected = Vec::new();
        for request in &requests {
            expected.extend(snapshots_at(&reader, &cut, vec![request.clone()], count + 4, 0, "epoch").await.unwrap());
        }
        let actual = snapshots_at(&reader, &cut, requests.clone(), count + 4, 0, "epoch").await.unwrap();
        assert_eq!(actual, expected);
        assert!(actual.iter().all(|envelope| envelope.payload.as_ref().unwrap().series[0].raw_points == samples.len() as u64));
        assert_eq!(
            actual.iter().map(|envelope| &envelope.plot_id).collect::<Vec<_>>(),
            requests.iter().map(|request| request.plot_id.as_ref().unwrap()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn pareto_orders_exact_totals_above_float_precision_with_their_categories() {
        let mut column = PlotColumn {
            x: vec![
                Value {
                    kind: Some(value::Kind::Text("small".into())),
                },
                Value {
                    kind: Some(value::Kind::Text("large".into())),
                },
            ],
            y: vec![NullableNumber { value: Some(9_007_199_254_740_992.0) }; 2],
            totals: vec![
                Value {
                    kind: Some(value::Kind::UnsignedInteger(9_007_199_254_740_992)),
                },
                Value {
                    kind: Some(value::Kind::UnsignedInteger(9_007_199_254_740_993)),
                },
            ],
            ..Default::default()
        };
        let expected = column.x[1].clone();
        order_categories(&mut column).unwrap();
        assert_eq!(column.x[0], expected);
        assert_eq!(column.totals[0].kind, Some(value::Kind::UnsignedInteger(9_007_199_254_740_993)));
    }

    #[test]
    fn pareto_refuses_misaligned_or_nonfinite_totals_before_reordering() {
        let mut column = PlotColumn {
            x: vec![Value::default()],
            y: vec![NullableNumber { value: Some(1.0) }],
            ..Default::default()
        };
        let before = column.clone();
        assert!(order_categories(&mut column).is_err());
        assert_eq!(column, before);
        column.totals = vec![Value {
            kind: Some(value::Kind::Number(f64::INFINITY)),
        }];
        assert!(order_categories(&mut column).is_err());
        assert_eq!(column.x, before.x);
        assert_eq!(column.y, before.y);
    }
}

crate::native::unary!(SnapshotsListRequest, SnapshotsListResponse, context, "dokime.host.v1.plots", "Snapshots", "List");
