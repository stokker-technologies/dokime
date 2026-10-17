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

// Third Party Imports
use tonic::Status;
// Local Imports
use super::super::value::{self, canonical};
use dokime_domain::model::identity::CaseId;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_runtime::domains::evidence::notes::{NoteCreation, NotePublication};
use dokime_runtime::execution::notes::NoteOwner;
use dokime_runtime::execution::{HealthItem, HealthSeverity, OperatorNote};
use dokime_domain::model::identity::wire as common;
use dokime_domain::operations::notes as contract;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn owner(value: Option<dokime_domain::model::notes::wire::AnnotationOwner>) -> Result<NoteOwner, Status> {
    use dokime_domain::model::notes::wire::AnnotationOwnerOwner as Owner;
    match value
        .and_then(|value| value.owner)
        .ok_or_else(|| Status::invalid_argument("explicit annotation owner required"))?
    {
        Owner::CycleId(id) => Ok(NoteOwner::Cycle {
            cycle_id: TestCycleId::from_uuid(id)
                .map_err(|_| Status::invalid_argument("invalid cycle UUID"))?,
        }),
        Owner::PlannedCase(value) => {
            value::text(&value.case_id)?;
            Ok(NoteOwner::Case {
                cycle_id: TestCycleId::from_uuid(value.cycle_id)
                    .map_err(|_| Status::invalid_argument("invalid cycle UUID"))?,
                case_id: CaseId::from_wire(value.case_id)
                    .map_err(|_| Status::invalid_argument("invalid case identity"))?,
            })
        }
        Owner::Execution(value) => Ok(NoteOwner::Execution {
            cycle_id: TestCycleId::from_uuid(value.cycle_id)
                .map_err(|_| Status::invalid_argument("invalid cycle UUID"))?,
            execution_id: TestExecutionId::from_uuid(value.execution_id)
                .map_err(|_| Status::invalid_argument("invalid execution UUID"))?,
        }),
    }
}

pub(super) fn encode_owner(value: &NoteOwner) -> dokime_domain::model::notes::wire::AnnotationOwner {
    use dokime_domain::model::notes::wire::AnnotationOwnerOwner as Owner;
    dokime_domain::model::notes::wire::AnnotationOwner {
        owner: Some(match value {
            NoteOwner::Cycle { cycle_id } => Owner::CycleId(cycle_id.to_string()),
            NoteOwner::Case { cycle_id, case_id } => {
                Owner::PlannedCase(dokime_domain::model::notes::wire::PlannedCaseNoteOwner {
                    cycle_id: cycle_id.to_string(),
                    case_id: case_id.to_string(),
                })
            }
            NoteOwner::Execution {
                cycle_id,
                execution_id,
            } => Owner::Execution(dokime_domain::model::notes::wire::ExecutionNoteOwner {
                cycle_id: cycle_id.to_string(),
                execution_id: execution_id.to_string(),
            }),
        }),
    }
}

pub(super) fn note_size(note: &OperatorNote) -> Result<usize, Status> {
    bounded_text(&note.message, dokime_runtime::execution::notes::NOTE_TEXT_BYTES)?;
    if let Some(author) = &note.author {
        bounded_text(author, dokime_runtime::execution::notes::NOTE_ACTOR_BYTES)?;
    }
    Ok(1024usize
        .saturating_add(scope_bytes(&note.scope)?)
        .saturating_add(note.message.len())
        .saturating_add(note.author.as_ref().map_or(0, String::len)))
}

pub(super) fn health_size(item: &HealthItem) -> Result<usize, Status> {
    bounded_text(&item.message, dokime_runtime::execution::notes::NOTE_TEXT_BYTES)?;
    if let Some(detail) = &item.detail {
        bounded_text(detail, dokime_runtime::execution::notes::NOTE_TEXT_BYTES)?;
    }
    Ok(1024usize
        .saturating_add(scope_bytes(&item.scope)?)
        .saturating_add(item.message.len())
        .saturating_add(item.detail.as_ref().map_or(0, String::len)))
}

pub(crate) fn note(note: &OperatorNote) -> Result<dokime_domain::model::notes::wire::OperatorNote, Status> {
    note_size(note)?;
    Ok(dokime_domain::model::notes::wire::OperatorNote {
        note_id: note.note_id.as_str().to_owned(),
        scope: Some(
            canonical::scope(&note.scope)
                .map_err(|_| Status::resource_exhausted("note scope construction failed"))?,
        ),
        author: note.author.clone(),
        message: note.message.clone(),
        created_at: Some(canonical::timestamp(&note.created_at)?),
    })
}

pub(crate) fn health(item: &HealthItem) -> Result<dokime_domain::model::notes::wire::HealthItem, Status> {
    health_size(item)?;
    Ok(dokime_domain::model::notes::wire::HealthItem {
        health_item_id: item.health_item_id.as_str().to_owned(),
        scope: Some(
            canonical::scope(&item.scope)
                .map_err(|_| Status::resource_exhausted("health scope construction failed"))?,
        ),
        severity: match item.severity {
            HealthSeverity::Info => common::Severity::Info,
            HealthSeverity::Warning => common::Severity::Warn,
            HealthSeverity::Error => common::Severity::Error,
        },
        message: item.message.clone(),
        detail: item.detail.clone(),
        observed_at: Some(canonical::timestamp(&item.observed_at)?),
    })
}

pub(super) fn receipt(value: &NoteCreation) -> Result<dokime_domain::model::notes::wire::NoteReceipt, Status> {
    let (publication, unconfirmed_targets) = match &value.publication {
        NotePublication::NoExistingDatasets => {
            (dokime_domain::model::notes::wire::NotePublication::NoExistingDatasets, Vec::new())
        }
        NotePublication::Confirmed => (dokime_domain::model::notes::wire::NotePublication::Confirmed, Vec::new()),
        NotePublication::Unconfirmed { targets } => (
            dokime_domain::model::notes::wire::NotePublication::Unconfirmed,
            targets.iter().map(ToString::to_string).collect(),
        ),
    };
    let epoch = value.epoch.to_string();
    let value = &value.admission;
    Ok(dokime_domain::model::notes::wire::NoteReceipt {
        epoch,
        mutation_id: value.mutation_id.to_string(),
        owner: Some(encode_owner(&value.owner)),
        note: Some(note(&value.note)?),
        checkpoint_targets: value
            .checkpoint_targets
            .iter()
            .map(ToString::to_string)
            .collect(),
        publication,
        unconfirmed_targets,
    })
}

/* === Private Functions === */

// Check borrowed strings and total payload before conversion. UUID scope fields
// are fixed-size native identities; timestamp parsing never copies its text.
fn bounded_text(value: &str, limit: usize) -> Result<(), Status> {
    if value.len() > limit {
        Err(Status::resource_exhausted(
            "annotation text exceeds byte budget",
        ))
    } else {
        Ok(())
    }
}

fn scope_bytes(scope: &dokime_domain::model::execution::RecordScope) -> Result<usize, Status> {
    if let Some(case) = &scope.test_case_id {
        value::text(case.as_str())?;
        Ok(case.as_str().len())
    } else {
        Ok(0)
    }
}

/* === Tests === */
