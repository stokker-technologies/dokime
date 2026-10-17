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

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn read(path: &std::path::Path) -> Result<Vec<u8>, crate::Error> {
    files::read(path, harmos_packaging::bundle::Limits::default().archive_bytes)
}

/* === Private Functions === */

/* === Tests === */
