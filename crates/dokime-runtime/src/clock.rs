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
use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

// Third Party Imports
use async_trait::async_trait;
use harmos_runtime::clock::{Clock, Controller, Reading, Sleep, Source};
use tokio::sync::watch;
use tokio::time::Instant;

// Local Imports
use crate::Result;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct ControlledClock {
    now: watch::Sender<i64>,
    manual: bool,
}

pub(crate) struct SchedulerSource {
    source: Arc<dyn SchedulerClock>,
    origin: Instant,
    origin_ns: i64,
}

/* === Enums === */

/* === Traits === */

#[async_trait]
pub trait SchedulerClock: Send + Sync {
    fn now_ns(&self) -> i64;
    async fn wait_until(&self, at_ns: i64) -> Result<()>;
    async fn reached(&self, at_ns: i64);
    fn execution_clock(&self) -> (Clock, Option<Controller>) {
        (Clock::real_at(date(self.now_ns())), None)
    }
}

/* === Implementations === */

impl ControlledClock {
    pub fn new(at_ns: i64) -> Self {
        Self {
            now: watch::channel(at_ns).0,
            manual: false,
        }
    }
    pub fn manual(at_ns: i64) -> Self {
        Self {
            now: watch::channel(at_ns).0,
            manual: true,
        }
    }
}
#[async_trait]
impl SchedulerClock for ControlledClock {
    fn execution_clock(&self) -> (Clock, Option<Controller>) {
        if self.manual {
            return (Clock::with_clock(SchedulerSource::new(Arc::new(self.clone()))), None);
        }
        let (clock, controller) = Clock::virtual_at(date(self.now_ns()));
        (clock, Some(controller))
    }
    fn now_ns(&self) -> i64 {
        *self.now.borrow()
    }
    async fn wait_until(&self, at_ns: i64) -> Result<()> {
        self.now.send_if_modified(|now| {
            if *now >= at_ns {
                return false;
            }
            *now = at_ns;
            true
        });
        Ok(())
    }
    async fn reached(&self, at_ns: i64) {
        let mut now = self.now.subscribe();
        loop {
            if *now.borrow_and_update() >= at_ns {
                return;
            }
            if now.changed().await.is_err() {
                return;
            }
        }
    }
}

impl SchedulerSource {
    pub(crate) fn new(source: Arc<dyn SchedulerClock>) -> Self {
        Self {
            origin_ns: source.now_ns(),
            source,
            origin: Instant::now(),
        }
    }
}

impl Source for SchedulerSource {
    fn read(&self) -> Reading {
        let now_ns = self.source.now_ns();
        Reading {
            instant: self.origin + Duration::from_nanos(now_ns.saturating_sub(self.origin_ns).max(0) as u64),
            date: date(now_ns),
        }
    }
    fn sleep_until(&self, deadline: Instant) -> Sleep {
        let source = self.source.clone();
        let deadline_ns =
            self.origin_ns.saturating_add(deadline.saturating_duration_since(self.origin).as_nanos().min(i64::MAX as u128) as i64);
        Box::pin(async move { source.reached(deadline_ns).await })
    }
}

/* === Public Functions === */

pub(crate) fn date(at_ns: i64) -> SystemTime {
    if at_ns >= 0 {
        SystemTime::UNIX_EPOCH + Duration::from_nanos(at_ns as u64)
    } else {
        SystemTime::UNIX_EPOCH - Duration::from_nanos(at_ns.unsigned_abs())
    }
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{ControlledClock, SchedulerClock, SchedulerSource};
    use harmos_runtime::clock::Source;
    use std::future::{Future, poll_fn};
    use std::sync::Arc;
    use std::task::Poll;
    use std::time::{Duration, SystemTime};

    #[tokio::test]
    async fn scheduler_source_converts_dates_and_waits_from_its_origin() {
        let scheduler = Arc::new(ControlledClock::manual(-10));
        let source = SchedulerSource::new(scheduler.clone());
        let start = source.read();
        assert_eq!(start.date, SystemTime::UNIX_EPOCH - Duration::from_nanos(10));
        let mut waiting = source.sleep_until(start.instant + Duration::from_nanos(20));
        assert!(poll_fn(|context| Poll::Ready(waiting.as_mut().poll(context).is_pending())).await);
        scheduler.wait_until(9).await.unwrap();
        assert!(poll_fn(|context| Poll::Ready(waiting.as_mut().poll(context).is_pending())).await);
        scheduler.wait_until(10).await.unwrap();
        waiting.await;
        let end = source.read();
        assert_eq!(end.date, SystemTime::UNIX_EPOCH + Duration::from_nanos(10));
        assert_eq!(end.instant - start.instant, Duration::from_nanos(20));
    }

    #[tokio::test]
    async fn controlled_clock_waits_for_ticks_and_never_moves_backwards() {
        let clock = ControlledClock::new(10);
        let mut waiting = Box::pin(clock.reached(20));
        let pending = poll_fn(|context| Poll::Ready(waiting.as_mut().poll(context).is_pending())).await;
        assert!(pending);
        clock.wait_until(19).await.unwrap();
        let pending = poll_fn(|context| Poll::Ready(waiting.as_mut().poll(context).is_pending())).await;
        assert!(pending);
        clock.wait_until(20).await.unwrap();
        waiting.await;
        clock.wait_until(5).await.unwrap();
        assert_eq!(clock.now_ns(), 20);
    }

    #[tokio::test]
    async fn a_tick_releases_all_waiters_and_late_subscribers() {
        let clock = ControlledClock::new(0);
        let mut first = Box::pin(clock.reached(10));
        let mut second = Box::pin(clock.reached(10));
        let pending = poll_fn(|context| {
            let first = first.as_mut().poll(context).is_pending();
            let second = second.as_mut().poll(context).is_pending();
            Poll::Ready(first && second)
        })
        .await;
        assert!(pending);
        clock.wait_until(10).await.unwrap();
        tokio::join!(first, second);
        clock.reached(10).await;
    }
}
