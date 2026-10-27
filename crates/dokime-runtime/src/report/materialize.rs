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

use futures_util::StreamExt;

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports

use harmos::{
    operations::Context,
    storage::ReadBatch,
    storage::query::transform::{
        Transform,
        downsample::{Algorithm, Downsample},
    },
};

// Local Imports
use super::{CapturedExecutionReportInput, CycleReportInput, ReportArtifactLink};
use crate::storage::{
    Manager, ReadRows,
    projection::{self},
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
    let records = context_records(inspection).await?;
    let selection = harmos::storage::ReadBatch::new(())
        .try_map(|_| Ok::<_, Error>(crate::document::DocumentSelection { scope: execution.scope.clone() }))?;
    let context = crate::document::input::Context::new(std::slice::from_ref(execution), &selection.scope);
    let needs_rows =
        captured.context.execution_documents.values().any(|definition| crate::document::binding::telemetry_needed(definition, context));
    let telemetry = context_telemetry(ctx, inspection.session(), captured, execution, false).await?;
    let maximum = usize::try_from(captured.report_defaults.plot_points).map_err(|_| invalid("plot point policy exceeds native size"))?;
    let mut exact_telemetry = crate::execution::evidence_rows::EvidenceRows::default();
    if needs_rows {
        for versions in captured.telemetry_contracts.values() {
            for contract in versions.values() {
                super::query::telemetry::spool(
                    ctx,
                    inspection.session(),
                    &captured.evidence_through,
                    execution.test_execution_id.as_str(),
                    contract,
                    &mut exact_telemetry,
                )
                .await?;
            }
        }
    }
    let limits = crate::document::bounded::DocumentLimits {
        source_bytes: 64 * 1024 * 1024,
        materialized_bytes: 64 * 1024 * 1024,
        ..crate::document::bounded::DocumentLimits::default()
    };
    let mut documents = ReadBatch::new(BTreeMap::<DocumentId, crate::document::capture::Capture>::new());
    let mut cached = None;
    for (id, definition) in &captured.context.execution_documents {
        let sources = crate::document::binding::selected_sources(definition, context);
        if cached.as_ref().is_none_or(|(previous, _, _, _, _)| previous != &sources) {
            let mut selected_records = Vec::new();
            let mut selected_samples = Vec::new();
            let mut bytes = 0usize;
            for record in records.iter().filter(|record| sources.0.iter().any(|kind| kind == record.record_type.as_str())) {
                let size = crate::document::bounded::document_size(&record, limits.source_bytes, limits.depth)
                    .map_err(|_| invalid("document record exceeds its 64 MiB input limit"))?;
                bytes = bytes
                    .checked_add(size)
                    .filter(|bytes| *bytes <= limits.source_bytes)
                    .ok_or_else(|| invalid("selected document data exceeds its 64 MiB input limit"))?;
                selected_records.try_reserve(1).map_err(|_| invalid("document record workspace allocation failed"))?;
                selected_records.push(record);
            }
            for sample in
                exact_telemetry.iter().filter(|sample| sources.2 || sources.1.iter().any(|stream| stream == sample.stream.as_str()))
            {
                let size = crate::document::bounded::document_size(&sample, limits.source_bytes, limits.depth)
                    .map_err(|_| invalid("document sample exceeds its 64 MiB input limit"))?;
                bytes = bytes
                    .checked_add(size)
                    .filter(|bytes| *bytes <= limits.source_bytes)
                    .ok_or_else(|| invalid("selected document data exceeds its 64 MiB input limit"))?;
                selected_samples.try_reserve(1).map_err(|_| invalid("document telemetry workspace allocation failed"))?;
                selected_samples.push(sample);
            }
            selected_samples.sort_by(|left, right| left.timestamp.cmp(&right.timestamp).then(left.sequence.cmp(&right.sequence)));
            let data = crate::document::input::BorrowedData {
                context,
                records: crate::document::input::Records::Native(selected_records.as_slice().into()),
                telemetry: &selected_samples,
                artifacts: crate::document::input::Artifacts::Outputs(&captured.outputs),
            };
            let validated = crate::document::bounded::SharedBindings::new(&selection, data, limits)?.accounting();
            cached = Some((sources, selected_records, selected_samples, validated.0, validated.1));
        }
        let (_, selected_records, selected_samples, bytes, rows) = cached.as_ref().expect("document selection is loaded");
        let bindings = crate::document::bounded::SharedBindings::validated(
            &selection,
            crate::document::input::BorrowedData {
                context,
                records: crate::document::input::Records::Native(selected_records.as_slice().into()),
                telemetry: selected_samples,
                artifacts: crate::document::input::Artifacts::Outputs(&captured.outputs),
            },
            limits,
            *bytes,
            *rows,
        );
        let _permit = operation.report_work().await;
        let document = bindings.capture(definition)?;
        documents = documents.try_map(|mut documents| {
            documents.insert(id.clone(), document);
            Ok::<_, Error>(documents)
        })?;
    }
    drop(cached);
    drop(exact_telemetry);
    operation.phase("Querying Report Plots");
    let plots = plots(ctx, inspection.session(), captured, execution, &records, &documents, operation).await?;
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

async fn context_records(inspection: &ExecutionView) -> Result<crate::execution::evidence_rows::EvidenceRows<RecordEntry>> {
    let mut records = crate::execution::evidence_rows::EvidenceRows::default();
    let mut reader = inspection.context_records().await?;
    while let Some(rows) = reader.next().await? {
        for row in rows.iter() {
            records.append(&row.record).map_err(Error::Operation)?;
        }
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
    execution: &dokime_domain::model::execution::result::TestExecution,
    records: &crate::execution::evidence_rows::EvidenceRows<RecordEntry>,
    documents: &BTreeMap<DocumentId, crate::document::capture::Capture>, operation: &Operation,
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

    let mut pending_groups = futures_util::stream::iter(groups.into_iter().map(|(plan, requests)| async move {
        let _group = operation.report_group().await;
        let _plan = plan;
        let shared = super::query::ReadCache::new();
        let mut kinds = Vec::new();
        for (_, _, request) in &requests {
            kinds.extend(crate::runtime::query::plots::request_record_types(request));
        }
        kinds.sort();
        kinds.dedup();
        let mut selected_records = Vec::new();
        let mut bytes = 0usize;
        for record in records.iter().filter(|record| kinds.contains(&record.record_type)) {
            bytes = bytes
                .checked_add(
                    crate::document::bounded::document_size(&record, 64 * 1024 * 1024, 32)
                        .map_err(|_| invalid("plot record exceeds bounded workspace"))?,
                )
                .filter(|bytes| *bytes <= 64 * 1024 * 1024)
                .ok_or_else(|| invalid("plot record selection exceeds bounded workspace"))?;
            selected_records.try_reserve(1).map_err(|_| invalid("plot record workspace allocation failed"))?;
            selected_records.push(record);
        }
        let records: crate::records::view::View<'_> = selected_records.as_slice().into();
        let results = futures_util::stream::iter(requests.into_iter().map(|(current, id, request)| {
            let shared = &shared;
            async move {
                let _permit = operation.report_work().await;
                let payload =
                    super::query::materialize(super::query::ReadContext { ctx, session }, captured, execution, records, request, shared)
                        .await?;
                Ok::<_, Error>((current, payload.try_map(|payload| Ok::<_, Error>((id, payload)))?))
            }
        }))
        .buffer_unordered(8)
        .collect::<Vec<_>>()
        .await;
        results.into_iter().collect::<Result<Vec<_>>>()
    }))
    .buffer_unordered(4);
    while let Some(group) = pending_groups.next().await {
        completed.extend(group?);
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
    (records, preview): (crate::execution::evidence_rows::EvidenceRows<RecordEntry>, ReadRows<crate::telemetry::TelemetrySample>),
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
