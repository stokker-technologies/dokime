/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Process entry for Dokime binaries: one runtime, on threads with a stack
// large enough for unoptimized async state machines on every platform.
//
// Windows gives the main thread one megabyte, an eighth of what Linux does,
// and a dev-profile server or plugin future overflows it before serving its
// first request. The future therefore runs on a dedicated thread, and the
// runtime's workers get the same stack, so the same binary behaves alike in
// development and release on every operating system.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::future::Future;

// Third Party Imports

// Local Imports

/* === Constants === */

// Thirty-two megabytes covers the deepest dev-profile futures observed with a
// wide margin; release binaries use a fraction of it.
pub const STACK_BYTES: usize = 32 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

// Runs the future to completion on a multi-threaded runtime whose driving and
// worker threads all have `STACK_BYTES` of stack. A panic inside the future
// propagates to the caller as it would from `block_on`.
pub fn block_on<F>(future: F) -> std::io::Result<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    // Run On A Sized Thread
    let driver = std::thread::Builder::new().name("dokime-main".to_owned()).stack_size(STACK_BYTES).spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().thread_stack_size(STACK_BYTES).build()?;
        Ok::<_, std::io::Error>(runtime.block_on(future))
    })?;

    // Surface The Outcome
    match driver.join() {
        Ok(outcome) => outcome,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

/* === Private Functions === */

/* === Tests === */
