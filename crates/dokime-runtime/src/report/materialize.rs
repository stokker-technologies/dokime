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
use std::collections::BTreeMap;

// Third Party Imports
use futures_util::{StreamExt, stream::FuturesUnordered};
use harmos::{
    operations::Context,
    storage::ReadBatch,
    storage::query::transform::{
        Transform,
        downsample::{Algorithm, Downsample},
    },
};
use tokio::sync::OnceCell;

// Local Imports
use super::{CapturedExecutionReportInput, CycleReportInput, ReportArtifactLink};
use crate::storage::{
    Manager, ReadRows,
    projection::{self, Pages},
};
use crate::{
    document::capture::CardBindingResult,
    execution::context::captured::source::CapturedSource,
    runtime::Error,
    runtime::Result,
    runtime::{progress::Operation, storage::execution::ExecutionView},
};
use dokime_domain::model::execution::saved::ExecutionOutputContent;
use dokime_domain::model::identity::DocumentId;
use dokime_domain::model::records::evidence::RecordEntry;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn sealed_member(
    ctx: &Context<'_>, storage: &Manager, seal: &dokime_domain::model::execution::saved::ExecutionSeal, operation: &Operation,
) -> Result<ReadBatch<CapturedExecutionReportInput>> {
    let inspection = ExecutionView::from_seal_source(storage, seal).await?;
    execution(ctx, &inspection, true, operation).await
}

pub(crate) async fn execution(
    ctx: &Context<'_>, inspection: &ExecutionView, final_report: bool, operation: &Operation,
) -> Result<ReadBatch<CapturedExecutionReportInput>> {
    operation.phase("Materializing Evidence");
    let captured = inspection.captured_source().ok_or_else(|| invalid("report has no captured execution context"))?;
    captured.validate_header()?;
    if captured.evidence_through != *inspection.evidence_through()? {
        return Err(invalid("report capture differs from the requested durable boundary"));
    }
    let execution = inspection.execution();
    let records = ReadRows::from(context_records(inspection).await?);
    let selection = harmos::storage::ReadBatch::new(())
        .try_map(|_| Ok::<_, Error>(crate::document::DocumentSelection { scope: execution.scope.clone() }))?;
    let context = crate::document::input::Context::new(std::slice::from_ref(execution), &selection.scope);
    let needs_rows =
        captured.context.execution_documents.values().any(|definition| crate::document::binding::telemetry_needed(definition, context));
    let telemetry = context_telemetry(ctx, inspection.session(), captured, execution, needs_rows).await?;
    let maximum = usize::try_from(captured.report_defaults.plot_points).map_err(|_| invalid("plot point policy exceeds native size"))?;

    let limit = 64 * 1024 * 1024;
    let mut documents = harmos::storage::ReadBatch::new(())
        .try_map(|_| Ok::<_, Error>(BTreeMap::<DocumentId, crate::document::capture::Capture>::new()))?;
    for (id, definition) in &captured.context.execution_documents {
        let input = crate::document::input::BorrowedInput {
            definition,
            selection: &selection,
            data: crate::document::input::BorrowedData {
                context,
                records: crate::document::input::Records::Native((&records).into()),
                telemetry: if needs_rows { &telemetry } else { &[] },
                artifacts: crate::document::input::Artifacts::Outputs(&captured.outputs),
            },
        };
        let limits = crate::document::bounded::DocumentLimits {
            source_bytes: limit,
            materialized_bytes: limit,
            ..crate::document::bounded::DocumentLimits::default()
        };
        documents = documents.try_map(|mut documents| {
            let document = crate::document::bounded::capture_document_borrowed(input, limits)?;
            documents.insert(id.clone(), document);
            Ok::<_, Error>(documents)
        })?;
    }
    operation.phase("Querying Report Plots");
    let plots = plots(ctx, inspection.session(), captured, execution, (&records).into(), &documents).await?;
    let selected = super::render::cycle::selection(telemetry.len(), maximum);
    let count = selected.clone().count();
    let preview = telemetry.try_map(|telemetry| {
        let mut preview = Vec::new();
        preview.try_reserve_exact(count).map_err(|_| invalid("report preview allocation failed"))?;
        let mut selected = selected.peekable();
        for (index, sample) in telemetry.into_iter().enumerate() {
            if selected.peek() == Some(&index) {
                preview.push(sample);
                selected.next();
            }
        }
        Ok::<_, Error>(preview)
    })?;
    let preview = ReadRows::from(projection::push(harmos::storage::ReadBatch::new(Vec::new()), preview)?);
    operation.phase("Assembling Report");

    let defaults = captured.report_defaults;
    documents
        .try_map_with(plots, |documents, plots| assemble(final_report, captured, execution, documents, plots, (records, preview), defaults))
}

/* === Private Functions === */

async fn context_records(inspection: &ExecutionView) -> Result<Pages<RecordEntry>> {
    let mut records = harmos::storage::ReadBatch::new(Vec::new());
    let mut reader = inspection.context_records().await?;
    while let Some(rows) = reader.next().await? {
        let rows = projection::page(rows, |row| Ok(row.record))?;
        records = projection::push(records, rows)?;
    }
    Ok(records)
}

// Display Telemetry: Every Row When A Document Binds Them, Else A Downsampled Preview.
async fn context_telemetry(
    ctx: &Context<'_>, session: &crate::storage::Manager, captured: &CapturedSource,
    execution: &dokime_domain::model::execution::result::TestExecution, needs_rows: bool,
) -> Result<harmos::storage::ReadBatch<Vec<crate::telemetry::TelemetrySample>>> {
    let mut telemetry = super::query::telemetry::empty()?;
    for versions in captured.telemetry_contracts.values() {
        for contract in versions.values() {
            let transforms = if needs_rows {
                vec![]
            } else {
                vec![Transform::Downsample(Downsample {
                    max_rows: usize::try_from(captured.report_defaults.plot_points).expect("validated report plot point count"),
                    algorithm: Algorithm::Decimate,
                    value: None,
                    axis: None,
                })]
            };
            telemetry = super::query::telemetry::append(
                telemetry,
                super::query::telemetry::read(
                    ctx,
                    session,
                    &captured.evidence_through,
                    execution.test_execution_id.as_str(),
                    contract,
                    transforms,
                )
                .await?,
            )?;
        }
    }
    super::query::telemetry::sort(telemetry)
}

// Plot Queries Respect The Supplied Concurrency; Results Keep Card Order.
async fn plots(
    ctx: &Context<'_>, session: &crate::storage::Manager, captured: &CapturedSource,
    execution: &dokime_domain::model::execution::result::TestExecution, records: crate::records::view::View<'_>,
    documents: &BTreeMap<DocumentId, crate::document::capture::Capture>,
) -> Result<ReadBatch<BTreeMap<String, super::value::Value>>> {
    let mut groups = Vec::<(super::query::ReadPlan, Vec<_>)>::new();
    let mut completed = Vec::new();
    let mut index = 0;
    for document in documents.values() {
        for card in &document.cards {
            let CardBindingResult::Plot { descriptor } = &card.result else {
                continue;
            };
            let request = crate::plot::plot_payload_request_for_descriptor(descriptor, Some(execution.scope.clone()), None, None);
            let plan = super::query::read_plan(&request, &captured.telemetry_contracts)?;
            let plot = (index, descriptor.plot_id.to_string(), request);
            index += 1;
            if let Some((_, requests)) = groups.iter_mut().find(|(candidate, _)| candidate == &plan) {
                reserve_plot_slot(requests)?;
                requests.push(plot);
            } else {
                let mut requests = Vec::new();
                reserve_plot_slot(&mut requests)?;
                requests.push(plot);
                reserve_plot_slot(&mut groups)?;
                groups.push((plan, requests));
            }
        }
    }

    // Shared Query Data Survives Every Concurrent Materialization.
    for (plan, requests) in groups {
        drop(plan);
        let shared = OnceCell::new();
        let mut pending = FuturesUnordered::new();
        for (current, id, request) in requests {
            let shared = &shared;
            pending.push(async move {
                let payload =
                    super::query::materialize(super::query::ReadContext { ctx, session }, captured, execution, records, request, shared)
                        .await?;
                // The Identifier Moves To The Final Map Under Its New Native Owner.
                let payload = payload.try_map(|payload| Ok::<_, Error>((id, payload)))?;
                Ok::<_, Error>((current, payload))
            });
            if pending.len() as u64 == 8_u64
                && let Some(plot) = pending.next().await
            {
                reserve_plot_slot(&mut completed)?;
                completed.push(plot?);
            }
        }
        while let Some(plot) = pending.next().await {
            reserve_plot_slot(&mut completed)?;
            completed.push(plot?);
        }
        drop(pending);
    }
    completed.sort_unstable_by_key(|(index, _)| *index);
    let mut plots = harmos::storage::ReadBatch::new(()).try_map(|_| Ok::<_, Error>(BTreeMap::new()))?;
    for (_, plot) in completed {
        plots = plots.try_map_with(plot, |mut plots, (id, plot)| {
            if plots.insert(id, plot).is_some() {
                return Err(invalid("report contains duplicate plot identifiers"));
            }
            Ok::<_, Error>(plots)
        })?;
    }
    Ok(plots)
}

fn reserve_plot_slot<T>(values: &mut Vec<T>) -> Result<()> {
    if values.len() < values.capacity() {
        return Ok(());
    }
    let capacity = values
        .len()
        .checked_add(1)
        .and_then(usize::checked_next_power_of_two)
        .ok_or_else(|| invalid("report plot workspace capacity overflows"))?;
    // Replacement Backing Can Coexist With The Original Allocation.
    values.try_reserve_exact(capacity - values.len()).map_err(|_| invalid("report plot workspace allocation failed"))?;
    Ok(())
}

fn assemble(
    final_report: bool, captured: &CapturedSource, execution: &dokime_domain::model::execution::result::TestExecution,
    documents: BTreeMap<DocumentId, crate::document::capture::Capture>, plots: BTreeMap<String, super::value::Value>,
    (records, preview): (ReadRows<RecordEntry>, ReadRows<crate::telemetry::TelemetrySample>),
    report_defaults: crate::config::ReportDefaults,
) -> Result<CapturedExecutionReportInput> {
    let source_cycle = &captured.context.cycle.cycle;
    let mut captured_execution = execution.clone();
    captured_execution.saved = None;
    let cycle = dokime_domain::model::execution::result::CycleView {
        saved: source_cycle.saved.clone(),
        test_cycle_id: source_cycle.test_cycle_id.clone(),
        session_id: source_cycle.session_id.clone(),
        metadata: source_cycle.metadata.clone(),
        phase: source_cycle.phase,
        lifecycle: source_cycle.lifecycle,
        created_at: source_cycle.created_at.clone(),
        started_at: source_cycle.started_at.clone(),
        updated_at: source_cycle.updated_at.clone(),
        test_executions: vec![captured_execution],
        records: source_cycle.records.clone(),
        continue_automatically: source_cycle.continue_automatically,
        result: source_cycle.result.clone(),
    };
    let mut artifact_links = Vec::with_capacity(captured.outputs.len());
    for output in &captured.outputs {
        if let ExecutionOutputContent::Artifact { record, .. } = &output.content {
            let mut href = String::with_capacity("outputs/".len().saturating_add(output.relative_path.len()));
            href.push_str("outputs/");
            href.push_str(&output.relative_path);
            artifact_links.push(ReportArtifactLink {
                artifact_id: record.artifact_id.as_str().to_owned(),
                label: record.filename.clone(),
                href,
                path: output.relative_path.clone(),
                media_type: record.content_type.clone(),
                description: None,
                size_bytes: Some(output.file.length),
                group_id: None,
                group_label: None,
                scope: Some(record.scope.clone()),
            });
        }
    }
    let appearance = super::value::Value::from_serializable(&captured.context.appearance, 32)?;
    Ok(CapturedExecutionReportInput {
        report_defaults,
        final_report,
        owner_node_id: captured.owner_node_id.clone(),
        execution_id: captured.execution_id.clone(),
        execution: captured.execution.clone(),
        lifecycle_id: captured.lifecycle_id.clone(),
        evidence_through: captured.evidence_through.clone(),
        record_contracts: captured.record_contracts.clone(),
        telemetry_contracts: captured.telemetry_contracts.clone(),
        context: captured.context.clone(),
        input: CycleReportInput {
            session_id: captured.context.session_id.clone(),
            cycle,
            documents: BTreeMap::new(),
            plots: BTreeMap::new(),
            case_documents: vec![super::ReportCaseDocument {
                test_case_id: execution.test_case_id.clone(),
                test_execution_id: Some(execution.test_execution_id.clone()),
                documents,
                plots,
            }],
            records,
            telemetry: preview,
            traceability: captured.context.traceability.clone(),
            artifact_links,
            appearance: Some(appearance),
            generated_at: captured.captured_at.clone(),
        },
    })
}

fn invalid(reason: &'static str) -> Error {
    Error::Storage(crate::storage::Error::Invalid(reason))
}

/* === Tests === */
