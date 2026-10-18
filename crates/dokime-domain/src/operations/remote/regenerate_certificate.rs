/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Explicitly Regenerate Remote Certificate Identity

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
#[harmos_domain::operation(response = RemoteListenerStatus, id = "dokime.remote.regenerate_certificate", version = 1,
    policy = TRANSFER, visibility = harmos_domain::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Remote", method = "RegenerateCertificate"))]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RegenerateCertificate {
    pub certificate_names: Vec<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
