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

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

const UNSCOPED_TRACK_LABEL: &str = "-";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct CycleTrack(Option<String>);

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub struct ExecutionTrack(Option<String>);

#[harmos::stream(
    id = "telemetry",
    window = 16384,
    track = (cycle, execution),
    axis = ordinal
, description = "One live telemetry row: hierarchical scope, coordinates, and an encoded sample. Cycle and execution tracks narrow queries over the bounded in-memory window. Execution-owned historical evidence is stored separately in native storage.")]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TelemetryRow {
    pub cycle: CycleTrack,
    pub execution: ExecutionTrack,
    pub ordinal: u64,
    pub timestamp_millis: u64,
    pub stream: String,
    pub payload: Vec<u8>,
    pub storage_payload: Option<Vec<u8>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl harmos::streams::Track for CycleTrack {
    fn label(&self) -> String {
        self.0.as_ref().map_or_else(|| UNSCOPED_TRACK_LABEL.to_owned(), |cycle| format!("cycle_{cycle}"))
    }
}

impl harmos::streams::Track for ExecutionTrack {
    fn label(&self) -> String {
        self.0.as_ref().map_or_else(|| UNSCOPED_TRACK_LABEL.to_owned(), |execution| format!("execution_{execution}"))
    }
}

impl CycleTrack {
    pub fn from_id(cycle: Option<TestCycleId>) -> Self {
        Self(cycle.map(|cycle| cycle.to_string()))
    }

    pub fn as_str(&self) -> Option<&str> {
        self.0.as_deref()
    }
}

impl ExecutionTrack {
    pub fn from_id(execution: Option<TestExecutionId>) -> Self {
        Self(execution.map(|execution| execution.to_string()))
    }

    pub fn as_str(&self) -> Option<&str> {
        self.0.as_deref()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
