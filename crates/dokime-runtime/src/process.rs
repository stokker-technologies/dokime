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

pub const STACK_BYTES: usize = 32 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

// Runs the future to completion on a multi-threaded runtime whose driving and
// worker threads use the supplied execution stack policy. A panic inside the future
// propagates to the caller as it would from `block_on`.
pub fn block_on<F>(future: F) -> std::io::Result<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    let stack_bytes = STACK_BYTES;
    let shutdown = std::time::Duration::from_millis(250);

    // Run On A Sized Thread
    let driver = std::thread::Builder::new().name("dokime-main".to_owned()).stack_size(stack_bytes).spawn(move || {
        let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().thread_stack_size(stack_bytes).build()?;
        let outcome = runtime.block_on(future);
        // A collector still blocked in the pool runs out its own bound without holding the exit.
        runtime.shutdown_timeout(shutdown);
        Ok::<_, std::io::Error>(outcome)
    })?;

    // Surface The Outcome
    match driver.join() {
        Ok(outcome) => outcome,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

/* === Private Functions === */

/* === Tests === */
