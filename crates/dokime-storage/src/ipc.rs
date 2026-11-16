/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Durable Arrow IPC Blocks and Metadata-Only Finalization

/* === Modules === */

mod projection;

/* === Re-Exports === */

pub(crate) use projection::Projection;

/* === Imports === */

// Standard Library Imports
use std::io::Result as IoResult;
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

// Third Party Imports
use arrow_array::RecordBatch;
use arrow_buffer::Buffer;
use arrow_ipc::{
    Block, MessageHeader, MetadataVersion,
    convert::IpcSchemaEncoder,
    writer::{DictionaryTracker, IpcDataGenerator, IpcWriteContext, IpcWriteOptions, write_message},
};
use arrow_schema::Schema;
use arrow_schema::SchemaRef;
use sha2::{
    Digest, Sha256,
    digest::common::hazmat::{SerializableState, SerializedState},
};

// Local Imports
use super::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct Writer {
    file: HashedFile,
    schema: SchemaRef,
    blocks: Vec<Block>,
    finished: bool,

    #[cfg(test)]
    fail_sync: bool,
}

struct HashedFile {
    file: File,
    hash: Sha256,
    length: u64,
}

/// One Record Batch Message, Encoded and Hashed Apart From Any File So It Can Be Built in Parallel.
///
/// Message bytes do not depend on where they land: the IPC prefix and padding are relative to the
/// message itself, so writing them at any offset yields the bytes a direct append would.
pub(crate) struct Encoded {
    bytes: Vec<u8>,
    metadata_bytes: u32,
    body_bytes: u64,
    rows: u64,
    sha256: [u8; 32],
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(crate) struct BatchIndex {
    pub offset: u64,
    pub metadata_bytes: u32,
    pub body_bytes: u64,
    pub rows: u64,
    pub sha256: [u8; 32],
}

#[derive(Clone)]
pub(crate) struct Checkpoint {
    pub end: u64,
    pub hash: Vec<u8>,
    pub batches: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Checkpoint {
    pub(crate) fn digest(&self) -> Result<[u8; 32]> {
        Ok(hash_state(&self.hash)?.finalize().into())
    }
}

impl Writer {
    // A File Without A Catalog Entry Has No Saved Rows. Only Its Valid Header Is Retained.
    pub(crate) fn recover_empty(path: &Path, schema: SchemaRef) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut magic = [0; 8];
        file.read_exact(&mut magic)?;
        if &magic != b"ARROW1\0\0" {
            return Err(Error::Corrupt("IPC file magic differs"));
        }
        let embedded = arrow_ipc::reader::StreamReader::try_new(&mut file, None)?;
        if embedded.schema() != schema {
            return Err(Error::Corrupt("uncataloged IPC schema differs"));
        }
        drop(embedded);
        let end = file.stream_position()?;
        file.rewind()?;
        let mut header = vec![0; usize::try_from(end).map_err(|_| Error::Corrupt("IPC header exceeds address space"))?];
        file.read_exact(&mut header)?;
        drop(file);
        let mut hash = Sha256::new();
        hash.update(&header);
        Self::resume(
            path,
            schema,
            &[],
            &Checkpoint {
                end,
                hash: hash.serialize().to_vec(),
                batches: 0,
            },
        )
    }

    pub(crate) fn resume(path: &Path, schema: SchemaRef, indices: &[BatchIndex], checkpoint: &Checkpoint) -> Result<Self> {
        let file = OpenOptions::new().write(true).open(path)?;
        if file.metadata()?.len() < checkpoint.end {
            return Err(Error::Corrupt("IPC committed prefix is missing"));
        }
        let mut writer = Self {
            file: HashedFile {
                file,
                hash: hash_state(&checkpoint.hash)?,
                length: checkpoint.end,
            },
            schema,
            blocks: indices.iter().map(BatchIndex::block).collect::<Result<Vec<_>>>()?,
            finished: false,
            #[cfg(test)]
            fail_sync: false,
        };
        writer.rollback(checkpoint)?;
        Ok(writer)
    }

    pub fn create(path: &Path, schema: SchemaRef) -> Result<Self> {
        let file = OpenOptions::new().create_new(true).write(true).open(path)?;
        let mut file = HashedFile { file, hash: Sha256::new(), length: 0 };
        let options = options()?;
        // IPC V5 Uses Eight-Byte Alignment And A Continuation Prefix.
        file.write_all(b"ARROW1\0\0")?;
        let message =
            IpcDataGenerator::default().schema_to_bytes_with_dictionary_tracker(&schema, &mut DictionaryTracker::new(true), &options);
        write_message(&mut file, message, &options)?;
        {
            let _span = tracing::debug_span!("ingest.ipc.sync").entered();
            file.file.sync_all()?;
        }
        sync_parent(path)?;
        Ok(Self {
            file,
            schema,
            blocks: Vec::new(),
            finished: false,
            #[cfg(test)]
            fail_sync: false,
        })
    }

    #[cfg(test)]
    pub(crate) fn fail_next_sync(&mut self) {
        self.fail_sync = true;
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            end: self.file.length,
            hash: self.file.hash.serialize().to_vec(),
            batches: self.blocks.len(),
        }
    }

    /// Encode, Write and Sync One Batch.
    pub fn append(&mut self, batch: &RecordBatch) -> Result<BatchIndex> {
        let encoded = encode(&self.schema, batch)?;
        let index = self.write(&encoded)?;
        self.sync()?;
        Ok(index)
    }

    /// Write One Encoded Batch After the Current End; It Is Durable Only After [`Writer::sync`].
    pub(crate) fn write(&mut self, encoded: &Encoded) -> Result<BatchIndex> {
        if self.finished {
            return Err(Error::Data("open a new Arrow writer; this one is finished"));
        }
        let index = BatchIndex {
            offset: self.file.length,
            metadata_bytes: encoded.metadata_bytes,
            body_bytes: encoded.body_bytes,
            rows: encoded.rows,
            sha256: encoded.sha256,
        };
        let block = index.block()?;
        self.file.write_all(&encoded.bytes)?;
        self.blocks.push(block);
        Ok(index)
    }

    /// Make Every Written Batch Durable.
    pub(crate) fn sync(&mut self) -> Result<()> {
        #[cfg(test)]
        if std::mem::take(&mut self.fail_sync) {
            return Err(Error::Io(std::io::Error::other("injected batch sync failure")));
        }
        let _span = tracing::debug_span!("ingest.ipc.sync").entered();
        self.file.file.sync_all()?;
        Ok(())
    }

    // Restore The Last Committed Prefix After A Failed Append Or Catalog Commit.
    pub fn rollback(&mut self, checkpoint: &Checkpoint) -> Result<()> {
        self.file.file.set_len(checkpoint.end)?;
        self.file.file.seek(SeekFrom::Start(checkpoint.end))?;
        self.file.hash = hash_state(&checkpoint.hash)?;
        self.file.length = checkpoint.end;
        self.blocks.truncate(checkpoint.batches);
        Ok(())
    }

    pub fn finish(&mut self) -> Result<(u64, [u8; 32])> {
        if !self.finished {
            let checkpoint = self.checkpoint();
            let result = footer(&mut self.file, &self.schema, &self.blocks);
            if let Err(error) = result {
                self.rollback(&checkpoint)?;
                return Err(error);
            }
            self.finished = true;
        }
        Ok((self.file.length, self.file.hash.clone().finalize().into()))
    }
}

impl BatchIndex {
    fn block(&self) -> Result<Block> {
        let offset = i64::try_from(self.offset).map_err(|_| Error::Corrupt("IPC offset exceeds range"))?;
        let metadata = i32::try_from(self.metadata_bytes).map_err(|_| Error::Corrupt("IPC metadata exceeds range"))?;
        let body = i64::try_from(self.body_bytes).map_err(|_| Error::Corrupt("IPC body exceeds range"))?;
        Ok(Block::new(offset, metadata, body))
    }

    pub fn end(&self) -> Result<u64> {
        self.offset
            .checked_add(u64::from(self.metadata_bytes))
            .and_then(|end| end.checked_add(self.body_bytes))
            .ok_or(Error::Corrupt("IPC block end overflows"))
    }
}

impl Write for HashedFile {
    fn write(&mut self, bytes: &[u8]) -> IoResult<usize> {
        let write_span = tracing::debug_span!("ingest.ipc.write").entered();
        let written = self.file.write(bytes)?;
        drop(write_span);
        let _hash = tracing::debug_span!("ingest.ipc.hash").entered();
        self.hash.update(&bytes[..written]);
        self.length += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> IoResult<()> {
        self.file.flush()
    }
}

/* === Public Functions === */

/// Encode One Batch of the Given Schema as a Compressed Record Batch Message.
pub(crate) fn encode(schema: &SchemaRef, batch: &RecordBatch) -> Result<Encoded> {
    if batch.schema() != *schema {
        return Err(Error::Data("encode batches with the Arrow writer's schema"));
    }
    let options = options()?;
    let (dictionaries, message) =
        IpcDataGenerator::default().encode(batch, &mut DictionaryTracker::new(true), &options, &mut IpcWriteContext::default())?;
    if !dictionaries.is_empty() {
        return Err(Error::Data("telemetry IPC does not use dictionary encoding"));
    }
    let mut bytes = Vec::new();
    let (metadata, body) = write_message(&mut bytes, message, &options)?;
    Ok(Encoded {
        metadata_bytes: u32::try_from(metadata).map_err(|_| Error::Data("IPC metadata length overflows"))?,
        body_bytes: body as u64,
        rows: batch.num_rows() as u64,
        sha256: Sha256::digest(&bytes).into(),
        bytes,
    })
}

#[cfg(test)]
pub(crate) fn read_batch(path: &Path, schema: SchemaRef, index: &BatchIndex) -> Result<RecordBatch> {
    let buffer = read_buffer(path, index)?;
    let decoder = arrow_ipc::reader::FileDecoder::new(schema, MetadataVersion::V5);
    let batch = decoder.read_record_batch(&index.block()?, &buffer)?.ok_or(Error::Corrupt("IPC block is not a record batch"))?;
    if batch.num_rows() as u64 != index.rows {
        return Err(Error::Corrupt("IPC batch row count differs"));
    }
    Ok(batch)
}

pub(crate) fn read_query_batch(path: &Path, index: &BatchIndex, projection: &Projection, maximum: usize) -> Result<RecordBatch> {
    let length = index.body_bytes.saturating_add(u64::from(index.metadata_bytes));
    if length > maximum as u64 {
        return Err(Error::Data("query batch exceeds working budget"));
    }
    let buffer = read_buffer(path, index)?;
    projection.check_budget(&buffer, index, maximum)?;
    projection.decode(&buffer, index)
}

// Inspect Only Committed Batch Headers. Serialized SHA State Avoids A Payload Hash Scan.
pub(crate) fn recover(path: &Path, schema: SchemaRef, indices: &[BatchIndex], checkpoint: &Checkpoint) -> Result<(u64, [u8; 32])> {
    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    let length = file.metadata()?.len();
    if checkpoint.end > length || checkpoint.batches != indices.len() {
        return Err(Error::Corrupt("IPC committed prefix is missing"));
    }
    let mut magic = [0; 8];
    file.read_exact(&mut magic)?;
    if &magic != b"ARROW1\0\0" {
        return Err(Error::Corrupt("IPC file magic differs"));
    }
    let embedded = arrow_ipc::reader::StreamReader::try_new(&mut file, None)?;
    if embedded.schema() != schema {
        return Err(Error::Corrupt("IPC header schema differs from catalog"));
    }
    drop(embedded);
    let mut prior = file.stream_position()?;
    for index in indices {
        if index.offset != prior || index.end()? > checkpoint.end || index.metadata_bytes < 8 {
            return Err(Error::Corrupt("IPC batch index is outside the committed prefix"));
        }
        file.seek(SeekFrom::Start(index.offset))?;
        let mut prefix = [0; 8];
        file.read_exact(&mut prefix)?;
        if prefix[..4] != [255; 4] || u32::from_le_bytes(prefix[4..].try_into().unwrap()) != index.metadata_bytes - 8 {
            return Err(Error::Corrupt("IPC batch metadata length differs"));
        }
        let mut header = vec![0; index.metadata_bytes as usize - 8];
        file.read_exact(&mut header)?;
        let message = arrow_ipc::root_as_message(&header).map_err(|_| Error::Corrupt("IPC batch header is invalid"))?;
        if message.header_type() != MessageHeader::RecordBatch
            || message.bodyLength() != index.body_bytes as i64
            || message.header_as_record_batch().is_none_or(|batch| batch.length() != index.rows as i64)
        {
            return Err(Error::Corrupt("IPC batch header differs from catalog"));
        }
        prior = index.end()?;
    }
    if prior != checkpoint.end {
        return Err(Error::Corrupt("IPC committed prefix differs from batch index"));
    }
    // Only An Uncommitted Suffix Can Be Removed; Saved Batches Are Never Rewritten.
    file.set_len(checkpoint.end)?;
    file.seek(SeekFrom::Start(checkpoint.end))?;
    let mut file = HashedFile {
        file,
        hash: hash_state(&checkpoint.hash)?,
        length: checkpoint.end,
    };
    let blocks = indices.iter().map(BatchIndex::block).collect::<Result<Vec<_>>>()?;
    footer(&mut file, &schema, &blocks)?;
    Ok((file.length, file.hash.finalize().into()))
}

/* === Private Functions === */

fn read_buffer(path: &Path, index: &BatchIndex) -> Result<Buffer> {
    let mut file = File::open(path)?;
    file.seek(SeekFrom::Start(index.offset))?;
    let length = usize::try_from(index.end()? - index.offset).map_err(|_| Error::Corrupt("IPC block exceeds address space"))?;
    let mut bytes = vec![0; length];
    file.read_exact(&mut bytes)?;
    if Sha256::digest(&bytes).as_slice() != index.sha256 {
        return Err(Error::Corrupt("IPC batch digest differs from catalog"));
    }
    Ok(Buffer::from_vec(bytes))
}

fn options() -> Result<IpcWriteOptions> {
    Ok(IpcWriteOptions::try_new(8, false, MetadataVersion::V5)?.try_with_compression(Some(arrow_ipc::CompressionType::ZSTD))?)
}

fn hash_state(bytes: &[u8]) -> Result<Sha256> {
    let state = SerializedState::<Sha256>::try_from(bytes).map_err(|_| Error::Corrupt("IPC hash checkpoint length differs"))?;
    Sha256::deserialize(&state).map_err(|_| Error::Corrupt("IPC hash checkpoint is invalid"))
}

fn footer(file: &mut HashedFile, schema: &Schema, blocks: &[Block]) -> Result<()> {
    file.write_all(&[255, 255, 255, 255, 0, 0, 0, 0])?;
    let mut builder = flatbuffers::FlatBufferBuilder::new();
    let batches = builder.create_vector(blocks);
    let dictionaries = builder.create_vector::<Block>(&[]);
    let schema = IpcSchemaEncoder::new().schema_to_fb_offset(&mut builder, schema);
    let root = arrow_ipc::Footer::create(
        &mut builder,
        &arrow_ipc::FooterArgs {
            version: MetadataVersion::V5,
            schema: Some(schema),
            dictionaries: Some(dictionaries),
            recordBatches: Some(batches),
            custom_metadata: None,
        },
    );
    builder.finish(root, None);
    let bytes = builder.finished_data();
    let length = i32::try_from(bytes.len()).map_err(|_| Error::Data("IPC footer exceeds format range"))?;
    file.write_all(bytes)?;
    file.write_all(&length.to_le_bytes())?;
    file.write_all(b"ARROW1")?;
    {
        let _span = tracing::debug_span!("ingest.ipc.sync").entered();
        file.file.sync_all()?;
    }
    Ok(())
}

pub(crate) fn sync_parent(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        let parent = File::open(path.parent().ok_or(Error::Data("IPC file has no parent"))?)?;
        let _span = tracing::debug_span!("ingest.ipc.sync").entered();
        parent.sync_all()?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use arrow_array::{Int64Array, StringArray};
    use arrow_schema::{DataType, Field, Schema};
    use std::sync::Arc;
    fn batch() -> RecordBatch {
        RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new("integer", DataType::Int64, true), Field::new("text", DataType::Utf8, true)])),
            vec![
                Arc::new(Int64Array::from(vec![Some(-5), None, Some(i64::MAX)])),
                Arc::new(StringArray::from(vec![Some("hi"), Some("世界"), None])),
            ],
        )
        .unwrap()
    }
    #[test]
    fn separately_encoded_batches_match_the_arrow_file_writer_bytes() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("samples.arrow");
        let batch = batch();
        let mut writer = Writer::create(&path, batch.schema()).unwrap();
        writer.append(&batch).unwrap();
        let encoded = encode(&batch.schema(), &batch).unwrap();
        writer.write(&encoded).unwrap();
        writer.sync().unwrap();
        let ours = std::fs::read(&path).unwrap();
        let mut reference = Vec::new();
        let mut file = arrow_ipc::writer::FileWriter::try_new_with_options(&mut reference, &batch.schema(), options().unwrap()).unwrap();
        file.write(&batch).unwrap();
        file.write(&batch).unwrap();
        file.finish().unwrap();
        drop(file);
        assert_eq!(ours, reference[..ours.len()]);
    }
    #[test]
    fn finalize_appends_only_metadata_and_is_idempotent() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("samples.arrow");
        let batch = batch();
        let mut writer = Writer::create(&path, batch.schema()).unwrap();
        writer.append(&batch).unwrap();
        let before = std::fs::read(&path).unwrap();
        let result = writer.finish().unwrap();
        assert_eq!(writer.finish().unwrap(), result);
        let after = std::fs::read(&path).unwrap();
        assert_eq!(&after[..before.len()], before);
        assert_eq!(Sha256::digest(&after).as_slice(), result.1);
        assert!(writer.append(&batch).is_err());
    }
    #[test]
    fn rollback_restores_hash_and_block_list() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("samples.arrow");
        let batch = batch();
        let mut writer = Writer::create(&path, batch.schema()).unwrap();
        let checkpoint = writer.checkpoint();
        writer.fail_next_sync();
        assert!(writer.append(&batch).is_err());
        writer.rollback(&checkpoint).unwrap();
        writer.append(&batch).unwrap();
        let (length, hash) = writer.finish().unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(length, bytes.len() as u64);
        assert_eq!(hash, <[u8; 32]>::from(Sha256::digest(&bytes)));
        let reader = arrow_ipc::reader::FileReader::try_new(File::open(&path).unwrap(), None).unwrap();
        assert_eq!(reader.num_batches(), 1);
    }
    #[test]
    fn uncataloged_first_flush_can_resume_from_its_verified_header() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("samples.arrow");
        let batch = batch();
        let mut writer = Writer::create(&path, batch.schema()).unwrap();
        let header = writer.checkpoint();
        writer.append(&batch).unwrap();
        drop(writer);
        let mut writer = Writer::recover_empty(&path, batch.schema()).unwrap();
        assert_eq!(writer.checkpoint().end, header.end);
        writer.append(&batch).unwrap();
        writer.finish().unwrap();
        drop(writer);
        let reader = arrow_ipc::reader::FileReader::try_new(File::open(&path).unwrap(), None).unwrap();
        assert_eq!(reader.num_batches(), 1);
        assert_eq!(reader.collect::<std::result::Result<Vec<_>, _>>().unwrap(), [batch]);
    }
    #[test]
    fn zstd_file_round_trip_and_metadata_only_recovery() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("samples.arrow");
        let batch = batch();
        let mut writer = Writer::create(&path, batch.schema()).unwrap();
        let first = writer.append(&batch).unwrap();
        let second = writer.append(&batch).unwrap();
        let checkpoint = writer.checkpoint();
        let mut corrupt = second.clone();
        corrupt.sha256[0] ^= 1;
        assert!(read_batch(&path, batch.schema(), &corrupt).is_err());
        assert_eq!(read_batch(&path, batch.schema(), &second).unwrap(), batch);
        // Simulate A Crash During The Next Write, Before A Footer Exists.
        drop(writer);
        let mut file = OpenOptions::new().append(true).open(&path).unwrap();
        file.write_all(b"uncommitted partial batch").unwrap();
        drop(file);
        let before = std::fs::read(&path).unwrap()[..checkpoint.end as usize].to_vec();
        let (length, digest) = recover(&path, batch.schema(), &[first, second], &checkpoint).unwrap();
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(bytes.len() as u64, length);
        assert_eq!(Sha256::digest(&bytes).as_slice(), digest);
        assert_eq!(&bytes[..before.len()], before);
        let reader = arrow_ipc::reader::FileReader::try_new(File::open(&path).unwrap(), None).unwrap();
        let batches = reader.collect::<std::result::Result<Vec<_>, _>>().unwrap();
        assert_eq!(batches, vec![batch.clone(), batch]);
    }
}
