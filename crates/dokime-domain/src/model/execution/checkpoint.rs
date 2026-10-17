/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Validate native saved evidence checkpoints

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos::storage::ObjectVersionToken;
use harmos::storage::StorageCursor;
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use crate::model::identity::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
pub type Result<T> = std::result::Result<T, SavedEvidenceError>;

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionCheckpoint {
    pub execution_id: TestExecutionId,
    pub position: StorageCursor,
    pub version: ObjectVersionToken,
    pub evidence_through: StorageCursor,
}

/* === Enums === */
#[derive(Debug, thiserror::Error)]
pub enum SavedEvidenceError {
    #[error("invalid saved execution evidence: {0}")]
    Invalid(&'static str),
    #[error(transparent)]
    Storage(#[from] harmos::Error),
    #[error(transparent)]
    Timestamp(#[from] time::error::Parse),
}

/* === Traits === */

/* === Implementations === */
impl harmos::MemorySize for ExecutionCheckpoint {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.execution_id),
                std::mem::size_of_val(&self.execution_id),
            ),
            (
                harmos::MemorySize::memory_size(&self.position),
                std::mem::size_of_val(&self.position),
            ),
            (
                harmos::MemorySize::memory_size(&self.version),
                std::mem::size_of_val(&self.version),
            ),
            (
                harmos::MemorySize::memory_size(&self.evidence_through),
                std::mem::size_of_val(&self.evidence_through),
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
impl ExecutionCheckpoint {
    pub fn validate(&self) -> Result<()> {
        self.version.check_position(&self.position)?;
        if !self.position.covers(&self.evidence_through)? {
            return Err(SavedEvidenceError::Invalid(
                "dependency exceeds saved boundary",
            ));
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
