/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Write Bounded Verified ZIP Payloads And Manifest

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    fs::File,
    io::{self, Read, Seek, SeekFrom, Write},
    path::Path,
};

// Third Party Imports
use harmos::storage::FileReader;
use serde::Serialize;
use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;
use zip::{ZipWriter, write::SimpleFileOptions};

// Local Imports
use super::{
    InventoryBudget,
    error::{Error, Result},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Writer {
    zip: ZipWriter<BoundedFile>,
    entries: Vec<Entry>,
    cancellation: CancellationToken,
}

struct BoundedFile {
    file: File,
    cancellation: CancellationToken,
    maximum_bytes: u64,
}

#[derive(Serialize)]
pub(super) struct Entry {
    pub path: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub source: Option<harmos::storage::FileReference>,
    pub relative_path: Option<String>,
}

struct LimitedBytes {
    bytes: Vec<u8>,
    maximum: usize,
}

#[derive(Serialize)]
struct Document<'a, T> {
    #[serde(flatten)]
    manifest: &'a T,

    payloads: &'a [Entry],
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Writer {
    pub fn open(path: &Path, cancellation: &CancellationToken) -> Result<Self> {
        let file = File::create_new(path)?;
        let bounded = BoundedFile {
            file,
            cancellation: cancellation.clone(),
            maximum_bytes: super::MAX_EXPORT_BYTES,
        };
        Ok(Self {
            zip: ZipWriter::new(bounded),
            entries: Vec::new(),
            cancellation: cancellation.clone(),
        })
    }

    fn start(&mut self, path: &str) -> Result<()> {
        if self.entries.len() as u64 >= super::MAX_INVENTORY_ENTRIES {
            return Err(Error::TooLarge);
        }
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored).large_file(true).unix_permissions(0o600);
        self.zip.start_file(path, options)?;
        Ok(())
    }

    pub async fn stored_file(
        &mut self, path: &str, relative_path: Option<&str>, source: harmos::storage::FileReference, mut reader: FileReader,
        budget: &mut InventoryBudget,
    ) -> Result<()> {
        self.start(path)?;
        let mut length = 0_u64;
        loop {
            let batch = tokio::select! {
                biased;
                _ = self.cancellation.cancelled() => return Err(Error::Cancelled),
                result = reader.next() => result?,
            };
            let Some(batch) = batch else {
                break;
            };
            if batch.offset() != length {
                return Err(Error::Invalid("saved file has a noncontiguous batch"));
            }
            length = length.checked_add(batch.bytes().len() as u64).ok_or(Error::TooLarge)?;
            self.zip.write_all(batch.bytes())?;
        }
        let completion = reader.completion()?;
        if length != source.length || completion.bytes != length {
            return Err(Error::Invalid("saved file length or digest differs from its reference"));
        }
        let entry = Entry {
            path: path.to_owned(),
            size_bytes: length,
            // Harmos Verifies The Whole Immutable File Before Returning Its Final Batch.
            sha256: hex(&source.sha256),
            source: Some(source),
            relative_path: relative_path.map(str::to_owned),
        };
        budget.reserve(&entry)?;
        self.entries.push(entry);
        Ok(())
    }

    pub fn telemetry(
        &mut self, path: &str, source: &crate::storage::sqlite::StreamFile, buffer_bytes: usize, budget: &mut InventoryBudget,
    ) -> Result<()> {
        self.start(path)?;
        let mut file = File::open(&source.path)?;
        let mut buffer = vec![0; buffer_bytes];
        let mut hash = Sha256::new();
        let mut size = 0_u64;
        loop {
            if self.cancellation.is_cancelled() {
                return Err(Error::Cancelled);
            }
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            size = size
                .checked_add(count as u64)
                .filter(|size| *size <= source.size_bytes)
                .ok_or(Error::Invalid("telemetry file grew after finalization"))?;
            hash.update(&buffer[..count]);
            self.zip.write_all(&buffer[..count])?;
        }
        let digest: [u8; 32] = hash.finalize().into();
        if size != source.size_bytes || digest != source.sha256 {
            return Err(Error::Invalid("telemetry file differs from catalog"));
        }
        let entry = Entry {
            path: path.to_owned(),
            size_bytes: size,
            sha256: hex(&digest),
            source: None,
            relative_path: None,
        };
        budget.reserve(&entry)?;
        self.entries.push(entry);
        Ok(())
    }

    pub async fn database(&mut self, reader: FileReader, budget: &mut InventoryBudget) -> Result<()> {
        self.start("evidence.sqlite")?;
        let (size_bytes, sha256) = copy(reader, &mut self.zip, &self.cancellation, super::MAX_EXPORT_BYTES).await?;
        let entry = Entry {
            path: "evidence.sqlite".into(),
            size_bytes,
            sha256,
            source: None,
            relative_path: None,
        };
        budget.reserve(&entry)?;
        self.entries.push(entry);
        Ok(())
    }

    pub fn finish(mut self, manifest: &impl Serialize) -> Result<File> {
        // Payload Checksums Exclude The Manifest Itself.
        let document = Document { manifest, payloads: &self.entries };
        let mut bytes = LimitedBytes {
            bytes: Vec::new(),
            maximum: super::MAX_MANIFEST_BYTES,
        };
        serde_json::to_writer(&mut bytes, &document)?;
        self.start("manifest.json")?;
        self.zip.write_all(&bytes.bytes)?;

        let output = self.zip.finish()?.file;
        output.sync_all()?;
        Ok(output)
    }
}

impl Write for LimitedBytes {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > self.maximum.saturating_sub(self.bytes.len()) {
            return Err(io::Error::other("evidence manifest exceeds its byte limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Write for BoundedFile {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.cancellation.is_cancelled() {
            return Err(io::Error::other("evidence export cancelled"));
        }
        let position = self.file.stream_position()?;
        if bytes.len() as u64 > self.maximum_bytes.saturating_sub(position) {
            return Err(io::Error::other("evidence archive exceeds its byte limit"));
        }
        self.file.write(bytes)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl Seek for BoundedFile {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        let position = self.file.seek(position)?;
        if position > self.maximum_bytes {
            return Err(io::Error::other("evidence archive exceeds its byte limit"));
        }
        Ok(position)
    }
}

/* === Public Functions === */

pub(super) async fn hash(reader: FileReader, cancellation: &CancellationToken, maximum: u64) -> Result<(u64, String)> {
    let mut sink = io::sink();
    copy(reader, &mut sink, cancellation, maximum).await
}

pub(super) fn output_path(execution: Option<&str>, relative: &str) -> Result<String> {
    if relative.split('/').any(|part| matches!(part, "" | "." | "..") || part.contains(['\\', ':', '\0'])) {
        return Err(Error::Invalid("artifact archive path is not a safe relative path"));
    }
    Ok(match execution {
        Some(execution) => format!("executions/{execution}/artifacts/{relative}"),
        None => format!("artifacts/{relative}"),
    })
}

pub(super) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/* === Private Functions === */

async fn copy(mut reader: FileReader, output: &mut impl Write, cancellation: &CancellationToken, maximum: u64) -> Result<(u64, String)> {
    let mut length = 0_u64;
    let mut hash = Sha256::new();
    loop {
        if cancellation.is_cancelled() {
            return Err(Error::Cancelled);
        }
        // Finish The Native Read Before Releasing Its Private Source Directory.
        let batch = reader.next().await?;
        if cancellation.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let Some(batch) = batch else { break };
        if batch.offset() != length {
            return Err(Error::Invalid("private export read is not contiguous"));
        }
        length = length.checked_add(batch.bytes().len() as u64).filter(|length| *length <= maximum).ok_or(Error::TooLarge)?;
        hash.update(batch.bytes());
        output.write_all(batch.bytes())?;
    }
    if reader.completion()?.bytes != length {
        return Err(Error::Invalid("private export read is incomplete"));
    }
    Ok((length, hex(&hash.finalize())))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_output_names() {
        assert_eq!(output_path(None, "sub/A.txt").unwrap(), "artifacts/sub/A.txt");
        assert_eq!(output_path(Some("execution"), "sub/A.txt").unwrap(), "executions/execution/artifacts/sub/A.txt");
        for path in ["../secret", "a/../../secret", "/absolute", "C:/secret", "a\\secret", "a//file"] {
            assert!(output_path(None, path).is_err());
        }
    }
}
