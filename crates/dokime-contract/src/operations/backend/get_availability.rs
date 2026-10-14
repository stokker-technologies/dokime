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
use super::{BackendAvailability, GetAvailabilityRequest, ORDINARY};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::operation(request = GetAvailabilityRequest, response = BackendAvailability, id = "dokime.backend.get_availability", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Backend", method = "GetAvailability"))]
pub struct GetAvailability;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
