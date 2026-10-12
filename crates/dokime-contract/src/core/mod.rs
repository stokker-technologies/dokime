/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod card;
pub mod command;
pub mod credentials;
pub mod debug;
pub mod error;
pub mod format;
pub mod id;
pub mod identity;
pub mod scope;
pub mod time;
pub mod unit;
pub mod validation;
pub mod value;

/* === Re-Exports === */
pub use card::{HostCardBlock, HostCardBody, HostCardKeyValue, HostCardMetric, HostCardTone};
pub use command::CommandContext;
pub use credentials::{CredentialBackendKind, RedactedCredentialBinding};
pub use debug::{DebugLevel, ErrorCause, StructuredErrorDetail};
pub use error::ValidationIssue;
pub use format::{ElapsedUnit, TIME_PATTERN_TOKENS, ValueFormat, validate_value_format};
pub use harmos::{Identifier, Metadata};
pub use id::{
    ActorId, ArtifactId, CaseId, CommandId, ContractId, CorrelationId, CriterionId, DocumentId,
    HealthItemId, JobRunId, NodeId, NoteId, ParameterId, PromptId, RecordId, SessionId,
    StepAttemptId, StepId, SuiteId, TelemetryBatchId, TelemetrySampleId, TelemetryStreamId,
    TemplateId, TestCycleId, TestExecutionId, TraceId, TraceabilityId, UploadId,
};
pub use identity::{
    DokimeActorIdentity, DokimeIdentity, DokimeNodeIdentity, ExternalIdentityMapping,
    ExternalIdentityProvider,
};
pub use scope::{CapturePhase, EvidenceOrigin, RecordScope, scope_visible_in};
pub use time::{DurationMillis, Timestamp, datetime_text};
pub use unit::{MeasurementUnit, UnitFamily};
pub use validation::{CompatibilityStatus, ConditionEvaluation, ValidationMode};
pub use value::{
    Value, ValueType, datetime_seconds, numeric_cmp, numeric_value, value_matches_type,
};

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
