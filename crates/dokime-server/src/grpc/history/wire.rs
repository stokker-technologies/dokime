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
use std::{collections::BTreeMap, num::NonZeroUsize};
// Third Party Imports
use harmos::storage::{ObjectVersionToken, StorageCursor, StoreHistory};
use tonic::Status;
// Local Imports
use super::super::error::detailed;
use super::super::{catalog, execution, value};
use super::document::wire as document_wire;
use dokime_app::definition::{MaterializedParameter, ParameterSource, TestCase};
use dokime_app::execution::result::ExecutionInvalidation;
use dokime_app::execution::{
    CycleStatus, CycleView, ExecutionSnapshot, ExecutionStatus, StepStatus, TestExecution,
};
use dokime_app::runtime::query::execution_history::{
    ExecutionAvailability, ExecutionBrowseEntry, ExecutionCheckpoint, ExecutionCheckpointContext,
    ExecutionCheckpointDocument, ExecutionCheckpointEvidence,
};
use dokime_protocol::v1;

/* === Constants === */
const MAX_CHECKPOINT_TOKEN_BYTES: usize = 4096;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn query_failure(error: dokime_app::Error) -> Status {
    if matches!(
        &error,
        dokime_app::Error::Harmos(harmos::Error::ResourceExhausted { .. })
    ) {
        return detailed(
            Status::resource_exhausted("historical response exceeds its original budget"),
            &error,
        );
    }
    detailed(
        Status::unavailable(format!("historical execution discovery failed: {error}")),
        &error,
    )
}

pub(super) fn checkpoint_failure(error: dokime_app::Error) -> Status {
    tracing::warn!(error = %error, "checkpoint read failed");
    if matches!(
        &error,
        dokime_app::Error::Harmos(harmos::Error::ResourceExhausted { .. })
    ) {
        return detailed(
            Status::resource_exhausted("checkpoint exceeds its original budget"),
            &error,
        );
    }
    detailed(
        Status::failed_precondition(format!("checkpoint cannot be read: {error}")),
        &error,
    )
}

pub(super) fn deletion_request_identifier(value: String) -> Result<String, Status> {
    if value.is_empty() || value.len() > 256 {
        return Err(Status::invalid_argument(
            "deletion request identifier must be 1 to 256 bytes",
        ));
    }
    value::text(&value)?;
    Ok(value)
}

pub(crate) fn checkpoint(
    value: Option<v1::ExecutionCheckpoint>,
) -> Result<ExecutionCheckpoint, Status> {
    let value = value.ok_or_else(|| Status::invalid_argument("checkpoint is required"))?;
    value::text(&value.execution_identifier)?;
    let execution_id = dokime_app::core::TestExecutionId::from_uuid(value.execution_identifier)
        .map_err(|_| Status::invalid_argument("invalid checkpoint execution identifier"))?;
    if value.version.len() > MAX_CHECKPOINT_TOKEN_BYTES {
        return Err(Status::resource_exhausted(
            "checkpoint version exceeds its byte limit",
        ));
    }
    let allowance =
        NonZeroUsize::new(MAX_CHECKPOINT_TOKEN_BYTES + std::mem::size_of::<ObjectVersionToken>())
            .expect("nonzero token allowance");
    let version = ObjectVersionToken::decode_bounded(&value.version, allowance)
        .map_err(|_| Status::invalid_argument("invalid checkpoint version token"))?;
    let canonical = version
        .encode_bounded(NonZeroUsize::new(MAX_CHECKPOINT_TOKEN_BYTES).unwrap())
        .map_err(|_| Status::invalid_argument("invalid checkpoint version token"))?;
    if canonical != value.version {
        return Err(Status::invalid_argument(
            "checkpoint version token is noncanonical",
        ));
    }
    let checkpoint = ExecutionCheckpoint {
        execution_id,
        position: cursor(value.position)?,
        version,
        evidence_through: cursor(value.evidence_through)?,
    };
    checkpoint
        .validate()
        .map_err(|_| Status::invalid_argument("checkpoint revision or evidence cut differs"))?;
    Ok(checkpoint)
}

pub(super) fn browse_page(
    page: &dokime_app::runtime::query::execution_history::ExecutionBrowsePage,
) -> Result<v1::ExecutionBrowsePage, Status> {
    let mut entries = exact_slots(page.entries.len())?;
    for entry in &page.entries {
        entries.push(browse_entry(entry)?);
    }
    let mut conflicts = exact_slots(page.conflicts.len())?;
    for id in &page.conflicts {
        conflicts.push(id.as_str().to_owned());
    }
    let mut issues = exact_slots(page.issues.len())?;
    for issue in &page.issues {
        issues.push(v1::HistoryIssue {
            source: issue.source.clone(),
            dataset: issue.dataset.clone(),
            message: issue.message.clone(),
        });
    }
    Ok(v1::ExecutionBrowsePage {
        entries,
        next_after_execution_id: page.next.as_ref().map(|id| id.as_str().to_owned()),
        discovery_complete: page.discovery_complete,
        conflicts,
        omitted_conflicts: page.omitted_conflicts as u32,
        issues,
        omitted_issues: page.omitted_issues as u32,
    })
}

fn exact_slots<T>(count: usize) -> Result<Vec<T>, Status> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| Status::resource_exhausted("historical wire slot allocation failed"))?;
    if values.capacity() > count {
        return Err(Status::resource_exhausted(
            "historical wire slots exceed admission",
        ));
    }
    Ok(values)
}

pub(super) fn checkpoint_wire(
    value: ExecutionCheckpoint,
) -> Result<v1::ExecutionCheckpoint, Status> {
    checkpoint_wire_ref(&value)
}

pub(super) fn checkpoint_wire_ref(
    value: &ExecutionCheckpoint,
) -> Result<v1::ExecutionCheckpoint, Status> {
    value.validate().map_err(|error| {
        detailed(
            Status::internal("invalid native checkpoint"),
            &dokime_app::Error::Operation(error.to_string()),
        )
    })?;
    let version = value
        .version
        .encode_bounded(NonZeroUsize::new(MAX_CHECKPOINT_TOKEN_BYTES).unwrap())
        .map_err(|error| {
            detailed(
                Status::resource_exhausted("checkpoint version exceeds its byte limit"),
                &error.into(),
            )
        })?;
    Ok(v1::ExecutionCheckpoint {
        execution_identifier: value.execution_id.as_str().to_owned(),
        position: Some(cursor_wire_ref(&value.position)),
        version,
        evidence_through: Some(cursor_wire_ref(&value.evidence_through)),
    })
}

fn cursor(value: Option<v1::StorageCursor>) -> Result<StorageCursor, Status> {
    let value = value.ok_or_else(|| Status::invalid_argument("checkpoint cursor is required"))?;
    let history = value
        .history
        .ok_or_else(|| Status::invalid_argument("checkpoint history is required"))?;
    value::text(&history.store)?;
    value::text(&history.epoch)?;
    let store = dokime_app::core::Identifier::new(history.store)
        .map_err(|_| Status::invalid_argument("invalid checkpoint store identifier"))?;
    let epoch = dokime_app::core::Identifier::new(history.epoch)
        .map_err(|_| Status::invalid_argument("invalid checkpoint history epoch"))?;
    Ok(StorageCursor {
        history: StoreHistory { store, epoch },
        sequence: value.sequence,
    })
}

fn cursor_wire_ref(value: &StorageCursor) -> v1::StorageCursor {
    v1::StorageCursor {
        history: Some(v1::StoreHistory {
            store: value.history.store.as_str().to_owned(),
            epoch: value.history.epoch.as_str().to_owned(),
        }),
        sequence: value.sequence,
    }
}

pub(super) fn checkpoint_document(
    value: ExecutionCheckpointDocument,
) -> Result<v1::CheckpointDocument, Status> {
    Ok(v1::CheckpointDocument {
        checkpoint: Some(checkpoint_wire(value.checkpoint)?),
        document: Some(execution_document(value.document)?),
        acquisition_finished: value.acquisition_finished,
    })
}

pub(super) fn sections(values: Vec<i32>) -> Result<Vec<v1::EvidenceSection>, Status> {
    if values.is_empty() {
        return Ok(vec![
            v1::EvidenceSection::Outputs,
            v1::EvidenceSection::Artifacts,
            v1::EvidenceSection::RecordSchemas,
            v1::EvidenceSection::TelemetryTables,
        ]);
    }
    value::collection(values.len())?;
    let mut selected = Vec::new();
    for raw in values {
        let section = v1::EvidenceSection::try_from(raw)
            .map_err(|_| Status::invalid_argument("unknown evidence section"))?;
        if section == v1::EvidenceSection::Unspecified {
            return Err(Status::invalid_argument(
                "evidence section must be explicitly selected",
            ));
        }
        if !selected.contains(&section) {
            selected.push(section);
        }
    }
    Ok(selected)
}

pub(super) fn checkpoint_outputs(
    checkpoint: &ExecutionCheckpoint,
    outputs: &[dokime_app::execution::saved::ExecutionOutput],
    sections: &[v1::EvidenceSection],
) -> Result<v1::CheckpointEvidence, Status> {
    let files = if sections.contains(&v1::EvidenceSection::Outputs) {
        collect(outputs.iter().map(|output| {
            let kind = dokime_app::ExecutionFileContentKind::from_metadata(
                &output.relative_path,
                Some(&output.file.metadata.media_type),
            );
            Ok(v1::ExecutionFileMetadata {
                relative_path: value::projection::text(&output.relative_path)
                    .map_err(construction_failure)?,
                content_type: Some(
                    value::projection::text(&output.file.metadata.media_type)
                        .map_err(construction_failure)?,
                ),
                kind: super::super::enums::execution_file_kind(&kind) as i32,
                state: match output.stream_state {
                    Some(dokime_app::execution::saved::SavedStreamState::Finalized) => {
                        v1::ExecutionFileState::Finalized
                    }
                    Some(dokime_app::execution::saved::SavedStreamState::Interrupted) => {
                        v1::ExecutionFileState::Interrupted
                    }
                    None => v1::ExecutionFileState::Streaming,
                } as i32,
                size_bytes: output.file.length,
                created_at: Some(value::timestamp(&output.created_at)?),
            })
        }))?
    } else {
        Vec::new()
    };
    let count = if sections.contains(&v1::EvidenceSection::Artifacts) {
        outputs
            .iter()
            .filter(|output| {
                matches!(
                    output.content,
                    dokime_app::execution::saved::ExecutionOutputContent::Artifact { .. }
                )
            })
            .count()
    } else {
        0
    };
    let mut artifacts = value::projection::slots(count).map_err(construction_failure)?;
    if count != 0 {
        for output in outputs {
            let dokime_app::execution::saved::ExecutionOutputContent::Artifact { record, .. } =
                &output.content
            else {
                continue;
            };
            artifacts.push(super::super::content::metadata(record)?);
        }
    }
    Ok(v1::CheckpointEvidence {
        checkpoint: Some(checkpoint_wire_ref(checkpoint)?),
        outputs: files,
        artifacts,
        record_schemas: Vec::new(),
        telemetry_tables: Vec::new(),
    })
}

pub(super) fn checkpoint_evidence(
    value: ExecutionCheckpointEvidence,
    sections: &[v1::EvidenceSection],
) -> Result<v1::CheckpointEvidence, Status> {
    let selected = |section| sections.contains(&section);
    let outputs = if selected(v1::EvidenceSection::Outputs) {
        value
            .outputs
            .into_iter()
            .map(file)
            .collect::<Result<_, _>>()?
    } else {
        Vec::new()
    };
    let artifacts = if selected(v1::EvidenceSection::Artifacts) {
        value
            .artifacts
            .iter()
            .map(super::super::content::metadata)
            .collect::<Result<_, _>>()?
    } else {
        Vec::new()
    };
    let record_schemas = if selected(v1::EvidenceSection::RecordSchemas) {
        value
            .record_contracts
            .iter()
            .map(catalog::schemas::record)
            .collect::<Result<_, _>>()?
    } else {
        Vec::new()
    };
    let telemetry_tables = if selected(v1::EvidenceSection::TelemetryTables) {
        value
            .telemetry
            .iter()
            .map(|item| {
                Ok(v1::CheckpointTelemetryTable {
                    table: item.table.clone(),
                    schema: Some(catalog::schemas::telemetry(&item.contract)?),
                })
            })
            .collect::<Result<_, Status>>()?
    } else {
        Vec::new()
    };
    Ok(v1::CheckpointEvidence {
        checkpoint: Some(checkpoint_wire(value.checkpoint)?),
        outputs,
        artifacts,
        record_schemas,
        telemetry_tables,
    })
}

pub(super) fn file_window(
    value: dokime_app::ExecutionFileContent,
) -> Result<v1::ExecutionFileWindow, Status> {
    value::text(&value.relative_path)?;
    if let Some(content_type) = &value.content_type {
        value::text(content_type)?;
    }
    Ok(v1::ExecutionFileWindow {
        relative_path: value.relative_path,
        content_type: value.content_type,
        kind: match value.content_kind {
            dokime_app::ExecutionFileContentKind::Text => v1::ExecutionFileKind::Text,
            dokime_app::ExecutionFileContentKind::Binary => v1::ExecutionFileKind::Binary,
            dokime_app::ExecutionFileContentKind::Media => v1::ExecutionFileKind::Media,
            dokime_app::ExecutionFileContentKind::Unknown => v1::ExecutionFileKind::Unknown,
        } as i32,
        state: match value.stream_state {
            dokime_app::ExecutionFileStreamState::Streaming => v1::ExecutionFileState::Streaming,
            dokime_app::ExecutionFileStreamState::Finalized => v1::ExecutionFileState::Finalized,
            dokime_app::ExecutionFileStreamState::Interrupted => {
                v1::ExecutionFileState::Interrupted
            }
        } as i32,
        offset: value.offset,
        observed_size_bytes: value.size_bytes,
        data: value.bytes,
    })
}

pub(super) fn telemetry_memory(
    row: &dokime_app::runtime::query::checkpoint_telemetry::ExecutionTelemetryRow,
) -> Result<usize, Status> {
    let sample = &row.sample;
    let mut bytes = row.epoch.as_str().len();
    for text in [
        sample.sample_id.as_str(),
        sample.stream.as_str(),
        &sample.source,
    ] {
        value::text(text)?;
        bytes = bytes.saturating_add(super::super::evidence::historical_copy_memory(text.len()));
    }
    for fields in [&sample.values, &sample.summary, &sample.metadata] {
        bytes = bytes.saturating_add(super::super::evidence::historical_object_memory(
            fields,
            &mut value::Budget::new(),
            0,
        )?);
    }
    if let Some(scope) = &sample.scope {
        bytes = bytes.saturating_add(super::super::evidence::historical_scope_memory(scope));
    }
    if bytes == usize::MAX {
        return Err(Status::resource_exhausted(
            "saved sample allocation size overflow",
        ));
    }
    Ok(bytes)
}

pub(super) fn telemetry_row(
    value: &dokime_app::runtime::query::checkpoint_telemetry::ExecutionTelemetryRow,
) -> Result<v1::SavedTelemetrySample, Status> {
    let sample = &value.sample;
    value::text(sample.stream.as_str())?;
    value::text(&sample.source)?;
    Ok(v1::SavedTelemetrySample {
        epoch: super::super::value::projection::text(value.epoch.as_str()).map_err(|source| {
            detailed(
                Status::resource_exhausted("saved sample epoch allocation failed"),
                &source.into(),
            )
        })?,
        ordinal: value.ordinal,
        sample: Some(v1::TelemetrySample {
            sample_id: sample.sample_id.to_string(),
            stream: sample.stream.to_string(),
            schema_version: sample.schema_version,
            sequence: sample.sequence,
            timestamp: Some(value::timestamp(&sample.timestamp)?),
            received_at: Some(value::timestamp(&sample.received_at)?),
            source: sample.source.clone(),
            scope: sample.scope.as_ref().map(value::scope),
            values: Some(value_object(&sample.values, &mut value::Budget::new(), 0)?),
            summary: Some(value_object(&sample.summary, &mut value::Budget::new(), 0)?),
            metadata: Some(value_object(
                &sample.metadata,
                &mut value::Budget::new(),
                0,
            )?),
        }),
    })
}

pub(super) fn checkpoint_context(
    value: ExecutionCheckpointContext,
) -> Result<v1::CheckpointContext, Status> {
    let cycle = Some(cycle_context(value.cycle)?);
    let execution_documents = collect(
        value
            .execution_documents
            .values()
            .map(|document| document_wire(document, &value.scope)),
    )?;
    let prompts = collect(
        value
            .prompts
            .iter()
            .map(super::super::prompts::historical_instance),
    )?;
    let health = collect(
        value
            .health
            .iter()
            .map(super::super::notes::historical_health),
    )?;
    let notes = collect(value.notes.iter().map(super::super::notes::historical_note))?;
    let traceability = collect(value.traceability.iter().map(traceability))?;
    Ok(v1::CheckpointContext {
        checkpoint: Some(checkpoint_wire(value.checkpoint)?),
        session_id: value.session_id.as_str().to_owned(),
        execution_id: value
            .scope
            .test_execution_id
            .as_ref()
            .map(|id| id.as_str().to_owned()),
        execution_scope: Some(
            value::projection::scope(&value.scope).map_err(construction_failure)?,
        ),
        cycle,
        execution_documents,
        prompts,
        health,
        notes,
        traceability,
        // The captured historical frame includes only a data URL for a logo;
        // the asset identity is intentionally not invented without its digest.
        appearance: Some(super::super::appearance::historical(value.appearance)?),
    })
}

pub(crate) fn traceability(
    value: &dokime_app::traceability::TraceabilitySnapshot,
) -> Result<v1::TraceabilitySnapshot, Status> {
    super::construction::traceability(value).map_err(construction_failure)?;
    let mut budget = value::Budget::new();
    Ok(v1::TraceabilitySnapshot {
        traceability_id: value.traceability_id.as_str().to_owned(),
        scope: Some(value::projection::scope(&value.scope).map_err(construction_failure)?),
        created_at: Some(value::timestamp(&value.created_at)?),
        metadata: value
            .metadata
            .as_ref()
            .map(canonical_metadata)
            .transpose()?,
        entries: collect(value.entries.iter().map(|entry| {
            Ok(v1::TraceabilityEntry {
                metadata: Some(canonical_metadata(&entry.metadata)?),
                value: Some(projected_value(&entry.value, &mut budget, 0)?),
                source: entry.source.clone(),
                unit: entry.unit.clone(),
            })
        }))?,
    })
}

pub(super) fn value_object(
    values: &BTreeMap<String, dokime_app::core::Value>,
    budget: &mut value::Budget,
    depth: usize,
) -> Result<v1::ValueObject, Status> {
    value::collection(values.len())?;
    Ok(v1::ValueObject {
        fields: values
            .iter()
            .map(|(key, item)| {
                value::text(key)?;
                Ok((key.clone(), value::encode(item, budget, depth + 1)?))
            })
            .collect::<Result<_, Status>>()?,
    })
}

/* === Private Functions === */
pub(super) fn construction_failure(error: harmos::Error) -> Status {
    match error {
        harmos::Error::ResourceExhausted { .. } => {
            Status::resource_exhausted("historical response construction exceeds its bound")
        }
        _ => Status::data_loss("historical response projection failed"),
    }
}

fn collect<T>(source: impl ExactSizeIterator<Item = Result<T, Status>>) -> Result<Vec<T>, Status> {
    let mut target = value::projection::slots(source.len()).map_err(construction_failure)?;
    for item in source {
        target.push(item?);
    }
    Ok(target)
}

fn canonical_metadata(source: &dokime_app::core::Metadata) -> Result<v1::Metadata, Status> {
    value::projection::metadata_count(source).map_err(construction_failure)?;
    value::projection::metadata(source).map_err(construction_failure)
}

fn projected_value(
    source: &dokime_app::core::Value,
    budget: &mut value::Budget,
    depth: usize,
) -> Result<v1::Value, Status> {
    value::projection::count(source, &mut value::Budget::new(), depth)
        .map_err(construction_failure)?;
    value::projection::project(source, budget, depth).map_err(construction_failure)
}

fn browse_entry(value: &ExecutionBrowseEntry) -> Result<v1::ExecutionBrowseEntry, Status> {
    Ok(v1::ExecutionBrowseEntry {
        metadata: Some(metadata(&value.metadata)?),
        checkpoint: value
            .checkpoint
            .as_ref()
            .map(checkpoint_wire_ref)
            .transpose()?,
        availability: match value.availability {
            ExecutionAvailability::Live => v1::ExecutionAvailability::Live,
            ExecutionAvailability::Stored => v1::ExecutionAvailability::Stored,
            ExecutionAvailability::Interrupted => v1::ExecutionAvailability::Interrupted,
            ExecutionAvailability::Uncheckpointed => v1::ExecutionAvailability::Uncheckpointed,
        } as i32,
    })
}

fn metadata(
    value: &dokime_app::execution::storage::ExecutionMetadata,
) -> Result<v1::HistoricalExecutionMetadata, Status> {
    Ok(v1::HistoricalExecutionMetadata {
        execution_id: value.execution_id.as_str().to_owned(),
        cycle_id: Some(value.cycle_id.as_str().to_owned()),
        case_id: value.case_id.as_str().to_owned(),
        title: value.title.clone(),
        status: execution_status(value.status.clone()) as i32,
        started_at: value
            .started_at
            .as_ref()
            .map(value::timestamp)
            .transpose()?,
        finished_at: value
            .finished_at
            .as_ref()
            .map(value::timestamp)
            .transpose()?,
        result: value.result.as_ref().map(execution::result_summary),
        template_id: value.template_id.clone(),
        suite_label: value.suite_label.clone(),
    })
}

fn execution_document(
    value: dokime_app::execution::storage::ExecutionDocument,
) -> Result<v1::HistoricalExecutionDocument, Status> {
    Ok(v1::HistoricalExecutionDocument {
        cycle: Some(v1::HistoricalExecutionCycle {
            cycle_id: value.cycle.cycle_id.as_str().to_owned(),
            metadata: Some(canonical_metadata(&value.cycle.metadata)?),
        }),
        execution: Some(test_execution(value.execution)?),
    })
}

fn test_execution(value: TestExecution) -> Result<v1::HistoricalTestExecution, Status> {
    let snapshot = execution_snapshot(value.snapshot, &value.scope)?;
    let output_parameters = materialized_parameters(&value.output_parameters)?;
    let documents = collect(
        value
            .documents
            .values()
            .map(|document| document_wire(document, &value.scope)),
    )?;
    let health = collect(
        value
            .records
            .health()
            .iter()
            .map(super::super::notes::historical_health),
    )?;
    let notes = collect(
        value
            .records
            .notes()
            .iter()
            .map(super::super::notes::historical_note),
    )?;
    let steps = collect(value.steps.iter().map(step))?;
    Ok(v1::HistoricalTestExecution {
        execution_id: value.test_execution_id.as_str().to_owned(),
        mode: match value.mode {
            dokime_app::execution::ExecutionMode::Automatic => v1::ExecutionMode::Automatic as i32,
            dokime_app::execution::ExecutionMode::Interactive => {
                v1::ExecutionMode::Interactive as i32
            }
        },
        case_id: value.test_case_id.as_str().to_owned(),
        suite: value.suite.as_ref().map(canonical_metadata).transpose()?,
        status: execution_status(value.status) as i32,
        scope: Some(value::projection::scope(&value.scope).map_err(construction_failure)?),
        snapshot: Some(snapshot),
        output_parameters,
        documents,
        health,
        notes,
        started_at: value
            .started_at
            .as_ref()
            .map(value::timestamp)
            .transpose()?,
        finished_at: value
            .finished_at
            .as_ref()
            .map(value::timestamp)
            .transpose()?,
        steps,
        active_step_attempt: value
            .active_step_attempt
            .as_ref()
            .map(|start| {
                Ok::<_, Status>(v1::HistoricalStepAttemptStart {
                    attempt_id: start.attempt_id.as_str().to_owned(),
                    step_id: start.step_id.as_str().to_owned(),
                    started_at: Some(value::timestamp(&start.started_at)?),
                })
            })
            .transpose()?,
        step_attempts: collect(value.step_attempts.iter().map(step_attempt))?,
        result: value.result.as_ref().map(execution::result_summary),
        invalidation: value.invalidation.as_ref().map(invalidation).transpose()?,
    })
}

fn step_attempt(
    attempt: &dokime_app::execution::TestStepAttempt,
) -> Result<v1::HistoricalStepAttempt, Status> {
    let detail = attempt
        .detail
        .as_ref()
        .map(|detail| {
            // Diagnostics Use The Preflighted Graph And Response Byte Budgets.
            // A Backtrace Is Not A Short Authored Metadata Field.
            if detail.chain.len() > value::MAX_COLLECTION
                || detail.span_trace.len() > value::MAX_COLLECTION
            {
                return Err(Status::resource_exhausted(
                    "step diagnostic exceeds history bounds",
                ));
            }
            Ok(v1::Diagnostic {
                message: detail.message.clone(),
                chain: detail.chain.clone(),
                location: detail.location.clone(),
                span_trace: detail.span_trace.clone(),
                backtrace: detail.backtrace.clone(),
            })
        })
        .transpose()?;
    Ok(v1::HistoricalStepAttempt {
        attempt_id: attempt.attempt_id.as_str().to_owned(),
        step_id: attempt.step_id.as_str().to_owned(),
        started_at: Some(value::timestamp(&attempt.started_at)?),
        finished_at: attempt
            .finished_at
            .as_ref()
            .map(value::timestamp)
            .transpose()?,
        termination: match attempt.termination {
            dokime_app::execution::TerminationReason::Completed => v1::StepTermination::Completed,
            dokime_app::execution::TerminationReason::Failed => v1::StepTermination::Failed,
            dokime_app::execution::TerminationReason::Cancelled => v1::StepTermination::Cancelled,
            dokime_app::execution::TerminationReason::Interrupted => {
                v1::StepTermination::Interrupted
            }
        } as i32,
        detail,
    })
}

fn execution_snapshot(
    value: ExecutionSnapshot,
    scope: &dokime_app::core::RecordScope,
) -> Result<v1::HistoricalExecutionSnapshot, Status> {
    let mut budget = value::Budget::new();
    let parameter_definitions = catalog::parameters(value.parameter_definitions(), &mut budget)?;
    let parameters = materialized_parameters(value.parameters())?;
    let documents = collect(
        value
            .documents()
            .values()
            .map(|document| document_wire(document, scope)),
    )?;
    if let Some(version) = value.host_version() {
        value::text(version)?;
    }
    Ok(v1::HistoricalExecutionSnapshot {
        case_definition: Some(case_definition(value.case())?),
        host_version: value.host_version().map(str::to_owned),
        suite: value.suite().map(canonical_metadata).transpose()?,
        template_id: value.template_id().map(|id| id.as_str().to_owned()),
        template: value
            .template()
            .map(|template| canonical_metadata(&template.metadata))
            .transpose()?,
        plugin: value.plugin().map(value::execution_plugin).transpose()?,
        parameter_definitions,
        parameters,
        documents,
    })
}

fn case_definition(case: &TestCase) -> Result<v1::HistoricalCaseDefinition, Status> {
    value::text(&case.suite_id)?;
    value::collection(case.parameters.len())?;
    let mut budget = value::Budget::new();
    let parameters = collect(case.parameters.iter().map(|(name, parameter)| {
        value::text(name)?;
        Ok(v1::CaseParameter {
            name: name.clone(),
            value: Some(projected_value(parameter, &mut budget, 0)?),
        })
    }))?;
    Ok(v1::HistoricalCaseDefinition {
        suite_id: case.suite_id.clone(),
        case_id: case.test_case_id.as_str().to_owned(),
        template_id: case.test_case_template_id.as_str().to_owned(),
        metadata: Some(canonical_metadata(&case.metadata)?),
        suite: case.suite.as_ref().map(canonical_metadata).transpose()?,
        parameters,
        generation_index: case.generation_index,
    })
}

fn invalidation(value: &ExecutionInvalidation) -> Result<v1::ExecutionInvalidation, Status> {
    Ok(v1::ExecutionInvalidation {
        previous: Some(value::execution_plugin(&value.previous)?),
        current: value
            .current
            .as_ref()
            .map(value::execution_plugin)
            .transpose()?,
    })
}

fn materialized_parameters(
    values: &BTreeMap<String, MaterializedParameter>,
) -> Result<Vec<v1::MaterializedParameter>, Status> {
    let mut budget = value::Budget::new();
    collect(values.iter().map(|(name, parameter)| {
        value::text(name)?;
        let unit = parameter
            .unit
            .as_ref()
            .map(super::super::catalog::unit)
            .transpose()?;
        Ok(v1::MaterializedParameter {
            name: name.clone(),
            metadata: Some(canonical_metadata(&parameter.metadata)?),
            group_id: parameter.group_id.as_ref().map(|id| id.as_str().to_owned()),
            source: parameter_source(&parameter.source) as i32,
            value: Some(projected_value(&parameter.value, &mut budget, 0)?),
            unit,
        })
    }))
}

fn parameter_source(value: &ParameterSource) -> v1::ParameterSource {
    match value {
        ParameterSource::Definition => v1::ParameterSource::Definition,
        ParameterSource::Traceability => v1::ParameterSource::Traceability,
        ParameterSource::Environment => v1::ParameterSource::Environment,
        ParameterSource::Resource => v1::ParameterSource::Resource,
        ParameterSource::Operator => v1::ParameterSource::Operator,
        ParameterSource::Runtime => v1::ParameterSource::Runtime,
        ParameterSource::Telemetry => v1::ParameterSource::Telemetry,
        ParameterSource::Record => v1::ParameterSource::Record,
        ParameterSource::Output => v1::ParameterSource::Output,
    }
}

fn step(value: &dokime_app::execution::StepRecord) -> Result<v1::HistoricalStep, Status> {
    let measurements = collect(value.measurements.iter().enumerate().map(
        |(ordinal, measurement)| {
            Ok(v1::StepMeasurement {
                metadata: Some(canonical_metadata(&measurement.metadata)?),
                value: Some(projected_value(
                    &measurement.value,
                    &mut value::Budget::new(),
                    0,
                )?),
                unit: measurement
                    .unit
                    .as_ref()
                    .map(super::super::catalog::unit)
                    .transpose()?,
                ordinal: ordinal as u64,
            })
        },
    ))?;
    let criteria = collect(
        value
            .criteria
            .iter()
            .enumerate()
            .map(|(ordinal, criterion)| {
                Ok(v1::StepCriterion {
                    criterion_id: criterion.criterion_id.as_str().to_owned(),
                    metadata: Some(canonical_metadata(&criterion.metadata)?),
                    status: match criterion.status {
                        dokime_app::execution::CriterionStatus::Pending => {
                            v1::CriterionStatus::Pending
                        }
                        dokime_app::execution::CriterionStatus::Passed => {
                            v1::CriterionStatus::Passed
                        }
                        dokime_app::execution::CriterionStatus::Failed => {
                            v1::CriterionStatus::Failed
                        }
                        dokime_app::execution::CriterionStatus::Indeterminate => {
                            v1::CriterionStatus::Indeterminate
                        }
                        dokime_app::execution::CriterionStatus::Skipped => {
                            v1::CriterionStatus::Skipped
                        }
                    } as i32,
                    observed: criterion
                        .observed
                        .as_ref()
                        .map(|item| projected_value(item, &mut value::Budget::new(), 0))
                        .transpose()?,
                    message: criterion.message.clone(),
                    ordinal: ordinal as u64,
                })
            }),
    )?;
    Ok(v1::HistoricalStep {
        step_id: value.step_id.as_str().to_owned(),
        metadata: Some(canonical_metadata(&value.metadata)?),
        status: match value.status {
            StepStatus::Pending => v1::StepStatus::Pending,
            StepStatus::Running => v1::StepStatus::Running,
            StepStatus::Passed => v1::StepStatus::Passed,
            StepStatus::Failed => v1::StepStatus::Failed,
            StepStatus::Skipped => v1::StepStatus::Skipped,
            StepStatus::Cancelled => v1::StepStatus::Cancelled,
        } as i32,
        started_at: value
            .started_at
            .as_ref()
            .map(value::timestamp)
            .transpose()?,
        finished_at: value
            .finished_at
            .as_ref()
            .map(value::timestamp)
            .transpose()?,
        measurements,
        criteria,
    })
}

fn file(value: dokime_app::ExecutionFileEntry) -> Result<v1::ExecutionFileMetadata, Status> {
    value::text(&value.relative_path)?;
    if let Some(content_type) = &value.content_type {
        value::text(content_type)?;
    }
    Ok(v1::ExecutionFileMetadata {
        relative_path: value.relative_path,
        content_type: value.content_type,
        kind: match value.content_kind {
            dokime_app::ExecutionFileContentKind::Text => v1::ExecutionFileKind::Text,
            dokime_app::ExecutionFileContentKind::Binary => v1::ExecutionFileKind::Binary,
            dokime_app::ExecutionFileContentKind::Media => v1::ExecutionFileKind::Media,
            dokime_app::ExecutionFileContentKind::Unknown => v1::ExecutionFileKind::Unknown,
        } as i32,
        state: match value.stream_state {
            dokime_app::ExecutionFileStreamState::Streaming => v1::ExecutionFileState::Streaming,
            dokime_app::ExecutionFileStreamState::Finalized => v1::ExecutionFileState::Finalized,
            dokime_app::ExecutionFileStreamState::Interrupted => {
                v1::ExecutionFileState::Interrupted
            }
        } as i32,
        size_bytes: value.size_bytes,
        created_at: Some(value::timestamp(&value.updated_at)?),
    })
}

fn cycle_context(
    value: dokime_app::runtime::ExecutionCycleContext,
) -> Result<v1::HistoricalCycleContext, Status> {
    let cycle = cycle(value.cycle, &value.cases)?;
    Ok(v1::HistoricalCycleContext {
        cycle: Some(cycle),
        execution_order: collect(
            value
                .execution_order
                .into_iter()
                .map(|id| Ok(id.as_str().to_owned())),
        )?,
    })
}

fn cycle(
    value: CycleView,
    cases: &[dokime_app::definition::TestCase],
) -> Result<v1::HistoricalCycle, Status> {
    let cases = collect(
        cases
            .iter()
            .enumerate()
            .map(|(ordinal, case)| execution::historical_case_summary(&value, case, ordinal)),
    )?;
    Ok(v1::HistoricalCycle {
        cycle_id: value.test_cycle_id.as_str().to_owned(),
        metadata: Some(canonical_metadata(&value.metadata)?),
        lifecycle: cycle_status(value.lifecycle) as i32,
        created_at: Some(value::timestamp(&value.created_at)?),
        started_at: value
            .started_at
            .as_ref()
            .map(value::timestamp)
            .transpose()?,
        updated_at: Some(value::timestamp(&value.updated_at)?),
        cases,
        result: value.result.as_ref().map(execution::result_summary),
    })
}

fn execution_status(value: ExecutionStatus) -> v1::ExecutionStatus {
    match value {
        ExecutionStatus::Invalidated => v1::ExecutionStatus::Invalidated,
        ExecutionStatus::Skipped => v1::ExecutionStatus::Skipped,
        ExecutionStatus::Pending => v1::ExecutionStatus::Pending,
        ExecutionStatus::Running => v1::ExecutionStatus::Running,
        ExecutionStatus::Passed => v1::ExecutionStatus::Passed,
        ExecutionStatus::Failed => v1::ExecutionStatus::Failed,
        ExecutionStatus::Blocked => v1::ExecutionStatus::Blocked,
        ExecutionStatus::Cancelled => v1::ExecutionStatus::Cancelled,
        ExecutionStatus::Interrupted => v1::ExecutionStatus::Interrupted,
    }
}

fn cycle_status(value: CycleStatus) -> v1::CycleStatus {
    match value {
        CycleStatus::Previewed => v1::CycleStatus::Previewed,
        CycleStatus::Ready => v1::CycleStatus::Ready,
        CycleStatus::Running => v1::CycleStatus::Running,
        CycleStatus::Paused => v1::CycleStatus::Paused,
        CycleStatus::Stopping => v1::CycleStatus::Stopping,
        CycleStatus::Completed => v1::CycleStatus::Completed,
        CycleStatus::Failed => v1::CycleStatus::Failed,
        CycleStatus::Blocked => v1::CycleStatus::Blocked,
        CycleStatus::Stopped => v1::CycleStatus::Stopped,
        CycleStatus::Cleared => v1::CycleStatus::Cleared,
    }
}

/* === Tests === */
