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
mod memory;
pub(crate) mod query;

/* === Re-Exports === */
pub use crate::storage::error::Error;
pub use catalog::Source;
pub use data::{Backend, Limits, Recovery, RecoveryLimits};
pub use rusqlite;

/* === Imports === */
// Standard Library Imports
use std::{
    fs::{File, OpenOptions},
    num::NonZeroUsize,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
};
// Third Party Imports
use rusqlite::Connection;
use tokio::sync::Mutex;
// Local Imports

/* === Constants === */
const SCHEMA_VERSION: i64 = 1;
const SQLITE_MEMORY_BYTES: usize = 32 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone)]
pub struct State(Arc<Owner>);

struct Owner {
    live: Arc<Mutex<Option<Live>>>,
    closing: AtomicBool,
    data_pins: AtomicUsize,
    memory: Arc<memory::Budget>,
    index_schema: std::sync::Mutex<Option<data::IndexSchema>>,
    scanned: AtomicU64,
}

pub(crate) struct DataPin {
    owner: Arc<Owner>,
}

struct Live {
    connection: Connection,
    lease: File,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Drop for DataPin {
    fn drop(&mut self) {
        self.owner.data_pins.fetch_sub(1, Ordering::AcqRel);
    }
}

impl State {
    pub(crate) fn pin_data(&self, _connection: &Connection) -> Result<DataPin, Error> {
        if self.0.closing.load(Ordering::Acquire) {
            return Err(Error::Closed);
        }
        self.0
            .data_pins
            .try_update(Ordering::AcqRel, Ordering::Acquire, |pins| {
                pins.checked_add(1)
            })
            .map_err(|_| Error::Data("live data pin count overflow"))?;
        Ok(DataPin {
            owner: self.0.clone(),
        })
    }

    pub fn scanned_rows(&self) -> u64 {
        self.0.scanned.load(Ordering::Acquire)
    }

    pub(crate) fn has_data_pins(&self) -> bool {
        self.0.data_pins.load(Ordering::Acquire) != 0
    }

    pub async fn open(root: impl AsRef<Path>) -> Result<Self, Error> {
        Self::open_with_memory(root, NonZeroUsize::new(SQLITE_MEMORY_BYTES).unwrap()).await
    }

    pub(crate) async fn open_with_memory(
        root: impl AsRef<Path>,
        ceiling: NonZeroUsize,
    ) -> Result<Self, Error> {
        initialize_sqlite()?;
        let memory = memory::Budget::new(ceiling);
        let root = root.as_ref().to_path_buf();
        tokio::task::spawn_blocking(move || {
            let _context = memory.enter();
            std::fs::create_dir_all(&root)?;
            let lease = OpenOptions::new()
                .create(true)
                .truncate(false)
                .read(true)
                .write(true)
                .open(root.join("local.lock"))?;
            lease.try_lock().map_err(|error| match error {
                std::fs::TryLockError::WouldBlock => Error::Owned,
                std::fs::TryLockError::Error(error) => Error::Io(error),
            })?;

            // Recovery Shares `local.sqlite`; An Older Separate Journal Is Never Merged.
            if root.join("recovery.sqlite").exists() {
                return Err(Error::Conflict(
                    "a separate recovery database requires a fresh storage root",
                ));
            }
            let mut connection = Connection::open(root.join("local.sqlite"))?;
            check_version(&connection)?;
            data::check_format(&connection)?;
            let retired: bool = connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE name IN
                 ('storage_operations','storage_deletions'))",
                [],
                |row| row.get(0),
            )?;
            if retired {
                return Err(Error::Conflict(
                    "retired storage tables require a fresh root",
                ));
            }

            connection.execute_batch(
                "PRAGMA journal_mode=WAL; PRAGMA synchronous=FULL;
                PRAGMA foreign_keys=ON; PRAGMA temp_store=MEMORY;
                PRAGMA threads=0; PRAGMA cache_size=-512;",
            )?;

            let transaction = connection.transaction()?;
            transaction.execute_batch(catalog::SCHEMA)?;
            transaction.pragma_update(None, "user_version", SCHEMA_VERSION)?;
            transaction.commit()?;
            data::recovery::initialize(&mut connection)?;

            Ok(Self(Arc::new(Owner {
                live: Arc::new(Mutex::new(Some(Live { connection, lease }))),
                closing: AtomicBool::new(false),
                data_pins: AtomicUsize::new(0),
                memory,
                index_schema: std::sync::Mutex::new(None),
                scanned: AtomicU64::new(0),
            })))
        })
        .await?
    }

    pub async fn close(&self) -> Result<(), Error> {
        self.0.closing.store(true, Ordering::Release);
        let mut live = self.0.live.clone().lock_owned().await;
        let memory = self.0.memory.clone();
        let owner = self.0.clone();
        tokio::task::spawn_blocking(move || {
            let _context = memory.enter();
            *owner.index_schema.lock().unwrap() = None;
            let Some(Live { connection, lease }) = live.take() else {
                return Ok(());
            };
            match connection.close() {
                Ok(()) => {
                    drop(lease);
                    Ok(())
                }
                Err((connection, error)) => {
                    *live = Some(Live { connection, lease });
                    Err(Error::Sqlite(error))
                }
            }
        })
        .await?
    }

    pub async fn call<T: Send + 'static>(
        &self,
        operation: impl FnOnce(&mut Connection) -> Result<T, rusqlite::Error> + Send + 'static,
    ) -> Result<T, Error> {
        self.dispatch(None, None, operation).await
    }

    pub(crate) async fn call_recovery<T: Send + 'static>(
        &self,
        operation: impl FnOnce(&mut Connection) -> Result<T, rusqlite::Error> + Send + 'static,
    ) -> Result<T, Error> {
        self.dispatch(None, None, operation).await
    }

    pub(crate) fn reserve_query(
        &self,
        bytes: usize,
        ceiling: NonZeroUsize,
    ) -> Result<memory::Reservation, Error> {
        self.0.memory.reserve_owned(bytes, ceiling.get())
    }

    pub(crate) fn reserve_memory(&self, bytes: usize) -> Result<memory::Reservation, Error> {
        if self.0.closing.load(Ordering::Acquire) {
            return Err(Error::Closed);
        }
        self.0.memory.reserve_owned(bytes, usize::MAX)
    }

    pub(crate) async fn call_read<T: Send + 'static>(
        &self,
        query: Option<Arc<query::Budget>>,
        operation: impl FnOnce(&mut Connection) -> Result<Result<T, Error>, rusqlite::Error>
        + Send
        + 'static,
    ) -> Result<Result<T, Error>, Error> {
        let ceiling = query.as_ref().map(|query| query.limits.memory_bytes);
        let allocation = self.0.memory.clone();
        // Inspect Both Result Layers Before Another Physical Call Resets Refusal
        self.dispatch(ceiling, query, move |connection| {
            operation(connection).map(|result| result.map_err(|error| allocation.classify(error)))
        })
        .await
    }

    async fn dispatch<T: Send + 'static>(
        &self,
        ceiling: Option<NonZeroUsize>,
        query: Option<Arc<query::Budget>>,
        operation: impl FnOnce(&mut Connection) -> Result<T, rusqlite::Error> + Send + 'static,
    ) -> Result<T, Error> {
        let owner = self.0.clone();
        if owner.closing.load(Ordering::Acquire) {
            return Err(Error::Closed);
        }
        // Acquire before spawning: a cancelled caller cannot leave a queued
        // older write that overtakes its retry or a subsequent transition.
        let mut live = owner.live.clone().lock_owned().await;
        if owner.closing.load(Ordering::Acquire) {
            return Err(Error::Closed);
        }
        tokio::task::spawn_blocking(move || {
            let _context = owner.memory.enter();
            owner.memory.begin();
            let _ceiling = ceiling
                .map(|ceiling| owner.memory.restrict(ceiling))
                .transpose()?;
            let result = live.as_mut().ok_or(Error::Closed).and_then(|live| {
                let connection = &mut live.connection;
                let result = match query {
                    Some(query) => query.run(connection, operation),
                    None => operation(connection).map_err(Error::Sqlite),
                };
                result.map_err(|error| owner.memory.classify(error))
            });
            drop(live);
            drop(owner);
            result
        })
        .await?
    }
}

/* === Public Functions === */
pub fn initialize_sqlite() -> Result<(), Error> {
    memory::install()
}

/* === Private Functions === */
fn check_version(connection: &Connection) -> Result<(), Error> {
    let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version == SCHEMA_VERSION {
        return Ok(());
    }
    if version != 0 {
        return Err(Error::Schema(version));
    }
    let unversioned: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%')",
        [], |row| row.get(0),
    )?;
    if unversioned {
        return Err(Error::Conflict(
            "unversioned database requires a fresh storage root",
        ));
    }
    Ok(())
}

/* === Tests === */
