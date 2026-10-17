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
#[harmos::message]
#[harmos::operation(response = PluginInstallation, input = StagePluginFrame, id = "dokime.plugins.stage", version = 1,
        policy = INSTALLATION, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Plugins", method = "Stage"))]
#[derive(Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Stage {}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
