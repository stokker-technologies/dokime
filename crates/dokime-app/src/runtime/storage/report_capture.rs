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
use std::num::NonZeroUsize;

// Third Party Imports
use harmos::{
    MemorySize,
    storage::{ObjectVersion, QueryValue, StorageProductionPreparation},
};

// Local Imports
use super::execution::ExecutionView;
use crate::{
    Error, Result, RuntimeHandle,
    execution::{CycleView, TestExecution},
};
use dokime::core::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub async fn execution_snapshot(
        &self,
        execution: &dokime::core::TestExecutionId,
    ) -> Result<ExecutionView> {
        let identity = self
            .storage()
            .produce(bound(execution.memory_size())?, |_| Ok(execution.clone()))?
            .try_into_value()?;
        let runtime = self.clone();
        let operation = async move {
            let _order = runtime.execution_storage.cycle_save.lock().await;
            let source = runtime.snapshot_source(&identity).await?;
            let execution = source
                .test_executions
                .iter()
                .find(|member| member.test_execution_id == *identity)
                .ok_or_else(|| invalid("snapshot execution is outside its cycle"))?;
            runtime.snapshot_member(&source, execution).await
        };
        let bytes = std::mem::size_of_val(&operation)
            .checked_add(std::mem::size_of::<StorageProductionPreparation>())
            .ok_or_else(|| invalid("report capture task memory overflows"))?;
        let memory = self.storage().prepare_production(bound(bytes)?)?;
        tokio::spawn(async move {
            let _memory = memory;
            operation.await
        })
        .await?
    }

    async fn snapshot_source(&self, execution: &TestExecutionId) -> Result<QueryValue<CycleView>> {
        self.execution_storage
            .state_manager
            .state()
            .read(|state| {
                let id = &state
                    .executions
                    .get(execution)
                    .ok_or_else(|| invalid("report execution is not active"))?
                    .test_cycle_id;
                let header = state
                    .cycles
                    .get(id)
                    .ok_or_else(|| invalid("report cycle is not active"))?;
                let bytes = header.execution_ids.iter().try_fold(
                    header
                        .memory_size()
                        .saturating_add(std::mem::size_of::<CycleView>()),
                    |bytes, id| {
                        state
                            .executions
                            .get(id)
                            .and_then(|member| bytes.checked_add(member.memory_size()))
                            .ok_or_else(|| invalid("report cycle membership memory overflows"))
                    },
                )?;
                self.storage()
                    .produce(
                        bounded(bytes, self.config().storage_reads.decoded_bytes)?,
                        |_| {
                            state.cycle_view(id).ok_or(harmos::Error::InvalidInput {
                                field: "report cycle",
                                reason: "cycle disappeared",
                            })
                        },
                    )?
                    .try_into_value()
                    .map_err(Error::from)
            })
            .await?
            .value
    }

    async fn snapshot_member(
        &self,
        cycle: &CycleView,
        execution: &TestExecution,
    ) -> Result<ExecutionView> {
        let view = self
            .storage()
            .view_with_limits(self.config().storage_reads.execution)
            .await?;
        let limit = view.decoded_bytes()?;
        if let Some(inspection) = ExecutionView::open(
            self.storage(),
            view,
            &execution.test_execution_id,
            ObjectVersion::Latest,
            limit,
        )
        .await?
            && inspection.execution().saved.is_some()
        {
            if execution.saved.is_some() && inspection.execution() != execution {
                return Err(invalid("saved report member differs from frozen cycle"));
            }
            if execution.saved.is_none() {
                let expected = inspection.view().project_shared(
                    inspection.revision(),
                    bound(inspection.execution().memory_size())?,
                    |revision, _| {
                        let mut execution = revision.value.clone();
                        execution.saved = None;
                        Ok(execution)
                    },
                )?;
                if &*expected != execution {
                    return Err(invalid("saved snapshot differs from working execution"));
                }
            }
            return inspection.snapshot(self.storage()).await;
        }
        if execution.saved.is_some() {
            return Err(invalid("sealed cycle member has no saved execution"));
        }
        if let Some(snapshot) = self
            .execution_storage
            .retained_report_view(&execution.test_execution_id)
            .await
        {
            if snapshot.execution() != execution {
                return Err(invalid("retained snapshot differs from working execution"));
            }
            return Ok(snapshot);
        }
        let inputs = self
            .execution_storage
            .prepare_capture(cycle, execution, false)
            .await?;
        self.capture_execution_view(cycle, &inputs).await
    }

    pub(super) async fn report_appearance(
        &self,
        limit: NonZeroUsize,
    ) -> Result<QueryValue<crate::installed_plugins::RuntimeAppearanceFrame>> {
        let memory = self.storage().prepare_production(limit)?;
        let appearance = self
            .capture_appearance(limit.get())
            .map_err(|_| invalid("report appearance exceeds its source budget"))?;
        let mut frame = appearance.frame;
        if let Some(asset) = appearance.logo {
            let url = super::presentation::capture_logo(self.storage(), &asset).await?;
            drop(asset);
            bounded(frame.memory_size().saturating_add(url.memory_size()), limit)?;
            if let Some(logo) = &mut frame.logo {
                logo.url = (*url).clone();
            }
        }
        Ok(memory.finish(frame)?.try_into_value()?)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn bound(bytes: usize) -> Result<NonZeroUsize> {
    NonZeroUsize::new(bytes)
        .filter(|bytes| bytes.get() != usize::MAX)
        .ok_or_else(|| invalid("report capture memory overflows"))
}
fn bounded(bytes: usize, limit: NonZeroUsize) -> Result<NonZeroUsize> {
    let bytes = bound(bytes)?;
    if bytes > limit {
        return Err(exhausted().into());
    }
    Ok(bytes)
}
fn exhausted() -> harmos::Error {
    harmos::Error::ResourceExhausted {
        resource: "cycle report capture",
    }
}
fn invalid(message: &str) -> Error {
    Error::InvalidReportInputs(message.into())
}

/* === Tests === */
