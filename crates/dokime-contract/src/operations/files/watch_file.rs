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
use super::{FOLLOW, WatchFileOutcome, WatchFileRequest};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Shared, so a template reaches it through `ctx.host()` without listing it.
// A step built-in only: listeners watch through `FileWatch`, and a gRPC
// binding would make the contract a plugin requires differ from the host's.
#[harmos::operation(request = WatchFileRequest, response = WatchFileOutcome, id = "dokime.files.watch_file", version = 1,
        policy = FOLLOW, visibility = harmos::operations::Visibility::Shared)]
pub struct WatchFile;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
