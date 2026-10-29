/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(crate) mod files;

use serde::{Deserialize, Serialize};

pub(crate) const COMPATIBILITY_FILE: &str = "dokime.json";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Compatibility {
    pub schema: u32,
    pub requires_dokime: semver::VersionReq,
}

pub fn read(path: &std::path::Path) -> Result<Vec<u8>, crate::Error> {
    files::read(path, harmos_packaging::bundle::Limits::default().archive_bytes)
}
