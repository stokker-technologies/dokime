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
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

// Third Party Imports

// Local Imports
use super::directory::Directories;
use crate::transport::{ArtifactChunk, MAX_ARTIFACT_CHUNK_BYTES};

/* === Constants === */

const IMPORT_READ_BYTES: usize = 64 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct Artifacts {
    sink: Arc<dyn ArtifactSink>,
    token: Arc<str>,
    next_ordinal: Arc<AtomicU64>,
    directories: Arc<Mutex<Directories>>,
}

pub struct ArtifactWriter {
    sink: Arc<dyn ArtifactSink>,
    transfer_id: String,
    name: String,
    mime: String,
    buffer: Vec<u8>,
    sequence: u64,
    closed: bool,
    abandoned: bool,
}

/* === Enums === */

enum Ending {
    Open,
    Complete,
    Abort,
}

/* === Traits === */

pub trait ArtifactSink: Send + Sync {
    fn push(&self, chunk: ArtifactChunk);
}

/* === Implementations === */

impl Artifacts {
    pub fn new(token: impl Into<String>, sink: Arc<dyn ArtifactSink>) -> Self {
        let token: Arc<str> = Arc::from(token.into());
        let next_ordinal = Arc::new(AtomicU64::new(0));
        Self {
            directories: Arc::new(Mutex::new(Directories::new(Arc::clone(&token), Arc::clone(&sink), Arc::clone(&next_ordinal)))),
            sink,
            token,
            next_ordinal,
        }
    }

    pub fn create(&self, name: impl AsRef<str>) -> crate::Result<ArtifactWriter> {
        let name = validated(name.as_ref())?;
        let mime = inferred_mime(&name)?;
        Ok(self.opened(name, mime)?)
    }

    pub fn write(&self, name: impl AsRef<str>, bytes: impl AsRef<[u8]>) -> crate::Result<()> {
        let mut writer = self.create(name)?;
        if let Err(error) = writer.write_all(bytes.as_ref()) {
            writer.abandon();
            return Err(error);
        }
        writer.finish()
    }

    pub fn directory(&self, name: impl AsRef<str>) -> crate::Result<PathBuf> {
        Ok(self.locked().directory(name.as_ref())?)
    }

    // The execution's artifact folder itself: every file written anywhere in
    // it is filed as an artifact named by its path relative to it.
    pub fn root(&self) -> crate::Result<PathBuf> {
        Ok(self.locked().root()?)
    }

    pub(crate) fn scan(&self) -> Vec<String> {
        self.locked().scan()
    }

    pub(crate) fn sweep(&self) -> Vec<String> {
        self.locked().sweep()
    }

    fn locked(&self) -> std::sync::MutexGuard<'_, Directories> {
        self.directories.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn opened(&self, name: String, mime: String) -> io::Result<ArtifactWriter> {
        Ok(ArtifactWriter {
            sink: Arc::clone(&self.sink),
            transfer_id: transfer_identity(&self.token, &self.next_ordinal),
            name,
            mime,
            buffer: Vec::new(),
            sequence: 0,
            closed: false,
            abandoned: false,
        })
    }

    pub fn import(&self, path: impl AsRef<Path>, name: impl AsRef<str>) -> crate::Result<()> {
        let name = validated(name.as_ref())?;
        let mime = inferred_mime(&name)?;
        let mut source = std::fs::File::open(path.as_ref())?;
        let mut writer = self.opened(name, mime)?;
        let mut buffer = vec![0; IMPORT_READ_BYTES];
        loop {
            let count = match source.read(&mut buffer) {
                Ok(0) => break,
                Ok(count) => count,
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                Err(error) => {
                    writer.abandon();
                    return Err(error.into());
                }
            };
            if let Err(error) = writer.write_all(&buffer[..count]) {
                writer.abandon();
                return Err(error);
            }
        }
        writer.finish()
    }
}

impl std::fmt::Debug for Artifacts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Artifacts")
            .field("token", &self.token)
            .field("opened", &self.next_ordinal.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

impl ArtifactWriter {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn write_all(&mut self, bytes: &[u8]) -> crate::Result<()> {
        Write::write_all(self, bytes).map_err(Into::into)
    }

    pub fn flush(&mut self) -> crate::Result<()> {
        Write::flush(self).map_err(Into::into)
    }

    pub fn finish(mut self) -> crate::Result<()> {
        self.emit(Ending::Complete);
        self.closed = true;
        Ok(())
    }

    pub fn abandon(mut self) {
        self.abandoned = true;
        self.buffer.clear();
        self.emit(Ending::Abort);
    }

    fn emit(&mut self, ending: Ending) {
        let (last, aborted) = match ending {
            Ending::Open => (false, false),
            Ending::Complete => (true, false),
            Ending::Abort => (true, true),
        };
        self.sink.push(ArtifactChunk {
            transfer_id: self.transfer_id.clone(),
            name: self.name.clone(),
            mime: self.mime.clone(),
            sequence: self.sequence,
            bytes: std::mem::take(&mut self.buffer),
            last,
            aborted,
            // A writer produces its content once, in order: there is nothing
            // written before a piece for a later piece to take back.
            replace: false,
        });
        self.sequence += 1;
    }
}

impl std::fmt::Debug for ArtifactWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArtifactWriter")
            .field("transfer_id", &self.transfer_id)
            .field("name", &self.name)
            .field("mime", &self.mime)
            .field("sequence", &self.sequence)
            .field("buffered", &self.buffer.len())
            .finish_non_exhaustive()
    }
}

impl Write for ArtifactWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.closed || self.abandoned {
            return Err(closed("write"));
        }
        let room = MAX_ARTIFACT_CHUNK_BYTES - self.buffer.len();
        let taken = bytes.len().min(room);
        self.buffer.extend_from_slice(&bytes[..taken]);
        if self.buffer.len() == MAX_ARTIFACT_CHUNK_BYTES {
            self.emit(Ending::Open);
        }
        Ok(taken)
    }

    fn flush(&mut self) -> io::Result<()> {
        if self.closed || self.abandoned {
            return Err(closed("flush"));
        }
        if !self.buffer.is_empty() {
            self.emit(Ending::Open);
        }
        Ok(())
    }
}

impl Drop for ArtifactWriter {
    fn drop(&mut self) {
        if self.closed || self.abandoned {
            return;
        }
        self.emit(Ending::Complete);
        self.closed = true;
    }
}

/* === Public Functions === */

pub(super) fn transfer_identity(token: &str, ordinals: &AtomicU64) -> String {
    let ordinal = ordinals.fetch_add(1, Ordering::Relaxed);

    format!("{token}:artifact:{ordinal}")
}

pub(super) fn validated(name: &str) -> io::Result<String> {
    let escapes =
        name.is_empty() || name.contains('\\') || name.split('/').any(|segment| segment.is_empty() || segment == "." || segment == "..");
    if escapes {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("artifact name {name:?} must be a non-empty relative `/`-separated path with no empty, `.`, or `..` segment"),
        ));
    }
    Ok(name.to_owned())
}

pub(crate) fn inferred_mime(name: &str) -> io::Result<String> {
    let extension = Path::new(name).extension().and_then(|extension| extension.to_str()).map(str::to_ascii_lowercase);
    let mime = match extension.as_deref() {
        Some("json") => "application/json",
        Some("csv") => "text/csv",
        Some("tsv") => "text/tab-separated-values",
        Some("log" | "txt") => "text/plain",
        Some("md") => "text/markdown",
        Some("xml") => "text/xml",
        Some("yaml" | "yml") => "application/yaml",
        Some("html") => "text/html",
        Some("pdf") => "application/pdf",
        Some("zip") => "application/zip",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        Some("svg") => "image/svg+xml",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("bin") => "application/octet-stream",
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "artifact name {name:?} does not say what the content is: name it with an extension whose media type is known, such as `.json`, `.csv`, `.txt`, `.log`, `.png` or `.bin`"
                ),
            ));
        }
    };

    Ok(mime.to_owned())
}

/* === Private Functions === */

fn closed(operation: &str) -> io::Error {
    io::Error::new(io::ErrorKind::BrokenPipe, format!("cannot {operation} an artifact that has already been closed"))
}

/* === Tests === */
