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
pub use crate::runtime::storage::context_records::ContextRecord;

/* === Imports === */
// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use crate::{Result, RuntimeHandle};
use harmos::{MemorySize, storage::QueryValue};
use serde::{Deserialize, Serialize};
// Local Imports
use super::execution_history::{ExecutionBrowseIssue, ExecutionCheckpoint};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionRecordQuery {
    pub after: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionRecordPage {
    pub checkpoint: ExecutionCheckpoint,
    pub records: Vec<ContextRecord>,
    pub next: Option<String>,
    pub issues: Vec<ExecutionBrowseIssue>,
    pub issues_truncated: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos::MemorySize for ExecutionRecordPage {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                harmos::MemorySize::memory_size(&self.checkpoint),
                std::mem::size_of_val(&self.checkpoint),
            ),
            (
                harmos::MemorySize::memory_size(&self.records),
                std::mem::size_of_val(&self.records),
            ),
            (
                harmos::MemorySize::memory_size(&self.next),
                std::mem::size_of_val(&self.next),
            ),
            (
                harmos::MemorySize::memory_size(&self.issues),
                std::mem::size_of_val(&self.issues),
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

impl RuntimeHandle {
    pub async fn execution_checkpoint_records(
        &self,
        checkpoint: ExecutionCheckpoint,
        query: ExecutionRecordQuery,
    ) -> Result<QueryValue<ExecutionRecordPage>> {
        let view = self.checkpoint_execution(&checkpoint).await?;
        crate::runtime::storage::record_pages::read(self.storage(), &view, query).await
    }

    pub async fn execution_checkpoint_records_projected<U: MemorySize>(
        &self,
        checkpoint: ExecutionCheckpoint,
        query: ExecutionRecordQuery,
        map: impl FnOnce(&ExecutionRecordPage, NonZeroUsize) -> harmos::Result<U>,
    ) -> Result<QueryValue<U>> {
        let view = self.checkpoint_execution(&checkpoint).await?;
        crate::runtime::storage::record_pages::read_projected(self.storage(), &view, query, map)
            .await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
