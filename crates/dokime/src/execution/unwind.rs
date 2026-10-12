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
use std::future::Future;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::pin::Pin;
use std::task::{Context, Poll};
// Third Party Imports

// Local Imports
use crate::plugin::{Diagnostic, panicked};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct Caught<F> {
    work: F,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl<F, T> Future for Caught<F>
where
    F: Future<Output = Result<T, Diagnostic>> + Unpin,
{
    type Output = Result<T, Diagnostic>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        // Caught per poll, not per await: an async step panics inside whichever
        // poll reaches the failing line, and the `.await` above it never returns.
        match catch_unwind(AssertUnwindSafe(|| Pin::new(&mut self.work).poll(ctx))) {
            Ok(polled) => polled,
            Err(carried) => Poll::Ready(Err(panicked(carried.as_ref()))),
        }
    }
}

/* === Public Functions === */
pub(crate) fn caught<F, T>(work: F) -> Caught<F>
where
    F: Future<Output = Result<T, Diagnostic>> + Unpin,
{
    Caught { work }
}

/* === Private Functions === */

/* === Tests === */
