/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use async_trait::async_trait;

// Local Imports
use crate::{Result, SchedulerClock};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct WallClock;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[async_trait]
impl SchedulerClock for WallClock {
    fn now_ns(&self) -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_nanos().min(i64::MAX as u128) as i64)
            .unwrap_or(0)
    }
    async fn wait_until(&self, at_ns: i64) -> Result<()> {
        self.reached(at_ns).await;
        Ok(())
    }
    async fn reached(&self, at_ns: i64) {
        let remaining_ns = at_ns.saturating_sub(self.now_ns());
        if remaining_ns > 0 {
            tokio::time::sleep(std::time::Duration::from_nanos(remaining_ns as u64)).await;
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
