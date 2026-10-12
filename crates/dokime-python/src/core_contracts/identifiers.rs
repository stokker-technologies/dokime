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
use pyo3::{prelude::*, types::PyAny};
use sha2::{Digest, Sha256};
// Local Imports
use super::convert::*;
use crate::{pyserde, to_value_error};
use dokime_app::core::{Identifier, MeasurementUnit, Metadata};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
#[pyfunction]
pub(crate) fn identifier_contract(value: String) -> PyResult<String> {
    Ok(Identifier::new(value).map_err(to_value_error)?.to_string())
}

#[pyfunction]
#[pyo3(signature = (identifier, label, description, tags, attributes=None, created_at=None, updated_at=None))]
pub(crate) fn metadata_contract(
    py: Python<'_>,
    identifier: String,
    label: String,
    description: Option<String>,
    tags: Option<Vec<String>>,
    attributes: Option<Bound<'_, PyAny>>,
    created_at: Option<Bound<'_, PyAny>>,
    updated_at: Option<Bound<'_, PyAny>>,
) -> PyResult<Py<PyAny>> {
    let mut metadata = Metadata::definition(
        Identifier::new(identifier).map_err(to_value_error)?,
        label,
        description,
        tags.unwrap_or_default(),
    );

    metadata.attributes = attributes
        .as_ref()
        .map(pyserde::value_map_from_py)
        .transpose()?
        .unwrap_or_default();
    let date = |value: Option<&Bound<'_, PyAny>>| -> PyResult<Option<time::OffsetDateTime>> {
        value
            .map(|value| match pyserde::value_from_py(value)? {
                dokime_app::core::Value::Datetime(date) => Ok(date),
                _ => Err(pyo3::exceptions::PyValueError::new_err(
                    "metadata dates must be timezone-aware datetimes",
                )),
            })
            .transpose()
    };
    metadata.created_at = date(created_at.as_ref())?;
    metadata.updated_at = date(updated_at.as_ref())?;

    pyserde::to_py(py, &metadata)
}

#[pyfunction]
pub(crate) fn measurement_unit_contract(py: Python<'_>, symbol: String) -> PyResult<Py<PyAny>> {
    let unit = MeasurementUnit::custom(
        Identifier::new(unit_identifier(&symbol)).map_err(to_value_error)?,
        symbol.clone(),
        symbol,
    );

    pyserde::to_py(py, &unit)
}

#[pyfunction]
pub(crate) fn value_type_name(value_type: Bound<'_, PyAny>) -> PyResult<String> {
    Ok(value_type_wire_name(declaration_value_type_from_py(&value_type)?).to_owned())
}

#[pyfunction]
pub(crate) fn nullable_value_type(value_type: Bound<'_, PyAny>) -> PyResult<bool> {
    is_nullable_value_type(&value_type)
}

#[pyfunction]
pub(crate) fn stable_wire_key(value: Bound<'_, PyAny>) -> PyResult<String> {
    canonical_json_with_spacing(&authoring_json_value(&value)?)
}

#[pyfunction]
pub(crate) fn stable_wire_hash(value: Bound<'_, PyAny>) -> PyResult<String> {
    let canonical = canonical_json_compact(&authoring_json_value(&value)?)?;
    let digest = Sha256::digest(canonical.as_bytes());

    Ok(hex_lower(&digest))
}

/* === Private Functions === */

/* === Tests === */
