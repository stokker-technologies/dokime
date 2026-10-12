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
pub(crate) use dokime::execution::saved::EXECUTION_CONTEXT_MEDIA_TYPE as MEDIA_TYPE;

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, num::NonZeroUsize};

// Third Party Imports

use harmos::{MemorySize, storage::StorageCursor};

// Local Imports

use super::{context::Context, records::Contracts as RecordContracts, telemetry::Contracts};
use crate::execution::context::{ExecutionContext, source::CapturedSource};
use crate::{Error, Result};
use dokime::core::{Identifier, NodeId, Timestamp};
use dokime::execution::saved::{EXECUTION_CONTEXT_SCHEMA_VERSION, ExecutionRevision};
use dokime::records::RecordContract;
use dokime::storage::objects::presentation as native;
use dokime::telemetry::contract::TelemetryContract;

/* === Constants === */
const SCHEMA_VERSION: u32 = EXECUTION_CONTEXT_SCHEMA_VERSION;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone, PartialEq)]
pub(crate) struct Source {
    schema_version: u32,
    owner_node_id: String,
    execution: Option<native::ExecutionRevision>,
    evidence_through: Option<StorageCursor>,
    telemetry_contracts: Option<Contracts>,
    context: Option<Context>,
    record_contracts: Option<RecordContracts>,
    captured_at: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Source {
    pub(crate) fn encoding_memory(value: &CapturedSource) -> Result<usize> {
        let mut bytes = std::mem::size_of::<Self>()
            .saturating_add(value.owner_node_id.as_str().len())
            .saturating_add(value.validation_memory())
            .saturating_add(value.captured_at.as_str().len());
        bytes = excess(
            bytes,
            native::ExecutionRevision::encoding_memory(&value.execution)?,
            std::mem::size_of::<native::ExecutionRevision>(),
        );
        bytes = excess(
            bytes,
            value.evidence_through.memory_size(),
            std::mem::size_of::<StorageCursor>(),
        );
        bytes = excess(
            bytes,
            Contracts::encoding_memory(&value.telemetry_contracts)?,
            std::mem::size_of::<Contracts>(),
        );
        bytes = excess(
            bytes,
            RecordContracts::encoding_memory(&value.record_contracts)?,
            std::mem::size_of::<RecordContracts>(),
        );
        Ok(excess(
            bytes,
            Context::encoding_memory(&value.context)?,
            std::mem::size_of::<Context>(),
        ))
    }

    pub(crate) fn projection_memory(&self) -> Result<usize> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(invalid("unsupported native captured report schema"));
        }
        let execution = required(&self.execution)?;
        let evidence = required(&self.evidence_through)?;
        let contracts = required(&self.telemetry_contracts)?;
        let records = required(&self.record_contracts)?;
        let context = required(&self.context)?;
        let mut bytes = std::mem::size_of::<CapturedSource>()
            .saturating_add(self.owner_node_id.capacity())
            .saturating_add(self.captured_at.capacity());
        bytes = excess(
            bytes,
            execution.projection_memory()?,
            std::mem::size_of::<ExecutionRevision>(),
        );
        bytes = excess(
            bytes,
            evidence.memory_size(),
            std::mem::size_of::<StorageCursor>(),
        );
        bytes = excess(
            bytes,
            contracts.projection_memory()?,
            std::mem::size_of::<BTreeMap<Identifier, BTreeMap<u32, TelemetryContract>>>(),
        );
        bytes = excess(
            bytes,
            records.projection_memory()?,
            std::mem::size_of::<BTreeMap<Identifier, BTreeMap<u32, RecordContract>>>(),
        );
        Ok(excess(
            bytes,
            context.projection_memory()?,
            std::mem::size_of::<ExecutionContext>(),
        ))
    }

    pub(crate) fn into_source(self, allowance: NonZeroUsize) -> Result<CapturedSource> {
        let bound = self.projection_memory()?;
        if bound == usize::MAX || bound > allowance.get() {
            return Err(invalid(
                "native captured report exceeds original projection admission",
            ));
        }
        if self
            .owner_node_id
            .bytes()
            .any(|byte| byte.is_ascii_uppercase())
        {
            return Err(invalid("native captured report owner is not canonical"));
        }
        let value = CapturedSource {
            owner_node_id: NodeId::from_uuid(self.owner_node_id)
                .map_err(dokime::storage::Error::from)?,
            execution: take(self.execution)?.try_into()?,
            evidence_through: take(self.evidence_through)?,
            telemetry_contracts: take(self.telemetry_contracts)?.try_into()?,
            record_contracts: take(self.record_contracts)?.try_into()?,
            outputs: Vec::new(),
            captured_at: Timestamp::new(self.captured_at),
            context: take(self.context)?.try_into()?,
        };
        value.validate_header()?;
        Ok(value)
    }
}

impl TryFrom<&CapturedSource> for Source {
    type Error = Error;
    fn try_from(value: &CapturedSource) -> Result<Self> {
        if Self::encoding_memory(value)? == usize::MAX {
            return Err(invalid("native captured report encoding memory overflows"));
        }
        value.validate_header()?;
        Ok(Self {
            schema_version: SCHEMA_VERSION,
            owner_node_id: text(value.owner_node_id.as_str())?,
            execution: Some(native::ExecutionRevision::try_from(&value.execution)?),
            evidence_through: Some(value.evidence_through.clone()),
            telemetry_contracts: Some(Contracts::try_from(&value.telemetry_contracts)?),
            record_contracts: Some(RecordContracts::try_from(&value.record_contracts)?),
            captured_at: text(value.captured_at.as_str())?,
            context: Some(Context::try_from(&value.context)?),
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn invalid(message: &str) -> Error {
    Error::InvalidReportInputs(message.into())
}
fn excess(bytes: usize, owned: usize, inline: usize) -> usize {
    if owned == usize::MAX {
        usize::MAX
    } else {
        bytes.saturating_add(owned.saturating_sub(inline))
    }
}
fn required<T>(value: &Option<T>) -> Result<&T> {
    value
        .as_ref()
        .ok_or_else(|| invalid("required native captured report field is missing"))
}
fn take<T>(value: Option<T>) -> Result<T> {
    value.ok_or_else(|| invalid("required native captured report field is missing"))
}
fn text(source: &str) -> Result<String> {
    let mut value = String::new();
    value
        .try_reserve_exact(source.len())
        .map_err(|_| invalid("captured report owner allocation failed"))?;
    if value.capacity() > source.len() {
        return Err(invalid("captured report owner exceeds admission"));
    }
    value.push_str(source);
    Ok(value)
}

/* === Tests === */
