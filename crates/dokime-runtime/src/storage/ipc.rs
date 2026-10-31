/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Durable Arrow IPC Blocks And Metadata-Only Finalization

/* === Modules === */

mod projection;
mod schema;
mod values;

/* === Re-Exports === */

pub(crate) use projection::Projection;
pub(crate) use schema::schema as contract_schema;
pub(crate) use values::{
    batch as record_batch, entry as telemetry_entry, metadata_text, native_encoding, normalize_values, row_key, row_keys,
};

/* === Imports === */

// Standard Library Imports
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
    reader::FileDecoder,
    writer::{DictionaryTracker, IpcDataGenerator, IpcWriteContext, IpcWriteOptions, write_message},
};
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
    options: IpcWriteOptions,
    dictionaries: DictionaryTracker,
    context: IpcWriteContext,
    finished: bool,
    #[cfg(test)]
    fail_sync: bool,
}

struct HashedFile {
    file: File,
    hash: Sha256,
    length: u64,
    batch_hash: Option<Sha256>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
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
                batch_hash: None,
            },
            schema,
            blocks: indices.iter().map(BatchIndex::block).collect::<Result<Vec<_>>>()?,
            options: options()?,
            dictionaries: DictionaryTracker::new(true),
            context: IpcWriteContext::default(),
            finished: false,
            #[cfg(test)]
            fail_sync: false,
        };
        writer.rollback(checkpoint)?;
        Ok(writer)
    }

    pub fn create(path: &Path, schema: SchemaRef) -> Result<Self> {
        let file = OpenOptions::new().create_new(true).write(true).open(path)?;
        let mut file = HashedFile {
            file,
            hash: Sha256::new(),
            length: 0,
            batch_hash: None,
        };
        let options = options()?;
        // IPC V5 Uses Eight-Byte Alignment And A Continuation Prefix.
        file.write_all(b"ARROW1\0\0")?;
        let mut dictionaries = DictionaryTracker::new(true);
        let message = IpcDataGenerator::default().schema_to_bytes_with_dictionary_tracker(&schema, &mut dictionaries, &options);
        write_message(&mut file, message, &options)?;
        file.file.sync_all()?;
        sync_parent(path)?;
        Ok(Self {
            file,
            schema,
            blocks: Vec::new(),
            options,
            dictionaries,
            context: IpcWriteContext::default(),
            finished: false,
            #[cfg(test)]
            fail_sync: false,
        })
    }

    #[cfg(test)]
    pub(crate) fn fail_next_sync(&mut self) {
        self.fail_sync = true;
    }

    pub(crate) fn digest(&self) -> [u8; 32] {
        self.file.hash.clone().finalize().into()
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            end: self.file.length,
            hash: self.file.hash.serialize().to_vec(),
            batches: self.blocks.len(),
        }
    }

    pub fn append(&mut self, batch: &RecordBatch) -> Result<BatchIndex> {
        if self.finished || batch.schema() != self.schema {
            return Err(Error::Invalid("Arrow writer is finished or its batch schema differs"));
        }
        let generator = IpcDataGenerator::default();
        let (dictionaries, message) = generator.encode(batch, &mut self.dictionaries, &self.options, &mut self.context)?;
        if !dictionaries.is_empty() {
            return Err(Error::Invalid("telemetry IPC does not use dictionary encoding"));
        }
        let offset = self.file.length;
        self.file.batch_hash = Some(Sha256::new());
        let (metadata, body) = write_message(&mut self.file, message, &self.options)?;
        let index = BatchIndex {
            offset,
            metadata_bytes: u32::try_from(metadata).map_err(|_| Error::Invalid("IPC metadata length overflows"))?,
            body_bytes: body as u64,
            rows: batch.num_rows() as u64,
            sha256: self.file.batch_hash.take().expect("batch hash is active").finalize().into(),
        };
        #[cfg(test)]
        if std::mem::take(&mut self.fail_sync) {
            return Err(Error::Io(std::io::Error::other("injected batch sync failure")));
        }
        self.file.file.sync_all()?;
        self.blocks.push(index.block()?);
        Ok(index)
    }

    // Restore The Last Committed Prefix After A Failed Append Or Catalog Commit.
    pub fn rollback(&mut self, checkpoint: &Checkpoint) -> Result<()> {
        self.file.file.set_len(checkpoint.end)?;
        self.file.file.seek(SeekFrom::Start(checkpoint.end))?;
        self.file.hash = hash_state(&checkpoint.hash)?;
        self.file.length = checkpoint.end;
        self.file.batch_hash = None;
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
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let written = self.file.write(bytes)?;
        self.hash.update(&bytes[..written]);
        if let Some(hash) = &mut self.batch_hash {
            hash.update(&bytes[..written]);
        }
        self.length += written as u64;
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

/* === Public Functions === */

pub(crate) fn read_batch(path: &Path, schema: SchemaRef, index: &BatchIndex) -> Result<RecordBatch> {
    let buffer = read_buffer(path, index)?;
    let decoder = FileDecoder::new(schema, MetadataVersion::V5);
    let batch = decoder.read_record_batch(&index.block()?, &buffer)?.ok_or(Error::Corrupt("IPC block is not a record batch"))?;
    if batch.num_rows() as u64 != index.rows {
        return Err(Error::Corrupt("IPC batch row count differs"));
    }
    Ok(batch)
}

pub(crate) fn read_projected_batch(path: &Path, index: &BatchIndex, projection: &Projection) -> Result<RecordBatch> {
    let buffer = read_buffer(path, index)?;
    projection.decode(&buffer, index)
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
        batch_hash: None,
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

fn footer(file: &mut HashedFile, schema: &arrow_schema::Schema, blocks: &[Block]) -> Result<()> {
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
    let length = i32::try_from(bytes.len()).map_err(|_| Error::Invalid("IPC footer exceeds format range"))?;
    file.write_all(bytes)?;
    file.write_all(&length.to_le_bytes())?;
    file.write_all(b"ARROW1")?;
    file.file.sync_all()?;
    Ok(())
}

fn sync_parent(path: &Path) -> Result<()> {
    #[cfg(unix)]
    File::open(path.parent().ok_or(Error::Invalid("IPC file has no parent"))?)?.sync_all()?;
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
        let fields = vec![Field::new("integer", DataType::Int64, true), Field::new("text", DataType::Utf8, true)];
        let schema = Arc::new(Schema::new(fields));
        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(Int64Array::from(vec![Some(-5), None, Some(i64::MAX)])),
                Arc::new(StringArray::from(vec![Some("hi"), Some("世界"), None])),
            ],
        )
        .unwrap()
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
        writer.append(&batch).unwrap();
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
}
