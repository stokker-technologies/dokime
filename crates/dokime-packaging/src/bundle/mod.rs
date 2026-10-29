/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub(crate) mod files;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports

/* === Constants === */

pub(crate) const COMPATIBILITY_FILE: &str = "dokime.json";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Compatibility {
    pub schema: u32,
    pub requires_dokime: semver::VersionReq,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn read(path: &std::path::Path) -> Result<Vec<u8>, crate::Error> {
    files::read(path, harmos_packaging::bundle::Limits::default().archive_bytes)
}

/* === Private Functions === */

/* === Tests === */
