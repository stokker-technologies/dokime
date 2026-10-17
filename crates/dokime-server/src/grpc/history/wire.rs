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
use std::num::NonZeroUsize;
// Third Party Imports
use harmos::storage::{ObjectVersionToken, StorageCursor, StoreHistory};
use tonic::Status;
// Local Imports
use super::super::error::detailed;
use super::super::{execution, value};
use dokime_runtime::execution::ResultStatus;
use dokime_runtime::runtime::query::execution_history::{
    ExecutionAvailability, ExecutionBrowseCycle, ExecutionBrowseEntry, ExecutionBrowseLifecycle,
    ExecutionCheckpoint,
};
use dokime_domain::operations::checkpoint as checkpoint_wire;
use dokime_domain::operations::execution as execution_wire;
use dokime_domain::operations::history as history_wire;

/* === Constants === */
const MAX_CHECKPOINT_TOKEN_BYTES: usize = 4096;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn query_failure(error: dokime_runtime::runtime::Error) -> Status {
    if matches!(
        &error,
        dokime_runtime::runtime::Error::Harmos(harmos::Error::ResourceExhausted { .. })
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

pub(super) fn checkpoint_failure(error: dokime_runtime::runtime::Error) -> Status {
    tracing::warn!(error = %error, "checkpoint read failed");
    if matches!(
        &error,
        dokime_runtime::runtime::Error::Harmos(harmos::Error::ResourceExhausted { .. })
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
    value: Option<checkpoint_wire::ExecutionCheckpoint>,
) -> Result<ExecutionCheckpoint, Status> {
    let value = value.ok_or_else(|| Status::invalid_argument("checkpoint is required"))?;
    value::text(&value.execution_identifier)?;
    let execution_id = dokime_domain::model::identity::TestExecutionId::from_uuid(value.execution_identifier)
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
    page: &dokime_runtime::runtime::query::execution_history::ExecutionBrowsePage,
) -> Result<history_wire::ExecutionBrowsePage, Status> {
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
        issues.push(history_wire::HistoryIssue {
            source: issue.source.clone(),
            dataset: issue.dataset.clone(),
            message: issue.message.clone(),
        });
    }
    let mut cycles = exact_slots(page.cycles.len())?;
    for cycle in &page.cycles {
        cycles.push(browse_cycle(cycle)?);
    }
    Ok(history_wire::ExecutionBrowsePage {
        entries,
        next_after_execution_id: page.next.as_ref().map(|id| id.as_str().to_owned()),
        discovery_complete: page.discovery_complete,
        conflicts,
        omitted_conflicts: page.omitted_conflicts as u32,
        issues,
        omitted_issues: page.omitted_issues as u32,
        cycles,
    })
}

fn browse_cycle(
    value: &ExecutionBrowseCycle,
) -> Result<history_wire::ExecutionBrowseCycle, Status> {
    Ok(history_wire::ExecutionBrowseCycle {
        cycle_id: value.cycle_id.as_str().to_owned(),
        label: value.label.clone(),
        lifecycle: match value.lifecycle {
            ExecutionBrowseLifecycle::Running => history_wire::ExecutionBrowseLifecycle::Running,
            ExecutionBrowseLifecycle::Interrupted => {
                history_wire::ExecutionBrowseLifecycle::Interrupted
            }
            ExecutionBrowseLifecycle::Pending => history_wire::ExecutionBrowseLifecycle::Pending,
            ExecutionBrowseLifecycle::Uncheckpointed => {
                history_wire::ExecutionBrowseLifecycle::Uncheckpointed
            }
            ExecutionBrowseLifecycle::Completed => {
                history_wire::ExecutionBrowseLifecycle::Completed
            }
        },
        result_status: value.result_status.as_ref().map(|status| match status {
            ResultStatus::Pending => execution_wire::ResultStatus::Pending,
            ResultStatus::Passed => execution_wire::ResultStatus::Passed,
            ResultStatus::Failed => execution_wire::ResultStatus::Failed,
            ResultStatus::Blocked => execution_wire::ResultStatus::Blocked,
            ResultStatus::Cancelled => execution_wire::ResultStatus::Cancelled,
        }),
        case_count: value.case_count as u64,
        execution_count: value.execution_count as u64,
        unrun_count: value.unrun_count as u64,
        queued_count: value.queued_count as u64,
        running_count: value.running_count as u64,
        passed_count: value.passed_count as u64,
        failed_count: value.failed_count as u64,
        blocked_count: value.blocked_count as u64,
        started_at: value
            .started_at
            .as_ref()
            .map(value::canonical::timestamp)
            .transpose()?,
        finished_at: value
            .finished_at
            .as_ref()
            .map(value::canonical::timestamp)
            .transpose()?,
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
) -> Result<checkpoint_wire::ExecutionCheckpoint, Status> {
    checkpoint_wire_ref(&value)
}

pub(super) fn checkpoint_wire_ref(
    value: &ExecutionCheckpoint,
) -> Result<checkpoint_wire::ExecutionCheckpoint, Status> {
    value.validate().map_err(|error| {
        detailed(
            Status::internal("invalid native checkpoint"),
            &dokime_runtime::runtime::Error::Operation(error.to_string()),
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
    Ok(checkpoint_wire::ExecutionCheckpoint {
        execution_identifier: value.execution_id.as_str().to_owned(),
        position: Some(cursor_wire_ref(&value.position)),
        version,
        evidence_through: Some(cursor_wire_ref(&value.evidence_through)),
    })
}

fn cursor(value: Option<checkpoint_wire::StorageCursor>) -> Result<StorageCursor, Status> {
    let value = value.ok_or_else(|| Status::invalid_argument("checkpoint cursor is required"))?;
    let history = value
        .history
        .ok_or_else(|| Status::invalid_argument("checkpoint history is required"))?;
    value::text(&history.store)?;
    value::text(&history.epoch)?;
    let store = dokime_domain::model::identity::Identifier::new(history.store)
        .map_err(|_| Status::invalid_argument("invalid checkpoint store identifier"))?;
    let epoch = dokime_domain::model::identity::Identifier::new(history.epoch)
        .map_err(|_| Status::invalid_argument("invalid checkpoint history epoch"))?;
    Ok(StorageCursor {
        history: StoreHistory { store, epoch },
        sequence: value.sequence,
    })
}

fn cursor_wire_ref(value: &StorageCursor) -> checkpoint_wire::StorageCursor {
    checkpoint_wire::StorageCursor {
        history: Some(checkpoint_wire::StoreHistory {
            store: value.history.store.as_str().to_owned(),
            epoch: value.history.epoch.as_str().to_owned(),
        }),
        sequence: value.sequence,
    }
}

pub(crate) fn traceability_memory(
    source: &dokime_runtime::traceability::TraceabilitySnapshot,
) -> Result<usize, Status> {
    use super::super::value::canonical;
    use dokime_domain::operations::common;
    value::text(source.traceability_id.as_str())?;
    value::collection(source.entries.len())?;
    let mut bytes = std::mem::size_of::<common::TraceabilitySnapshot>()
        .saturating_add(source.traceability_id.as_str().len())
        .saturating_add(canonical::scope_memory(&source.scope)?)
        .saturating_add(
            source
                .entries
                .len()
                .saturating_mul(std::mem::size_of::<common::TraceabilityEntry>()),
        );
    if let Some(metadata) = &source.metadata {
        bytes = bytes.saturating_add(canonical::metadata_memory(metadata)?);
    }
    let mut budget = value::Budget::new();
    for entry in &source.entries {
        bytes = bytes
            .saturating_add(canonical::metadata_memory(&entry.metadata)?)
            .saturating_add(canonical::memory(&entry.value, &mut budget, 0)?);
        for text in [entry.source.as_ref(), entry.unit.as_ref()]
            .into_iter()
            .flatten()
        {
            value::text(text)?;
            bytes = bytes.saturating_add(text.len());
        }
    }
    Ok(bytes)
}

pub(crate) fn traceability(
    source: &dokime_runtime::traceability::TraceabilitySnapshot,
) -> Result<dokime_domain::operations::common::TraceabilitySnapshot, Status> {
    use super::super::value::canonical;
    use dokime_domain::operations::common;
    traceability_memory(source)?;
    let mut budget = value::Budget::new();
    let mut entries = canonical::slots(source.entries.len())?;
    for entry in &source.entries {
        entries.push(common::TraceabilityEntry {
            metadata: Some(canonical::metadata(&entry.metadata)?),
            value: Some(canonical::encode(&entry.value, &mut budget, 0)?),
            source: entry.source.clone(),
            unit: entry.unit.clone(),
        });
    }
    Ok(common::TraceabilitySnapshot {
        traceability_id: source.traceability_id.as_str().to_owned(),
        scope: Some(canonical::scope(&source.scope)?),
        created_at: Some(canonical::timestamp(&source.created_at)?),
        metadata: source
            .metadata
            .as_ref()
            .map(canonical::metadata)
            .transpose()?,
        entries,
    })
}

/* === Private Functions === */
fn browse_entry(
    value: &ExecutionBrowseEntry,
) -> Result<history_wire::ExecutionBrowseEntry, Status> {
    Ok(history_wire::ExecutionBrowseEntry {
        archived: value.archived,
        metadata: Some(metadata(&value.metadata)?),
        checkpoint: value
            .checkpoint
            .as_ref()
            .map(checkpoint_wire_ref)
            .transpose()?,
        availability: match value.availability {
            ExecutionAvailability::Live => history_wire::ExecutionAvailability::Live,
            ExecutionAvailability::Stored => history_wire::ExecutionAvailability::Stored,
            ExecutionAvailability::Interrupted => history_wire::ExecutionAvailability::Interrupted,
            ExecutionAvailability::Uncheckpointed => {
                history_wire::ExecutionAvailability::Uncheckpointed
            }
        },
    })
}

fn metadata(
    value: &dokime_runtime::execution::storage::ExecutionMetadata,
) -> Result<history_wire::HistoricalExecutionMetadata, Status> {
    Ok(history_wire::HistoricalExecutionMetadata {
        execution_id: value.execution_id.as_str().to_owned(),
        cycle_id: Some(value.cycle_id.as_str().to_owned()),
        case_id: value.case_id.as_str().to_owned(),
        title: value.title.clone(),
        status: execution::execution_status(&value.status),
        started_at: value
            .started_at
            .as_ref()
            .map(value::canonical::timestamp)
            .transpose()?,
        finished_at: value
            .finished_at
            .as_ref()
            .map(value::canonical::timestamp)
            .transpose()?,
        result: value.result.as_ref().map(execution::result_summary),
        template_id: value.template_id.clone(),
        suite_label: value.suite_label.clone(),
    })
}

/* === Tests === */
