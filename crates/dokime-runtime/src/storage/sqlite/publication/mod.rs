/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// File Publication Durability Before Group Acknowledgement

/* === Modules === */

#[cfg(unix)]
pub(super) mod unix;
#[cfg(target_os = "windows")]
pub(super) mod windows;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::time::Instant;

// Third Party Imports

// Local Imports
#[cfg(unix)]
pub(super) use self::unix as platform;
#[cfg(target_os = "windows")]
pub(super) use self::windows as platform;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
pub(super) struct Publication {
    directories: BTreeSet<PathBuf>,
    sources: BTreeSet<PathBuf>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Publication {
    pub(super) fn install(&mut self, source: &Path, destination: &Path) -> Result<(), super::Error> {
        let started = Instant::now();
        platform::install(source, destination)?;
        super::FILE_SYNC_NS.fetch_add(started.elapsed().as_nanos() as u64, Ordering::Relaxed);
        super::DURABLE_FILES.fetch_add(1, Ordering::Relaxed);
        self.directories.insert(destination.parent().expect("file parent").to_owned());
        Ok(())
    }

    pub(super) fn retire(&mut self, source: PathBuf) {
        self.sources.insert(source);
    }

    pub(super) fn touched(&mut self, path: &Path) {
        self.directories.insert(path.parent().expect("file parent").to_owned());
    }

    pub(super) fn sync(&mut self) -> Result<(), super::Error> {
        for directory in &self.directories {
            platform::sync_directory(directory)?;
        }
        self.directories.clear();
        Ok(())
    }

    pub(super) fn committed(&mut self) {
        for source in std::mem::take(&mut self.sources) {
            if let Err(error) = std::fs::remove_file(&source)
                && error.kind() != std::io::ErrorKind::NotFound
            {
                tracing::warn!(%error, ?source, "retained redundant staging link after publication");
            }
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
