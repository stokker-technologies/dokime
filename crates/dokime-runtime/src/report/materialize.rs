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

use harmos::storage::ReadBatch;

use super::Scope;

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

type TelemetryRows = crate::execution::evidence_rows::EvidenceRows<crate::telemetry::TelemetrySample>;
type ScannedTelemetry = (TelemetryRows, Vec<std::ops::Range<usize>>);

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn sealed_member(
    ctx: &Scope, storage: &Manager, seal: &dokime_domain::model::execution::saved::ExecutionSeal, operation: &Operation,
) -> Result<ReadBatch<CapturedExecutionReportInput>> {
    let inspection = ExecutionView::from_seal_source(storage, seal).await?;
    execution(ctx, &inspection, true, operation).await
}

pub(crate) async fn execution(
    ctx: &Scope, inspection: &ExecutionView, final_report: bool, operation: &Operation,
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
    let needs_rows = captured.context.execution_documents.values().any(|definition| {
        crate::document::binding::telemetry_needed(definition, context)
            || definition.cards.iter().any(|card| matches!(card.body, crate::document::DocumentCardBody::Plot { .. }))
    });
    let maximum = usize::try_from(captured.report_defaults.plot_points).map_err(|_| invalid("plot point policy exceeds native size"))?;
    let (telemetry, exact_telemetry) = context_telemetry(ctx, inspection.session(), captured, execution, needs_rows).await?;
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
            if sources.2 || !sources.1.is_empty() {
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
    operation.phase("Querying Report Plots");
    let plots = plots(ctx, captured, execution, &records, &exact_telemetry, &documents, operation).await?;
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

// One Saved-Evidence Pass Feeds The Preview And Every Exact Consumer.
async fn context_telemetry(
    ctx: &Scope, session: &crate::storage::Manager, captured: &CapturedSource,
    execution: &dokime_domain::model::execution::result::TestExecution, needs_rows: bool,
) -> Result<(
    ReadBatch<Vec<crate::telemetry::TelemetrySample>>,
    crate::execution::evidence_rows::EvidenceRows<crate::telemetry::TelemetrySample>,
)> {
    let scan = async {
        let mut rows = TelemetryRows::default();
        let mut spans = Vec::new();
        for versions in captured.telemetry_contracts.values() {
            for contract in versions.values() {
                let start = rows.len();
                super::query::telemetry::visit(
                    ctx,
                    session,
                    &captured.evidence_through,
                    execution.test_execution_id.as_str(),
                    contract,
                    |sample| rows.append(sample).map_err(Error::Operation),
                )
                .await?;
                spans.push(start..rows.len());
            }
        }
        Ok((rows, spans))
    };
    telemetry_pass(scan, captured.report_defaults.plot_points as usize, needs_rows).await
}

async fn telemetry_pass(
    scan: impl std::future::Future<Output = Result<ScannedTelemetry>>, maximum: usize, needs_rows: bool,
) -> Result<(ReadBatch<Vec<crate::telemetry::TelemetrySample>>, TelemetryRows)> {
    let (mut rows, spans) = scan.await?;
    let selected = spans.iter().flat_map(|span| harmos::signal::decimate_indices(span.len(), maximum).map(move |index| span.start + index));
    let mut selected = selected.peekable();
    let mut preview = Vec::new();
    for (index, sample) in rows.iter().enumerate() {
        if selected.peek() == Some(&index) {
            preview.try_reserve(1).map_err(|_| invalid("report preview allocation failed"))?;
            preview.push(sample);
            selected.next();
        }
    }
    let preview = super::query::telemetry::sort(ReadBatch::new(preview))?;
    if !needs_rows {
        rows = Default::default();
    }
    Ok((preview, rows))
}

// Plot Queries Respect The Supplied Concurrency; Results Keep Card Order.
async fn plots(
    ctx: &Scope, captured: &CapturedSource, execution: &dokime_domain::model::execution::result::TestExecution,
    records: &crate::execution::evidence_rows::EvidenceRows<RecordEntry>,
    telemetry: &crate::execution::evidence_rows::EvidenceRows<crate::telemetry::TelemetrySample>,
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

    if groups.is_empty() {
        return Ok(ReadBatch::new(BTreeMap::new()));
    }
    let input = plot_inputs(telemetry, records)?;
    let input = &input;
    let mut pending_groups = futures_util::stream::iter(groups.into_iter().map(|(plan, requests)| async move {
        let _group = operation.report_group().await;
        let _plan = plan;
        let shared = super::query::ReadCache::new();
        let results = futures_util::stream::iter(requests.into_iter().map(|(current, id, request)| {
            let shared = &shared;
            async move {
                let _permit = operation.report_work().await;
                let payload = super::query::materialize(ctx, captured, execution, request, input, telemetry, shared).await?;
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

fn plot_inputs(
    telemetry: &TelemetryRows, records: &crate::execution::evidence_rows::EvidenceRows<RecordEntry>,
) -> Result<crate::plot::spilled::Input> {
    let input = crate::plot::spilled::Input::new()?;
    for sample in telemetry.iter() {
        input.append(&sample)?;
    }
    for record in records.iter() {
        input.append_record(&record)?;
    }
    Ok(input)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Value,
        plot::{PlotPayloadContext, PlotPayloadRequest},
        telemetry::TelemetrySample,
    };
    use dokime_domain::model::{identity::Identifier, time::Timestamp};

    #[tokio::test]
    async fn report_plans_share_one_telemetry_scan() {
        let scans = std::cell::Cell::new(0);
        let timestamp = Timestamp::new("2026-10-21T00:00:00Z");
        let stream = Identifier::new("fixture.capture").unwrap();
        let scan = async {
            scans.set(scans.get() + 1);
            let mut rows = TelemetryRows::default();
            for index in 0..4096_u64 {
                let mut sample = TelemetrySample::new(
                    dokime_domain::model::identity::TelemetrySampleId::from_seed(index.to_string()),
                    stream.clone(),
                    index,
                    timestamp.clone(),
                    "fixture",
                );
                sample.values.insert("value".into(), Value::Number(index as f64));
                rows.append(&sample).map_err(Error::Operation)?;
            }
            let length = rows.len();
            Ok((rows, std::iter::once(0..length).collect()))
        };
        let (preview, exact) = telemetry_pass(scan, 32, true).await.unwrap();
        assert_eq!(preview.len(), 32);
        assert_eq!(exact.len(), 4096);
        assert_eq!(exact.retained_rows(), 0);
        assert!(exact.spooled_bytes() > 0);
        let input = plot_inputs(&exact, &Default::default()).unwrap();
        for maximum in [8, 16, 32] {
            let request = PlotPayloadRequest {
                stream: stream.to_string(),
                y_keys: vec!["value".into()],
                max_points: Some(maximum),
                plot_type: Some(crate::document::PlotType::TimeSeries),
                ..Default::default()
            };
            let context = PlotPayloadContext {
                default_max_points: 32,
                x_format: None,
                series_format: None,
                run_started_at: None,
                cycle_started_at: None,
            };
            let payload = crate::plot::spilled::project(&input, &request, timestamp.as_str(), &context).unwrap();
            assert_eq!(payload.series[0].raw_points, 4096);
            assert!(payload.series[0].points.len() <= maximum);
        }
        assert_eq!(scans.get(), 1);
    }

    #[tokio::test]
    async fn report_preview_releases_rows_without_exact_plans() {
        let timestamp = Timestamp::new("2026-10-21T00:00:00Z");
        let mut rows = TelemetryRows::default();
        for index in 0..16_u64 {
            let sample = TelemetrySample::new(
                dokime_domain::model::identity::TelemetrySampleId::from_seed(index.to_string()),
                Identifier::new("fixture.preview").unwrap(),
                index,
                timestamp.clone(),
                "fixture",
            );
            rows.append(&sample).unwrap();
        }
        let spans = std::iter::once(0..rows.len()).collect();
        let (preview, exact) = telemetry_pass(async { Ok((rows, spans)) }, 4, false).await.unwrap();
        assert_eq!(preview.len(), 4);
        assert!(exact.is_empty());
    }
}
