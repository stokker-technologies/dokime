/* ***
 *
 * Dokime
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
use dokime_protocol::v1;

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
    pub phase: v1::ExecutionFilePhase,
    pub artifact: bool,
}

#[derive(Clone, PartialEq, Message)]
pub struct ArchivedContent {
    #[prost(string,tag="1")]
    pub identifier:String,
    #[prost(string,tag="2")]
    pub path:String,
    #[prost(bool,tag="3")]
    pub artifact:bool,
    #[prost(bytes="vec",tag="4")]
    pub metadata:Vec<u8>,
    #[prost(uint64,tag="5")]
    pub length:u64,
    #[prost(string,tag="6")]
    pub sha256:String,
    #[prost(int32,tag="7")]
    pub phase:i32,
}
#[derive(Clone, PartialEq, Message)]
pub struct ArchivedContentChunk {
    #[prost(string,tag="1")]
    pub identifier:String,
    #[prost(uint64,tag="2")]
    pub offset:u64,
    #[prost(bytes="vec",tag="3")]
    pub payload:Vec<u8>,
}
#[derive(Clone, PartialEq, Message)]
pub struct CycleArchive {
    #[prost(message,repeated,tag="1")]
    pub executions:Vec<crate::ExecutionArchive>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn publish_artifact_chunk(&self, execution: &str, request: v1::PublishArtifact, identifier: &str, now_ns: i64) -> Result<v1::ArtifactReceipt> {
        let mut connection=self.lock()?;
        let transaction=connection.transaction()?;
        require_open(&transaction,execution)?;
        let fingerprint=Sha256::digest(request.encode_to_vec()).to_vec();
        let replay: Option<(Vec<u8>,Vec<u8>)>=transaction.query_row("SELECT fingerprint,receipt FROM artifact_calls WHERE execution=?1 AND transfer=?2 AND sequence=?3",params![execution,request.transfer_id,request.sequence as i64],|row|Ok((row.get(0)?,row.get(1)?))).optional()?;
        if let Some((expected,receipt))=replay {
            if expected!=fingerprint {return Err(Error::Invalid("Publish Artifact: Retry the Same Chunk or Use a New Transfer Identity".into()));}
            return Ok(v1::ArtifactReceipt::decode(receipt.as_slice())?);
        }
        let transfer: Option<(String,i64,bool)>=transaction.query_row("SELECT object,sequence,terminal FROM artifact_transfers WHERE execution=?1 AND transfer=?2",params![execution,request.transfer_id],|row|Ok((row.get(0)?,row.get(1)?,row.get(2)?))).optional()?;
        let object=if let Some((object,sequence,terminal))=transfer {
            if terminal || request.sequence!=sequence as u64 {return Err(Error::Invalid("Publish Artifact: Send the Next Sequence Before Its Terminal Chunk".into()));}
            object
        } else {
            if request.sequence!=0 {return Err(Error::Invalid("Publish Artifact: Start Its Sequence at Zero".into()));}
            let payload:Vec<u8>=transaction.query_row("SELECT payload FROM executions WHERE id=?1",[execution],|row|row.get(0))?;
            let owner=crate::Execution::decode(payload.as_slice())?;
            let metadata=v1::ArtifactMetadata {artifact_id:identifier.into(),upload_id:request.transfer_id.clone(),filename:request.name.clone(),content_type:Some(request.mime.clone()),
                scope:Some(v1::ArtifactScope {cycle_id:Some(owner.cycle),execution_id:Some(execution.into()),..Default::default()}),created_at:Some(prost_types::Timestamp {seconds:now_ns.div_euclid(1_000_000_000),nanos:now_ns.rem_euclid(1_000_000_000) as i32}),..Default::default()};
            transaction.execute("INSERT INTO content_objects VALUES(?1,?2,?3,1,?4,0,'',0)",params![identifier,execution,request.name,metadata.encode_to_vec()])?;
            transaction.execute("INSERT INTO artifact_transfers VALUES(?1,?2,?3,0,0)",params![execution,request.transfer_id,identifier])?;
            identifier.into()
        };
        let (payload,length):(Vec<u8>,i64)=transaction.query_row("SELECT metadata,length FROM content_objects WHERE id=?1",[&object],|row|Ok((row.get(0)?,row.get(1)?)))?;
        let mut metadata=v1::ArtifactMetadata::decode(payload.as_slice())?;
        if metadata.filename!=request.name || metadata.content_type.as_deref()!=Some(&request.mime) {return Err(Error::Invalid("Publish Artifact: Keep Its Name and Media Type Unchanged".into()));}
        let length=if request.replace || request.aborted {transaction.execute("DELETE FROM content_chunks WHERE object=?1",[&object])?;0} else {length};
        let length=length.checked_add(request.data.len() as i64).ok_or_else(||Error::Invalid("Publish Artifact: Start a New Bounded Transfer".into()))?;
        if !request.data.is_empty() {transaction.execute("INSERT INTO content_chunks VALUES(?1,?2,?3)",params![object,length-request.data.len() as i64,request.data])?;}
        metadata.size_bytes=length as u64;
        let phase=if request.aborted {3} else if request.last {2} else {0};
        metadata.state=if request.aborted {v1::ArtifactState::Cancelled as i32} else if request.last {v1::ArtifactState::Complete as i32} else {v1::ArtifactState::Unspecified as i32};
        let digest=if request.last && !request.aborted {content_digest(&transaction,&object)?} else {String::new()};
        metadata.sha256=digest.clone();
        transaction.execute("UPDATE content_objects SET metadata=?2,length=?3,sha256=?4,phase=?5 WHERE id=?1",params![object,metadata.encode_to_vec(),length,digest,phase])?;
        if request.last {append_reference(&transaction,execution,"artifact",metadata.encode_to_vec())?;}
        transaction.execute("UPDATE artifact_transfers SET sequence=sequence+1,terminal=?3 WHERE execution=?1 AND transfer=?2",params![execution,request.transfer_id,request.last])?;
        let receipt=v1::ArtifactReceipt {artifact:object,size_bytes:length as u64,sealed:request.last && !request.aborted};
        transaction.execute("INSERT INTO artifact_calls VALUES(?1,?2,?3,?4,?5)",params![execution,request.transfer_id,request.sequence as i64,fingerprint,receipt.encode_to_vec()])?;
        transaction.commit()?;
        self.changed.send_modify(|revision|*revision+=1);
        Ok(receipt)
    }
    pub fn begin_artifact(&self, metadata: v1::ArtifactMetadata) -> Result<()> {
        let execution = metadata.scope.as_ref().and_then(|scope| scope.execution_id.as_deref())
            .ok_or_else(|| Error::Invalid("Upload Artifact: Select Its Execution".into()))?;
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        require_open(&transaction, execution)?;
        transaction.execute("INSERT INTO content_objects(id,execution,path,artifact,metadata,length,sha256,phase) VALUES(?1,?2,?3,1,?4,0,'',0)",
            params![metadata.artifact_id, execution, metadata.filename, metadata.encode_to_vec()])?;
        transaction.commit()?;
        Ok(())
    }
    pub fn abort_artifact(&self, identifier: &str) -> Result<()> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        transaction.execute("DELETE FROM content_chunks WHERE object=?1 AND EXISTS(SELECT 1 FROM content_objects WHERE id=?1 AND artifact=1 AND phase=0)", [identifier])?;
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
        let (execution, length, phase): (String, u64, i32) = transaction.query_row("SELECT execution,length,phase FROM content_objects WHERE id=?1", [identifier], |row| Ok((row.get(0)?,row.get::<_,i64>(1)? as u64,row.get(2)?)))?;
        require_open(&transaction, &execution)?;
        if phase > 1 || offset != length {
            return Err(Error::Invalid("Write Content: Append at the Next Offset of an Open Production".into()));
        }
        transaction.execute("INSERT INTO content_chunks VALUES(?1,?2,?3)", params![identifier, offset as i64, bytes])?;
        transaction.execute("UPDATE content_objects SET length=?2 WHERE id=?1", params![identifier, (offset + bytes.len() as u64) as i64])?;
        transaction.commit()?;
        self.changed.send_modify(|revision| *revision += 1);
        Ok(())
    }
    pub fn finish_artifact(&self, identifier: &str, expected_size: u64, expected_sha256: &str) -> Result<v1::ArtifactMetadata> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let (execution, length, metadata, phase): (String, u64, Vec<u8>, i32) = transaction.query_row("SELECT execution,length,metadata,phase FROM content_objects WHERE id=?1 AND artifact=1", [identifier], |row| Ok((row.get(0)?,row.get::<_,i64>(1)? as u64,row.get(2)?,row.get(3)?)))?;
        require_open(&transaction, &execution)?;
        let digest = content_digest(&transaction, identifier)?;
        let mut metadata = v1::ArtifactMetadata::decode(metadata.as_slice())?;
        if phase != 0 || length != expected_size || length != metadata.size_bytes || digest != expected_sha256
            || !metadata.sha256.is_empty() && metadata.sha256 != digest {
            return Err(Error::Invalid("Complete Artifact: Match the Declared Length and SHA-256 Before Committing".into()));
        }
        metadata.sha256 = digest.clone();
        metadata.state = v1::ArtifactState::Complete as i32;
        transaction.execute("UPDATE content_objects SET phase=2,sha256=?2,metadata=?3 WHERE id=?1", params![identifier,digest,metadata.encode_to_vec()])?;
        append_reference(&transaction, &execution, "artifact", metadata.encode_to_vec())?;
        transaction.commit()?;
        self.changed.send_modify(|revision| *revision += 1);
        Ok(metadata)
    }
    pub fn register_execution_file(&self, identifier: &str, execution: &str, metadata: v1::ExecutionFileMetadata) -> Result<()> {
        validate_path(&metadata.relative_path)?;
        if metadata.size_bytes != 0 || metadata.state != v1::ExecutionFileState::Streaming as i32 {
            return Err(Error::Invalid("Register Execution File: Start an Empty Streaming Production".into()));
        }
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        require_open(&transaction, execution)?;
        transaction.execute("INSERT INTO content_objects(id,execution,path,artifact,metadata,length,sha256,phase) VALUES(?1,?2,?3,0,?4,0,'',1)",
            params![identifier,execution,metadata.relative_path,metadata.encode_to_vec()])?;
        transaction.commit()?;
        self.changed.send_modify(|revision| *revision += 1);
        Ok(())
    }
    pub fn finish_execution_file(&self, identifier: &str, abandoned: bool) -> Result<()> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let (execution, phase): (String, i32) = transaction.query_row("SELECT execution,phase FROM content_objects WHERE id=?1 AND artifact=0", [identifier], |row| Ok((row.get(0)?,row.get(1)?)))?;
        require_open(&transaction, &execution)?;
        if phase != 1 { return Err(Error::Invalid("Finish Execution File: Select Its Open Production".into())); }
        let digest = content_digest(&transaction, identifier)?;
        transaction.execute("UPDATE content_objects SET phase=?2,sha256=?3 WHERE id=?1", params![identifier,if abandoned {3} else {2},digest])?;
        append_reference(&transaction, &execution, "execution-file", format!("{identifier}:{digest}").into_bytes())?;
        transaction.commit()?;
        self.changed.send_modify(|revision| *revision += 1);
        Ok(())
    }
    pub fn content(&self, identifier: &str) -> Result<ContentEntry> {
        let connection = self.lock()?;
        Ok(connection.query_row("SELECT id,execution,path,metadata,length,sha256,phase,artifact FROM content_objects WHERE id=?1 AND phase>0", [identifier], entry)?)
    }
    pub fn execution_file(&self, execution: &str, path: &str) -> Result<ContentEntry> {
        validate_path(path)?;
        let connection = self.lock()?;
        Ok(connection.query_row("SELECT id,execution,path,metadata,length,sha256,phase,artifact FROM content_objects WHERE execution=?1 AND path=?2 AND artifact=0 ORDER BY rowid DESC LIMIT 1", params![execution,path], entry)?)
    }
    pub fn content_window(&self, identifier: &str, offset: u64, maximum: usize) -> Result<Vec<u8>> {
        if maximum == 0 || maximum > CONTENT_CHUNK_BYTES || offset > i64::MAX as u64 - maximum as u64 {
            return Err(Error::Invalid("Read Content: Request at Most 64 KiB Within the File Bound".into()));
        }
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT offset,payload FROM content_chunks WHERE object=?1 AND offset<?2 AND offset+length(payload)>?3 ORDER BY offset")?;
        let mut bytes = Vec::with_capacity(maximum);
        for row in statement.query_map(params![identifier,(offset + maximum as u64) as i64, offset as i64], |row| Ok((row.get::<_,i64>(0)? as u64,row.get::<_,Vec<u8>>(1)?)))? {
            let (start, payload) = row?;
            let from = offset.saturating_sub(start) as usize;
            let through = payload.len().min((offset + maximum as u64 - start) as usize);
            bytes.extend_from_slice(&payload[from..through]);
        }
        Ok(bytes)
    }
    pub fn artifacts_page(&self, after: &str, limit: usize) -> Result<Vec<v1::ArtifactMetadata>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT metadata FROM content_objects WHERE artifact=1 AND phase=2 AND id>?1 ORDER BY id LIMIT ?2")?;
        statement.query_map(params![after,limit as i64], |row| row.get::<_,Vec<u8>>(0))?.map(|row| Ok(v1::ArtifactMetadata::decode(row?.as_slice())?)).collect()
    }
    pub fn files_page(&self, execution: &str, after: &str, limit: usize) -> Result<Vec<ContentEntry>> {
        self.metadata(execution)?;
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT id,execution,path,metadata,length,sha256,phase,artifact FROM content_objects WHERE execution=?1 AND artifact=0 AND path>?2 AND rowid IN(SELECT MAX(rowid) FROM content_objects WHERE execution=?1 AND artifact=0 GROUP BY path) ORDER BY path LIMIT ?3")?;
        Ok(statement.query_map(params![execution,after,limit as i64], entry)?.collect::<std::result::Result<_,_>>()?)
    }
    pub fn write_execution_archive(&self, execution: &str, output: &mut impl std::io::Write) -> Result<()> {
        let connection = self.lock()?;
        let (state,payload,seal): (i32,Vec<u8>,Vec<u8>) = connection.query_row("SELECT state,payload,seal FROM executions WHERE id=?1",[execution],|row| Ok((row.get(0)?,row.get(1)?,row.get(2)?)))?;
        if state != 2 || crate::store::digest(&connection,execution)? != seal {
            return Err(Error::Invalid("Download Evidence: Select a Valid Sealed Execution".into()));
        }
        write_field(output,1,&payload)?;
        let mut query = connection.prepare("SELECT payload FROM evidence WHERE execution=?1 ORDER BY position")?;
        for row in query.query_map([execution],|row| row.get::<_,Vec<u8>>(0))? { write_field(output,2,&row?)?; }
        let mut query = connection.prepare("SELECT stream,payload FROM samples WHERE execution=?1 ORDER BY position")?;
        for row in query.query_map([execution],|row| Ok((row.get::<_,String>(0)?,row.get::<_,Vec<u8>>(1)?)))? {
            let (stream,payload)=row?;
            let sample=crate::streams::decode(stream,payload)?;
            write_field(output,3,&sample.encode_to_vec())?;
        }
        let mut query=connection.prepare("SELECT id,path,artifact,metadata,length,sha256,phase FROM content_objects WHERE execution=?1 AND phase>=2 ORDER BY id")?;
        for row in query.query_map([execution],|row|Ok(ArchivedContent {identifier:row.get(0)?,path:row.get(1)?,artifact:row.get(2)?,metadata:row.get(3)?,length:row.get::<_,i64>(4)? as u64,sha256:row.get(5)?,phase:row.get(6)?}))? {
            let content=row?;
            if !content.sha256.is_empty() && content_digest(&connection,&content.identifier)?!=content.sha256 {return Err(Error::Invalid("Export Evidence: Restore Content Matching Its Frozen Digest".into()));}
            write_field(output,7,&content.encode_to_vec())?;
            let mut chunks=connection.prepare("SELECT offset,payload FROM content_chunks WHERE object=?1 ORDER BY offset")?;
            for row in chunks.query_map([&content.identifier],|row|Ok(ArchivedContentChunk {identifier:content.identifier.clone(),offset:row.get::<_,i64>(0)? as u64,payload:row.get(1)?}))? {write_field(output,8,&row?.encode_to_vec())?;}
        }
        write_field(output,4,&seal)?;
        let mut query = connection.prepare("SELECT payload FROM reports WHERE execution=?1 ORDER BY id")?;
        for row in query.query_map([execution],|row| row.get::<_,Vec<u8>>(0))? { write_field(output,5,&row?)?; }
        let mut query = connection.prepare("SELECT position,stream,payload FROM sample_batches WHERE execution=?1 ORDER BY position")?;
        for row in query.query_map([execution],|row| Ok(crate::SampleBatch { position:row.get::<_,i64>(0)? as u64,stream:row.get(1)?,payload:row.get(2)? }))? { write_field(output,6,&row?.encode_to_vec())?; }
        Ok(())
    }
    pub fn content_fingerprint(&self, execution: Option<&str>) -> Result<Vec<u8>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT id,metadata,length,phase FROM content_objects WHERE (?1 IS NULL AND artifact=1 AND phase=2) OR (execution=?1 AND artifact=0) ORDER BY id")?;
        let mut hash = Sha256::new();
        for row in statement.query_map([execution], |row| Ok((row.get::<_,String>(0)?,row.get::<_,Vec<u8>>(1)?,row.get::<_,i64>(2)?,row.get::<_,i32>(3)?)))? {
            let (id, metadata, length, phase) = row?;
            hash.update((id.len() as u64).to_le_bytes()); hash.update(id.as_bytes());
            hash.update((metadata.len() as u64).to_le_bytes()); hash.update(metadata);
            hash.update(length.to_le_bytes()); hash.update(phase.to_le_bytes());
        }
        Ok(hash.finalize().to_vec())
    }
}

impl ContentEntry {
    pub fn file_metadata(&self) -> Result<v1::ExecutionFileMetadata> {
        if self.artifact {return Err(Error::Invalid("Read Execution File: Select a Registered File Production".into()));}
        let mut metadata = v1::ExecutionFileMetadata::decode(self.metadata.as_slice())?;
        metadata.size_bytes = self.length;
        metadata.state = match self.phase { v1::ExecutionFilePhase::Producing => v1::ExecutionFileState::Streaming, v1::ExecutionFilePhase::Finished => v1::ExecutionFileState::Finalized, _ => v1::ExecutionFileState::Interrupted } as i32;
        Ok(metadata)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn write_field(output: &mut impl std::io::Write, tag: u32, bytes: &[u8]) -> Result<()> {
    let mut prefix=Vec::new();
    prost::encoding::encode_key(tag,prost::encoding::WireType::LengthDelimited,&mut prefix);
    prost::encoding::encode_varint(bytes.len() as u64,&mut prefix);
    output.write_all(&prefix)?;
    output.write_all(bytes)?;
    Ok(())
}
fn validate_path(path: &str) -> Result<()> {
    if path.is_empty() || path.contains('\\') || path.contains(':') || path.chars().any(char::is_control) || path.split('/').any(|part| part.is_empty() || part == "." || part == "..") {
        return Err(Error::Invalid("Select a Registered Relative File Name Without Parent or Absolute Paths".into()));
    }
    Ok(())
}
fn require_open(connection: &rusqlite::Connection, execution: &str) -> Result<()> {
    let state: i32 = connection.query_row("SELECT state FROM executions WHERE id=?1", [execution], |row| row.get(0))?;
    if state != 0 { return Err(Error::Invalid("Write Content: Select an Open Execution".into())); }
    Ok(())
}
fn content_digest(connection: &rusqlite::Connection, identifier: &str) -> Result<String> {
    let mut hash = Sha256::new();
    let mut statement = connection.prepare("SELECT payload FROM content_chunks WHERE object=?1 ORDER BY offset")?;
    for row in statement.query_map([identifier], |row| row.get::<_,Vec<u8>>(0))? { hash.update(row?); }
    Ok(hash.finalize().iter().map(|byte| format!("{byte:02x}")).collect())
}
fn append_reference(connection: &rusqlite::Connection, execution: &str, kind: &str, payload: Vec<u8>) -> Result<()> {
    let evidence = crate::Evidence { position: next_position(connection,execution)?,kind: kind.into(),payload,..Default::default() };
    connection.execute("INSERT INTO evidence VALUES(?1,?2,?3)",params![execution,evidence.position as i64,evidence.encode_to_vec()])?;
    Ok(())
}
fn entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<ContentEntry> {
    let phase = match row.get::<_,i32>(6)? { 1=>v1::ExecutionFilePhase::Producing,2=>v1::ExecutionFilePhase::Finished,_=>v1::ExecutionFilePhase::Abandoned };
    Ok(ContentEntry {identifier:row.get(0)?,execution:row.get(1)?,path:row.get(2)?,metadata:row.get(3)?,length:row.get::<_,i64>(4)? as u64,sha256:row.get(5)?,phase,artifact:row.get(7)?})
}

/* === Tests === */
