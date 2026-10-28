/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::sync::Arc;

use super::executions::ExecutionStorage;
use crate::storage::objects::cycle::Cycle;
use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::identity::TestCycleId;

impl ExecutionStorage {
    pub(crate) async fn save_cycle(self: &Arc<Self>, id: TestCycleId) -> Result<()> {
        let storage = self.clone();
        tokio::spawn(async move {
            let _order = storage.cycle_save.lock().await;
            if sealed(&storage.storage, &id).await? {
                return Ok(());
            }
            let selected = storage
                .state_manager
                .state()
                .read(|state| {
                    let Some(header) = state.cycles.get(&id) else {
                        return Ok::<_, Error>(None);
                    };
                    if header.saved.is_some() {
                        return Ok(None);
                    }
                    let cycle = state.cycle_view(&id).ok_or(harmos::Error::InvalidInput {
                        field: "cycle membership",
                        reason: "execution disappeared",
                    })?;
                    Ok(Some(cycle))
                })
                .await?
                .value?;
            let Some(cycle) = selected else {
                return Ok(());
            };
            let mut pending = Vec::new();
            pending
                .try_reserve_exact(cycle.test_executions.len())
                .map_err(|_| harmos::Error::ResourceExhausted { resource: "cycle checkpoints" })?;
            for execution in &cycle.test_executions {
                if execution.saved.is_some() {
                    continue;
                }
                if let Some(owner) = storage.acquisition(&execution.test_execution_id).await {
                    if owner.lock().await.execution.saved.is_none() {
                        storage.checkpoint_targets(&cycle, std::slice::from_ref(&execution.test_execution_id)).await?;
                    }
                } else {
                    pending.push(execution);
                }
            }
            save_members(&storage.storage, &pending).await?;
            let header = native_cycle_view(cycle)?;
            let receipt = storage.storage.write_object(&header).await?;
            drop(header);
            storage.storage.flush_until(&receipt.position).await?;
            Ok(())
        })
        .await?
    }

    pub(crate) async fn save_cycle_header(self: &Arc<Self>, id: TestCycleId) -> Result<()> {
        let storage = self.clone();
        tokio::spawn(async move {
            let _order = storage.cycle_save.lock().await;
            if sealed(&storage.storage, &id).await? {
                return Ok(());
            }
            let native = storage
                .state_manager
                .state()
                .read(|state| {
                    let Some(header) = state.cycles.get(&id) else {
                        return Ok::<_, Error>(None);
                    };
                    if header.saved.is_some() {
                        return Ok(None);
                    }
                    native_cycle(header).map(Some)
                })
                .await?
                .value?;
            if let Some(native) = native {
                let receipt = storage.storage.write_object(&native).await?;
                drop(native);
                storage.storage.flush_until(&receipt.position).await?;
            }
            Ok(())
        })
        .await?
    }

    pub(crate) async fn checkpoint_shutdown(self: &Arc<Self>) -> Result<()> {
        let ids = self.state_manager.state().read(|state| state.cycles.keys().cloned().collect::<Vec<_>>()).await?.value;
        for id in ids {
            self.save_cycle(id).await?;
        }
        self.storage.flush().await?;
        Ok(())
    }
}

async fn save_members(
    storage: &crate::storage::Manager, mut members: &[&dokime_domain::model::execution::result::TestExecution],
) -> Result<()> {
    let allowance = storage.settings().max_batch_bytes();
    let mut capacity = 1;
    let mut largest = 1;
    while !members.is_empty() {
        let count = capacity.min(members.len());
        let (selected, remaining) = members.split_at(count);
        let count = std::num::NonZeroUsize::new(count).expect("nonempty checkpoint group");
        let mut conversion_error = None;
        let mut rejected = false;
        let admitted = storage
            .write_object_group(allowance, count, |group| {
                for member in selected {
                    let native = match crate::storage::objects::execution::Execution::try_from(*member) {
                        Ok(native) => native,
                        Err(error) => {
                            conversion_error = Some(error);
                            return Err(harmos::storage::Error::InvalidInput {
                                field: "cycle checkpoint",
                                reason: "native execution conversion failed",
                            });
                        }
                    };
                    let before = group.remaining_bytes();
                    if let Err(error) = group.write(&native) {
                        rejected = true;
                        return Err(error);
                    }
                    largest = largest.max(before - group.remaining_bytes());
                }
                Ok(())
            })
            .await;
        if let Some(error) = conversion_error {
            return Err(error.into());
        }
        if rejected && count.get() > 1 {
            // A Rejected Builder Publishes Nothing. Split Within The Same Configured
            // Allowance; Individual Objects Still Undergo Full Storage Validation.
            capacity = (count.get() / 2).min(allowance.get() / largest).max(1);
            continue;
        }
        // Admission, Persistence And Single-Object Failures Are Never Retried Here.
        admitted?;
        members = remaining;
        capacity = (allowance.get() / largest).max(1);
    }
    Ok(())
}

async fn sealed(storage: &crate::storage::Manager, id: &TestCycleId) -> Result<bool> {
    // Native Publication Precedes The State Mirror; Both Are Immutable Once Sealed.
    let revision = crate::storage::objects::cycle::read(&storage.clone(), id, harmos::storage::ObjectVersion::Latest).await?;
    Ok(revision.is_some_and(|revision| revision.value.saved.is_some()))
}

fn native_cycle(header: &dokime_domain::model::execution::result::TestCycle) -> Result<Cycle> {
    Ok(Cycle::try_from(header)?)
}

fn native_cycle_view(mut source: dokime_domain::model::execution::result::CycleView) -> Result<Cycle> {
    let executions = std::mem::take(&mut source.test_executions);
    let mut ids = Vec::new();
    ids.try_reserve_exact(executions.len()).map_err(|_| harmos::Error::ResourceExhausted { resource: "cycle membership" })?;
    for execution in executions {
        ids.push(execution.test_execution_id);
    }
    Ok(Cycle::from_view(&source, &ids)?)
}
