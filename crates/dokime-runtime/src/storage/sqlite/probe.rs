/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Opt-In SQLite xSync Counts And Durations

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;
use std::sync::{
    Mutex, OnceLock,
    atomic::{AtomicU64, Ordering},
};
use std::time::Instant;

// Third Party Imports
use rusqlite::{Connection, ffi};

// Local Imports

/* === Constants === */

static SYNCS: [AtomicU64; 3] = [const { AtomicU64::new(0) }; 3];
static NANOS: [AtomicU64; 3] = [const { AtomicU64::new(0) }; 3];

/* === Macros === */

/* === Type Aliases === */

type Sync = unsafe extern "C" fn(*mut ffi::sqlite3_file, i32) -> i32;

/* === Structs === */

#[derive(Default)]
pub(super) struct Probe {
    methods: BTreeMap<usize, Box<ffi::sqlite3_io_methods>>,
}

#[derive(Clone, Copy, Debug, serde::Serialize)]
pub struct SyncCounters {
    pub wal: u64,
    pub database: u64,
    pub journal: u64,
    pub wal_ns: u64,
    pub database_ns: u64,
    pub journal_ns: u64,
}

#[derive(Clone, Copy)]
struct Original {
    sync: Sync,
    kind: usize,
    table: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Probe {
    pub(super) fn attach(&mut self, connection: &Connection) -> rusqlite::Result<()> {
        if std::env::var_os("DOKIME_STORAGE_PROBE").is_none() {
            return Ok(());
        }
        let mode: String = connection.pragma_query_value(None, "journal_mode", |row| row.get(0))?;
        for (control, kind) in [(ffi::SQLITE_FCNTL_FILE_POINTER, 1), (ffi::SQLITE_FCNTL_JOURNAL_POINTER, if mode == "wal" { 0 } else { 2 })]
        {
            let mut file: *mut ffi::sqlite3_file = std::ptr::null_mut();
            // SAFETY: The Exclusive Connection Supplies A Live File Pointer Into This Slot.
            let result = unsafe {
                ffi::sqlite3_file_control(connection.handle(), c"main".as_ptr(), control, (&mut file as *mut *mut ffi::sqlite3_file).cast())
            };
            if result != ffi::SQLITE_OK || file.is_null() || self.methods.contains_key(&(file as usize)) {
                continue;
            }
            // SAFETY: SQLite Owns The File; Its Immutable Method Table Contains Only
            // Function Pointers. Our Owned Copy Outlives Connection.close And Retains
            // Every Original Operation, Delegating xSync With The Unchanged Arguments.
            unsafe {
                if (*file).pMethods.is_null() {
                    continue;
                }
                let mut methods = Box::new(std::ptr::read((*file).pMethods));
                let Some(sync) = methods.xSync else {
                    continue;
                };
                originals().lock().unwrap().insert(
                    file as usize,
                    Original {
                        sync,
                        kind,
                        table: (&*methods as *const ffi::sqlite3_io_methods) as usize,
                    },
                );
                methods.xSync = Some(measure_sync);
                (*file).pMethods = &*methods;
                self.methods.insert(file as usize, methods);
            }
        }
        Ok(())
    }
}

impl Drop for Probe {
    fn drop(&mut self) {
        let mut originals = originals().lock().unwrap();
        for (file, methods) in &self.methods {
            let table = (&**methods as *const ffi::sqlite3_io_methods) as usize;
            if originals.get(file).is_some_and(|original| original.table == table) {
                originals.remove(file);
            }
        }
    }
}

/* === Public Functions === */

pub(super) fn counters() -> SyncCounters {
    SyncCounters {
        wal: SYNCS[0].load(Ordering::Relaxed),
        database: SYNCS[1].load(Ordering::Relaxed),
        journal: SYNCS[2].load(Ordering::Relaxed),
        wal_ns: NANOS[0].load(Ordering::Relaxed),
        database_ns: NANOS[1].load(Ordering::Relaxed),
        journal_ns: NANOS[2].load(Ordering::Relaxed),
    }
}

/* === Private Functions === */

fn originals() -> &'static Mutex<BTreeMap<usize, Original>> {
    static ORIGINALS: OnceLock<Mutex<BTreeMap<usize, Original>>> = OnceLock::new();
    ORIGINALS.get_or_init(Mutex::default)
}

unsafe extern "C" fn measure_sync(file: *mut ffi::sqlite3_file, flags: i32) -> i32 {
    let original = originals().lock().ok().and_then(|originals| originals.get(&(file as usize)).copied());
    let Some(original) = original else { return ffi::SQLITE_IOERR_FSYNC };
    let started = Instant::now();
    // SAFETY: This Is SQLite's Original xSync For This Exact Live File.
    let result = unsafe { (original.sync)(file, flags) };
    SYNCS[original.kind].fetch_add(1, Ordering::Relaxed);
    NANOS[original.kind].fetch_add(started.elapsed().as_nanos() as u64, Ordering::Relaxed);
    result
}

/* === Tests === */
