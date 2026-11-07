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

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

#[cfg(feature = "native")]
pub(crate) fn operation_attempt(
    operation: &str, instance: u64, generation: Option<u64>, stage: &str, attempt: usize, outcome: harmos::operations::instance::Outcome,
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
        })
        .to_string(),
    )
}

/* === Private Functions === */

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
            let entry = operation_attempt("example.op", 1, Some(2), "step/read", attempt, outcome);
            assert_eq!(Level::try_from(entry.level).unwrap(), level, "{outcome:?}, attempt {attempt}");
        }
    }
}
