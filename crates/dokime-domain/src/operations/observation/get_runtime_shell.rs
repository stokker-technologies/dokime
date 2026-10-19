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
// Returns a bounded cycle page at one exact runtime revision.
#[harmos_domain::message]
#[harmos_domain::operation(response = RuntimeShellSnapshot, id = "dokime.observation.get_runtime_shell", version = 1,
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Observation", method = "GetRuntimeShell"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct GetRuntimeShell {
    // Zero selects the default page size (50); the maximum is 100.
    #[row(description = "Zero selects the default page size (50); the maximum is 100.")]
    pub limit: u32,
    pub after_cycle_id: String,
    pub cursor: Option<Cursor>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
