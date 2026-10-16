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
use super::{Buffer, check_size};
use crate::storage::sqlite::Error;
use harmos::storage::{ObjectEntry, ObjectKey, ObjectVersionToken, SchemaKey, StorageCursor};
use serde::Serialize;

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */
const PREFIX: usize = std::mem::size_of::<u32>();

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Serialize)]
struct Header<'a> {
    position: &'a StorageCursor,
    version: &'a ObjectVersionToken,
    value: Value<'a>,
}
#[derive(Serialize)]
struct Value<'a> {
    schema: &'a SchemaKey,
    item: &'a ObjectKey,
    payload: &'static [u8],
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl<'a> From<&'a ObjectEntry> for Header<'a> {
    fn from(value: &'a ObjectEntry) -> Self {
        Self {
            position: &value.position,
            version: &value.version,
            value: Value {
                schema: &value.value.schema,
                item: &value.value.item,
                payload: &[],
            },
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */
pub(super) fn portable_bytes(bytes: &[u8]) -> Result<usize, Error> {
    let prefix = bytes
        .get(..PREFIX)
        .ok_or(Error::Corrupt("truncated SQLite object frame"))?;
    let metadata = u32::from_le_bytes(prefix.try_into().expect("fixed prefix")) as usize;
    let end = PREFIX
        .checked_add(metadata)
        .filter(|end| *end <= bytes.len())
        .ok_or(Error::Corrupt("invalid SQLite object metadata length"))?;
    let payload = &bytes[end..];
    // The Header Already Contains The Empty JSON Array's Brackets.
    payload.iter().try_fold(
        metadata
            .checked_add(payload.len().saturating_sub(1))
            .ok_or(Error::Corrupt("portable object size overflow"))?,
        |length, byte| {
            length
                .checked_add(if *byte < 10 {
                    1
                } else if *byte < 100 {
                    2
                } else {
                    3
                })
                .ok_or(Error::Corrupt("portable object size overflow"))
        },
    )
}

pub(super) fn length(value: &ObjectEntry, maximum: usize) -> Result<usize, Error> {
    let metadata = check_size(&Header::from(value), maximum)?;
    metadata
        .checked_add(PREFIX)
        .and_then(|length| length.checked_add(value.value.payload.len()))
        .filter(|length| *length <= maximum)
        .ok_or(Error::Data("SQLite object frame exceeds its bound"))
}

pub(super) fn encode(value: &ObjectEntry, length: usize) -> Result<Vec<u8>, Error> {
    let metadata = length
        .checked_sub(PREFIX)
        .and_then(|length| length.checked_sub(value.value.payload.len()))
        .ok_or(Error::Data(
            "SQLite object frame length differs from counted size",
        ))?;
    let size = u32::try_from(metadata)
        .map_err(|_| Error::Data("SQLite object metadata length overflows"))?;
    if check_size(&Header::from(value), metadata)? != metadata {
        return Err(Error::Data(
            "SQLite object metadata differs from counted size",
        ));
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(length)
        .map_err(std::io::Error::other)?;
    bytes.extend_from_slice(&size.to_le_bytes());
    let mut output = Buffer { bytes, length };
    serde_json::to_writer(&mut output, &Header::from(value))?;
    if output.bytes.len() != PREFIX + metadata {
        return Err(Error::Data(
            "SQLite object header differs from counted size",
        ));
    }
    output.bytes.extend_from_slice(&value.value.payload);
    if output.bytes.len() != length {
        return Err(Error::Data("SQLite object frame differs from counted size"));
    }
    Ok(output.bytes)
}

pub(super) fn decode(bytes: &[u8]) -> Result<ObjectEntry, Error> {
    let prefix = bytes
        .get(..PREFIX)
        .ok_or(Error::Corrupt("truncated SQLite object frame"))?;
    let metadata = u32::from_le_bytes(prefix.try_into().expect("fixed prefix")) as usize;
    let end = PREFIX
        .checked_add(metadata)
        .filter(|end| *end <= bytes.len())
        .ok_or(Error::Corrupt("invalid SQLite object metadata length"))?;
    let mut value: ObjectEntry = serde_json::from_slice(&bytes[PREFIX..end])?;
    if !value.value.payload.is_empty() {
        return Err(Error::Corrupt("SQLite object header contains a payload"));
    }
    let payload = &bytes[end..];
    value
        .value
        .payload
        .try_reserve_exact(payload.len())
        .map_err(std::io::Error::other)?;
    value.value.payload.extend_from_slice(payload);
    Ok(value)
}

/* === Tests === */
