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
use std::path::PathBuf;

use dokime::plugin::Secret;

use crate::operations::chamber::DetectPort;
use crate::operations::status::GetStatus;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime::settings]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BenchSettings {
    pub chamber: Chamber,
}

#[dokime::section(label = "Chamber", status = GetStatus, description = "Where the bench sensor is and how it uploads.\nThe upload token is an `Option<Secret>` because this bench runs without one: the shape of the field is what says whether an operator must fill it in, so a required secret is written as a bare `Secret` instead.")]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Chamber {
                                        #[setting(label = "Sensor Port", discovered = DetectPort, description = "Serial port of the bench sensor.\n`discovered = DetectPort` names the operation that computes it. The host calls it once the plugin is initialized, and only for a field nobody saved, and what it found is stored beside the saved values with a marker the card shows. It is not used by a run until an operator saves it, which is why this is an `Option<PathBuf>`: the bench has to open without it, and a discovered value is a suggestion rather than a setting.")]
    pub sensor_port: Option<PathBuf>,

        #[setting(label = "Cloud Token", description = "Optional token for result upload.")]
    pub cloud_token: Option<Secret>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
