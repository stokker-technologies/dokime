/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod source;

/* === Re-Exports === */
pub use source::CapturedRecord;
pub(crate) use source::{Process, Projection, Source, group_limits};

/* === Imports === */
// Standard Library Imports
use std::time::Duration;
// Third Party Imports

// Local Imports
use crate::Error;

/* === Constants === */
/// The first pause after the read ledger refuses a reader.
const CAPACITY_BACKOFF_FIRST: Duration = Duration::from_millis(10);
/// The longest pause between retries of a refused reader.
const CAPACITY_BACKOFF_MAX: Duration = Duration::from_millis(100);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// Paces a reader the shared storage read ledger refused: each retry waits
/// twice as long as the last, up to a cap, so contended readers yield the
/// ledger to the views that hold it instead of spinning against it.
#[derive(Debug)]
pub(crate) struct CapacityBackoff {
    delay: Duration,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl CapacityBackoff {
    pub(crate) fn new() -> Self {
        Self {
            delay: CAPACITY_BACKOFF_FIRST,
        }
    }

    /// Waits before the next retry.
    pub(crate) async fn wait(&mut self) {
        tokio::time::sleep(self.advance()).await;
    }

    /// Waits before the next retry unless that retry would start past
    /// `deadline`; false means the caller's budget is spent.
    pub(crate) async fn wait_until(&mut self, deadline: tokio::time::Instant) -> bool {
        let now = tokio::time::Instant::now();
        if now >= deadline {
            return false;
        }
        let delay = self.advance().min(deadline - now);
        tokio::time::sleep(delay).await;
        true
    }

    /// The pause to wait now, doubling the next one up to the cap.
    fn advance(&mut self) -> Duration {
        let delay = self.delay;
        self.delay = (delay * 2).min(CAPACITY_BACKOFF_MAX);
        delay
    }
}

impl Default for CapacityBackoff {
    fn default() -> Self {
        Self::new()
    }
}

/* === Public Functions === */
/// Whether a read was refused only because the shared read ledger is full.
pub(crate) fn read_capacity(error: &Error) -> bool {
    let full = |error: &harmos::Error| {
        matches!(
            error,
            harmos::Error::ResourceExhausted {
                resource: "storage read memory" | "storage views"
            }
        )
    };
    match error {
        Error::Harmos(error) | Error::Storage(dokime::storage::Error::Runtime(error)) => {
            full(error)
        }
        Error::RequestReceipt(source) => source.downcast_ref::<harmos::Error>().is_some_and(full),
        _ => false,
    }
}

/* === Private Functions === */
fn error(value: impl std::fmt::Display) -> Error {
    Error::Operation(format!("record capture: {value}"))
}

/* === Tests === */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_doubles_to_its_cap() {
        let mut backoff = CapacityBackoff::new();

        let delays = (0..8).map(|_| backoff.advance()).collect::<Vec<_>>();

        assert_eq!(delays[0], CAPACITY_BACKOFF_FIRST);
        assert_eq!(delays[1], CAPACITY_BACKOFF_FIRST * 2);
        assert_eq!(delays[7], CAPACITY_BACKOFF_MAX);
    }

    #[tokio::test(start_paused = true)]
    async fn backoff_stops_at_the_deadline() {
        let mut backoff = CapacityBackoff::new();
        let deadline = tokio::time::Instant::now() + Duration::from_millis(25);

        assert!(backoff.wait_until(deadline).await);
        assert!(backoff.wait_until(deadline).await);
        assert!(!backoff.wait_until(deadline).await);
    }
}
