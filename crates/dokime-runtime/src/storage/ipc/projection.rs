/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Projected Arrow Telemetry Batch Decoding

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::HashMap, sync::Arc};

// Third Party Imports
use arrow_array::{RecordBatch, StructArray};
use arrow_buffer::Buffer;
use arrow_schema::{DataType, Fields, Schema, SchemaRef};

// Local Imports
use super::BatchIndex;
use crate::storage::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct Projection {
    schema: SchemaRef,
    flat_schema: SchemaRef,
    columns: Vec<usize>,
    storage_nodes: usize,
    storage_buffers: usize,
    storage: Fields,
    values: Fields,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Projection {
    pub(crate) fn new(schema: SchemaRef, metadata: &[&str], keys: &[String]) -> Result<Self> {
        let [storage, values] = schema.fields().as_ref() else {
            return Err(Error::Corrupt("telemetry roots differ"));
        };
        let (DataType::Struct(storage_fields), DataType::Struct(value_fields)) = (storage.data_type(), values.data_type()) else {
            return Err(Error::Corrupt("telemetry roots are not structs"));
        };
        // Storage Children Are Scalars In The Existing File Format. Only The Two
        // Non-Null Root Nodes And Their Validity Buffers Are Removed From The Read View.
        let storage_nodes = storage_fields.len() + 1;
        let mut storage_buffers = 1;
        for field in storage_fields {
            storage_buffers += match field.data_type() {
                DataType::Utf8 => 3,
                DataType::UInt64 | DataType::Timestamp(_, _) => 2,
                _ => return Err(Error::Corrupt("telemetry storage metadata is not scalar")),
            };
        }
        let selected_storage =
            storage_fields.iter().enumerate().filter(|(_, field)| metadata.contains(&field.name().as_str())).collect::<Vec<_>>();
        let selected_values = value_fields.iter().enumerate().filter(|(_, field)| keys.contains(field.name())).collect::<Vec<_>>();
        let columns = selected_storage
            .iter()
            .map(|(index, _)| *index)
            .chain(selected_values.iter().map(|(index, _)| storage_fields.len() + index))
            .collect();
        let selected_storage: Fields = selected_storage.into_iter().map(|(_, field)| field.clone()).collect();
        let selected_values: Fields = selected_values.into_iter().map(|(_, field)| field.clone()).collect();
        let flat_fields = storage_fields
            .iter()
            .chain(value_fields)
            .enumerate()
            .map(|(index, field)| field.as_ref().clone().with_name(index.to_string()))
            .collect::<Vec<_>>();
        let flat_schema = Arc::new(Schema::new(flat_fields));
        let fields = vec![
            storage.as_ref().clone().with_data_type(DataType::Struct(selected_storage.clone())),
            values.as_ref().clone().with_data_type(DataType::Struct(selected_values.clone())),
        ];
        let schema = Arc::new(Schema::new_with_metadata(fields, schema.metadata().clone()));
        Ok(Self {
            schema,
            flat_schema,
            columns,
            storage_nodes,
            storage_buffers,
            storage: selected_storage,
            values: selected_values,
        })
    }

    pub(crate) fn columns(&self) -> &[usize] {
        &self.columns
    }

    pub(crate) fn decode(&self, buffer: &Buffer, index: &BatchIndex) -> Result<RecordBatch> {
        let metadata = index.metadata_bytes as usize;
        if metadata < 8
            || metadata > buffer.len()
            || buffer[..4] != [255; 4]
            || u32::from_le_bytes(buffer[4..8].try_into().unwrap()) != index.metadata_bytes - 8
        {
            return Err(Error::Corrupt("IPC batch metadata length differs"));
        }
        let message = arrow_ipc::root_as_message(&buffer[8..metadata]).map_err(|_| Error::Corrupt("IPC batch header is invalid"))?;
        let batch = message.header_as_record_batch().ok_or(Error::Corrupt("IPC block is not a record batch"))?;
        if message.bodyLength() != index.body_bytes as i64 || batch.length() != index.rows as i64 {
            return Err(Error::Corrupt("IPC batch dimensions differ"));
        }
        let nodes = batch.nodes().ok_or(Error::Corrupt("IPC field nodes are absent"))?;
        let buffers = batch.buffers().ok_or(Error::Corrupt("IPC buffers are absent"))?;
        if nodes.len() <= self.storage_nodes || buffers.len() <= self.storage_buffers {
            return Err(Error::Corrupt("IPC root nodes are absent"));
        }
        for node in [nodes.get(0), nodes.get(self.storage_nodes)] {
            if node.null_count() != 0 || node.length() != batch.length() {
                return Err(Error::Corrupt("telemetry root has nulls or a differing length"));
            }
        }
        let nodes = nodes
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != 0 && *index != self.storage_nodes)
            .map(|(_, node)| *node)
            .collect::<Vec<_>>();
        let buffers = buffers
            .iter()
            .enumerate()
            .filter(|(index, _)| *index != 0 && *index != self.storage_buffers)
            .map(|(_, buffer)| *buffer)
            .collect::<Vec<_>>();
        let mut builder = flatbuffers::FlatBufferBuilder::new();
        let nodes = builder.create_vector(&nodes);
        let buffers = builder.create_vector(&buffers);
        let compression = batch.compression().map(|compression| {
            arrow_ipc::BodyCompression::create(
                &mut builder,
                &arrow_ipc::BodyCompressionArgs {
                    codec: compression.codec(),
                    method: compression.method(),
                },
            )
        });
        let variadic = batch.variadicBufferCounts().map(|counts| builder.create_vector(&counts.iter().collect::<Vec<_>>()));
        let flattened = arrow_ipc::RecordBatch::create(
            &mut builder,
            &arrow_ipc::RecordBatchArgs {
                length: batch.length(),
                nodes: Some(nodes),
                buffers: Some(buffers),
                compression,
                variadicBufferCounts: variadic,
            },
        );
        builder.finish(flattened, None);
        let flattened = flatbuffers::root::<arrow_ipc::RecordBatch<'_>>(builder.finished_data())
            .map_err(|_| Error::Corrupt("IPC projected header is invalid"))?;
        let body = buffer.slice(metadata);
        // Arrow Skips Unselected Buffers Before Decompression And Validates Selected Arrays.
        let decoded = arrow_ipc::reader::read_record_batch(
            &body,
            flattened,
            self.flat_schema.clone(),
            &HashMap::new(),
            Some(&self.columns),
            &message.version(),
        )?;
        let structure = |fields: &Fields, columns: &[arrow_array::ArrayRef]| {
            if fields.is_empty() {
                StructArray::new_empty_fields(decoded.num_rows(), None)
            } else {
                StructArray::new(fields.clone(), columns.to_vec(), None)
            }
        };
        let (storage, values) = decoded.columns().split_at(self.storage.len());
        let columns = vec![Arc::new(structure(&self.storage, storage)) as _, Arc::new(structure(&self.values, values)) as _];
        Ok(RecordBatch::try_new(self.schema.clone(), columns)?)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use arrow_array::{ArrayRef, Float64Array, Int64Array, ListArray, StringArray, TimestampMicrosecondArray, UInt64Array};
    use arrow_schema::Field;

    fn fixture() -> (RecordBatch, Buffer, BatchIndex) {
        let field = |name: &str, array: ArrayRef| (Arc::new(Field::new(name, array.data_type().clone(), true)), array);
        let storage = StructArray::from(vec![
            field("position", Arc::new(UInt64Array::from(vec![1, 2, 3, 4]))),
            field("at", Arc::new(TimestampMicrosecondArray::from(vec![-1, 0, 1, 2]).with_timezone("UTC"))),
            field("sequence", Arc::new(UInt64Array::from(vec![7, 5, 9, 11]))),
            field(
                "timestamp",
                Arc::new(StringArray::from(vec![
                    "1969-12-31T23:59:59.999999999Z",
                    "1970-01-01T01:00:00+01:00",
                    "1970-01-01T00:00:00.000001001Z",
                    "1970-01-01T00:00:00.000002002Z",
                ])),
            ),
            field("scope", Arc::new(StringArray::from(vec!["null"; 4]))),
            field("sample_id", Arc::new(StringArray::from(vec!["a", "b", "c", "d"]))),
            field("native_metadata", Arc::new(StringArray::from(vec!["unselected"; 4]))),
        ]);
        let list = ListArray::from_iter_primitive::<arrow_array::types::Int64Type, _, _>(vec![
            Some(vec![Some(1), None]),
            None,
            Some(Vec::new()),
            Some(vec![Some(-5)]),
        ]);
        let values = StructArray::from(vec![
            field("list", Arc::new(list)),
            field("number", Arc::new(Float64Array::from(vec![Some(3.25), None, Some(-0.0), Some(-7.0)]))),
            field("integer", Arc::new(Int64Array::from(vec![i64::MIN, -3, 2, i64::MAX]))),
            field("unsigned", Arc::new(UInt64Array::from(vec![0, u64::MAX, 9_007_199_254_740_993, 17]))),
            field("timestamp", Arc::new(Float64Array::from(vec![1.0, 2.0, 3.0, 4.0]))),
            field("text", Arc::new(StringArray::from(vec!["compressible unselected payload ".repeat(2048); 4]))),
        ]);
        let columns: Vec<(&str, ArrayRef)> = vec![("_storage", Arc::new(storage)), ("values", Arc::new(values))];
        let batch = RecordBatch::try_from_iter(columns).unwrap();
        let options = arrow_ipc::writer::IpcWriteOptions::default().try_with_compression(Some(arrow_ipc::CompressionType::ZSTD)).unwrap();
        let mut writer = arrow_ipc::writer::FileWriter::try_new_with_options(Vec::new(), batch.schema().as_ref(), options).unwrap();
        writer.write(&batch).unwrap();
        writer.finish().unwrap();
        let bytes = writer.into_inner().unwrap();
        let footer_length = i32::from_le_bytes(bytes[bytes.len() - 10..bytes.len() - 6].try_into().unwrap()) as usize;
        let footer = arrow_ipc::root_as_footer(&bytes[bytes.len() - 10 - footer_length..bytes.len() - 10]).unwrap();
        let block = footer.recordBatches().unwrap().get(0);
        let index = BatchIndex {
            offset: block.offset() as u64,
            metadata_bytes: block.metaDataLength() as u32,
            body_bytes: block.bodyLength() as u64,
            rows: batch.num_rows() as u64,
            sha256: [0; 32],
        };
        let buffer = Buffer::from_vec(bytes[index.offset as usize..index.end().unwrap() as usize].to_vec());
        (batch, buffer, index)
    }

    #[test]
    fn projected_columns_match_full_decode_without_decompressing_unselected_values() {
        let (batch, buffer, index) = fixture();
        let decoded = arrow_ipc::reader::FileDecoder::new(batch.schema(), arrow_ipc::MetadataVersion::V5)
            .read_record_batch(&index.block().unwrap(), &buffer)
            .unwrap()
            .unwrap();
        assert_eq!(decoded, batch);
        let original = batch.column(1).as_any().downcast_ref::<StructArray>().unwrap();
        let keys = ["unsigned", "timestamp", "integer", "number"].map(str::to_owned);
        let metadata_keys = ["position", "at", "sequence", "timestamp", "scope", "sample_id"];
        let projection = Projection::new(batch.schema(), &metadata_keys, &keys).unwrap();
        let selected = projection.decode(&buffer, &index).unwrap();
        let metadata = selected.column(0).as_any().downcast_ref::<StructArray>().unwrap();
        let expected = batch.column(0).as_any().downcast_ref::<StructArray>().unwrap();
        for field in metadata.fields() {
            assert_eq!(metadata.column_by_name(field.name()), expected.column_by_name(field.name()));
        }
        let values = selected.column(1).as_any().downcast_ref::<StructArray>().unwrap();
        for key in &keys {
            assert_eq!(values.column_by_name(key), original.column_by_name(key));
        }
        assert!(values.column_by_name("list").is_none());
        let missing = Projection::new(batch.schema(), &metadata_keys, &["missing".into()]).unwrap().decode(&buffer, &index).unwrap();
        assert_eq!(missing.num_rows(), batch.num_rows());
        assert!(missing.column(1).as_any().downcast_ref::<StructArray>().unwrap().columns().is_empty());

        // A Broken Unselected ZSTD Frame Is Never Decoded; The Full Decoder Rejects It.
        let message = arrow_ipc::root_as_message(&buffer[8..index.metadata_bytes as usize]).unwrap();
        let buffers = message.header_as_record_batch().unwrap().buffers().unwrap();
        let text = buffers.get(buffers.len() - 1);
        let offset = index.metadata_bytes as usize + text.offset() as usize + 8;
        let mut broken = buffer.to_vec();
        broken[offset] ^= 255;
        let broken = Buffer::from_vec(broken);
        assert!(
            arrow_ipc::reader::FileDecoder::new(batch.schema(), message.version())
                .read_record_batch(&index.block().unwrap(), &broken)
                .is_err()
        );
        assert_eq!(projection.decode(&broken, &index).unwrap(), selected);
    }
}
