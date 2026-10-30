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

use harmos::storage::ReadBatch;

use super::Scope;
use crate::execution::context::captured::source::CapturedSource;

// Local Imports
use super::{CapturedExecutionReportInput, CycleReportInput, ReportArtifactLink};
use crate::storage::{
    Manager, ReadRows,
    projection::{self},
};
use crate::{
    document::capture::CardBindingResult,
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
    ctx: &Scope, storage: &Manager, seal: &dokime_domain::model::execution::saved::ExecutionSeal, operation: &Operation,
) -> Result<ReadBatch<CapturedExecutionReportInput>> {
    let inspection = ExecutionView::from_seal_source(storage, seal).await?;
    execution(ctx, &inspection, true, operation).await
}

// Presentation Reads Are Bounded By Ingestion And Indexed Previews.
pub(crate) async fn execution(
    ctx: &Scope, inspection: &ExecutionView, final_report: bool, operation: &Operation,
) -> Result<ReadBatch<CapturedExecutionReportInput>> {
    let span = tracing::info_span!("report.materialize", final_report);
    tracing::Instrument::instrument(materialize_execution(ctx, inspection, final_report, operation), span).await
}

async fn materialize_execution(
    ctx: &Scope, inspection: &ExecutionView, final_report: bool, operation: &Operation,
) -> Result<ReadBatch<CapturedExecutionReportInput>> {
    operation.phase("Materializing Evidence");
    let captured = inspection.captured_source().ok_or_else(|| invalid("report has no captured execution context"))?;
    captured.validate_header()?;
    if captured.evidence_through != *inspection.evidence_through()? {
        return Err(invalid("report capture differs from the requested durable boundary"));
    }
    let execution = inspection.execution();
    #[cfg(feature = "testing")]
    if final_report {
        crate::domains::execution::run::testing::record(&execution.test_execution_id, "report");
        crate::domains::execution::run::testing::before_report(&execution.test_execution_id).await?;
    }
    let local = &ctx.storage;
    let table_rows = captured.report_defaults.table_rows as usize;
    let mut records = crate::execution::evidence_rows::EvidenceRows::default();
    for record in local
        .report_record_rows(
            execution.test_execution_id.to_string(),
            execution.test_cycle_id.to_string(),
            captured.evidence_through.clone(),
            table_rows,
        )
        .await?
    {
        records.append(&record).map_err(Error::Operation)?;
    }
    let fields = local.report_previews(execution.test_execution_id.to_string(), captured.evidence_through.clone()).await?;
    let mut exact = local.numeric_summaries(execution.test_execution_id.to_string()).await?;
    let mut percentiles = std::collections::BTreeSet::new();
    for definition in captured.context.execution_documents.values() {
        let bodies = definition
            .cards
            .iter()
            .map(|card| &card.body)
            .chain(definition.sections.iter().flat_map(|section| section.cards.iter().map(|card| &card.body)));
        for body in bodies {
            let crate::document::DocumentCardBody::TelemetrySummary { summary } = body else { continue };
            for tile in summary.groups.iter().flat_map(|group| &group.tiles) {
                if let crate::expressions::DataSource::Telemetry { stream, field } = &tile.value.source
                    && let [crate::expressions::DataTransform::Aggregate(harmos::Aggregate::Percentile(rank))] =
                        tile.value.transforms.as_slice()
                {
                    percentiles.insert((stream.clone(), field.clone(), *rank));
                }
            }
        }
    }
    for (stream, field, rank) in percentiles {
        for summary in
            exact.iter_mut().filter(|summary| crate::plot::stream_matches_alias(&summary.stream, &stream) && summary.field == field)
        {
            let name = format!("p{rank}");
            if summary.statistics.iter().any(|(key, _)| *key == name) {
                continue;
            }
            if let Some(value) =
                local.numeric_percentile(execution.test_execution_id.to_string(), summary.stream.clone(), field.clone(), rank).await?
            {
                summary.statistics.push((name, value));
            }
        }
    }
    let mut samples = Vec::new();
    for versions in captured.telemetry_contracts.values() {
        for contract in versions.values() {
            let codec = super::query::telemetry::saved_codec(ctx, inspection.session(), &captured.evidence_through, contract).await?;
            let rows = local
                .report_sample_rows(
                    execution.test_execution_id.to_string(),
                    contract.stream.to_string(),
                    captured.evidence_through.clone(),
                    table_rows,
                )
                .await?;
            for row in rows {
                if row.record.schema != *codec.schema() {
                    continue;
                }
                samples.push(codec.decode(&row.record.payload, usize::MAX)?.sample);
            }
        }
    }
    samples.sort_by(|left, right| left.timestamp.cmp(&right.timestamp).then(left.sequence.cmp(&right.sequence)));
    let selection = crate::document::DocumentSelection { scope: execution.scope.clone() };
    let context = crate::document::input::Context::new(std::slice::from_ref(execution), &selection.scope);
    let selected_records: Vec<_> = records.iter().collect();
    let outputs: Vec<_> =
        super::previews::indices(captured.outputs.len(), table_rows).map(|index| captured.outputs[index].clone()).collect();
    let bindings = crate::document::bounded::SharedBindings::new(
        &selection,
        crate::document::input::BorrowedData {
            context,
            records: crate::document::input::Records::Native(selected_records.as_slice().into()),
            telemetry: &samples,
            artifacts: crate::document::input::Artifacts::Outputs(&outputs),
        },
        crate::document::bounded::DocumentLimits::default(),
    )?;
    let mut documents = BTreeMap::new();
    let mut plots = BTreeMap::new();
    for (id, definition) in &captured.context.execution_documents {
        let _permit = operation.report_work().await;
        let mut definition = definition.clone();
        let limit = table_rows;
        let repeats_unavailable = super::previews::bound_definition(&mut definition, limit);
        let mut document = bindings.capture(&definition)?;
        if repeats_unavailable {
            document.warnings.push(super::previews::warning(
                "History-dependent repeated sections are unavailable in bounded reports; inspect the complete execution evidence.".into(),
            ));
        }
        super::previews::summaries(&mut document, &exact, &fields, context);
        document.warnings.push(super::previews::warning(format!(
            "Tables contain at most the first and last {} rows per stream. Plots show durable extrema envelopes. Complete raw data: execution {}, evidence through {} (use execution evidence paging or cycle.save).",
            table_rows, execution.test_execution_id, captured.evidence_through.sequence)));
        for card in &document.cards {
            let CardBindingResult::Plot { descriptor } = &card.result else { continue };
            let request = crate::plot::plot_payload_request_for_descriptor(descriptor, Some(execution.scope.clone()), None, None);
            let (payload, warning) = super::previews::plot(&request, &fields, &exact, captured, execution)?;
            if let Some(warning) = warning {
                document.warnings.push(super::previews::warning(warning));
            }
            if plots.insert(descriptor.plot_id.to_string(), payload).is_some() {
                return Err(invalid("report contains duplicate plot identifiers"));
            }
        }
        documents.insert(id.clone(), document);
    }
    let preview = ReadRows::from(projection::push(ReadBatch::new(Vec::new()), ReadBatch::new(samples))?);
    let report = assemble(final_report, captured, execution, documents, plots, (records, preview), captured.report_defaults)?;
    Ok(ReadBatch::new(report))
}

/* === Private Functions === */

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
    for index in super::previews::indices(captured.outputs.len(), captured.report_defaults.table_rows as usize) {
        let output = &captured.outputs[index];
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
