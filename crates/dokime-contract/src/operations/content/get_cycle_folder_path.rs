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
use super::{GetCycleFolderPathRequest, LocalPathResponse, ORDINARY};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Reserves a local cycle folder lookup. Remote listeners reject it; the native runtime currently exposes no cycle directory.
#[harmos::operation(request = GetCycleFolderPathRequest, response = LocalPathResponse, id = "dokime.content.get_cycle_folder_path", version = 1,
        policy = ORDINARY, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Content", method = "GetCycleFolderPath"))]
pub struct GetCycleFolderPath;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
