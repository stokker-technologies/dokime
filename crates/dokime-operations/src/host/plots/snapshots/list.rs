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
use tracing::Instrument;
use tracing::dispatcher;

// Local Imports
use crate::{
    Error, Result,
    host::{self, Context, evidence::selection, paging, projection, views},
};
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
use dokime_storage::{ExecutionCut, FrozenCatalog, KindWindow, NumericWindow, Store};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.plots.Snapshots/List";

// M4 Keeps Four Points per Pixel Column; a 4K-Wide Plot Needs 4 x 3840 to Draw Every Sample Exactly.
const DEFAULT_POINTS: u64 = 15_360;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct PlotSource {
    cut: ExecutionCut,
    catalog: Arc<FrozenCatalog>,
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

pub async fn handle(context: &Context, request: SnapshotsListRequest) -> Result<SnapshotsListResponse> {
    serve(context, request, true).await
}

/// Serve a Watch Page. Unlike a List's, a Reply That Fits One Page Retains No View: a Watch Continues Only a Reply
/// Spanning Pages, and Its Cut Asks a Later `at` Read to Resynchronize.
pub(super) async fn watched(context: &Context, request: SnapshotsListRequest) -> Result<SnapshotsListResponse> {
    serve(context, request, false).await
}

/// Serve Plot Responses at Their Cut, or at the Accepted Frontier While Their Execution Takes Data.
pub async fn snapshots(
    store: &Store, requests: Vec<PlotRequest>, maximum: usize, now_ns: i64, epoch: &str, message_bytes: Option<usize>,
) -> Result<Vec<PlotEnvelope>> {
    let live = requests.iter().any(|request| request.at.is_none());
    let (store, _cancellation) = {
        let _span = tracing::debug_span!("plot.session").entered();
        if live { store.live_read_session()? } else { store.read_session()? }
    };
    batch(&store, requests, None, maximum, now_ns, epoch, message_bytes).await
}

/// Capture Complete Plot Responses at a Fixed Cut Without Live Reply Coarsening.
pub async fn snapshots_at(
    store: &Store, cut: &ExecutionCut, requests: Vec<PlotRequest>, maximum: usize, now_ns: i64, epoch: &str,
) -> Result<Vec<PlotEnvelope>> {
    batch(store, requests, Some(cut), maximum, now_ns, epoch, None).await
}

/* === Private Functions === */

/// Serve One Page; a List Retains Every Current View So Its Cut Reads the Same Reply Again, a Watch Only a View It
/// Continues.
#[tracing::instrument(name = "plot.operation", skip_all)]
async fn serve(context: &Context, request: SnapshotsListRequest, listed: bool) -> Result<SnapshotsListResponse> {
    host::complete(ID, context.settings.read_timeout, async {
        views::continuation(request.page.as_ref(), request.at.as_ref())?;
        let key = format!(
            "plots/{}",
            views::hex(&paging::bytes(&SnapshotsListRequest {
                requests: request.requests.clone(),
                ..Default::default()
            }))
        );
        let executions =
            request.requests.iter().map(|request| projection::execution_scope(request.scope.as_ref())).collect::<Result<BTreeSet<_>>>()?;
        let scope = if executions.len() == 1 {
            Scope::Execution(executions.first().unwrap()).identifier()?
        } else {
            Scope::Root(&context.host.identity().backend_id).identifier()?
        };
        let (saved, cut, view) = views::resolve(context, scope, &key, request.at.as_ref(), async {
            let reply = SnapshotsListResponse {
                envelopes: snapshots(
                    &context.host.store(),
                    request.requests,
                    context.settings.page_items,
                    context.host.now_ns(),
                    &context.host.identity().epoch,
                    Some(context.settings.message_bytes),
                )
                .await?,
                ..Default::default()
            };
            Ok(reply)
        })
        .await?;
        let (envelopes, next_page_token) =
            views::page(saved.envelopes, &cut, request.page.as_ref(), context.settings.page_items, context.settings.message_bytes)?;
        // A Continuation, or an `at` Read of a List's Cut, Finds the Reply Only Through Its Retained View.
        let continued = !next_page_token.is_empty();
        if let Some(view) = view
            && (listed || continued)
        {
            view.retain(context).instrument(tracing::debug_span!("plot.retain")).await?;
        }
        host::bounded_reply(
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

async fn batch(
    store: &Store, requests: Vec<PlotRequest>, fixed: Option<&ExecutionCut>, maximum: usize, now_ns: i64, epoch: &str,
    message_bytes: Option<usize>,
) -> Result<Vec<PlotEnvelope>> {
    let mut sources = BTreeMap::new();
    let mut selections = BTreeMap::new();
    let mut work = Vec::new();
    let mut placements = Vec::new();
    for request in &requests {
        let execution = projection::execution_scope(request.scope.as_ref())?;
        if !sources.contains_key(execution) {
            let cut = match fixed {
                Some(cut) if cut.execution.identifier == execution => cut.clone(),
                Some(_) => return Err(Error::Invalid("Capture Plots: Keep Every Plot Within Its Execution Cut".into())),
                None => store.metadata(execution)?,
            };
            let catalog = {
                let _span = tracing::debug_span!("plot.schema").entered();
                store.frozen_catalog(&cut.execution.catalog)?
            };
            let pool = catalog
                .pool
                .clone()
                .ok_or_else(|| Error::Invalid("Query Plot: Restore the Frozen Stream Descriptors".into()))?
                .map_err(Error::Invalid)?;
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
            let Some((index, (mut request, source))) = pending.next() else {
                break;
            };
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
                } else if !capturing {
                    cut.frontier = reader.accepted_frontier(&cut.execution.identifier)?;
                }
                loop {
                    let envelope = snapshot_prepared(
                        PlotRead {
                            store: &reader,
                            cut: &cut,
                            maximum,
                            now_ns,
                            epoch: &epoch,
                            catalog: &source.catalog.declarations,
                            pool: &source.pool,
                        },
                        &request,
                    )?;
                    let size = envelope.encoded_len();
                    let reply_bytes = 1 + prost::length_delimiter_len(size) + size;
                    if message_bytes.is_none_or(|maximum| reply_bytes <= maximum) {
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
                    let points = request.max_points.unwrap_or(DEFAULT_POINTS);
                    if points <= 4 || request.technique == Downsample::Raw as i32 {
                        return Err(Error::Invalid(
                            "Query Plot: Narrow the Raw Window or Split the Batch to Fit Its Irreducible Message Budget".into(),
                        ));
                    }
                    request.max_points = Some((points / 2).max(4));
                }
            });
        }
        let Some(result) = workers.join_next().await else {
            break;
        };
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
    request.validate_heat_strip().map_err(Error::Invalid)?;
    let selection_key = Sha256::digest(request.encode_to_vec()).iter().map(|byte| format!("{byte:02x}")).collect();
    let selected = store.plot_selection(cut, request)?;
    let request = &selected.request;
    let stream = request.stream.as_str();
    let descriptor = if stream.is_empty() {
        None
    } else {
        let declaration = catalog
            .streams
            .iter()
            .find(|declaration| declaration.id == stream)
            .ok_or_else(|| Error::Invalid(format!("Query Plot: Select a Declared Stream {stream}")))?;
        Some(
            pool.get_message_by_name(&declaration.row)
                .ok_or_else(|| Error::Invalid("Query Plot: Restore the Frozen Row Descriptor".into()))?,
        )
    };
    let kind = PlotKind::try_from(request.plot_type).map_err(|_| Error::Invalid("Query Plot: Select a Declared Plot Kind".into()))?;
    let mut fields = if request.y_expressions.is_empty() && !stream.is_empty() { request.y_keys.clone() } else { Vec::new() };
    let mut aggregate = None;
    for expression in &request.y_expressions {
        match expression.kind.as_ref() {
            Some(data_expression::Kind::Stream(reference)) if reference.identifier == stream => fields.push(reference.field.clone()),
            Some(data_expression::Kind::Aggregate(expression)) => {
                let Some(data_expression::Kind::Stream(reference)) = expression.input.as_ref().and_then(|input| input.kind.as_ref()) else {
                    return Err(Error::Invalid("Query Plot: Aggregate a Numeric Stream Column".into()));
                };
                if reference.identifier != stream {
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
    if fields.is_empty()
        && kind == PlotKind::Histogram
        && let Some(data_expression::Kind::Stream(reference)) =
            request.x_expression.as_ref().and_then(|expression| expression.kind.as_ref())
        && reference.identifier == stream
    {
        fields.push(reference.field.clone());
    }
    if fields.is_empty()
        && let Some(descriptor) = &descriptor
    {
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
            && descriptor
                .as_ref()
                .and_then(|descriptor| descriptor.get_field_by_name(field))
                .is_none_or(|field| field.is_list() || field.is_map() || !numeric(&field.kind()))
        {
            return Err(Error::Invalid(format!("Query Plot: Select a Declared Numeric Scalar Field {stream}.{field}")));
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
    if request.z_stream.as_ref().is_some_and(|selected| selected != stream) {
        return Err(Error::Invalid("Query Plot: Select Coordinate Columns from One Stream".into()));
    }
    if aggregate.is_some_and(|kind| !matches!(kind, Aggregate::Count | Aggregate::Sum | Aggregate::Min | Aggregate::Max | Aggregate::Mean))
    {
        return Err(Error::Invalid("Query Category Plot: Select Count, Sum, Min, Max or Mean".into()));
    }
    if request.panel_width == Some(0) || request.panel_height == Some(0) {
        return Err(Error::Invalid("Query Plot: Supply Positive Panel Dimensions".into()));
    }
    let temporal = matches!(kind, PlotKind::Unspecified | PlotKind::TimeSeries | PlotKind::Xy | PlotKind::Trajectory);
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
            Some(data_expression::Kind::Stream(reference)) if reference.identifier == stream => Ok(Some(reference.field.clone())),
            None => Ok(fallback.filter(|_| !stream.is_empty()).map(str::to_owned)),
            _ => Err(Error::Invalid("Query Plot: Select Coordinate Columns from the Requested Stream".into())),
        }
    };
    let x = field_reference(request.x_expression.as_ref(), request.x_key.as_deref())?;
    let group = field_reference(request.series_by_expression.as_ref(), request.series_by.as_deref())?;
    let z = field_reference(request.z_expression.as_ref(), request.z_field.as_deref())?;
    let cell = field_reference(request.cell_expression.as_ref(), None)?;
    let facet = field_reference(request.facet_expression.as_ref(), None)?;
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
    if fields.len() != 1
        && (matches!(kind, PlotKind::Pie | PlotKind::Contour | PlotKind::GridChart | PlotKind::HeatStrip) || request.pareto)
    {
        return Err(Error::Invalid("Query Plot: Select Exactly One Value Column for This Kind".into()));
    }
    let requested = usize::try_from(request.max_points.unwrap_or(DEFAULT_POINTS))
        .map_err(|_| Error::Invalid("Query Plot: Select a Display Resolution Within This Host's Address Range".into()))?;
    let display = if kind == PlotKind::Xy {
        requested
    } else if temporal {
        requested.max(4)
    } else {
        request.panel_width.map(|width| width as usize * 4).unwrap_or(requested).min(requested).max(4)
    };
    let bins = NonZeroUsize::new(request.panel_width.map(|width| width as usize).unwrap_or(display / 4).min(display / 4).max(1)).unwrap();
    let (after, through) = store.step_range(
        execution,
        request.scope.as_ref().and_then(|scope| scope.step_id.as_deref()).unwrap_or_default(),
        cut.frontier,
        NonZeroUsize::new(maximum).unwrap(),
    )?;
    let (start_ns, end_ns) = if let Some(viewport) =
        request.viewport.as_ref().filter(|_| matches!(kind, PlotKind::TimeSeries | PlotKind::Unspecified | PlotKind::HeatStrip))
    {
        (ns(viewport.from)?, ns(viewport.to)?)
    } else {
        let bounds = if kind == PlotKind::HeatStrip {
            store.plot_source_bounds(cut, after, through)?
        } else {
            store.numeric_bounds(execution, stream, &fields[0], through)?
        }
        .unwrap_or((0, 0));
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
        stream: stream.to_owned(),
        x_key: x.clone().unwrap_or_else(|| "timestamp".into()),
        series_by: group.clone(),
        max_points: display as u64,
        window_s: request.window_s,
        generated_at: projection::timestamp(now_ns).to_string(),
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
            stream,
            field: &field,
            after,
            through,
            start_ns,
            end_ns,
            bins,
        };

        let mut column_groups = None;
        let frame_start = frame.series.len();
        let count = if let Some(group_field) = group
            .as_deref()
            .filter(|_| matches!(kind, PlotKind::TimeSeries | PlotKind::Unspecified | PlotKind::Histogram | PlotKind::BoxPlot))
        {
            let groups = store.plot_group_values(window, group_field, (display / 4).max(1))?;
            let mut total = 0;
            let mut counts = Vec::new();
            let within_budget =
                matches!(kind, PlotKind::TimeSeries | PlotKind::Unspecified) && store.time_series_window(window)?.2 <= display as u64;
            let group_display = if within_budget { display } else { (display / groups.len().max(1)).max(4) };
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
            let settings = KindWindow {
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
        } else if kind == PlotKind::HeatStrip {
            let reduction = request
                .reduction
                .map(Reduction::try_from)
                .transpose()
                .map_err(|_| Error::Invalid("Query Heat Strip: Select Last, Max, Mean or Count".into()))?
                .unwrap_or(Reduction::Last);
            let (series, count, groups) = store.heat_strip_window(
                KindWindow {
                    window,
                    x: cell.as_deref(),
                    z: None,
                    group: facet.as_deref(),
                    limit: display,
                },
                reduction,
            )?;
            column_groups = Some(groups);
            frame.point_count += series.point_count;
            frame.series.extend(series.series);
            count
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
                KindWindow {
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
                KindWindow {
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
            let mut summary = store.series_summary(&summary_window)?;
            payload.summaries.retain(|summary| summary.series_id != field);
            summary.series_id = field.clone();
            summary.window = request.summary_window;
            payload.summaries.push(summary);
        }
        examined += count;
        if let Some(groups) = column_groups {
            for (index, (group, raw_points)) in groups.into_iter().enumerate() {
                let served = if temporal {
                    frame.series.get(frame_start + index).map_or(display as u64, |column| column.y.len() as u64)
                } else {
                    display as u64
                };
                payload.series.push(PlotSeries {
                    key: if group.is_empty() { field.clone() } else { format!("{field}/{group}") },
                    label: if group.is_empty() { field.clone() } else { group },
                    unit: request.unit.clone(),
                    raw_points,
                    downsampled: raw_points > served,
                    ..Default::default()
                });
            }
        } else {
            let served = if temporal {
                frame.series.get(frame_start).map_or(display as u64, |column| column.y.len() as u64)
            } else {
                display as u64
            };
            payload.series.push(PlotSeries {
                key: field.clone(),
                label: field,
                unit: request.unit.clone(),
                raw_points: count,
                downsampled: count > served,
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
                if !request.marker_record_types.iter().any(|pattern| selection::matches(pattern, kind))
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
    let bucketed = matches!(kind, PlotKind::Unspecified | PlotKind::TimeSeries)
        && !matches!(technique, Downsample::Raw | Downsample::Lttb)
        && point_count < examined;
    if !bucketed {
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
        plot_id: request.plot_id.clone().unwrap_or_else(|| stream.to_owned()),
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
            resolution_nanos: if bucketed { resolution } else { 0 },
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
    use std::f64::consts::TAU;
    use std::thread;

    use harmos_protocol::model::{Declarations, StreamDeclaration};
    use prost::Message;
    use prost_types::{DescriptorProto, FieldDescriptorProto, FileDescriptorProto, FileDescriptorSet, field_descriptor_proto::Type};
    use tempfile::TempDir;

    use super::{DEFAULT_POINTS, PlotColumn, PlotPayload, Value, batch, order_categories, plot_analysis_result, snapshots_at, value};
    use dokime_protocol::host::evidence::RecordsAppendRequest;
    use dokime_protocol::host::plots::NullableNumber;
    use dokime_protocol::host::plots::{PlotRequest, PlotWindow, SnapshotsListResponse};
    use dokime_protocol::model::{Downsample, EvidenceScope, FieldReference, Import, Origin, PlotKind, origin};
    use dokime_protocol::model::{FilterOperator, Metadata, PlotFilter, RecordContract, RecordContracts};
    use dokime_storage::{Evidence, Execution, Sample, Store};

    fn readings(fields: &[(&str, Type)]) -> (TempDir, Store) {
        let root = tempfile::tempdir().unwrap();
        let store = Store::open(root.path()).unwrap();
        let field = fields
            .iter()
            .enumerate()
            .map(|(index, (name, kind))| FieldDescriptorProto {
                name: Some((*name).into()),
                number: Some(index as i32 + 1),
                r#type: Some(*kind as i32),
                ..Default::default()
            })
            .collect();
        let mut catalog = Declarations {
            types: Some(FileDescriptorSet {
                file: vec![FileDescriptorProto {
                    name: Some("plots.proto".into()),
                    package: Some("plots".into()),
                    syntax: Some("proto3".into()),
                    message_type: vec![DescriptorProto {
                        name: Some("Row".into()),
                        field,
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
        catalog.sections.push(Default::default());
        let section = catalog.sections.last_mut().unwrap();
        section.r#type = "dokime.model.v1.RecordContracts".into();
        section.payload = RecordContracts {
            records: vec![RecordContract {
                metadata: Some(Metadata {
                    identifier: "status".into(),
                    ..Default::default()
                }),
                message: "plots.Row".into(),
            }],
        }
        .encode_to_vec();
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
        (root, store)
    }

    #[tokio::test]
    async fn batch_larger_than_worker_bound_preserves_every_envelope_in_request_order() {
        let (_root, store) = readings(&[("value", Type::Double)]);
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

    #[derive(Clone, PartialEq, Message)]
    struct Waveform {
        #[prost(double, tag = "1")]
        value: f64,

        #[prost(double, tag = "2")]
        elapsed_s: f64,

        #[prost(bool, tag = "3")]
        phase: bool,
    }

    fn waveform(period_ns: i64) -> (TempDir, Store, Vec<Sample>) {
        let (root, store) = readings(&[("value", Type::Double), ("elapsed_s", Type::Double), ("phase", Type::Bool)]);
        let samples = (0..11801)
            .map(|index| Sample {
                stream: "readings".into(),
                source_ns: index * period_ns,
                row: Waveform {
                    value: (index as f64 * TAU / 20.0).sin(),
                    elapsed_s: index as f64 * period_ns as f64 / 1e9,
                    phase: index > 1000,
                }
                .encode_to_vec(),
                sequence: index as u64,
                ..Default::default()
            })
            .collect::<Vec<_>>();
        store.sample_batch("execution", &samples).unwrap();
        store.flush_samples().unwrap();
        (root, store, samples)
    }

    #[tokio::test]
    async fn default_window_reports_every_statistic_from_constructed_rows() {
        let (_root, store, _) = waveform(1_000_000_000);
        let mut cut = store.cut("execution").unwrap();
        cut.frontier = 5;
        let (reader, _cancellation) = store.read_session().unwrap();
        let request = PlotRequest {
            scope: Some(EvidenceScope {
                execution_id: Some("execution".into()),
                ..Default::default()
            }),
            stream: "readings".into(),
            y_keys: vec!["elapsed_s".into()],
            plot_type: PlotKind::TimeSeries as i32,
            max_points: Some(4),
            ..Default::default()
        };
        for technique in [Downsample::Auto, Downsample::MinMax, Downsample::Lttb] {
            let request = PlotRequest {
                technique: technique as i32,
                ..request.clone()
            };
            let envelopes = snapshots_at(&reader, &cut, vec![request.clone()], 1000, 0, "epoch").await.unwrap();
            let summary = &envelopes[0].payload.as_ref().unwrap().summaries[0];
            assert_eq!(summary.count, 5);
            for (actual, expected) in [(&summary.min, 0.0), (&summary.max, 4.0), (&summary.mean, 2.0), (&summary.median, 2.0)] {
                assert_eq!(actual.as_ref().unwrap().kind, Some(value::Kind::Number(expected)));
            }
            assert!((summary.std.unwrap() - 2.0_f64.sqrt()).abs() < 1e-12);
            assert!((summary.rms.unwrap() - 6.0_f64.sqrt()).abs() < 1e-12);
            let explicit = PlotRequest {
                summary_window: Some(PlotWindow { from: 0.0, to: 4.0 }),
                ..request
            };
            let explicit = snapshots_at(&reader, &cut, vec![explicit], 1000, 0, "epoch").await.unwrap();
            assert_eq!(summary, &explicit[0].payload.as_ref().unwrap().summaries[0]);
        }
    }

    #[tokio::test]
    async fn boolean_time_series_reduce_to_states_without_changing_numeric_reduction() {
        let (_root, store, samples) = waveform(60_000_000_000);
        let cut = store.cut("execution").unwrap();
        let (reader, _cancellation) = store.read_session().unwrap();
        let request = PlotRequest {
            scope: Some(EvidenceScope {
                execution_id: Some("execution".into()),
                ..Default::default()
            }),
            stream: "readings".into(),
            y_keys: vec!["phase".into(), "elapsed_s".into()],
            plot_type: PlotKind::TimeSeries as i32,
            max_points: Some(16),
            summary_window: Some(PlotWindow { from: 0.0, to: 708000.0 }),
            ..Default::default()
        };
        for technique in [Downsample::Auto, Downsample::MinMax, Downsample::Lttb] {
            let request = PlotRequest {
                technique: technique as i32,
                ..request.clone()
            };
            let envelopes = snapshots_at(&reader, &cut, vec![request.clone(), request], 1000, 0, "epoch").await.unwrap();
            assert_eq!(envelopes[0].payload, envelopes[1].payload);
            let payload = envelopes[0].payload.as_ref().unwrap();
            for field in ["phase", "elapsed_s"] {
                let index = payload.series.iter().position(|series| series.key == field).unwrap();
                let column = &payload.frame.as_ref().unwrap().series[index];
                assert!(column.y.len() <= 16);
                assert_eq!(payload.series[index].raw_points, samples.len() as u64);
                let summary = payload.summaries.iter().find(|summary| summary.series_id == field).unwrap();
                assert_eq!(summary.count, samples.len() as u64);
                if field == "phase" {
                    assert!(column.y.iter().all(|value| matches!(value.value, Some(0.0 | 1.0))));
                    assert_eq!(column.y.first().unwrap().value, Some(0.0));
                    assert_eq!(column.y.last().unwrap().value, Some(1.0));
                    assert!(summary.mean.is_none());
                    assert!(summary.rms.is_none());
                    assert!(summary.std.is_none());
                } else {
                    assert_eq!(column.y.first().unwrap().value, Some(0.0));
                    assert_eq!(column.y.last().unwrap().value, Some(708000.0));
                    assert_eq!(summary.mean.as_ref().unwrap().kind, Some(value::Kind::Number(354000.0)));
                }
            }
        }
    }

    #[tokio::test]
    async fn xy_display_budget_preserves_samples_and_uses_the_requested_point_count() {
        let (_root, store, samples) = waveform(1_000_000);
        let cut = store.cut("execution").unwrap();
        let (reader, _cancellation) = store.read_session().unwrap();
        for technique in [Downsample::Auto, Downsample::Raw, Downsample::MinMax, Downsample::Lttb] {
            for budget in [1, 2, 3, 2000, 11801, 20000] {
                let request = PlotRequest {
                    plot_id: Some("voltage".into()),
                    scope: Some(EvidenceScope {
                        execution_id: Some("execution".into()),
                        ..Default::default()
                    }),
                    stream: "readings".into(),
                    x_key: Some("elapsed_s".into()),
                    y_keys: vec!["value".into()],
                    plot_type: PlotKind::Xy as i32,
                    technique: technique as i32,
                    max_points: Some(budget),
                    ..Default::default()
                };
                let envelopes = batch(&reader, vec![request.clone()], None, 1000, 0, "epoch", Some(4 * 1024 * 1024)).await.unwrap();
                let payload = envelopes[0].payload.as_ref().unwrap();
                let columns = &payload.frame.as_ref().unwrap().series;
                assert_eq!(columns[0].y.len(), budget.min(samples.len() as u64) as usize);
                assert_eq!(envelopes[0].diagnostics.as_ref().unwrap().raw_point_count, samples.len() as u64);
                if budget >= samples.len() as u64 {
                    assert!(payload.series.iter().all(|series| !series.downsampled));
                    for (index, (x, y)) in columns[0].x.iter().zip(&columns[0].y).enumerate() {
                        assert_eq!(x.kind, Some(value::Kind::Number(index as f64 / 1000.0)));
                        assert_eq!(y.value, Some((index as f64 * TAU / 20.0).sin()));
                    }
                }
                if budget == 2000 && technique == Downsample::Auto {
                    let grouped = PlotRequest {
                        series_by: Some("phase".into()),
                        ..request.clone()
                    };
                    let grouped = snapshots_at(&reader, &cut, vec![grouped], 1000, 0, "epoch").await.unwrap();
                    let payload = grouped[0].payload.as_ref().unwrap();
                    assert_eq!(payload.frame.as_ref().unwrap().point_count, budget);
                    assert_eq!(payload.series.len(), 2);
                    assert!(payload.series.iter().all(|series| series.downsampled));
                }
                let response = SnapshotsListResponse { envelopes, ..Default::default() };
                if budget >= samples.len() as u64 {
                    let temporal = PlotRequest {
                        plot_type: PlotKind::TimeSeries as i32,
                        panel_width: Some(1),
                        ..request.clone()
                    };
                    let temporal = snapshots_at(&reader, &cut, vec![temporal], 1000, 0, "epoch").await.unwrap();
                    let payload = temporal[0].payload.as_ref().unwrap();
                    assert_eq!(payload.frame.as_ref().unwrap().series[0].y.len(), samples.len());
                    assert_eq!(payload.bucket_width_s, None);
                    assert_eq!(temporal[0].diagnostics.as_ref().unwrap().resolution_nanos, 0);
                    let grouped = PlotRequest {
                        plot_type: PlotKind::TimeSeries as i32,
                        series_by: Some("phase".into()),
                        ..request.clone()
                    };
                    let grouped = snapshots_at(&reader, &cut, vec![grouped], 1000, 0, "epoch").await.unwrap();
                    let frame = grouped[0].payload.as_ref().unwrap().frame.as_ref().unwrap();
                    assert_eq!(frame.point_count, samples.len() as u64);
                    assert_eq!(frame.series.len(), 2);
                }
                eprintln!(
                    "XY {technique:?}: {budget} Budget, {} Points, {} Encoded Bytes",
                    response.envelopes[0].diagnostics.as_ref().unwrap().point_count,
                    response.encoded_len()
                );
                if budget == 2000 && technique == Downsample::Auto {
                    let exact = batch(&reader, vec![request], None, 1000, 0, "epoch", Some(response.encoded_len())).await.unwrap();
                    assert_eq!(exact, response.envelopes);
                }
            }
        }
    }

    #[derive(Clone, PartialEq, Message)]
    struct Tracking {
        #[prost(double, tag = "1")]
        value: f64,

        #[prost(double, tag = "2")]
        lost: f64,

        #[prost(uint64, tag = "3")]
        cell: u64,
    }

    #[tokio::test]
    async fn nonfinite_samples_are_skipped_like_missing_values_by_every_plot() {
        // Declare a Tracked Distance, an Always-Lost Distance and a Status Record
        let (_root, store) = readings(&[("value", Type::Double), ("lost", Type::Double), ("cell", Type::Uint64)]);

        // Lose the Target Between Finite Readings and Sample It at Status Events
        let values = [1.0, f64::NAN, 2.0, f64::INFINITY, 3.0, f64::NEG_INFINITY, 4.0, 5.0];
        for (index, value) in values.into_iter().enumerate() {
            let source_ns = index as i64 * 1_000_000_000;
            let row = Tracking {
                value,
                lost: [f64::NAN, f64::INFINITY, f64::NEG_INFINITY][index % 3],
                cell: index as u64 % 2 + 1,
            };
            let sample = Sample {
                stream: "readings".into(),
                source_ns,
                sequence: index as u64,
                row: row.encode_to_vec(),
                ..Default::default()
            };
            store.sample_batch("execution", &[sample]).unwrap();
            if [1, 2, 3, 6].contains(&index) {
                let status = Evidence {
                    kind: "record:status".into(),
                    source_ns: source_ns + 500_000_000,
                    payload: RecordsAppendRequest::default().encode_to_vec(),
                    ..Default::default()
                };
                store.append("execution", status).unwrap();
            }
        }
        store.flush_samples().unwrap();

        // Request Every Plot Reading the Distances
        let cut = store.cut("execution").unwrap();
        let (reader, _cancellation) = store.read_session().unwrap();
        let request = |kind: PlotKind, field: &str| PlotRequest {
            scope: Some(EvidenceScope {
                execution_id: Some("execution".into()),
                ..Default::default()
            }),
            stream: "readings".into(),
            y_keys: vec![field.into()],
            plot_type: kind as i32,
            ..Default::default()
        };
        let distance = FieldReference::new("readings").field("value").at("status");
        let sampled = PlotRequest {
            stream: String::new(),
            y_keys: Vec::new(),
            x_expression: Some(distance.clone()),
            filters: vec![PlotFilter {
                expression: Some(distance),
                operator: FilterOperator::Gt as i32,
                value: Some(Value { kind: Some(value::Kind::Number(0.0)) }),
            }],
            ..request(PlotKind::Histogram, "value")
        };
        let heat_strip = PlotRequest {
            cell_expression: Some(FieldReference::new("readings").field("cell")),
            ..request(PlotKind::HeatStrip, "value")
        };
        let mut payloads = Vec::new();
        for request in [
            request(PlotKind::TimeSeries, "value"),
            request(PlotKind::TimeSeries, "lost"),
            request(PlotKind::Histogram, "value"),
            request(PlotKind::Histogram, "lost"),
            sampled,
            heat_strip,
            PlotRequest {
                series_by: Some("value".into()),
                ..request(PlotKind::TimeSeries, "value")
            },
            PlotRequest {
                max_points: Some(4),
                ..request(PlotKind::TimeSeries, "value")
            },
        ] {
            payloads.push(snapshots_at(&reader, &cut, vec![request], 1000, 0, "epoch").await.unwrap().remove(0).payload.unwrap());
        }
        let [series, lost, histogram, lost_histogram, sampled, heat_strip, grouped, bucketed] = payloads.try_into().unwrap();

        // Check Time Series Serve and Summarize Only Finite Samples
        let number = |value: f64| Some(Value { kind: Some(value::Kind::Number(value)) });
        let summary = &series.summaries[0];
        // An Open Execution's Full-History Median Waits for Its Seal.
        assert_eq!((summary.count, &summary.min, &summary.max, &summary.median), (5, &number(1.0), &number(5.0), &None));
        let ys = |payload: &PlotPayload| {
            payload.frame.as_ref().unwrap().series.iter().flat_map(|column| &column.y).map(|y| y.value).collect::<Vec<_>>()
        };
        assert_eq!(ys(&series), [Some(1.0), Some(2.0), Some(3.0), Some(4.0), Some(5.0)]);
        assert_eq!((bucketed.summaries[0].count, bucketed.bucket_width_s.is_some(), ys(&bucketed)), (5, true, vec![Some(1.0), Some(5.0)]));
        assert_eq!((lost.summaries[0].count, &lost.summaries[0].min), (0, &None));
        assert!(ys(&lost).is_empty());
        assert_eq!((grouped.series.len(), ys(&grouped)), (5, ys(&series)));
        let percentile = store.document_value(&cut, &FieldReference::new("readings").field("value").percentile(75.0)).unwrap();
        assert_eq!(Some(percentile), number(4.0));

        // Check Histograms of Streams and of Status Events Bin Only Finite Samples
        // A Filtered Selection Ranks Its Own Rows; the Open Stream's Full History Waits for Its Seal.
        for (payload, count, median) in [(&histogram, 5, None), (&sampled, 4, number(2.0)), (&lost_histogram, 0, None)] {
            let Some(plot_analysis_result::Kind::Histogram(analysis)) = payload.analysis.clone().and_then(|analysis| analysis.result?.kind)
            else {
                panic!("expected a histogram");
            };
            assert_eq!(analysis.series[0].count, count);
            assert_eq!(analysis.series[0].bins.iter().map(|bin| bin.count).sum::<u64>(), count);
            assert_eq!(analysis.series[0].statistics.clone().and_then(|statistics| statistics.median), median);
        }

        // Check Heat Strip Cells Show No Value for Nonfinite Samples
        assert_eq!(ys(&heat_strip), [Some(1.0), Some(2.0), Some(3.0), Some(4.0), None, None, None, Some(5.0)]);

        // Lose the Target for One Second in Every Ten of Two Minutes Sampled at 1 kHz
        let (_dense_root, dense) = readings(&[("value", Type::Double)]);
        let value = |index: i64| match index % 10_000 {
            0..1000 => f64::NAN,
            5000 => f64::INFINITY,
            _ => (index as f64 / 100.0).sin(),
        };
        for chunk in (0..120_000).collect::<Vec<i64>>().chunks(10_000) {
            let samples = chunk
                .iter()
                .map(|index| Sample {
                    stream: "readings".into(),
                    source_ns: index * 1_000_000,
                    sequence: *index as u64,
                    row: Tracking {
                        value: value(*index),
                        ..Default::default()
                    }
                    .encode_to_vec(),
                    ..Default::default()
                })
                .collect::<Vec<_>>();
            dense.sample_batch("execution", &samples).unwrap();
        }
        dense.flush_samples().unwrap();

        // Check Every Downsampling Technique Renders Only the Finite Samples
        let finite = (0..120_000).filter(|index| value(*index).is_finite()).count() as u64;
        let (cut, (reader, _cancellation)) = (dense.cut("execution").unwrap(), dense.read_session().unwrap());
        for technique in [Downsample::Auto, Downsample::MinMax, Downsample::Lttb] {
            let request = PlotRequest {
                technique: technique as i32,
                ..request(PlotKind::TimeSeries, "value")
            };
            let payload = snapshots_at(&reader, &cut, vec![request], 1000, 0, "epoch").await.unwrap().remove(0).payload.unwrap();
            let points = ys(&payload);
            assert_eq!((payload.summaries[0].count, payload.series[0].raw_points, payload.series[0].downsampled), (finite, finite, true));
            assert!((2..=DEFAULT_POINTS as usize).contains(&points.len()) && points.iter().all(|y| y.is_some_and(f64::is_finite)));
        }
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
