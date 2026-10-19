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
use harmos::storage::ObjectVersion;

// Local Imports
use super::executions::{Acquisition, ExecutionStorage};
use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::notes::NOTE_CHECKPOINT_LIMIT;
use dokime_domain::model::execution::notes::NOTE_SCAN_LIMIT;
use dokime_domain::model::execution::notes::OperatorNote;
use dokime_domain::model::execution::result::CycleView;
use dokime_domain::model::identity::SessionId;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct AnnotationAdmission {
    _guards: Vec<tokio::sync::OwnedMutexGuard<()>>,
    _owners: Vec<Arc<tokio::sync::Mutex<Acquisition>>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ExecutionStorage {
    pub(crate) async fn admit_annotations(&self, scope: &RecordScope) -> Result<AnnotationAdmission> {
        // Snapshot Handles In Key Order; Release The Registry Before Awaiting Owners
        let owners = {
            let active = self.active.lock().await;
            if active.len() > NOTE_SCAN_LIMIT {
                return Err(annotation_limit("annotation owner scan exceeds budget"));
            }
            active.values().cloned().collect::<Vec<_>>()
        };
        let mut candidates = Vec::with_capacity(owners.len());
        for owner in &owners {
            let acquisition = owner.lock().await;
            if scope.test_execution_id.as_ref().is_none_or(|id| id == &acquisition.execution_id)
                && scope.test_cycle_id.as_ref().is_none_or(|id| id == &acquisition.execution.test_cycle_id)
                && scope.test_case_id.as_ref().is_none_or(|id| id == &acquisition.execution.test_case_id)
            {
                candidates.push((acquisition.execution_id.clone(), acquisition.gate.clone()));
            }
        }
        let guards = lock_annotations(candidates, scope.test_execution_id.is_some(), |id, gate| async move {
            let Some(owner) = self.acquisition(&id).await else {
                return false;
            };
            let acquisition = owner.lock().await;
            annotation_open(&gate, Some((&acquisition.gate, acquisition.finish_requested, acquisition.execution.saved.is_some())))
        })
        .await?;
        Ok(AnnotationAdmission { _guards: guards, _owners: owners })
    }

    pub(crate) async fn note_targets(&self, cycle: &CycleView, scope: &RecordScope) -> Result<Vec<TestExecutionId>> {
        let mut targets = Vec::with_capacity(NOTE_CHECKPOINT_LIMIT);
        for (index, execution) in cycle.test_executions.iter().enumerate() {
            if index >= NOTE_SCAN_LIMIT {
                return Err(annotation_limit("note checkpoint scan exceeds budget"));
            }
            if scope.test_case_id.as_ref().is_some_and(|id| id != &execution.test_case_id)
                || scope.test_execution_id.as_ref().is_some_and(|id| id != &execution.test_execution_id)
            {
                continue;
            }
            if let Some(owner) = self.acquisition(&execution.test_execution_id).await {
                let acquisition = owner.lock().await;
                if !acquisition.finish_requested && acquisition.execution.saved.is_none() {
                    if targets.len() == NOTE_CHECKPOINT_LIMIT {
                        return Err(annotation_limit("note checkpoint fanout exceeds 64 targets"));
                    }
                    targets.push(execution.test_execution_id.clone());
                }
            }
            if scope.test_execution_id.is_some() {
                break;
            }
        }
        targets.sort_unstable();
        Ok(targets)
    }

    pub(crate) async fn note_is_published(&self, target: &TestExecutionId, note: &OperatorNote) -> Result<bool> {
        let Some(inspection) = super::execution::ExecutionView::open(&self.storage, target, ObjectVersion::Latest)
            .await?
            .filter(|inspection| inspection.execution().saved.is_some())
        else {
            return Ok(false);
        };
        let inspection = inspection.snapshot().await?;
        let context = inspection.context()?;
        includes_note(&inspection.execution().test_execution_id, &context.session_id, &context.notes, target, note)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn annotation_limit(reason: &'static str) -> Error {
    Error::Operation(reason.into())
}

async fn lock_annotations<F, Fut>(
    candidates: Vec<(TestExecutionId, Arc<tokio::sync::Mutex<()>>)>, explicit_execution: bool, mut current: F,
) -> Result<Vec<tokio::sync::OwnedMutexGuard<()>>>
where
    F: FnMut(TestExecutionId, Arc<tokio::sync::Mutex<()>>) -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let mut guards = Vec::with_capacity(candidates.len());
    for (id, gate) in candidates {
        let guard = gate.clone().lock_owned().await;
        if current(id, gate).await {
            guards.push(guard);
        }
    }
    if explicit_execution && guards.is_empty() {
        return Err(Error::conflict("execution_sealed", "execution no longer accepts annotations"));
    }
    Ok(guards)
}

fn annotation_open(expected: &Arc<tokio::sync::Mutex<()>>, current: Option<(&Arc<tokio::sync::Mutex<()>>, bool, bool)>) -> bool {
    current.is_some_and(|(gate, finishing, sealed)| Arc::ptr_eq(expected, gate) && !finishing && !sealed)
}

fn includes_note(
    execution: &TestExecutionId, session: &SessionId, notes: &[OperatorNote], target: &TestExecutionId, note: &OperatorNote,
) -> Result<bool> {
    if execution != target || note.scope.session_id.as_ref() != Some(session) {
        return Ok(false);
    }
    if notes.len() > NOTE_SCAN_LIMIT {
        return Err(Error::Operation("checkpoint note verification exceeds scan budget".into()));
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
