/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read Remote Listener State

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

#[harmos_domain::operation(response = RemoteListenerStatus, id = "dokime.remote.read_listener",
    policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Remote", method = "ReadListener"))]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadListener {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
