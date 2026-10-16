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
use std::{
    cell::{Cell, RefCell},
    ffi::{c_int, c_void},
    marker::PhantomData,
    ptr,
    rc::Rc,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

// Third Party Imports
use harmos::storage::QueryLimits;
use rusqlite::{Connection, ffi};

// Local Imports
use super::Error;

/* === Constants === */
const PROGRESS_STEPS: c_int = 256;
thread_local! {
    static CURRENT: Cell<Option<Deadline>> = const { Cell::new(None) };
    static CANCELLED: RefCell<Option<Arc<AtomicBool>>> = const { RefCell::new(None) };
}

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct Budget {
    pub limits: QueryLimits,
    remaining: Mutex<Duration>,
}

#[derive(Clone, Copy)]
struct Deadline {
    started: Instant,
    allowance: Duration,
}

struct Scope(Option<Deadline>);

struct Progress {
    connection: *mut ffi::sqlite3,
}

struct Active<'a> {
    remaining: std::sync::MutexGuard<'a, Duration>,
    started: Instant,
}

pub(super) struct Cancellation(Arc<AtomicBool>);

pub(super) struct CancellationScope {
    previous: Option<Arc<AtomicBool>>,
    _thread: PhantomData<Rc<()>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Cancellation {
    pub(super) fn new() -> Self {
        Self(Arc::new(AtomicBool::new(false)))
    }
    pub(super) fn flag(&self) -> Arc<AtomicBool> {
        self.0.clone()
    }
}
impl Drop for Cancellation {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}
impl Drop for CancellationScope {
    fn drop(&mut self) {
        CANCELLED.with(|current| current.replace(self.previous.take()));
    }
}

impl Budget {
    pub(crate) fn new(limits: QueryLimits) -> Result<Self, Error> {
        limits.check()?;
        // All temporary SQLite storage remains in the accounted heap. A larger
        // caller spill allowance permits this zero-spill implementation too.
        Ok(Self {
            limits,
            remaining: Mutex::new(limits.timeout),
        })
    }

    pub(super) fn prepare<T>(
        &self,
        operation: impl FnOnce() -> Result<T, Error>,
    ) -> Result<T, Error> {
        let remaining = self
            .remaining
            .lock()
            .map_err(|_| Error::Corrupt("query execution budget is poisoned"))?;
        if remaining.is_zero() {
            return Err(Error::QueryTimeout(None));
        }
        let active = Active {
            remaining,
            started: Instant::now(),
        };
        let allowance = *active.remaining;
        if active.started.elapsed() >= allowance {
            return Err(Error::QueryTimeout(None));
        }
        let result = operation();
        if active.started.elapsed() >= allowance {
            return Err(Error::QueryTimeout(None));
        }
        result
    }

    pub(super) fn run<T>(
        &self,
        connection: &mut Connection,
        operation: impl FnOnce(&mut Connection) -> Result<T, rusqlite::Error>,
    ) -> Result<T, Error> {
        let remaining = self
            .remaining
            .lock()
            .map_err(|_| Error::Corrupt("query execution budget is poisoned"))?;
        if remaining.is_zero() {
            return Err(Error::QueryTimeout(None));
        }
        let active = Active {
            remaining,
            started: Instant::now(),
        };
        let deadline = Deadline {
            started: active.started,
            allowance: *active.remaining,
        };
        // Borrow This Original Deadline For Bounded Non-SQLite Work
        let _scope = Scope(CURRENT.with(|current| current.replace(Some(deadline))));
        // SAFETY: The connection is live and exclusively borrowed throughout
        // this synchronous operation. SQLite worker threads are disabled. The
        // callback only borrows deadline, which outlives progress registration.
        let connection_handle = unsafe { connection.handle() };
        let progress = Progress {
            connection: connection_handle,
        };
        unsafe {
            ffi::sqlite3_progress_handler(
                connection_handle,
                PROGRESS_STEPS,
                Some(expired),
                (&deadline as *const Deadline).cast_mut().cast(),
            );
        }
        checkpoint()?;
        if deadline.started.elapsed() >= deadline.allowance {
            return Err(Error::QueryTimeout(None));
        }
        let result = operation(connection);
        // Clear before inspecting or dropping the stack-owned callback state,
        // including when unwinding: Progress drops before Deadline and Active.
        drop(progress);
        if cancelled() {
            return Err(Error::Runtime(harmos::Error::ViewExpired));
        }
        if deadline.started.elapsed() >= deadline.allowance {
            return Err(Error::QueryTimeout(result.err()));
        }
        result.map_err(Error::Sqlite)
    }
}

impl Drop for Active<'_> {
    fn drop(&mut self) {
        *self.remaining = self.remaining.saturating_sub(self.started.elapsed());
    }
}

impl Drop for Scope {
    fn drop(&mut self) {
        CURRENT.with(|current| current.set(self.0));
    }
}

impl Drop for Progress {
    fn drop(&mut self) {
        // SAFETY: Budget::run retains the exclusive connection borrow until
        // this guard drops. Clearing also removes SQLite's pointer to Deadline.
        unsafe {
            ffi::sqlite3_progress_handler(self.connection, 0, None, ptr::null_mut());
        }
    }
}

/* === Public Functions === */
pub(super) fn checkpoint() -> Result<(), Error> {
    if cancelled() {
        return Err(Error::Runtime(harmos::Error::ViewExpired));
    }
    let expired = CURRENT.with(|current| {
        current
            .get()
            .is_some_and(|deadline| deadline.started.elapsed() >= deadline.allowance)
    });
    if expired {
        return Err(Error::QueryTimeout(None));
    }
    Ok(())
}

pub(super) fn cancellation(flag: Arc<AtomicBool>) -> CancellationScope {
    CancellationScope {
        previous: CANCELLED.with(|current| current.replace(Some(flag))),
        _thread: PhantomData,
    }
}

/* === Private Functions === */
fn cancelled() -> bool {
    CANCELLED.with(|current| {
        current
            .borrow()
            .as_ref()
            .is_some_and(|flag| flag.load(Ordering::Acquire))
    })
}
unsafe extern "C" fn expired(context: *mut c_void) -> c_int {
    // SAFETY: Only Budget::run registers this callback. Its Deadline remains
    // at a stable stack address until the callback is cleared. No allocation,
    // lock or panic-producing operation occurs across this C ABI boundary.
    let deadline = unsafe { &*context.cast::<Deadline>() };
    c_int::from(cancelled() || deadline.started.elapsed() >= deadline.allowance)
}

/* === Tests === */
