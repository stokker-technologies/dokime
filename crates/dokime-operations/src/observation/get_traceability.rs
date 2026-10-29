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

use super::*;

// Returns the bounded, scope-aware current traceability projection. This is a read of the current runtime state and never creates a checkpoint.
#[harmos::operation(resources = [crate::hosting::Host<GetTraceability>], response = TraceabilityPage, id = "dokime.observation.get_traceability",
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Observation", method = "GetTraceability"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetTraceability {
    #[row(
        description = "Absent selects all current traceability groups. A supplied scope selects that scope and its visible ancestors/global values."
    )]
    pub scope: Option<EvidenceScope>,

    #[row(description = "Zero selects the default page size (50); the maximum is 100.")]
    pub limit: u32,

    #[row(description = "Continuations require the exact cursor returned by the preceding page.")]
    pub after_traceability_id: String,

    pub cursor: Option<Cursor>,

    #[row(
        description = "Continuations must echo the host telemetry fingerprint returned by the preceding page; an empty-host fingerprint is still nonempty."
    )]
    pub host_traceability_fingerprint: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(GetTraceability);
}
