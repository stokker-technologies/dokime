/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{Arc, OnceLock};

// Third Party Imports
use arrow_array::{BinaryArray, Int64Array, RecordBatch, UInt64Array};
use arrow_ipc::writer::FileWriter;
use arrow_schema::{DataType, Field, Schema};

// Local Imports
use crate::{Error, Result, Sample};
use prost::Message;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn encode_batch(samples: &[Sample]) -> Result<Vec<u8>> {
    static SCHEMA: OnceLock<Arc<Schema>> = OnceLock::new();
    let schema = SCHEMA
        .get_or_init(|| {
            Arc::new(Schema::new(vec![
                Field::new("position", DataType::UInt64, false),
                Field::new("source_ns", DataType::Int64, false),
                Field::new("receipt_ns", DataType::Int64, false),
                Field::new("sequence", DataType::UInt64, false),
                Field::new("row", DataType::Binary, false),
            ]))
        })
        .clone();
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from_iter_values(samples.iter().map(|sample| sample.position))),
            Arc::new(Int64Array::from_iter_values(samples.iter().map(|sample| sample.source_ns))),
            Arc::new(Int64Array::from_iter_values(samples.iter().map(|sample| sample.receipt_ns))),
            Arc::new(UInt64Array::from_iter_values(samples.iter().map(|sample| sample.sequence))),
            Arc::new(BinaryArray::from_iter_values(samples.iter().map(|sample| sample.row.as_slice()))),
        ],
    )?;
    let mut bytes = Vec::new();
    let mut writer = FileWriter::try_new(&mut bytes, &schema)?;
    writer.write(&batch)?;
    writer.finish()?;
    drop(writer);
    Ok(bytes)
}
pub(crate) fn decode(stream: String, bytes: Vec<u8>) -> Result<Sample> {
    let sample = Sample::decode(bytes.as_slice())?;
    if sample.stream != stream {
        return Err(Error::Invalid("Read Samples: Restore a Valid Backup of the Stream Index".into()));
    }
    Ok(sample)
}

/* === Private Functions === */

/* === Tests === */
