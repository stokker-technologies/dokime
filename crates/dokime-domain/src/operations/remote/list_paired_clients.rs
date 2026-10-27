/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// ListPairedClients

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
#[harmos_domain::operation(response = PairedClientPage, id = "dokime.remote.list_paired_clients", version = 1,
    policy = TRANSFER, visibility = harmos_domain::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Remote", method = "ListPairedClients"))]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListPairedClients {
    pub cursor: Option<String>,
    pub limit: u32,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
