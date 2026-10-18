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
use std::{collections::BTreeMap, sync::Mutex};
// Third Party Imports

// Local Imports
use crate::execution::ExecutionProgress;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq)]
pub struct LiveExecutionProgress {
    pub test_execution_id: TestExecutionId,
    pub step: Identifier,
    pub progress: ExecutionProgress,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ProgressSnapshot {
    revision: u64,
    entries: BTreeMap<TestExecutionId, LiveExecutionProgress>,
}

pub struct ProgressRegistry {
    state: Mutex<ProgressSnapshot>,
    sender: tokio::sync::watch::Sender<ProgressSnapshot>,
}

pub struct ProgressWatch {
    receiver: tokio::sync::watch::Receiver<ProgressSnapshot>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ProgressSnapshot {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn get(&self, execution: &TestExecutionId) -> Option<&LiveExecutionProgress> {
        self.entries.get(execution)
    }
}

impl ProgressRegistry {
    pub fn new() -> Self {
        let initial = ProgressSnapshot::default();
        let (sender, _) = tokio::sync::watch::channel(initial.clone());
        Self { state: Mutex::new(initial), sender }
    }

    pub fn update(&self, value: LiveExecutionProgress) {
        self.change(|entries| {
            entries.insert(value.test_execution_id.clone(), value);
        });
    }

    pub fn clear(&self, execution: &TestExecutionId) {
        let execution = execution.clone();
        self.change(|entries| {
            entries.remove(&execution);
        });
    }

    pub fn watch(&self) -> ProgressWatch {
        ProgressWatch { receiver: self.sender.subscribe() }
    }

    fn change(&self, change: impl FnOnce(&mut BTreeMap<TestExecutionId, LiveExecutionProgress>)) {
        let snapshot = {
            let mut state = self.state.lock().expect("progress registry lock poisoned");
            change(&mut state.entries);
            state.revision = state.revision.saturating_add(1);
            state.clone()
        };
        self.sender.send_replace(snapshot);
    }
}

impl Default for ProgressRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressWatch {
    pub fn latest(&self) -> ProgressSnapshot {
        self.receiver.borrow().clone()
    }

    pub async fn next(&mut self) -> Option<ProgressSnapshot> {
        self.receiver.changed().await.ok()?;
        Some(self.latest())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
