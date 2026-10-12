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
use harmos::storage::{ObjectVersion, QueryLimits, QueryValue, StorageProductionPreparation};

// Local Imports
use super::executions::{Acquisition, ExecutionStorage};
use crate::execution::notes::{NOTE_CHECKPOINT_LIMIT, NOTE_SCAN_LIMIT};
use crate::execution::{CycleView, OperatorNote};
use crate::{Error, Result};
use dokime::core::{RecordScope, SessionId, TestExecutionId};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct AnnotationAdmission {
    _guards: Vec<tokio::sync::OwnedMutexGuard<()>>,
    _owners: Vec<Arc<tokio::sync::Mutex<Acquisition>>>,
    _memory: StorageProductionPreparation,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ExecutionStorage {
    pub(crate) async fn admit_annotations(
        &self,
        scope: &RecordScope,
    ) -> Result<AnnotationAdmission> {
        // Snapshot Handles In Key Order; Release The Registry Before Awaiting Owners
        let (owners, memory) = {
            let active = self.active.lock().await;
            if active.len() > NOTE_SCAN_LIMIT {
                return Err(annotation_limit("annotation owner scan exceeds budget"));
            }
            let slots = std::mem::size_of::<Arc<tokio::sync::Mutex<Acquisition>>>()
                + std::mem::size_of::<(TestExecutionId, Arc<tokio::sync::Mutex<()>>)>()
                + std::mem::size_of::<tokio::sync::OwnedMutexGuard<()>>();
            let mut bytes = active
                .len()
                .checked_mul(slots)
                .and_then(|bytes| bytes.checked_add(3 * std::mem::size_of::<Vec<()>>()))
                .ok_or_else(|| annotation_limit("annotation admission memory overflows"))?;
            for id in active.keys() {
                bytes = bytes
                    .checked_add(id.as_str().len())
                    .ok_or_else(|| annotation_limit("annotation admission memory overflows"))?;
            }
            let allowance = NonZeroUsize::new(bytes)
                .ok_or_else(|| annotation_limit("empty annotation admission allowance"))?;
            let memory = self.storage.prepare_production(allowance)?;
            let mut owners = reserved_slots(active.len())?;
            owners.extend(active.values().cloned());
            (owners, memory)
        };
        let mut candidates = reserved_slots(owners.len())?;
        for owner in &owners {
            let acquisition = owner.lock().await;
            if scope
                .test_execution_id
                .as_ref()
                .is_none_or(|id| id == &acquisition.execution_id)
                && scope
                    .test_cycle_id
                    .as_ref()
                    .is_none_or(|id| id == &acquisition.execution.test_cycle_id)
                && scope
                    .test_case_id
                    .as_ref()
                    .is_none_or(|id| id == &acquisition.execution.test_case_id)
            {
                candidates.push((acquisition.execution_id.clone(), acquisition.gate.clone()));
            }
        }
        let guards = lock_annotations(
            candidates,
            scope.test_execution_id.is_some(),
            |id, gate| async move {
                let Some(owner) = self.acquisition(&id).await else {
                    return false;
                };
                let acquisition = owner.lock().await;
                annotation_open(
                    &gate,
                    Some((
                        &acquisition.gate,
                        acquisition.finish_requested,
                        acquisition.execution.saved.is_some(),
                    )),
                )
            },
        )
        .await?;
        Ok(AnnotationAdmission {
            _guards: guards,
            _owners: owners,
            _memory: memory,
        })
    }

    pub(crate) async fn note_targets(
        &self,
        cycle: &CycleView,
        scope: &RecordScope,
    ) -> Result<QueryValue<Vec<TestExecutionId>>> {
        let key_bytes = cycle
            .test_executions
            .iter()
            .take(NOTE_SCAN_LIMIT)
            .map(|execution| execution.test_execution_id.as_str().len())
            .max()
            .unwrap_or(0);
        let bytes = std::mem::size_of::<TestExecutionId>()
            .checked_add(key_bytes)
            .and_then(|bytes| bytes.checked_mul(NOTE_CHECKPOINT_LIMIT))
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Vec<TestExecutionId>>()))
            .and_then(NonZeroUsize::new)
            .ok_or_else(|| annotation_limit("note target projection memory overflows"))?;
        let memory = self.storage.prepare_production(bytes)?;
        let mut targets = reserved_slots(NOTE_CHECKPOINT_LIMIT)?;
        for (index, execution) in cycle.test_executions.iter().enumerate() {
            if index >= NOTE_SCAN_LIMIT {
                return Err(annotation_limit("note checkpoint scan exceeds budget"));
            }
            if scope
                .test_case_id
                .as_ref()
                .is_some_and(|id| id != &execution.test_case_id)
                || scope
                    .test_execution_id
                    .as_ref()
                    .is_some_and(|id| id != &execution.test_execution_id)
            {
                continue;
            }
            if let Some(owner) = self.acquisition(&execution.test_execution_id).await {
                let acquisition = owner.lock().await;
                if !acquisition.finish_requested && acquisition.execution.saved.is_none() {
                    if targets.len() == NOTE_CHECKPOINT_LIMIT {
                        return Err(annotation_limit(
                            "note checkpoint fanout exceeds 64 targets",
                        ));
                    }
                    targets.push(execution.test_execution_id.clone());
                }
            }
            if scope.test_execution_id.is_some() {
                break;
            }
        }
        targets.sort_unstable();
        Ok(memory.finish(targets)?.try_into_value()?)
    }

    pub(crate) async fn note_is_published(
        &self,
        target: &TestExecutionId,
        note: &OperatorNote,
        limits: QueryLimits,
    ) -> Result<bool> {
        let view = self.storage.view_with_limits(limits).await?;
        let projection = view.decoded_bytes()?;
        let Some(inspection) = super::execution::ExecutionView::open(
            &self.storage,
            view,
            target,
            ObjectVersion::Latest,
            projection,
        )
        .await?
        .filter(|inspection| inspection.execution().saved.is_some()) else {
            return Ok(false);
        };
        let inspection = inspection.snapshot(&self.storage).await?;
        let confirmed = inspection.project_context(
            NonZeroUsize::new(std::mem::size_of::<bool>()).unwrap(),
            |context, _| {
                includes_note(
                    &inspection.execution().test_execution_id,
                    &context.session_id,
                    &context.notes,
                    target,
                    note,
                )
                .map_err(|source| harmos::Error::Backend {
                    operation: "verify frozen note publication",
                    kind: harmos::BackendFailure::Corrupt,
                    source: Box::new(source),
                })
            },
        )?;
        Ok(*confirmed)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn annotation_limit(reason: &'static str) -> Error {
    Error::Operation(reason.into())
}

fn reserved_slots<T>(count: usize) -> Result<Vec<T>> {
    let mut slots = Vec::new();
    slots
        .try_reserve_exact(count)
        .map_err(|_| annotation_limit("annotation slot allocation failed"))?;
    if slots.capacity() > count {
        return Err(annotation_limit(
            "annotation slots exceed their original claim",
        ));
    }
    Ok(slots)
}

async fn lock_annotations<F, Fut>(
    candidates: Vec<(TestExecutionId, Arc<tokio::sync::Mutex<()>>)>,
    explicit_execution: bool,
    mut current: F,
) -> Result<Vec<tokio::sync::OwnedMutexGuard<()>>>
where
    F: FnMut(TestExecutionId, Arc<tokio::sync::Mutex<()>>) -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let mut guards = reserved_slots(candidates.len())?;
    for (id, gate) in candidates {
        let guard = gate.clone().lock_owned().await;
        if current(id, gate).await {
            guards.push(guard);
        }
    }
    if explicit_execution && guards.is_empty() {
        return Err(Error::conflict(
            "execution_sealed",
            "execution no longer accepts annotations",
        ));
    }
    Ok(guards)
}

fn annotation_open(
    expected: &Arc<tokio::sync::Mutex<()>>,
    current: Option<(&Arc<tokio::sync::Mutex<()>>, bool, bool)>,
) -> bool {
    current.is_some_and(|(gate, finishing, sealed)| {
        Arc::ptr_eq(expected, gate) && !finishing && !sealed
    })
}

fn includes_note(
    execution: &TestExecutionId,
    session: &SessionId,
    notes: &[OperatorNote],
    target: &TestExecutionId,
    note: &OperatorNote,
) -> Result<bool> {
    if execution != target || note.scope.session_id.as_ref() != Some(session) {
        return Ok(false);
    }
    if notes.len() > NOTE_SCAN_LIMIT {
        return Err(Error::Operation(
            "checkpoint note verification exceeds scan budget".into(),
        ));
    }
    let mut matching = notes.iter().filter(|item| item.note_id == note.note_id);
    let Some(found) = matching.next() else {
        return Ok(false);
    };
    if matching.next().is_some() {
        return Ok(false);
    }
    Ok(found == note)
}

/* === Tests === */
