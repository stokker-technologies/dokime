/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub use crate::runtime::storage::context_records::ContextRecord;

use serde::{Deserialize, Serialize};

use super::execution_history::{ExecutionBrowseIssue, ExecutionCheckpoint};
use crate::{RuntimeHandle, runtime::Result};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionRecordQuery {
    pub after: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExecutionRecordPage {
    pub checkpoint: ExecutionCheckpoint,
    pub records: crate::storage::ReadRows<ContextRecord>,
    pub next: Option<String>,
    pub issues: Vec<ExecutionBrowseIssue>,
    pub issues_truncated: bool,
}

impl RuntimeHandle {
    pub async fn execution_checkpoint_records(
        &self, checkpoint: ExecutionCheckpoint, query: ExecutionRecordQuery,
    ) -> Result<harmos::storage::ReadBatch<ExecutionRecordPage>> {
        let view = self.checkpoint_execution(&checkpoint).await?;
        crate::runtime::storage::record_pages::read(&view, query).await
    }
}
