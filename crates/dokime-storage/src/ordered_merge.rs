/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Two Ordered Arrow Inputs

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use arrow_array::{Array, ArrayRef, Int64Array, RecordBatch, StructArray, new_null_array};
use arrow_schema::{ArrowError, DataType, SchemaRef};
use arrow_select::concat::concat;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

// Each Right Row Is Visited Once; Only The Current Page And Last Match Survive.
pub(crate) fn as_of(
    left: impl Iterator<Item = Result<RecordBatch, ArrowError>>, mut right: impl Iterator<Item = Result<RecordBatch, ArrowError>>,
    right_schema: SchemaRef, mut emit: impl FnMut(RecordBatch, StructArray) -> Result<(), ArrowError>,
) -> Result<(), ArrowError> {
    let mut page = None::<RecordBatch>;
    let mut offset = 0;
    let mut previous = None::<ArrayRef>;
    let mut last_left = i64::MIN;
    let mut last_right = i64::MIN;
    for batch in left {
        let batch = batch?;
        let left_times = times(&batch)?;
        let mut selected = Vec::<ArrayRef>::with_capacity(batch.num_rows());
        for row in 0..batch.num_rows() {
            let time = left_times.value(row);
            if time < last_left {
                return Err(ArrowError::InvalidArgumentError("Order the left input by source time".into()));
            }
            last_left = time;
            loop {
                if page.as_ref().is_none_or(|page| offset == page.num_rows()) {
                    page = right.next().transpose()?;
                    offset = 0;
                    if page.as_ref().is_some_and(|page| page.num_rows() == 0) {
                        continue;
                    }
                }
                let Some(current) = &page else { break };
                let next_time = times(current)?.value(offset);
                if next_time < last_right {
                    return Err(ArrowError::InvalidArgumentError("Order the right input by source time".into()));
                }
                if next_time > time {
                    break;
                }
                last_right = next_time;
                previous = Some(Arc::new(StructArray::from(current.clone()).slice(offset, 1)));
                offset += 1;
            }
            selected.push(previous.clone().unwrap_or_else(|| new_null_array(&DataType::Struct(right_schema.fields().clone()), 1)));
        }
        if !selected.is_empty() {
            let arrays = selected.iter().map(|array| array.as_ref()).collect::<Vec<_>>();
            let aligned = concat(&arrays)?;
            emit(batch, aligned.as_any().downcast_ref::<StructArray>().unwrap().clone())?;
        }
    }
    Ok(())
}

/* === Private Functions === */

fn times(batch: &RecordBatch) -> Result<&Int64Array, ArrowError> {
    batch
        .column_by_name("_storage")
        .and_then(|array| array.as_any().downcast_ref::<StructArray>())
        .and_then(|array| array.column_by_name("source_ns"))
        .and_then(|array| array.as_any().downcast_ref::<Int64Array>())
        .ok_or_else(|| ArrowError::InvalidArgumentError("Supply Int64 _storage.source_ns columns".into()))
}

/* === Tests === */
