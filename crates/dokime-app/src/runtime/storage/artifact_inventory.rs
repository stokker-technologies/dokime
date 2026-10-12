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
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

// Third Party Imports
use harmos::{
    MemorySize,
    storage::{ObjectComparison, ObjectQuery, ObjectScalar, QueryOptions, QueryValue, ReadLimits},
};
use tokio::sync::Mutex;

// Local Imports
use super::{
    execution::check_seal_position,
    executions::{Acquisition, ExecutionStorage},
};
use crate::{Error, Result};
use dokime::{
    artifacts::{ArtifactRecord, ArtifactStatus},
    core::{ArtifactId, TestExecutionId},
    execution::saved::{ExecutionOutput, ExecutionOutputContent},
    storage::objects::execution::Execution,
};

/* === Constants === */
const MAX_SOURCES: usize = 10_000;
const UUID_BYTES: usize = 36;
const FRAME_BYTES: usize = 4 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Owners(Vec<Arc<Mutex<Acquisition>>>);

struct Records {
    values: Vec<ArtifactRecord>,
    bytes: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl MemorySize for Owners {
    fn memory_size(&self) -> usize {
        // The Shared Acquisition Already Retains Its Original Graph And Producer Charges
        std::mem::size_of::<Self>().saturating_add(
            self.0
                .capacity()
                .saturating_mul(std::mem::size_of::<Arc<Mutex<Acquisition>>>()),
        )
    }
}

/* === Public Functions === */
pub(super) async fn count(storage: &ExecutionStorage) -> Result<usize> {
    inspect(storage, |_, _| Ok(())).await
}

pub(super) async fn inspect(
    storage: &ExecutionStorage,
    visit: impl FnMut(&TestExecutionId, &ExecutionOutput) -> harmos::Result<()>,
) -> Result<usize> {
    let count = project(storage, visit, |count, _| Ok(count as u64)).await?;
    Ok(*count as usize)
}

pub(super) async fn project_records<T: MemorySize>(
    storage: &ExecutionStorage,
    scan_limit: usize,
    map: impl FnOnce(Vec<ArtifactRecord>, NonZeroUsize) -> harmos::Result<T>,
) -> Result<QueryValue<T>> {
    let limit = scan_limit.min(MAX_SOURCES);
    let bytes = std::mem::size_of::<std::sync::Mutex<Records>>()
        .checked_add(
            limit
                .checked_mul(std::mem::size_of::<ArtifactRecord>())
                .ok_or_else(|| exhausted("artifact inventory record slots"))?,
        )
        .ok_or_else(|| exhausted("artifact inventory record slots"))?;
    if bytes > FRAME_BYTES {
        return Err(exhausted("artifact inventory record slots").into());
    }
    // Mutable Collection Keeps Its Original Full Claim Until The Final Projection.
    let _memory = storage
        .storage
        .prepare_production(NonZeroUsize::new(FRAME_BYTES).unwrap())?;
    let mut values = Vec::new();
    values
        .try_reserve_exact(limit)
        .map_err(|_| exhausted("artifact inventory record slots"))?;
    if values.capacity() > limit {
        return Err(exhausted("artifact inventory record slot capacity").into());
    }
    let records = std::sync::Mutex::new(Records { values, bytes });
    project(
        storage,
        |_, output| {
            let ExecutionOutputContent::Artifact { record, .. } = &output.content else {
                unreachable!("inventory visits only artifact descriptors");
            };
            let mut records = records
                .try_lock()
                .map_err(|_| exhausted("artifact inventory collection busy"))?;
            if records.values.len() == limit {
                return Err(exhausted("artifact inventory record limit"));
            }
            let owned = record.as_ref().memory_size();
            if owned == usize::MAX {
                return Err(exhausted("artifact inventory record memory"));
            }
            records.bytes = records
                .bytes
                .checked_add(owned.saturating_sub(std::mem::size_of::<ArtifactRecord>()))
                .filter(|bytes| *bytes <= FRAME_BYTES)
                .ok_or_else(|| exhausted("artifact inventory record memory"))?;
            records.values.push(record.as_ref().clone());
            Ok(())
        },
        |_, allowance| {
            let mut records = records
                .try_lock()
                .map_err(|_| exhausted("artifact inventory collection busy"))?;
            records.values.sort_unstable_by(|left, right| {
                left.artifact_id.as_str().cmp(right.artifact_id.as_str())
            });
            map(std::mem::take(&mut records.values), allowance)
        },
    )
    .await
}

async fn project<T: MemorySize>(
    storage: &ExecutionStorage,
    mut visit: impl FnMut(&TestExecutionId, &ExecutionOutput) -> harmos::Result<()>,
    finish: impl FnOnce(usize, NonZeroUsize) -> harmos::Result<T>,
) -> Result<QueryValue<T>> {
    let allowance = NonZeroUsize::new(FRAME_BYTES).unwrap();
    let mut query = ObjectQuery::<Execution>::new().project(vec![])?;
    query.filter(
        harmos::Identifier::new("finished_at").unwrap(),
        ObjectComparison::NotEqual,
        ObjectScalar::Null,
    )?;
    query.options = QueryOptions {
        page: Some(ReadLimits {
            rows: 64.try_into().unwrap(),
            bytes: (64 * 1024).try_into().unwrap(),
        }),
        decoded_bytes: Some(allowance),
        ..QueryOptions::default()
    };
    let mut anchor = storage
        .storage
        .query(ObjectQuery::<Execution>::new())
        .await?;
    anchor.retain_view()?;
    let view = anchor
        .view()
        .ok_or_else(|| invalid("artifact count has no original view"))?
        .clone();
    let mut reader = storage.storage.query_in(&view, query)?;
    let seed = storage
        .storage
        .produce(
            NonZeroUsize::new(std::mem::size_of::<bool>()).unwrap(),
            |_| Ok(false),
        )?
        .try_into_value()?;
    let owners = anchor.project(&seed, allowance, |_, limit| {
        let active = storage
            .active
            .try_lock()
            .map_err(|_| exhausted("artifact owner registry busy"))?;
        if active.len() > MAX_SOURCES {
            return Err(exhausted("artifact owner inventory source bound"));
        }
        let bytes = std::mem::size_of::<Owners>()
            .checked_add(
                active
                    .len()
                    .checked_mul(std::mem::size_of::<Arc<Mutex<Acquisition>>>())
                    .ok_or_else(|| exhausted("artifact owner slots"))?,
            )
            .ok_or_else(|| exhausted("artifact owner slots"))?;
        if bytes > limit.get() {
            return Err(exhausted("artifact owner projection"));
        }
        let mut owners = Vec::new();
        owners
            .try_reserve_exact(active.len())
            .map_err(|_| exhausted("artifact owner slots"))?;
        if owners.capacity() > active.len() {
            return Err(exhausted("artifact owner slot capacity"));
        }
        owners.extend(active.values().cloned());
        Ok(Owners(owners))
    })?;
    // One Explicit Claim Covers All Mutable Identity Nodes Until The Count Is Finished
    let bytes = harmos::btree_map_memory::<ArtifactId, TestExecutionId>(MAX_SOURCES)
        .checked_add(MAX_SOURCES * 2 * UUID_BYTES)
        .and_then(NonZeroUsize::new)
        .ok_or_else(|| invalid("artifact identity inventory memory overflows"))?;
    let _identities = storage.storage.prepare_production(bytes)?;
    let mut seen = BTreeMap::new();
    let mut scanned = 0usize;
    while let Some(page) = reader.next_page().await? {
        for selected in page.iter() {
            scanned = scanned
                .checked_add(1)
                .filter(|count| *count <= MAX_SOURCES)
                .ok_or_else(|| exhausted("artifact execution source bound"))?;
            let row = storage
                .storage
                .read_object_in::<Execution>(
                    &view,
                    selected.item.as_str(),
                    harmos::storage::ObjectVersion::Exact(selected.version.clone()),
                    allowance,
                )
                .await?
                .ok_or_else(|| invalid("artifact execution revision disappeared"))?;
            let _count = view.project_owned(row, anchor.decoded_bytes(), |row, limit| {
                let saved = view.saved()?;
                row.version.check_position(&row.position)?;
                if !saved.covers(&row.position)? {
                    return Err(invalid("artifact execution exceeds its original view"));
                }
                if !row.value.is_sealed() {
                    return Ok(seen.len() as u64);
                }
                // Consume The Native Row Under The Execution Profile; Artifact
                // Output Pages Keep Their Smaller Independent Frame Bound.
                let bytes = row.value.projection_memory().map_err(codec_error)?;
                if bytes > limit.get() {
                    return Err(exhausted("artifact execution projection"));
                }
                let execution = row.value.into_execution(limit).map_err(codec_error)?;
                if let Some(seal) = &execution.saved {
                    check_seal_position(&row.position, &seal.dependencies_saved)
                        .map_err(query_error)?;
                    for output in &seal.source.outputs {
                        if let ExecutionOutputContent::Artifact { record, .. } = &output.content
                            && observe(&mut seen, &execution.test_execution_id, record)?
                        {
                            visit(&execution.test_execution_id, output)?;
                        }
                    }
                }
                Ok(seen.len() as u64)
            })?;
        }
    }
    // Wait For One Owner Within The Original Query Deadline
    let mut uploads = 0usize;
    for owner in &owners.0 {
        let work = view.begin_computation()?;
        let acquisition = tokio::time::timeout_at(work.deadline(), owner.lock())
            .await
            .map_err(|_| exhausted("artifact acquisition deadline"))?;
        work.check()?;
        drop(work);
        let _count = anchor.project(&owners, allowance, |_, _| {
            let saved = view.saved()?;
            uploads = uploads
                .checked_add(acquisition.artifacts.len())
                .filter(|count| *count <= MAX_SOURCES)
                .ok_or_else(|| exhausted("artifact upload inventory source bound"))?;
            for upload in acquisition.artifacts.values() {
                if let Some(output) = upload.descriptor()
                    && saved.covers(&output.file.position)?
                    && let ExecutionOutputContent::Artifact { record, .. } = &output.content
                    && observe(&mut seen, &acquisition.execution_id, record)?
                {
                    visit(&acquisition.execution_id, output)?;
                }
            }
            Ok(seen.len() as u64)
        })?;
    }
    Ok(anchor.project(&owners, allowance, |_, limit| finish(seen.len(), limit))?)
}

/* === Private Functions === */
fn observe(
    seen: &mut BTreeMap<ArtifactId, TestExecutionId>,
    owner: &TestExecutionId,
    record: &ArtifactRecord,
) -> harmos::Result<bool> {
    if record.status == ArtifactStatus::Staged
        || record.scope.test_execution_id.as_ref() != Some(owner)
        || record.artifact_id.as_str().len() != UUID_BYTES
        || owner.as_str().len() != UUID_BYTES
    {
        return Err(invalid(
            "artifact inventory attribution differs from its owner",
        ));
    }
    if let Some(previous) = seen.get(&record.artifact_id) {
        if previous != owner {
            return Err(invalid("artifact identity belongs to multiple executions"));
        }
        return Ok(false);
    }
    if seen.len() == MAX_SOURCES {
        return Err(exhausted("artifact identity inventory source bound"));
    }
    seen.insert(record.artifact_id.clone(), owner.clone());
    Ok(true)
}

fn exhausted(resource: &'static str) -> harmos::Error {
    harmos::Error::ResourceExhausted { resource }
}

fn invalid(reason: &'static str) -> harmos::Error {
    codec_error(dokime::storage::Error::Invalid(reason))
}

fn codec_error(source: dokime::storage::Error) -> harmos::Error {
    harmos::Error::backend(
        "inspect native artifact inventory",
        harmos::BackendFailure::Corrupt,
        source,
    )
}

fn query_error(source: Error) -> harmos::Error {
    harmos::Error::backend(
        "verify artifact execution seal",
        harmos::BackendFailure::Corrupt,
        source,
    )
}

/* === Tests === */
