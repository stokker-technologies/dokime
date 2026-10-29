/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Canonical Identities And Identifier Observation Patterns

/* === Modules === */

pub mod id;
pub mod identities;
mod pattern;
pub mod wire;

/* === Re-Exports === */

pub use harmos::Identifier;
pub use harmos::Metadata;
pub use id::ActorId;
pub use id::ArtifactId;
pub use id::CaseId;
pub use id::CommandId;
pub use id::ContractId;
pub use id::CorrelationId;
pub use id::CriterionId;
pub use id::DocumentId;
pub use id::HealthItemId;
pub use id::JobRunId;
pub use id::NodeId;
pub use id::NoteId;
pub use id::ParameterId;
pub use id::PromptId;
pub use id::RecordId;
pub use id::SessionId;
pub use id::StepAttemptId;
pub use id::StepId;
pub use id::SuiteId;
pub use id::TelemetryBatchId;
pub use id::TelemetrySampleId;
pub use id::TelemetryStreamId;
pub use id::TemplateId;
pub use id::TestCycleId;
pub use id::TestExecutionId;
pub use id::TraceId;
pub use id::TraceabilityId;
pub use id::UploadId;
pub use identities::DokimeActorIdentity;
pub use identities::DokimeIdentity;
pub use identities::DokimeNodeIdentity;
pub use identities::ExternalIdentityMapping;
pub use identities::ExternalIdentityProvider;
pub use pattern::IdentifierPattern;

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
