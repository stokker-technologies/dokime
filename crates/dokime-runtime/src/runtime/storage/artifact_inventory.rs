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

// Local Imports
use super::executions::ExecutionStorage;
use crate::artifacts::ArtifactRecord;
use crate::artifacts::ArtifactStatus;
use crate::runtime::Result;
use dokime_domain::model::execution::saved::ExecutionOutput;
use dokime_domain::model::execution::saved::ExecutionOutputContent;
use dokime_domain::model::identity::ArtifactId;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */
const UUID_BYTES: usize = 36;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) async fn count(storage: &ExecutionStorage) -> Result<usize> {
    inspect(storage, |_, _| Ok(())).await
}

pub(super) async fn inspect(
    storage: &ExecutionStorage, mut visit: impl FnMut(&TestExecutionId, &ExecutionOutput) -> harmos::Result<()>,
) -> Result<usize> {
    let saved = storage.storage.cursor_saved();
    let mut seen = BTreeMap::new();
    for row in super::history::executions(&storage.storage).await? {
        let Some(seal) = &row.value.saved else {
            continue;
        };
        for output in &seal.source.outputs {
            if let ExecutionOutputContent::Artifact { record, .. } = &output.content
                && observe(&mut seen, &row.value.test_execution_id, record)?
            {
                visit(&row.value.test_execution_id, output)?;
            }
        }
    }
    let owners = storage.active.lock().await.values().cloned().collect::<Vec<_>>();
    for owner in owners {
        let acquisition = owner.lock().await;
        for upload in acquisition.artifacts.values() {
            if let Some(output) = upload.descriptor()
                && saved.covers(&output.file.position)?
                && let ExecutionOutputContent::Artifact { record, .. } = &output.content
                && observe(&mut seen, &acquisition.execution_id, record)?
            {
                visit(&acquisition.execution_id, output)?;
            }
        }
    }
    Ok(seen.len())
}

pub(super) async fn records(storage: &ExecutionStorage) -> Result<Vec<ArtifactRecord>> {
    let mut records = Vec::new();
    inspect(storage, |_, output| {
        if let ExecutionOutputContent::Artifact { record, .. } = &output.content {
            records.push(record.as_ref().clone());
        }
        Ok(())
    })
    .await?;
    records.sort_unstable_by(|left, right| left.artifact_id.as_str().cmp(right.artifact_id.as_str()));
    Ok(records)
}

/* === Private Functions === */
fn observe(seen: &mut BTreeMap<ArtifactId, TestExecutionId>, owner: &TestExecutionId, record: &ArtifactRecord) -> harmos::Result<bool> {
    if record.status == ArtifactStatus::Staged
        || record.scope.test_execution_id.as_ref() != Some(owner)
        || record.artifact_id.as_str().len() != UUID_BYTES
        || owner.as_str().len() != UUID_BYTES
    {
        return Err(invalid("artifact inventory attribution differs from its owner"));
    }
    if let Some(previous) = seen.get(&record.artifact_id) {
        if previous != owner {
            return Err(invalid("artifact identity belongs to multiple executions"));
        }
        return Ok(false);
    }
    seen.insert(record.artifact_id.clone(), owner.clone());
    Ok(true)
}

fn invalid(reason: &'static str) -> harmos::Error {
    codec_error(crate::storage::Error::Invalid(reason))
}

fn codec_error(source: crate::storage::Error) -> harmos::Error {
    harmos::Error::backend("inspect native artifact inventory", harmos::BackendFailure::Corrupt, source)
}

/* === Tests === */
