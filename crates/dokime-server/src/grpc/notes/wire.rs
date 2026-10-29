/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use tonic::Status;

use super::super::value::canonical;
use dokime::domain::model::execution::health::HealthItem;
use dokime::domain::model::execution::health::HealthSeverity;
use dokime::domain::model::execution::notes::NoteOwner;
use dokime::domain::model::execution::notes::OperatorNote;
use dokime::domain::model::identity::CaseId;
use dokime::domain::model::identity::TestCycleId;
use dokime::domain::model::identity::TestExecutionId;
use dokime::domain::model::identity::wire as common;
use dokime::domains::evidence::notes::{NoteCreation, NotePublication};

pub(super) fn owner(value: Option<dokime::domain::model::notes::wire::AnnotationOwner>) -> Result<NoteOwner, Status> {
    use dokime::domain::model::notes::wire::AnnotationOwnerOwner as Owner;
    match value.and_then(|value| value.owner).ok_or_else(|| Status::invalid_argument("explicit annotation owner required"))? {
        Owner::CycleId(id) => Ok(NoteOwner::Cycle {
            cycle_id: TestCycleId::from_uuid(id).map_err(|_| Status::invalid_argument("invalid cycle UUID"))?,
        }),
        Owner::PlannedCase(value) => Ok(NoteOwner::Case {
            cycle_id: TestCycleId::from_uuid(value.cycle_id).map_err(|_| Status::invalid_argument("invalid cycle UUID"))?,
            case_id: CaseId::from_wire(value.case_id).map_err(|_| Status::invalid_argument("invalid case identity"))?,
        }),
        Owner::Execution(value) => Ok(NoteOwner::Execution {
            cycle_id: TestCycleId::from_uuid(value.cycle_id).map_err(|_| Status::invalid_argument("invalid cycle UUID"))?,
            execution_id: TestExecutionId::from_uuid(value.execution_id).map_err(|_| Status::invalid_argument("invalid execution UUID"))?,
        }),
    }
}

pub(super) fn encode_owner(value: &NoteOwner) -> dokime::domain::model::notes::wire::AnnotationOwner {
    use dokime::domain::model::notes::wire::AnnotationOwnerOwner as Owner;
    dokime::domain::model::notes::wire::AnnotationOwner {
        owner: Some(match value {
            NoteOwner::Cycle { cycle_id } => Owner::CycleId(cycle_id.to_string()),
            NoteOwner::Case { cycle_id, case_id } => Owner::PlannedCase(dokime::domain::model::notes::wire::PlannedCaseNoteOwner {
                cycle_id: cycle_id.to_string(),
                case_id: case_id.to_string(),
            }),
            NoteOwner::Execution { cycle_id, execution_id } => Owner::Execution(dokime::domain::model::notes::wire::ExecutionNoteOwner {
                cycle_id: cycle_id.to_string(),
                execution_id: execution_id.to_string(),
            }),
        }),
    }
}

pub(crate) fn note(note: &OperatorNote) -> Result<dokime::domain::model::notes::wire::OperatorNote, Status> {
    Ok(dokime::domain::model::notes::wire::OperatorNote {
        note_id: note.note_id.as_str().to_owned(),
        scope: Some(canonical::scope(&note.scope).map_err(|_| Status::resource_exhausted("note scope construction failed"))?),
        author: note.author.clone(),
        message: note.message.clone(),
        created_at: Some(canonical::timestamp(&note.created_at)?),
    })
}

pub(crate) fn health(item: &HealthItem) -> Result<dokime::domain::model::notes::wire::HealthItem, Status> {
    Ok(dokime::domain::model::notes::wire::HealthItem {
        health_item_id: item.health_item_id.as_str().to_owned(),
        scope: Some(canonical::scope(&item.scope).map_err(|_| Status::resource_exhausted("health scope construction failed"))?),
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

pub(super) fn receipt(value: &NoteCreation) -> Result<dokime::domain::model::notes::wire::NoteReceipt, Status> {
    let (publication, unconfirmed_targets) = match &value.publication {
        NotePublication::NoExistingDatasets => (dokime::domain::model::notes::wire::NotePublication::NoExistingDatasets, Vec::new()),
        NotePublication::Confirmed => (dokime::domain::model::notes::wire::NotePublication::Confirmed, Vec::new()),
        NotePublication::Unconfirmed { targets } => {
            (dokime::domain::model::notes::wire::NotePublication::Unconfirmed, targets.iter().map(ToString::to_string).collect())
        }
    };
    let epoch = value.epoch.to_string();
    let value = &value.admission;
    Ok(dokime::domain::model::notes::wire::NoteReceipt {
        epoch,
        mutation_id: value.mutation_id.to_string(),
        owner: Some(encode_owner(&value.owner)),
        note: Some(note(&value.note)?),
        checkpoint_targets: value.checkpoint_targets.iter().map(ToString::to_string).collect(),
        publication,
        unconfirmed_targets,
    })
}
