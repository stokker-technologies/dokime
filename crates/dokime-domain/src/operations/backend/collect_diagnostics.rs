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

// Local Imports
use super::{CollectDiagnosticsRequest, DiagnosticsBundleFrame, TRANSFER};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Builds one support archive and streams it. Read access, like GetDiagnostics; the archive never carries credential material.
#[harmos::operation(request = CollectDiagnosticsRequest, response = (), output = DiagnosticsBundleFrame, id = "dokime.backend.collect_diagnostics", version = 1,
        policy = TRANSFER, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Backend", method = "CollectDiagnostics"))]
pub struct CollectDiagnostics;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
