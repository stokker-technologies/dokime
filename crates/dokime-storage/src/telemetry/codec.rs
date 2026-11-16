/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Frozen Descriptor Arrow Columns

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::HashMap, sync::Arc};

// Third Party Imports
use arrow_array::builder::{BinaryBuilder, StringBuilder};
use arrow_array::{
    Array, ArrayRef, BinaryArray, BooleanArray, Float64Array, Int64Array, ListArray, RecordBatch, StringArray, StructArray, UInt64Array,
};
use arrow_buffer::{NullBuffer, OffsetBuffer, ScalarBuffer};
use arrow_schema::{DataType, Field, Fields, Schema, SchemaRef};
use harmos_domain::message::postcard::{Reader, Value as Postcard};
use prost::Message;
use prost_reflect::{DynamicMessage, Kind, MapKey, MessageDescriptor, ReflectMessage, Value};
use sha2::{Digest, Sha256};

// Local Imports
use crate::{Error, Result, Sample};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub(crate) struct Codec {
    pub descriptor: MessageDescriptor,
    pub schema: SchemaRef,
    pub digest: Vec<u8>,
}

/// One Stored Interval Batch: `count` Samples, the First at `start_ns`, Each `period_ns` Later.
pub(crate) struct Interval {
    pub start_ns: i64,
    pub period_ns: u64,
    pub count: usize,
    pub values: ArrayRef,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Codec {
    pub fn new(descriptor: MessageDescriptor) -> Result<Self> {
        let fields = message_fields(&descriptor, &mut Vec::new())?;
        let storage = Fields::from(vec![
            Field::new("position", DataType::UInt64, false),
            Field::new("source_ns", DataType::Int64, false),
            Field::new("receipt_ns", DataType::Int64, false),
            Field::new("sequence", DataType::UInt64, false),
        ]);
        let schema = Arc::new(Schema::new(vec![
            Field::new("_storage", DataType::Struct(storage), false),
            Field::new("values", DataType::Struct(fields), false),
        ]));
        let digest =
            Sha256::digest(format!("{}:{:?}", descriptor.full_name(), descriptor.parent_pool().encode_to_vec()).as_bytes()).to_vec();
        Ok(Self { descriptor, schema, digest })
    }
    pub fn batch(&self, samples: &[Sample]) -> Result<RecordBatch> {
        let mut rows = Vec::with_capacity(samples.len());
        for sample in samples {
            let row = DynamicMessage::decode(self.descriptor.clone(), sample.row.as_slice())?;
            if row.unknown_fields().next().is_some() {
                return Err(Error::Data("include every sample field in the frozen descriptors"));
            }
            rows.push(Some(Value::Message(row)));
        }
        let storage = StructArray::from(vec![
            (
                Arc::new(Field::new("position", DataType::UInt64, false)),
                Arc::new(UInt64Array::from_iter_values(samples.iter().map(|s| s.position))) as ArrayRef,
            ),
            (
                Arc::new(Field::new("source_ns", DataType::Int64, false)),
                Arc::new(Int64Array::from_iter_values(samples.iter().map(|s| s.source_ns))) as ArrayRef,
            ),
            (
                Arc::new(Field::new("receipt_ns", DataType::Int64, false)),
                Arc::new(Int64Array::from_iter_values(samples.iter().map(|s| s.receipt_ns))) as ArrayRef,
            ),
            (
                Arc::new(Field::new("sequence", DataType::UInt64, false)),
                Arc::new(UInt64Array::from_iter_values(samples.iter().map(|s| s.sequence))) as ArrayRef,
            ),
        ]);
        let values = column(self.schema.field(1).data_type(), &rows)?;
        Ok(RecordBatch::try_new(self.schema.clone(), vec![Arc::new(storage), values])?)
    }
    /// Decode One Stored `harmos.batch.<row>` Row Straight Into This Row's Arrow Columns.
    ///
    /// The payload is the host's Postcard encoding of the derived batch message: the first
    /// sample's Unix nanoseconds, the period, the count, then the columns message, whose fields
    /// repeat the row's fields in declaration order with one value per sample.
    pub fn interval(&self, payload: &[u8]) -> Result<Interval> {
        let malformed = |_| Error::Data("store interval batches in the host's derived batch layout");
        let mut input = Reader::new(payload);
        let start_ns: i64 = input.value().map_err(malformed)?;
        let period_ns: u64 = input.value().map_err(malformed)?;
        let count: u64 = input.value().map_err(malformed)?;
        let present: u8 = Postcard::read(&mut input).map_err(malformed)?;
        let count = usize::try_from(count).map_err(|_| Error::Data("reduce the interval batch size"))?;
        if present != 1 || count == 0 {
            return Err(Error::Data("store interval batches with at least one sample of every column"));
        }
        let DataType::Struct(fields) = self.schema.field(1).data_type() else {
            return Err(Error::Corrupt("values layout differs"));
        };
        let mut columns = Vec::with_capacity(fields.len());
        for (field, arrow) in self.descriptor.fields().zip(fields.iter()) {
            let length: usize = input.value().map_err(malformed)?;
            if length != count {
                return Err(Error::Data("send one value per sample in every interval column"));
            }
            columns.push(interval_column(&mut input, &field.kind(), arrow.data_type(), count)?);
        }
        input.finish().map_err(malformed)?;
        let values: ArrayRef = if fields.is_empty() {
            Arc::new(StructArray::new_empty_fields(count, None))
        } else {
            Arc::new(StructArray::try_new(fields.clone(), columns, None)?)
        };
        Ok(Interval { start_ns, period_ns, count, values })
    }
    /// Assemble Admitted Values With Their Positions, Times and Sequences, One Arrow Row per Sample.
    pub fn assemble(&self, first: u64, sequence: u64, interval: &Interval) -> Result<RecordBatch> {
        let span = |index: u64| -> Result<i64> {
            i64::try_from(i128::from(interval.start_ns) + i128::from(index) * i128::from(interval.period_ns))
                .map_err(|_| Error::Data("keep interval sample times before 2262"))
        };
        let count = interval.count as u64;
        span(count - 1)?;
        let times = Arc::new(Int64Array::from_iter_values(
            // The Last Time Fits, So Every Earlier Offset Fits Too; Wrapping Keeps Two's-Complement Exact.
            (0..count).map(|index| interval.start_ns.wrapping_add(index.wrapping_mul(interval.period_ns) as i64)),
        )) as ArrayRef;
        let storage = StructArray::from(vec![
            (
                Arc::new(Field::new("position", DataType::UInt64, false)),
                Arc::new(UInt64Array::from_iter_values(first..first + count)) as ArrayRef,
            ),
            (Arc::new(Field::new("source_ns", DataType::Int64, false)), times.clone()),
            (Arc::new(Field::new("receipt_ns", DataType::Int64, false)), times),
            (
                Arc::new(Field::new("sequence", DataType::UInt64, false)),
                Arc::new(UInt64Array::from_iter_values(sequence..sequence + count)) as ArrayRef,
            ),
        ]);
        Ok(RecordBatch::try_new(self.schema.clone(), vec![Arc::new(storage), interval.values.clone()])?)
    }
    pub fn sample(&self, stream: &str, batch: &RecordBatch, row: usize) -> Result<Sample> {
        let storage = batch.column(0).as_any().downcast_ref::<StructArray>().ok_or(Error::Corrupt("storage metadata differs"))?;
        let metadata = |name| storage.column_by_name(name).ok_or(Error::Corrupt("sample metadata missing"));
        let position =
            metadata("position")?.as_any().downcast_ref::<UInt64Array>().ok_or(Error::Corrupt("position type differs"))?.value(row);
        let sequence =
            metadata("sequence")?.as_any().downcast_ref::<UInt64Array>().ok_or(Error::Corrupt("sequence type differs"))?.value(row);
        let source_ns = metadata("source_ns")?.as_any().downcast_ref::<Int64Array>().ok_or(Error::Corrupt("time type differs"))?.value(row);
        let receipt_ns =
            metadata("receipt_ns")?.as_any().downcast_ref::<Int64Array>().ok_or(Error::Corrupt("time type differs"))?.value(row);
        let values = batch.column(1).as_any().downcast_ref::<StructArray>().ok_or(Error::Corrupt("values differ"))?;
        let message = message(&self.descriptor, values, row)?;
        Ok(Sample {
            stream: stream.into(),
            position,
            source_ns,
            receipt_ns,
            sequence,
            row: message.encode_to_vec(),
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn layout() -> Error {
    Error::Data("store interval batches in the host's derived batch layout")
}

fn read<'a, T: serde::Deserialize<'a>>(input: &mut Reader<'a>) -> Result<T> {
    input.value().map_err(|_| layout())
}

/// One Interval Column, Decoded Straight Into the Row Field's Arrow Array Without Per-Sample Rows.
fn interval_column(input: &mut Reader<'_>, kind: &Kind, arrow: &DataType, count: usize) -> Result<ArrayRef> {
    macro_rules! values {
        ($ty:ty) => {{
            let mut values = Vec::<$ty>::with_capacity(count);
            for _ in 0..count {
                values.push(read(input)?);
            }
            values
        }};
    }
    Ok(match kind {
        Kind::Bool => Arc::new(BooleanArray::from(values!(bool))),
        Kind::Int32 | Kind::Sint32 | Kind::Sfixed32 | Kind::Enum(_) => {
            Arc::new(Int64Array::from_iter_values(values!(i32).into_iter().map(i64::from)))
        }
        Kind::Int64 | Kind::Sint64 | Kind::Sfixed64 => Arc::new(Int64Array::from(values!(i64))),
        Kind::Uint32 | Kind::Fixed32 => Arc::new(UInt64Array::from_iter_values(values!(u32).into_iter().map(u64::from))),
        Kind::Uint64 | Kind::Fixed64 => Arc::new(UInt64Array::from(values!(u64))),
        Kind::Float => Arc::new(Float64Array::from_iter_values(values!(f32).into_iter().map(f64::from))),
        Kind::Double => Arc::new(Float64Array::from(values!(f64))),
        Kind::String => {
            let mut values = StringBuilder::with_capacity(count, 0);
            for _ in 0..count {
                values.append_value(read::<&str>(input)?);
            }
            Arc::new(values.finish())
        }
        Kind::Bytes => {
            let mut values = BinaryBuilder::with_capacity(count, 0);
            for _ in 0..count {
                let length: usize = read(input)?;
                values.append_value(input.take(length).map_err(|_| layout())?);
            }
            Arc::new(values.finish())
        }
        // Nested Messages Are Rare in Interval Rows; They Reuse the Descriptor Column Builder.
        Kind::Message(descriptor) => {
            let rows = (0..count).map(|_| Ok(Some(Value::Message(postcard_message(input, descriptor, 1)?)))).collect::<Result<Vec<_>>>()?;
            column(arrow, &rows)?
        }
    })
}

/// Decode One Postcard Message the Way the Host Encodes Stream Rows: Fields in Declaration Order,
/// a Presence Byte Before Each Singular Field That Tracks Presence, Lengths Before Collections.
fn postcard_message(input: &mut Reader<'_>, descriptor: &MessageDescriptor, depth: usize) -> Result<DynamicMessage> {
    if depth > harmos_domain::schema::MAX_DEPTH {
        return Err(layout());
    }
    let mut message = DynamicMessage::new(descriptor.clone());
    for field in descriptor.fields() {
        if field.is_map() {
            let Kind::Message(entry) = field.kind() else { return Err(layout()) };
            let (key, item) = (entry.map_entry_key_field(), entry.map_entry_value_field());
            let length: usize = read(input)?;
            let mut entries = HashMap::with_capacity(length.min(input.remaining().len()));
            for _ in 0..length {
                let key = match postcard_value(input, &key.kind(), depth)? {
                    Value::Bool(value) => MapKey::Bool(value),
                    Value::I32(value) => MapKey::I32(value),
                    Value::I64(value) => MapKey::I64(value),
                    Value::U32(value) => MapKey::U32(value),
                    Value::U64(value) => MapKey::U64(value),
                    Value::String(value) => MapKey::String(value),
                    _ => return Err(layout()),
                };
                if item.supports_presence() && read::<u8>(input)? != 1 {
                    return Err(layout());
                }
                entries.insert(key, postcard_value(input, &item.kind(), depth + 1)?);
            }
            message.set_field(&field, Value::Map(entries));
        } else if field.is_list() {
            let length: usize = read(input)?;
            let values = (0..length.min(input.remaining().len()))
                .map(|_| postcard_value(input, &field.kind(), depth))
                .collect::<Result<Vec<_>>>()?;
            if values.len() != length {
                return Err(layout());
            }
            message.set_field(&field, Value::List(values));
        } else {
            if field.supports_presence() && read::<u8>(input)? == 0 {
                continue;
            }
            message.set_field(&field, postcard_value(input, &field.kind(), depth)?);
        }
    }
    Ok(message)
}

fn postcard_value(input: &mut Reader<'_>, kind: &Kind, depth: usize) -> Result<Value> {
    Ok(match kind {
        Kind::Bool => Value::Bool(read(input)?),
        Kind::Int32 | Kind::Sint32 | Kind::Sfixed32 => Value::I32(read(input)?),
        Kind::Enum(_) => Value::EnumNumber(read(input)?),
        Kind::Int64 | Kind::Sint64 | Kind::Sfixed64 => Value::I64(read(input)?),
        Kind::Uint32 | Kind::Fixed32 => Value::U32(read(input)?),
        Kind::Uint64 | Kind::Fixed64 => Value::U64(read(input)?),
        Kind::Float => Value::F32(read(input)?),
        Kind::Double => Value::F64(read(input)?),
        Kind::String => Value::String(read::<&str>(input)?.into()),
        Kind::Bytes => {
            let length: usize = read(input)?;
            Value::Bytes(input.take(length).map_err(|_| layout())?.to_vec().into())
        }
        Kind::Message(descriptor) => Value::Message(postcard_message(input, descriptor, depth + 1)?),
    })
}

fn message_fields(descriptor: &MessageDescriptor, active: &mut Vec<String>) -> Result<Fields> {
    if active.iter().any(|name| name == descriptor.full_name()) {
        return Err(Error::Data("use a nonrecursive stream row descriptor"));
    }
    active.push(descriptor.full_name().into());
    let result = descriptor
        .fields()
        .map(|field| {
            let kind = match field.kind() {
                Kind::Bool => DataType::Boolean,
                Kind::Int32 | Kind::Sint32 | Kind::Sfixed32 | Kind::Int64 | Kind::Sint64 | Kind::Sfixed64 | Kind::Enum(_) => {
                    DataType::Int64
                }
                Kind::Uint32 | Kind::Fixed32 | Kind::Uint64 | Kind::Fixed64 => DataType::UInt64,
                Kind::Float | Kind::Double => DataType::Float64,
                Kind::String => DataType::Utf8,
                Kind::Bytes => DataType::Binary,
                Kind::Message(descriptor) => DataType::Struct(message_fields(&descriptor, active)?),
            };
            let kind = if field.is_list() || field.is_map() { DataType::List(Arc::new(Field::new("item", kind, false))) } else { kind };
            Ok(Arc::new(Field::new(field.name(), kind, field.supports_presence())))
        })
        .collect::<Result<Fields>>();
    active.pop();
    result
}
fn column(kind: &DataType, values: &[Option<Value>]) -> Result<ArrayRef> {
    macro_rules! primitive {($array:ident,$($variant:ident => $convert:expr),+) => {
        Arc::new($array::from(values.iter().map(|v| match v {$(Some(Value::$variant(x))=>Some(($convert)(*x)),)+ _=>None}).collect::<Vec<_>>())) as ArrayRef
    };}
    Ok(match kind {
        DataType::Boolean => primitive!(BooleanArray,Bool=>|x|x),
        DataType::Int64 => primitive!(Int64Array,I64=>|x|x,I32=>i64::from,EnumNumber=>i64::from),
        DataType::UInt64 => primitive!(UInt64Array,U64=>|x|x,U32=>u64::from),
        DataType::Float64 => primitive!(Float64Array,F64=>|x|x,F32=>f64::from),
        DataType::Utf8 => Arc::new(StringArray::from(
            values
                .iter()
                .map(|v| match v {
                    Some(Value::String(x)) => Some(x.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>(),
        )),
        DataType::Binary => Arc::new(BinaryArray::from(
            values
                .iter()
                .map(|v| match v {
                    Some(Value::Bytes(x)) => Some(x.as_ref()),
                    _ => None,
                })
                .collect::<Vec<_>>(),
        )),
        DataType::Struct(fields) => {
            let mut columns = Vec::new();
            for field in fields {
                let child = values
                    .iter()
                    .map(|v| match v {
                        Some(Value::Message(m)) => m.descriptor().get_field_by_name(field.name()).and_then(|f| {
                            if f.supports_presence() && !m.has_field(&f) {
                                return None;
                            }
                            let value = m.get_field(&f).into_owned();
                            if let Value::Map(map) = value {
                                let Kind::Message(descriptor) = f.kind() else { unreachable!() };
                                let mut entries = map.into_iter().collect::<Vec<_>>();
                                entries.sort_by(|a, b| a.0.cmp(&b.0));
                                return Some(Value::List(
                                    entries
                                        .into_iter()
                                        .map(|(key, value)| {
                                            let mut entry = DynamicMessage::new(descriptor.clone());
                                            let key = match key {
                                                MapKey::Bool(v) => Value::Bool(v),
                                                MapKey::I32(v) => Value::I32(v),
                                                MapKey::I64(v) => Value::I64(v),
                                                MapKey::U32(v) => Value::U32(v),
                                                MapKey::U64(v) => Value::U64(v),
                                                MapKey::String(v) => Value::String(v),
                                            };
                                            entry.set_field_by_name("key", key);
                                            entry.set_field_by_name("value", value);
                                            Value::Message(entry)
                                        })
                                        .collect(),
                                ));
                            }
                            Some(value)
                        }),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                columns.push(column(field.data_type(), &child)?);
            }
            let nulls = Some(NullBuffer::from(values.iter().map(Option::is_some).collect::<Vec<_>>()));
            if fields.is_empty() {
                Arc::new(StructArray::new_empty_fields(values.len(), nulls))
            } else {
                Arc::new(StructArray::new(fields.clone(), columns, nulls))
            }
        }
        DataType::List(item) => {
            let mut offsets = vec![0i32];
            let mut child = Vec::new();
            for v in values {
                if let Some(Value::List(items)) = v {
                    child.extend(items.iter().cloned().map(Some));
                }
                offsets.push(i32::try_from(child.len()).map_err(|_| Error::Data("reduce the sample list size"))?);
            }
            Arc::new(ListArray::new(
                item.clone(),
                OffsetBuffer::new(ScalarBuffer::from(offsets)),
                column(item.data_type(), &child)?,
                Some(NullBuffer::from(values.iter().map(Option::is_some).collect::<Vec<_>>())),
            ))
        }
        _ => return Err(Error::Corrupt("unsupported Arrow column")),
    })
}
fn message(descriptor: &MessageDescriptor, array: &StructArray, row: usize) -> Result<DynamicMessage> {
    let mut decoded = DynamicMessage::new(descriptor.clone());
    for field in descriptor.fields() {
        let Some(array) = array.column_by_name(field.name()) else { continue };
        if array.is_null(row) {
            continue;
        }
        if field.is_map() {
            let list = array.as_any().downcast_ref::<ListArray>().ok_or(Error::Corrupt("map entry list differs"))?.value(row);
            let Kind::Message(descriptor) = field.kind() else { unreachable!() };
            let mut entries = std::collections::HashMap::new();
            for index in 0..list.len() {
                let entry =
                    message(&descriptor, list.as_any().downcast_ref::<StructArray>().ok_or(Error::Corrupt("map entry differs"))?, index)?;
                let key = match entry.get_field_by_name("key").unwrap().into_owned() {
                    Value::Bool(v) => MapKey::Bool(v),
                    Value::I32(v) => MapKey::I32(v),
                    Value::I64(v) => MapKey::I64(v),
                    Value::U32(v) => MapKey::U32(v),
                    Value::U64(v) => MapKey::U64(v),
                    Value::String(v) => MapKey::String(v),
                    _ => return Err(Error::Corrupt("map key differs")),
                };
                entries.insert(key, entry.get_field_by_name("value").unwrap().into_owned());
            }
            decoded.set_field(&field, Value::Map(entries));
        } else if field.is_list() {
            let list = array.as_any().downcast_ref::<ListArray>().ok_or(Error::Corrupt("list type differs"))?.value(row);
            let values = (0..list.len()).map(|index| value(&field.kind(), &list, index)).collect::<Result<Vec<_>>>()?;
            decoded.set_field(&field, Value::List(values));
        } else {
            decoded.set_field(&field, value(&field.kind(), array, row)?);
        }
    }
    Ok(decoded)
}
fn value(kind: &Kind, array: &ArrayRef, row: usize) -> Result<Value> {
    macro_rules! get {
        ($ty:ty) => {
            array.as_any().downcast_ref::<$ty>().ok_or(Error::Corrupt("column type differs"))?.value(row)
        };
    }
    Ok(match kind {
        Kind::Bool => Value::Bool(get!(BooleanArray)),
        Kind::Int32 | Kind::Sint32 | Kind::Sfixed32 => Value::I32(get!(Int64Array) as i32),
        Kind::Int64 | Kind::Sint64 | Kind::Sfixed64 => Value::I64(get!(Int64Array)),
        Kind::Enum(_) => Value::EnumNumber(get!(Int64Array) as i32),
        Kind::Uint32 | Kind::Fixed32 => Value::U32(get!(UInt64Array) as u32),
        Kind::Uint64 | Kind::Fixed64 => Value::U64(get!(UInt64Array)),
        Kind::Float => Value::F32(get!(Float64Array) as f32),
        Kind::Double => Value::F64(get!(Float64Array)),
        Kind::String => Value::String(get!(StringArray).into()),
        Kind::Bytes => Value::Bytes(get!(BinaryArray).to_vec().into()),
        Kind::Message(descriptor) => Value::Message(message(
            descriptor,
            array.as_any().downcast_ref::<StructArray>().ok_or(Error::Corrupt("struct type differs"))?,
            row,
        )?),
    })
}

/* === Tests === */
