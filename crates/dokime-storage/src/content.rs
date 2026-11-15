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
use std::num::NonZeroUsize;

// Third Party Imports
use prost::Message;
use rusqlite::Connection;
use rusqlite::{OptionalExtension, params};
use sha2::{Digest, Sha256};

// Local Imports
use crate::{Error, Result, Store, store::next_position};
use dokime_protocol::host::content::ArtifactsCreateRequest;
use dokime_protocol::host::content::ArtifactsCreateResponse;
use dokime_protocol::host::content::ExecutionFileMetadata;
use dokime_protocol::host::content::ExecutionFileState;
use dokime_protocol::model::ArtifactMetadata;
use dokime_protocol::model::ArtifactState;
use prost_types::Timestamp;
use std::io::Write;

/* === Constants === */

pub const CONTENT_CHUNK_BYTES: usize = 64 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct ArchivedNativeFile {
    pub reference: harmos_protocol::model::FileReference,
    pub path: tempfile::TempPath,
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
    pub file: Option<harmos_protocol::model::FileReference>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn attach_artifact(&self, operation: &str, request: ArtifactsCreateRequest, now_ns: i64) -> Result<ArtifactsCreateResponse> {
        use harmos_protocol::model::{History, Position, Receipt};
        if request.request_id.is_empty() {
            return Err(Error::Invalid("Create Artifact: Supply a Request ID".into()));
        }
        let scope = request.scope.as_ref().ok_or_else(|| Error::Invalid("Create Artifact: Select Its Execution Scope".into()))?;
        let execution =
            scope.execution_id.as_deref().ok_or_else(|| Error::Invalid("Create Artifact: Select Its Execution Scope".into()))?;
        let reference = request.file.as_ref().ok_or_else(|| Error::Invalid("Create Artifact: Select Published Content".into()))?;
        let file = reference.metadata.as_ref().ok_or_else(|| Error::Invalid("Create Artifact: Select a Named File Reference".into()))?;
        if file.scope.as_deref() != Some(execution)
            || reference.sha256.len() != 32
            || file.expected_length.is_some_and(|expected| expected != reference.length)
            || file.expected_sha256.as_ref().is_some_and(|expected| expected != &reference.sha256)
            || reference
                .position
                .as_ref()
                .and_then(|position| position.history.as_ref())
                .is_none_or(|history| history.store != self.identity.as_ref() || history.epoch.is_empty())
        {
            return Err(Error::Invalid("Create Artifact: Keep Its Verified Native File Reference Within the Execution Scope".into()));
        }
        let fingerprint = Sha256::digest(request.encode_to_vec()).to_vec();
        let identifier = Sha256::digest(format!("{operation}/{execution}/{}", request.request_id))
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let key = format!("artifact/{identifier}");
        let mut connection = self.lock()?;
        self.flush_samples_locked(&mut connection)?;
        let transaction = connection.transaction()?;
        let replay: Option<(Vec<u8>, i64)> = transaction
            .query_row(
                "SELECT fingerprint,position FROM resource_requests WHERE execution=?1 AND resource=?2 AND request=?3",
                params![execution, operation, request.request_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        let (artifact, sequence) = if let Some((saved, sequence)) = replay {
            if saved != fingerprint {
                return Err(Error::Conflict("Create Artifact: Use a New Request ID for Changed Content or Metadata".into()));
            }
            let payload: Vec<u8> = transaction.query_row("SELECT payload FROM administration WHERE id=?1", [&key], |row| row.get(0))?;
            let stored = StoredArtifact::decode(payload.as_slice())?;
            (stored.artifact.ok_or_else(|| Error::Invalid("Create Artifact: Restore Its Committed Metadata".into()))?, sequence as u64)
        } else {
            require_open(&transaction, execution)?;
            let artifact = ArtifactMetadata {
                artifact_id: identifier,
                upload_id: file.id.clone(),
                scope: Some(scope.clone()),
                filename: request.filename.clone(),
                content_type: Some(request.content_type.clone()),
                size_bytes: reference.length,
                sha256: reference.sha256.iter().map(|byte| format!("{byte:02x}")).collect(),
                state: ArtifactState::Complete as i32,
                created_at: Some(Timestamp {
                    seconds: now_ns.div_euclid(1_000_000_000),
                    nanos: now_ns.rem_euclid(1_000_000_000) as i32,
                }),
                ..Default::default()
            };
            let stored = StoredArtifact {
                request: Some(request.clone()),
                artifact: Some(artifact.clone()),
            };
            transaction.execute("INSERT INTO administration VALUES(?1,?2)", params![key, stored.encode_to_vec()])?;
            let sequence = next_position(&transaction, execution)?;
            append_reference(&transaction, execution, "artifact:metadata", artifact.encode_to_vec())?;
            transaction.execute(
                "INSERT INTO resource_requests VALUES(?1,?2,?3,?4,?5)",
                params![execution, operation, request.request_id, fingerprint, sequence as i64],
            )?;
            (artifact, sequence)
        };
        transaction.commit()?;
        self.changed.send_modify(|revision| *revision += 1);
        let position = Position {
            history: Some(History {
                store: self.identity.to_string(),
                epoch: execution.into(),
            }),
            sequence,
        };
        Ok(ArtifactsCreateResponse {
            artifact: Some(artifact),
            receipt: Some(Receipt {
                request_id: request.request_id,
                accepted_at: Some(position.clone()),
                durable_at: Some(position),
                completed: true,
            }),
        })
    }
    pub fn artifact(&self, identifier: &str) -> Result<StoredArtifact> {
        self.administration(&format!("artifact/{identifier}"))?
            .ok_or_else(|| Error::NotFound("Read Artifact: Select a Committed Artifact Identifier".into()))
    }
    pub fn has_incompatible_content(&self) -> Result<bool> {
        Ok(self.lock()?.query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name IN ('content_objects','content_chunks'))",
            [],
            |row| row.get(0),
        )?)
    }
    pub fn artifact_records_page(&self, after: &str, limit: usize) -> Result<Vec<StoredArtifact>> {
        let connection = self.lock()?;
        let mut statement =
            connection.prepare("SELECT payload FROM administration WHERE id LIKE 'artifact/%' AND id>?1 ORDER BY id LIMIT ?2")?;
        statement
            .query_map(params![format!("artifact/{after}"), limit as i64], |row| row.get::<_, Vec<u8>>(0))?
            .map(|row| Ok(StoredArtifact::decode(row?.as_slice())?))
            .collect()
    }
    pub fn attach_execution_file(
        &self, execution: &str, mut metadata: ExecutionFileMetadata, file: harmos_protocol::model::FileReference,
    ) -> Result<()> {
        crate::validate_execution_path(&metadata.relative_path)?;
        let scope = file.metadata.as_ref().and_then(|metadata| metadata.scope.as_deref());
        if scope != Some(execution) {
            return Err(Error::Invalid("Attach Execution File: Select a Native File in This Execution's Scope".into()));
        }
        metadata.size_bytes = file.length;
        metadata.state = ExecutionFileState::Finalized as i32;
        let key = format!("execution-file/{execution}/{}", metadata.relative_path);
        let stored = StoredExecutionFile {
            metadata: Some(metadata.clone()),
            file: Some(file),
        };
        let mut connection = self.lock()?;
        self.flush_samples_locked(&mut connection)?;
        let transaction = connection.transaction()?;
        require_open(&transaction, execution)?;
        transaction.execute(
            "INSERT INTO administration(id,payload) VALUES(?1,?2) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload",
            params![key, stored.encode_to_vec()],
        )?;
        append_reference(&transaction, execution, "file:metadata", metadata.encode_to_vec())?;
        transaction.commit()?;
        self.changed.send_modify(|revision| *revision += 1);
        Ok(())
    }
    pub fn execution_file(&self, execution: &str, path: &str) -> Result<StoredExecutionFile> {
        self.metadata(execution)?;
        crate::validate_execution_path(path)?;
        self.administration(&format!("execution-file/{execution}/{path}"))?
            .ok_or_else(|| Error::NotFound("Read Execution File: Select a Registered Logical Path".into()))
    }
    pub fn artifacts_page(&self, after: &str, limit: usize) -> Result<Vec<ArtifactMetadata>> {
        self.artifact_records_page(after, limit)?
            .into_iter()
            .map(|stored| stored.artifact.ok_or_else(|| Error::Invalid("Read Artifact: Restore Its Committed Metadata".into())))
            .collect()
    }
    pub fn files_page(&self, execution: &str, after: &str, limit: usize) -> Result<Vec<StoredExecutionFile>> {
        self.metadata(execution)?;
        let prefix = format!("execution-file/{execution}/");
        let connection = self.lock()?;
        let mut statement =
            connection.prepare("SELECT payload FROM administration WHERE substr(id,1,length(?1))=?1 AND id>?2 ORDER BY id LIMIT ?3")?;
        statement
            .query_map(params![prefix, format!("{prefix}{after}"), limit as i64], |row| row.get::<_, Vec<u8>>(0))?
            .map(|row| Ok(StoredExecutionFile::decode(row?.as_slice())?))
            .collect()
    }
    pub fn write_cycle_archive(&self, cycle: &str, page: NonZeroUsize, output: &mut impl Write) -> Result<()> {
        self.write_cycle_archive_with_files(cycle, page, output, &mut std::collections::HashMap::new())
    }
    pub fn write_cycle_archive_with_files(
        &self, cycle: &str, page: NonZeroUsize, output: &mut impl Write,
        files: &mut std::collections::HashMap<String, Vec<ArchivedNativeFile>>,
    ) -> Result<()> {
        use std::io::{Read, Seek};
        use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
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
                self.write_execution_archive_with_files(&id, &mut execution, files.remove(&id).unwrap_or_default())?;
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
        if manifest.is_empty() {
            return Err(Error::Data("select an existing cycle for export"));
        }
        archive.start_file("manifest.json", options)?;
        serde_json::to_writer(&mut archive, &serde_json::json!({"version":1,"cycle":cycle,"executions":manifest}))?;
        archive.finish()?;
        Ok(())
    }
    pub fn write_execution_archive(&self, execution: &str, output: &mut impl Write) -> Result<()> {
        self.write_execution_archive_with_files(execution, output, Vec::new())
    }
    pub fn write_execution_archive_with_files(
        &self, execution: &str, output: &mut impl Write, files: Vec<ArchivedNativeFile>,
    ) -> Result<()> {
        use std::io::{Read, Seek};
        use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};
        let mut metadata = tempfile::tempfile()?;
        self.write_execution_metadata(execution, &mut metadata)?;
        metadata.rewind()?;
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        let mut archive = ZipWriter::new_stream(output);
        archive.start_file("evidence.pb", options)?;
        std::io::copy(&mut metadata, &mut archive)?;
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
            let mut file = std::fs::File::open(path)?;
            if file.metadata()?.len() != size {
                return Err(Error::Data("restore the telemetry file matching its catalog size"));
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
                return Err(Error::Data("restore the telemetry file matching its catalog digest"));
            }
            let digest = digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
            manifest.push(serde_json::json!({"path":member,"size":size,"sha256":digest}));
        }
        let mut native = Vec::new();
        for file in files {
            let reference = file.reference;
            let metadata = reference.metadata.as_ref().ok_or(Error::Data("Export Native File: Restore Its Metadata"))?;
            if metadata.scope.as_deref() != Some(execution) {
                return Err(Error::Data("Export Native File: Select the Execution's Own File Reference"));
            }
            let identity = Sha256::digest(reference.encode_to_vec());
            let identity = identity.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
            let member = format!("files/{identity}");
            let mut input = std::fs::File::open(&file.path)?;
            archive.start_file(&member, options)?;
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
            if size != reference.length || digest.as_slice() != reference.sha256 {
                return Err(Error::Data("Export Native File: Restore Bytes Matching Its Retained Reference"));
            }
            let reference_member = format!("files/{identity}.pb");
            archive.start_file(&reference_member, options)?;
            archive.write_all(&reference.encode_to_vec())?;
            native.push(serde_json::json!({
                "path": member, "reference": reference_member, "size": size,
                "sha256": digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>()
            }));
        }
        archive.start_file("manifest.json", options)?;
        serde_json::to_writer(&mut archive, &serde_json::json!({"version":1,"execution":execution,"telemetry":manifest,"files":native}))?;
        archive.finish()?;
        Ok(())
    }
    fn write_execution_metadata(&self, execution: &str, output: &mut impl Write) -> Result<()> {
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
        let mut query = connection.prepare("SELECT payload FROM administration WHERE id LIKE 'artifact/%' ORDER BY id")?;
        for row in query.query_map([], |row| row.get::<_, Vec<u8>>(0))? {
            let payload = row?;
            let artifact = StoredArtifact::decode(payload.as_slice())?;
            if artifact.request.as_ref().and_then(|request| request.scope.as_ref()).and_then(|scope| scope.execution_id.as_deref())
                == Some(execution)
            {
                write_field(output, 9, &payload)?;
            }
        }
        let prefix = format!("execution-file/{execution}/");
        let mut query = connection.prepare("SELECT payload FROM administration WHERE substr(id,1,length(?1))=?1 ORDER BY id")?;
        for row in query.query_map([prefix], |row| row.get::<_, Vec<u8>>(0))? {
            write_field(output, 10, &row?)?;
        }
        let mut query = connection.prepare("SELECT payload FROM reports WHERE execution=?1 ORDER BY id")?;
        for row in query.query_map([execution], |row| row.get::<_, Vec<u8>>(0))? {
            write_field(output, 5, &row?)?;
        }
        Ok(())
    }
    pub fn content_fingerprint(&self, execution: Option<&str>) -> Result<Vec<u8>> {
        let prefix = execution.map_or_else(|| "artifact/".into(), |execution| format!("execution-file/{execution}/"));
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT id,payload FROM administration WHERE substr(id,1,length(?1))=?1 ORDER BY id")?;
        let mut hash = Sha256::new();
        for row in statement.query_map([prefix], |row| Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?)))? {
            let (id, payload) = row?;
            hash.update((id.len() as u64).to_le_bytes());
            hash.update(id.as_bytes());
            hash.update((payload.len() as u64).to_le_bytes());
            hash.update(payload);
        }
        Ok(hash.finalize().to_vec())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn write_field(output: &mut impl Write, tag: u32, bytes: &[u8]) -> Result<()> {
    let mut prefix = Vec::new();
    prost::encoding::encode_key(tag, prost::encoding::WireType::LengthDelimited, &mut prefix);
    prost::encoding::encode_varint(bytes.len() as u64, &mut prefix);
    output.write_all(&prefix)?;
    output.write_all(bytes)?;
    Ok(())
}

fn require_open(connection: &Connection, execution: &str) -> Result<()> {
    let state: i32 = connection.query_row("SELECT state FROM executions WHERE id=?1", [execution], |row| row.get(0))?;
    if state != 0 {
        return Err(Error::Invalid("Write Content: Select an Open Execution".into()));
    }
    Ok(())
}

fn append_reference(connection: &Connection, execution: &str, kind: &str, payload: Vec<u8>) -> Result<()> {
    let evidence = crate::Evidence {
        position: next_position(connection, execution)?,
        kind: kind.into(),
        payload,
        ..Default::default()
    };
    connection.execute(
        "INSERT INTO evidence(execution,position,payload,kind) VALUES(?1,?2,?3,?4)",
        params![execution, evidence.position as i64, evidence.encode_to_vec(), evidence.kind],
    )?;
    Ok(())
}

/* === Tests === */
