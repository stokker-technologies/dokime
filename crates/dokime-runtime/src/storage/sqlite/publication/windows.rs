/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Windows Publication Through Writable File Handles

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::Path;

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

pub(super) fn install(source: &Path, destination: &Path) -> Result<(), super::super::Error> {
    // Keep The Acknowledged Source Until The Catalog Commit. Flush The Linked
    // File's Data And NTFS Name Metadata Together Through Its Writable Handle.
    std::fs::hard_link(source, destination)?;
    std::fs::OpenOptions::new().read(true).write(true).open(destination)?.sync_all()?;
    Ok(())
}

pub(super) fn sync_directory(_: &Path) -> Result<(), super::super::Error> {
    // Windows Publication Flushes The Renamed File's NTFS Metadata Through Its
    // Writable Handle; Opening And Flushing A Directory Is Not Supported.
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
