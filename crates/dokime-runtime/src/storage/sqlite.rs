/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod catalog;
pub(crate) mod data;
mod settings;

/* === Re-Exports === */

pub use crate::storage::Error;
pub use catalog::Source;
pub use data::Backend;
pub use rusqlite;

/* === Imports === */

// Standard Library Imports
use std::{
    fs::{File, OpenOptions},
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

// Third Party Imports
use rusqlite::{Connection, InterruptHandle};
use tokio::sync::Mutex;

// Local Imports

/* === Constants === */

pub const SCHEMA_VERSION: i64 = 1;

// A runtime being replaced releases the root as it exits; a successor waits
// this long for that before calling the root owned.

static COMMITTED_TRANSACTIONS: AtomicU64 = AtomicU64::new(0);
static DATA_BATCHES: AtomicU64 = AtomicU64::new(0);
static DURABLE_FILES: AtomicU64 = AtomicU64::new(0);
pub(super) static RECORD_CAPTURE_ROWS: AtomicU64 = AtomicU64::new(0);
pub(super) static RECORD_CAPTURE_PAGES: AtomicU64 = AtomicU64::new(0);
pub(super) static RECORD_CAPTURE_ADMISSIONS: AtomicU64 = AtomicU64::new(0);
pub(super) static TELEMETRY_ANCHOR_ROWS: AtomicU64 = AtomicU64::new(0);
pub(super) static DATA_SCAN_ROWS: AtomicU64 = AtomicU64::new(0);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct State(Arc<Owner>);

// Process-Wide SQLite Diagnostics; These Exclude Autocommit Statements And The Harmos State Journal.
#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct StorageCounters {
    pub committed_transactions: u64,
    pub data_batches: u64,
    // Successful File Durability Confirmations, Including Repeated Confirmations Of One File.
    pub durable_files: u64,
    pub record_capture_rows: u64,
    pub record_capture_pages: u64,
    pub record_capture_admissions: u64,
    pub telemetry_anchor_rows: u64,
    pub data_scan_rows: u64,
}

#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct ReadCounters {
    pub data_scan_rows: u64,
    pub telemetry_anchor_rows: u64,
}

#[derive(Default)]
struct ReadMetrics {
    data_scan_rows: AtomicU64,
    telemetry_anchor_rows: AtomicU64,
}

struct Owner {
    read_metrics: Arc<ReadMetrics>,
    live: Arc<Mutex<Option<Live>>>,
    closing: AtomicBool,
    interrupt: InterruptHandle,
    // The interruptible read running now, so a late cancellation cannot interrupt a successor.
    running: std::sync::Mutex<u64>,
    reads: AtomicU64,
    files: std::sync::Mutex<std::collections::BTreeMap<harmos::Identifier, File>>,
}

struct Live {
    connection: Connection,
    _lease: File,
}

// Interrupts The In-Flight Read When Its Caller Is Cancelled.
struct Interruption {
    owner: Arc<Owner>,
    read: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Drop for Interruption {
    fn drop(&mut self) {
        let running = self.owner.running.lock().unwrap();
        if self.read != 0 && *running == self.read {
            self.owner.interrupt.interrupt();
        }
    }
}

impl State {
    pub fn read_counters(&self) -> ReadCounters {
        ReadCounters {
            data_scan_rows: self.0.read_metrics.data_scan_rows.load(Ordering::Relaxed),
            telemetry_anchor_rows: self.0.read_metrics.telemetry_anchor_rows.load(Ordering::Relaxed),
        }
    }

    pub async fn open(root: impl AsRef<Path>) -> Result<Self, Error> {
        let root = root.as_ref().to_path_buf();
        tokio::task::spawn_blocking(move || {
            std::fs::create_dir_all(&root)?;
            let lease = OpenOptions::new().create(true).truncate(false).read(true).write(true).open(root.join("local.lock"))?;
            acquire_lease(&lease)?;
            let mut connection = Connection::open(root.join("local.sqlite"))?;
            check_layout(&connection)?;
            connection.execute_batch(
                "PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;
                PRAGMA foreign_keys=ON; PRAGMA temp_store=MEMORY; PRAGMA cache_size=-65536;",
            )?;
            let transaction = connection.transaction()?;
            transaction.execute_batch(catalog::SCHEMA)?;
            transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
            commit(transaction)?;
            let interrupt = connection.get_interrupt_handle();
            Ok(Self(Arc::new(Owner {
                read_metrics: Arc::new(ReadMetrics::default()),
                live: Arc::new(Mutex::new(Some(Live { connection, _lease: lease }))),
                closing: AtomicBool::new(false),
                interrupt,
                running: std::sync::Mutex::new(0),
                reads: AtomicU64::new(1),
                files: std::sync::Mutex::new(std::collections::BTreeMap::new()),
            })))
        })
        .await?
    }

    pub async fn close(&self) -> Result<(), Error> {
        self.0.closing.store(true, Ordering::Release);
        let mut live = self.0.live.clone().lock_owned().await;
        let owner = self.0.clone();
        tokio::task::spawn_blocking(move || {
            owner.files.lock().unwrap().clear();
            let Some(Live { connection, _lease }) = live.take() else {
                return Ok(());
            };
            connection.close().map_err(|(connection, error)| {
                *live = Some(Live { connection, _lease });
                Error::Sqlite(error)
            })
        })
        .await?
    }

    pub async fn call<T: Send + 'static>(
        &self, operation: impl FnOnce(&mut Connection) -> Result<T, rusqlite::Error> + Send + 'static,
    ) -> Result<T, Error> {
        self.dispatch(false, operation).await
    }

    pub(crate) async fn read<T: Send + 'static>(
        &self, operation: impl FnOnce(&mut Connection) -> Result<T, rusqlite::Error> + Send + 'static,
    ) -> Result<T, Error> {
        self.dispatch(true, operation).await
    }

    async fn dispatch<T: Send + 'static>(
        &self, interruptible: bool, operation: impl FnOnce(&mut Connection) -> Result<T, rusqlite::Error> + Send + 'static,
    ) -> Result<T, Error> {
        if self.0.closing.load(Ordering::Acquire) {
            return Err(Error::Closed);
        }
        // Acquire before spawning: a cancelled caller cannot leave a queued
        // older write that overtakes its retry or a subsequent transition.
        let owner = self.0.clone();
        let mut live = owner.live.clone().lock_owned().await;
        let read = if interruptible { owner.reads.fetch_add(1, Ordering::Relaxed) } else { 0 };
        let mut interruption = Interruption { owner: owner.clone(), read };
        let result = tokio::task::spawn_blocking(move || {
            *owner.running.lock().unwrap() = read;
            let result = live.as_mut().ok_or(Error::Closed).and_then(|live| operation(&mut live.connection).map_err(Error::Sqlite));
            *owner.running.lock().unwrap() = 0;
            drop(live);
            result
        })
        .await;
        interruption.read = 0;
        result?
    }
}

/* === Public Functions === */

pub fn counters() -> StorageCounters {
    StorageCounters {
        committed_transactions: COMMITTED_TRANSACTIONS.load(Ordering::Relaxed),
        data_batches: DATA_BATCHES.load(Ordering::Relaxed),
        durable_files: DURABLE_FILES.load(Ordering::Relaxed),
        record_capture_rows: RECORD_CAPTURE_ROWS.load(Ordering::Relaxed),
        record_capture_pages: RECORD_CAPTURE_PAGES.load(Ordering::Relaxed),
        record_capture_admissions: RECORD_CAPTURE_ADMISSIONS.load(Ordering::Relaxed),
        telemetry_anchor_rows: TELEMETRY_ANCHOR_ROWS.load(Ordering::Relaxed),
        data_scan_rows: DATA_SCAN_ROWS.load(Ordering::Relaxed),
    }
}

/* === Private Functions === */

fn commit(transaction: rusqlite::Transaction<'_>) -> rusqlite::Result<()> {
    transaction.commit()?;
    COMMITTED_TRANSACTIONS.fetch_add(1, Ordering::Relaxed);
    Ok(())
}

// Takes the root's exclusive lease, waiting a bounded time for a departing
// owner to let go. A lease still held after that belongs to a live runtime.
fn acquire_lease(lease: &File) -> Result<(), Error> {
    let wait = Duration::from_secs(5);
    let deadline = Instant::now()
        .checked_add(wait)
        .ok_or_else(|| Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, "SQLite lease wait exceeds clock range")))?;
    let poll = Duration::from_millis(25);
    if poll.is_zero() {
        return Err(Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, "SQLite lease poll must be positive")));
    }
    loop {
        match lease.try_lock() {
            Ok(()) => return Ok(()),
            Err(std::fs::TryLockError::Error(error)) => return Err(Error::Io(error)),
            Err(std::fs::TryLockError::WouldBlock) if Instant::now() < deadline => {
                std::thread::sleep(poll.min(deadline.saturating_duration_since(Instant::now())));
            }
            Err(std::fs::TryLockError::WouldBlock) => return Err(Error::Owned),
        }
    }
}

// One Layout Check: A Fresh File Or This Build's Exact Layout.
fn check_layout(connection: &Connection) -> Result<(), Error> {
    let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let tables: bool =
        connection.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%')", [], |row| {
            row.get(0)
        })?;
    if version == 0 && !tables {
        return Ok(());
    }
    if version != SCHEMA_VERSION {
        return Err(Error::Incompatible(format!(
            "local.sqlite declares schema version {version}, and this build reads version {SCHEMA_VERSION}"
        )));
    }
    data::check_format(connection)
}

/* === Tests === */
