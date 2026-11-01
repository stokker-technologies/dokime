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
use serde::{Deserialize, Serialize};

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::operation(resources = [crate::hosting::Host<AwaitFileWatchArmed>], response = FileWatchArmed, id = "dokime.files.await_file_watch_armed",
        policy = FOLLOW, visibility = harmos::operations::Visibility::Public)]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AwaitFileWatchArmed {
    pub instance_id: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
