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
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos_domain::message]
#[harmos_domain::operation(response = PluginInstallation, input = InstallPluginFrame, id = "dokime.plugins.install", version = 1,
        policy = INSTALLATION, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Plugins", method = "Install"))]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Install {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
