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
    InventoryBudget, MAX_INVENTORY_ENTRIES, MAX_MANIFEST_BYTES,
    error::{Error, Result},
};
use dokime_domain::model::exports::EVIDENCE_EXPORT_MAX_BYTES;

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
}

#[derive(Serialize)]
pub(super) struct Entry {
    pub path: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub source: Option<harmos::storage::FileReference>,
    pub relative_path: Option<String>,
}

struct LimitedBytes(Vec<u8>);

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
        let bounded = BoundedFile { file, cancellation: cancellation.clone() };
        Ok(Self { zip: ZipWriter::new(bounded), entries: Vec::new(), cancellation: cancellation.clone() })
    }

    fn start(&mut self, path: &str) -> Result<()> {
        if self.entries.len() >= MAX_INVENTORY_ENTRIES {
            return Err(Error::TooLarge);
        }
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored).large_file(true).unix_permissions(0o600);
        self.zip.start_file(path, options)?;
        Ok(())
    }

    pub async fn stored_file(
        &mut self, path: &str, relative_path: Option<&str>, mut reader: FileReader, budget: &mut InventoryBudget,
    ) -> Result<()> {
        let source = reader.reference().clone();
        self.start(path)?;
        let mut length = 0_u64;
        let mut hash = Sha256::new();
        loop {
            let batch = tokio::select! {
                biased;
                _ = self.cancellation.cancelled() => return Err(Error::Cancelled),
                result = reader.next() => result?,
            };
            let Some(batch) = batch else {
                break;
            };
            if batch.offset != length {
                return Err(Error::Invalid("saved file has a noncontiguous batch"));
            }
            length = length.checked_add(batch.bytes.len() as u64).ok_or(Error::TooLarge)?;
            hash.update(&batch.bytes);
            self.zip.write_all(&batch.bytes)?;
        }
        let completion = reader.completion()?;
        let sha256: [u8; 32] = hash.finalize().into();
        if length != source.length || completion.bytes != length || sha256 != source.sha256 {
            return Err(Error::Invalid("saved file length or digest differs from its reference"));
        }
        let entry = Entry {
            path: path.to_owned(),
            size_bytes: length,
            sha256: hex(&sha256),
            source: Some(source),
            relative_path: relative_path.map(str::to_owned),
        };
        budget.reserve(&entry)?;
        self.entries.push(entry);
        Ok(())
    }

    pub fn database(&mut self, path: &Path, budget: &mut InventoryBudget) -> Result<()> {
        self.start("evidence.sqlite")?;
        let mut input = File::open(path)?;
        let (size_bytes, sha256) = copy(&mut input, &mut self.zip, &self.cancellation)?;
        let entry = Entry { path: "evidence.sqlite".into(), size_bytes, sha256, source: None, relative_path: None };
        budget.reserve(&entry)?;
        self.entries.push(entry);
        Ok(())
    }

    pub fn finish(mut self, manifest: &impl Serialize) -> Result<(File, u64, String)> {
        // Payload Checksums Exclude The Manifest Itself.
        let document = Document { manifest, payloads: &self.entries };
        let mut bytes = LimitedBytes(Vec::new());
        serde_json::to_writer(&mut bytes, &document)?;
        self.start("manifest.json")?;
        self.zip.write_all(&bytes.0)?;

        let mut output = self.zip.finish()?.file;
        output.sync_all()?;
        output.seek(SeekFrom::Start(0))?;
        let (length, sha256) = copy(&mut output, &mut io::sink(), &self.cancellation)?;
        output.seek(SeekFrom::Start(0))?;
        Ok((output, length, sha256))
    }
}

impl Write for LimitedBytes {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if bytes.len() > MAX_MANIFEST_BYTES.saturating_sub(self.0.len()) {
            return Err(io::Error::other("evidence manifest exceeds its byte limit"));
        }
        self.0.extend_from_slice(bytes);
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
        if bytes.len() as u64 > EVIDENCE_EXPORT_MAX_BYTES.saturating_sub(position) {
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
        if position > EVIDENCE_EXPORT_MAX_BYTES {
            return Err(io::Error::other("evidence archive exceeds its byte limit"));
        }
        Ok(position)
    }
}

/* === Public Functions === */
pub(super) fn output_path(execution: &str, index: usize, relative: &str) -> String {
    let safe = super::super::storage::paths::safe_artifact_path(relative);
    let filename = Path::new(safe.rsplit('/').next().unwrap_or("file"));
    let stem = filename.file_stem().and_then(|value| value.to_str()).unwrap_or("file");
    let stem = stem.chars().take(160).collect::<String>();
    let suffix = filename.extension().and_then(|value| value.to_str()).map(|value| format!(".{value}")).unwrap_or_default();
    // A Numeric Prefix Avoids Windows Device Names And Sanitization Collisions.
    format!("files/{execution}/{index:06}-{stem}{suffix}")
}

/* === Private Functions === */
fn copy(input: &mut impl Read, output: &mut impl Write, cancellation: &CancellationToken) -> Result<(u64, String)> {
    let mut buffer = [0_u8; 64 * 1024];
    let mut length = 0_u64;
    let mut hash = Sha256::new();
    loop {
        if cancellation.is_cancelled() {
            return Err(Error::Cancelled);
        }
        let read = input.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        length = length.checked_add(read as u64).filter(|length| *length <= EVIDENCE_EXPORT_MAX_BYTES).ok_or(Error::TooLarge)?;
        hash.update(&buffer[..read]);
        output.write_all(&buffer[..read])?;
    }
    Ok((length, hex(&hash.finalize())))
}

pub(super) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/* === Tests === */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_output_names() {
        assert_eq!(output_path("execution", 0, "sub/CON.txt"), "files/execution/000000-con.txt");
        assert_ne!(output_path("execution", 0, "sub/A.txt"), output_path("execution", 1, "other/a.txt"));
        let long = format!("{}.csv", "x".repeat(1000));
        let path = output_path("execution", 9, &long);
        assert!(path.ends_with(".csv"));
        assert!(path.rsplit('/').next().unwrap().len() < 255);
        assert_eq!(path.split('/').count(), 3);
        assert!(!output_path("execution", 0, "../A?:b.csv").contains(".."));
    }
}
