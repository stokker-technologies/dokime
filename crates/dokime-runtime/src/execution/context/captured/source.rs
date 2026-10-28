/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;

use harmos::storage::StorageCursor;

use super::ExecutionContext;
use crate::records::RecordContract;
use crate::telemetry::contract::TelemetryContract;
use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::execution::saved::ExecutionOutput;
use dokime_domain::model::execution::saved::ExecutionRevision;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::NodeId;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CapturedSource {
    pub report_defaults: crate::config::ReportDefaults,
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
