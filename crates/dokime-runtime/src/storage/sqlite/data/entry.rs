/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod object;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::result::Result;
use std::sync::Arc;
use std::{io::Write, time::UNIX_EPOCH};

// Third Party Imports
use harmos::storage::*;
use serde::Serialize;
use sha2::{Digest, Sha256};

// Local Imports
use super::super::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Encoded<'a> {
    pub position: u64,
    pub kind: &'static str,
    pub identity: Identity,
    pub schema_id: Option<&'a str>,
    pub schema_key: Option<Arc<str>>,
    pub item: &'a str,
    pub scope: Option<&'a str>,
    pub stream: Option<&'a str>,
    pub source_time: Option<[u8; 16]>,
    pub object_version: Option<String>,
    pub body: std::ops::Range<usize>,
    pub digest: [u8; 32],
}

struct Counter {
    length: usize,
    maximum: usize,
}

struct Buffer {
    bytes: Vec<u8>,
    length: usize,
}

#[derive(Default)]
pub(super) struct Encoder<'a> {
    schema: Option<(&'a SchemaKey, Arc<str>)>,
    pub bytes: Vec<u8>,
}

pub(super) struct Identity {
    positioned: [u8; 23],
    file: Option<String>,
}

#[derive(Serialize)]
struct StreamHeader<'a> {
    record: RecordHeader<'a>,
    stream: &'a harmos::Identifier,
    sequence: u64,
}

#[derive(Serialize)]
struct RecordHeader<'a> {
    position: &'a StorageCursor,
    schema: &'a SchemaKey,
    scope: Option<&'a harmos::Identifier>,
    item: &'a harmos::Identifier,
    source: Option<&'a harmos::Identifier>,
    source_time: Option<harmos::domain::model::storage::time::Wire>,
    payload: &'static [u8],
}

/* === Enums === */

pub(crate) enum Entry {
    Schema(StorageSchema),
    Object(ObjectEntry),
    Record(StoredRecord),
    Stream(StoredRow),
    File(FileReference),
}

#[derive(Clone, Copy, Serialize)]
#[serde(untagged)]
pub(super) enum Borrowed<'a> {
    Schema(&'a StorageSchema),
    Object(&'a ObjectEntry),
    Record(&'a StoredRecord),
    Stream(&'a StoredRow),
    File(&'a FileReference),
}

/* === Traits === */

/* === Implementations === */

impl Identity {
    pub(super) fn as_str(&self) -> &str {
        self.file.as_deref().unwrap_or_else(|| std::str::from_utf8(&self.positioned).expect("ASCII identity"))
    }

    pub(super) fn position(&self) -> &[u8] {
        &self.positioned[7..]
    }
}

impl<'a> From<&'a StorageSchema> for Borrowed<'a> {
    fn from(value: &'a StorageSchema) -> Self {
        Self::Schema(value)
    }
}

impl<'a> From<&'a ObjectEntry> for Borrowed<'a> {
    fn from(value: &'a ObjectEntry) -> Self {
        Self::Object(value)
    }
}

impl<'a> From<&'a StoredRecord> for Borrowed<'a> {
    fn from(value: &'a StoredRecord) -> Self {
        Self::Record(value)
    }
}

impl<'a> From<&'a StoredRow> for Borrowed<'a> {
    fn from(value: &'a StoredRow) -> Self {
        Self::Stream(value)
    }
}

impl<'a> From<&'a FileReference> for Borrowed<'a> {
    fn from(value: &'a FileReference) -> Self {
        Self::File(value)
    }
}

impl Borrowed<'_> {
    pub(super) fn position(&self) -> &StorageCursor {
        match self {
            Self::Schema(value) => &value.position,
            Self::Object(value) => &value.position,
            Self::Record(value) => &value.position,
            Self::Stream(value) => &value.record.position,
            Self::File(value) => &value.position,
        }
    }
}

impl Entry {
    pub(super) fn portable_bytes(kind: &str, body: &[u8]) -> Result<usize, Error> {
        if matches!(kind, "object" | "stream") { object::portable_bytes(body) } else { Ok(body.len()) }
    }

    pub(super) fn decode(kind: &str, body: &[u8]) -> Result<Self, Error> {
        let entry = match kind {
            "schema" => Self::Schema(serde_json::from_slice(body)?),
            "object" => Self::Object(object::decode(body)?),
            "record" => Self::Record(serde_json::from_slice(body)?),
            "stream" => Self::Stream(decode_stream(body)?),
            "file" => Self::File(serde_json::from_slice(body)?),
            _ => return Err(Error::Data("unknown stored entry kind")),
        };
        Ok(entry)
    }
    pub(crate) fn position(&self) -> &StorageCursor {
        match self {
            Self::Schema(value) => &value.position,
            Self::Object(value) => &value.position,
            Self::Record(value) => &value.position,
            Self::Stream(value) => &value.record.position,
            Self::File(value) => &value.position,
        }
    }
}

impl Write for Counter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let Some(length) = self.length.checked_add(bytes.len()) else {
            return Err(std::io::Error::other("SQLite entry exceeds its encoded bound"));
        };
        if length > self.maximum {
            return Err(std::io::Error::other("SQLite entry exceeds its encoded bound"));
        }
        self.length = length;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Write for Buffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let Some(length) = self.bytes.len().checked_add(bytes.len()) else {
            return Err(std::io::Error::other("SQLite entry exceeds its counted size"));
        };
        if length > self.length {
            return Err(std::io::Error::other("SQLite entry exceeds its counted size"));
        }
        self.bytes.try_reserve(bytes.len()).map_err(std::io::Error::other)?;
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl<'a> Encoder<'a> {
    pub(super) fn encode(&mut self, value: Borrowed<'a>) -> Result<Encoded<'a>, Error> {
        let record = match value {
            Borrowed::Record(value) => Some(value),
            Borrowed::Stream(value) => Some(&value.record),
            _ => None,
        };

        let (kind, schema, item, borrowed) = match value {
            Borrowed::Schema(value) => ("schema", Some(&value.key), value.key.id.as_str(), Borrowed::Schema(value)),
            Borrowed::Object(value) => ("object", Some(&value.value.schema), value.value.item.as_str(), Borrowed::Object(value)),
            Borrowed::Record(value) => ("record", Some(&value.schema), value.item.as_str(), Borrowed::Record(value)),
            Borrowed::Stream(value) => ("stream", Some(&value.record.schema), value.record.item.as_str(), Borrowed::Stream(value)),
            Borrowed::File(value) => ("file", None, value.metadata.id.as_str(), Borrowed::File(value)),
        };
        let position = value.position().sequence;
        let schema_id = schema.map(|schema| schema.id.as_str());
        let schema_key = match schema {
            Some(schema) => {
                if !self.schema.as_ref().is_some_and(|(previous, _)| *previous == schema) {
                    self.schema = Some((schema, Arc::from(encode_string(schema)?)));
                }
                Some(self.schema.as_ref().expect("encoded schema").1.clone())
            }
            None => None,
        };

        let scope = if let Borrowed::File(file) = value {
            file.metadata.scope.as_ref().map(harmos::Identifier::as_str)
        } else {
            match record {
                Some(record) => record.scope.as_ref().map(harmos::Identifier::as_str),
                None => None,
            }
        };
        let stream = match value {
            Borrowed::Stream(value) => Some(value.stream.as_str()),
            _ => None,
        };
        let object_version = if let Borrowed::Object(object) = value { Some(encode_string(&object.version)?) } else { None };

        let identity = identity(value, position, item)?;
        let start = self.bytes.len();
        let mut buffer = Buffer {
            bytes: std::mem::take(&mut self.bytes),
            length: usize::MAX,
        };
        match value {
            Borrowed::Object(value) => buffer.write_all(&object::encode(value, object::length(value, usize::MAX)?)?)?,
            Borrowed::Stream(value) => encode_stream(&mut buffer, value)?,
            _ => serde_json::to_writer(&mut buffer, &borrowed)?,
        }
        let body = start..buffer.bytes.len();
        let digest = Sha256::digest(&buffer.bytes[body.clone()]).into();
        self.bytes = buffer.bytes;
        let source_time = match record {
            Some(record) => record.source_time.map(timestamp_key),
            None => None,
        };

        Ok(Encoded {
            position,
            kind,
            identity,
            schema_id,
            schema_key,
            item,
            scope,
            stream,
            source_time,
            object_version,
            body,
            digest,
        })
    }
}

/* === Public Functions === */

pub(super) fn read_exhausted() -> Error {
    Error::Data("SQLite size overflows")
}

pub(super) fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, Error> {
    let mut buffer = Buffer { bytes: Vec::new(), length: usize::MAX };
    serde_json::to_writer(&mut buffer, value)?;
    Ok(buffer.bytes)
}

pub(super) fn check_size<T: Serialize>(value: &T, maximum: usize) -> Result<usize, Error> {
    let mut counter = Counter { length: 0, maximum };
    serde_json::to_writer(&mut counter, value)?;
    Ok(counter.length)
}

pub(super) fn timestamp_key(value: std::time::SystemTime) -> [u8; 16] {
    let nanos = match value.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos() as i128,
        Err(error) => -(error.duration().as_nanos() as i128),
    };
    ((nanos as u128) ^ (1_u128 << 127)).to_be_bytes()
}

pub(super) fn key(value: &str) -> Result<String, Error> {
    let mut key = String::new();
    key.try_reserve_exact(value.len()).map_err(std::io::Error::other)?;
    key.push_str(value);
    Ok(key)
}
/* === Private Functions === */

// Store Native Stream Payloads As Bytes, Not A JSON Integer For Every Byte.
fn stream_header(value: &StoredRow) -> StreamHeader<'_> {
    StreamHeader {
        record: RecordHeader {
            position: &value.record.position,
            schema: &value.record.schema,
            scope: value.record.scope.as_ref(),
            item: &value.record.item,
            source: value.record.source.as_ref(),
            source_time: value.record.source_time.map(Into::into),
            payload: &[],
        },
        stream: &value.stream,
        sequence: value.sequence,
    }
}

pub(super) fn stream_length(value: &StoredRow, maximum: usize) -> Result<usize, Error> {
    let header = check_size(&stream_header(value), maximum)?;
    u32::try_from(header).map_err(|_| Error::Data("SQLite stream header length overflows"))?;
    header
        .checked_add(4)
        .and_then(|length| length.checked_add(value.record.payload.len()))
        .filter(|length| *length <= maximum)
        .ok_or(Error::Data("SQLite stream frame exceeds its bound"))
}

fn encode_stream(buffer: &mut Buffer, value: &StoredRow) -> Result<(), Error> {
    let start = buffer.bytes.len();
    buffer.write_all(&[0; 4])?;
    serde_json::to_writer(&mut *buffer, &stream_header(value))?;
    let prefix = u32::try_from(buffer.bytes.len() - start - 4).map_err(|_| Error::Data("SQLite stream header length overflows"))?;
    buffer.bytes[start..start + 4].copy_from_slice(&prefix.to_le_bytes());
    buffer.bytes.try_reserve(value.record.payload.len()).map_err(std::io::Error::other)?;
    buffer.bytes.extend_from_slice(&value.record.payload);
    Ok(())
}

fn decode_stream(bytes: &[u8]) -> Result<StoredRow, Error> {
    let prefix = bytes.get(..4).ok_or(Error::Corrupt("truncated SQLite stream frame"))?;
    let length = u32::from_le_bytes(prefix.try_into().expect("fixed prefix")) as usize;
    let end = 4usize.checked_add(length).filter(|end| *end <= bytes.len()).ok_or(Error::Corrupt("invalid SQLite stream header length"))?;
    let mut row: StoredRow = serde_json::from_slice(&bytes[4..end])?;
    if !row.record.payload.is_empty() {
        return Err(Error::Corrupt("SQLite stream header contains a payload"));
    }
    row.record.payload.try_reserve_exact(bytes.len() - end).map_err(std::io::Error::other)?;
    row.record.payload.extend_from_slice(&bytes[end..]);
    Ok(row)
}

fn encode_counted<T: Serialize>(value: &T, length: usize) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(length).map_err(std::io::Error::other)?;
    let mut buffer = Buffer { bytes, length };
    serde_json::to_writer(&mut buffer, value)?;
    if buffer.bytes.len() != length {
        return Err(Error::Data("SQLite entry differs from its counted size"));
    }
    Ok(buffer.bytes)
}

fn encode_string<T: Serialize>(value: &T) -> Result<String, Error> {
    let length = check_size(value, usize::MAX)?;
    let bytes = encode_counted(value, length)?;
    let text = match String::from_utf8(bytes) {
        Ok(text) => text,
        Err(_) => return Err(Error::Data("SQLite index is not UTF-8")),
    };
    Ok(text)
}

fn identity(value: Borrowed<'_>, position: u64, item: &str) -> Result<Identity, Error> {
    let (prefix, item) = match value {
        Borrowed::Schema(_) => ("schema:", None),
        Borrowed::Object(_) => ("object:", None),
        Borrowed::Record(_) => ("record:", None),
        Borrowed::Stream(_) => ("stream:", None),
        Borrowed::File(_) => ("file:", Some(item)),
    };
    let mut positioned = [b'0'; 23];
    for (digits, byte) in positioned[7..].as_chunks_mut::<2>().0.iter_mut().zip(position.to_be_bytes()) {
        digits[0] = b"0123456789abcdef"[(byte >> 4) as usize];
        digits[1] = b"0123456789abcdef"[(byte & 15) as usize];
    }
    let file = if let Some(item) = item {
        let length = prefix.len().checked_add(item.len()).ok_or_else(read_exhausted)?;
        let mut identity = String::new();
        identity.try_reserve_exact(length).map_err(std::io::Error::other)?;
        identity.push_str(prefix);
        identity.push_str(item);
        Some(identity)
    } else {
        positioned[..7].copy_from_slice(prefix.as_bytes());
        None
    };
    Ok(Identity { positioned, file })
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrowed_stream_headers_preserve_stored_frames() {
        for source_time in [None, Some(UNIX_EPOCH), Some(UNIX_EPOCH - std::time::Duration::from_nanos(1))] {
            let row = StoredRow {
                record: StoredRecord {
                    position: StorageCursor {
                        history: StoreHistory {
                            store: harmos::Identifier::new("store").unwrap(),
                            epoch: harmos::Identifier::new("epoch").unwrap(),
                        },
                        sequence: 1,
                    },
                    schema: SchemaKey {
                        id: harmos::Identifier::new("samples").unwrap(),
                        version: std::num::NonZeroU32::new(1).unwrap(),
                        payload_encoding: PayloadEncoding::PostcardV1,
                    },
                    scope: Some(harmos::Identifier::new("owner").unwrap()),
                    item: harmos::Identifier::new("item").unwrap(),
                    source: Some(harmos::Identifier::new("source").unwrap()),
                    source_time,
                    payload: vec![0, 127, 255],
                },
                stream: harmos::Identifier::new("stream").unwrap(),
                sequence: 9,
            };
            let mut header = row.clone();
            header.record.payload.clear();
            let header = serde_json::to_vec(&header).unwrap();
            let mut encoder = Encoder::default();
            let entry = encoder.encode(Borrowed::Stream(&row)).unwrap();
            let encoded = &encoder.bytes[entry.body];
            assert_eq!(&encoded[4..4 + header.len()], header);
            assert_eq!(encoded.len(), stream_length(&row, usize::MAX).unwrap());
            assert_eq!(decode_stream(encoded).unwrap(), row);
        }
    }
}
