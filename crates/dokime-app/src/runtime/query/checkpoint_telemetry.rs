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
pub use dokime::storage::telemetry::TelemetryEntry as ExecutionTelemetryRow;

/* === Imports === */
// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use crate::{Result, RuntimeHandle};
use dokime::core::Identifier;
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
pub struct ExecutionTelemetryQuery {
    pub after: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionTelemetryTable {
    pub table: String,
    pub contract: dokime::telemetry::TelemetryContract,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionTelemetryPage {
    pub checkpoint: ExecutionCheckpoint,
    pub table: String,
    pub rows: Vec<ExecutionTelemetryRow>,
    pub next: Option<String>,
    pub issues: Vec<ExecutionBrowseIssue>,
    pub issues_truncated: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl MemorySize for ExecutionTelemetryTable {
    fn memory_size(&self) -> usize {
        std::mem::size_of::<Self>()
            .saturating_add(self.table.capacity())
            .saturating_add(
                self.contract
                    .memory_size()
                    .saturating_sub(std::mem::size_of_val(&self.contract)),
            )
    }
}

impl MemorySize for ExecutionTelemetryPage {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                self.checkpoint.memory_size(),
                std::mem::size_of_val(&self.checkpoint),
            ),
            (self.table.memory_size(), std::mem::size_of_val(&self.table)),
            (self.rows.memory_size(), std::mem::size_of_val(&self.rows)),
            (self.next.memory_size(), std::mem::size_of_val(&self.next)),
            (
                self.issues.memory_size(),
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
    pub async fn execution_checkpoint_telemetry(
        &self,
        checkpoint: ExecutionCheckpoint,
        stream: &Identifier,
        query: ExecutionTelemetryQuery,
    ) -> Result<QueryValue<ExecutionTelemetryPage>> {
        let view = self.checkpoint_execution(&checkpoint).await?;
        crate::runtime::storage::telemetry_pages::read(self.storage(), &view, stream, query).await
    }

    pub async fn execution_checkpoint_telemetry_projected<U: MemorySize>(
        &self,
        checkpoint: ExecutionCheckpoint,
        stream: &Identifier,
        query: ExecutionTelemetryQuery,
        map: impl FnOnce(&ExecutionTelemetryPage, NonZeroUsize) -> harmos::Result<U>,
    ) -> Result<QueryValue<U>> {
        let view = self.checkpoint_execution(&checkpoint).await?;
        crate::runtime::storage::telemetry_pages::read_projected(
            self.storage(),
            &view,
            stream,
            query,
            map,
        )
        .await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
