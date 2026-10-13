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
use std::{
    io::{Read, Seek, SeekFrom, Write},
    num::NonZeroUsize,
};
// Third Party Imports
use harmos::{
    MemorySize, Protobuf,
    operations::StreamOutput,
    storage::{FileReference, ObjectQuery, ObjectVersion, StorageView},
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use tonic::Status;
use zip::{ZipWriter, write::SimpleFileOptions};
// Local Imports
use dokime_app::{
    RuntimeHandle,
    core::TestCycleId,
    runtime::query::{
        checkpoint_records::ExecutionRecordQuery, checkpoint_telemetry::ExecutionTelemetryQuery,
    },
};
use dokime_contract::{execution::ExecutionCheckpoint, operations::content as wire};

/* === Constants === */
const CHUNK_BYTES: usize = 64 * 1024;
const DIRECTORY_BYTES: usize = 512 * 1024;
const ENTRY_MEMORY: usize = 4096;
/* === Macros === */
/* === Type Aliases === */
/* === Structs === */
struct BundleFile {
    file: std::fs::File,
    position: u64,
}

struct Archive {
    writer: ZipWriter<BundleFile>,
    entries: Vec<Entry>,
    allowance: usize,
    memory: usize,
}

#[derive(Serialize)]
struct Entry {
    path: String,
    size_bytes: u64,
    sha256: String,
}

#[derive(Serialize)]
struct Manifest<'a> {
    schema_version: u32,
    cycle_id: &'a str,
    entries: &'a [Entry],
}

struct EntryWriter<'a> {
    writer: &'a mut ZipWriter<BundleFile>,
    digest: Sha256,
    length: u64,
}
/* === Enums === */
/* === Traits === */
/* === Implementations === */
impl Write for BundleFile {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.position
            .checked_add(bytes.len() as u64)
            .filter(|position| *position <= super::super::MAX_CONTENT_BYTES)
            .ok_or_else(|| std::io::Error::other("bundle exceeds transfer limit"))?;
        let count = self.file.write(bytes)?;
        self.position += count as u64;
        Ok(count)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

impl Seek for BundleFile {
    fn seek(&mut self, from: SeekFrom) -> std::io::Result<u64> {
        self.position = self.file.seek(from)?;
        Ok(self.position)
    }
}

impl Read for BundleFile {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        let count = self.file.read(bytes)?;
        self.position += count as u64;
        Ok(count)
    }
}

impl Write for EntryWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let next = self
            .length
            .checked_add(bytes.len() as u64)
            .filter(|next| *next <= super::super::MAX_CONTENT_BYTES)
            .ok_or_else(|| std::io::Error::other("bundle entry exceeds transfer limit"))?;
        self.writer.write_all(bytes)?;
        self.digest.update(bytes);
        self.length = next;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

impl Archive {
    fn start(&mut self, path: &str) -> Result<EntryWriter<'_>, Status> {
        dokime::execution::saved::validate_relative_output_path(path).map_err(failure)?;
        if self.entries.iter().any(|entry| entry.path == path) {
            return Err(Status::data_loss("duplicate bundle path"));
        }
        let memory = self
            .memory
            .saturating_add(ENTRY_MEMORY)
            .saturating_add(path.len().saturating_mul(8));
        if memory > self.allowance {
            return Err(Status::resource_exhausted(
                "bundle directory exceeds original memory allowance",
            ));
        }
        self.memory = memory;
        let options = SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .last_modified_time(zip::DateTime::default())
            .unix_permissions(0o644)
            .large_file(true);
        self.writer.start_file(path, options).map_err(failure)?;
        Ok(EntryWriter {
            writer: &mut self.writer,
            digest: Sha256::new(),
            length: 0,
        })
    }

    fn json(&mut self, path: &str, value: &impl Serialize) -> Result<(), Status> {
        let mut writer = self.start(path)?;
        serde_json::to_writer(&mut writer, value).map_err(failure)?;
        let entry = Entry {
            path: path.to_owned(),
            size_bytes: writer.length,
            sha256: hex(&writer.digest.finalize()),
        };
        self.entries.push(entry);
        Ok(())
    }

    async fn file(
        &mut self,
        path: &str,
        view: &StorageView,
        file: &FileReference,
    ) -> Result<(), Status> {
        let mut reader = view.read_file(file).await.map_err(failure)?;
        let mut writer = self.start(path)?;
        while let Some(chunk) = reader
            .next_chunk(NonZeroUsize::new(CHUNK_BYTES).unwrap())
            .await
            .map_err(failure)?
        {
            // Keep The Original Charged Chunk Until Its Bytes Reach The Archive
            writer.write_all(chunk.as_bytes()).map_err(failure)?;
            tokio::task::yield_now().await;
        }
        let digest = writer.digest.finalize();
        if writer.length != file.length || digest.as_slice() != file.sha256 {
            return Err(Status::data_loss("sealed bundle file integrity differs"));
        }
        let entry = Entry {
            path: path.to_owned(),
            size_bytes: writer.length,
            sha256: hex(&digest),
        };
        self.entries.push(entry);
        Ok(())
    }
}

/* === Public Functions === */
pub(super) async fn download(
    runtime: &RuntimeHandle,
    cycle: &TestCycleId,
    maximum: usize,
    output: &mut StreamOutput<wire::CycleBundleFrame>,
) -> Result<(), Status> {
    let chunk_bytes = super::super::transfer_chunk_bytes(maximum);
    if chunk_bytes == 0 {
        return Err(Status::resource_exhausted(
            "bundle message allowance is empty",
        ));
    }
    let storage = runtime.storage();
    let mut query = ObjectQuery::<dokime::storage::objects::cycle::Cycle>::new();
    query.item = Some(cycle.as_str());
    let selection = storage.query(query).await.map_err(failure)?;
    let view = selection
        .view()
        .ok_or_else(|| Status::internal("bundle query has no saved view"))?
        .clone();
    let selected = dokime::storage::objects::cycle::read_in(
        storage,
        &view,
        cycle,
        ObjectVersion::Latest,
        selection.decoded_bytes(),
    )
    .await
    .map_err(failure)?
    .ok_or_else(|| Status::not_found("cycle not found"))?;
    let seal = selected
        .value
        .saved
        .as_ref()
        .ok_or_else(|| Status::failed_precondition("cycle has no saved final report"))?;
    seal.validate(&selected.value).map_err(failure)?;

    // Retain A Native Loan For The ZIP Directory And Hash Manifest
    let directory = storage
        .prepare_production(NonZeroUsize::new(DIRECTORY_BYTES).unwrap())
        .map_err(failure)?;
    let temporary = tempfile::tempfile().map_err(failure)?;
    let mut archive = Archive {
        writer: ZipWriter::new(BundleFile {
            file: temporary,
            position: 0,
        }),
        entries: Vec::new(),
        allowance: directory.allowance().get(),
        memory: 0,
    };
    archive.json("cycle.json", &selected.value)?;
    archive
        .file("reports/final.html", &view, &seal.report)
        .await?;
    for member in &seal.executions {
        let execution = dokime::storage::objects::execution::read_in(
            storage,
            &view,
            &member.execution_id,
            ObjectVersion::Exact(member.version.clone()),
            selection.decoded_bytes(),
        )
        .await
        .map_err(failure)?
        .ok_or_else(|| Status::data_loss("sealed bundle execution is missing"))?;
        if execution.position != member.position || execution.value.test_cycle_id != *cycle {
            return Err(Status::data_loss(
                "sealed bundle execution identity differs",
            ));
        }
        let saved = execution
            .value
            .saved
            .as_ref()
            .ok_or_else(|| Status::data_loss("bundle execution has no seal"))?;
        saved.validate(&execution.value).map_err(failure)?;
        let checkpoint = ExecutionCheckpoint {
            execution_id: member.execution_id.clone(),
            position: member.position.clone(),
            version: member.version.clone(),
            evidence_through: saved.source.evidence_through.clone(),
        };
        let prefix = format!("executions/{}", member.execution_id.as_str());
        archive.json(&format!("{prefix}/execution.json"), &execution.value)?;
        archive
            .file(
                &format!("{prefix}/reports/final.html"),
                &view,
                &saved.report,
            )
            .await?;
        archive
            .file(
                &format!("{prefix}/context.protobuf"),
                &view,
                &saved.source.context,
            )
            .await?;
        for file in &saved.source.outputs {
            archive
                .file(
                    &format!("{prefix}/outputs/{}", file.relative_path),
                    &view,
                    &file.file,
                )
                .await?;
        }
        runtime
            .execution_checkpoint_context(checkpoint.clone(), |context, _| {
                archive
                    .json(
                        &format!("{prefix}/document.json"),
                        &context.execution_documents,
                    )
                    .map_err(native_failure)?;
                archive
                    .json(&format!("{prefix}/notes.json"), &context.notes)
                    .map_err(native_failure)?;
                archive
                    .json(&format!("{prefix}/context.json"), &context)
                    .map_err(native_failure)?;
                Ok(0u64)
            })
            .await
            .map_err(failure)?;
        let tables = runtime
            .execution_checkpoint_evidence(checkpoint.clone(), |evidence, _| {
                archive
                    .json(&format!("{prefix}/evidence.json"), &evidence)
                    .map_err(native_failure)?;
                Ok(evidence)
            })
            .await
            .map_err(failure)?;
        let mut after = None;
        let mut page_number = 0;
        loop {
            let page = runtime
                .execution_checkpoint_records(
                    checkpoint.clone(),
                    ExecutionRecordQuery {
                        after,
                        limit: Some(100),
                    },
                )
                .await
                .map_err(failure)?;
            if !page.issues.is_empty() || page.issues_truncated {
                return Err(Status::data_loss(
                    "bundle records could not be read completely",
                ));
            }
            archive.json(&format!("{prefix}/records/{page_number:06}.json"), &*page)?;
            after = page.next.clone();
            if after.is_none() {
                break;
            }
            page_number += 1;
        }
        let mut previous = None;
        for table in &tables.telemetry {
            if previous.as_ref() == Some(&table.table) {
                continue;
            }
            previous = Some(table.table.clone());
            let mut after = None;
            let mut page_number = 0;
            loop {
                let page = runtime
                    .execution_checkpoint_telemetry(
                        checkpoint.clone(),
                        &table.contract.stream,
                        ExecutionTelemetryQuery {
                            after,
                            limit: Some(100),
                        },
                    )
                    .await
                    .map_err(failure)?;
                if !page.issues.is_empty() || page.issues_truncated {
                    return Err(Status::data_loss(
                        "bundle samples could not be read completely",
                    ));
                }
                // Encode Stream Identity In Hex So It Cannot Introduce ZIP Paths
                archive.json(
                    &format!(
                        "{prefix}/samples/{}/{page_number:06}.json",
                        hex(table.table.as_bytes())
                    ),
                    &*page,
                )?;
                after = page.next.clone();
                if after.is_none() {
                    break;
                }
                page_number += 1;
            }
        }
    }
    archive
        .entries
        .sort_by(|left, right| left.path.cmp(&right.path));
    let entries = std::mem::take(&mut archive.entries);
    archive.json(
        "manifest.json",
        &Manifest {
            schema_version: 1,
            cycle_id: cycle.as_str(),
            entries: &entries,
        },
    )?;
    let mut file = archive.writer.finish().map_err(failure)?;
    drop(archive.entries);
    drop(entries);
    drop(directory);
    let size = file.seek(SeekFrom::End(0)).map_err(failure)?;
    if size > super::super::MAX_CONTENT_BYTES {
        return Err(Status::resource_exhausted("bundle exceeds transfer limit"));
    }
    file.rewind().map_err(failure)?;
    let mut digest = Sha256::new();
    let mut buffer = [0; CHUNK_BYTES];
    loop {
        let count = file.read(&mut buffer).map_err(failure)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
        tokio::task::yield_now().await;
    }
    let sha256 = hex(&digest.finalize());
    file.rewind().map_err(failure)?;
    send(
        output,
        maximum,
        wire::CycleBundleFramePayload::Metadata(wire::CycleBundleMetadata {
            cycle_id: cycle.to_string(),
            filename: format!("{cycle}.zip"),
            content_type: "application/zip".into(),
            size_bytes: size,
            sha256: sha256.clone(),
        }),
    )
    .await?;
    let mut offset = 0;
    while offset < size {
        let count = file.read(&mut buffer[..chunk_bytes]).map_err(failure)?;
        if count == 0 {
            return Err(Status::data_loss("bundle ended before declared length"));
        }
        let memory = std::mem::size_of::<wire::CycleBundleFrame>().saturating_add(count);
        output
            .send_with(memory, || {
                Ok(wire::CycleBundleFrame {
                    payload: Some(wire::CycleBundleFramePayload::Chunk(wire::ArtifactChunk {
                        offset,
                        data: buffer[..count].to_vec(),
                    })),
                })
            })
            .await
            .map_err(failure)?;
        offset += count as u64;
    }
    send(
        output,
        maximum,
        wire::CycleBundleFramePayload::Complete(wire::ArtifactCompletion {
            size_bytes: size,
            sha256,
        }),
    )
    .await
}

/* === Private Functions === */
async fn send(
    output: &mut StreamOutput<wire::CycleBundleFrame>,
    maximum: usize,
    payload: wire::CycleBundleFramePayload,
) -> Result<(), Status> {
    let frame = wire::CycleBundleFrame {
        payload: Some(payload),
    };
    if frame.encoded_len() > maximum {
        return Err(Status::resource_exhausted(
            "bundle metadata exceeds message allowance",
        ));
    }
    output
        .send_with(frame.memory_size(), || Ok(frame))
        .await
        .map_err(failure)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn failure(error: impl std::fmt::Display) -> Status {
    Status::failed_precondition(format!("bundle export failed: {error}"))
}
fn native_failure(error: Status) -> harmos::Error {
    harmos::Error::backend("write cycle bundle", harmos::BackendFailure::Corrupt, error)
}

/* === Tests === */
