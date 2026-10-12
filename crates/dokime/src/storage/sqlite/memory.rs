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
    alloc::{Layout, alloc, dealloc},
    cell::RefCell,
    ffi::{c_int, c_void},
    marker::PhantomData,
    num::NonZeroUsize,
    ptr,
    rc::Rc,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

// Third Party Imports
use rusqlite::ffi;

// Local Imports
use super::Error;

/* === Constants === */
const ALIGNMENT: usize = 8;
const HEADER_BYTES: usize = std::mem::size_of::<Header>();
static INSTALLATION: OnceLock<c_int> = OnceLock::new();
thread_local! {
    static CURRENT: RefCell<Option<Arc<Budget>>> = const { RefCell::new(None) };
}

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Budget {
    state: Mutex<Usage>,
    refused: AtomicBool,
}

struct Usage {
    used: usize,
    ceiling: usize,
}

pub(crate) struct Reservation {
    owner: Arc<Budget>,
    bytes: usize,
}

pub(super) struct Ceiling {
    owner: Arc<Budget>,
    previous: usize,
}

pub(super) struct Context {
    previous: Option<Arc<Budget>>,
    thread: PhantomData<Rc<()>>,
}

#[repr(C, align(8))]
struct Header {
    owner: *const Budget,
    bytes: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Budget {
    pub(super) fn new(ceiling: NonZeroUsize) -> Arc<Self> {
        Arc::new(Self {
            state: Mutex::new(Usage {
                used: 0,
                ceiling: ceiling.get(),
            }),
            refused: AtomicBool::new(false),
        })
    }

    pub(super) fn restrict(self: &Arc<Self>, ceiling: NonZeroUsize) -> Result<Ceiling, Error> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let ceiling = ceiling.get().min(state.ceiling);
        if state.used > ceiling {
            return Err(Error::Data("retained SQLite memory exceeds query bound"));
        }
        let previous = state.ceiling;
        state.ceiling = ceiling;
        Ok(Ceiling {
            owner: self.clone(),
            previous,
        })
    }

    pub(super) fn enter(self: &Arc<Self>) -> Context {
        let previous = CURRENT.with(|current| current.replace(Some(self.clone())));
        Context {
            previous,
            thread: PhantomData,
        }
    }

    pub(super) fn begin(&self) {
        self.refused.store(false, Ordering::Release);
    }
    pub(super) fn refused(&self) -> bool {
        self.refused.load(Ordering::Acquire)
    }

    pub(super) fn classify(&self, error: Error) -> Error {
        match error {
            Error::Sqlite(error)
                if self.refused()
                    && matches!(&error, rusqlite::Error::SqliteFailure(error, _)
                    if error.code == rusqlite::ErrorCode::OutOfMemory) =>
            {
                Error::Memory(error)
            }
            other => other,
        }
    }

    pub(super) fn reserve_owned(
        self: &Arc<Self>,
        bytes: usize,
        ceiling: usize,
    ) -> Result<Reservation, Error> {
        if !self.reserve_up_to(bytes, ceiling) {
            return Err(Error::Data("owned query state exceeds memory bound"));
        }
        Ok(Reservation {
            owner: self.clone(),
            bytes,
        })
    }

    fn reserve(&self, bytes: usize) -> bool {
        self.reserve_up_to(bytes, usize::MAX)
    }

    fn reserve_up_to(&self, bytes: usize, ceiling: usize) -> bool {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let next = state
            .used
            .checked_add(bytes)
            .filter(|next| *next <= ceiling.min(state.ceiling));
        let Some(next) = next else {
            self.refused.store(true, Ordering::Release);
            return false;
        };
        state.used = next;
        true
    }

    fn release(&self, bytes: usize) {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        // A header or reservation owns one release. An impossible double
        // release fails closed without unwinding across SQLite's C callback.
        state.used = state
            .used
            .checked_sub(bytes)
            .unwrap_or_else(|| std::process::abort());
    }
}

impl Reservation {
    pub(super) fn shrink(&mut self, bytes: usize) -> Result<(), Error> {
        let released = self.bytes.checked_sub(bytes).ok_or(Error::Data(
            "retained allocation exceeds its construction reservation",
        ))?;
        self.owner.release(released);
        self.bytes = bytes;
        Ok(())
    }
}

impl Drop for Reservation {
    fn drop(&mut self) {
        self.owner.release(self.bytes);
    }
}

impl Drop for Ceiling {
    fn drop(&mut self) {
        self.owner
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .ceiling = self.previous;
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        CURRENT.with(|current| {
            current.replace(self.previous.take());
        });
    }
}

/* === Public Functions === */
pub(super) fn install() -> Result<(), Error> {
    let code = *INSTALLATION.get_or_init(|| {
        let methods = ffi::sqlite3_mem_methods {
            xMalloc: Some(malloc),
            xFree: Some(free),
            xRealloc: Some(reallocate),
            xSize: Some(size),
            xRoundup: Some(roundup),
            xInit: Some(initialize),
            xShutdown: Some(shutdown),
            pAppData: ptr::null_mut(),
        };
        // SAFETY: SQLite copies this method table during pre-initialization
        // configuration. Every callback remains available for the process life.
        unsafe { ffi::sqlite3_config(ffi::SQLITE_CONFIG_MALLOC, &methods) }
    });
    if code != ffi::SQLITE_OK {
        return Err(Error::Allocator(rusqlite::Error::SqliteFailure(
            ffi::Error::new(code),
            None,
        )));
    }
    Ok(())
}

/* === Private Functions === */
unsafe extern "C" fn initialize(_context: *mut c_void) -> c_int {
    // SQLite invokes xInit unconditionally for a configured allocator. All
    // Rust allocator and ownership state initializes independently of SQLite.
    ffi::SQLITE_OK
}

unsafe extern "C" fn shutdown(_context: *mut c_void) {}

fn current() -> Option<Arc<Budget>> {
    CURRENT
        .try_with(|current| current.try_borrow().ok().and_then(|owner| owner.clone()))
        .ok()
        .flatten()
}

fn allocation(bytes: usize, owner: Option<Arc<Budget>>) -> *mut c_void {
    let Some(total) = bytes.checked_add(HEADER_BYTES) else {
        return ptr::null_mut();
    };
    let Ok(layout) = Layout::from_size_align(total, ALIGNMENT) else {
        return ptr::null_mut();
    };
    if owner.as_ref().is_some_and(|owner| !owner.reserve(total)) {
        return ptr::null_mut();
    }
    // SAFETY: The nonzero layout includes Header and the requested payload.
    // Allocation failures release the reservation and return SQLite's null ABI.
    let base = unsafe { alloc(layout) };
    if base.is_null() {
        if let Some(owner) = owner {
            owner.release(total);
        }
        return ptr::null_mut();
    }
    let owner = owner.map(Arc::into_raw).unwrap_or(ptr::null());
    // SAFETY: Header's alignment is the SQLite-required eight bytes, its size
    // is a multiple of eight, and both header/payload fit the allocated layout.
    unsafe {
        base.cast::<Header>().write(Header { owner, bytes });
        base.add(HEADER_BYTES).cast()
    }
}

unsafe extern "C" fn malloc(bytes: c_int) -> *mut c_void {
    if bytes <= 0 {
        return ptr::null_mut();
    }
    allocation(bytes as usize, current())
}

unsafe extern "C" fn free(value: *mut c_void) {
    if value.is_null() {
        return;
    }
    // SAFETY: SQLite only frees pointers returned by this installed allocator.
    // Its preceding initialized Header owns exactly one Arc strong reference.
    unsafe {
        let base = value.cast::<u8>().sub(HEADER_BYTES);
        let header = base.cast::<Header>().read();
        if !header.owner.is_null() {
            let owner = Arc::from_raw(header.owner);
            owner.release(header.bytes + HEADER_BYTES);
        }
        let layout = Layout::from_size_align_unchecked(header.bytes + HEADER_BYTES, ALIGNMENT);
        dealloc(base, layout);
    }
}

unsafe extern "C" fn reallocate(value: *mut c_void, bytes: c_int) -> *mut c_void {
    if value.is_null() {
        return unsafe { malloc(bytes) };
    }
    if bytes <= 0 {
        unsafe {
            free(value);
        }
        return ptr::null_mut();
    }
    // SAFETY: The source allocation remains live until copying succeeds. Its
    // header owns a strong reference; incrementing it produces an independent
    // owner for the replacement if no calling SQLite context is present.
    unsafe {
        let header = &*value.cast::<u8>().sub(HEADER_BYTES).cast::<Header>();
        let owner = current().or_else(|| {
            if header.owner.is_null() {
                return None;
            }
            Arc::increment_strong_count(header.owner);
            Some(Arc::from_raw(header.owner))
        });
        // Allocate-before-free accounts for the actual peak of this copying
        // implementation. Refusal preserves SQLite's original allocation.
        let replacement = allocation(bytes as usize, owner);
        if replacement.is_null() {
            return ptr::null_mut();
        }
        ptr::copy_nonoverlapping(
            value.cast::<u8>(),
            replacement.cast::<u8>(),
            header.bytes.min(bytes as usize),
        );
        free(value);
        replacement
    }
}

unsafe extern "C" fn size(value: *mut c_void) -> c_int {
    if value.is_null() {
        return 0;
    }
    // SAFETY: SQLite's xSize receives a live allocator payload pointer. Payload
    // sizes originate from positive c_int requests and fit that return type.
    unsafe { (*value.cast::<u8>().sub(HEADER_BYTES).cast::<Header>()).bytes as c_int }
}

unsafe extern "C" fn roundup(bytes: c_int) -> c_int {
    if bytes <= 0 {
        return 0;
    }
    bytes
        .checked_add((ALIGNMENT - 1) as c_int)
        .map(|value| value & !((ALIGNMENT - 1) as c_int))
        .unwrap_or(0)
}

/* === Tests === */
