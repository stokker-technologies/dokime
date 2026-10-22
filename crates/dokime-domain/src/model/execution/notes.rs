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
use serde::{Deserialize, Serialize};

// Local Imports
use crate::model::execution::RecordScope;
use crate::model::identity::NoteId;
use crate::model::time::Timestamp;

/* === Constants === */

pub const NOTE_TEXT_BYTES: usize = 16 * 1024;
pub const NOTE_OWNER_BYTES: usize = 4096;
pub const NOTE_ACTOR_BYTES: usize = 256;
pub const NOTE_RECEIPT_LIMIT: usize = 4096;
pub const NOTE_RECEIPT_BYTES: usize = 8 * 1024 * 1024;
pub const NOTE_CHECKPOINT_LIMIT: usize = 64;
pub const NOTE_SCAN_LIMIT: usize = 10_000;
pub const NOTE_EDIT_NOT_AUTHOR: &str = "only the note's author may edit it";
pub const NOTE_OWNER_SEALED: &str = "note owner is sealed; its report keeps the notes it froze";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OperatorNote {
    pub note_id: NoteId,
    pub scope: RecordScope,
    pub author: Option<String>,
    pub message: String,
    pub created_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorNoteRequest {
    pub scope: Option<RecordScope>,
    pub author: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NoteAdmission {
    pub mutation_id: crate::model::identity::CommandId,
    pub owner: NoteOwner,
    pub note: crate::model::execution::notes::OperatorNote,
    pub checkpoint_targets: Vec<crate::model::identity::TestExecutionId>,
}

/* === Enums === */

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NoteOwner {
    Cycle { cycle_id: crate::model::identity::TestCycleId },
    Case { cycle_id: crate::model::identity::TestCycleId, case_id: crate::model::identity::CaseId },
    Execution { cycle_id: crate::model::identity::TestCycleId, execution_id: crate::model::identity::TestExecutionId },
}

/* === Traits === */

/* === Implementations === */

impl harmos_domain::MemorySize for OperatorNote {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.note_id), std::mem::size_of_val(&self.note_id)),
            (harmos_domain::MemorySize::memory_size(&self.scope), std::mem::size_of_val(&self.scope)),
            (harmos_domain::MemorySize::memory_size(&self.author), std::mem::size_of_val(&self.author)),
            (harmos_domain::MemorySize::memory_size(&self.message), std::mem::size_of_val(&self.message)),
            (harmos_domain::MemorySize::memory_size(&self.created_at), std::mem::size_of_val(&self.created_at)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl NoteOwner {
    pub fn cycle_id(&self) -> &crate::model::identity::TestCycleId {
        match self {
            Self::Cycle { cycle_id } | Self::Case { cycle_id, .. } | Self::Execution { cycle_id, .. } => cycle_id,
        }
    }
}

/* === Public Functions === */

pub fn note_identity(epoch: &crate::model::identity::SessionId, actor: &str, mutation: &crate::model::identity::CommandId) -> NoteId {
    NoteId::from_seed(format!("note:{}:{}:{}:{}", epoch, actor.len(), actor, mutation))
}

/* === Private Functions === */

/* === Tests === */
