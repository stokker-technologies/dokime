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
use std::{io::Write, time::UNIX_EPOCH};
// Third Party Imports
use harmos::storage::*;
use serde::Serialize;
use sha2::{Digest, Sha256};
// Local Imports
use super::super::Error;

/* === Constants === */
const MAX_KEY_BYTES: usize = 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Encoded {
    pub position: u64,
    pub kind: &'static str,
    pub identity: String,
    pub schema_id: Option<String>,
    pub schema_key: Option<String>,
    pub item: String,
    pub scope: Option<String>,
    pub stream: Option<String>,
    pub source_time: Option<[u8; 16]>,
    pub object_version: Option<String>,
    pub body: Vec<u8>,
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

pub(super) struct Retained {
    pub schema_key: Option<String>,
    pub position: u64,
    pub kind: &'static str,
    pub body: Vec<u8>,
    pub digest: [u8; 32],
}

/* === Enums === */
pub(crate) enum Entry {
    Schema(StorageSchema),
    Object(ObjectEntry),
    Record(StoredRecord),
    Stream(StoredRow),
    File(FileReference),
}

#[derive(Serialize)]
#[serde(untagged)]
enum Borrowed<'a> {
    Schema(&'a StorageSchema),
    Object(&'a ObjectEntry),
    Record(&'a StoredRecord),
    Stream(&'a StoredRow),
    File(&'a FileReference),
}

/* === Traits === */

/* === Implementations === */
impl<'a> From<StorageEntry<'a>> for Borrowed<'a> {
    fn from(value: StorageEntry<'a>) -> Self {
        match value {
            StorageEntry::Schema(value) => Self::Schema(value),
            StorageEntry::Object(value) => Self::Object(value),
            StorageEntry::Record(value) => Self::Record(value),
            StorageEntry::Row(value) => Self::Stream(value),
            StorageEntry::File(value) => Self::File(value),
        }
    }
}

impl Retained {
    pub(super) fn encoded_bytes(value: StorageEntry<'_>, maximum: usize) -> Result<usize, Error> {
        check_size(&Borrowed::from(value), maximum)
    }

    pub(super) fn new(value: StorageEntry<'_>, maximum: usize) -> Result<Self, Error> {
        let kind = match value {
            StorageEntry::Schema(_) => "schema",
            StorageEntry::Object(_) => "object",
            StorageEntry::Record(_) => "record",
            StorageEntry::Row(_) => "stream",
            StorageEntry::File(_) => "file",
        };
        let body = match value {
            StorageEntry::Object(value) => object::encode(value, object::length(value, maximum)?)?,
            _ => encode(&Borrowed::from(value), maximum)?,
        };
        let digest = Sha256::digest(&body).into();
        let schema_key = match value {
            StorageEntry::Schema(value) => Some(serde_json::to_string(&value.key)?),
            _ => None,
        };
        Ok(Self {
            schema_key,
            position: value.position().sequence,
            kind,
            body,
            digest,
        })
    }
}

impl Entry {
    pub(super) fn portable_bytes(kind: &str, body: &[u8]) -> Result<usize, Error> {
        if kind == "object" {
            object::portable_bytes(body)
        } else {
            Ok(body.len())
        }
    }

    pub(super) fn part(&self) -> StoragePart<'_> {
        match self {
            Self::Schema(value) => StoragePart::Schemas(std::slice::from_ref(value)),
            Self::Object(value) => StoragePart::Objects(std::slice::from_ref(value)),
            Self::Record(value) => StoragePart::Records(std::slice::from_ref(value)),
            Self::Stream(value) => StoragePart::Stream(std::slice::from_ref(value)),
            Self::File(value) => StoragePart::Files(std::slice::from_ref(value)),
        }
    }

    pub(super) fn decoding_bytes(kind: &str, encoded: usize) -> Result<usize, Error> {
        if kind == "schema" {
            return StorageSchema::decoding_memory_bytes(encoded)
                .and_then(|bytes| {
                    encoded
                        .checked_mul(3)
                        .and_then(|metadata| bytes.checked_add(metadata))
                })
                .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Self>()))
                .ok_or(Error::Data("decoded schema allocation size overflow"));
        }
        encoded
            .checked_mul(3)
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Self>()))
            .ok_or(Error::Data("decoded entry allocation size overflow"))
    }

    pub(super) fn fields_decoding_bytes(encoded: usize) -> Result<usize, Error> {
        let factor = 3 + (4 * std::mem::size_of::<harmos::schema::graph::Field>()).div_ceil(32);
        encoded
            .checked_mul(factor)
            .and_then(|bytes| bytes.checked_add(std::mem::size_of::<Self>()))
            .ok_or(Error::Data("decoded field allocation size overflow"))
    }

    pub(super) fn decode(kind: &str, body: &[u8]) -> Result<Self, Error> {
        Ok(match kind {
            "schema" => Self::Schema(serde_json::from_slice(body)?),
            "object" => Self::Object(object::decode(body)?),
            "record" => Self::Record(serde_json::from_slice(body)?),
            "stream" => Self::Stream(serde_json::from_slice(body)?),
            "file" => Self::File(serde_json::from_slice(body)?),
            _ => return Err(Error::Data("unknown stored entry kind")),
        })
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
        self.length = self
            .length
            .checked_add(bytes.len())
            .filter(|length| *length <= self.maximum)
            .ok_or_else(|| std::io::Error::other("SQLite entry exceeds its encoded bound"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Write for Buffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.bytes
            .len()
            .checked_add(bytes.len())
            .filter(|length| *length <= self.length)
            .ok_or_else(|| std::io::Error::other("SQLite entry exceeds its counted size"))?;
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl Encoded {
    pub(super) fn allocation_bytes(
        value: StorageEntry<'_>,
        maximum: usize,
    ) -> Result<(usize, usize), Error> {
        let (schema, item, scope, stream, version, body) = match value {
            StorageEntry::Schema(value) => (
                Some(&value.key),
                value.key.id.as_str(),
                None,
                None,
                None,
                Borrowed::Schema(value),
            ),
            StorageEntry::Object(value) => (
                Some(&value.value.schema),
                value.value.item.as_str(),
                None,
                None,
                Some(&value.version),
                Borrowed::Object(value),
            ),
            StorageEntry::Record(value) => (
                Some(&value.schema),
                value.item.as_str(),
                value.scope.as_ref(),
                None,
                None,
                Borrowed::Record(value),
            ),
            StorageEntry::Row(value) => (
                Some(&value.record.schema),
                value.record.item.as_str(),
                value.record.scope.as_ref(),
                Some(&value.stream),
                None,
                Borrowed::Stream(value),
            ),
            StorageEntry::File(value) => (
                None,
                value.metadata.id.as_str(),
                value.metadata.scope.as_ref(),
                None,
                None,
                Borrowed::File(value),
            ),
        };
        let mut strings = 0usize;
        for key in [
            Some(item),
            schema.map(|schema| schema.id.as_str()),
            scope.map(|scope| scope.as_str()),
            stream.map(|stream| stream.as_str()),
        ]
        .into_iter()
        .flatten()
        {
            if key.len() > MAX_KEY_BYTES {
                return Err(Error::Data("storage index identity exceeds 1024 bytes"));
            }
            strings = strings
                .checked_add(key.len())
                .ok_or(Error::Data("entry allocation size overflow"))?;
        }
        let schema_bytes = schema
            .map(|schema| check_size(schema, maximum))
            .transpose()?
            .unwrap_or(0);
        let version_bytes = version
            .map(|version| check_size(version, maximum))
            .transpose()?
            .unwrap_or(0);
        let identity_bytes = match value {
            StorageEntry::Schema(_) => Some(23),
            StorageEntry::Object(_) => Some(23),
            StorageEntry::Record(_) | StorageEntry::Row(_) => 5usize.checked_add(item.len()),
            StorageEntry::File(_) => 5usize.checked_add(item.len()),
        }
        .ok_or(Error::Data("entry allocation size overflow"))?;
        let payload_bytes = match value {
            StorageEntry::Object(value) => object::length(value, maximum)?,
            _ => check_size(&body, maximum)?,
        };
        let allocation = strings
            .checked_add(schema_bytes)
            .and_then(|bytes| bytes.checked_add(version_bytes))
            .and_then(|bytes| bytes.checked_add(identity_bytes))
            .and_then(|bytes| bytes.checked_mul(2))
            .and_then(|bytes| bytes.checked_add(payload_bytes))
            .ok_or(Error::Data("entry allocation size exceeds its bound"))?;
        Ok((allocation, payload_bytes))
    }

    pub(super) fn new(value: StorageEntry<'_>, length: usize) -> Result<Self, Error> {
        let record = match value {
            StorageEntry::Record(value) => Some(value),
            StorageEntry::Row(value) => Some(&value.record),
            _ => None,
        };
        let (kind, schema, item, borrowed) = match value {
            StorageEntry::Schema(value) => (
                "schema",
                Some(&value.key),
                value.key.id.as_str(),
                Borrowed::Schema(value),
            ),
            StorageEntry::Object(value) => (
                "object",
                Some(&value.value.schema),
                value.value.item.as_str(),
                Borrowed::Object(value),
            ),
            StorageEntry::Record(value) => (
                "record",
                Some(&value.schema),
                value.item.as_str(),
                Borrowed::Record(value),
            ),
            StorageEntry::Row(value) => (
                "stream",
                Some(&value.record.schema),
                value.record.item.as_str(),
                Borrowed::Stream(value),
            ),
            StorageEntry::File(value) => (
                "file",
                None,
                value.metadata.id.as_str(),
                Borrowed::File(value),
            ),
        };
        let position = value.position().sequence;
        let item = key(item)?;
        let schema_id = schema.map(|schema| key(schema.id.as_str())).transpose()?;
        let schema_key = schema.map(serde_json::to_string).transpose()?;
        let scope = match value {
            StorageEntry::File(value) => value.metadata.scope.as_ref(),
            _ => record.and_then(|record| record.scope.as_ref()),
        }
        .map(|scope| key(scope.as_str()))
        .transpose()?;
        let stream = match value {
            StorageEntry::Row(value) => Some(key(value.stream.as_str())?),
            _ => None,
        };
        let object_version = match value {
            StorageEntry::Object(value) => Some(serde_json::to_string(&value.version)?),
            _ => None,
        };
        let identity = match value {
            StorageEntry::Schema(_) => format!("schema:{position:016x}"),
            StorageEntry::Object(_) => format!("object:{position:016x}"),
            StorageEntry::Record(_) | StorageEntry::Row(_) => format!("item:{item}"),
            StorageEntry::File(_) => format!("file:{item}"),
        };
        let body = match value {
            StorageEntry::Object(value) => object::encode(value, length)?,
            _ => encode_counted(&borrowed, length)?,
        };
        let digest = Sha256::digest(&body).into();
        Ok(Self {
            position,
            kind,
            identity,
            schema_id,
            schema_key,
            item,
            scope,
            stream,
            source_time: record
                .and_then(|record| record.source_time)
                .map(timestamp_key),
            object_version,
            body,
            digest,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
pub(super) fn encoding_memory_bytes(value: StorageEntry<'_>) -> Result<usize, Error> {
    match value {
        StorageEntry::Schema(schema) => schema.encoding_memory_bytes().map_err(Error::Runtime),
        _ => Ok(0),
    }
}

pub(super) fn check_size<T: Serialize>(value: &T, maximum: usize) -> Result<usize, Error> {
    let mut counter = Counter { length: 0, maximum };
    serde_json::to_writer(&mut counter, value)?;
    Ok(counter.length)
}

pub(super) fn encode<T: Serialize>(value: &T, maximum: usize) -> Result<Vec<u8>, Error> {
    let length = check_size(value, maximum)?;
    encode_counted(value, length)
}

fn encode_counted<T: Serialize>(value: &T, length: usize) -> Result<Vec<u8>, Error> {
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length)
        .map_err(std::io::Error::other)?;
    if bytes.capacity() > length {
        return Err(Error::Data("SQLite entry capacity exceeds its preclaim"));
    }
    let mut buffer = Buffer { bytes, length };
    serde_json::to_writer(&mut buffer, value)?;
    if buffer.bytes.len() != length {
        return Err(Error::Data("SQLite entry differs from its counted size"));
    }
    Ok(buffer.bytes)
}

pub(super) fn timestamp_key(value: std::time::SystemTime) -> [u8; 16] {
    let nanos = match value.duration_since(UNIX_EPOCH) {
        Ok(duration) => duration.as_nanos() as i128,
        Err(error) => -(error.duration().as_nanos() as i128),
    };
    ((nanos as u128) ^ (1_u128 << 127)).to_be_bytes()
}

pub(super) fn key(value: &str) -> Result<String, Error> {
    if value.len() > MAX_KEY_BYTES {
        return Err(Error::Data("storage index identity exceeds 1024 bytes"));
    }
    Ok(value.to_owned())
}

/* === Tests === */
