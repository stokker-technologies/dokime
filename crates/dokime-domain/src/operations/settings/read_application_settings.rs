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

// The backend's settings.json: appearance and the data root. Saved values apply to the next start of the backend; what is in effect now is reported beside them.
#[harmos_domain::message]
#[harmos_domain::operation(response = ApplicationSettings, id = "dokime.settings.read_application_settings", version = 1,
        policy = ORDINARY, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Settings", method = "ReadApplicationSettings"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadApplicationSettings {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
