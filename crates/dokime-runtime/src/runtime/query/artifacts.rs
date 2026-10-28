/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod read;

pub use crate::execution_runtime::outputs::{ExecutionFileContentKind, ExecutionFileStreamState};
pub use dokime_domain::model::execution::RecordScope;
pub use read::RuntimeExecutionFileRead;
pub(crate) use read::execution_file_frame;

use serde::{Deserialize, Serialize};

use crate::artifacts::ArtifactRecord;
use crate::queries::{RuntimeResolution, RuntimeWindow, reduction::reduce_timestamped_items};
use dokime_domain::model::execution::RecordScope as DomainRecordScope;
use dokime_domain::model::time::Timestamp;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeArtifactFrame {
    pub artifacts: Vec<ArtifactRecord>,
    pub total_artifacts: usize,
    pub returned_artifacts: usize,
    pub truncated: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeExecutionFileRecord {
    pub artifact_id: String,
    pub scope: RecordScope,
    pub filename: String,
    pub content_type: Option<String>,
    pub content_kind: ExecutionFileContentKind,
    pub stream_state: ExecutionFileStreamState,
    pub size_bytes: u64,
    pub updated_at: Timestamp,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeExecutionFileFrame {
    pub files: Vec<RuntimeExecutionFileRecord>,
    pub total_files: usize,
    pub returned_files: usize,
    pub truncated: bool,
    pub error: Option<String>,
}

pub(crate) fn artifact_frame<'a>(
    artifacts: impl Iterator<Item = &'a ArtifactRecord>, scope: Option<&DomainRecordScope>, window: &RuntimeWindow,
    resolution: &RuntimeResolution,
) -> RuntimeArtifactFrame {
    let artifacts = artifacts.filter(|artifact| scope.is_none_or(|scope| artifact.scope.is_visible_in(scope))).cloned().collect::<Vec<_>>();
    let artifacts = reduce_timestamped_items(artifacts, window, resolution, |artifact| &artifact.created_at, |_| None);
    let returned_artifacts = artifacts.returned.len();

    RuntimeArtifactFrame {
        artifacts: artifacts.returned,
        total_artifacts: artifacts.total,
        returned_artifacts,
        truncated: returned_artifacts < artifacts.total,
    }
}
