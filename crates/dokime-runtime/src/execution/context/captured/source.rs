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
use crate::records::RecordContract;
use crate::telemetry::contract::TelemetryContract;
use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::execution::saved::ExecutionOutput;
use dokime_domain::model::execution::saved::ExecutionRevision;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::NodeId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CapturedSource {
    pub captured_at: dokime_domain::model::time::Timestamp,
    pub owner_node_id: NodeId,
    pub execution_id: dokime_domain::model::identity::TestExecutionId,
    pub execution: Option<ExecutionRevision>,
    pub lifecycle_id: String,
    pub lifecycle: dokime_domain::model::execution::result::TestExecution,
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
            (self.execution_id.memory_size(), std::mem::size_of_val(&self.execution_id)),
            (self.lifecycle_id.memory_size(), std::mem::size_of_val(&self.lifecycle_id)),
            (self.lifecycle.memory_size(), std::mem::size_of_val(&self.lifecycle)),
            (self.captured_at.memory_size(), std::mem::size_of_val(&self.captured_at)),
            (self.owner_node_id.memory_size(), std::mem::size_of_val(&self.owner_node_id)),
            (self.execution.memory_size(), std::mem::size_of_val(&self.execution)),
            (self.evidence_through.memory_size(), std::mem::size_of_val(&self.evidence_through)),
            (self.outputs.memory_size(), std::mem::size_of_val(&self.outputs)),
            (self.context.memory_size(), std::mem::size_of_val(&self.context)),
            (self.record_contracts.memory_size(), std::mem::size_of_val(&self.record_contracts)),
            (self.telemetry_contracts.memory_size(), std::mem::size_of_val(&self.telemetry_contracts)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl CapturedSource {
    pub(crate) fn validate_header(&self) -> Result<()> {
        if let Some(anchor) = &self.execution {
            anchor.version.check_position(&anchor.position)?;
            self.evidence_through.history.check(&anchor.position.history)?;
            if anchor.execution_id != self.execution_id {
                return Err(Error::InvalidReportInputs("native anchor belongs to another execution".into()));
            }
        }
        uuid::Uuid::parse_str(&self.lifecycle_id).map_err(|_| Error::InvalidReportInputs("invalid lifecycle identity".into()))?;
        if self.lifecycle.test_execution_id != self.execution_id
            || self.lifecycle.saved.is_some()
            || self.context.scope.test_execution_id.as_ref() != Some(&self.execution_id)
        {
            return Err(Error::InvalidReportInputs("captured lifecycle identity differs from owner".into()));
        }
        self.context.validate(&self.lifecycle)?;
        time::OffsetDateTime::parse(self.captured_at.as_str(), &time::format_description::well_known::Rfc3339)
            .map_err(|_| Error::Storage(crate::storage::Error::Invalid("execution capture time is invalid")))?;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
