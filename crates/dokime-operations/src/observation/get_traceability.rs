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

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::Deserialize;
use serde::Serialize;

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// Returns the bounded, scope-aware current traceability projection. This is a read of the current runtime state and never creates a checkpoint.
#[harmos::operation(resources = [crate::hosting::Host<GetTraceability>], response = TraceabilityPage, id = "dokime.observation.get_traceability",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Observation", method = "GetTraceability"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetTraceability {
    // Absent selects all current traceability groups. A supplied scope selects
    // that scope and its visible ancestors/global values.
    #[row(
        description = "Absent selects all current traceability groups. A supplied scope selects that scope and its visible ancestors/global values."
    )]
    pub scope: Option<EvidenceScope>,

    // Zero selects the default page size (50); the maximum is 100.
    #[row(description = "Zero selects the default page size (50); the maximum is 100.")]
    pub limit: u32,

    // Continuations require the exact cursor returned by the preceding page.
    #[row(description = "Continuations require the exact cursor returned by the preceding page.")]
    pub after_traceability_id: String,

    pub cursor: Option<Cursor>,

    // Continuations must echo the host telemetry fingerprint returned by the
    // preceding page; an empty-host fingerprint is still nonempty.
    #[row(
        description = "Continuations must echo the host telemetry fingerprint returned by the preceding page; an empty-host fingerprint is still nonempty."
    )]
    pub host_traceability_fingerprint: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

/* === --- === Runtime Module === --- === */

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetTraceability);
}
