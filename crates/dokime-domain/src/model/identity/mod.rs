/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Canonical Identities and Identifier Observation Patterns

/* === Modules === */

pub mod distribution;
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

pub(crate) fn identifier_from_label(value: &str) -> Identifier {
    let normalized = normalized_identifier(value);
    let identifier = if normalized.is_empty() { "document.item" } else { &normalized };
    Identifier::new(identifier).expect("normalized document identifier is valid")
}

pub(crate) fn normalized_identifier(value: &str) -> String {
    let mut normalized = String::with_capacity(value.len());
    let mut last_was_separator = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            normalized.push(character);
            last_was_separator = false;
        } else if !last_was_separator {
            normalized.push('.');
            last_was_separator = true;
        }
    }
    normalized.trim_matches('.').to_owned()
}

/* === Private Functions === */

/* === Tests === */
