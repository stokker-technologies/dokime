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
use super::{GetRuntimeShellRequest, ORDINARY, RuntimeShellSnapshot};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Returns a bounded cycle page at one exact runtime revision.
#[harmos::operation(request = GetRuntimeShellRequest, response = RuntimeShellSnapshot, id = "dokime.observation.get_runtime_shell", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Observation", method = "GetRuntimeShell"))]
pub struct GetRuntimeShell;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
