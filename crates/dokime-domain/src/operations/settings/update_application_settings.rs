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

use super::*;
use serde::{Deserialize, Serialize};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos_domain::operation(response = SettingsSnapshot, id = "dokime.settings.update_application_settings",
    policy = MUTATION, visibility = harmos_domain::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Settings", method = "UpdateApplicationSettings"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateApplicationSettings {
    pub update: UpdateSettings,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
