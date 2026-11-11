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

#[harmos::operation(resources = [crate::hosting::Host<UpdateApplicationSettings>], response = SettingsSnapshot, id = "dokime.settings.update_application_settings",
    policy = MUTATION, visibility = harmos::operations::Visibility::Host,
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
