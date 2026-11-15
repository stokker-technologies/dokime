/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// File Capture Operations

/* === Modules === */

pub(crate) mod engine;
pub mod start;
pub mod stop;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, RwLock};

// Third Party Imports
use tokio::sync::watch;

// Local Imports
use crate::{Error, Result};
use dokime_protocol::host::files::CapturesStopResponse;
use engine::Capture;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type Outcome = std::result::Result<CapturesStopResponse, Refusal>;

/* === Structs === */

/// Host-Owned Local File Capture Bounds; Requests Carry No Limits.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaptureSettings {
    /// Directories Plugins May Capture From; Empty Refuses Every Local Capture.
    pub roots: Vec<PathBuf>,
    pub maximum_files: usize,
    pub maximum_bytes: u64,
    pub maximum_captures: usize,
    pub scan_entries: usize,
}

/// Running Local Captures, Each Owned by the Execution That Started It.
pub struct Captures {
    settings: RwLock<CaptureSettings>,
    entries: Mutex<BTreeMap<u64, Entry>>,
    next: AtomicU64,
}

struct Entry {
    execution: String,
    capture: Option<Capture>,
    outcome: watch::Sender<Option<Outcome>>,
}

// A Sealing Failure Replayed to Every Later Stop of the Same Capture.
#[derive(Clone, Debug)]
struct Refusal {
    bound: bool,
    message: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Default for CaptureSettings {
    fn default() -> Self {
        Self {
            roots: Vec::new(),
            maximum_files: 64,
            maximum_bytes: 256 * 1024 * 1024,
            maximum_captures: 256,
            scan_entries: 100_000,
        }
    }
}

impl Captures {
    pub fn new(settings: CaptureSettings) -> Self {
        Self {
            settings: RwLock::new(settings),
            entries: Mutex::new(BTreeMap::new()),
            next: AtomicU64::new(1),
        }
    }
    /// The Current Bounds; Each New Capture Reads Them Once When It Opens.
    pub fn settings(&self) -> CaptureSettings {
        self.settings.read().unwrap_or_else(std::sync::PoisonError::into_inner).clone()
    }
    /// Replaces the Allowed Roots for Captures Opened From Now On; Running Captures Keep Theirs.
    pub fn set_roots(&self, roots: Vec<PathBuf>) {
        self.settings.write().unwrap_or_else(std::sync::PoisonError::into_inner).roots = roots;
    }
    /// Seals Every Running Capture of an Ending Execution, Then Forgets Them All.
    ///
    /// The execution calls this before closing its evidence, after its plugin calls settled;
    /// a capture nothing stopped takes its final pass here, without a warning.
    pub async fn settle(&self, execution: &str) {
        let identifiers = self.lock().iter().filter(|(_, entry)| entry.execution == execution).map(|(id, _)| *id).collect::<Vec<_>>();
        for identifier in identifiers {
            if let Err(error) = self.stop(execution, identifier).await {
                tracing::warn!(%execution, capture = identifier, %error, "Settle File Capture");
            }
        }
        self.discard(execution);
    }
    /// Forgets an Execution's Captures Without Sealing; Used After Its Evidence Closed.
    pub fn discard(&self, execution: &str) {
        self.lock().retain(|_, entry| entry.execution != execution);
    }
    pub(crate) fn identifier(&self) -> u64 {
        self.next.fetch_add(1, Ordering::SeqCst)
    }
    pub(crate) fn admit(&self, capture: Capture) -> Result<u64> {
        let mut entries = self.lock();
        let running = entries.values().filter(|entry| entry.outcome.borrow().is_none()).count();
        if running >= self.settings().maximum_captures {
            return Err(Error::Bound("Capture Files: Stop a Running Capture or Raise the Host's Capture Capacity".into()));
        }
        let identifier = capture.identifier();
        let (outcome, _) = watch::channel(None);
        entries.insert(
            identifier,
            Entry {
                execution: capture.execution().to_owned(),
                capture: Some(capture),
                outcome,
            },
        );
        Ok(identifier)
    }
    pub(crate) async fn stop(&self, execution: &str, identifier: u64) -> Result<CapturesStopResponse> {
        let mut outcome = {
            let mut entries = self.lock();
            let entry = entries
                .get_mut(&identifier)
                .ok_or_else(|| Error::NotFound(format!("{}: Select a Capture Started by This Execution", stop::ID)))?;
            if entry.execution != execution {
                return Err(Error::Permission(format!("{}: Select a Capture Started by This Execution", stop::ID)));
            }
            // The First Stop Seals in Its Own Task; an Abandoned Reply Leaves Sealing Running.
            if let Some(capture) = entry.capture.take() {
                let sender = entry.outcome.clone();
                tokio::spawn(async move {
                    let sealed = capture.seal().await.map_err(|error| Refusal {
                        bound: matches!(error, Error::Bound(_)),
                        message: error.to_string(),
                    });
                    sender.send_replace(Some(sealed));
                });
            }
            entry.outcome.subscribe()
        };
        let settled = outcome
            .wait_for(Option::is_some)
            .await
            .map_err(|_| Error::Unavailable(format!("{}: Retry After the Host Restarts", stop::ID)))?
            .clone()
            .expect("the wait ends on an outcome");
        settled.map_err(|refusal| if refusal.bound { Error::Bound(refusal.message) } else { Error::Unavailable(refusal.message) })
    }
    fn lock(&self) -> std::sync::MutexGuard<'_, BTreeMap<u64, Entry>> {
        self.entries.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
