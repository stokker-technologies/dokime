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
use std::sync::Arc;
// Third Party Imports
// Local Imports
use super::executions::ExecutionStorage;
use crate::{runtime::Error, runtime::Result};
use crate::{core::TestCycleId, storage::objects::cycle::Cycle};

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
                    let cycle = state.cycle_view(&id).ok_or(harmos::Error::InvalidInput {
                        field: "cycle membership",
                        reason: "execution disappeared",
                    })?;
                    Ok(Some((cycle, native_cycle(header)?)))
                })
                .await?
                .value?;
            let Some((cycle, header)) = selected else {
                return Ok(());
            };
            for execution in &cycle.test_executions {
                if execution.saved.is_some() {
                    continue;
                }
                if let Some(owner) = storage.acquisition(&execution.test_execution_id).await {
                    if owner.lock().await.execution.saved.is_none() {
                        storage
                            .checkpoint_targets(
                                &cycle,
                                std::slice::from_ref(&execution.test_execution_id),
                            )
                            .await?;
                    }
                } else {
                    let native =
                        crate::storage::objects::execution::Execution::try_from(execution)?;
                    storage.storage.write_object(&native).await?;
                }
            }
            let receipt = storage.storage.write_object(&header).await?;
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
                .read(|state| state.cycles.get(&id).map(native_cycle).transpose())
                .await?
                .value?;
            if let Some(native) = native {
                let receipt = storage.storage.write_object(&native).await?;
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
            .read(|state| state.cycles.keys().cloned().collect::<Vec<_>>())
            .await?
            .value;
        for id in ids {
            self.save_cycle(id).await?;
        }
        self.storage.flush().await?;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn native_cycle(header: &crate::execution::TestCycle) -> Result<Cycle> {
    Ok(Cycle::try_from(header)?)
}

/* === Tests === */
