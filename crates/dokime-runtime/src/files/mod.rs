/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The behaviour behind the file-watch declarations: which relative paths a
// watch tracks, and the rows and records its captured lines become.

/* === Modules === */

mod lines;
mod rules;

/* === Re-Exports === */

pub use lines::{InvalidLineMatcher, LineBuffer, LineMatch, LineMatchers};
pub use rules::{FileMatcher, InvalidRule};

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

/* === Private Functions === */

pub(crate) fn segment_name(base: &str, segment: u64) -> String {
    match base.rsplit_once('.') {
        Some((stem, extension)) if !extension.contains('/') => format!("{stem}.segment{segment}.{extension}"),
        _ => format!("{base}.segment{segment}.bin"),
    }
}

#[cfg(unix)]
pub(crate) fn identity(_: &std::path::Path, metadata: &std::fs::Metadata) -> Option<(u64, u64)> {
    use std::os::unix::fs::MetadataExt;
    Some((metadata.dev(), metadata.ino()))
}

#[cfg(windows)]
pub(crate) fn identity(path: &std::path::Path, _: &std::fs::Metadata) -> Option<(u64, u64)> {
    match file_id::get_low_res_file_id(path).ok()? {
        file_id::FileId::LowRes { volume_serial_number, file_index } => Some((u64::from(volume_serial_number), file_index)),
        _ => None,
    }
}

#[cfg(not(any(unix, windows)))]
pub(crate) fn identity(_: &std::path::Path, _: &std::fs::Metadata) -> Option<(u64, u64)> {
    None
}

/* === Tests === */
