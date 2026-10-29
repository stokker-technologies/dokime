/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::collections::BTreeMap;

use super::executions::ExecutionStorage;
use crate::artifacts::ArtifactRecord;
use crate::artifacts::ArtifactStatus;
use crate::runtime::Result;
use dokime_domain::model::execution::saved::ExecutionOutput;
use dokime_domain::model::execution::saved::ExecutionOutputContent;
use dokime_domain::model::identity::ArtifactId;
use dokime_domain::model::identity::TestExecutionId;

const UUID_BYTES: usize = 36;

pub(super) async fn count(storage: &ExecutionStorage) -> Result<usize> {
    inspect(storage, |_, _| Ok(())).await
}

pub(super) async fn inspect(
    storage: &ExecutionStorage, mut visit: impl FnMut(&TestExecutionId, &ExecutionOutput) -> harmos::Result<()>,
) -> Result<usize> {
    let owners = storage.active.lock().await.values().cloned().collect::<Vec<_>>();
    let mut live = Vec::new();
    for owner in owners {
        let acquisition = owner.lock().await;
        for upload in acquisition.artifacts.values() {
            if let Some(output) = upload.descriptor() {
                live.push((acquisition.execution_id.clone(), output.clone()));
            }
        }
    }
    // Observe All Completed Uploads Through One Logical Durability Boundary.
    // Native Finish Has Already Saved Each File's Immutable Bytes.
    if let Some(through) = live.iter().map(|(_, output)| &output.file.position).max_by_key(|position| position.sequence)
        && !storage.storage.cursor_saved().covers(through)?
    {
        storage.storage.flush_until(through).await?;
    }
    let saved = storage.storage.cursor_saved();
    let mut seen = BTreeMap::new();
    let session = storage.storage.clone();
    let history = super::history::executions(&session, storage.storage.cursor_saved()).await?;
    for row in history.iter().flat_map(|page| page.iter()) {
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
    for (id, output) in live {
        if saved.covers(&output.file.position)?
            && let ExecutionOutputContent::Artifact { record, .. } = &output.content
            && observe(&mut seen, &id, record)?
        {
            visit(&id, &output)?;
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

pub(super) async fn records_at_in(
    storage: &ExecutionStorage, session: &crate::storage::Manager, at: &harmos::storage::StorageCursor,
) -> Result<harmos::storage::ReadBatch<Vec<ArtifactRecord>>> {
    let active = storage.active.lock().await;
    let count = active.len();
    let owners = harmos::storage::ReadBatch::new(()).try_map(|_| -> Result<_> {
        let mut owners = Vec::new();
        owners.try_reserve_exact(count).map_err(|_| invalid("artifact owner allocation refused"))?;
        owners.extend(active.values().cloned());
        Ok(owners)
    })?;
    drop(active);
    let mut live =
        harmos::storage::ReadBatch::new(()).try_map(|_| -> Result<_> { Ok(Vec::<(TestExecutionId, ExecutionOutput)>::new()) })?;
    for owner in owners.iter() {
        let acquisition = owner.lock().await;
        let count = acquisition.artifacts.values().filter(|upload| upload.descriptor().is_some()).count();
        let target = growth_capacity(live.capacity(), live.len(), count)?;
        live = live.try_map(|mut live| -> Result<_> {
            reserve_slots(&mut live, target)?;
            for upload in acquisition.artifacts.values() {
                if let Some(output) = upload.descriptor() {
                    live.push((acquisition.execution_id.clone(), output.clone()));
                }
            }
            Ok(live)
        })?;
    }
    drop(owners);
    let saved = at.clone();
    let history = super::history::executions(session, saved.clone()).await?;
    let mut records = harmos::storage::ReadBatch::new(()).try_map(|_| -> Result<_> {
        Ok(RetainedRecords {
            records: Vec::new(),
            seen: BTreeMap::new(),
        })
    })?;
    for row in history.iter().flat_map(|page| page.iter()) {
        let Some(seal) = &row.value.saved else { continue };
        for output in &seal.source.outputs {
            if let ExecutionOutputContent::Artifact { record, .. } = &output.content {
                records = retain_record(records, &row.value.test_execution_id, record)?;
            }
        }
    }
    for (id, output) in live.iter() {
        if saved.covers(&output.file.position)?
            && let ExecutionOutputContent::Artifact { record, .. } = &output.content
        {
            records = retain_record(records, id, record)?;
        }
    }
    records.try_map(|mut records| -> Result<_> {
        records.records.sort_unstable_by(|left, right| left.artifact_id.as_str().cmp(right.artifact_id.as_str()));
        Ok(records.records)
    })
}

struct RetainedRecords {
    records: Vec<ArtifactRecord>,
    seen: BTreeMap<ArtifactId, TestExecutionId>,
}

fn retain_record(
    records: harmos::storage::ReadBatch<RetainedRecords>, owner: &TestExecutionId, record: &ArtifactRecord,
) -> Result<harmos::storage::ReadBatch<RetainedRecords>> {
    validate_record(owner, record)?;
    if let Some(previous) = records.seen.get(&record.artifact_id) {
        if previous != owner {
            return Err(invalid("artifact identity belongs to multiple executions").into());
        }
        return Ok(records);
    }
    let target = growth_capacity(records.records.capacity(), records.records.len(), 1)?;
    records.try_map(|mut records| -> Result<_> {
        if observe(&mut records.seen, owner, record)? {
            reserve_slots(&mut records.records, target)?;
            records.records.push(record.clone());
        }
        Ok(records)
    })
}

fn growth_capacity(capacity: usize, length: usize, added: usize) -> Result<usize> {
    let required = length.checked_add(added).ok_or_else(|| invalid("artifact projection count overflows"))?;
    if required <= capacity {
        return Ok(capacity);
    }
    Ok(capacity.checked_mul(2).unwrap_or(required).max(required))
}

fn reserve_slots<T>(rows: &mut Vec<T>, target: usize) -> Result<()> {
    if target <= rows.capacity() {
        return Ok(());
    }
    rows.try_reserve_exact(target - rows.len()).map_err(|_| invalid("artifact projection allocation refused"))?;
    Ok(())
}

fn observe(seen: &mut BTreeMap<ArtifactId, TestExecutionId>, owner: &TestExecutionId, record: &ArtifactRecord) -> harmos::Result<bool> {
    validate_record(owner, record)?;
    if let Some(previous) = seen.get(&record.artifact_id) {
        if previous != owner {
            return Err(invalid("artifact identity belongs to multiple executions"));
        }
        return Ok(false);
    }
    seen.insert(record.artifact_id.clone(), owner.clone());
    Ok(true)
}

fn validate_record(owner: &TestExecutionId, record: &ArtifactRecord) -> harmos::Result<()> {
    if record.status == ArtifactStatus::Staged
        || record.scope.test_execution_id.as_ref() != Some(owner)
        || record.artifact_id.as_str().len() != UUID_BYTES
        || owner.as_str().len() != UUID_BYTES
    {
        return Err(invalid("artifact inventory attribution differs from its owner"));
    }
    Ok(())
}

fn invalid(reason: &'static str) -> harmos::Error {
    codec_error(crate::storage::Error::Invalid(reason))
}

fn codec_error(source: crate::storage::Error) -> harmos::Error {
    harmos::Error::backend("inspect native artifact inventory", harmos::BackendFailure::Corrupt, source)
}
