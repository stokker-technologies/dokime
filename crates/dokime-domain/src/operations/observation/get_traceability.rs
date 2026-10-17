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
use super::{GetTraceabilityRequest, ORDINARY, TraceabilityPage};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Returns the bounded, scope-aware current traceability projection. This is a read of the current runtime state and never creates a checkpoint.
#[harmos::operation(request = GetTraceabilityRequest, response = TraceabilityPage, id = "dokime.observation.get_traceability", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Observation", method = "GetTraceability"))]
pub struct GetTraceability;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
