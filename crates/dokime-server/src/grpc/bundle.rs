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
use std::io::{Cursor, Write};
use std::path::PathBuf;
// Third Party Imports
use serde_json::Value;
use sha2::{Digest as _, Sha256};
use tonic::Status;
use zip::{ZipWriter, write::SimpleFileOptions};
// Local Imports
use super::redaction;
use dokime_domain::model::time::Timestamp;

/* === Constants === */
pub(super) const MAX_BUNDLE_BYTES: usize = 16 * 1024 * 1024;

pub(super) const MAX_LOG_TAIL_BYTES: u64 = 2 * 1024 * 1024;

const MANIFEST_PATH: &str = "manifest.json";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Bundle {
    pub(super) filename: String,
    pub(super) generated_at: Timestamp,
    pub(super) entries: Vec<Entry>,
    pub(super) bytes: Vec<u8>,
    pub(super) sha256: String,
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
    Json {
        path: String,
        value: Value,
    },
    Tail {
        path: String,
        source: PathBuf,
        max_bytes: u64,
    },
    File {
        path: String,
        source: PathBuf,
        max_bytes: u64,
    },
}

/* === Traits === */

/* === Implementations === */
impl Part {
    pub(super) fn json(path: impl Into<String>, value: Value) -> Self {
        Self::Json {
            path: path.into(),
            value,
        }
    }

    pub(super) fn tail(path: impl Into<String>, source: PathBuf, max_bytes: u64) -> Self {
        Self::Tail {
            path: path.into(),
            source,
            max_bytes,
        }
    }

    pub(super) fn file(path: impl Into<String>, source: PathBuf, max_bytes: u64) -> Self {
        Self::File {
            path: path.into(),
            source,
            max_bytes,
        }
    }

    fn path(&self) -> &str {
        match self {
            Self::Json { path, .. } | Self::Tail { path, .. } | Self::File { path, .. } => path,
        }
    }

    fn read(self) -> (Option<Vec<u8>>, Option<String>) {
        match self {
            Self::Json { mut value, .. } => {
                redaction::scrub(&mut value);
                match serde_json::to_vec_pretty(&value) {
                    Ok(bytes) => (Some(bytes), None),
                    Err(error) => (None, Some(redaction::text(&error.to_string()))),
                }
            }
            Self::Tail {
                source, max_bytes, ..
            } => match read_tail(&source, max_bytes) {
                Ok(bytes) => (Some(bytes), None),
                Err(note) => (None, Some(note)),
            },
            Self::File {
                source, max_bytes, ..
            } => match std::fs::read(&source) {
                Ok(bytes) if bytes.len() as u64 > max_bytes => (
                    None,
                    Some(format!("source exceeds the {max_bytes} byte entry ceiling")),
                ),
                Ok(bytes) => (Some(bytes), None),
                Err(error) => (None, Some(absence(&error))),
            },
        }
    }
}

/* === Public Functions === */
pub(super) fn log_tail_ceiling(requested: Option<u64>) -> u64 {
    requested
        .unwrap_or(MAX_LOG_TAIL_BYTES)
        .min(MAX_LOG_TAIL_BYTES)
}

pub(super) fn build(
    parts: Vec<Part>,
    context: Value,
    generated_at: Timestamp,
) -> Result<Bundle, Status> {
    let parts = parts
        .into_iter()
        .map(|part| {
            let path = part.path().to_owned();
            let (bytes, note) = part.read();
            (path, bytes, note)
        })
        .collect();
    assemble(parts, context, generated_at)
}

fn assemble(
    parts: Vec<(String, Option<Vec<u8>>, Option<String>)>,
    context: Value,
    generated_at: Timestamp,
) -> Result<Bundle, Status> {
    let filename = archive_filename(&generated_at);
    let mut members = Vec::with_capacity(parts.len());
    let mut entries = Vec::with_capacity(parts.len());

    for (path, bytes, note) in parts {
        entries.push(Entry {
            path: path.clone(),
            size_bytes: bytes.as_ref().map_or(0, |bytes| bytes.len() as u64),
            present: bytes.is_some(),
            note,
        });
        if let Some(bytes) = bytes {
            members.push((path, bytes));
        }
    }

    // Written first so an operator reading the archive sequentially sees the
    // inventory before the payloads it describes. It lists every other entry
    // and never itself.
    let mut manifest = serde_json::json!({
        "filename": filename,
        "generated_at": generated_at.as_str(),
        "backend": context,
        "entries": entries,
    });
    redaction::scrub(&mut manifest);
    let manifest = serde_json::to_vec_pretty(&manifest)
        .map_err(|_| Status::internal("diagnostics manifest could not be encoded"))?;

    let mut archive = Vec::new();
    write_archive(
        &mut archive,
        std::iter::once((MANIFEST_PATH.to_owned(), manifest)).chain(members),
    )
    .map_err(|_| Status::internal("diagnostics archive could not be assembled"))?;
    if archive.len() > MAX_BUNDLE_BYTES {
        return Err(Status::resource_exhausted(format!(
            "diagnostics archive exceeds {MAX_BUNDLE_BYTES} bytes; request a smaller log tail"
        )));
    }

    Ok(Bundle {
        filename,
        generated_at,
        entries,
        sha256: Sha256::digest(&archive)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
        bytes: archive,
    })
}

/* === Private Functions === */
fn write_archive(
    output: &mut Vec<u8>,
    members: impl Iterator<Item = (String, Vec<u8>)>,
) -> std::io::Result<()> {
    let mut writer = ZipWriter::new(Cursor::new(output));
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .large_file(false);
    for (path, bytes) in members {
        writer
            .start_file(path, options)
            .map_err(std::io::Error::other)?;
        writer.write_all(&bytes)?;
    }
    writer.finish().map_err(std::io::Error::other)?;
    Ok(())
}

fn read_tail(source: &PathBuf, max_bytes: u64) -> Result<Vec<u8>, String> {
    if max_bytes == 0 {
        return Err("the request asked for no log tail".to_owned());
    }
    let mut file = std::fs::File::open(source).map_err(|error| absence(&error))?;
    tail_from(&mut file, max_bytes)
}

fn tail_from(
    file: &mut (impl std::io::Read + std::io::Seek),
    max_bytes: u64,
) -> Result<Vec<u8>, String> {
    use std::io::SeekFrom;
    if max_bytes == 0 {
        return Err("the request asked for no log tail".to_owned());
    }
    let length = file
        .seek(SeekFrom::End(0))
        .map_err(|error| absence(&error))?;
    let start = length.saturating_sub(max_bytes);
    file.seek(SeekFrom::Start(start))
        .map_err(|error| absence(&error))?;
    let mut bytes = Vec::with_capacity(usize::try_from(length - start).unwrap_or_default());
    file.read_to_end(&mut bytes)
        .map_err(|error| absence(&error))?;
    if start > 0 {
        // The window opened mid-record. Drop that fragment so every archived
        // line is a whole one.
        if let Some(boundary) = bytes.iter().position(|byte| *byte == b'\n') {
            bytes.drain(..=boundary);
        }
    }
    Ok(bytes)
}

fn absence(error: &std::io::Error) -> String {
    if error.kind() == std::io::ErrorKind::NotFound {
        "this backend produces no such file".to_owned()
    } else {
        redaction::text(&error.to_string())
    }
}

fn archive_filename(generated_at: &Timestamp) -> String {
    let stamp = generated_at.to_datetime().map_or_else(
        |_| "unknown".to_owned(),
        |moment| {
            format!(
                "{:04}{:02}{:02}T{:02}{:02}{:02}Z",
                moment.year(),
                u8::from(moment.month()),
                moment.day(),
                moment.hour(),
                moment.minute(),
                moment.second(),
            )
        },
    );
    format!("dokime-diagnostics-{stamp}.zip")
}

/* === Tests === */
