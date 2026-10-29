/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub use crate::storage::telemetry::TelemetryEntry as ExecutionTelemetryRow;

use serde::{Deserialize, Serialize};

use super::execution_history::{ExecutionBrowseIssue, ExecutionCheckpoint};
use crate::{RuntimeHandle, runtime::Result};
use dokime_domain::model::identity::Identifier;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionTelemetryQuery {
    pub after: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionTelemetryTable {
    pub table: String,
    pub contract: crate::telemetry::TelemetryContract,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionTelemetryPage {
    pub checkpoint: ExecutionCheckpoint,
    pub table: String,
    pub rows: crate::storage::ReadRows<ExecutionTelemetryRow>,
    pub next: Option<String>,
    pub issues: Vec<ExecutionBrowseIssue>,
    pub issues_truncated: bool,
}

impl RuntimeHandle {
    pub async fn execution_checkpoint_telemetry(
        &self, checkpoint: ExecutionCheckpoint, stream: &Identifier, query: ExecutionTelemetryQuery,
    ) -> Result<harmos::storage::ReadBatch<ExecutionTelemetryPage>> {
        let view = self.checkpoint_execution(&checkpoint).await?;
        crate::runtime::storage::telemetry_pages::read(&view, stream, query, self.schema_settings()).await
    }
}
