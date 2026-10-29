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

pub use crate::operations::execution::transport::{
    ArtifactChunk, EvidenceBatch, EvidenceReceipt, LeaseAsk, LeaseResolution, MAX_ARTIFACT_CHUNK_BYTES, OperatorAnswer, OperatorQuestion,
    SidecarCriterionOutcome, SidecarExecutionOutcome, StepAdmission,
};

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports

/* === Constants === */

pub const RUN_BOUND: Duration = Duration::from_secs(30 * 24 * 60 * 60);
pub const SIGNAL_BOUND: Duration = Duration::from_secs(30);
pub const MAX_EVIDENCE_PAGE_BYTES: usize = 2 * MAX_ARTIFACT_CHUNK_BYTES + 256 * 1024;
pub const MAX_ARTIFACT_BYTES: u64 = 2 * 1024 * 1024 * 1024;

/* === Macros === */

// A template-selected declaration delegates its portable encoding to the inner
// request, so every template shares one wire shape under its own identity.
macro_rules! templated {
    ($name:ident, $inner:ty) => {
        impl<T> $name<T> {
            pub fn new(request: $inner) -> Self {
                Self(request, std::marker::PhantomData)
            }
        }

        impl<T> Clone for $name<T> {
            fn clone(&self) -> Self {
                Self::new(self.0.clone())
            }
        }

        impl<T> harmos::Message for $name<T> {
            const CODEC: &'static str = <$inner as harmos::Message>::CODEC;

            fn encode(&self) -> Vec<u8> {
                harmos::Message::encode(&self.0)
            }

            fn encoded_size(&self) -> usize {
                harmos::Message::encoded_size(&self.0)
            }

            fn decode(bytes: &[u8]) -> Result<Self, harmos::MessageError> {
                <$inner as harmos::Message>::decode(bytes).map(Self::new)
            }

            fn decoding_memory(bytes: &[u8]) -> Result<usize, harmos::MessageError> {
                <$inner as harmos::Message>::decoding_memory(bytes)
            }
        }

        impl<T> harmos::MemorySize for $name<T> {
            fn memory_size(&self) -> usize {
                harmos::MemorySize::memory_size(&self.0)
            }
        }

        impl<T: 'static> harmos::schema::Described for $name<T> {
            const NAME: &'static str = <$inner as harmos::schema::Described>::NAME;
            const FIELDS: &'static [harmos::schema::graph::Field<harmos::schema::declaration::Definition>] =
                <$inner as harmos::schema::Described>::FIELDS;
            const MARKER: &'static [u8] = <$inner as harmos::schema::Described>::MARKER;
            const DEFAULT_VARIANT: Option<usize> = <$inner as harmos::schema::Described>::DEFAULT_VARIANT;
            // The wrapper shares the inner request's schema identity, so one
            // definition name keeps one owner across every template.
            const DEFINITION: harmos::schema::declaration::Definition = <$inner as harmos::schema::Described>::DEFINITION;
        }
    };
}

pub(crate) use templated;

/* === Type Aliases === */

/* === Structs === */

#[harmos::message(skip_debug)]
#[derive(Clone, PartialEq)]
pub struct RunExecution {
    pub execution_token: String,
    pub test_cycle_id: String,
    pub test_execution_id: String,
    pub template_identifier: String,
    pub workflow: String,
    pub parameters_json: String,

    #[row(description = "Host-resolved secrets for this owner. Never persisted as execution data.")]
    pub secrets_json: String,

    #[row(description = "Runner policy: automatic or interactive.")]
    pub mode: String,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct ExecutionSignal {
    pub execution_token: String,

    #[row(description = "Cooperatively cancel the execution; idempotent.")]
    pub cancel: bool,

    #[row(description = "Admit or retry one interactive command.")]
    pub command: Option<CommandExecutionRequest>,

    #[row(description = "Hand down one admitted record a step may be waiting for.")]
    pub record: Option<DeliverRecordRequest>,

    #[row(description = "Hand down rows of a leased global stream.")]
    pub samples: Option<DeliverSamplesRequest>,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct SignalReply {
    #[row(description = "The command's receipt, when the signal carried one.")]
    pub command: Option<CommandExecutionReply>,

    pub samples: Vec<SampleReceipt>,
    pub record_ordinal: u64,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct SampleReceipt {
    pub stream: String,
    pub sample_id: String,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct DeliverRecordRequest {
    pub ordinal: u64,
    #[row(description = "JSON-encoded [`Record`](dokime_domain::model::records::evidence::Record), as the host filed it.")]
    pub record_json: String,

    #[row(description = "Execution scopes and origins frozen at admission, JSON-encoded.")]
    pub captured_scopes: Vec<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct DeliverSamplesRequest {
    // Host Contract: New Rows Arrive In Strictly Increasing Sequence Order Per Stream.
    // Exact Retries Are Accepted Within The Receiver's 64-Row Recent Window.
    // Settlement Must Resume After Its Durable Acknowledged Position; Older Rows Are Protocol Errors.
    #[row(
        description = "JSON-encoded [`TelemetrySample`](crate::telemetry::TelemetrySample) values, scope-stamped by the host with the run whose lease they answer."
    )]
    pub samples: Vec<String>,

    #[row(
        description = "JSON-encoded [`LeaseGap`](crate::telemetry::LeaseGap) values.\nThe absence of rows, on the same road as the rows, because it is the same fact about the same hold: the stream's owner went away and this stretch of the run has no measurements in it. A criterion reads them with `ctx.gaps::<T>()` rather than inferring a gap from a row count it has no expected value for."
    )]
    pub gaps: Vec<String>,

    #[row(
        description = "The scope the host captured these for, JSON-encoded.\nPresent so a sidecar running several executions can tell which of them a delivery is evidence for without reading it off every sample."
    )]
    pub scope_json: String,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct CommandExecutionRequest {
    pub execution_token: String,
    pub command_id: String,
    pub action: String,
    pub step: String,
}

#[harmos::message]
#[derive(Clone, PartialEq)]
pub struct CommandExecutionReply {
    pub state: String,
    pub attempt_id: String,
    pub message: String,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq, ::prost::Enumeration)]
#[repr(i32)]
pub enum SidecarCriterionStatus {
    Unspecified = 0,
    Passed = 1,
    Failed = 2,
    Indeterminate = 3,
}

/* === Traits === */

/* === Implementations === */

impl std::fmt::Debug for RunExecution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RunExecution")
            .field("execution_token", &self.execution_token)
            .field("test_execution_id", &self.test_execution_id)
            .field("template_identifier", &self.template_identifier)
            .field("workflow", &self.workflow)
            .field("mode", &self.mode)
            .field("secrets", &"<redacted>")
            .finish_non_exhaustive()
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
