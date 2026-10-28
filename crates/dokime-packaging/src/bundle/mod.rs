/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(crate) mod files;

pub fn read(path: &std::path::Path) -> Result<Vec<u8>, crate::Error> {
    files::read(path, harmos_packaging::bundle::Limits::default().archive_bytes)
}
