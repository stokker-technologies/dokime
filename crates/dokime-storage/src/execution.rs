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

// Local Imports
use crate::Result;
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Internal Durable Shape. Public Operation Shapes Remain dokime.v1 Messages.
#[derive(Clone, PartialEq, Message)]
pub struct Execution {
    #[prost(string, tag = "1")]
    pub identifier: String,

    #[prost(string, tag = "2")]
    pub cycle: String,

    #[prost(message, optional, tag = "3")]
    pub origin: Option<v1::Origin>,

    #[prost(bytes = "vec", tag = "4")]
    pub catalog: Vec<u8>,

    #[prost(message, optional, tag = "5")]
    pub outcome: Option<v1::ExecutionOutcome>,

    #[prost(enumeration = "v1::Termination", tag = "6")]
    pub termination: i32,
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

#[derive(Clone, PartialEq, Message)]
pub struct ExecutionArchive {
    #[prost(message, optional, tag = "1")]
    pub execution: Option<Execution>,

    #[prost(message, repeated, tag = "2")]
    pub evidence: Vec<Evidence>,

    #[prost(message, repeated, tag = "3")]
    pub samples: Vec<Sample>,

    #[prost(bytes = "vec", tag = "4")]
    pub seal: Vec<u8>,

    #[prost(bytes = "vec", repeated, tag = "5")]
    pub reports: Vec<Vec<u8>>,

    #[prost(message, repeated, tag = "6")]
    pub sample_batches: Vec<SampleBatch>,

    #[prost(message, repeated, tag = "7")]
    pub content: Vec<crate::ArchivedContent>,

    #[prost(message, repeated, tag = "8")]
    pub content_chunks: Vec<crate::ArchivedContentChunk>,
}

#[derive(Clone, PartialEq, Message)]
pub struct SampleBatch {
    #[prost(uint64, tag = "1")]
    pub position: u64,

    #[prost(string, tag = "2")]
    pub stream: String,

    #[prost(bytes = "vec", tag = "3")]
    pub payload: Vec<u8>,
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

impl ExecutionSnapshot {
    pub fn records(&self, kind: &str) -> Vec<&Evidence> {
        self.evidence.iter().filter(|entry| entry.kind == format!("record:{kind}")).collect()
    }
    pub fn logs(&self) -> Result<Vec<dokime_protocol::context::evidence::LogsAppendRequest>> {
        self.evidence
            .iter()
            .filter(|entry| entry.kind == "log")
            .map(|entry| dokime_protocol::context::evidence::LogsAppendRequest::decode(entry.payload.as_slice()).map_err(Into::into))
            .collect()
    }
    pub fn progress(&self) -> Result<Vec<v1::ReportProgress>> {
        self.evidence
            .iter()
            .filter(|entry| entry.kind == "progress")
            .map(|entry| v1::ReportProgress::decode(entry.payload.as_slice()).map_err(Into::into))
            .collect()
    }
    pub fn step_contains(&self, step: &str, position: u64) -> bool {
        if step.is_empty() {
            return true;
        }
        let start = self.evidence.iter().find(|entry| entry.kind == "step:start" && entry.step == step);
        let end = self.evidence.iter().rev().find(|entry| entry.kind == "step:end" && entry.step == step);
        start.is_some_and(|start| start.position < position && end.is_none_or(|end| position < end.position))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
