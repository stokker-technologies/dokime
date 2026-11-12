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
use std::{io::Cursor, sync::Arc};

// Third Party Imports
use arrow_array::{Array, BinaryArray, Int64Array, RecordBatch, UInt64Array};
use arrow_ipc::{reader::FileReader, writer::FileWriter};
use arrow_schema::{DataType, Field, Schema};

// Local Imports
use crate::{Error, Result, Sample};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn encode(sample: &Sample) -> Result<Vec<u8>> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("position", DataType::UInt64, false),
        Field::new("source_ns", DataType::Int64, false),
        Field::new("receipt_ns", DataType::Int64, false),
        Field::new("sequence", DataType::UInt64, false),
        Field::new("row", DataType::Binary, false),
    ]));
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(UInt64Array::from(vec![sample.position])),
            Arc::new(Int64Array::from(vec![sample.source_ns])),
            Arc::new(Int64Array::from(vec![sample.receipt_ns])),
            Arc::new(UInt64Array::from(vec![sample.sequence])),
            Arc::new(BinaryArray::from_vec(vec![sample.row.as_slice()])),
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
    let mut reader = FileReader::try_new(Cursor::new(bytes), None)?;
    let batch = reader.next().ok_or_else(|| Error::Invalid("Restore the Missing Arrow Evidence Batch".into()))??;
    if batch.num_rows() != 1 || batch.num_columns() != 5 {
        return Err(Error::Invalid("Restore the Invalid Arrow Evidence Batch".into()));
    }
    let unsigned = |column: usize| -> Result<u64> {
        batch
            .column(column)
            .as_any()
            .downcast_ref::<UInt64Array>()
            .map(|array| array.value(0))
            .ok_or_else(|| Error::Invalid("Restore Invalid Arrow Position Columns".into()))
    };
    let signed = |column: usize| -> Result<i64> {
        batch
            .column(column)
            .as_any()
            .downcast_ref::<Int64Array>()
            .map(|array| array.value(0))
            .ok_or_else(|| Error::Invalid("Restore Invalid Arrow Clock Columns".into()))
    };
    let row = batch
        .column(4)
        .as_any()
        .downcast_ref::<BinaryArray>()
        .ok_or_else(|| Error::Invalid("Restore the Invalid Arrow Payload Column".into()))?
        .value(0)
        .to_vec();
    Ok(Sample {
        stream,
        position: unsigned(0)?,
        source_ns: signed(1)?,
        receipt_ns: signed(2)?,
        sequence: unsigned(3)?,
        row,
    })
}

/* === Private Functions === */

/* === Tests === */
