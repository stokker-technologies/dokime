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
pub use dokime::execution::notes::*;

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};
// Local Imports
use dokime::core::NoteId;

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
pub struct NoteAdmission {
    pub mutation_id: dokime::core::CommandId,
    pub owner: NoteOwner,
    pub note: crate::execution::OperatorNote,
    pub checkpoint_targets: Vec<dokime::core::TestExecutionId>,
}

/* === Enums === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NoteOwner {
    Cycle {
        cycle_id: dokime::core::TestCycleId,
    },
    Case {
        cycle_id: dokime::core::TestCycleId,
        case_id: dokime::core::CaseId,
    },
    Execution {
        cycle_id: dokime::core::TestCycleId,
        execution_id: dokime::core::TestExecutionId,
    },
}

/* === Traits === */

/* === Implementations === */
impl NoteOwner {
    pub fn cycle_id(&self) -> &dokime::core::TestCycleId {
        match self {
            Self::Cycle { cycle_id }
            | Self::Case { cycle_id, .. }
            | Self::Execution { cycle_id, .. } => cycle_id,
        }
    }
}

/* === Public Functions === */
pub fn note_identity(
    epoch: &dokime::core::SessionId,
    actor: &str,
    mutation: &dokime::core::CommandId,
) -> NoteId {
    NoteId::from_seed(format!(
        "note:{}:{}:{}:{}",
        epoch,
        actor.len(),
        actor,
        mutation
    ))
}

/* === Private Functions === */

/* === Tests === */
