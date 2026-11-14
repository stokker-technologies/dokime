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
use std::sync::Arc;

// Third Party Imports
use arrow_array::{
    Array, ArrayRef, BinaryArray, BooleanArray, Float64Array, Int64Array, ListArray, RecordBatch, StringArray, StructArray, UInt64Array,
};
use arrow_buffer::{NullBuffer, OffsetBuffer, ScalarBuffer};
use arrow_schema::{DataType, Field, Fields, Schema, SchemaRef};
use prost::Message;
use prost_reflect::{DynamicMessage, Kind, MessageDescriptor, ReflectMessage, Value};
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

fn message_fields(descriptor: &MessageDescriptor, active: &mut Vec<String>) -> Result<Fields> {
    if active.iter().any(|name| name == descriptor.full_name()) {
        return Err(Error::Data("use a nonrecursive stream row descriptor"));
    }
    active.push(descriptor.full_name().into());
    let result = descriptor
        .fields()
        .map(|field| {
            if field.is_map() {
                return Err(Error::Data("stream map columns require an explicit entry list"));
            }
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
            let kind = if field.is_list() { DataType::List(Arc::new(Field::new("item", kind, false))) } else { kind };
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
                            if f.supports_presence() && !m.has_field(&f) { None } else { Some(m.get_field(&f).into_owned()) }
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
    let mut message = DynamicMessage::new(descriptor.clone());
    for field in descriptor.fields() {
        let Some(array) = array.column_by_name(field.name()) else { continue };
        if array.is_null(row) {
            continue;
        }
        if field.is_list() {
            let list = array.as_any().downcast_ref::<ListArray>().ok_or(Error::Corrupt("list type differs"))?.value(row);
            let values = (0..list.len()).map(|index| value(&field.kind(), &list, index)).collect::<Result<Vec<_>>>()?;
            message.set_field(&field, Value::List(values));
        } else {
            message.set_field(&field, value(&field.kind(), array, row)?);
        }
    }
    Ok(message)
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
