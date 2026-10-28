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

// Stream One Support Archive Without Secret Material.
#[harmos::operation(resources = [crate::hosting::Host<CollectDiagnostics>], response = (), output = DiagnosticsBundleFrame, id = "dokime.backend.collect_diagnostics",
        policy = TRANSFER, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Backend", method = "CollectDiagnostics"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CollectDiagnostics {
    #[row(
        description = "Ceiling on the hub-log tail copied into the archive. Absent uses 2 MiB and larger values are clamped to it; zero omits the log entirely. The whole log is never archived."
    )]
    pub max_log_tail_bytes: Option<u64>,
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

    crate::hosted!(CollectDiagnostics);
}
