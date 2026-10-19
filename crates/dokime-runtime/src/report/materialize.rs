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
use std::{collections::BTreeMap, sync::Arc};

// Third Party Imports
use futures_util::{StreamExt, stream::FuturesUnordered};
use harmos::{
    operations::Context,
    storage::query::transform::{
        Transform,
        downsample::{Algorithm, Downsample},
    },
};

// Local Imports
use super::{CapturedExecutionReportInput, CycleReportInput, ReportArtifactLink};
use crate::storage::Manager;
use crate::{
    document::capture::CardBindingResult,
    execution::context::captured::source::CapturedSource,
    runtime::Error,
    runtime::Result,
    runtime::{progress::Operation, storage::execution::ExecutionView},
};
use dokime_domain::model::execution::saved::ExecutionOutputContent;
use dokime_domain::model::identity::DocumentId;

/* === Constants === */

const ACTIVE_PLOT_PLANS: usize = 4;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn sealed_member(
    ctx: &Context<'_>, storage: &Manager, settings: &crate::config::StorageSettings,
    seal: &dokime_domain::model::execution::saved::ExecutionSeal, operation: &Operation,
) -> Result<CapturedExecutionReportInput> {
    let mut source = crate::execution::context::captured::read::read_file(storage, &seal.source.context).await?;
    source.validate_header()?;
    if source.owner_node_id != seal.source.owner_node_id
        || source.execution_id != seal.source.execution.execution_id
        || source.evidence_through != seal.source.evidence_through
    {
        return Err(invalid("cycle member source differs from its execution seal"));
    }
    if let Some(anchor) = &source.execution
        && !seal.source.execution.position.covers(&anchor.position)?
    {
        return Err(invalid("cycle member source follows its published revision"));
    }
    source.outputs = seal.source.outputs.clone();
    let inspection = ExecutionView::captured(storage, Arc::new(source), &seal.source.owner_node_id).await?;
    execution(ctx, settings, &inspection, true, operation).await
}

pub(crate) async fn execution(
    ctx: &Context<'_>, settings: &crate::config::StorageSettings, inspection: &ExecutionView, final_report: bool, operation: &Operation,
) -> Result<CapturedExecutionReportInput> {
    operation.phase("Materializing Evidence");
    let captured = inspection.captured_source().ok_or_else(|| invalid("report has no captured execution context"))?;
    captured.validate_header()?;
    if captured.evidence_through != *inspection.evidence_through()? {
        return Err(invalid("report capture differs from the requested durable boundary"));
    }
    let execution = inspection.execution();
    let records = context_records(inspection).await?;
    let selection = crate::document::DocumentSelection { scope: execution.scope.clone() };
    let context = crate::document::input::Context::new(std::slice::from_ref(execution), &selection.scope);
    let needs_rows =
        captured.context.execution_documents.values().any(|definition| crate::document::binding::telemetry_needed(definition, context));
    let telemetry = context_telemetry(ctx, settings, captured, execution, needs_rows).await?;
    let preview = super::render::cycle::selection(telemetry.len()).into_iter().map(|index| telemetry[index].clone()).collect::<Vec<_>>();
    let limit = settings.budget.read_memory_bytes.get();
    let mut documents = BTreeMap::<DocumentId, crate::document::capture::Capture>::new();
    for (id, definition) in &captured.context.execution_documents {
        let input = crate::document::input::BorrowedInput {
            definition,
            selection: &selection,
            data: crate::document::input::BorrowedData {
                context,
                records: crate::document::input::Records::Native(&records),
                telemetry: if needs_rows { &telemetry } else { &[] },
                artifacts: crate::document::input::Artifacts::Outputs(&captured.outputs),
            },
        };
        let document = crate::document::bounded::capture_document_borrowed(
            input,
            crate::document::bounded::DocumentLimits { source_bytes: limit, materialized_bytes: limit, ..Default::default() },
        )?;
        documents.insert(id.clone(), document);
    }
    operation.phase("Querying Report Plots");
    let plots = plots(ctx, settings, captured, execution, &records, &documents).await?;
    operation.phase("Assembling Report");
    assemble(final_report, captured, execution, documents, plots, records, preview)
}

/* === Private Functions === */

async fn context_records(inspection: &ExecutionView) -> Result<Vec<dokime_domain::model::records::evidence::RecordEntry>> {
    let mut records = Vec::new();
    let mut reader = inspection.context_records().await?;
    while let Some(rows) = reader.next().await? {
        records.extend(rows.into_iter().map(|row| row.record));
    }
    Ok(records)
}

// Display Telemetry: Every Row When A Document Binds Them, Else A Downsampled Preview.
async fn context_telemetry(
    ctx: &Context<'_>, settings: &crate::config::StorageSettings, captured: &CapturedSource,
    execution: &dokime_domain::model::execution::result::TestExecution, needs_rows: bool,
) -> Result<Vec<crate::telemetry::TelemetrySample>> {
    let mut telemetry = Vec::new();
    for versions in captured.telemetry_contracts.values() {
        for contract in versions.values() {
            let transforms = if needs_rows {
                vec![]
            } else {
                vec![Transform::Downsample(Downsample {
                    max_rows: crate::document::DEFAULT_PLOT_MAX_POINTS,
                    algorithm: Algorithm::Decimate,
                    value: None,
                    axis: None,
                })]
            };
            telemetry.extend(
                super::query::telemetry::read(
                    ctx,
                    settings,
                    &captured.evidence_through,
                    execution.test_execution_id.as_str(),
                    contract,
                    transforms,
                )
                .await?,
            );
        }
    }
    telemetry.sort_by(|left, right| left.timestamp.cmp(&right.timestamp).then(left.sequence.cmp(&right.sequence)));
    Ok(telemetry)
}

// At Most `ACTIVE_PLOT_PLANS` Plot Queries Run At Once; Results Keep Card Order.
async fn plots(
    ctx: &Context<'_>, settings: &crate::config::StorageSettings, captured: &CapturedSource,
    execution: &dokime_domain::model::execution::result::TestExecution, records: &[dokime_domain::model::records::evidence::RecordEntry],
    documents: &BTreeMap<DocumentId, crate::document::capture::Capture>,
) -> Result<BTreeMap<String, super::value::Value>> {
    let mut pending = FuturesUnordered::new();
    let mut completed = Vec::new();
    let mut index = 0;
    for document in documents.values() {
        for card in &document.cards {
            let CardBindingResult::Plot { descriptor } = &card.result else {
                continue;
            };
            let request = crate::plot::plot_payload_request_for_descriptor(descriptor, Some(execution.scope.clone()), None, None, None);
            let current = index;
            index += 1;
            pending.push(async move {
                let payload = super::query::materialize(ctx, settings, captured, execution, records, request).await?;
                Ok::<_, Error>((current, descriptor.plot_id.to_string(), payload))
            });
            if pending.len() == ACTIVE_PLOT_PLANS
                && let Some(plot) = pending.next().await
            {
                completed.push(plot?);
            }
        }
    }
    while let Some(plot) = pending.next().await {
        completed.push(plot?);
    }
    drop(pending);
    completed.sort_by_key(|(index, _, _)| *index);
    let mut plots = BTreeMap::new();
    for (_, id, plot) in completed {
        if plots.insert(id, plot).is_some() {
            return Err(invalid("report contains duplicate plot identifiers"));
        }
    }
    Ok(plots)
}

fn assemble(
    final_report: bool, captured: &CapturedSource, execution: &dokime_domain::model::execution::result::TestExecution,
    documents: BTreeMap<DocumentId, crate::document::capture::Capture>, plots: BTreeMap<String, super::value::Value>,
    records: Vec<dokime_domain::model::records::evidence::RecordEntry>, preview: Vec<crate::telemetry::TelemetrySample>,
) -> Result<CapturedExecutionReportInput> {
    let mut cycle = captured.context.cycle.cycle.clone();
    let mut captured_execution = execution.clone();
    captured_execution.saved = None;
    cycle.test_executions = vec![captured_execution];
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
    let appearance = super::value::Value::from_serializable(&captured.context.appearance)?;
    Ok(CapturedExecutionReportInput {
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
                documents: documents.into_iter().collect(),
                plots: plots.into_iter().collect(),
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
