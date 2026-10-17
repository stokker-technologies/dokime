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
pub(crate) use dokime_domain::model::execution::saved::EXECUTION_CONTEXT_MEDIA_TYPE as MEDIA_TYPE;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

use harmos::storage::StorageCursor;

// Local Imports

use super::{context::Context, records::Contracts as RecordContracts, telemetry::Contracts};
use crate::execution::context::captured::source::CapturedSource;
use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::identity::NodeId;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::execution::saved::EXECUTION_CONTEXT_SCHEMA_VERSION;
use crate::storage::objects::presentation as native;

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
    execution_id: String,
    lifecycle_id: String,
    lifecycle: Option<crate::storage::objects::execution::Execution>,
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
    pub(crate) fn into_source(self) -> Result<CapturedSource> {
        if self.schema_version != SCHEMA_VERSION {
            return Err(invalid("unsupported native captured report schema"));
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
                .map_err(crate::storage::Error::from)?,
            execution: self.execution.map(TryInto::try_into).transpose()?,
            execution_id: dokime_domain::model::identity::TestExecutionId::from_uuid(self.execution_id)
                .map_err(crate::storage::Error::from)?,
            lifecycle_id: self.lifecycle_id,
            lifecycle: take(self.lifecycle)?.try_into()?,
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
        value.validate_header()?;
        Ok(Self {
            schema_version: SCHEMA_VERSION,
            owner_node_id: text(value.owner_node_id.as_str())?,
            execution: value
                .execution
                .as_ref()
                .map(native::ExecutionRevision::try_from)
                .transpose()?,
            execution_id: text(value.execution_id.as_str())?,
            lifecycle_id: text(&value.lifecycle_id)?,
            lifecycle: Some(crate::storage::objects::execution::Execution::try_from(
                &value.lifecycle,
            )?),
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
fn take<T>(value: Option<T>) -> Result<T> {
    value.ok_or_else(|| invalid("required native captured report field is missing"))
}
fn text(source: &str) -> Result<String> {
    Ok(source.to_owned())
}

/* === Tests === */
