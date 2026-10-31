/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Finite Query Page Contracts

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::{reduction::Bucket, schema::Schema, snapshot::Snapshot};
use crate::model::{
    evidence::wire::CapturedRecord,
    execution::{
        result::CyclePhase,
        wire::{ExecutionStatus, ResultSummary},
    },
    history::wire::ExecutionAvailability,
    identity::wire::Timestamp,
    telemetry::wire::TelemetrySample,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageRequest {
    pub rows: Option<u32>,
    pub bytes: Option<u64>,
    pub after: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct PageLimits {
    pub rows: u32,
    pub bytes: u64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionSummary {
    pub execution_id: String,
    pub cycle_id: Option<String>,
    pub case_id: String,
    pub template_id: Option<String>,
    pub label: Option<String>,
    pub status: ExecutionStatus,
    pub started_at: Option<Timestamp>,
    pub finished_at: Option<Timestamp>,
    pub result: Option<ResultSummary>,
    pub availability: ExecutionAvailability,
    pub checkpoint_available: bool,
    pub archived: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CycleSummary {
    pub cycle_id: String,
    pub label: Option<String>,
    pub phase: CyclePhase,
    pub created_at: Option<Timestamp>,
    pub updated_at: Option<Timestamp>,
    pub execution_count: u64,
    pub archived: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionPage {
    pub rows: Vec<ExecutionSummary>,
    pub schema: Option<Schema>,
    pub snapshot: Option<Snapshot>,
    pub limits: Option<PageLimits>,
    pub next: Option<String>,
    pub complete: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CyclePage {
    pub rows: Vec<CycleSummary>,
    pub schema: Option<Schema>,
    pub snapshot: Option<Snapshot>,
    pub limits: Option<PageLimits>,
    pub next: Option<String>,
    pub complete: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordPage {
    pub rows: Vec<CapturedRecord>,
    pub schema: Option<Schema>,
    pub snapshot: Option<Snapshot>,
    pub limits: Option<PageLimits>,
    pub next: Option<String>,
    pub complete: bool,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct SamplePage {
    pub rows: Vec<SampleRow>,
    pub schema: Option<Schema>,
    pub snapshot: Option<Snapshot>,
    pub limits: Option<PageLimits>,
    pub next: Option<String>,
    pub complete: bool,
}

/* === Enums === */

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum SampleRow {
    Raw(Box<TelemetrySample>),
    Bucket(Bucket),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
