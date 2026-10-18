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
use std::time::Duration;
// Third Party Imports
// Third-Party Imports
#[cfg(feature = "native")]
use tokio::sync::watch;

#[cfg(feature = "native")]
use tokio::time::{Instant, MissedTickBehavior, interval_at};
// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tick {
    index: u64,
    elapsed: Duration,
}

#[cfg(feature = "native")]
pub(crate) struct Cadence {
    started: Instant,
    timer: tokio::time::Interval,
    index: u64,
    stop: watch::Receiver<bool>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Tick {
    pub fn new(index: u64, elapsed: Duration) -> Self {
        Self { index, elapsed }
    }

    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    pub fn index(&self) -> u64 {
        self.index
    }
}

#[cfg(feature = "native")]
impl Cadence {
    pub(crate) fn every(interval: Duration, stop: watch::Receiver<bool>) -> Self {
        let started = Instant::now();
        let mut timer = interval_at(started + interval, interval);
        // A stream that fell behind is late, not owed a burst: catching up would
        // publish several rows carrying almost the same instant and would charge
        // the device for time nobody asked it about.
        timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
        Self { started, timer, index: 0, stop }
    }

    pub(crate) async fn next(&mut self) -> Option<Tick> {
        tokio::select! {
            biased;
            _ = self.stop.wait_for(|stopped| *stopped) => None,
            _ = self.timer.tick() => {
                let tick = Tick::new(self.index, self.started.elapsed());
                self.index += 1;

                Some(tick)
            }
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
