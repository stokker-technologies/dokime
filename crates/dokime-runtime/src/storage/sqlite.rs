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
    num::NonZeroUsize,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

// Third Party Imports
use rusqlite::{Connection, InterruptHandle};
use tokio::sync::{Mutex, Semaphore};

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
    // The Only Writer Connection; Reads Never Hold It, So Concurrent Queries Cannot Delay A Save.
    live: Arc<Mutex<Option<Live>>>,
    // WAL Readers, One Per Admitted Active Query; Each Read Sees One Committed Snapshot.
    readers: Arc<Readers>,
    closing: AtomicBool,
    reads: AtomicU64,
    files: std::sync::Mutex<std::collections::BTreeMap<harmos::Identifier, File>>,
    verification_prefix: std::sync::Mutex<data::VerifiedPrefix>,
}

struct Live {
    connection: Connection,
    _lease: File,
}

struct Readers {
    permits: Arc<Semaphore>,
    count: u32,
    idle: std::sync::Mutex<Vec<Reader>>,
}

struct Reader {
    connection: Connection,
    slot: Arc<ReaderSlot>,
}

struct ReaderSlot {
    interrupt: InterruptHandle,
    // The interruptible read running now, so a late cancellation cannot interrupt a successor.
    running: std::sync::Mutex<u64>,
}

// Interrupts The In-Flight Read When Its Caller Is Cancelled.
struct Interruption {
    slot: Arc<ReaderSlot>,
    read: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Drop for Interruption {
    fn drop(&mut self) {
        let running = self.slot.running.lock().unwrap();
        if self.read != 0 && *running == self.read {
            self.slot.interrupt.interrupt();
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

    // Opens With The Configured Default Reader Count.
    #[cfg(feature = "configuration")]
    pub async fn open(root: impl AsRef<Path>) -> Result<Self, Error> {
        Self::open_with(root, crate::config::storage_defaults().request.max_active_queries).await
    }

    // `readers` Bounds The Concurrent Read Connections: The Storage Settings' Active Queries.
    pub async fn open_with(root: impl AsRef<Path>, readers: NonZeroUsize) -> Result<Self, Error> {
        let root = root.as_ref().to_path_buf();
        let count = u32::try_from(readers.get())
            .ok()
            .filter(|count| *count as usize <= Semaphore::MAX_PERMITS)
            .ok_or_else(|| Error::Io(std::io::Error::new(std::io::ErrorKind::InvalidInput, "SQLite reader count exceeds its range")))?;
        tokio::task::spawn_blocking(move || {
            std::fs::create_dir_all(&root)?;
            let lease = OpenOptions::new().create(true).truncate(false).read(true).write(true).open(root.join("local.lock"))?;
            acquire_lease(&lease)?;
            let mut connection = Connection::open(root.join("local.sqlite"))?;
            let fresh = check_layout(&connection)?;
            connection.execute_batch(
                "PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;
                PRAGMA foreign_keys=ON; PRAGMA temp_store=MEMORY; PRAGMA cache_size=-65536;",
            )?;
            // Idempotent DDL Over A Current Layout Dirties No Page, So Its Commit Writes And
            // Syncs Nothing; Restamping The Version Would Rewrite The Header On Every Open.
            let transaction = connection.transaction()?;
            transaction.execute_batch(catalog::SCHEMA)?;
            if fresh {
                transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
            }
            commit(transaction)?;
            let mut idle = Vec::new();
            idle.try_reserve_exact(readers.get()).map_err(std::io::Error::other)?;
            for _ in 0..readers.get() {
                let connection = Connection::open(root.join("local.sqlite"))?;
                connection.execute_batch("PRAGMA query_only=ON; PRAGMA temp_store=MEMORY;")?;
                let slot = Arc::new(ReaderSlot {
                    interrupt: connection.get_interrupt_handle(),
                    running: std::sync::Mutex::new(0),
                });
                idle.push(Reader { connection, slot });
            }
            let verification_prefix = data::VerifiedPrefix::default();
            let verification_prefix = std::sync::Mutex::new(verification_prefix);
            Ok(Self(Arc::new(Owner {
                read_metrics: Arc::new(ReadMetrics::default()),
                live: Arc::new(Mutex::new(Some(Live { connection, _lease: lease }))),
                readers: Arc::new(Readers {
                    permits: Arc::new(Semaphore::new(count as usize)),
                    count,
                    idle: std::sync::Mutex::new(idle),
                }),
                closing: AtomicBool::new(false),
                reads: AtomicU64::new(1),
                files: std::sync::Mutex::new(std::collections::BTreeMap::new()),
                verification_prefix,
            })))
        })
        .await?
    }

    pub async fn close(&self) -> Result<(), Error> {
        self.0.closing.store(true, Ordering::Release);
        // Accepted Reads Settle And Return Their Connections Before The Writer Closes.
        let readers = self.0.readers.clone();
        if let Ok(permits) = readers.permits.clone().acquire_many_owned(readers.count).await {
            readers.permits.close();
            drop(permits);
            drop(std::mem::take(&mut *readers.idle.lock().unwrap()));
        }
        let mut live = self.0.live.clone().lock_owned().await;
        let owner = self.0.clone();
        tokio::task::spawn_blocking(move || {
            owner.files.lock().unwrap().clear();
            *owner.verification_prefix.lock().unwrap() = data::VerifiedPrefix::default();
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
        if self.0.closing.load(Ordering::Acquire) {
            return Err(Error::Closed);
        }
        // Acquire before spawning: a cancelled caller cannot leave a queued
        // older write that overtakes its retry or a subsequent transition.
        let mut live = self.0.live.clone().lock_owned().await;
        tokio::task::spawn_blocking(move || {
            let result = live.as_mut().ok_or(Error::Closed).and_then(|live| operation(&mut live.connection).map_err(Error::Sqlite));
            drop(live);
            result
        })
        .await?
    }

    // Runs On A Reader Connection Inside One Read Transaction, So It Observes A Single
    // Committed Snapshot And Never Waits For Or Delays The Writer.
    pub(crate) async fn read<T: Send + 'static>(
        &self, operation: impl FnOnce(&mut Connection) -> Result<T, rusqlite::Error> + Send + 'static,
    ) -> Result<T, Error> {
        if self.0.closing.load(Ordering::Acquire) {
            return Err(Error::Closed);
        }
        let readers = self.0.readers.clone();
        let permit = readers.permits.clone().acquire_owned().await.map_err(|_| Error::Closed)?;
        let reader = readers.idle.lock().unwrap().pop().ok_or(Error::Closed)?;
        let read = self.0.reads.fetch_add(1, Ordering::Relaxed);
        let mut interruption = Interruption { slot: reader.slot.clone(), read };
        let result = tokio::task::spawn_blocking(move || {
            let Reader { mut connection, slot } = reader;
            *slot.running.lock().unwrap() = read;
            let result = read_snapshot(&mut connection, operation);
            *slot.running.lock().unwrap() = 0;
            readers.idle.lock().unwrap().push(Reader { connection, slot });
            drop(permit);
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

fn read_snapshot<T>(
    connection: &mut Connection, operation: impl FnOnce(&mut Connection) -> Result<T, rusqlite::Error>,
) -> Result<T, Error> {
    connection.execute_batch("BEGIN DEFERRED")?;
    let result = operation(connection);
    let finished = if result.is_ok() { connection.execute_batch("COMMIT") } else { connection.execute_batch("ROLLBACK") };
    if finished.is_err() && !connection.is_autocommit() {
        connection.execute_batch("ROLLBACK")?;
    }
    let value = result?;
    finished?;
    Ok(value)
}

// One Layout Check: A Fresh File (True) Or This Build's Exact Layout (False).
fn check_layout(connection: &Connection) -> Result<bool, Error> {
    let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    let tables: bool =
        connection.query_row("SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%')", [], |row| {
            row.get(0)
        })?;
    if version == 0 && !tables {
        return Ok(true);
    }
    if version != SCHEMA_VERSION {
        return Err(Error::Incompatible(format!(
            "local.sqlite declares schema version {version}, and this build reads version {SCHEMA_VERSION}"
        )));
    }
    data::check_format(connection)?;
    Ok(false)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    // Every Reader Holds An Open Snapshot While The Writer Commits 100 Rows: The Commit
    // Never Waits For Them, And Each Held Snapshot Still Reads Its Original Cut.
    #[tokio::test]
    async fn readers_never_hold_the_writer() {
        let root = tempfile::tempdir().unwrap();
        let readers = NonZeroUsize::new(4).unwrap();
        let state = State::open_with(root.path(), readers).await.unwrap();
        state.call(|connection| connection.execute_batch("CREATE TABLE samples(value INTEGER NOT NULL)")).await.unwrap();

        let (entered, mut entries) = tokio::sync::mpsc::unbounded_channel();
        let mut releases = Vec::new();
        let mut held = Vec::new();
        for _ in 0..readers.get() {
            let (release, released) = std::sync::mpsc::channel::<()>();
            releases.push(release);
            let entered = entered.clone();
            let state = state.clone();
            held.push(tokio::spawn(async move {
                state
                    .read(move |connection| {
                        let count =
                            |connection: &Connection| connection.query_row("SELECT count(*) FROM samples", [], |row| row.get::<_, i64>(0));
                        let before = count(connection)?;
                        entered.send(()).unwrap();
                        released.recv().unwrap();
                        Ok((before, count(connection)?))
                    })
                    .await
                    .unwrap()
            }));
        }
        for _ in 0..readers.get() {
            entries.recv().await.unwrap();
        }

        let saved = state
            .call(|connection| {
                let transaction = connection.transaction()?;
                for value in 0..100 {
                    transaction.execute("INSERT INTO samples VALUES (?1)", [value])?;
                }
                commit(transaction)?;
                connection.query_row("SELECT count(*) FROM samples", [], |row| row.get::<_, i64>(0))
            })
            .await
            .unwrap();
        assert_eq!(saved, 100);

        for release in releases {
            release.send(()).unwrap();
        }
        for reader in held {
            assert_eq!(reader.await.unwrap(), (0, 0));
        }
        let fresh = state.read(|connection| connection.query_row("SELECT count(*) FROM samples", [], |row| row.get::<_, i64>(0))).await;
        assert_eq!(fresh.unwrap(), 100);
        state.close().await.unwrap();
        assert!(matches!(state.read(|_| Ok(())).await, Err(Error::Closed)));
    }
}
