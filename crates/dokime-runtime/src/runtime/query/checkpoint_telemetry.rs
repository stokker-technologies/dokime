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
pub use crate::storage::telemetry::TelemetryEntry as ExecutionTelemetryRow;

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use crate::{RuntimeHandle, runtime::Result};
use dokime_domain::model::identity::Identifier;
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
    pub contract: crate::telemetry::TelemetryContract,
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

impl RuntimeHandle {
    pub async fn execution_checkpoint_telemetry(
        &self, checkpoint: ExecutionCheckpoint, stream: &Identifier, query: ExecutionTelemetryQuery,
    ) -> Result<ExecutionTelemetryPage> {
        let view = self.checkpoint_execution(&checkpoint).await?;
        crate::runtime::storage::telemetry_pages::read(&view, stream, query).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
