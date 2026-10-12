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
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
// Third Party Imports
use tracing_appender::non_blocking::{NonBlockingBuilder, WorkerGuard};
use tracing_subscriber::{EnvFilter, Layer, fmt};
// Local Imports
use super::logging::BoxedLayer;
use crate::{Error, Result};

/* === Constants === */
const MAX_HUB_LOG_BYTES: u64 = 32 * 1024 * 1024;

const BUFFERED_RECORDS: usize = 8_192;

const ROTATED_SUFFIX: &str = ".1";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct RotatingFile {
    path: PathBuf,
    previous: PathBuf,
    file: File,
    written: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RotatingFile {
    fn open(directory: &Path, filename: &str) -> Result<Self> {
        std::fs::create_dir_all(directory).map_err(|error| {
            Error::Setup(format!(
                "failed to create log directory {}: {error}",
                directory.display()
            ))
        })?;
        let path = directory.join(filename);
        let previous = directory.join(format!("{filename}{ROTATED_SUFFIX}"));
        let file = append(&path)?;
        let written = file.metadata().map(|metadata| metadata.len()).unwrap_or(0);
        Ok(Self {
            path,
            previous,
            file,
            written,
        })
    }

    fn rotate(&mut self) {
        let _ = self.file.flush();
        let _ = std::fs::remove_file(&self.previous);
        if std::fs::rename(&self.path, &self.previous).is_err() {
            return;
        }

        // The counter resets either way: with the live path renamed away
        // there is nothing left to rotate, and retrying on every record
        // would only churn the filesystem.
        self.written = 0;
        if let Ok(file) = append(&self.path) {
            self.file = file;
        }
    }
}

impl Write for RotatingFile {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self.written > 0 && self.written.saturating_add(bytes.len() as u64) > MAX_HUB_LOG_BYTES {
            self.rotate();
        }
        let written = self.file.write(bytes)?;
        self.written = self.written.saturating_add(written as u64);
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

/* === Public Functions === */
pub(crate) fn hub_log_layer(root: &Path) -> Result<(BoxedLayer, WorkerGuard)> {
    let path = hub_log_path(root);
    let directory = path
        .parent()
        .ok_or_else(|| Error::Setup(format!("the hub log path {} has no parent", path.display())))?
        .to_path_buf();
    let filename = path
        .file_name()
        .ok_or_else(|| {
            Error::Setup(format!(
                "the hub log path {} has no file name",
                path.display()
            ))
        })?
        .to_string_lossy()
        .into_owned();

    let (writer, guard) = NonBlockingBuilder::default()
        .buffered_lines_limit(BUFFERED_RECORDS)
        .lossy(true)
        .finish(RotatingFile::open(&directory, &filename)?);
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let layer = fmt::layer()
        .json()
        // Nested, never flattened: the reader lifts every non-message field
        // out of the `fields` object.
        .flatten_event(false)
        .with_target(true)
        // The span stack is what gives a record its context; the innermost
        // span alone repeats its tail and the reader discards it.
        .with_span_list(true)
        .with_current_span(false)
        .with_writer(writer)
        .with_filter(filter)
        .boxed();
    Ok((layer, guard))
}

/* === Private Functions === */
fn hub_log_path(root: &Path) -> PathBuf {
    use dokime_app::capabilities::exports::storage::DiagnosticPathKey;
    dokime_app::capabilities::exports::storage::path_conventions(root)
        .into_iter()
        .find(|convention| convention.key == DiagnosticPathKey::HubLog)
        .and_then(|convention| convention.absolute_path)
        .map_or_else(|| root.join("logs").join("dokime.jsonl"), PathBuf::from)
}

fn append(path: &Path) -> Result<File> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| {
            Error::Setup(format!(
                "failed to open the hub log {}: {error}",
                path.display()
            ))
        })
}

/* === Tests === */
