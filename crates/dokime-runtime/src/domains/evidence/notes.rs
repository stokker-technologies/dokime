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

pub(crate) use crate::transactions::AddOperatorNote;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::RuntimeHandle;
use crate::transactions::notes::{CreateOperatorNote, EditOperatorNote, admission_fits};
use dokime_domain::model::execution::notes::NOTE_ACTOR_BYTES;
use dokime_domain::model::execution::notes::NOTE_EDIT_NOT_AUTHOR;
use dokime_domain::model::execution::notes::NOTE_OWNER_SEALED;
use dokime_domain::model::execution::notes::NOTE_TEXT_BYTES;
use dokime_domain::model::execution::notes::NoteAdmission;
use dokime_domain::model::execution::notes::NoteOwner;
use dokime_domain::model::execution::notes::OperatorNote;
use dokime_domain::model::execution::notes::note_identity;
use dokime_domain::model::identity::CommandId;
use dokime_domain::model::identity::NoteId;
use dokime_domain::model::identity::SessionId;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug)]
pub struct CreateNote {
    pub epoch: SessionId,
    pub mutation_id: CommandId,
    pub owner: NoteOwner,
    pub actor: String,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct EditNote {
    pub epoch: SessionId,
    pub note_id: NoteId,
    pub actor: String,
    pub message: String,
}

#[derive(Clone, Debug)]
pub struct NoteCreation {
    pub epoch: SessionId,
    pub admission: NoteAdmission,
    pub publication: NotePublication,
}

/* === Enums === */

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NotePublication {
    NoExistingDatasets,
    Confirmed,
    Unconfirmed { targets: Vec<TestExecutionId> },
}

#[derive(Debug, thiserror::Error)]
pub enum NoteError {
    #[error("{0}")]
    Invalid(String),

    #[error("{0}")]
    Owner(String),

    #[error("note epoch changed; resynchronize before creating")]
    Epoch,

    #[error("note mutation identity was reused with different input")]
    Conflict,

    #[error("note creation capacity exhausted")]
    Overloaded,

    #[error("{NOTE_OWNER_SEALED}")]
    Sealed,

    #[error("note is absent from current evidence")]
    Absent,

    #[error("{NOTE_EDIT_NOT_AUTHOR}")]
    NotAuthor,

    #[error("{0}")]
    Unavailable(String),
}

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    pub async fn create_current_note(&self, request: CreateNote) -> Result<NoteCreation, NoteError> {
        if request.actor.is_empty()
            || request.actor.len() > NOTE_ACTOR_BYTES
            || request.message.trim().is_empty()
            || request.message.len() > NOTE_TEXT_BYTES
        {
            return Err(NoteError::Invalid("note actor/text exceeds bounds or is empty".into()));
        }
        if self.is_shutting_down() {
            return Err(NoteError::Unavailable("runtime is shutting down".into()));
        }
        let permit = self.admit_owned_work().ok_or_else(|| NoteError::Unavailable("runtime is shutting down".into()))?;
        if self.is_shutting_down() {
            return Err(NoteError::Unavailable("runtime is shutting down".into()));
        }
        let runtime = self.clone();
        tokio::spawn(async move {
            let _permit = permit;
            let _admission = runtime.evidence_admission.lock().await;
            runtime.create_note_owned(request).await
        })
        .await
        .map_err(|error| NoteError::Unavailable(error.to_string()))?
    }

    async fn create_note_owned(&self, request: CreateNote) -> Result<NoteCreation, NoteError> {
        // A sealed owner refuses before anything is written; a retry still answers.
        {
            let state = self.state_read();
            let id = note_identity(&request.epoch, &request.actor, &request.mutation_id);
            if !state.note_admissions.contains_key(&id)
                && state.note_owner_scope(&request.owner).is_ok_and(|scope| state.note_scope_sealed(&scope))
            {
                return Err(NoteError::Sealed);
            }
        }
        // Save native membership before selecting the annotation targets.
        self.execution_storage
            .save_cycle(request.owner.cycle_id().clone())
            .await
            .map_err(|error| NoteError::Unavailable(error.to_string()))?;
        let state = self.state_read();
        if state.process_epoch() != &request.epoch {
            return Err(NoteError::Epoch);
        }
        let id = note_identity(&request.epoch, &request.actor, &request.mutation_id);
        let admission = if let Some(existing) = self.read_note_admission(&request.epoch, &id).await? {
            if existing.owner != request.owner
                || existing.note.author.as_deref() != Some(&request.actor)
                || existing.note.message != request.message
            {
                return Err(NoteError::Conflict);
            }
            existing
        } else {
            let scope = state.note_owner_scope(&request.owner).map_err(NoteError::Owner)?;
            let _guards = self.execution_storage.admit_annotations(&scope).await.map_err(|error| NoteError::Owner(error.to_string()))?;
            let targets = self
                .execution_storage
                .note_targets(&state.cycle_view(request.owner.cycle_id()).expect("note cycle exists"), &scope)
                .await
                .map_err(|_| NoteError::Overloaded)?;
            let admission = NoteAdmission {
                mutation_id: request.mutation_id,
                owner: request.owner,
                note: OperatorNote {
                    note_id: id.clone(),
                    scope,
                    author: Some(request.actor),
                    message: request.message,
                    created_at: crate::model::time::now_utc(),
                },
                checkpoint_targets: targets,
            };
            if !admission_fits(&state, &admission) {
                return Err(NoteError::Overloaded);
            }
            self.commit(CreateOperatorNote { admission: admission.clone() })
                .await
                .map_err(|error| NoteError::Unavailable(error.to_string()))?;
            self.read_note_admission(&request.epoch, &id)
                .await?
                .ok_or_else(|| NoteError::Unavailable("committed note receipt could not be read".into()))?
        };
        drop(state);
        // Current ownership can disappear only through its native lifecycle.
        // Retired targets are verified read-only below; never replay a stored
        // checkpoint annotation or redirect this note to another owner.
        let snapshot = self.state_read();
        if let Some(cycle) = snapshot.cycle_view(admission.owner.cycle_id())
            && let Err(error) = self.execution_storage.checkpoint_targets(&cycle, &admission.checkpoint_targets).await
        {
            tracing::warn!(%error, note_id=%id, "committed note checkpoint publication failed");
        }
        let publication = self.note_publication(&admission).await;
        Ok(NoteCreation { epoch: request.epoch, admission, publication })
    }

    pub async fn edit_current_note(&self, request: EditNote) -> Result<OperatorNote, NoteError> {
        if request.message.trim().is_empty() || request.message.len() > NOTE_TEXT_BYTES {
            return Err(NoteError::Invalid("note text exceeds bounds or is empty".into()));
        }
        if self.is_shutting_down() {
            return Err(NoteError::Unavailable("runtime is shutting down".into()));
        }
        let runtime = self.clone();
        tokio::spawn(async move {
            let _admission = runtime.evidence_admission.lock().await;
            runtime.edit_note_owned(request).await
        })
        .await
        .map_err(|error| NoteError::Unavailable(error.to_string()))?
    }

    async fn edit_note_owned(&self, request: EditNote) -> Result<OperatorNote, NoteError> {
        let scope = {
            let state = self.state_read();
            if state.process_epoch() != &request.epoch {
                return Err(NoteError::Epoch);
            }
            let note = state.find_note(&request.note_id).ok_or(NoteError::Absent)?;
            if note.author.as_deref() != Some(request.actor.as_str()) {
                return Err(NoteError::NotAuthor);
            }
            if state.note_edit_sealed(&note.scope) {
                return Err(NoteError::Sealed);
            }
            note.scope.clone()
        };
        let guards = self.execution_storage.admit_annotations(&scope).await.map_err(|error| NoteError::Owner(error.to_string()))?;
        self.commit(EditOperatorNote {
            note_id: request.note_id.clone(),
            actor: request.actor,
            message: request.message,
            edited_at: crate::model::time::now_utc(),
        })
        .await
        .map_err(|error| NoteError::Unavailable(error.to_string()))?;
        drop(guards);
        if let Err(error) = self.checkpoint_evidence_scope(&scope).await {
            tracing::warn!(%error, note_id=%request.note_id, "edited note checkpoint failed");
        }
        self.state_read().find_note(&request.note_id).cloned().ok_or(NoteError::Absent)
    }

    pub async fn get_note_creation(&self, epoch: &SessionId, id: &NoteId) -> Result<Option<NoteCreation>, NoteError> {
        let Some(admission) = self.read_note_admission(epoch, id).await? else {
            return Ok(None);
        };
        let publication = self.note_publication(&admission).await;
        Ok(Some(NoteCreation { epoch: epoch.clone(), admission, publication }))
    }

    async fn read_note_admission(&self, epoch: &SessionId, id: &NoteId) -> Result<Option<NoteAdmission>, NoteError> {
        let epoch = epoch.clone();
        let id = id.clone();
        self.state_manager
            .state()
            .read(move |state| {
                if state.process_epoch() != &epoch {
                    return Err(NoteError::Epoch);
                }
                Ok(state.note_admissions.get(&id).cloned())
            })
            .await
            .map_err(|error| NoteError::Unavailable(error.to_string()))?
            .value
    }

    async fn note_publication(&self, admission: &NoteAdmission) -> NotePublication {
        if admission.checkpoint_targets.is_empty() {
            return NotePublication::NoExistingDatasets;
        }
        let mut targets = Vec::new();
        for id in &admission.checkpoint_targets {
            match self.execution_storage.note_is_published(id, &admission.note).await {
                Ok(true) => {}
                Ok(false) => targets.push(id.clone()),
                Err(error) => {
                    tracing::warn!(%error, note_id=%admission.note.note_id, "note publication could not be verified");
                    targets.push(id.clone());
                }
            }
        }
        if targets.is_empty() { NotePublication::Confirmed } else { NotePublication::Unconfirmed { targets } }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
