/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Operation progress: the slow work the backend is doing right now.
//
// Any host code that may keep an operator waiting begins an operation, names
// its phases and, when it can, how far it has come. The guard it receives
// settles the operation when its last clone drops, so an operation never
// outlives the work that began it. Finished and failed operations stay
// visible for a short retention, then retire. Nothing here is persisted: the
// list describes this process only and is published beside runtime state.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{
    collections::BTreeMap,
    fmt::Display,
    sync::{Arc, Mutex, Weak},
    time::Duration,
};
// Third Party Imports

// Local Imports
use dokime::core::Timestamp;

/* === Constants === */
/// How long a settled operation stays visible before it retires.
pub const SETTLED_RETENTION: Duration = Duration::from_secs(3);

/// The most operations one snapshot carries, running ones first.
pub const MAX_OPERATIONS: usize = 32;

/// The longest label, phase or failure an operation publishes, in characters.
pub const MAX_TEXT_CHARS: usize = 160;

/// The smallest change of fraction worth publishing on its own.
const FRACTION_STEP: f64 = 0.01;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// One operation as the runtime last published it.
#[derive(Clone, Debug, PartialEq)]
pub struct LiveOperation {
    pub operation_id: String,
    pub label: String,
    pub phase: String,
    /// How far the operation has come, from zero to one; absent while it
    /// cannot tell.
    pub fraction: Option<f64>,
    pub started_at: Timestamp,
    pub outcome: OperationOutcome,
}

/// Every operation the runtime currently shows, oldest first.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct OperationSnapshot {
    revision: u64,
    operations: Vec<LiveOperation>,
}

/// The process-wide list of operations.
pub struct OperationRegistry {
    state: Mutex<Entries>,
    sender: tokio::sync::watch::Sender<OperationSnapshot>,
}

/// Follows the operation list.
pub struct OperationWatch {
    receiver: tokio::sync::watch::Receiver<OperationSnapshot>,
}

/// The handle the code doing the work holds. Clones share one operation, which
/// finishes when the last clone drops unless it was already settled.
#[derive(Clone)]
pub struct Operation {
    owner: Arc<Owner>,
}

struct Owner {
    registry: Weak<OperationRegistry>,
    operation_id: String,
}

#[derive(Default)]
struct Entries {
    revision: u64,
    operations: BTreeMap<String, Entry>,
}

struct Entry {
    operation: LiveOperation,
    retires_at: Option<tokio::time::Instant>,
}

/* === Enums === */
/// Where an operation stands.
#[derive(Clone, Debug, PartialEq)]
pub enum OperationOutcome {
    Running,
    Finished { at: Timestamp },
    Failed { at: Timestamp, failure: String },
}

/* === Traits === */

/* === Implementations === */
impl LiveOperation {
    /// The bytes this operation occupies once published.
    pub fn memory_size(&self) -> usize {
        let failure = match &self.outcome {
            OperationOutcome::Failed { failure, .. } => failure.len(),
            _ => 0,
        };
        std::mem::size_of::<Self>()
            .saturating_add(self.operation_id.len())
            .saturating_add(self.label.len())
            .saturating_add(self.phase.len())
            .saturating_add(failure)
    }
}

impl OperationSnapshot {
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn operations(&self) -> &[LiveOperation] {
        &self.operations
    }

    pub fn get(&self, operation_id: &str) -> Option<&LiveOperation> {
        self.operations
            .iter()
            .find(|operation| operation.operation_id == operation_id)
    }
}

impl OperationRegistry {
    pub fn new() -> Arc<Self> {
        let (sender, _) = tokio::sync::watch::channel(OperationSnapshot::default());
        Arc::new(Self {
            state: Mutex::new(Entries::default()),
            sender,
        })
    }

    /// Begins an operation named `label`, running with no phase and no
    /// fraction yet.
    pub fn begin(self: &Arc<Self>, label: impl Into<String>) -> Operation {
        let operation_id = uuid::Uuid::now_v7().to_string();
        let operation = LiveOperation {
            operation_id: operation_id.clone(),
            label: bounded(label.into()),
            phase: String::new(),
            fraction: None,
            started_at: Timestamp::now_utc(),
            outcome: OperationOutcome::Running,
        };
        self.change(|entries| {
            entries.operations.insert(
                operation_id.clone(),
                Entry {
                    operation,
                    retires_at: None,
                },
            );
            true
        });
        Operation {
            owner: Arc::new(Owner {
                registry: Arc::downgrade(self),
                operation_id,
            }),
        }
    }

    pub fn watch(&self) -> OperationWatch {
        OperationWatch {
            receiver: self.sender.subscribe(),
        }
    }

    pub fn latest(&self) -> OperationSnapshot {
        self.sender.borrow().clone()
    }

    fn update(&self, operation_id: &str, update: impl FnOnce(&mut LiveOperation) -> bool) {
        self.change(|entries| {
            entries
                .operations
                .get_mut(operation_id)
                .filter(|entry| entry.operation.outcome == OperationOutcome::Running)
                .is_some_and(|entry| update(&mut entry.operation))
        });
    }

    fn settle(self: &Arc<Self>, operation_id: &str, outcome: OperationOutcome) {
        let retires_at = tokio::time::Instant::now() + SETTLED_RETENTION;
        self.change(|entries| {
            let Some(entry) = entries
                .operations
                .get_mut(operation_id)
                .filter(|entry| entry.operation.outcome == OperationOutcome::Running)
            else {
                return false;
            };
            if matches!(outcome, OperationOutcome::Finished { .. }) {
                entry.operation.fraction = entry.operation.fraction.map(|_| 1.0);
            }
            entry.operation.outcome = outcome;
            entry.retires_at = Some(retires_at);
            true
        });
        // Without a runtime to wait on, the next change retires it instead.
        if let Ok(runtime) = tokio::runtime::Handle::try_current() {
            let registry = Arc::downgrade(self);
            runtime.spawn(async move {
                tokio::time::sleep_until(retires_at).await;
                if let Some(registry) = registry.upgrade() {
                    registry.change(|_| false);
                }
            });
        }
    }

    // Applies `change`, retires what has outlived its retention and publishes
    // when either altered the list.
    fn change(&self, change: impl FnOnce(&mut Entries) -> bool) {
        let snapshot = {
            let mut entries = self.state.lock().unwrap_or_else(|error| error.into_inner());
            let changed = change(&mut entries);
            let now = tokio::time::Instant::now();
            let before = entries.operations.len();
            entries
                .operations
                .retain(|_, entry| entry.retires_at.is_none_or(|at| at > now));
            if !changed && entries.operations.len() == before {
                return;
            }
            entries.revision = entries.revision.saturating_add(1);
            entries.snapshot()
        };
        self.sender.send_replace(snapshot);
    }
}

impl Entries {
    // Running operations first, then the most recently settled, each group
    // oldest first, so a burst of settled work never hides what still runs.
    fn snapshot(&self) -> OperationSnapshot {
        let mut shown: Vec<&Entry> = self.operations.values().collect();
        shown.sort_by(|left, right| {
            left.retires_at
                .is_some()
                .cmp(&right.retires_at.is_some())
                .then_with(|| right.retires_at.cmp(&left.retires_at))
        });
        shown.truncate(MAX_OPERATIONS);
        // Identifiers are time-ordered, so this restores start order.
        shown.sort_by(|left, right| {
            left.operation
                .operation_id
                .cmp(&right.operation.operation_id)
        });
        OperationSnapshot {
            revision: self.revision,
            operations: shown
                .into_iter()
                .map(|entry| entry.operation.clone())
                .collect(),
        }
    }
}

impl OperationWatch {
    pub fn latest(&self) -> OperationSnapshot {
        self.receiver.borrow().clone()
    }

    pub async fn next(&mut self) -> Option<OperationSnapshot> {
        self.receiver.changed().await.ok()?;
        Some(self.receiver.borrow_and_update().clone())
    }
}

impl Operation {
    pub fn operation_id(&self) -> &str {
        &self.owner.operation_id
    }

    /// Names the step the operation is on. The fraction stays until the
    /// step states its own.
    pub fn phase(&self, phase: impl Into<String>) {
        let phase = bounded(phase.into());
        self.update(|operation| {
            if operation.phase == phase {
                return false;
            }
            operation.phase = phase;
            true
        });
    }

    /// States how far the operation has come, from zero to one. Changes too
    /// small to see are not published on their own.
    pub fn fraction(&self, fraction: f64) {
        let fraction = fraction.is_finite().then(|| fraction.clamp(0.0, 1.0));
        self.update(|operation| {
            let visible = match (operation.fraction, fraction) {
                (Some(previous), Some(next)) => {
                    (next - previous).abs() >= FRACTION_STEP || next >= 1.0
                }
                (previous, next) => previous != next,
            };
            if visible {
                operation.fraction = fraction;
            }
            visible
        });
    }

    /// States `done` of `total` units complete; indeterminate while `total`
    /// is zero.
    pub fn progress(&self, done: u64, total: u64) {
        if total == 0 {
            self.indeterminate();
        } else {
            self.fraction(done as f64 / total as f64);
        }
    }

    /// States that the operation cannot tell how far it has come.
    pub fn indeterminate(&self) {
        self.update(|operation| operation.fraction.take().is_some());
    }

    pub fn finish(&self) {
        self.settle(OperationOutcome::Finished {
            at: Timestamp::now_utc(),
        });
    }

    pub fn fail(&self, failure: impl Display) {
        self.settle(OperationOutcome::Failed {
            at: Timestamp::now_utc(),
            failure: bounded(failure.to_string()),
        });
    }

    /// Records `result` on the operation: a failure settles it failed, success
    /// leaves it running for the caller to finish.
    pub fn track<T, E: Display>(&self, result: Result<T, E>) -> Result<T, E> {
        if let Err(error) = &result {
            self.fail(error);
        }
        result
    }

    fn update(&self, update: impl FnOnce(&mut LiveOperation) -> bool) {
        if let Some(registry) = self.owner.registry.upgrade() {
            registry.update(&self.owner.operation_id, update);
        }
    }

    fn settle(&self, outcome: OperationOutcome) {
        if let Some(registry) = self.owner.registry.upgrade() {
            registry.settle(&self.owner.operation_id, outcome);
        }
    }
}

impl Drop for Owner {
    fn drop(&mut self) {
        if let Some(registry) = self.registry.upgrade() {
            registry.settle(
                &self.operation_id,
                OperationOutcome::Finished {
                    at: Timestamp::now_utc(),
                },
            );
        }
    }
}

impl std::fmt::Debug for Operation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Operation")
            .field("operation_id", &self.owner.operation_id)
            .finish()
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn bounded(mut text: String) -> String {
    if let Some((end, _)) = text.char_indices().nth(MAX_TEXT_CHARS) {
        text.truncate(end);
    }
    text
}

/* === Tests === */
