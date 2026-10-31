/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Finite Query Selection Contracts

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::error::{Failure, Reason};
use crate::model::execution::result::CyclePhase;
use crate::model::execution::wire::{ArchiveScope, ExecutionStatus};
use crate::model::identity::wire::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// All Bounds Are UTC Instants, Inclusive Start And Exclusive End.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Window {
    pub start: Option<Timestamp>,
    pub end: Option<Timestamp>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionSelection {
    #[serde(default)]
    pub template_ids: Vec<String>,

    #[serde(default)]
    pub cycle_ids: Vec<String>,

    #[serde(default)]
    pub statuses: Vec<ExecutionStatus>,
    pub started_at: Option<Window>,

    #[row(description = "Absent selects unarchived rows; alternatives are exact and conjunctive across fields.")]
    pub archive: Option<ArchiveScope>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CycleSelection {
    #[serde(default)]
    pub cycle_ids: Vec<String>,

    #[serde(default)]
    pub phases: Vec<CyclePhase>,
    pub created_at: Option<Window>,
    pub archive: Option<ArchiveScope>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CycleScope {
    pub cycle_id: String,
    pub execution_id: Option<String>,
}

/* === Enums === */

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum RecordScope {
    Execution(String),
    Cycle(CycleScope),
}

/* === Traits === */

/* === Implementations === */

impl Window {
    pub fn validate(&self, operation: &str) -> Result<(), Box<Failure>> {
        let start = self.start.map(|t| t.to_datetime()).transpose();
        let end = self.end.map(|t| t.to_datetime()).transpose();
        if let (Ok(start), Ok(end)) = (start, end)
            && !matches!((start, end), (Some(a), Some(b)) if a >= b)
        {
            return Ok(());
        }
        Err(Failure::new(Reason::InvalidWindow, operation, "Use valid UTC instants with start strictly before end"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_bounds() {
        let time = Timestamp { seconds: 0, nanos: 1 };
        assert!(Window { start: Some(time), end: Some(time) }.validate("test.query").is_err());
        assert!(
            Window {
                start: None,
                end: Some(Timestamp { seconds: 0, nanos: -1 })
            }
            .validate("test.query")
            .is_err()
        );
        assert!(Window { start: None, end: Some(time) }.validate("test.query").is_ok());
    }
}
