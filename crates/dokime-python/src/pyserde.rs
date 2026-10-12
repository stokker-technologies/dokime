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

// Third Party Imports
use pyo3::{
    IntoPyObjectExt,
    exceptions::PyValueError,
    prelude::*,
    types::{
        PyAnyMethods, PyBool, PyDict, PyDictMethods, PyFloat, PyInt, PyList, PyListMethods,
        PyString, PyTuple, PyTupleMethods,
    },
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Map, Number, Value};
// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn from_py<T>(payload: &Bound<'_, PyAny>) -> PyResult<T>
where
    T: DeserializeOwned,
{
    serde_json::from_value(py_to_json(payload)?).map_err(crate::to_value_error)
}

pub(crate) fn to_py<T>(py: Python<'_>, value: &T) -> PyResult<Py<PyAny>>
where
    T: Serialize,
{
    json_to_py(
        py,
        serde_json::to_value(value).map_err(crate::to_value_error)?,
    )
}

pub(crate) fn round_trip_py<T>(py: Python<'_>, payload: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>>
where
    T: DeserializeOwned + Serialize,
{
    let value = from_py::<T>(payload)?;

    to_py(py, &value)
}

pub(crate) fn py_to_json(value: &Bound<'_, PyAny>) -> PyResult<Value> {
    if value.is_none() {
        return Ok(Value::Null);
    }

    if value.is_exact_instance_of::<PyBool>() {
        return Ok(Value::Bool(value.extract::<bool>()?));
    }

    if value.is_instance_of::<PyInt>() {
        return value
            .extract::<i64>()
            .map(|integer| Value::Number(Number::from(integer)))
            .or_else(|_| {
                value
                    .extract::<u64>()
                    .map(|integer| Value::Number(Number::from(integer)))
            });
    }

    if value.is_instance_of::<PyFloat>() {
        let number = Number::from_f64(value.extract::<f64>()?)
            .ok_or_else(|| PyValueError::new_err("JSON numbers must be finite"))?;

        return Ok(Value::Number(number));
    }

    if value.is_instance_of::<PyString>() {
        return Ok(Value::String(value.extract::<String>()?));
    }

    if let Ok(dict) = value.cast::<PyDict>() {
        let mut object = Map::new();
        for (key, item) in dict.iter() {
            object.insert(key.extract::<String>()?, py_to_json(&item)?);
        }

        return Ok(Value::Object(object));
    }

    if let Ok(list) = value.cast::<PyList>() {
        let mut items = Vec::with_capacity(list.len());
        for item in list.iter() {
            items.push(py_to_json(&item)?);
        }

        return Ok(Value::Array(items));
    }

    if let Ok(tuple) = value.cast::<PyTuple>() {
        let mut items = Vec::with_capacity(tuple.len());
        for item in tuple.iter() {
            items.push(py_to_json(&item)?);
        }

        return Ok(Value::Array(items));
    }

    let type_name = value.get_type().name()?;

    Err(PyValueError::new_err(format!(
        "cannot convert Python {type_name} to JSON-compatible contract value"
    )))
}

pub(crate) fn value_from_py(value: &Bound<'_, PyAny>) -> PyResult<dokime_app::core::Value> {
    use dokime_app::core::Value as DomainValue;
    if value.is_none() {
        return Ok(DomainValue::Null);
    }
    if value.is_exact_instance_of::<PyBool>() {
        return Ok(DomainValue::Bool(value.extract()?));
    }
    if value.is_instance_of::<PyInt>() {
        if value
            .getattr("__dokime_unsigned__")
            .is_ok_and(|flag| flag.is_truthy().unwrap_or(false))
        {
            return value.extract::<u64>().map(DomainValue::Unsigned);
        }
        return value
            .extract::<i64>()
            .map(DomainValue::Integer)
            .or_else(|_| value.extract::<u64>().map(DomainValue::Unsigned));
    }
    if value.is_exact_instance_of::<PyFloat>() {
        return Ok(DomainValue::Number(value.extract()?));
    }
    if value.is_exact_instance_of::<PyString>() {
        return Ok(DomainValue::Text(value.extract()?));
    }
    if let Ok(dict) = value.cast::<PyDict>() {
        return dict
            .iter()
            .map(|(key, item)| Ok((key.extract::<String>()?, value_from_py(&item)?)))
            .collect::<PyResult<_>>()
            .map(DomainValue::Object);
    }
    if let Ok(list) = value.cast::<PyList>() {
        return list
            .iter()
            .map(|item| value_from_py(&item))
            .collect::<PyResult<_>>()
            .map(DomainValue::Array);
    }
    if let Ok(tuple) = value.cast::<PyTuple>() {
        return tuple
            .iter()
            .map(|item| value_from_py(&item))
            .collect::<PyResult<_>>()
            .map(DomainValue::Array);
    }
    let datetime = value.py().import("datetime")?;
    if value.is_instance(&datetime.getattr("datetime")?)? {
        let iso: String = if let Ok(wire) = value.getattr("_wire_text")
            && !wire.is_none()
        {
            wire.extract()?
        } else {
            value.call_method0("isoformat")?.extract()?
        };
        let parsed =
            time::OffsetDateTime::parse(&iso, &time::format_description::well_known::Rfc3339)
                .map_err(|_| PyValueError::new_err("datetimes must include a timezone"))?;
        return Ok(DomainValue::Datetime(
            parsed.to_offset(time::UtcOffset::UTC),
        ));
    }
    Err(PyValueError::new_err("unsupported authored value"))
}

pub(crate) fn values_from_py(value: &Bound<'_, PyAny>) -> PyResult<Vec<dokime_app::core::Value>> {
    match value_from_py(value)? {
        dokime_app::core::Value::Array(values) => Ok(values),
        _ => Err(PyValueError::new_err("expected a list of values")),
    }
}

pub(crate) fn value_map_from_py(
    value: &Bound<'_, PyAny>,
) -> PyResult<std::collections::BTreeMap<String, dokime_app::core::Value>> {
    match value_from_py(value)? {
        dokime_app::core::Value::Object(values) => Ok(values),
        _ => Err(PyValueError::new_err("expected a value mapping")),
    }
}

pub(crate) fn value_lists_from_py(
    value: &Bound<'_, PyAny>,
) -> PyResult<std::collections::BTreeMap<String, Vec<dokime_app::core::Value>>> {
    value
        .cast::<PyDict>()?
        .iter()
        .map(|(key, values)| Ok((key.extract()?, values_from_py(&values)?)))
        .collect()
}

/* === Private Functions === */
fn json_to_py(py: Python<'_>, value: Value) -> PyResult<Py<PyAny>> {
    match value {
        Value::Null => Ok(py.None()),
        Value::Bool(value) => value.into_py_any(py),
        Value::Number(number) => number_to_py(py, number),
        Value::String(value) => value.into_py_any(py),
        Value::Array(values) => {
            let list = PyList::empty(py);
            for value in values {
                list.append(json_to_py(py, value)?)?;
            }

            Ok(list.into_any().unbind())
        }
        Value::Object(values) => {
            let dict = PyDict::new(py);
            for (key, value) in values {
                dict.set_item(key, json_to_py(py, value)?)?;
            }

            Ok(dict.into_any().unbind())
        }
    }
}

fn number_to_py(py: Python<'_>, number: Number) -> PyResult<Py<PyAny>> {
    if let Some(value) = number.as_i64() {
        return value.into_py_any(py);
    }

    if let Some(value) = number.as_u64() {
        return value.into_py_any(py);
    }

    if let Some(value) = number.as_f64() {
        return value.into_py_any(py);
    }

    Err(PyValueError::new_err("unsupported JSON number"))
}

/* === Tests === */
