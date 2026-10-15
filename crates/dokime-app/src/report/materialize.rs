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
use std::{
    collections::{BTreeMap, BTreeSet},
    num::NonZeroUsize,
};

// Third Party Imports
use harmos::{
    MemorySize,
    storage::{QueryValue, ReadLimits, StorageProductionPreparation, StorageView},
};

// Local Imports
use super::{CapturedExecutionReportInput, CycleReportInput, ReportArtifactLink};
use crate::{
    Error, Result, RuntimeHandle,
    document::capture::CardBindingResult,
    execution::{TestExecution, context::source::CapturedSource},
    plot::StreamedPlot,
    runtime::storage::execution::ExecutionView,
};
use dokime::execution::saved::ExecutionOutputContent;

/* === Constants === */
/// How often a step meets contended storage memory before it gives way.
const CONTENTION_TRIES: u32 = 5;
const CONTENTION_FIRST_DELAY: std::time::Duration = std::time::Duration::from_millis(100);

/// The fewest points a halved plot keeps; under this a waveform is unreadable.
const MIN_PLOT_POINTS: usize = 100;

/// Every step a report takes before it gives up, from whole to bare.
pub(super) const LADDER: [Degradation; 8] = [
    Degradation::Full,
    Degradation::Halved(1),
    Degradation::Halved(2),
    Degradation::Halved(3),
    Degradation::Halved(4),
    Degradation::Omitted,
    Degradation::Truncated(1_000),
    Degradation::Truncated(0),
];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// Plot collectors for a report no document of which evaluates raw rows.
struct PagedPlots {
    /// Plots reduced sample by sample, keyed by document and plot.
    streamed: BTreeMap<(String, String), StreamedPlot>,
    /// Streams the remaining whole-capture plots read; only their rows are kept.
    streams: Vec<String>,
}

/// The step a report renders at, why, and what it has already left out.
struct Ladder<'a> {
    degradation: Degradation,
    cause: &'a str,
    omissions: Vec<String>,
}

/// What a report materializes from.
struct Evidence<'a> {
    records: &'a [dokime::records::RecordEntry],
    source: PlotSource,
    preview: &'a [dokime::telemetry::TelemetrySample],
}

/* === Enums === */
/// How far a report gives way when its evidence does not fit its budget. Each
/// step keeps more of the report than the next: a report is always produced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Degradation {
    /// Everything the documents ask for.
    Full,
    /// Every plot keeps its points halved this many times, never under
    /// [`MIN_PLOT_POINTS`].
    Halved(u32),
    /// Plots become placeholders that say why they are missing.
    Omitted,
    /// Plots omitted, and records and samples cut after this many rows.
    Truncated(usize),
}

/// Where a report's plots read their evidence.
enum PlotSource {
    /// The execution's rows, read whole for documents that evaluate them.
    Rows(QueryValue<Vec<dokime::telemetry::TelemetrySample>>),
    /// Collectors fed one page at a time, beside the rows of only those
    /// streams the report's whole-capture plots read.
    Paged {
        streamed: BTreeMap<(String, String), StreamedPlot>,
        rows: QueryValue<Vec<dokime::telemetry::TelemetrySample>>,
    },
}

/* === Traits === */

/* === Implementations === */
impl Degradation {
    /// The points a plot keeps at this step, given what it asked for.
    fn points(self, requested: Option<usize>) -> Option<usize> {
        let Self::Halved(times) = self else {
            return requested;
        };
        let asked = requested
            .filter(|points| *points > 0)
            .unwrap_or(dokime::document::DEFAULT_PLOT_MAX_POINTS);
        Some((asked >> times).max(MIN_PLOT_POINTS).min(asked))
    }

    fn plots(self) -> bool {
        matches!(self, Self::Full | Self::Halved(_))
    }

    fn rows(self) -> Option<usize> {
        match self {
            Self::Truncated(rows) => Some(rows),
            _ => None,
        }
    }
}

/* === Public Functions === */
/// One execution's report input, degraded step by step until it fits.
pub(super) async fn execution(
    runtime: &RuntimeHandle,
    inspection: &ExecutionView,
    final_report: bool,
) -> Result<QueryValue<CapturedExecutionReportInput>> {
    climb(|degradation, cause| attempt(runtime, inspection, final_report, degradation, cause)).await
}

/// Runs `step` down [`LADDER`] until one fits. A step that meets other
/// readers holding the shared storage memory is tried again before the
/// report gives anything up; a step over its own budget gives way to the next.
pub(super) async fn climb<T, F>(mut step: impl FnMut(Degradation, Option<String>) -> F) -> Result<T>
where
    F: std::future::Future<Output = Result<T>>,
{
    let mut cause = None::<String>;
    for degradation in LADDER {
        let mut delay = CONTENTION_FIRST_DELAY;
        let mut tries = 0;
        loop {
            match step(degradation, cause.clone()).await {
                Err(error) if contended(&error) && tries < CONTENTION_TRIES => {
                    tries += 1;
                    tokio::time::sleep(delay).await;
                    delay = delay.saturating_mul(2);
                }
                Err(error) if exceeds_budget(&error) => {
                    tracing::warn!(%error, ?degradation, "report does not fit its budget; degrading");
                    cause = Some(error.to_string());
                    break;
                }
                outcome => return outcome,
            }
        }
    }
    Err(Error::InvalidReportInputs(format!(
        "report does not fit its budget even without plots, records and samples: {}",
        cause.unwrap_or_default()
    )))
}

/// Whether `error` is other readers holding the shared storage memory: a
/// moment to wait out rather than a budget this report is over.
pub(crate) fn contended(error: &Error) -> bool {
    matches!(
        error,
        Error::Harmos(harmos::Error::ResourceExhausted {
            resource: "storage read memory"
        }) | Error::Storage(dokime::storage::Error::Runtime(
            harmos::Error::ResourceExhausted {
                resource: "storage read memory"
            }
        ))
    )
}

/// Whether `error` is a budget a degraded report could fit, rather than
/// evidence that cannot be read at all.
fn exceeds_budget(error: &Error) -> bool {
    let budget = |error: &harmos::Error| {
        matches!(
            error,
            harmos::Error::ResourceExhausted { .. }
                | harmos::Error::InvalidInput {
                    field: "storage production",
                    ..
                }
        )
    };
    match error {
        Error::Harmos(error) | Error::Storage(dokime::storage::Error::Runtime(error)) => {
            budget(error)
        }
        _ => false,
    }
}

/// One execution's report input at one step of the ladder. `cause` is why the
/// previous step did not fit, recorded beside whatever this step leaves out.
pub(super) async fn attempt(
    runtime: &RuntimeHandle,
    inspection: &ExecutionView,
    final_report: bool,
    degradation: Degradation,
    cause: Option<String>,
) -> Result<QueryValue<CapturedExecutionReportInput>> {
    let cause = cause.unwrap_or_else(|| "its budget".to_owned());
    let mut omissions = Vec::new();
    let captured = inspection
        .captured_source()
        .ok_or_else(|| invalid("report snapshot has no captured execution context"))?
        .clone();
    captured.check_origin(runtime.storage())?;
    let view = inspection.view().clone();
    // The Report Fills Its Own Profile; Contention For It Is Retried By The Ladder.
    let limit = runtime.config().storage_reads.materialization();
    let page = ReadLimits {
        rows: runtime.config().storage_reads.page_rows,
        bytes: runtime.config().storage_reads.page_bytes,
    };
    let mut records = empty(runtime.storage())?;
    let mut reader = inspection.context_records(runtime.storage(), page, limit)?;
    while let Some(rows) = reader.next().await? {
        let room = degradation
            .rows()
            .map_or(usize::MAX, |rows| rows.saturating_sub(records.len()));
        let projected = rows.try_project(limit, |rows, allowance| {
            let bytes = rows.iter().take(room).fold(
                std::mem::size_of::<Vec<dokime::records::RecordEntry>>(),
                |bytes, row| bytes.saturating_add(row.record.memory_size()),
            );
            check(bytes, allowance)?;
            Ok(rows
                .iter()
                .take(room)
                .map(|row| row.record.clone())
                .collect::<Vec<_>>())
        })?;
        let cut = rows.len() > room;
        drop(rows);
        records = append(&view, records, &projected, limit)?;
        if cut {
            omissions.push(format!(
                "Records truncated at {} rows: the report did not fit {cause}.",
                records.len()
            ));
            break;
        }
    }
    drop(reader);
    let execution = inspection.revision().clone();
    // A report no document of which evaluates raw rows reads the capture one
    // bounded page at a time: plots that reduce sample by sample keep compact
    // rows, and plots that need the whole capture keep only the rows of the
    // streams they read. A document that evaluates raw rows itself still
    // reads the execution's rows whole, charged and failing closed.
    let (source, telemetry) =
        match paged_plots(&captured, &execution.value, &records, limit, degradation)? {
            Some(PagedPlots {
                mut streamed,
                streams,
            }) => {
                let mut kept = empty(runtime.storage())?;
                // The preview keeps every row of a capture no longer than a plot's
                // point budget, so such a capture is read once.
                let mut early = empty(runtime.storage())?;
                let mut count = 0usize;
                let mut reader = inspection.telemetry(runtime.storage(), page, limit)?;
                let mut cut = false;
                while !cut && let Some(rows) = reader.next().await? {
                    let mut selected = Vec::new();
                    let mut first = Vec::new();
                    for row in rows.iter() {
                        if degradation.rows().is_some_and(|rows| count >= rows) {
                            cut = true;
                            break;
                        }
                        if count < dokime::document::DEFAULT_PLOT_MAX_POINTS {
                            first.push(row.sample.clone());
                        }
                        count = count.saturating_add(1);
                        for plot in streamed.values_mut() {
                            plot.push(&row.sample);
                        }
                        if streams.iter().any(|stream| {
                            crate::plot::stream_matches_alias(row.sample.stream.as_str(), stream)
                        }) {
                            selected.push(row.sample.clone());
                        }
                    }
                    drop(rows);
                    kept = keep(runtime, &view, kept, selected, limit)?;
                    early = keep(runtime, &view, early, first, limit)?;
                    let held = streamed.values().fold(
                        records
                            .memory_size()
                            .saturating_add(kept.memory_size())
                            .saturating_add(early.memory_size()),
                        |bytes, plot| bytes.saturating_add(plot.memory_size()),
                    );
                    check(held, limit)?;
                }
                drop(reader);
                if cut {
                    omissions.push(format!(
                        "Samples truncated at {count} rows: the report did not fit {cause}."
                    ));
                }
                let preview = if count <= dokime::document::DEFAULT_PLOT_MAX_POINTS {
                    early
                } else {
                    drop(early);
                    preview(runtime, inspection, &view, page, limit, count).await?
                };
                (
                    PlotSource::Paged {
                        streamed,
                        rows: kept,
                    },
                    preview,
                )
            }
            None => {
                let mut telemetry = empty(runtime.storage())?;
                let mut reader = inspection.context_telemetry(runtime.storage(), page, limit)?;
                while let Some(rows) = reader.next().await? {
                    let room = degradation
                        .rows()
                        .map_or(usize::MAX, |rows| rows.saturating_sub(telemetry.len()));
                    let projected = rows.try_project(limit, |rows, allowance| {
                        let bytes = rows.iter().take(room).fold(
                            std::mem::size_of::<Vec<dokime::telemetry::TelemetrySample>>(),
                            |bytes, row| bytes.saturating_add(row.sample.memory_size()),
                        );
                        check(bytes, allowance)?;
                        Ok(rows
                            .iter()
                            .take(room)
                            .map(|row| row.sample.clone())
                            .collect::<Vec<_>>())
                    })?;
                    let cut = rows.len() > room;
                    drop(rows);
                    telemetry = append(&view, telemetry, &projected, limit)?;
                    if cut {
                        omissions.push(format!(
                            "Samples truncated at {} rows: the report did not fit {cause}.",
                            telemetry.len()
                        ));
                        break;
                    }
                }
                drop(reader);
                let rows = telemetry;
                let preview = view.project_owned(
                    empty::<dokime::telemetry::TelemetrySample>(runtime.storage())?,
                    limit,
                    |_, allowance| {
                        let selected = super::render::cycle::selection(rows.len())
                            .into_iter()
                            .map(|index| rows[index].clone())
                            .collect::<Vec<_>>();
                        check(selected.memory_size(), allowance)?;
                        Ok(selected)
                    },
                )?;
                (PlotSource::Rows(rows), preview)
            }
        };
    let renderer = runtime.clone();
    let render_view = view.clone();
    // The Projection's Allowance Bounds Everything Rendering Builds, Documents
    // Included; A Second View-Less Reservation Would Charge The Pool Twice.
    let render = move || {
        render_view
            .project_shared(&captured, limit, |captured, allowance| {
                materialize(
                    &renderer,
                    captured,
                    &execution.value,
                    Evidence {
                        records: &records,
                        source,
                        preview: &telemetry,
                    },
                    final_report,
                    Ladder {
                        degradation,
                        cause: &cause,
                        omissions,
                    },
                    allowance,
                )
                .map_err(codec)
            })
            .map_err(Error::from)
    };
    let render_bytes = std::mem::size_of_val(&render)
        .checked_add(std::mem::size_of::<StorageProductionPreparation>())
        .ok_or_else(|| invalid("report materialization task memory overflows"))?;
    let render_memory = runtime.storage().prepare_production(bound(render_bytes)?)?;
    let report = tokio::task::spawn_blocking(move || {
        let _memory = render_memory;
        render()
    })
    .await??;
    Ok(report)
}

/* === Private Functions === */
fn materialize(
    runtime: &RuntimeHandle,
    captured: &CapturedSource,
    execution: &TestExecution,
    evidence: Evidence<'_>,
    final_report: bool,
    ladder: Ladder<'_>,
    allowance: NonZeroUsize,
) -> Result<CapturedExecutionReportInput> {
    let Ladder {
        degradation,
        cause,
        mut omissions,
    } = ladder;
    let Evidence {
        records,
        source,
        preview,
    } = evidence;
    let (rows, mut streamed, paged) = match source {
        PlotSource::Rows(rows) => (rows, BTreeMap::new(), false),
        PlotSource::Paged { streamed, rows } => (rows, streamed, true),
    };
    // Plots read the kept rows; documents read raw rows only when they were
    // all kept for them.
    let telemetry: &[dokime::telemetry::TelemetrySample] = &rows;
    let document_telemetry = if paged { &[][..] } else { telemetry };
    let streamed_bytes = streamed.values().fold(0usize, |bytes, plot| {
        bytes.saturating_add(plot.memory_size())
    });
    let records_bytes = records.iter().fold(
        std::mem::size_of::<Vec<dokime::records::RecordEntry>>(),
        |bytes, row| bytes.saturating_add(row.memory_size()),
    );
    let rows_bytes = telemetry.iter().fold(
        std::mem::size_of::<Vec<dokime::telemetry::TelemetrySample>>(),
        |bytes, row| bytes.saturating_add(row.memory_size()),
    );
    let evidence = records_bytes
        .saturating_add(rows_bytes)
        .saturating_add(preview.iter().fold(
            std::mem::size_of::<Vec<dokime::telemetry::TelemetrySample>>(),
            |bytes, row| bytes.saturating_add(row.memory_size()),
        ))
        .saturating_add(streamed_bytes);
    // The Output Owns One Evidence Copy; Documents Borrow Their Inputs.
    let catalogs = captured
        .record_contracts
        .memory_size()
        .saturating_add(captured.telemetry_contracts.memory_size());
    let construction = catalogs
        .saturating_add(captured.memory_size().saturating_sub(catalogs))
        .saturating_add(evidence)
        .saturating_add(std::mem::size_of::<CapturedExecutionReportInput>())
        .saturating_add(captured.context.cycle.cycle.memory_size())
        .saturating_add(execution.memory_size())
        .saturating_add(captured.context.traceability.memory_size())
        .saturating_add(captured.context.session_id.memory_size())
        .saturating_add(execution.test_case_id.memory_size())
        .saturating_add(execution.test_execution_id.memory_size());
    tracing::debug!(
        construction,
        evidence,
        catalogs,
        allowance = allowance.get(),
        "terminal report allocation preflight"
    );
    check(construction, allowance)?;
    // Evidence Is Copied Only Into The Final Input, After Plot Working Buffers Drop.
    // Its Original Query Owners Remain Charged While Documents And Plots Borrow It.
    let working = construction.saturating_sub(evidence);
    let mut cycle = captured.context.cycle.cycle.clone();
    let mut captured_execution = execution.clone();
    captured_execution.saved = None;
    cycle.test_executions = vec![captured_execution];
    let generated_at = captured.captured_at.clone();
    let mut documents = BTreeMap::new();
    let mut plots = BTreeMap::new();
    let selection = crate::document::DocumentSelection {
        scope: execution.scope.clone(),
    };
    for (id, definition) in &captured.context.execution_documents {
        let retained = working
            .saturating_add(documents.memory_size())
            .saturating_add(plots.memory_size());
        let remaining = allowance
            .get()
            .checked_sub(retained)
            .ok_or_else(exhausted)?;
        let selected = crate::document::bounded::capture_document_borrowed(
            crate::document::input::BorrowedInput {
                definition,
                selection: &selection,
                data: crate::document::input::BorrowedData {
                    context: crate::document::input::Context::new(
                        std::slice::from_ref(execution),
                        &selection.scope,
                    ),
                    records: crate::document::input::Records::Native(records),
                    telemetry: document_telemetry,
                    artifacts: crate::document::input::Artifacts::Outputs(&captured.outputs),
                },
            },
            crate::document::bounded::DocumentLimits {
                source_bytes: allowance.get(),
                materialized_bytes: remaining,
                ..Default::default()
            },
        )?;
        check(selected.memory_size().saturating_add(retained), allowance)?;
        for descriptor in selected.cards.iter().filter_map(|card| match &card.result {
            CardBindingResult::Plot { descriptor } => Some(descriptor),
            _ => None,
        }) {
            if !degradation.plots() {
                let reason = format!(
                    "Plot {} omitted: the report did not fit {cause}.",
                    descriptor.metadata.label
                );
                let value = crate::report::value::Value::from_serializable(
                    &serde_json::json!({ "omitted": reason }),
                    allowance,
                )?;
                plots.insert(descriptor.plot_id.to_string(), value);
                omissions.push(reason);
                continue;
            }
            let key = (id.to_string(), descriptor.plot_id.to_string());
            let collector = streamed.remove(&key);
            // Plot Working Buffers Need Their Evidence Allowance. A builder
            // works over its own input only: a streamed plot, already charged
            // with the evidence, becomes points once more while reducing, and
            // a whole-capture plot filters, indexes and projects at most every
            // kept row and record twice.
            let working_input = match &collector {
                Some(plot) => plot.memory_size(),
                None => records_bytes.saturating_add(rows_bytes).saturating_mul(2),
            };
            check(
                working
                    .saturating_add(documents.memory_size())
                    .saturating_add(plots.memory_size())
                    .saturating_add(selected.memory_size())
                    .saturating_add(evidence)
                    .saturating_add(working_input),
                allowance,
            )?;
            let mut request = crate::plot::plot_payload_request_for_descriptor(
                descriptor,
                Some(execution.scope.clone()),
                None,
                None,
                None,
            );
            let asked = request.max_points;
            request.max_points = degradation.points(asked);
            if request.max_points != asked {
                omissions.push(format!(
                    "Plot {} drawn with at most {} points: the report did not fit {cause}.",
                    descriptor.metadata.label,
                    request.max_points.unwrap_or_default()
                ));
            }
            let context = crate::plot::PlotPayloadContext {
                x_format: request.x_format.clone(),
                series_format: request.y_format.clone(),
                run_started_at: execution.started_at.as_ref().map(ToString::to_string),
                cycle_started_at: cycle.started_at.as_ref().map(ToString::to_string),
            };
            let payload = if let Some(plot) = collector {
                plot.finish(records, generated_at.as_str(), &context)
            } else if request.analysis.is_some() {
                let payload = crate::plot::project_plot_payload(
                    telemetry,
                    records,
                    &request,
                    generated_at.as_str(),
                    &context,
                );
                let stream = dokime::core::Identifier::new(&request.stream)
                    .map_err(|_| invalid("invalid analysis stream"))?;
                let schema = telemetry
                    .iter()
                    .find(|sample| sample.stream == stream)
                    .map(|sample| sample.schema_version);
                let contract = schema
                    .and_then(|version| captured.telemetry_contracts.get(&stream)?.get(&version));
                // This Closure Runs On The Existing Blocking Render Worker.
                // The Original Operation Manager Still Owns Callback Execution.
                tokio::runtime::Handle::current().block_on(
                    runtime.analyze_projected_plot(&request, telemetry, payload, contract),
                )?
            } else {
                crate::plot::build_plot_payload(
                    telemetry,
                    records,
                    request,
                    generated_at.as_str(),
                    &context,
                )
            };
            let value = crate::report::value::Value::from_serializable(&payload, allowance)?;
            plots.insert(descriptor.plot_id.to_string(), value);
        }
        documents.insert(id.clone(), selected);
        check(
            documents
                .memory_size()
                .saturating_add(plots.memory_size())
                .saturating_add(working),
            allowance,
        )?;
    }
    let artifact_links = captured
        .outputs
        .iter()
        .filter_map(|output| match &output.content {
            ExecutionOutputContent::Artifact { record, .. } => Some(ReportArtifactLink {
                artifact_id: record.artifact_id.to_string(),
                label: record.filename.clone(),
                href: format!("outputs/{}", output.relative_path),
                path: output.relative_path.clone(),
                media_type: record.content_type.clone(),
                description: None,
                size_bytes: Some(output.file.length),
                group_id: None,
                group_label: None,
                scope: Some(record.scope.clone()),
            }),
            _ => None,
        })
        .collect();
    let appearance =
        crate::report::value::Value::from_serializable(&captured.context.appearance, allowance)?;
    check(
        construction
            .saturating_add(documents.memory_size())
            .saturating_add(plots.memory_size())
            .saturating_add(appearance.memory_size()),
        allowance,
    )?;
    Ok(CapturedExecutionReportInput {
        final_report,
        owner_node_id: captured.owner_node_id.clone(),
        execution: captured.execution.clone(),
        evidence_through: captured.evidence_through.clone(),
        record_contracts: captured.record_contracts.clone(),
        telemetry_contracts: captured.telemetry_contracts.clone(),
        context: captured.context.clone(),
        input: CycleReportInput {
            session_id: captured.context.session_id.clone(),
            cycle,
            // Execution Evidence Belongs To Its Case; Cycle-Level Maps Stay Empty.
            documents: BTreeMap::new(),
            plots: BTreeMap::new(),
            case_documents: vec![crate::report::ReportCaseDocument {
                test_case_id: execution.test_case_id.clone(),
                test_execution_id: Some(execution.test_execution_id.clone()),
                documents,
                plots,
            }],
            records: records.to_vec(),
            telemetry: preview.to_vec(),
            traceability: captured.context.traceability.clone(),
            artifact_links,
            appearance: Some(appearance),
            generated_at,
            omissions,
        },
    })
}
/// Collectors for every report plot that reduces sample by sample, and the
/// streams every other plot reads, when no document evaluates raw telemetry
/// rows and plot ids are unique; `None` otherwise.
fn paged_plots(
    captured: &CapturedSource,
    execution: &TestExecution,
    records: &[dokime::records::RecordEntry],
    allowance: NonZeroUsize,
    degradation: Degradation,
) -> Result<Option<PagedPlots>> {
    let selection = crate::document::DocumentSelection {
        scope: execution.scope.clone(),
    };
    let context =
        crate::document::input::Context::new(std::slice::from_ref(execution), &selection.scope);
    let mut streamed = BTreeMap::new();
    let mut whole = BTreeSet::new();
    let mut streams = Vec::new();
    for (id, definition) in &captured.context.execution_documents {
        if crate::document::binding::telemetry_needed(definition, context) {
            return Ok(None);
        }
        let selected = crate::document::bounded::capture_document_borrowed(
            crate::document::input::BorrowedInput {
                definition,
                selection: &selection,
                data: crate::document::input::BorrowedData {
                    context,
                    records: crate::document::input::Records::Native(records),
                    telemetry: &[],
                    artifacts: crate::document::input::Artifacts::Outputs(&captured.outputs),
                },
            },
            crate::document::bounded::DocumentLimits {
                source_bytes: allowance.get(),
                materialized_bytes: allowance.get(),
                ..Default::default()
            },
        )?;
        for descriptor in selected.cards.iter().filter_map(|card| match &card.result {
            CardBindingResult::Plot { descriptor } => Some(descriptor),
            _ => None,
        }) {
            if !degradation.plots() {
                continue;
            }
            let mut request = crate::plot::plot_payload_request_for_descriptor(
                descriptor,
                Some(execution.scope.clone()),
                None,
                None,
                None,
            );
            request.max_points = degradation.points(request.max_points);
            let key = (id.to_string(), descriptor.plot_id.to_string());
            if streamed.contains_key(&key) || whole.contains(&key) {
                return Ok(None);
            }
            let read = crate::plot::plot_streams(&request);
            match StreamedPlot::new(request) {
                Some(plot) => {
                    streamed.insert(key, plot);
                }
                None => {
                    streams.extend(read);
                    whole.insert(key);
                }
            }
        }
    }
    streams.sort_unstable();
    streams.dedup();
    Ok(Some(PagedPlots { streamed, streams }))
}

/// The bounded sample preview a report shows, selected from the execution's
/// `count` rows by a second pass over the same pages.
async fn preview(
    runtime: &RuntimeHandle,
    inspection: &ExecutionView,
    view: &StorageView,
    page: ReadLimits,
    limit: NonZeroUsize,
    count: usize,
) -> Result<QueryValue<Vec<dokime::telemetry::TelemetrySample>>> {
    let selected = super::render::cycle::selection(count);
    let mut wanted = selected.iter().copied().peekable();
    let mut preview = empty(runtime.storage())?;
    let mut index = 0usize;
    let mut reader = inspection.telemetry(runtime.storage(), page, limit)?;
    while wanted.peek().is_some()
        && let Some(rows) = reader.next().await?
    {
        let mut kept = Vec::new();
        for row in rows.iter() {
            if wanted.peek() == Some(&index) {
                wanted.next();
                kept.push(row.sample.clone());
            }
            index += 1;
        }
        drop(rows);
        preview = keep(runtime, view, preview, kept, limit)?;
    }
    if wanted.peek().is_some() {
        return Err(invalid("report telemetry changed between its two reads"));
    }
    Ok(preview)
}

fn empty<T: MemorySize>(storage: &dokime::storage::Manager) -> Result<QueryValue<Vec<T>>> {
    Ok(storage
        .produce(
            NonZeroUsize::new(std::mem::size_of::<Vec<T>>()).unwrap(),
            |_| Ok(Vec::new()),
        )?
        .try_into_value()?)
}
/// Charges `rows` to a production of their own before they join `original`.
fn keep<T: MemorySize + Clone>(
    runtime: &RuntimeHandle,
    view: &StorageView,
    original: QueryValue<Vec<T>>,
    rows: Vec<T>,
    limit: NonZeroUsize,
) -> Result<QueryValue<Vec<T>>> {
    if rows.is_empty() {
        return Ok(original);
    }
    let bytes = rows.memory_size();
    check(bytes, limit)?;
    let projected = runtime
        .storage()
        .produce(bound(bytes)?, |_| Ok(rows))?
        .try_into_value()?;
    append(view, original, &projected, limit)
}
fn append<T: MemorySize + Clone>(
    view: &StorageView,
    original: QueryValue<Vec<T>>,
    page: &QueryValue<Vec<T>>,
    limit: NonZeroUsize,
) -> Result<QueryValue<Vec<T>>> {
    Ok(
        view.project_owned(original, limit, |mut values, allowance| {
            check(
                values.memory_size().saturating_add(page.memory_size()),
                allowance,
            )?;
            values
                .try_reserve_exact(page.len())
                .map_err(|_| exhausted())?;
            for row in page.iter() {
                values.push(row.clone());
            }
            Ok(values)
        })?,
    )
}
fn bound(bytes: usize) -> Result<NonZeroUsize> {
    NonZeroUsize::new(bytes).ok_or_else(|| invalid("terminal source memory overflows"))
}
fn check(bytes: usize, allowance: NonZeroUsize) -> harmos::Result<()> {
    if bytes == usize::MAX || bytes > allowance.get() {
        tracing::debug!(
            requested = bytes,
            allowed = allowance.get(),
            "terminal report allocation exceeds allowance"
        );
        Err(exhausted())
    } else {
        Ok(())
    }
}
fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "final report materialization memory",
    }
}
fn invalid(reason: &'static str) -> Error {
    Error::Storage(dokime::storage::Error::Invalid(reason))
}
/// A storage refusal keeps its own classification, so shared-budget
/// contention stays retryable; any other failure to build the report is
/// permanent.
fn codec(source: Error) -> harmos::Error {
    if let Error::Harmos(source) | Error::Storage(dokime::storage::Error::Runtime(source)) = source
    {
        return source;
    }
    harmos::Error::backend(
        "materialize terminal report",
        harmos::BackendFailure::Permanent,
        source,
    )
}

/* === Tests === */
