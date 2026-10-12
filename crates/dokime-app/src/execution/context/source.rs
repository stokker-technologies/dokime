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
use std::collections::BTreeMap;

// Third Party Imports
use harmos::{MemorySize, storage::StorageCursor};

// Local Imports
use super::ExecutionContext;
use crate::{Error, Result};
use dokime::{
    core::{Identifier, NodeId},
    execution::saved::{ExecutionOutput, ExecutionRevision},
    records::RecordContract,
    telemetry::contract::TelemetryContract,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CapturedSource {
    pub captured_at: dokime::core::Timestamp,
    pub owner_node_id: NodeId,
    pub execution: ExecutionRevision,
    pub evidence_through: StorageCursor,
    pub outputs: Vec<ExecutionOutput>,
    pub context: ExecutionContext,
    pub telemetry_contracts: BTreeMap<Identifier, BTreeMap<u32, TelemetryContract>>,
    pub record_contracts: BTreeMap<Identifier, BTreeMap<u32, RecordContract>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl MemorySize for CapturedSource {
    fn memory_size(&self) -> usize {
        let fields = [
            (
                self.captured_at.memory_size(),
                std::mem::size_of_val(&self.captured_at),
            ),
            (
                self.owner_node_id.memory_size(),
                std::mem::size_of_val(&self.owner_node_id),
            ),
            (
                self.execution.memory_size(),
                std::mem::size_of_val(&self.execution),
            ),
            (
                self.evidence_through.memory_size(),
                std::mem::size_of_val(&self.evidence_through),
            ),
            (
                self.outputs.memory_size(),
                std::mem::size_of_val(&self.outputs),
            ),
            (
                self.context.memory_size(),
                std::mem::size_of_val(&self.context),
            ),
            (
                self.record_contracts.memory_size(),
                std::mem::size_of_val(&self.record_contracts),
            ),
            (
                self.telemetry_contracts.memory_size(),
                std::mem::size_of_val(&self.telemetry_contracts),
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

impl CapturedSource {
    pub(crate) fn validation_memory(&self) -> usize {
        self.context.validation_memory()
    }

    pub(crate) fn validate_header(&self) -> Result<()> {
        self.execution
            .version
            .check_position(&self.execution.position)?;
        if !self.evidence_through.covers(&self.execution.position)?
            || self.context.scope.test_execution_id.as_ref() != Some(&self.execution.execution_id)
        {
            return Err(Error::Storage(dokime::storage::Error::Invalid(
                "execution source boundary or owner differs",
            )));
        }
        time::OffsetDateTime::parse(
            self.captured_at.as_str(),
            &time::format_description::well_known::Rfc3339,
        )
        .map_err(|_| {
            Error::Storage(dokime::storage::Error::Invalid(
                "execution capture time is invalid",
            ))
        })?;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
