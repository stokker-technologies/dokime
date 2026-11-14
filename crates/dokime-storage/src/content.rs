/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Durable Chunked Content

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use prost::Message;
use rusqlite::{OptionalExtension, params};
use sha2::{Digest, Sha256};

// Local Imports
use crate::{Error, Result, Store, store::next_position};
use dokime_protocol::client::content::ArtifactsCreateRequest;
use dokime_protocol::client::content::ArtifactsCreateResponse;
use dokime_protocol::client::content::ExecutionFileMetadata;
use dokime_protocol::client::content::ExecutionFileState;
use dokime_protocol::v1::{ArtifactMetadata, ArtifactState};
use prost_types::Timestamp;
use std::io::Write;

/* === Constants === */

pub const CONTENT_CHUNK_BYTES: usize = 64 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct ContentEntry {
    pub identifier: String,
    pub execution: String,
    pub path: String,
    pub metadata: Vec<u8>,
    pub length: u64,
    pub sha256: String,
    pub phase: ExecutionFileState,
    pub artifact: bool,
}

#[derive(Clone, PartialEq, Message)]
pub struct ArchivedContent {
    #[prost(string, tag = "1")]
    pub identifier: String,

    #[prost(string, tag = "2")]
    pub path: String,

    #[prost(bool, tag = "3")]
    pub artifact: bool,

    #[prost(bytes = "vec", tag = "4")]
    pub metadata: Vec<u8>,

    #[prost(uint64, tag = "5")]
    pub length: u64,

    #[prost(string, tag = "6")]
    pub sha256: String,

    #[prost(int32, tag = "7")]
    pub phase: i32,
}

#[derive(Clone, PartialEq, Message)]
pub struct ArchivedContentChunk {
    #[prost(string, tag = "1")]
    pub identifier: String,

    #[prost(uint64, tag = "2")]
    pub offset: u64,

    #[prost(bytes = "vec", tag = "3")]
    pub payload: Vec<u8>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn attach_artifact(&self, operation: &str, request: ArtifactsCreateRequest, now_ns: i64) -> Result<ArtifactsCreateResponse> {
        use harmos_protocol::v1::{History, Position, Receipt};
        if request.request_id.is_empty() {
            return Err(Error::Invalid("Create Artifact: Supply a Request ID".into()));
        }
        let scope = request.scope.as_ref().ok_or_else(|| Error::Invalid("Create Artifact: Select Its Execution Scope".into()))?;
        let execution =
            scope.execution_id.as_deref().ok_or_else(|| Error::Invalid("Create Artifact: Select Its Execution Scope".into()))?;
        let reference = request.file.as_ref().ok_or_else(|| Error::Invalid("Create Artifact: Select Published Content".into()))?;
        let source = reference.metadata.as_ref().ok_or_else(|| Error::Invalid("Create Artifact: Select a Named File Reference".into()))?;
        let fingerprint = Sha256::digest(request.encode_to_vec()).to_vec();
        let identifier = Sha256::digest(format!("{operation}/{execution}/{}", request.request_id))
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        let mut connection = self.lock()?;
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
            let payload: Vec<u8> =
                transaction.query_row("SELECT metadata FROM content_objects WHERE id=?1", [&identifier], |row| row.get(0))?;
            (ArtifactMetadata::decode(payload.as_slice())?, sequence as u64)
        } else {
            require_open(&transaction, execution)?;
            let (owner, length, sha256, phase): (String, i64, String, i32) =
                transaction.query_row("SELECT execution,length,sha256,phase FROM content_objects WHERE id=?1", [&source.id], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                })?;
            let digest = (0..sha256.len())
                .step_by(2)
                .map(|index| {
                    u8::from_str_radix(&sha256[index..index + 2], 16)
                        .map_err(|_| Error::Invalid("Create Artifact: Restore Its Published Digest".into()))
                })
                .collect::<Result<Vec<_>>>()?;
            if owner != execution
                || phase != 2
                || reference.length != length as u64
                || reference.sha256 != digest
                || source.expected_length.is_some_and(|expected| expected != length as u64)
                || source.expected_sha256.as_ref().is_some_and(|expected| expected != &digest)
                || reference
                    .position
                    .as_ref()
                    .and_then(|position| position.history.as_ref())
                    .is_none_or(|history| history.store != self.identity.as_ref() || history.epoch != execution)
            {
                return Err(Error::Invalid(
                    "Create Artifact: Keep Its Original Published File Reference Within the Execution Scope".into(),
                ));
            }
            let artifact = ArtifactMetadata {
                artifact_id: identifier.clone(),
                scope: Some(scope.clone()),
                filename: request.filename.clone(),
                content_type: Some(request.content_type.clone()),
                size_bytes: length as u64,
                sha256: sha256.clone(),
                state: ArtifactState::Complete as i32,
                created_at: Some(Timestamp {
                    seconds: now_ns.div_euclid(1_000_000_000),
                    nanos: now_ns.rem_euclid(1_000_000_000) as i32,
                }),
                ..Default::default()
            };
            transaction.execute(
                "INSERT INTO content_objects VALUES(?1,?2,?3,1,?4,?5,?6,2)",
                params![identifier, execution, request.filename, artifact.encode_to_vec(), length, sha256],
            )?;
            transaction.execute(
                "INSERT INTO content_chunks SELECT ?1,offset,payload FROM content_chunks WHERE object=?2",
                params![identifier, source.id],
            )?;
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
    pub fn begin_artifact(&self, metadata: ArtifactMetadata) -> Result<()> {
        let execution = metadata
            .scope
            .as_ref()
            .and_then(|scope| scope.execution_id.as_deref())
            .ok_or_else(|| Error::Invalid("Upload Artifact: Select Its Execution".into()))?;
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        require_open(&transaction, execution)?;
        transaction.execute(
            "INSERT INTO content_objects(id,execution,path,artifact,metadata,length,sha256,phase) VALUES(?1,?2,?3,1,?4,0,'',0)",
            params![metadata.artifact_id, execution, metadata.filename, metadata.encode_to_vec()],
        )?;
        transaction.commit()?;
        Ok(())
    }
    pub fn abort_artifact(&self, identifier: &str) -> Result<()> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        transaction.execute(
            "DELETE FROM content_chunks WHERE object=?1 AND EXISTS(SELECT 1 FROM content_objects WHERE id=?1 AND artifact=1 AND phase=0)",
            [identifier],
        )?;
        transaction.execute("DELETE FROM content_objects WHERE id=?1 AND artifact=1 AND phase=0", [identifier])?;
        transaction.commit()?;
        Ok(())
    }
    pub fn append_content(&self, identifier: &str, offset: u64, bytes: &[u8]) -> Result<()> {
        if bytes.is_empty() || bytes.len() > CONTENT_CHUNK_BYTES || offset > i64::MAX as u64 - bytes.len() as u64 {
            return Err(Error::Invalid("Write Content: Send Nonempty Chunks of at Most 64 KiB Within the File Bound".into()));
        }
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let (execution, length, phase): (String, u64, i32) =
            transaction.query_row("SELECT execution,length,phase FROM content_objects WHERE id=?1", [identifier], |row| {
                Ok((row.get(0)?, row.get::<_, i64>(1)? as u64, row.get(2)?))
            })?;
        require_open(&transaction, &execution)?;
        if phase > 1 || offset != length {
            return Err(Error::Invalid("Write Content: Append at the Next Offset of an Open Production".into()));
        }
        transaction.execute("INSERT INTO content_chunks VALUES(?1,?2,?3)", params![identifier, offset as i64, bytes])?;
        transaction
            .execute("UPDATE content_objects SET length=?2 WHERE id=?1", params![identifier, (offset + bytes.len() as u64) as i64])?;
        transaction.commit()?;
        self.changed.send_modify(|revision| *revision += 1);
        Ok(())
    }
    pub fn finish_artifact(&self, identifier: &str, expected_size: u64, expected_sha256: &str) -> Result<ArtifactMetadata> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let (execution, length, metadata, phase): (String, u64, Vec<u8>, i32) = transaction.query_row(
            "SELECT execution,length,metadata,phase FROM content_objects WHERE id=?1 AND artifact=1",
            [identifier],
            |row| Ok((row.get(0)?, row.get::<_, i64>(1)? as u64, row.get(2)?, row.get(3)?)),
        )?;
        require_open(&transaction, &execution)?;
        let digest = content_digest(&transaction, identifier)?;
        let mut metadata = ArtifactMetadata::decode(metadata.as_slice())?;
        if phase != 0
            || length != expected_size
            || length != metadata.size_bytes
            || digest != expected_sha256
            || !metadata.sha256.is_empty() && metadata.sha256 != digest
        {
            return Err(Error::Invalid("Complete Artifact: Match the Declared Length and SHA-256 Before Committing".into()));
        }
        metadata.sha256 = digest.clone();
        metadata.state = ArtifactState::Complete as i32;
        transaction.execute(
            "UPDATE content_objects SET phase=2,sha256=?2,metadata=?3 WHERE id=?1",
            params![identifier, digest, metadata.encode_to_vec()],
        )?;
        append_reference(&transaction, &execution, "artifact", metadata.encode_to_vec())?;
        transaction.commit()?;
        self.changed.send_modify(|revision| *revision += 1);
        Ok(metadata)
    }
    pub fn register_execution_file(&self, identifier: &str, execution: &str, metadata: ExecutionFileMetadata) -> Result<()> {
        crate::validate_execution_path(&metadata.relative_path)?;
        if metadata.size_bytes != 0 || metadata.state != ExecutionFileState::Streaming as i32 {
            return Err(Error::Invalid("Register Execution File: Start an Empty Streaming Production".into()));
        }
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        require_open(&transaction, execution)?;
        transaction.execute(
            "INSERT INTO content_objects(id,execution,path,artifact,metadata,length,sha256,phase) VALUES(?1,?2,?3,0,?4,0,'',1)",
            params![identifier, execution, metadata.relative_path, metadata.encode_to_vec()],
        )?;
        transaction.commit()?;
        self.changed.send_modify(|revision| *revision += 1);
        Ok(())
    }
    pub fn finish_execution_file(&self, identifier: &str, abandoned: bool) -> Result<()> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let (execution, phase): (String, i32) =
            transaction.query_row("SELECT execution,phase FROM content_objects WHERE id=?1 AND artifact=0", [identifier], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })?;
        require_open(&transaction, &execution)?;
        if phase != 1 {
            return Err(Error::Invalid("Finish Execution File: Select Its Open Production".into()));
        }
        let digest = content_digest(&transaction, identifier)?;
        transaction.execute(
            "UPDATE content_objects SET phase=?2,sha256=?3 WHERE id=?1",
            params![identifier, if abandoned { 3 } else { 2 }, digest],
        )?;
        append_reference(&transaction, &execution, "execution-file", format!("{identifier}:{digest}").into_bytes())?;
        transaction.commit()?;
        self.changed.send_modify(|revision| *revision += 1);
        Ok(())
    }
    pub fn content(&self, identifier: &str) -> Result<ContentEntry> {
        let connection = self.lock()?;
        connection
            .query_row(
                "SELECT id,execution,path,metadata,length,sha256,phase,artifact FROM content_objects WHERE id=?1 AND phase>0",
                [identifier],
                entry,
            )
            .optional()?
            .ok_or_else(|| {
                Error::NotFound("Download Content: Select a Completed Artifact or Materialize the Temporary Report Again".into())
            })
    }
    pub fn execution_file(&self, execution: &str, path: &str) -> Result<ContentEntry> {
        crate::validate_execution_path(path)?;
        let connection = self.lock()?;
        connection.query_row("SELECT id,execution,path,metadata,length,sha256,phase,artifact FROM content_objects WHERE execution=?1 AND path=?2 AND artifact=0 AND id NOT IN (SELECT id FROM file_uploads) ORDER BY rowid DESC LIMIT 1",params![execution,path],entry).optional()?.ok_or_else(||Error::NotFound("Read Execution File: Select a Registered File from This Execution's Listing".into()))
    }
    pub fn content_window(&self, identifier: &str, offset: u64, maximum: usize) -> Result<Vec<u8>> {
        if maximum == 0 || maximum > CONTENT_CHUNK_BYTES || offset > i64::MAX as u64 - maximum as u64 {
            return Err(Error::Invalid("Read Content: Request at Most 64 KiB Within the File Bound".into()));
        }
        let connection = self.lock()?;
        let mut statement = connection.prepare(
            "SELECT offset,payload FROM content_chunks WHERE object=?1 AND offset<?2 AND offset+length(payload)>?3 AND offset>=?4 ORDER BY offset",
        )?;
        let mut bytes = Vec::with_capacity(maximum);
        for row in statement.query_map(
            params![
                identifier,
                (offset + maximum as u64) as i64,
                offset as i64,
                offset.saturating_sub(CONTENT_CHUNK_BYTES as u64 - 1) as i64
            ],
            |row| Ok((row.get::<_, i64>(0)? as u64, row.get::<_, Vec<u8>>(1)?)),
        )? {
            let (start, payload) = row?;
            let from = offset.saturating_sub(start) as usize;
            let through = payload.len().min((offset + maximum as u64 - start) as usize);
            bytes.extend_from_slice(&payload[from..through]);
        }
        Ok(bytes)
    }
    pub fn artifacts_page(&self, after: &str, limit: usize) -> Result<Vec<ArtifactMetadata>> {
        let connection = self.lock()?;
        let mut statement =
            connection.prepare("SELECT metadata FROM content_objects WHERE artifact=1 AND phase=2 AND id>?1 ORDER BY id LIMIT ?2")?;
        statement
            .query_map(params![after, limit as i64], |row| row.get::<_, Vec<u8>>(0))?
            .map(|row| Ok(ArtifactMetadata::decode(row?.as_slice())?))
            .collect()
    }
    pub fn files_page(&self, execution: &str, after: &str, limit: usize) -> Result<Vec<ContentEntry>> {
        self.metadata(execution)?;
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT id,execution,path,metadata,length,sha256,phase,artifact FROM content_objects WHERE execution=?1 AND artifact=0 AND id NOT IN (SELECT id FROM file_uploads) AND path>?2 AND rowid IN(SELECT MAX(rowid) FROM content_objects WHERE execution=?1 AND artifact=0 AND id NOT IN (SELECT id FROM file_uploads) GROUP BY path) ORDER BY path LIMIT ?3")?;
        Ok(statement.query_map(params![execution, after, limit as i64], entry)?.collect::<std::result::Result<_, _>>()?)
    }
    pub fn write_cycle_archive(&self, cycle: &str, page: std::num::NonZeroUsize, output: &mut impl Write) -> Result<()> {
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
                self.write_execution_archive(&id, &mut execution)?;
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
        archive.start_file("manifest.json", options)?;
        serde_json::to_writer(&mut archive, &serde_json::json!({"version":1,"execution":execution,"telemetry":manifest}))?;
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
        let mut query = connection.prepare(
            "SELECT id,path,artifact,metadata,length,sha256,phase FROM content_objects WHERE execution=?1 AND phase>=2 AND id NOT IN (SELECT id FROM file_uploads) ORDER BY id",
        )?;
        for row in query.query_map([execution], |row| {
            Ok(ArchivedContent {
                identifier: row.get(0)?,
                path: row.get(1)?,
                artifact: row.get(2)?,
                metadata: row.get(3)?,
                length: row.get::<_, i64>(4)? as u64,
                sha256: row.get(5)?,
                phase: row.get(6)?,
            })
        })? {
            let content = row?;
            if !content.sha256.is_empty() && content_digest(&connection, &content.identifier)? != content.sha256 {
                return Err(Error::Invalid("Export Evidence: Restore Content Matching Its Frozen Digest".into()));
            }
            write_field(output, 7, &content.encode_to_vec())?;
            let mut chunks = connection.prepare("SELECT offset,payload FROM content_chunks WHERE object=?1 ORDER BY offset")?;
            for row in chunks.query_map([&content.identifier], |row| {
                Ok(ArchivedContentChunk {
                    identifier: content.identifier.clone(),
                    offset: row.get::<_, i64>(0)? as u64,
                    payload: row.get(1)?,
                })
            })? {
                write_field(output, 8, &row?.encode_to_vec())?;
            }
        }
        write_field(output, 4, &seal)?;
        let mut query = connection.prepare("SELECT payload FROM reports WHERE execution=?1 ORDER BY id")?;
        for row in query.query_map([execution], |row| row.get::<_, Vec<u8>>(0))? {
            write_field(output, 5, &row?)?;
        }
        Ok(())
    }
    pub fn content_fingerprint(&self, execution: Option<&str>) -> Result<Vec<u8>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT id,metadata,length,phase FROM content_objects WHERE (?1 IS NULL AND artifact=1 AND phase=2) OR (execution=?1 AND artifact=0 AND id NOT IN (SELECT id FROM file_uploads)) ORDER BY id")?;
        let mut hash = Sha256::new();
        for row in statement.query_map([execution], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?, row.get::<_, i64>(2)?, row.get::<_, i32>(3)?))
        })? {
            let (id, metadata, length, phase) = row?;
            hash.update((id.len() as u64).to_le_bytes());
            hash.update(id.as_bytes());
            hash.update((metadata.len() as u64).to_le_bytes());
            hash.update(metadata);
            hash.update(length.to_le_bytes());
            hash.update(phase.to_le_bytes());
        }
        Ok(hash.finalize().to_vec())
    }
}

impl ContentEntry {
    pub fn file_metadata(&self) -> Result<ExecutionFileMetadata> {
        if self.artifact {
            return Err(Error::Invalid("Read Execution File: Select a Registered File Production".into()));
        }
        let mut metadata = ExecutionFileMetadata::decode(self.metadata.as_slice())?;
        metadata.size_bytes = self.length;
        metadata.state = match self.phase {
            ExecutionFileState::Streaming => ExecutionFileState::Streaming,
            ExecutionFileState::Finalized => ExecutionFileState::Finalized,
            _ => ExecutionFileState::Interrupted,
        } as i32;
        Ok(metadata)
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

fn require_open(connection: &rusqlite::Connection, execution: &str) -> Result<()> {
    let state: i32 = connection.query_row("SELECT state FROM executions WHERE id=?1", [execution], |row| row.get(0))?;
    if state != 0 {
        return Err(Error::Invalid("Write Content: Select an Open Execution".into()));
    }
    Ok(())
}

fn content_digest(connection: &rusqlite::Connection, identifier: &str) -> Result<String> {
    let mut hash = Sha256::new();
    let mut statement = connection.prepare("SELECT payload FROM content_chunks WHERE object=?1 ORDER BY offset")?;
    for row in statement.query_map([identifier], |row| row.get::<_, Vec<u8>>(0))? {
        hash.update(row?);
    }
    Ok(hash.finalize().iter().map(|byte| format!("{byte:02x}")).collect())
}

fn append_reference(connection: &rusqlite::Connection, execution: &str, kind: &str, payload: Vec<u8>) -> Result<()> {
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

fn entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<ContentEntry> {
    let phase = match row.get::<_, i32>(6)? {
        1 => ExecutionFileState::Streaming,
        2 => ExecutionFileState::Finalized,
        _ => ExecutionFileState::Interrupted,
    };
    Ok(ContentEntry {
        identifier: row.get(0)?,
        execution: row.get(1)?,
        path: row.get(2)?,
        metadata: row.get(3)?,
        length: row.get::<_, i64>(4)? as u64,
        sha256: row.get(5)?,
        phase,
        artifact: row.get(7)?,
    })
}

/* === Tests === */
