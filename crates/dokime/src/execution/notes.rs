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
use crate::core::{NoteId, RecordScope, Timestamp};

/* === Constants === */

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

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos::MemorySize for OperatorNote {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.note_id),
                std::mem::size_of_val(&self.note_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.scope),
                std::mem::size_of_val(&self.scope),
            ),
            (
                harmos::MemorySize::memory_size(&self.author),
                std::mem::size_of_val(&self.author),
            ),
            (
                harmos::MemorySize::memory_size(&self.message),
                std::mem::size_of_val(&self.message),
            ),
            (
                harmos::MemorySize::memory_size(&self.created_at),
                std::mem::size_of_val(&self.created_at),
            ),
        ];
        fields
            .into_iter()
            .fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
                if owned == usize::MAX {
                    usize::MAX
                } else {
                    bytes.saturating_add(owned.saturating_sub(inline))
                }
            })
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
