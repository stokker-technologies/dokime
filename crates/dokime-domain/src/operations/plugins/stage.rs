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
#[harmos_domain::operation(response = PluginInstallation, input = StagePluginFrame, id = "dokime.plugins.stage", version = 1,
        policy = INSTALLATION, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Plugins", method = "Stage"))]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Stage {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
