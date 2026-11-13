/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::atomic::{AtomicI64, Ordering};

// Third Party Imports
use async_trait::async_trait;

// Local Imports
use crate::Result;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct ControlledClock {
    now: AtomicI64,
    changed: tokio::sync::Notify,
}

/* === Enums === */

/* === Traits === */

#[async_trait]
pub trait SchedulerClock: Send + Sync {
    fn now_ns(&self) -> i64;
    async fn wait_until(&self, at_ns: i64) -> Result<()>;
    async fn reached(&self, at_ns: i64);
}

/* === Implementations === */

impl ControlledClock {
    pub fn new(at_ns: i64) -> Self {
        Self {
            now: AtomicI64::new(at_ns),
            changed: tokio::sync::Notify::new(),
        }
    }
}
#[async_trait]
impl SchedulerClock for ControlledClock {
    fn now_ns(&self) -> i64 {
        self.now.load(Ordering::SeqCst)
    }
    async fn wait_until(&self, at_ns: i64) -> Result<()> {
        self.now.fetch_max(at_ns, Ordering::SeqCst);
        self.changed.notify_waiters();
        Ok(())
    }
    async fn reached(&self, at_ns: i64) {
        loop {
            let changed = self.changed.notified();
            if self.now_ns() >= at_ns {
                return;
            }
            changed.await;
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
