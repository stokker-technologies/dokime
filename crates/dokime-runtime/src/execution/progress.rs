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

pub use crate::operations::execution::transport::ExecutionProgress;

/* === Imports === */

// Standard Library Imports
use std::iter::FusedIterator;
use std::sync::Arc;
use std::time::Instant;

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct Progress<I> {
    warmup: u64,
    iterator: I,
    sink: Arc<dyn ProgressSink>,
    phase: String,
    started: Instant,
    completed: u64,
    total: u64,
    pending: bool,
    finished: bool,
}

/* === Enums === */

/* === Traits === */

pub trait ProgressSink: Send + Sync {
    fn update(&self, progress: ExecutionProgress);
}

/* === Implementations === */

impl<I: ExactSizeIterator> Progress<I> {
    pub fn new(phase: impl Into<String>, iterator: I, sink: Arc<dyn ProgressSink>, warmup: u64) -> Self {
        let phase = phase.into();
        let total = u64::try_from(iterator.len()).unwrap_or(u64::MAX);
        let progress =
            Self { iterator, sink, phase, started: Instant::now(), completed: 0, total, pending: false, finished: false, warmup };
        progress.report();

        progress
    }

    fn complete_pending(&mut self) {
        if !self.pending {
            return;
        }
        self.pending = false;
        self.completed = self.completed.saturating_add(1).min(self.total);
        self.report();
    }

    fn report(&self) {
        let elapsed = self.started.elapsed();
        let remaining_ms = if self.completed >= self.total {
            Some(0)
        } else if self.completed >= self.warmup {
            let elapsed_ms = elapsed.as_millis();
            let remaining = u128::from(self.total.saturating_sub(self.completed));
            Some(saturating_u64(elapsed_ms.saturating_mul(remaining) / u128::from(self.completed)))
        } else {
            None
        };
        self.sink.update(ExecutionProgress {
            phase: self.phase.clone(),
            completed: self.completed,
            total: self.total,
            elapsed_ms: saturating_u64(elapsed.as_millis()),
            remaining_ms,
        });
    }
}

impl<I: ExactSizeIterator> Iterator for Progress<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }

        self.complete_pending();
        match self.iterator.next() {
            Some(item) => {
                self.pending = true;
                Some(item)
            }
            None => {
                self.finished = true;
                None
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iterator.size_hint()
    }
}

impl<I: ExactSizeIterator> ExactSizeIterator for Progress<I> {
    fn len(&self) -> usize {
        self.iterator.len()
    }
}

impl<I: ExactSizeIterator> FusedIterator for Progress<I> {}

/* === Public Functions === */

/* === Private Functions === */

fn saturating_u64(value: u128) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

/* === Tests === */
