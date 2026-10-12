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

// Standard Library Imports
use std::mem::size_of;

// Third Party Imports
use harmos::{
    schema::{Cardinality, Event, Field, Kind, PathElement, Scalar, Validator},
    storage::{ObjectColumn, ObjectScalar, ObjectScalarKind},
};
use prost::encoding::{DecodeContext, WireType, decode_key, decode_varint, skip_field};
use rusqlite::{
    ToSql,
    types::{ToSqlOutput, Value as SqlValue, ValueRef},
};

// Local Imports
use crate::storage::sqlite::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(in crate::storage::sqlite::data) struct Cell {
    pub tag: u32,
    pub kind: &'static str,
    pub key: Key,
}

struct Borrowed<'a> {
    tag: u32,
    kind: &'a Kind,
    value: Value<'a>,
}

/* === Enums === */
pub(in crate::storage::sqlite::data) enum Decoder {
    Protobuf,
    Postcard(Validator),
}

pub(in crate::storage::sqlite::data) enum Key {
    Null,
    Fixed([u8; 8]),
    Text(Vec<u8>),
}

enum Value<'a> {
    Null,
    Bits(u64),
    Text(&'a [u8]),
}

/* === Traits === */

/* === Implementations === */
impl Decoder {
    pub(in crate::storage::sqlite::data) fn construction_bytes(
        fields: &[Field],
        encoding: harmos::storage::PayloadEncoding,
    ) -> Result<usize, Error> {
        match encoding {
            harmos::storage::PayloadEncoding::ProtobufV1 => Ok(0),
            harmos::storage::PayloadEncoding::PostcardV1 => {
                Validator::construction_memory_bytes(fields).map_err(|error| {
                    Error::Runtime(harmos::Error::backend(
                        "budget Postcard scalar schema",
                        harmos::BackendFailure::Permanent,
                        error,
                    ))
                })
            }
        }
    }

    pub(in crate::storage::sqlite::data) fn new(
        fields: &[Field],
        encoding: harmos::storage::PayloadEncoding,
    ) -> Result<Self, Error> {
        match encoding {
            harmos::storage::PayloadEncoding::ProtobufV1 => Ok(Self::Protobuf),
            harmos::storage::PayloadEncoding::PostcardV1 => {
                let validator = Validator::new(fields).map_err(|error| {
                    Error::Runtime(harmos::Error::backend(
                        "compile Postcard scalar schema",
                        harmos::BackendFailure::Permanent,
                        error,
                    ))
                })?;
                Ok(Self::Postcard(validator))
            }
        }
    }
}

impl ToSql for Key {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::Borrowed(match self {
            Self::Null => ValueRef::Null,
            Self::Fixed(bytes) => ValueRef::Blob(bytes),
            Self::Text(bytes) => ValueRef::Blob(bytes),
        }))
    }
}

/* === Public Functions === */

/* === Private Functions === */
pub(in crate::storage::sqlite::data) fn operand(value: &ObjectScalar) -> SqlValue {
    match value {
        ObjectScalar::Null => SqlValue::Null,
        ObjectScalar::Bool(value) => SqlValue::Blob(u64::from(*value).to_be_bytes().to_vec()),
        ObjectScalar::Unsigned(value) => SqlValue::Blob(value.to_be_bytes().to_vec()),
        ObjectScalar::Signed(value) => SqlValue::Blob(signed(*value).to_vec()),
        ObjectScalar::Float(value) => SqlValue::Blob(float(*value).to_vec()),
        ObjectScalar::Text(value) => SqlValue::Blob(value.as_bytes().to_vec()),
    }
}

pub(in crate::storage::sqlite::data) fn scalar_kind(kind: ObjectScalarKind) -> &'static str {
    match kind {
        ObjectScalarKind::Bool => "bool",
        ObjectScalarKind::U32 => "u32",
        ObjectScalarKind::U64 => "u64",
        ObjectScalarKind::I32 => "i32",
        ObjectScalarKind::I64 => "i64",
        ObjectScalarKind::F32 => "f32",
        ObjectScalarKind::F64 => "f64",
        ObjectScalarKind::String => "string",
    }
}

pub(in crate::storage::sqlite::data) fn decode(
    column: &ObjectColumn,
    value: ValueRef<'_>,
) -> Result<ObjectScalar, Error> {
    if value == ValueRef::Null {
        return if column.optional {
            Ok(ObjectScalar::Null)
        } else {
            Err(Error::Corrupt("required scalar is absent"))
        };
    }
    let ValueRef::Blob(bytes) = value else {
        return Err(Error::Corrupt("scalar key is not a blob"));
    };
    if column.kind == ObjectScalarKind::String {
        let text = std::str::from_utf8(bytes)
            .map_err(|_| Error::Corrupt("scalar string key is not UTF-8"))?;
        return Ok(ObjectScalar::Text(text.to_owned()));
    }
    let key = u64::from_be_bytes(
        bytes
            .try_into()
            .map_err(|_| Error::Corrupt("invalid scalar key width"))?,
    );
    Ok(match column.kind {
        ObjectScalarKind::Bool if key <= 1 => ObjectScalar::Bool(key != 0),
        ObjectScalarKind::U32 if u32::try_from(key).is_ok() => ObjectScalar::Unsigned(key),
        ObjectScalarKind::U64 => ObjectScalar::Unsigned(key),
        ObjectScalarKind::I32 | ObjectScalarKind::I64 => {
            let value = (key ^ (1 << 63)) as i64;
            if column.kind == ObjectScalarKind::I32 && i32::try_from(value).is_err() {
                return Err(Error::Corrupt("signed scalar exceeds its declared range"));
            }
            ObjectScalar::Signed(value)
        }
        ObjectScalarKind::F32 | ObjectScalarKind::F64 => {
            let value = f64::from_bits(if key & (1 << 63) == 0 {
                !key
            } else {
                key ^ (1 << 63)
            });
            if !value.is_finite() {
                return Err(Error::Data("selected scalar float is not finite"));
            }
            if column.kind == ObjectScalarKind::F32 && (value as f32) as f64 != value {
                return Err(Error::Corrupt(
                    "float scalar exceeds its declared precision",
                ));
            }
            ObjectScalar::Float(value)
        }
        _ => return Err(Error::Corrupt("scalar key exceeds its declared range")),
    })
}

pub(in crate::storage::sqlite::data) fn allocation_bytes(
    fields: &[Field],
    payload_bytes: usize,
    decoder: &Decoder,
) -> Result<usize, Error> {
    let count = fields.iter().filter(|field| supported(field)).count();
    let traversal = match decoder {
        Decoder::Protobuf => 0,
        // A Path Contains At Most One Field And Index Per Schema Level.
        Decoder::Postcard(_) => {
            (2 * (harmos::schema::MAX_DEPTH + 1)).next_power_of_two() * size_of::<PathElement<'_>>()
        }
    };
    count
        .checked_mul(size_of::<Borrowed<'_>>() + size_of::<Cell>())
        .and_then(|bytes| bytes.checked_add(payload_bytes))
        .and_then(|bytes| bytes.checked_add(traversal))
        .ok_or(Error::Data("scalar index allocation size overflow"))
}

pub(in crate::storage::sqlite::data) fn encode(
    fields: &[Field],
    mut payload: &[u8],
    decoder: &Decoder,
) -> Result<Vec<Cell>, Error> {
    let count = fields.iter().filter(|field| supported(field)).count();
    let mut values = Vec::with_capacity(count);
    for field in fields.iter().filter(|field| supported(field)) {
        let value = if field.cardinality == Cardinality::Optional {
            Value::Null
        } else if field.kind == Kind::String {
            Value::Text(&[])
        } else {
            Value::Bits(0)
        };
        values.push(Borrowed {
            tag: field.tag,
            kind: &field.kind,
            value,
        });
    }
    values.sort_unstable_by_key(|value| value.tag);
    if let Decoder::Postcard(validator) = decoder {
        validator
            .visit(payload, |path, event| {
                let ([PathElement::Field { tag, .. }], Event::Scalar(scalar)) = (path, event)
                else {
                    return Ok::<(), std::convert::Infallible>(());
                };
                if let Ok(index) = values.binary_search_by_key(tag, |value| value.tag) {
                    values[index].value = match scalar {
                        Scalar::Bool(value) => Value::Bits(u64::from(value)),
                        Scalar::U32(value) => Value::Bits(u64::from(value)),
                        Scalar::U64(value) => Value::Bits(value),
                        Scalar::I32(value) => Value::Bits(value as i64 as u64),
                        Scalar::I64(value) => Value::Bits(value as u64),
                        Scalar::F32(value) => Value::Bits(u64::from(value.to_bits())),
                        Scalar::F64(value) => Value::Bits(value.to_bits()),
                        Scalar::String(value) => Value::Text(value.as_bytes()),
                        Scalar::Bytes(_) => unreachable!("bytes are not indexed"),
                    };
                }
                Ok(())
            })
            .map_err(|error| {
                Error::Runtime(harmos::Error::backend(
                    "index Postcard scalar",
                    harmos::BackendFailure::Permanent,
                    error,
                ))
            })?;
    } else {
        while !payload.is_empty() {
            let (tag, wire) = decode_key(&mut payload).map_err(codec)?;
            let Ok(index) = values.binary_search_by_key(&tag, |value| value.tag) else {
                skip_field(wire, tag, &mut payload, DecodeContext::default()).map_err(codec)?;
                continue;
            };
            let value = &mut values[index];
            let expected = match value.kind {
                Kind::F32 => WireType::ThirtyTwoBit,
                Kind::F64 => WireType::SixtyFourBit,
                Kind::String => WireType::LengthDelimited,
                _ => WireType::Varint,
            };
            if wire != expected {
                return Err(Error::Data("scalar wire type differs from its schema"));
            }
            value.value = match value.kind {
                Kind::F32 => Value::Bits(u32::from_le_bytes(
                    take(&mut payload, 4)?.try_into().unwrap(),
                ) as u64),
                Kind::F64 => Value::Bits(u64::from_le_bytes(
                    take(&mut payload, 8)?.try_into().unwrap(),
                )),
                Kind::String => {
                    let length = usize::try_from(decode_varint(&mut payload).map_err(codec)?)
                        .map_err(|_| Error::Data("scalar string length exceeds address space"))?;
                    let body = take(&mut payload, length)?;
                    std::str::from_utf8(body)
                        .map_err(|_| Error::Data("scalar string is not UTF-8"))?;
                    Value::Text(body)
                }
                _ => Value::Bits(decode_varint(&mut payload).map_err(codec)?),
            };
        }
    }
    let mut cells = Vec::with_capacity(count);
    for value in values {
        let kind = name(value.kind);
        let key = match value.value {
            Value::Null => Key::Null,
            Value::Text(bytes) => Key::Text(bytes.to_vec()),
            Value::Bits(bits) => Key::Fixed(match value.kind {
                Kind::Bool => u64::from(bits != 0).to_be_bytes(),
                Kind::U32 => (bits as u32 as u64).to_be_bytes(),
                Kind::U64 => bits.to_be_bytes(),
                Kind::I32 => signed(bits as i32 as i64),
                Kind::I64 => signed(bits as i64),
                Kind::F32 => float(f32::from_bits(bits as u32) as f64),
                Kind::F64 => float(f64::from_bits(bits)),
                _ => unreachable!("only scalar fields are planned"),
            }),
        };
        cells.push(Cell {
            tag: value.tag,
            kind,
            key,
        });
    }
    Ok(cells)
}

pub(in crate::storage::sqlite::data) fn supported(field: &Field) -> bool {
    matches!(
        field.cardinality,
        Cardinality::Required | Cardinality::Optional
    ) && !matches!(field.kind, Kind::Bytes | Kind::Message { .. })
}

fn name(kind: &Kind) -> &'static str {
    match kind {
        Kind::Bool => "bool",
        Kind::U32 => "u32",
        Kind::U64 => "u64",
        Kind::I32 => "i32",
        Kind::I64 => "i64",
        Kind::F32 => "f32",
        Kind::F64 => "f64",
        Kind::String => "string",
        _ => unreachable!("only scalar fields are planned"),
    }
}

fn signed(value: i64) -> [u8; 8] {
    ((value as u64) ^ (1 << 63)).to_be_bytes()
}

fn float(value: f64) -> [u8; 8] {
    // Canonical zero compares equally across both signs. Other finite values,
    // infinities and NaN payloads preserve their bits; selected nonfinite values
    // are refused by the query reader, never silently converted to SQL Null.
    let bits = if value == 0.0 { 0 } else { value.to_bits() };
    (if bits & (1 << 63) == 0 {
        bits ^ (1 << 63)
    } else {
        !bits
    })
    .to_be_bytes()
}

fn take<'a>(payload: &mut &'a [u8], length: usize) -> Result<&'a [u8], Error> {
    if length > payload.len() {
        return Err(Error::Data("truncated scalar wire payload"));
    }
    let (body, remaining) = payload.split_at(length);
    *payload = remaining;
    Ok(body)
}

fn codec(error: prost::DecodeError) -> Error {
    Error::Runtime(harmos::Error::backend(
        "index protobuf scalar",
        harmos::BackendFailure::Permanent,
        error,
    ))
}

/* === Tests === */
