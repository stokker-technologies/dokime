/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bind Pairing Input And Transport Validation

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::PathBuf;

// Third Party Imports
use pyo3::prelude::*;

// Local Imports
use super::value_error;
use dokime::client::{Target, pairing};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_function(wrap_pyfunction!(validate_target, module)?)?;
    module.add_function(wrap_pyfunction!(read_pairing, module)?)
}

/* === Private Functions === */

#[pyfunction]
#[pyo3(signature = (endpoint, ca=None))]
fn validate_target(py: Python<'_>, endpoint: String, ca: Option<String>) -> PyResult<String> {
    py.detach(|| Target::new(endpoint, ca).map(|target| target.endpoint().trim_end_matches('/').to_owned()).map_err(value_error))
}

#[pyfunction]
fn read_pairing(py: Python<'_>, path: PathBuf) -> PyResult<String> {
    py.detach(|| {
        let pairing = pairing::read(&path).map_err(value_error)?;
        serde_json::to_string(&pairing).map_err(value_error)
    })
}

/* === Tests === */
