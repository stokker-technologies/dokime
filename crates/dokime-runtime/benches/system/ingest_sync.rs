/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Benchmark-Only SQLite Sync Observation With Native VFS Delegation

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{
    OnceLock,
    atomic::{AtomicPtr, AtomicU64, Ordering},
};

// Third Party Imports
use rusqlite::ffi;

// Local Imports

/* === Constants === */

static ORIGINAL: AtomicPtr<ffi::sqlite3_vfs> = AtomicPtr::new(std::ptr::null_mut());
static OFFSET: OnceLock<usize> = OnceLock::new();
static SYNCS: AtomicU64 = AtomicU64::new(0);
static WAL_SYNCS: AtomicU64 = AtomicU64::new(0);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[repr(C)]
struct Observed {
    methods: ffi::sqlite3_io_methods,
    original: *const ffi::sqlite3_io_methods,
    wal: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

// Install Before Any Runtime Opens SQLite. SQLite Retains This VFS For The Process Lifetime.
pub fn install() {
    unsafe {
        let original = ffi::sqlite3_vfs_find(std::ptr::null());
        assert!(!original.is_null());
        let mut observer = Box::new(*original);
        let alignment = std::mem::align_of::<Observed>();
        let offset = ((*original).szOsFile as usize).next_multiple_of(alignment);
        OFFSET.set(offset).unwrap();
        observer.szOsFile = i32::try_from(offset + std::mem::size_of::<Observed>()).unwrap();
        observer.zName = c"dokime-ingest-observer".as_ptr();
        observer.pNext = std::ptr::null_mut();
        ORIGINAL.store(original, Ordering::Relaxed);
        observer.xOpen = Some(open);
        assert_eq!(ffi::sqlite3_vfs_register(Box::into_raw(observer), 1), ffi::SQLITE_OK);
    }
}

pub fn counts() -> (u64, u64) {
    (SYNCS.load(Ordering::Relaxed), WAL_SYNCS.load(Ordering::Relaxed))
}

/* === Private Functions === */

// The Extra Method Table Lives In SQLite's Own File Allocation. All Native Methods
// Still Receive Their Original File Layout; Only xSync Is Observed, Then Delegated.
unsafe extern "C" fn open(
    _vfs: *mut ffi::sqlite3_vfs, name: ffi::sqlite3_filename, file: *mut ffi::sqlite3_file, flags: i32, out: *mut i32,
) -> i32 {
    unsafe {
        let original = ORIGINAL.load(Ordering::Relaxed);
        let result = ((*original).xOpen.unwrap())(original, name, file, flags, out);
        if result != ffi::SQLITE_OK || (*file).pMethods.is_null() {
            return result;
        }
        let methods = (*file).pMethods;
        let observed = file.cast::<u8>().add(*OFFSET.get().unwrap()).cast::<Observed>();
        observed.write(Observed {
            methods: *methods,
            original: methods,
            wal: flags & ffi::SQLITE_OPEN_WAL != 0,
        });
        (*observed).methods.xSync = Some(sync);
        (*file).pMethods = std::ptr::addr_of!((*observed).methods);
        result
    }
}

unsafe extern "C" fn sync(file: *mut ffi::sqlite3_file, flags: i32) -> i32 {
    unsafe {
        let observed = &*file.cast::<u8>().add(*OFFSET.get().unwrap()).cast::<Observed>();
        let _span = tracing::debug_span!("ingest.sqlite.sync").entered();
        let result = ((*observed.original).xSync.unwrap())(file, flags);
        if result == ffi::SQLITE_OK {
            SYNCS.fetch_add(1, Ordering::Relaxed);
            if observed.wal {
                WAL_SYNCS.fetch_add(1, Ordering::Relaxed);
            }
        }
        result
    }
}

/* === Tests === */
