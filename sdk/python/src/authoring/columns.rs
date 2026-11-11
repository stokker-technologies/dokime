/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Column Batch Publication

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, num::NonZeroUsize, sync::Arc};

// Third Party Imports
use arrow_array::{
    Array, ArrayRef, BinaryArray, BooleanArray, Float32Array, Float64Array, Int32Array, Int64Array, StringArray, StructArray, UInt32Array,
    UInt64Array,
};
use arrow_schema::Field;
use pyo3::{
    buffer::{Element, PyBuffer},
    exceptions::PyValueError,
    prelude::*,
    types::PyDict,
};

// Local Imports
use super::schema::Shape;
use dokime::harmos::schema::{Cardinality, graph::Kind};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub(super) type Rows = Vec<(BTreeMap<String, dokime::Value>, Vec<u8>)>;

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn snapshot(py: Python<'_>, shape: &Shape, columns: &Bound<'_, PyDict>) -> PyResult<StructArray> {
    let fields = &shape.schema.definitions[shape.schema.root];
    if fields.len() != columns.len() {
        return Err(PyValueError::new_err("columns must match the declared row fields"));
    }
    let mut length = None;
    let mut arrays = Vec::new();
    for field in fields {
        if matches!(field.cardinality, Cardinality::Repeated | Cardinality::Variant) {
            return Err(PyValueError::new_err("batch columns require scalar row fields"));
        }
        let column =
            columns.get_item(field.name.as_ref())?.ok_or_else(|| PyValueError::new_err(format!("missing column {}", field.name)))?;
        let column = if column.hasattr("null_count")? && column.getattr("null_count")?.extract::<usize>()? != 0 {
            column.call_method0("to_pylist")?
        } else if column.hasattr("to_numpy")? {
            let keywords = PyDict::new(py);
            keywords.set_item("zero_copy_only", false)?;
            column.call_method("to_numpy", (), Some(&keywords))?
        } else {
            column
        };
        let array: ArrayRef = match field.kind {
            Kind::Bool => Arc::new(BooleanArray::from(sequence::<bool>(&column)?)),
            Kind::I32 => Arc::new(Int32Array::from(numeric::<i32>(py, &column)?)),
            Kind::I64 => Arc::new(Int64Array::from(numeric::<i64>(py, &column)?)),
            Kind::U32 => Arc::new(UInt32Array::from(numeric::<u32>(py, &column)?)),
            Kind::U64 => Arc::new(UInt64Array::from(numeric::<u64>(py, &column)?)),
            Kind::F32 => Arc::new(Float32Array::from(numeric::<f32>(py, &column)?)),
            Kind::F64 => Arc::new(Float64Array::from(numeric::<f64>(py, &column)?)),
            Kind::String => Arc::new(StringArray::from(sequence::<String>(&column)?)),
            Kind::Bytes => {
                let values = sequence::<Vec<u8>>(&column)?;
                Arc::new(BinaryArray::from_iter(values.iter().map(|value| value.as_deref())))
            }
            Kind::Message { .. } => return Err(PyValueError::new_err("batch columns require scalar row fields")),
        };
        if length.is_some_and(|length| length != array.len()) {
            return Err(PyValueError::new_err("columns have unequal lengths"));
        }
        length = Some(array.len());
        if field.cardinality == Cardinality::Required && array.null_count() != 0 {
            return Err(PyValueError::new_err(format!("required column {} contains nulls", field.name)));
        }
        arrays.push((
            Arc::new(Field::new(field.name.as_ref(), array.data_type().clone(), field.cardinality == Cardinality::Optional)),
            array,
        ));
    }
    if arrays.is_empty() {
        return Err(PyValueError::new_err("a column batch needs at least one field"));
    }
    Ok(StructArray::from(arrays))
}

pub(super) fn encode(shape: &Shape, columns: StructArray) -> Result<Rows, String> {
    let bytes = columns.get_array_memory_size().saturating_mul(4).saturating_add(columns.len().saturating_mul(64));
    let allowance = NonZeroUsize::new(bytes.max(1)).expect("positive column allowance");
    let payloads =
        dokime::harmos::storage::arrow::encode_payloads(&shape.schema, &columns, allowance).map_err(|error| error.to_string())?;
    rows(shape, payloads)
}

pub(super) fn rows(shape: &Shape, payloads: Vec<Vec<u8>>) -> Result<Rows, String> {
    payloads
        .into_iter()
        .map(|payload| {
            let value = shape.decode(&payload).map_err(|error| error.to_string())?;
            let values = value
                .as_object()
                .ok_or("row is not an object")?
                .iter()
                .map(|(key, value)| {
                    dokime::Value::from_plain_json(value.clone()).map(|value| (key.clone(), value)).map_err(|error| error.to_string())
                })
                .collect::<Result<BTreeMap<_, _>, _>>()?;
            Ok((values, payload))
        })
        .collect()
}

/* === Private Functions === */

fn sequence<T>(column: &Bound<'_, PyAny>) -> PyResult<Vec<Option<T>>>
where
    for<'a, 'py> T: FromPyObject<'a, 'py>,
{
    if column.hasattr("tolist")? {
        column.call_method0("tolist")?.extract()
    } else if column.hasattr("to_pylist")? {
        column.call_method0("to_pylist")?.extract()
    } else {
        column.extract()
    }
}

fn numeric<T>(py: Python<'_>, column: &Bound<'_, PyAny>) -> PyResult<Vec<Option<T>>>
where
    T: Element + Copy,
    for<'a, 'py> T: FromPyObject<'a, 'py>,
{
    if let Ok(buffer) = PyBuffer::<T>::get(column) {
        if buffer.dimensions() != 1 {
            return Err(PyValueError::new_err("columns must be one-dimensional"));
        }
        return buffer.to_vec(py).map(|values| values.into_iter().map(Some).collect());
    }
    sequence(column)
}

/* === Tests === */
