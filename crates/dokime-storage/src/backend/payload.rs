/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Reconstruct Native Postcard Values From Their Sole Arrow Representation

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::{BackendFailure, Error, Result, invalid};
use arrow_array::{
    Array, ArrayRef, BinaryArray, BooleanArray, Float32Array, Float64Array, Int32Array, Int64Array, ListArray, StringArray, StructArray,
    UInt32Array, UInt64Array,
};
use harmos_domain::schema::{
    Cardinality,
    graph::{Kind, Schema},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn encode(schema: &Schema, value: &StructArray, row: usize) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    message(schema, schema.root, value, row, &mut output)?;
    Ok(output)
}

/* === Private Functions === */
fn append<T: serde::Serialize + ?Sized>(value: &T, output: &mut Vec<u8>) -> Result<()> {
    let bytes =
        postcard::to_allocvec(value).map_err(|source| Error::backend("Encode Native Arrow Value", BackendFailure::Corrupt, source))?;
    output.extend(bytes);
    Ok(())
}

fn message(schema: &Schema, definition: usize, value: &StructArray, row: usize, output: &mut Vec<u8>) -> Result<()> {
    let fields = schema.definitions.get(definition).ok_or_else(|| invalid("Read Native Arrow: Restore the Schema Definition"))?;
    if fields.first().is_some_and(|field| field.cardinality == Cardinality::Variant) {
        let variants = value
            .column_by_name("__variant")
            .and_then(|array| array.as_any().downcast_ref::<UInt32Array>())
            .ok_or_else(|| invalid("Read Native Arrow: Restore the Variant Index"))?;
        let index = variants.value(row);
        append(&index, output)?;
        let field = fields.get(index as usize).ok_or_else(|| invalid("Read Native Arrow: Restore the Variant"))?;
        let array = value.column_by_name(&field.name).ok_or_else(|| invalid("Read Native Arrow: Restore the Variant Column"))?;
        return scalar(schema, field.kind, array, row, output);
    }
    for field in fields {
        let array = value.column_by_name(&field.name).ok_or_else(|| invalid("Read Native Arrow: Restore the Field Column"))?;
        match field.cardinality {
            Cardinality::Optional => {
                let present = !array.is_null(row);
                append(&present, output)?;
                if present {
                    scalar(schema, field.kind, array, row, output)?;
                }
            }
            Cardinality::Repeated => {
                let list =
                    array.as_any().downcast_ref::<ListArray>().ok_or_else(|| invalid("Read Native Arrow: Restore the List Column"))?;
                let values = list.value(row);
                append(&values.len(), output)?;
                for index in 0..values.len() {
                    scalar(schema, field.kind, &values, index, output)?;
                }
            }
            Cardinality::Required | Cardinality::Variant => scalar(schema, field.kind, array, row, output)?,
        }
    }
    Ok(())
}

fn scalar(schema: &Schema, kind: Kind, array: &ArrayRef, row: usize, output: &mut Vec<u8>) -> Result<()> {
    if array.is_null(row) {
        return Err(invalid("Read Native Arrow: Restore a Required Value"));
    }
    macro_rules! number {
        ($ty:ty) => {
            append(
                &array
                    .as_any()
                    .downcast_ref::<$ty>()
                    .ok_or_else(|| invalid("Read Native Arrow: Restore the Declared Scalar Type"))?
                    .value(row),
                output,
            )
        };
    }
    match kind {
        Kind::Bool => number!(BooleanArray),
        Kind::U32 => number!(UInt32Array),
        Kind::U64 => number!(UInt64Array),
        Kind::I32 => number!(Int32Array),
        Kind::I64 => number!(Int64Array),
        Kind::F32 => number!(Float32Array),
        Kind::F64 => number!(Float64Array),
        Kind::String => number!(StringArray),
        Kind::Bytes => number!(BinaryArray),
        Kind::Message { definition } => {
            let value =
                array.as_any().downcast_ref::<StructArray>().ok_or_else(|| invalid("Read Native Arrow: Restore the Message Column"))?;
            message(schema, definition, value, row, output)
        }
    }
}

/* === Tests === */
