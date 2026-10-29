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

use std::{
    collections::BTreeMap,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};

use tokio::sync::oneshot;

use dokime_domain::model::identity::TestExecutionId;

// Asks a capture to seal now; it answers once its artifact is stored.
pub(crate) type SealRequest = oneshot::Sender<oneshot::Sender<()>>;

pub(crate) struct Captures {
    open: Mutex<BTreeMap<TestExecutionId, Vec<(u64, SealRequest)>>>,
    next: AtomicU64,
    seal_timeout: Duration,
}

impl Captures {
    pub(crate) fn new() -> Self {
        Self {
            open: Mutex::new(BTreeMap::new()),
            next: AtomicU64::new(0),
            seal_timeout: Duration::from_secs(60),
        }
    }

    pub(crate) fn register(&self, execution: &TestExecutionId, seal: SealRequest) -> u64 {
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        self.open.lock().unwrap_or_else(std::sync::PoisonError::into_inner).entry(execution.clone()).or_default().push((id, seal));
        id
    }

    pub(crate) fn release(&self, execution: &TestExecutionId, id: u64) {
        let mut open = self.open.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(captures) = open.get_mut(execution) {
            captures.retain(|(capture, _)| *capture != id);
            if captures.is_empty() {
                open.remove(execution);
            }
        }
    }

    // Seals every capture of the execution before its output admission closes.
    pub(crate) async fn close(&self, execution: &TestExecutionId) {
        let captures = self.open.lock().unwrap_or_else(std::sync::PoisonError::into_inner).remove(execution).unwrap_or_default();

        // Ask All, Then Await Each Stored Artifact
        let mut stored = Vec::with_capacity(captures.len());
        for (_, seal) in captures {
            let (done, sealed) = oneshot::channel();
            if seal.send(done).is_ok() {
                stored.push(sealed);
            }
        }
        for sealed in stored {
            if tokio::time::timeout(self.seal_timeout, sealed).await.is_err() {
                tracing::warn!(execution = %execution, "capture did not seal before its execution closed");
            }
        }
    }
}
