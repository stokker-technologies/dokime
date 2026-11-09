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

pub use crate::operations::execution::transport::{Entry, Level, timestamp};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

#[macro_export]
macro_rules! trace {
    ($ctx:expr, $($arg:tt)+) => {
        $ctx.trace(::std::format!($($arg)+))
    };
}

#[macro_export]
macro_rules! debug {
    ($ctx:expr, $($arg:tt)+) => {
        $ctx.debug(::std::format!($($arg)+))
    };
}

#[macro_export]
macro_rules! info {
    ($ctx:expr, $($arg:tt)+) => {
        $ctx.info(::std::format!($($arg)+))
    };
}

#[macro_export]
macro_rules! warn {
    ($ctx:expr, $($arg:tt)+) => {
        $ctx.warn(::std::format!($($arg)+))
    };
}

#[macro_export]
macro_rules! error {
    ($ctx:expr, $($arg:tt)+) => {
        $ctx.error(::std::format!($($arg)+))
    };
}

/* === Type Aliases === */

/* === Structs === */

#[cfg(feature = "native")]
pub(crate) struct AttemptTiming {
    pub started_at: harmos::state::Timestamp,
    pub elapsed: std::time::Duration,
    pub attempt_timeout: std::time::Duration,
}

#[cfg(feature = "native")]
#[derive(Clone, serde::Deserialize)]
struct AttemptRecord {
    kind: String,
    operation: String,
    instance: u64,
    stage: String,
    outcome: harmos::operations::instance::Outcome,
    elapsed_ms: f64,
    attempt_timeout_ms: f64,
}

#[cfg(feature = "native")]
#[derive(Default)]
pub(crate) struct Latencies {
    stages: std::collections::BTreeMap<String, std::collections::BTreeMap<String, Vec<AttemptRecord>>>,
    execution: std::collections::BTreeMap<String, Vec<AttemptRecord>>,
}

#[cfg(feature = "native")]
#[derive(Default)]
pub(crate) struct Cadence {
    interval: std::time::Duration,
    previous: Option<std::time::Duration>,
    interval_sum: std::time::Duration,
    intervals: u64,
    maximum_gap: std::time::Duration,
    ticks: u64,
    skipped: u64,
    refused: u64,
    longest_callback: std::time::Duration,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[cfg(feature = "native")]
impl Latencies {
    pub(crate) fn observe(&mut self, entry: &Entry) {
        let Ok(record) = serde_json::from_str::<AttemptRecord>(&entry.message) else { return };
        if record.kind != "operation_attempt" || !record.elapsed_ms.is_finite() || record.elapsed_ms < 0.0 {
            return;
        }
        let stage = record.stage.clone();
        let operation = record.operation.clone();
        // The Execution Fold Retains Attempts After Their Step Fold Is Drained.
        let execution_record = record.clone();
        self.execution.entry(operation.clone()).or_default().push(execution_record);
        self.stages.entry(stage).or_default().entry(operation).or_default().push(record);
    }

    pub(crate) fn finish_stage(&mut self, stage: &str) -> Vec<Entry> {
        self.stages
            .remove(stage)
            .unwrap_or_default()
            .into_iter()
            .map(|(operation, records)| latency_record(&operation, stage, records))
            .collect()
    }

    pub(crate) fn finish(&mut self) -> Vec<Entry> {
        let mut entries = Vec::new();
        for (stage, operations) in std::mem::take(&mut self.stages) {
            for (operation, records) in operations {
                entries.push(latency_record(&operation, &stage, records));
            }
        }
        for (operation, records) in std::mem::take(&mut self.execution) {
            entries.push(latency_record(&operation, "execution", records));
        }
        entries
    }
}

#[cfg(feature = "native")]
impl Cadence {
    pub(crate) fn observe(&mut self, source: &str, report: &harmos::__private::sidecar::CadenceReport) -> Vec<Entry> {
        self.interval = report.interval;
        self.ticks += 1;
        self.skipped += u64::from(report.skipped);
        self.refused += u64::from(report.refused);
        self.longest_callback = self.longest_callback.max(report.callback_duration);
        let mut entries = Vec::new();
        if let Some(started) = report.started_at {
            if let Some(previous) = self.previous {
                let gap = started.saturating_sub(previous);
                self.interval_sum += gap;
                self.intervals += 1;
                self.maximum_gap = self.maximum_gap.max(gap);
                if gap > report.interval.saturating_mul(2) {
                    entries.push(Entry::new(
                        Level::Warn,
                        serde_json::json!({
                            "kind": "dokime.observers.gap", "plugin": source, "observer": report.observer,
                            "activation": report.activation, "gap_ms": gap.as_secs_f64() * 1000.0,
                            "configured_interval_ms": report.interval.as_secs_f64() * 1000.0,
                        })
                        .to_string(),
                    ));
                }
            }
            self.previous = Some(started);
        }
        if self.ticks.is_multiple_of(60) {
            entries.push(self.record(source, &report.observer, &report.activation, "periodic"));
        }
        entries
    }

    pub(crate) fn record(&self, source: &str, observer: &str, activation: &str, reason: &str) -> Entry {
        let mean = (self.intervals > 0).then(|| self.interval_sum.as_secs_f64() * 1000.0 / self.intervals as f64);
        let level = if self.maximum_gap > self.interval.saturating_mul(2) { Level::Warn } else { Level::Debug };
        Entry::new(
            level,
            serde_json::json!({
                "kind": "dokime.observers.cadence", "plugin": source, "observer": observer, "activation": activation,
                "reason": reason, "configured_interval_ms": self.interval.as_secs_f64() * 1000.0,
                "mean_actual_interval_ms": mean, "maximum_gap_ms": self.maximum_gap.as_secs_f64() * 1000.0,
                "ticks": self.ticks, "skipped_ticks": self.skipped, "refused_ticks": self.refused,
                "longest_callback_ms": self.longest_callback.as_secs_f64() * 1000.0,
            })
            .to_string(),
        )
    }
}

/* === Public Functions === */

#[cfg(feature = "native")]
pub(crate) fn operation_attempt(
    operation: &str, instance: u64, generation: Option<u64>, stage: &str, attempt: usize, outcome: harmos::operations::instance::Outcome,
    timing: AttemptTiming,
) -> Entry {
    use harmos::operations::instance::Outcome;

    let level = match outcome {
        Outcome::Succeeded if attempt == 0 => Level::Trace,
        Outcome::Succeeded => Level::Info,
        Outcome::Cancelled => Level::Debug,
        Outcome::Failed | Outcome::TimedOut | Outcome::Unavailable => Level::Warn,
    };
    Entry::new(
        level,
        serde_json::json!({
            "kind": "operation_attempt", "operation": operation, "instance": instance,
            "generation": generation, "stage": stage, "attempt": attempt, "outcome": outcome,
            "started_at": timing.started_at, "elapsed_ms": timing.elapsed.as_secs_f64() * 1000.0,
            "attempt_timeout_ms": timing.attempt_timeout.as_secs_f64() * 1000.0,
        })
        .to_string(),
    )
}

/* === Private Functions === */

#[cfg(feature = "native")]
fn latency_record(operation: &str, stage: &str, mut records: Vec<AttemptRecord>) -> Entry {
    records.sort_by(|left, right| left.elapsed_ms.total_cmp(&right.elapsed_ms).then(left.instance.cmp(&right.instance)));
    let attempts = records.len();
    let successes = records.iter().filter(|record| record.outcome == harmos::operations::instance::Outcome::Succeeded).count();
    let p50 = records[(attempts * 50).div_ceil(100) - 1].elapsed_ms;
    let p95 = records[(attempts * 95).div_ceil(100) - 1].elapsed_ms;
    let slowest = records.last().expect("a latency fold contains attempts");
    let level = if records.iter().any(|record| p95 > record.attempt_timeout_ms / 2.0) { Level::Info } else { Level::Debug };
    Entry::new(
        level,
        serde_json::json!({
            "kind": "dokime.operations.latency", "operation": operation, "stage": stage,
            "attempts": attempts, "successes": successes, "failures": attempts - successes,
            "p50_elapsed_ms": p50, "p95_elapsed_ms": p95, "max_elapsed_ms": slowest.elapsed_ms,
            "slowest_instance": slowest.instance,
        })
        .to_string(),
    )
}

/* === Tests === */

#[cfg(all(test, feature = "native"))]
mod tests {
    use super::*;
    use harmos::operations::instance::Outcome;

    #[test]
    fn operation_attempt_level_follows_outcome_and_retry() {
        for (outcome, attempt, level) in [
            (Outcome::Succeeded, 0, Level::Trace),
            (Outcome::Succeeded, 1, Level::Info),
            (Outcome::Succeeded, 2, Level::Info),
            (Outcome::Cancelled, 0, Level::Debug),
            (Outcome::Cancelled, 1, Level::Debug),
            (Outcome::Failed, 0, Level::Warn),
            (Outcome::Failed, 1, Level::Warn),
            (Outcome::TimedOut, 0, Level::Warn),
            (Outcome::TimedOut, 1, Level::Warn),
            (Outcome::Unavailable, 0, Level::Warn),
            (Outcome::Unavailable, 1, Level::Warn),
        ] {
            let entry = operation_attempt(
                "example.op",
                1,
                Some(2),
                "step/read",
                attempt,
                outcome,
                AttemptTiming {
                    started_at: harmos::state::Timestamp::UNIX_EPOCH,
                    elapsed: std::time::Duration::from_millis(7),
                    attempt_timeout: std::time::Duration::from_secs(1),
                },
            );
            assert_eq!(Level::try_from(entry.level).unwrap(), level, "{outcome:?}, attempt {attempt}");
        }
    }
    #[test]
    fn latency_folds_steps_and_execution_with_nearest_rank_percentiles() {
        let mut latencies = Latencies::default();
        for (instance, elapsed_ms, outcome, stage) in [
            (1, 10, Outcome::Succeeded, "step/read"),
            (2, 50, Outcome::Failed, "step/read"),
            (3, 90, Outcome::TimedOut, "step/read"),
            (4, 20, Outcome::Cancelled, "step/write"),
        ] {
            let entry = operation_attempt(
                "example.op",
                instance,
                None,
                stage,
                0,
                outcome,
                AttemptTiming {
                    started_at: harmos::state::Timestamp::UNIX_EPOCH,
                    elapsed: std::time::Duration::from_millis(elapsed_ms),
                    attempt_timeout: std::time::Duration::from_millis(100),
                },
            );
            let record: serde_json::Value = serde_json::from_str(&entry.message).unwrap();
            assert_eq!(record["started_at"], serde_json::to_value(harmos::state::Timestamp::UNIX_EPOCH).unwrap());
            assert_eq!(record["elapsed_ms"], elapsed_ms as f64);
            latencies.observe(&entry);
        }
        let read = latencies.finish_stage("step/read");
        assert_eq!(read.len(), 1);
        assert_eq!(Level::try_from(read[0].level).unwrap(), Level::Info);
        let read: serde_json::Value = serde_json::from_str(&read[0].message).unwrap();
        assert_eq!(read["attempts"], 3);
        assert_eq!(read["successes"], 1);
        assert_eq!(read["failures"], 2);
        assert_eq!(read["p50_elapsed_ms"], 50.0);
        assert_eq!(read["p95_elapsed_ms"], 90.0);
        assert_eq!(read["max_elapsed_ms"], 90.0);
        assert_eq!(read["slowest_instance"], 3);
        assert!(latencies.finish_stage("step/read").is_empty());
        let remaining = latencies.finish();
        assert_eq!(remaining.len(), 2);
        assert_eq!(Level::try_from(remaining[0].level).unwrap(), Level::Debug);
        let execution: serde_json::Value = serde_json::from_str(&remaining[1].message).unwrap();
        assert_eq!(execution["stage"], "execution");
        assert_eq!(execution["attempts"], 4);
        assert_eq!(execution["p50_elapsed_ms"], 20.0);
        assert!(latencies.finish().is_empty());
    }

    #[test]
    fn latency_threshold_is_strict_and_operations_are_independent() {
        let mut latencies = Latencies::default();
        for (operation, elapsed_ms) in [("a", 50), ("b", 51)] {
            latencies.observe(&operation_attempt(
                operation,
                1,
                None,
                "step/read",
                0,
                Outcome::Unavailable,
                AttemptTiming {
                    started_at: harmos::state::Timestamp::UNIX_EPOCH,
                    elapsed: std::time::Duration::from_millis(elapsed_ms),
                    attempt_timeout: std::time::Duration::from_millis(100),
                },
            ));
        }
        let entries = latencies.finish_stage("step/read");
        assert_eq!(entries.len(), 2);
        assert_eq!(Level::try_from(entries[0].level).unwrap(), Level::Debug);
        assert_eq!(Level::try_from(entries[1].level).unwrap(), Level::Info);
    }

    #[test]
    fn cadence_keeps_skipped_ticks_out_of_actual_intervals_and_flushes_at_sixty() {
        use std::time::Duration;
        let mut cadence = Cadence::default();
        for ordinal in 0..60 {
            let report = harmos::__private::sidecar::CadenceReport {
                observer: "meter.read".into(),
                activation: "activation-1".into(),
                interval: Duration::from_secs(1),
                scheduled_at: Duration::from_secs(ordinal),
                started_at: (ordinal != 1 && ordinal != 2).then(|| Duration::from_secs(ordinal)),
                callback_duration: Duration::from_millis(ordinal),
                skipped: ordinal == 1 || ordinal == 2,
                refused: ordinal == 4,
            };
            let entries = cadence.observe("meter", &report);
            match ordinal {
                3 => {
                    assert_eq!(entries.len(), 1);
                    assert_eq!(Level::try_from(entries[0].level).unwrap(), Level::Warn);
                    let gap: serde_json::Value = serde_json::from_str(&entries[0].message).unwrap();
                    assert_eq!(gap["observer"], "meter.read");
                    assert_eq!(gap["gap_ms"], 3000.0);
                }
                59 => assert_eq!(entries.len(), 1),
                _ => assert!(entries.is_empty()),
            }
        }
        let entry = cadence.record("meter", "meter.read", "activation-1", "execution_end");
        let record: serde_json::Value = serde_json::from_str(&entry.message).unwrap();
        assert_eq!(record["ticks"], 60);
        assert_eq!(record["skipped_ticks"], 2);
        assert_eq!(record["refused_ticks"], 1);
        assert_eq!(record["longest_callback_ms"], 59.0);
        assert_eq!(record["maximum_gap_ms"], 3000.0);
        assert_eq!(record["mean_actual_interval_ms"], 59000.0 / 57.0);
        let empty = Cadence::default().record("meter", "meter.read", "activation-2", "execution_end");
        let record: serde_json::Value = serde_json::from_str(&empty.message).unwrap();
        assert_eq!(record["mean_actual_interval_ms"], serde_json::Value::Null);
        assert_eq!(record["ticks"], 0);
    }

    #[test]
    fn cadence_gap_threshold_is_strict() {
        use std::time::Duration;
        let mut cadence = Cadence::default();
        for elapsed in [0, 2, 5] {
            let report = harmos::__private::sidecar::CadenceReport {
                observer: "meter.read".into(),
                activation: "activation-1".into(),
                interval: Duration::from_secs(1),
                scheduled_at: Duration::from_secs(elapsed),
                started_at: Some(Duration::from_secs(elapsed)),
                callback_duration: Duration::ZERO,
                skipped: false,
                refused: false,
            };
            assert_eq!(cadence.observe("meter", &report).len(), usize::from(elapsed == 5));
        }
    }
}
