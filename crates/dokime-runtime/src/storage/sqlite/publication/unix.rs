/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// POSIX Publication And Coalesced Directory Syncs

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    fs::{File, OpenOptions},
    path::Path,
    sync::atomic::Ordering,
    time::Instant,
};

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
    std::fs::hard_link(source, destination)?;
    OpenOptions::new().read(true).write(true).open(destination)?.sync_all()?;
    Ok(())
}

pub(super) fn sync_directory(path: &Path) -> Result<(), super::super::Error> {
    let started = Instant::now();
    File::open(path)?.sync_all()?;
    super::super::DIRECTORY_SYNCS.fetch_add(1, Ordering::Relaxed);
    super::super::DIRECTORY_SYNC_NS.fetch_add(started.elapsed().as_nanos() as u64, Ordering::Relaxed);
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
