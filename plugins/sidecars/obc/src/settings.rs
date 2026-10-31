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

// Simulated Fixture Observers Advance Model Samples Without Real-Time Pacing.
pub const SAMPLE_INTERVAL_MS: u64 = 1;
pub(crate) const STEADY_INTERVAL_MS: u64 = SAMPLE_INTERVAL_MS;
pub(crate) const SYSTEM_INTERVAL_MS: u64 = SAMPLE_INTERVAL_MS;
// The Host Admits File Watch Polling At Fifty Milliseconds Or Longer.
pub(crate) const FILE_POLL_MS: u32 = 50;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime::settings]
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BenchSettings {
    pub evidence: Evidence,
}

#[dokime::section(label = "Evidence", status = GetStatus, description = "Where this bench looks for the optional reference image an attached evidence step captures.\nElectrical inputs stay template parameters — they vary case to case — but this path is deployment shape: it does not change from one test case to the next.")]
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
