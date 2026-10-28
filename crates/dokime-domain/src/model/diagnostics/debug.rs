/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::Deserialize;
use serde::Serialize;

use crate::model::identity::ArtifactId;
use crate::model::identity::CorrelationId;
use crate::model::identity::TraceId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorCause {
    pub message: String,
    pub code: Option<String>,
    pub source: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructuredErrorDetail {
    pub message: String,
    pub code: Option<String>,
    pub source: Option<String>,
    pub cause_chain: Vec<ErrorCause>,
    pub stacktrace_attachment: Option<ArtifactId>,
    pub trace_id: Option<TraceId>,
    pub correlation_id: Option<CorrelationId>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DebugLevel {
    Debug,

    #[default]
    Info,

    Warning,
    Error,
}

impl harmos_domain::MemorySize for ErrorCause {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.message), std::mem::size_of_val(&self.message)),
            (harmos_domain::MemorySize::memory_size(&self.code), std::mem::size_of_val(&self.code)),
            (harmos_domain::MemorySize::memory_size(&self.source), std::mem::size_of_val(&self.source)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl harmos_domain::MemorySize for StructuredErrorDetail {
    fn memory_size(&self) -> usize {
        let fields = [
            (harmos_domain::MemorySize::memory_size(&self.message), std::mem::size_of_val(&self.message)),
            (harmos_domain::MemorySize::memory_size(&self.code), std::mem::size_of_val(&self.code)),
            (harmos_domain::MemorySize::memory_size(&self.source), std::mem::size_of_val(&self.source)),
            (harmos_domain::MemorySize::memory_size(&self.cause_chain), std::mem::size_of_val(&self.cause_chain)),
            (harmos_domain::MemorySize::memory_size(&self.stacktrace_attachment), std::mem::size_of_val(&self.stacktrace_attachment)),
            (harmos_domain::MemorySize::memory_size(&self.trace_id), std::mem::size_of_val(&self.trace_id)),
            (harmos_domain::MemorySize::memory_size(&self.correlation_id), std::mem::size_of_val(&self.correlation_id)),
        ];
        fields.into_iter().fold(std::mem::size_of::<Self>(), |bytes, (owned, inline)| {
            if owned == usize::MAX { usize::MAX } else { bytes.saturating_add(owned.saturating_sub(inline)) }
        })
    }
}

impl ErrorCause {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: None,
            source: None,
        }
    }
}

impl StructuredErrorDetail {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: None,
            source: None,
            cause_chain: Vec::new(),
            stacktrace_attachment: None,
            trace_id: None,
            correlation_id: None,
        }
    }
}
