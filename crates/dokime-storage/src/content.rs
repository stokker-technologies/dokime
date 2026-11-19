/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Retained Native Content Metadata

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Seek, Write};
use std::num::NonZeroUsize;

// Third Party Imports
use harmos_protocol::model::FileReference;
use prost::Message;
use sha2::{Digest, Sha256};
use tempfile::TempPath;
use zip::{CompressionMethod, DateTime, System, ZipWriter, write::SimpleFileOptions};

// Local Imports
use dokime_protocol::host::CyclePhase;
use crate::persisted::Scope;
use crate::{Error, Result, Store};
use dokime_protocol::host::CycleSummary;
use dokime_protocol::host::content::ArtifactsCreateRequest;
use dokime_protocol::host::content::ExecutionFileMetadata;
use dokime_protocol::host::evidence::OperatorNote;
use dokime_protocol::model::ArtifactMetadata;

/* === Constants === */

pub const CONTENT_CHUNK_BYTES: usize = 64 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
pub struct ArchivedContent {
    pub files: Vec<ArchivedNativeFile>,

    /// Original StoredArtifact protobuf bytes retained by the content owner.
    pub artifacts: Vec<Vec<u8>>,

    pub execution_files: Vec<StoredExecutionFile>,

    pub notes: Vec<OperatorNote>,

    pub report: Option<ArchivedNativeFile>,
}

#[derive(Clone, PartialEq, Message)]
pub struct CycleArchive {
    #[prost(message, optional, tag = "1")]
    pub cycle: Option<CycleSummary>,

    #[prost(message, repeated, tag = "2")]
    pub notes: Vec<OperatorNote>,

    #[prost(bytes = "vec", repeated, tag = "3")]
    pub artifacts: Vec<Vec<u8>>,
}

pub struct ArchivedNativeFile {
    pub reference: FileReference,
    pub path: TempPath,
}

#[derive(Clone, PartialEq, Message)]
pub struct StoredArtifact {
    #[prost(message, optional, tag = "1")]
    pub request: Option<ArtifactsCreateRequest>,

    #[prost(message, optional, tag = "2")]
    pub artifact: Option<ArtifactMetadata>,
}

#[derive(Clone, PartialEq, Message)]
pub struct StoredExecutionFile {
    #[prost(message, optional, tag = "1")]
    pub metadata: Option<ExecutionFileMetadata>,

    #[prost(message, optional, tag = "2")]
    pub file: Option<FileReference>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn has_incompatible_content(&self) -> Result<bool> {
        Ok(self.lock()?.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name IN ('content_objects','content_chunks'))",
            [],
            |row| row.get(0),
        )?)
    }
    pub fn write_cycle_archive(
        &self, cycle: &str, page: NonZeroUsize, output: &mut impl Write, contents: &mut HashMap<String, ArchivedContent>,
        mut content: ArchivedContent,
    ) -> Result<()> {
        let options = archive_options();
        let mut archive = ZipWriter::new_stream(output);
        let mut after = String::new();
        let mut manifest = Vec::new();
        loop {
            let ids = self.cycle_execution_page(cycle, &after, page)?;
            if ids.is_empty() {
                break;
            }
            for id in ids {
                after = id.clone();
                let mut execution = tempfile::tempfile()?;
                self.write_execution_archive(&id, &mut execution, contents.remove(&id).unwrap_or_default())?;
                let size = execution.stream_position()?;
                execution.rewind()?;
                let member = format!("executions/{id}.zip");
                crate::validate_execution_path(&member)?;
                archive.start_file(&member, options)?;
                let mut hash = Sha256::new();
                let mut buffer = [0u8; 64 * 1024];
                loop {
                    let length = execution.read(&mut buffer)?;
                    if length == 0 {
                        break;
                    }
                    hash.update(&buffer[..length]);
                    archive.write_all(&buffer[..length])?;
                }
                manifest.push(serde_json::json!({"path":member,"size":size,"sha256":hash.finalize().iter().map(|byte|format!("{byte:02x}")).collect::<String>()}));
            }
        }
        let mut summary = self.client_cycle(cycle)?.unwrap_or_else(|| CycleSummary {
            cycle_id: cycle.into(),
            ..Default::default()
        });
        summary.sealed = true;
        summary.phase = CyclePhase::Sealed as i32;
        // Retained Output References Are Derived From This Archive, Never Its Source.
        summary.report_file = None;
        summary.evidence_file = None;
        content.notes.sort_by(|left, right| left.note_id.cmp(&right.note_id));
        content.artifacts.sort();
        let metadata = CycleArchive {
            cycle: Some(summary),
            notes: content.notes,
            artifacts: content.artifacts,
        };
        archive.start_file("evidence.pb", options)?;
        archive.write_all(&metadata.encode_to_vec())?;
        let files = write_files(&mut archive, Scope::Cycle(cycle).identifier()?.as_str(), content.files)?;
        let report = content.report.map(|file| write_file(&mut archive, "report.html", &file)).transpose()?;
        archive.start_file("manifest.json", options)?;
        serde_json::to_writer(
            &mut archive,
            &serde_json::json!({"version":1,"cycle":cycle,"executions":manifest,"files":files,"report":report}),
        )?;
        archive.finish()?;
        Ok(())
    }
    pub fn write_execution_archive(&self, execution: &str, output: &mut impl Write, mut content: ArchivedContent) -> Result<()> {
        content.files.sort_by_cached_key(|file| file.reference.encode_to_vec());
        content.artifacts.sort();
        content.execution_files.sort_by_cached_key(Message::encode_to_vec);
        let mut metadata = tempfile::tempfile()?;
        self.write_execution_metadata(execution, &mut metadata, &content)?;
        metadata.rewind()?;
        let options = archive_options();
        let mut archive = ZipWriter::new_stream(output);
        archive.start_file("evidence.pb", options)?;
        io::copy(&mut metadata, &mut archive)?;
        let connection = self.lock()?;
        let mut query =
            connection.prepare("SELECT id,stream,length,sha256 FROM telemetry_files WHERE execution=?1 AND finalized=1 ORDER BY stream")?;
        let mut manifest = Vec::new();
        for row in query.query_map([execution], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)? as u64, row.get::<_, Vec<u8>>(3)?))
        })? {
            let (id, stream, size, expected) = row?;
            let member = format!("telemetry/{stream}.arrow");
            crate::validate_execution_path(&member)?;
            let path = crate::telemetry::catalog::path(self.database.parent().unwrap(), &id)?;
            let mut file = File::open(path)?;
            if file.metadata()?.len() != size {
                return Err(Error::Data("Export Execution: Restore the Telemetry File Matching Its Catalog Size"));
            }
            archive.start_file(&member, options)?;
            let mut hash = Sha256::new();
            let mut buffer = [0u8; 64 * 1024];
            loop {
                let length = file.read(&mut buffer)?;
                if length == 0 {
                    break;
                }
                hash.update(&buffer[..length]);
                archive.write_all(&buffer[..length])?;
            }
            let digest = hash.finalize();
            if digest.as_slice() != expected {
                return Err(Error::Data("Export Execution: Restore the Telemetry File Matching Its Catalog Digest"));
            }
            let digest = digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
            manifest.push(serde_json::json!({"path":member,"size":size,"sha256":digest}));
        }
        let native = write_files(&mut archive, Scope::Execution(execution).identifier()?.as_str(), content.files)?;
        let report = content.report.map(|file| write_file(&mut archive, "report.html", &file)).transpose()?;
        archive.start_file("manifest.json", options)?;
        serde_json::to_writer(
            &mut archive,
            &serde_json::json!({"version":1,"execution":execution,"telemetry":manifest,"files":native,"report":report}),
        )?;
        archive.finish()?;
        Ok(())
    }
    fn write_execution_metadata(&self, execution: &str, output: &mut impl Write, content: &ArchivedContent) -> Result<()> {
        let connection = self.lock()?;
        let (state, payload, seal): (i32, Vec<u8>, Vec<u8>) =
            connection.query_row("SELECT state,payload,seal FROM executions WHERE id=?1", [execution], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?;
        if state != 2 || crate::store::digest(&connection, execution)? != seal {
            return Err(Error::Invalid("Download Evidence: Select a Valid Sealed Execution".into()));
        }
        write_field(output, 1, &payload)?;
        let mut query = connection.prepare("SELECT payload FROM evidence WHERE execution=?1 ORDER BY position")?;
        for row in query.query_map([execution], |row| row.get::<_, Vec<u8>>(0))? {
            write_field(output, 2, &row?)?;
        }
        write_field(output, 4, &seal)?;
        for artifact in &content.artifacts {
            write_field(output, 9, artifact)?;
        }
        for file in &content.execution_files {
            write_field(output, 10, &file.encode_to_vec())?;
        }
        for note in &content.notes {
            write_field(output, 11, &note.encode_to_vec())?;
        }
        let mut query = connection.prepare("SELECT payload FROM reports WHERE execution=?1 ORDER BY id")?;
        for row in query.query_map([execution], |row| row.get::<_, Vec<u8>>(0))? {
            write_field(output, 5, &row?)?;
        }
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn write_files<W: Write + Seek>(
    archive: &mut ZipWriter<W>, owner: &str, mut files: Vec<ArchivedNativeFile>,
) -> Result<Vec<serde_json::Value>> {
    files.sort_by_cached_key(|file| file.reference.encode_to_vec());
    let mut manifest = Vec::new();
    for file in files {
        let reference = &file.reference;
        let metadata = reference.metadata.as_ref().ok_or(Error::Data("Export Native File: Restore Its Metadata"))?;
        if metadata.scope.as_deref() != Some(owner) {
            return Err(Error::Data("Export Native File: Select the Owner's Own File Reference"));
        }
        let identity = Sha256::digest(reference.encode_to_vec()).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        let mut member = write_file(archive, &format!("files/{identity}"), &file)?;
        let reference_member = format!("files/{identity}.pb");
        archive.start_file(&reference_member, archive_options())?;
        archive.write_all(&reference.encode_to_vec())?;
        member["reference"] = reference_member.into();
        manifest.push(member);
    }
    Ok(manifest)
}

fn write_file<W: Write + Seek>(archive: &mut ZipWriter<W>, member: &str, file: &ArchivedNativeFile) -> Result<serde_json::Value> {
    let mut input = File::open(&file.path)?;
    archive.start_file(member, archive_options())?;
    let mut hash = Sha256::new();
    let mut size = 0u64;
    let mut buffer = [0u8; CONTENT_CHUNK_BYTES];
    loop {
        let length = input.read(&mut buffer)?;
        if length == 0 {
            break;
        }
        size += length as u64;
        hash.update(&buffer[..length]);
        archive.write_all(&buffer[..length])?;
    }
    let digest = hash.finalize();
    if size != file.reference.length || digest.as_slice() != file.reference.sha256 {
        return Err(Error::Data("Export Native File: Restore Bytes Matching Its Retained Reference"));
    }
    Ok(serde_json::json!({"path":member,"size":size,"sha256":digest.iter().map(|byte|format!("{byte:02x}")).collect::<String>()}))
}

fn archive_options() -> SimpleFileOptions {
    SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .last_modified_time(DateTime::default())
        .system(System::Unix)
        .unix_permissions(0o644)
}

fn write_field(output: &mut impl Write, tag: u32, bytes: &[u8]) -> Result<()> {
    let mut prefix = Vec::new();
    prost::encoding::encode_key(tag, prost::encoding::WireType::LengthDelimited, &mut prefix);
    prost::encoding::encode_varint(bytes.len() as u64, &mut prefix);
    output.write_all(&prefix)?;
    output.write_all(bytes)?;
    Ok(())
}

/* === Tests === */
