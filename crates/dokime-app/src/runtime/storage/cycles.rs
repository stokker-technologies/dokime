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
use std::{num::NonZeroUsize, sync::Arc};
// Third Party Imports
use harmos::MemorySize;
// Local Imports
use super::executions::ExecutionStorage;
use crate::{Error, Result};
use dokime::{core::TestCycleId, storage::objects::cycle::Cycle};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ExecutionStorage {
    pub(crate) async fn save_cycle(self: &Arc<Self>, id: TestCycleId) -> Result<()> {
        let storage = self.clone();
        tokio::spawn(async move {
            let _order = storage.cycle_save.lock().await;
            let selected = storage
                .state_manager
                .state()
                .read(|state| {
                    let Some(header) = state.cycles.get(&id) else {
                        return Ok::<_, Error>(None);
                    };
                    let bytes = header.execution_ids.iter().try_fold(
                        header
                            .memory_size()
                            .saturating_add(std::mem::size_of::<crate::execution::CycleView>()),
                        |bytes, id| {
                            state
                                .executions
                                .get(id)
                                .and_then(|execution| bytes.checked_add(execution.memory_size()))
                                .ok_or_else(invalid)
                        },
                    )?;
                    let cycle = storage.storage.produce(allowance(bytes)?, |_| {
                        state.cycle_view(&id).ok_or(harmos::Error::InvalidInput {
                            field: "cycle membership",
                            reason: "execution disappeared",
                        })
                    })?;
                    let native = storage.storage.produce(
                        allowance(Cycle::encoding_memory(header)?)?,
                        |_| {
                            Cycle::try_from(header).map_err(|source| {
                                harmos::Error::backend(
                                    "encode native cycle",
                                    harmos::BackendFailure::Permanent,
                                    source,
                                )
                            })
                        },
                    )?;
                    Ok(Some((cycle, native)))
                })
                .await?
                .value?;
            let Some((cycle, header)) = selected else {
                return Ok(());
            };
            for execution in &cycle.value().test_executions {
                if execution.saved.is_some() {
                    continue;
                }
                if let Some(owner) = storage.acquisition(&execution.test_execution_id).await {
                    if owner.lock().await.execution.saved.is_none() {
                        storage
                            .checkpoint_targets(
                                cycle.value(),
                                std::slice::from_ref(&execution.test_execution_id),
                            )
                            .await?;
                    }
                } else {
                    let native = storage.storage.produce(
                        allowance(
                            dokime::storage::objects::execution::Execution::encoding_memory(
                                execution,
                            )?,
                        )?,
                        |_| {
                            dokime::storage::objects::execution::Execution::try_from(execution)
                                .map_err(|source| {
                                    harmos::Error::backend(
                                        "encode working execution",
                                        harmos::BackendFailure::Permanent,
                                        source,
                                    )
                                })
                        },
                    )?;
                    storage.storage.write_object(native.value()).await?;
                }
            }
            let receipt = storage.storage.write_object(header.value()).await?;
            storage.storage.flush_until(&receipt.position).await?;
            Ok(())
        })
        .await?
    }

    pub(crate) async fn save_cycle_header(self: &Arc<Self>, id: TestCycleId) -> Result<()> {
        let storage = self.clone();
        tokio::spawn(async move {
            let _order = storage.cycle_save.lock().await;
            let native = storage
                .state_manager
                .state()
                .read(|state| {
                    let Some(header) = state.cycles.get(&id) else {
                        return Ok::<_, Error>(None);
                    };
                    storage
                        .storage
                        .produce(allowance(Cycle::encoding_memory(header)?)?, |_| {
                            Cycle::try_from(header).map_err(|source| {
                                harmos::Error::backend(
                                    "encode native cycle",
                                    harmos::BackendFailure::Permanent,
                                    source,
                                )
                            })
                        })
                        .map(Some)
                        .map_err(Error::from)
                })
                .await?
                .value?;
            if let Some(native) = native {
                let receipt = storage.storage.write_object(native.value()).await?;
                storage.storage.flush_until(&receipt.position).await?;
            }
            Ok(())
        })
        .await?
    }

    pub(crate) async fn checkpoint_shutdown(self: &Arc<Self>) -> Result<()> {
        let ids = self
            .state_manager
            .state()
            .read(|state| {
                let bytes = state
                    .cycles
                    .keys()
                    .try_fold(std::mem::size_of::<Vec<TestCycleId>>(), |bytes, id| {
                        bytes.checked_add(id.memory_size()).ok_or_else(invalid)
                    })?;
                self.storage
                    .produce(allowance(bytes)?, |_| {
                        Ok(state.cycles.keys().cloned().collect::<Vec<_>>())
                    })
                    .and_then(|value| value.try_into_value())
                    .map_err(Error::from)
            })
            .await?
            .value?;
        for id in ids.iter() {
            self.save_cycle(id.clone()).await?;
        }
        self.storage.flush().await?;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn invalid() -> Error {
    Error::Storage(dokime::storage::Error::Invalid(
        "cycle membership exceeds its source bound",
    ))
}
fn allowance(bytes: usize) -> Result<NonZeroUsize> {
    NonZeroUsize::new(bytes)
        .filter(|bytes| bytes.get() != usize::MAX)
        .ok_or_else(invalid)
}

/* === Tests === */
