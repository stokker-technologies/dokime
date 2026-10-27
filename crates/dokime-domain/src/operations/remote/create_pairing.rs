/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// CreatePairing

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

#[harmos_domain::message]
#[harmos_domain::operation(response = CreatedPairing, id = "dokime.remote.create_pairing", version = 1,
    policy = TRANSFER, visibility = harmos_domain::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Remote", method = "CreatePairing"))]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreatePairing {
    pub label: String,
    pub endpoint: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
