/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Portable Content Names

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use crate::{Error, Result, Store};
use dokime_protocol::client::content::ExecutionFileMetadata;
use dokime_protocol::client::{FilesDeleteRequest, FilesDeleteResponse, files::*};
use dokime_protocol::v1::ArtifactMetadata;
use harmos_protocol::{
    storage::{FileMetadata, FilePhase, FileProducer, FileProgress, FileReference, FileSession, StagedFile, StorageCursor, StoreHistory},
    v1::{History, Position, ReadCut, Receipt},
};
use prost::Message;
use rusqlite::{Connection, OptionalExtension, Transaction, params};
use sha2::{Digest, Sha256};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn create_file_upload(&self, request: UploadsCreateRequest, chunk_bytes: u64, message_bytes: u64) -> Result<UploadsCreateResponse> {
        self.file_mutation("Files.Uploads/Create", &request.request_id, &request, |tx, receipt| {
            let mut metadata = request.metadata.clone().ok_or_else(|| Error::Invalid("Create Upload: Supply File Metadata".into()))?;
            if metadata.id.is_empty() {
                metadata.id = format!("upload-{}", hex(&Sha256::digest(request.request_id.as_bytes())));
            }
            if let Some(scope) = metadata.scope.as_deref().filter(|scope| !scope.is_empty()) {
                require_open(tx, scope)?;
            }
            tx.execute(
                "INSERT INTO content_objects VALUES(?1,?2,?1,0,?3,0,'',1)",
                params![metadata.id, metadata.scope.as_deref().unwrap_or_default(), Vec::<u8>::new()],
            )?;
            tx.execute("INSERT INTO file_uploads VALUES(?1,?2,1)", params![metadata.id, metadata.encode_to_vec()])?;
            Ok(UploadsCreateResponse {
                production: Some(FileProducer {
                    session: Some(self.file_session(&metadata, 1)),
                    length: 0,
                }),
                receipt: Some(receipt),
                chunk_bytes,
                message_bytes,
            })
        })
    }
    pub fn file_upload(&self, file: &StagedFile) -> Result<(FileMetadata, FileProgress, u64)> {
        self.check_file_history(file.history.as_ref())?;
        let connection = self.lock()?;
        upload(&connection, &file.id)
    }
    pub fn file_reference(&self, id: &str) -> Result<FileReference> {
        let entry = self.content(id)?;
        let digest = (0..entry.sha256.len())
            .step_by(2)
            .map(|offset| {
                u8::from_str_radix(&entry.sha256[offset..offset + 2], 16)
                    .map_err(|_| Error::Invalid("Get File: Restore Its SHA-256".into()))
            })
            .collect::<Result<Vec<_>>>()?;
        let metadata = if let Some(metadata) = self.file_upload_metadata(id)? {
            metadata
        } else {
            FileMetadata {
                id: entry.identifier.clone(),
                scope: Some(entry.execution.clone()),
                media_type: if entry.artifact {
                    ArtifactMetadata::decode(entry.metadata.as_slice())?.content_type.unwrap_or_default()
                } else {
                    ExecutionFileMetadata::decode(entry.metadata.as_slice())?.content_type.unwrap_or_default()
                },
                expected_length: Some(entry.length),
                expected_sha256: Some(digest.clone()),
            }
        };
        let sequence = if entry.execution.is_empty() {
            self.file_read_cut()?.positions[0].sequence
        } else {
            self.metadata(&entry.execution)?.frontier
        };
        Ok(FileReference {
            metadata: Some(metadata),
            length: entry.length,
            sha256: digest,
            position: Some(StorageCursor {
                history: Some(StoreHistory {
                    store: self.identity.to_string(),
                    epoch: if entry.execution.is_empty() { "files".into() } else { entry.execution },
                }),
                sequence,
            }),
        })
    }
    pub fn file_upload_metadata(&self, id: &str) -> Result<Option<FileMetadata>> {
        let connection = self.lock()?;
        let bytes: Option<Vec<u8>> =
            connection.query_row("SELECT metadata FROM file_uploads WHERE id=?1", [id], |row| row.get(0)).optional()?;
        bytes.map(|bytes| FileMetadata::decode(bytes.as_slice()).map_err(Error::from)).transpose()
    }
    pub fn resume_file_upload(&self, request: UploadsResumeRequest, chunk_bytes: u64, message_bytes: u64) -> Result<UploadsResumeResponse> {
        self.file_mutation("Files.Uploads/Resume", &request.request_id, &request, |tx, receipt| {
            let session = request.session.as_ref().ok_or_else(|| Error::Invalid("Resume Upload: Supply Its Writer Session".into()))?;
            let (metadata, progress, generation) = self.require_file_writer(tx, session)?;
            let generation = generation
                .checked_add(1)
                .filter(|generation| *generation <= i64::MAX as u64)
                .ok_or_else(|| Error::Invalid("Resume Upload: Start a New Upload".into()))?;
            tx.execute("UPDATE file_uploads SET generation=?2 WHERE id=?1", params![session.id, generation as i64])?;
            Ok(UploadsResumeResponse {
                production: Some(FileProducer {
                    session: Some(self.file_session(&metadata, generation)),
                    length: progress.length,
                }),
                receipt: Some(receipt),
                chunk_bytes,
                message_bytes,
            })
        })
    }
    pub fn append_file_upload(&self, request: UploadsAppendRequest, chunk_bytes: usize) -> Result<UploadsAppendResponse> {
        self.file_mutation("Files.Uploads/Append", &request.request_id, &request, |tx, receipt| {
            let session = request.session.as_ref().ok_or_else(|| Error::Invalid("Append Upload: Supply Its Writer Session".into()))?;
            let (_, progress, _) = self.require_file_writer(tx, session)?;
            if request.offset != progress.length || request.data.is_empty() || request.data.len() > chunk_bytes {
                return Err(Error::Invalid("Append Upload: Send One Bounded Chunk at the Accepted Offset".into()));
            }
            let length = progress
                .length
                .checked_add(request.data.len() as u64)
                .filter(|length| *length <= i64::MAX as u64)
                .ok_or_else(|| Error::Invalid("Append Upload: Keep Its Length Within the File Bound".into()))?;
            // Existing Content Chunks Retain Their Store-Owned Bound.
            for (index, bytes) in request.data.chunks(crate::CONTENT_CHUNK_BYTES).enumerate() {
                let offset = request.offset + (index * crate::CONTENT_CHUNK_BYTES) as u64;
                tx.execute("INSERT INTO content_chunks VALUES(?1,?2,?3)", params![session.id, offset as i64, bytes])?;
            }
            tx.execute("UPDATE content_objects SET length=?2 WHERE id=?1", params![session.id, length as i64])?;
            Ok(UploadsAppendResponse {
                production: Some(FileProducer { session: Some(session.clone()), length }),
                receipt: Some(receipt),
            })
        })
    }
    pub fn publish_file_upload(&self, request: UploadsPublishRequest) -> Result<UploadsPublishResponse> {
        self.file_mutation("Files.Uploads/Publish", &request.request_id, &request, |tx, receipt| {
            let session = request.session.as_ref().ok_or_else(|| Error::Invalid("Publish Upload: Supply Its Writer Session".into()))?;
            let (metadata, progress, _) = self.require_file_writer(tx, session)?;
            let mut digest = Sha256::new();
            let mut statement = tx.prepare("SELECT payload FROM content_chunks WHERE object=?1 ORDER BY offset")?;
            for bytes in statement.query_map([&session.id], |row| row.get::<_, Vec<u8>>(0))? {
                digest.update(bytes?);
            }
            let digest = digest.finalize().to_vec();
            if request.length != progress.length
                || request.sha256 != digest
                || metadata.expected_length.is_some_and(|length| length != request.length)
                || metadata.expected_sha256.as_ref().is_some_and(|expected| expected != &digest)
            {
                return Err(Error::Invalid("Publish Upload: Match Its Length and SHA-256".into()));
            }
            tx.execute("UPDATE content_objects SET phase=2,sha256=?2 WHERE id=?1", params![session.id, hex(&digest)])?;
            let position = receipt.durable_at.as_ref().expect("file mutation has a durable position");
            let file = FileReference {
                metadata: Some(metadata.clone()),
                length: request.length,
                sha256: digest,
                position: Some(StorageCursor {
                    history: Some(StoreHistory {
                        store: self.identity.to_string(),
                        epoch: metadata.scope.clone().filter(|scope| !scope.is_empty()).unwrap_or_else(|| "files".into()),
                    }),
                    sequence: position.sequence,
                }),
            };
            Ok(UploadsPublishResponse { file: Some(file), receipt: Some(receipt) })
        })
    }
    pub fn delete_file_upload(&self, request: UploadsDeleteRequest) -> Result<UploadsDeleteResponse> {
        self.file_mutation("Files.Uploads/Delete", &request.request_id, &request, |tx, receipt| {
            let session = request.session.as_ref().ok_or_else(|| Error::Invalid("Delete Upload: Supply Its Writer Session".into()))?;
            let (_, mut progress, _) = self.require_file_writer(tx, session)?;
            tx.execute("DELETE FROM content_chunks WHERE object=?1", [&session.id])?;
            tx.execute("UPDATE content_objects SET phase=3 WHERE id=?1", [&session.id])?;
            progress.phase = FilePhase::Abandoned as i32;
            Ok(UploadsDeleteResponse {
                production: Some(progress),
                receipt: Some(receipt),
            })
        })
    }
    pub fn delete_file(&self, request: FilesDeleteRequest) -> Result<FilesDeleteResponse> {
        self.file_mutation("Files/Delete", &request.request_id, &request, |tx, receipt| {
            let file = request.file.as_ref().ok_or_else(|| Error::Invalid("Delete File: Supply Its Reference".into()))?;
            let id = &file.metadata.as_ref().ok_or_else(|| Error::Invalid("Delete File: Supply Its Identity".into()))?.id;
            let (metadata, progress, _) = upload(tx, id)?;
            if file.position.as_ref().and_then(|position| position.history.as_ref()).is_none_or(|history| {
                history.store != self.identity.as_ref()
                    || history.epoch != metadata.scope.as_deref().filter(|scope| !scope.is_empty()).unwrap_or("files")
            }) {
                return Err(Error::Invalid("Delete File: Use Its Original Store Reference".into()));
            }
            if progress.phase != FilePhase::Finished as i32 {
                return Err(Error::Invalid("Delete File: Select Published Content".into()));
            }
            if let Some(scope) = metadata.scope.as_deref().filter(|scope| !scope.is_empty()) {
                require_open(tx, scope)?;
            }
            tx.execute("DELETE FROM content_chunks WHERE object=?1", [id])?;
            tx.execute("DELETE FROM file_uploads WHERE id=?1", [id])?;
            tx.execute("DELETE FROM content_objects WHERE id=?1", [id])?;
            Ok(FilesDeleteResponse { receipt: Some(receipt) })
        })
    }
    pub fn file_read_cut(&self) -> Result<ReadCut> {
        let connection = self.lock()?;
        let sequence: u64 = connection
            .query_row("SELECT COALESCE(MAX(sequence),0) FROM file_requests", [], |row| row.get::<_, i64>(0).map(|value| value as u64))?;
        Ok(ReadCut {
            positions: vec![Position {
                history: Some(History {
                    store: self.identity.to_string(),
                    epoch: "files".into(),
                }),
                sequence,
            }],
            revisions: Vec::new(),
        })
    }
    fn file_session(&self, metadata: &FileMetadata, generation: u64) -> FileSession {
        FileSession {
            history: Some(StoreHistory {
                store: self.identity.to_string(),
                epoch: "files".into(),
            }),
            id: metadata.id.clone(),
            generation,
        }
    }
    fn check_file_history(&self, history: Option<&StoreHistory>) -> Result<()> {
        if history.is_none_or(|history| history.store != self.identity.as_ref() || history.epoch != "files") {
            return Err(Error::Invalid("Access File: Use Its Original Store History".into()));
        }
        Ok(())
    }
    fn require_file_writer(&self, connection: &Connection, session: &FileSession) -> Result<(FileMetadata, FileProgress, u64)> {
        self.check_file_history(session.history.as_ref())?;
        let (metadata, progress, generation) = upload(connection, &session.id)?;
        if generation != session.generation || progress.phase != FilePhase::Producing as i32 {
            return Err(Error::Conflict("Write Upload: Resume Its Current Writer Before Appending or Publishing".into()));
        }
        if let Some(scope) = metadata.scope.as_deref().filter(|scope| !scope.is_empty()) {
            require_open(connection, scope)?;
        }
        Ok((metadata, progress, generation))
    }
    fn file_mutation<Q: Message, R: Message + Default>(
        &self, operation: &str, request_id: &str, request: &Q, apply: impl FnOnce(&Transaction<'_>, Receipt) -> Result<R>,
    ) -> Result<R> {
        if request_id.is_empty() {
            return Err(Error::Invalid(format!("{operation}: Supply a Request ID")));
        }
        let fingerprint = Sha256::digest(request.encode_to_vec()).to_vec();
        let mut connection = self.lock()?;
        let tx = connection.transaction()?;
        let saved: Option<(Vec<u8>, Vec<u8>)> = tx
            .query_row(
                "SELECT fingerprint,response FROM file_requests WHERE operation=?1 AND request=?2",
                params![operation, request_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?;
        if let Some((expected, response)) = saved {
            if expected != fingerprint {
                return Err(Error::Conflict(format!("{operation}: Retain the Original Payload or Use a New Request ID")));
            }
            return Ok(R::decode(response.as_slice())?);
        }
        tx.execute(
            "INSERT INTO file_requests(operation,request,fingerprint,response) VALUES(?1,?2,?3,?4)",
            params![operation, request_id, fingerprint, Vec::<u8>::new()],
        )?;
        let sequence = tx.last_insert_rowid() as u64;
        let position = Position {
            history: Some(History {
                store: self.identity.to_string(),
                epoch: "files".into(),
            }),
            sequence,
        };
        let receipt = Receipt {
            request_id: request_id.into(),
            accepted_at: Some(position.clone()),
            durable_at: Some(position),
            completed: true,
        };
        let response = apply(&tx, receipt)?;
        tx.execute("UPDATE file_requests SET response=?2 WHERE sequence=?1", params![sequence as i64, response.encode_to_vec()])?;
        tx.commit()?;
        self.changed.send_modify(|revision| *revision += 1);
        Ok(response)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn upload(connection: &Connection, id: &str) -> Result<(FileMetadata, FileProgress, u64)> {
    let (metadata, generation, length, phase): (Vec<u8>, u64, u64, i32) = connection
        .query_row(
            "SELECT u.metadata,u.generation,c.length,c.phase FROM file_uploads u JOIN content_objects c USING(id) WHERE u.id=?1",
            [id],
            |row| Ok((row.get(0)?, row.get::<_, i64>(1)? as u64, row.get::<_, i64>(2)? as u64, row.get(3)?)),
        )
        .optional()?
        .ok_or_else(|| Error::NotFound("Get Upload: Select an Existing Upload".into()))?;
    let phase = match phase {
        1 => FilePhase::Producing,
        2 => FilePhase::Finished,
        _ => FilePhase::Abandoned,
    };
    Ok((FileMetadata::decode(metadata.as_slice())?, FileProgress { length, phase: phase as i32 }, generation))
}
fn require_open(connection: &Connection, execution: &str) -> Result<()> {
    let state: i32 = connection.query_row("SELECT state FROM executions WHERE id=?1", [execution], |row| row.get(0))?;
    if state != 0 {
        return Err(Error::Invalid("Write File: Select an Open Execution".into()));
    }
    Ok(())
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/* === Tests === */
