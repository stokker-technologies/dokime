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
use std::fmt::Write as _;
use std::io::{Seek, Write};
use std::path::PathBuf;

// Third Party Imports
use dokime::harmos::storage::{ByteRange, FileReader, FrozenFile, ReadBatch};
use serde_json::Value;
use sha2::{Digest as _, Sha256};
use tonic::Status;
use zip::{ZipWriter, write::SimpleFileOptions};

// Local Imports
use super::redaction;
use dokime::domain::model::time::Timestamp;

/* === Constants === */

const MANIFEST_PATH: &str = "manifest.json";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Bundle {
    pub(super) metadata: ReadBatch<Metadata>,
    pub(super) reader: FileReader,
}

pub(super) struct Parts {
    session: dokime::storage::Manager,
    prepared: ReadBatch<Vec<ReadBatch<Prepared>>>,
}

pub(super) struct Metadata {
    pub(super) filename: String,
    pub(super) generated_at: Timestamp,
    pub(super) entries: Vec<Entry>,
    pub(super) size_bytes: u64,
    pub(super) sha256: String,
}

#[derive(serde::Serialize)]
struct Manifest<'a> {
    filename: &'a str,
    generated_at: &'a str,
    backend: &'a Value,
    entries: &'a [Entry],
}

struct CappedFile {
    file: std::fs::File,
    maximum: u64,
}

struct Prepared {
    path: String,
    input: Option<Input>,
    note: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize)]
pub(super) struct Entry {
    pub(super) path: String,
    pub(super) size_bytes: u64,
    pub(super) present: bool,
    pub(super) note: Option<String>,
}

/* === Enums === */

pub(super) enum Part {
    Json { path: String, value: Value },
    Tail { path: String, source: PathBuf, max_bytes: u64 },
}

enum Input {
    Opened { file: std::fs::File, range: ByteRange },
    Frozen { file: FrozenFile, range: ByteRange },
}

/* === Traits === */

/* === Implementations === */

impl Parts {
    pub(super) fn new(session: dokime::storage::Manager) -> Result<Self, Status> {
        let prepared = dokime::harmos::storage::ReadBatch::new(()).try_map(|_| Ok(Vec::new())).map_err(native_failure)?;
        Ok(Self { session, prepared })
    }

    pub(super) async fn push(self, part: Part) -> Result<Self, Status> {
        let part = part.prepare(&self.session).await?;
        let count = self.prepared.len().checked_add(1).ok_or_else(|| native_failure(allocation()))?;
        let capacity = self.prepared.capacity();
        let target = if count > capacity { capacity.checked_mul(2).unwrap_or(count).max(count) } else { capacity };

        let prepared = self
            .prepared
            .try_map(|mut prepared| {
                if target > capacity {
                    prepared.try_reserve_exact(target - prepared.len()).map_err(|_| allocation())?;
                }
                prepared.push(part);
                Ok(prepared)
            })
            .map_err(native_failure)?;
        Ok(Self { session: self.session, prepared })
    }
}

impl Write for CappedFile {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let position = self.file.stream_position()?;
        let end = position.checked_add(bytes.len() as u64);
        if end.is_none_or(|end| end > self.maximum) {
            return Err(std::io::Error::from(std::io::ErrorKind::FileTooLarge));
        }
        self.file.write(bytes)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

impl Seek for CappedFile {
    fn seek(&mut self, position: std::io::SeekFrom) -> std::io::Result<u64> {
        self.file.seek(position)
    }
}

impl Part {
    pub(super) fn json(path: impl Into<String>, value: Value) -> Self {
        Self::Json { path: path.into(), value }
    }

    pub(super) fn tail(path: impl Into<String>, source: PathBuf, max_bytes: u64) -> Self {
        Self::Tail { path: path.into(), source, max_bytes }
    }

    async fn prepare(self, session: &dokime::storage::Manager) -> Result<ReadBatch<Prepared>, Status> {
        let (path, input) = match self {
            Self::Json { path, value } => return prepare_json(path, value).await,
            Self::Tail { path, source, max_bytes } => (path, freeze_input(session, source, max_bytes).await?),
        };
        let (input, note) = match input {
            Ok(input) => (Some(input), None),
            Err(note) => (None, Some(note)),
        };

        dokime::harmos::storage::ReadBatch::new(())
            .try_map(|_| {
                Ok(Prepared {
                    path: path.clone(),
                    input,
                    note: note.clone(),
                })
            })
            .map_err(native_failure)
    }
}

impl Input {
    fn length(&self) -> u64 {
        match self {
            Self::Opened { range, .. } | Self::Frozen { range, .. } => range.limit.expect("prepared source range is bounded"),
        }
    }

    async fn read(&self, session: &dokime::storage::Manager) -> Result<FileReader, Status> {
        match self {
            Self::Frozen { file, range } => file.read(*range).await.map_err(native_failure),
            Self::Opened { file, range } => {
                let handle = file.try_clone().map_err(|_| Status::internal("diagnostics source could not be cloned"))?;
                session.read_opened_file(handle, *range).await.map_err(native_failure)
            }
        }
    }
}

/* === Public Functions === */

pub(super) fn log_tail_ceiling(requested: Option<u64>, maximum: u64) -> u64 {
    requested.unwrap_or(maximum).min(maximum)
}

pub(super) async fn build(parts: Parts, context: Value, generated_at: Timestamp, maximum: usize) -> Result<Bundle, Status> {
    let Parts { session, prepared } = parts;
    let (file, mut metadata) = assemble(&session, prepared, context, generated_at, maximum).await?;

    let handle = file.try_clone().map_err(|_| Status::internal("diagnostics archive could not be cloned"))?;
    let mut reader = session.read_opened_file(handle, ByteRange { offset: 0, limit: None }).await.map_err(native_failure)?;
    let mut digest = Sha256::new();
    let mut size_bytes = 0u64;
    while let Some(batch) = reader.next().await.map_err(native_failure)? {
        size_bytes = size_bytes
            .checked_add(batch.bytes().len() as u64)
            .filter(|length| *length <= maximum as u64)
            .ok_or_else(|| too_large(maximum))?;
        digest.update(batch.bytes());
    }
    reader.completion().map_err(native_failure)?;
    drop(reader);
    if file.metadata().map_err(|_| Status::internal("diagnostics archive could not be inspected"))?.len() != size_bytes {
        return Err(Status::data_loss("diagnostics archive length changed"));
    }
    metadata = metadata
        .try_map(|mut metadata| {
            metadata.sha256.try_reserve_exact(64).map_err(|_| allocation())?;
            for byte in digest.finalize() {
                write!(&mut metadata.sha256, "{byte:02x}").map_err(|_| allocation())?;
            }
            metadata.size_bytes = size_bytes;
            Ok(metadata)
        })
        .map_err(native_failure)?;
    let reader = session.read_opened_file(file, ByteRange { offset: 0, limit: Some(size_bytes) }).await.map_err(native_failure)?;
    Ok(Bundle { metadata, reader })
}

/* === Private Functions === */

async fn assemble(
    session: &dokime::storage::Manager, parts: ReadBatch<Vec<ReadBatch<Prepared>>>, mut context: Value, generated_at: Timestamp,
    maximum: usize,
) -> Result<(std::fs::File, ReadBatch<Metadata>), Status> {
    let metadata = dokime::harmos::storage::ReadBatch::new(())
        .try_map(|_| {
            let mut entries = Vec::new();
            entries.try_reserve_exact(parts.len()).map_err(|_| allocation())?;
            for part in parts.iter() {
                entries.push(Entry {
                    path: part.path.clone(),
                    size_bytes: part.input.as_ref().map_or(0, Input::length),
                    present: part.input.is_some(),
                    note: part.note.clone(),
                });
            }
            Ok(Metadata {
                filename: archive_filename(&generated_at)?,
                generated_at: generated_at.clone(),
                entries,
                size_bytes: 0,
                sha256: String::new(),
            })
        })
        .map_err(native_failure)?;
    // Written first so an operator reading the archive sequentially sees the
    // inventory before the payloads it describes. It lists every other entry
    // and never itself.
    redaction::scrub(&mut context);
    let held = metadata.clone();
    let mut writer = tokio::task::spawn_blocking(move || {
        let file = tempfile::NamedTempFile::new()?.into_file();
        let archive = CappedFile { file, maximum: maximum as u64 };
        let mut writer = ZipWriter::new(archive);
        writer.start_file(MANIFEST_PATH, file_options()).map_err(zip_failure)?;
        let manifest = Manifest {
            filename: &held.filename,
            generated_at: held.generated_at.as_str(),
            backend: &context,
            entries: &held.entries,
        };
        serde_json::to_writer_pretty(&mut writer, &manifest).map_err(|error| {
            let kind = error.io_error_kind().unwrap_or(std::io::ErrorKind::Other);
            std::io::Error::new(kind, error)
        })?;
        Ok::<_, std::io::Error>(writer)
    })
    .await
    .map_err(writer_task_failure)?
    .map_err(|error| archive_failure(error, maximum))?;

    for part in parts.iter() {
        let Some(input) = &part.input else { continue };
        let selected = part.clone();
        writer = tokio::task::spawn_blocking(move || {
            writer.start_file(&selected.path, file_options()).map_err(zip_failure)?;
            Ok::<_, std::io::Error>(writer)
        })
        .await
        .map_err(writer_task_failure)?
        .map_err(|error| archive_failure(error, maximum))?;
        {
            let mut reader = input.read(session).await?;
            while let Some(batch) = reader.next().await.map_err(native_failure)? {
                writer = tokio::task::spawn_blocking(move || {
                    writer.write_all(batch.bytes())?;
                    Ok::<_, std::io::Error>(writer)
                })
                .await
                .map_err(writer_task_failure)?
                .map_err(|error| archive_failure(error, maximum))?;
            }
            let completion = reader.completion().map_err(native_failure)?;
            if completion.bytes != input.length() {
                return Err(Status::data_loss("diagnostics member length changed"));
            }
        }
    }
    let archive = tokio::task::spawn_blocking(move || writer.finish().map_err(zip_failure))
        .await
        .map_err(writer_task_failure)?
        .map_err(|error| archive_failure(error, maximum))?;
    Ok((archive.file, metadata))
}

async fn prepare_json(path: String, mut value: Value) -> Result<ReadBatch<Prepared>, Status> {
    let prepared = dokime::harmos::storage::ReadBatch::new(())
        .try_map(|_| {
            Ok(Prepared {
                path: path.clone(),
                input: None,
                note: None,
            })
        })
        .map_err(native_failure)?;
    let prepared = tokio::task::spawn_blocking(move || {
        prepared.try_map(|mut prepared| {
            redaction::scrub(&mut value);
            let source = (|| {
                let mut file = tempfile::NamedTempFile::new().map_err(|error| redaction::text(&error.to_string()))?.into_file();
                serde_json::to_writer_pretty(&mut file, &value).map_err(|error| redaction::text(&error.to_string()))?;
                let length = file.metadata().map_err(|error| redaction::text(&error.to_string()))?.len();
                Ok::<_, String>(Input::Opened {
                    file,
                    range: ByteRange { offset: 0, limit: Some(length) },
                })
            })();
            let note = match source {
                Ok(input) => {
                    prepared.input = Some(input);
                    None
                }
                Err(note) => Some(note),
            };
            Ok::<_, dokime::harmos::storage::Error>((prepared, note))
        })
    })
    .await
    .map_err(writer_task_failure)?
    .map_err(native_failure)?;

    prepared
        .try_map(|(mut prepared, note)| {
            prepared.note = note.clone();
            Ok(prepared)
        })
        .map_err(native_failure)
}

async fn freeze_input(session: &dokime::storage::Manager, source: PathBuf, max_bytes: u64) -> Result<Result<Input, String>, Status> {
    if max_bytes == 0 {
        return Ok(Err("the request asked for no log tail".to_owned()));
    }
    let file = match tokio::fs::File::open(source).await {
        Ok(file) => file,
        Err(error) => return Ok(Err(absence(&error))),
    };
    let metadata = match file.metadata().await {
        Ok(metadata) => metadata,
        Err(error) => return Ok(Err(absence(&error))),
    };
    let length = metadata.len();
    let start = length.saturating_sub(max_bytes);
    let range = ByteRange {
        offset: start,
        limit: Some(length - start),
    };
    let file = match session.freeze_opened_file(file.into_std().await, range).await {
        Ok(file) => file,
        Err(error @ dokime::harmos::storage::Error::ResourceExhausted { .. }) => return Err(native_failure(error)),
        Err(error) => return Ok(Err(redaction::text(&error.to_string()))),
    };
    let mut offset = 0u64;
    if start > 0 {
        let mut reader = file.read(ByteRange { offset: 0, limit: None }).await.map_err(native_failure)?;
        let mut scanned = 0u64;
        while let Some(batch) = reader.next().await.map_err(native_failure)? {
            if let Some(boundary) = batch.bytes().iter().position(|byte| *byte == b'\n') {
                offset = scanned + boundary as u64 + 1;
                break;
            }
            scanned += batch.bytes().len() as u64;
        }
        if offset == 0 {
            reader.completion().map_err(native_failure)?;
        }
    }
    let range = ByteRange {
        offset,
        limit: Some(file.length() - offset),
    };
    Ok(Ok(Input::Frozen { file, range }))
}

fn file_options() -> SimpleFileOptions {
    SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored).large_file(false)
}

fn writer_task_failure(_: tokio::task::JoinError) -> Status {
    Status::internal("diagnostics archive writer failed")
}

fn archive_failure(error: std::io::Error, maximum: usize) -> Status {
    if error.kind() == std::io::ErrorKind::FileTooLarge {
        too_large(maximum)
    } else {
        Status::internal("diagnostics archive could not be assembled")
    }
}

fn absence(error: &std::io::Error) -> String {
    if error.kind() == std::io::ErrorKind::NotFound {
        "this backend produces no such file".to_owned()
    } else {
        redaction::text(&error.to_string())
    }
}

fn archive_filename_bytes(generated_at: &Timestamp) -> usize {
    let stamp = generated_at.to_datetime().map_or("unknown".len(), |moment| {
        let digits = moment.year().unsigned_abs().checked_ilog10().unwrap_or(0) as usize + 1;
        let year = (digits + usize::from(moment.year().is_negative())).max(4);
        year + "MMDDTHHMMSSZ".len()
    });
    "dokime-diagnostics-".len() + stamp + ".zip".len()
}

fn archive_filename(generated_at: &Timestamp) -> Result<String, dokime::harmos::storage::Error> {
    let mut filename = String::new();
    filename.try_reserve_exact(archive_filename_bytes(generated_at)).map_err(|_| allocation())?;
    filename.push_str("dokime-diagnostics-");
    match generated_at.to_datetime() {
        Ok(moment) => write!(
            &mut filename,
            "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
            moment.year(),
            u8::from(moment.month()),
            moment.day(),
            moment.hour(),
            moment.minute(),
            moment.second()
        )
        .map_err(|_| allocation())?,
        Err(_) => filename.push_str("unknown"),
    }
    filename.push_str(".zip");
    Ok(filename)
}

fn zip_failure(error: zip::result::ZipError) -> std::io::Error {
    match error {
        zip::result::ZipError::Io(error) => error,
        other => std::io::Error::other(other),
    }
}

fn allocation() -> dokime::harmos::storage::Error {
    dokime::harmos::storage::Error::ResourceExhausted { resource: "diagnostics archive metadata" }
}

pub(super) fn native_failure(error: dokime::harmos::storage::Error) -> Status {
    match error {
        dokime::harmos::storage::Error::ResourceExhausted { .. } => Status::resource_exhausted("diagnostics archive memory exhausted"),
        _ => Status::internal("diagnostics archive native read failed"),
    }
}

fn too_large(maximum: usize) -> Status {
    Status::resource_exhausted(format!("diagnostics archive exceeds {maximum} bytes; request a smaller log tail"))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{CappedFile, file_options, zip_failure};
    use std::io::Write as _;

    #[test]
    fn zip_writes_preserve_total_ceiling() {
        for (maximum, payload) in [(128, 256), (96, 1)] {
            let file = tempfile::NamedTempFile::new().unwrap().into_file();
            let observed = file.try_clone().unwrap();
            let output = CappedFile { file, maximum };
            let mut writer = zip::ZipWriter::new(output);
            let result = (|| {
                writer.start_file("payload.bin", file_options()).map_err(zip_failure)?;
                writer.write_all(&vec![7; payload])?;
                writer.finish().map_err(zip_failure)
            })();
            let error = result.err().expect("the ZIP exceeds its ceiling");
            assert_eq!(error.kind(), std::io::ErrorKind::FileTooLarge);
            assert!(observed.metadata().unwrap().len() <= maximum);
        }
    }
}
