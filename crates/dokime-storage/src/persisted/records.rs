/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Product History Declarations

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use harmos_domain::Identified;

// Local Imports
use super::bindings::Protobuf;
use super::shapes::State;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct CycleEvent {
    pub execution: String,
    pub kind: String,
    pub receipt_ns: i64,
}

#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct ExecutionEvent {
    pub state: State,
    pub kind: String,
    pub step: String,
    pub operation: String,
    pub attempt: u32,
    pub receipt_ns: i64,
}

#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct EvidenceEvent {
    pub kind: String,
    pub step: String,
    pub operation: String,
    pub attempt: u32,
    pub receipt_ns: i64,
    pub value: Protobuf,
}

/// Plugin Values Use Their Own Frozen Native Graph, Not This Envelope's Schema.
/// The Envelope Defines the Context Composed Into the Declared Record Graph.
#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct Record {
    pub step: String,
    pub operation: String,
    pub attempt: u32,
    pub receipt_ns: i64,
}

#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct HealthEvent {
    pub receipt_ns: i64,
    pub value: Protobuf,
}

#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct PromptEvent {
    pub prompt: String,
    pub receipt_ns: i64,
    pub value: Protobuf,
}

#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct AnswerEvent {
    pub prompt: String,
    pub receipt_ns: i64,
    pub value: Protobuf,
}

#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct DocumentPart {
    pub document: String,
    pub receipt_ns: i64,
    pub value: Protobuf,
}

#[derive(harmos_domain::Message, Clone, Debug, PartialEq)]
pub struct ReportPublished {
    pub report: String,
    pub evidence_cut: Vec<u8>,
    pub receipt_ns: i64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Identified for CycleEvent {
    const ID: &'static str = "dokime.cycle-event";
}
impl Identified for ExecutionEvent {
    const ID: &'static str = "dokime.execution-event";
}
impl Identified for EvidenceEvent {
    const ID: &'static str = "dokime.evidence-event";
}
impl Identified for HealthEvent {
    const ID: &'static str = "dokime.health-event";
}
impl Identified for PromptEvent {
    const ID: &'static str = "dokime.prompt-event";
}
impl Identified for AnswerEvent {
    const ID: &'static str = "dokime.answer-event";
}
impl Identified for DocumentPart {
    const ID: &'static str = "dokime.document-part";
}
impl Identified for ReportPublished {
    const ID: &'static str = "dokime.report-published";
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
