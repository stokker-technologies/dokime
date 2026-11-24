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

// Third Party Imports
use prost::Message;
use prost_types::Timestamp;

// Local Imports
use crate::Result;
use dokime_protocol::host::ExecutionOutcome;
use dokime_protocol::host::evidence::LogsAppendRequest;
use dokime_protocol::model::ExecutionProgressValue;
use dokime_protocol::model::Origin;
use dokime_protocol::model::Termination;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Internal Durable Shape. Public Operation Shapes Use the Host and Model Packages.
#[derive(Clone, PartialEq, Message)]
pub struct Execution {
    #[prost(string, tag = "1")]
    pub identifier: String,

    #[prost(string, tag = "2")]
    pub cycle: String,

    #[prost(message, optional, tag = "3")]
    pub origin: Option<Origin>,

    #[prost(bytes = "vec", tag = "4")]
    pub catalog: Vec<u8>,

    #[prost(message, optional, tag = "5")]
    pub outcome: Option<ExecutionOutcome>,

    #[prost(enumeration = "Termination", tag = "6")]
    pub termination: i32,

    #[prost(message, optional, tag = "7")]
    pub sealed_at: Option<Timestamp>,
}

#[derive(Clone, PartialEq, Message)]
pub struct Evidence {
    #[prost(uint64, tag = "1")]
    pub position: u64,

    #[prost(string, tag = "2")]
    pub kind: String,

    #[prost(string, tag = "3")]
    pub step: String,

    #[prost(bytes = "vec", tag = "4")]
    pub payload: Vec<u8>,

    #[prost(int64, tag = "5")]
    pub source_ns: i64,

    #[prost(int64, tag = "6")]
    pub receipt_ns: i64,

    #[prost(string, tag = "7")]
    pub operation: String,

    #[prost(uint32, tag = "8")]
    pub attempt: u32,
}

#[derive(Clone, PartialEq, Message)]
pub struct Sample {
    #[prost(string, tag = "1")]
    pub stream: String,

    #[prost(uint64, tag = "2")]
    pub position: u64,

    #[prost(int64, tag = "3")]
    pub source_ns: i64,

    #[prost(int64, tag = "4")]
    pub receipt_ns: i64,

    #[prost(uint64, tag = "5")]
    pub sequence: u64,

    #[prost(bytes = "vec", tag = "6")]
    pub row: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ExecutionSnapshot {
    pub execution: Execution,
    pub state: ExecutionState,
    pub evidence: Vec<Evidence>,
    pub samples: Vec<Sample>,
    pub seal: Vec<u8>,
}

#[derive(Default)]
pub(crate) struct StepBoundary {
    pub start: Option<u64>,
    pub end: Option<u64>,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExecutionState {
    Open,
    Closed,
    Sealed,
}

/* === Traits === */

/* === Implementations === */

impl ExecutionState {
    /// The State a Stored Execution Row Records.
    pub(crate) fn stored(value: i32) -> Result<Self> {
        match value {
            0 => Ok(Self::Open),
            1 => Ok(Self::Closed),
            2 => Ok(Self::Sealed),
            _ => Err(crate::Error::Incompatible("Invalid Execution State".into())),
        }
    }
}
impl ExecutionSnapshot {
    pub fn records(&self, kind: &str) -> Vec<&Evidence> {
        self.evidence.iter().filter(|entry| entry.kind == format!("record:{kind}")).collect()
    }
    pub fn logs(&self) -> Result<Vec<LogsAppendRequest>> {
        self.evidence
            .iter()
            .filter(|entry| entry.kind == "log")
            .map(|entry| LogsAppendRequest::decode(entry.payload.as_slice()).map_err(Into::into))
            .collect()
    }
    pub fn progress(&self) -> Result<Vec<ExecutionProgressValue>> {
        self.evidence
            .iter()
            .filter(|entry| entry.kind == "progress")
            .map(|entry| ExecutionProgressValue::decode(entry.payload.as_slice()).map_err(Into::into))
            .collect()
    }
    pub fn step_contains(&self, step: &str, position: u64) -> bool {
        if step.is_empty() {
            return true;
        }
        let mut boundary = StepBoundary::default();
        for evidence in &self.evidence {
            boundary.observe(step, evidence);
        }
        boundary.start.is_some_and(|start| start < position && boundary.end.is_none_or(|end| position < end))
    }
}

impl StepBoundary {
    pub fn observe(&mut self, step: &str, evidence: &Evidence) {
        if evidence.step != step {
            return;
        }
        if evidence.kind == "step:start" {
            self.start.get_or_insert(evidence.position);
        }
        if evidence.kind == "step:end" {
            self.end = Some(evidence.position);
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
