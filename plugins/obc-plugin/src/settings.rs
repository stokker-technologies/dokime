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
use std::path::PathBuf;

// Third Party Imports

// Local Imports
use crate::domain::operations::GetStatus;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime_runtime::settings]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BenchSettings {
    pub evidence: Evidence,
}

#[dokime_runtime::section(label = "Evidence", status = GetStatus, description = "Where this bench looks for the optional reference image an attached evidence step captures.\nElectrical inputs stay template parameters — they vary case to case — but this path is deployment shape: it does not change from one test case to the next.")]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Evidence {
    #[setting(
        label = "Reference Images Directory",
        default = ".",
        description = "Directory containing optional reference.png evidence. Electrical inputs are template parameters."
    )]
    pub images_directory: PathBuf,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
