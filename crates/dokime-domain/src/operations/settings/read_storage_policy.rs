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

#[harmos_domain::message]
#[harmos_domain::operation(response = StoragePolicyView, id = "dokime.settings.read_storage_policy", version = 1,
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Settings", method = "ReadStoragePolicy"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadStoragePolicy {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
