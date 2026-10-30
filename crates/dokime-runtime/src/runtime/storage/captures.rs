/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Running captures (media recordings, file watches) per execution. Closing an
// execution's outputs first asks each of its captures to stop and seal, so a
// capture becomes an artifact of that execution instead of outliving its
// admission.

/* === Modules === */

#[cfg(feature = "testing")]
pub(crate) mod testing;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

// Third Party Imports
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

// Local Imports
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

// Asks a capture to seal now; it answers once its artifact is stored.
pub(crate) type SealRequest = oneshot::Sender<oneshot::Sender<()>>;

/* === Structs === */

pub(crate) struct Captures {
    state: Arc<Mutex<State>>,
    next: AtomicU64,
    seal_timeout: Duration,
}

#[derive(Default)]
struct State {
    open: BTreeMap<TestExecutionId, Vec<(u64, SealRequest)>>,
    closing: BTreeMap<TestExecutionId, CancellationToken>,
    sealed: BTreeSet<TestExecutionId>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Captures {
    pub(crate) fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(State::default())),
            next: AtomicU64::new(0),
            seal_timeout: Duration::from_secs(60),
        }
    }

    pub(crate) fn register(&self, execution: &TestExecutionId, seal: SealRequest) -> crate::runtime::Result<u64> {
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.sealed.contains(execution) {
            return Err(crate::runtime::Error::conflict("execution_captures_closed", "execution capture admission is closed"));
        }
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        state.open.entry(execution.clone()).or_default().push((id, seal));
        Ok(id)
    }

    pub(crate) fn release(&self, execution: &TestExecutionId, id: u64) {
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(captures) = state.open.get_mut(execution) {
            captures.retain(|(capture, _)| *capture != id);
            if captures.is_empty() {
                state.open.remove(execution);
            }
        }
    }

    // Seals every capture of the execution before its output admission closes.
    pub(crate) async fn close(&self, execution: &TestExecutionId) {
        let settled = {
            let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            state.sealed.insert(execution.clone());
            if let Some(settled) = state.closing.get(execution) {
                settled.clone()
            } else {
                let captures = state.open.remove(execution).unwrap_or_default();
                if captures.is_empty() {
                    return;
                }
                let settled = CancellationToken::new();
                state.closing.insert(execution.clone(), settled.clone());
                let state = Arc::clone(&self.state);
                let execution = execution.clone();
                let completed = settled.clone();
                let timeout = self.seal_timeout;
                // The Owner Survives A Cancelled Stop/Shutdown Waiter.
                tokio::spawn(async move {
                    let mut stored = Vec::with_capacity(captures.len());
                    for (_, seal) in captures {
                        let (done, sealed) = oneshot::channel();
                        if seal.send(done).is_ok() {
                            stored.push(sealed);
                        }
                    }
                    let draining = futures_util::future::join_all(stored);
                    if tokio::time::timeout(timeout, draining).await.is_err() {
                        tracing::warn!(%execution, "capture did not seal before its execution closed");
                    }
                    let mut state = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                    completed.cancel();
                    state.closing.remove(&execution);
                });
                settled
            }
        };
        settled.cancelled().await;
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
