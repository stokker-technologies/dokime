/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Template Authoring Surface

/* === Modules === */

/* === Re-Exports === */

pub use crate::client::queries::{RecordRow, SampleRow};
pub use crate::context::{Context, RecordsQuery};
pub use crate::prompt::Prompt;
pub use crate::record::Record;
pub use crate::stream::{Observer, Stream};
pub use crate::template::{Criterion, Step, Template};

/* === Imports === */

// Standard Library Imports
use std::collections::HashMap;

// Third Party Imports

// Local Imports
use crate::model::Section;
use crate::{CaptureGap, CapturedArtifact, Card, ReadCut, Statistics, Value};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
pub struct StatisticsReadOptions {
    pub at: Option<ReadCut>,
}

#[derive(Clone, PartialEq)]
pub struct StatisticsSnapshot {
    pub value: Option<Statistics>,
    pub at: Option<ReadCut>,
}

pub struct PromptAnswer {
    pub accepted: bool,
    pub action: String,
    pub values: HashMap<String, Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CaptureResult {
    pub artifacts: Vec<CapturedArtifact>,
    pub gaps: Vec<CaptureGap>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Environment {
    pub backend_pid: u32,
    pub in_step: bool,
}

#[derive(Default)]
pub struct RecordAppendOptions {
    pub request_id: Option<String>,
}

#[derive(Default)]
pub struct ProgressSetOptions {
    pub request_id: Option<String>,
}

#[derive(Default)]
pub struct LogAppendOptions {
    pub request_id: Option<String>,
}

#[derive(Default)]
pub struct TraceabilitySetOptions {
    pub request_id: Option<String>,
    pub label: String,
    pub source: String,
    pub unit: String,
}

#[derive(Default)]
pub struct DocumentAppendOptions {
    pub request_id: Option<String>,
}

/* === Enums === */

pub enum DocumentPart {
    Section(Box<Section>),
    Card(Box<Card>),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
