/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// A directory on the backend host. Discovery never follows symbolic links, so
// a watch cannot leave its root or loop through a linked tree.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::{Path, PathBuf};

// Third Party Imports
use tokio::io::{AsyncReadExt, AsyncSeekExt};

// Local Imports
use super::source::{Entry, Listing};
use super::{SCAN_ENTRIES, capped, invalid, unreachable_source};
use crate::files::{FileMatcher, identity};
use crate::runtime::Result;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(in crate::domains::files) struct LocalSource {
    root: PathBuf,
    depth: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl LocalSource {
    pub(super) async fn open(root: &str, wait: bool) -> Result<Self> {
        let path = Path::new(root);
        if !path.is_absolute() {
            return Err(invalid("a local watch root must be an absolute path"));
        }
        let root = match tokio::fs::canonicalize(path).await {
            Ok(root) => root,
            Err(source) if wait && source.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Self { root: path.to_owned(), depth: usize::MAX });
            }
            Err(source) => return Err(source_error(root, source)),
        };
        if !tokio::fs::metadata(&root).await.map_err(|source| unreachable_source(format!("watch root {root:?}: {source}")))?.is_dir() {
            return Err(invalid("a local watch root must be a directory"));
        }
        Ok(Self { root, depth: usize::MAX })
    }

    pub(super) fn top_level(mut self) -> Self {
        self.depth = 1;
        self
    }

    pub(super) async fn scan(&self, matcher: &FileMatcher) -> Result<Listing> {
        let root = self.root.clone();
        let matcher = matcher.clone();
        let depth = self.depth;
        tokio::task::spawn_blocking(move || walk(&root, depth, &matcher)).await?
    }

    pub(super) async fn read_range(&self, relative: &str, offset: u64, length: u64, sink: &mut tokio::fs::File) -> Result<u64> {
        let path = self.root.join(relative);
        let mut file = tokio::fs::File::open(&path).await.map_err(|source| source_error(relative, source))?;
        file.seek(std::io::SeekFrom::Start(offset)).await.map_err(|source| unreachable_source(format!("{relative}: {source}")))?;
        let copied =
            tokio::io::copy(&mut file.take(length), sink).await.map_err(|source| unreachable_source(format!("{relative}: {source}")))?;
        Ok(copied)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn walk(root: &Path, depth: usize, matcher: &FileMatcher) -> Result<Listing> {
    let metadata = std::fs::metadata(root).map_err(|source| source_error(&root.display().to_string(), source))?;
    if !metadata.is_dir() {
        return Err(invalid("a local watch root must be a directory"));
    }
    let mut listing = Listing::new();
    let mut visited = 0_usize;
    for entry in walkdir::WalkDir::new(root).follow_links(false).min_depth(1).max_depth(depth) {
        visited += 1;
        if visited > SCAN_ENTRIES {
            return Err(capped(format!("the watch root holds more than {SCAN_ENTRIES} entries")));
        }
        let entry = match entry {
            Ok(entry) => entry,
            // A Concurrently Removed Entry Is Simply Absent From This Pass
            Err(source) if source.io_error().is_some_and(|io| io.kind() == std::io::ErrorKind::NotFound) => {
                continue;
            }
            Err(source) => return Err(invalid(format!("cannot scan watch root: {source}"))),
        };
        if !entry.file_type().is_file() {
            continue;
        }
        let Some(relative) = relative_path(root, entry.path()) else {
            tracing::warn!(path = %entry.path().display(), "file watch skips a path that is not UTF-8");
            continue;
        };
        if !matcher.matches(&relative) {
            continue;
        }
        let metadata = match entry.metadata() {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        std::fs::File::open(entry.path()).map_err(|source| source_error(&relative, source))?;
        listing.insert(
            relative,
            Entry { size: metadata.len(), modified: metadata.modified().ok(), identity: identity(entry.path(), &metadata) },
        );
    }
    Ok(listing)
}

fn source_error(path: &str, source: std::io::Error) -> crate::runtime::Error {
    if source.kind() == std::io::ErrorKind::NotFound {
        return crate::runtime::Error::not_found("file_watch_missing", format!("watch source {path:?}: {source}"));
    }
    invalid(format!("watch source {path:?}: {source}"))
}

fn relative_path(root: &Path, path: &Path) -> Option<String> {
    let relative = path.strip_prefix(root).ok()?;
    let mut segments = Vec::new();
    for component in relative.components() {
        segments.push(component.as_os_str().to_str()?);
    }
    Some(segments.join("/"))
}

/* === Tests === */
