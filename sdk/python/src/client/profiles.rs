/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bind Rust Pairing And Profile Input Without Secret Store Access

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::PathBuf;

// Third Party Imports
use pyo3::prelude::*;

// Local Imports
use super::value_error;
use dokime::client::{
    Target, pairing,
    profiles::{self, Store},
};

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
    module.add_function(wrap_pyfunction!(read_pairing, module)?)?;
    module.add_function(wrap_pyfunction!(profile_path, module)?)?;
    module.add_function(wrap_pyfunction!(decode_profiles, module)?)?;
    module.add_function(wrap_pyfunction!(load_profile, module)?)?;
    module.add_function(wrap_pyfunction!(remember_endpoint, module)?)
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

#[pyfunction]
#[pyo3(signature = (path=None))]
fn profile_path(py: Python<'_>, path: Option<PathBuf>) -> PyResult<PathBuf> {
    py.detach(|| Store::discover(path.as_deref()).map(|store| store.path().to_owned()).map_err(value_error))
}

#[pyfunction]
fn decode_profiles(py: Python<'_>, data: Vec<u8>) -> PyResult<String> {
    py.detach(|| {
        let document = profiles::decode(&data).map_err(value_error)?;
        serde_json::to_string(&document).map_err(value_error)
    })
}

#[pyfunction]
#[pyo3(signature = (identifier, path=None))]
fn load_profile(py: Python<'_>, identifier: String, path: Option<PathBuf>) -> PyResult<String> {
    py.detach(|| {
        let store = Store::discover(path.as_deref()).map_err(value_error)?;
        let document = store.load().map_err(value_error)?;
        let profile = profiles::get(&document, &identifier).map_err(value_error)?;
        serde_json::to_string(profile).map_err(value_error)
    })
}

#[pyfunction]
fn remember_endpoint(py: Python<'_>, profile: String, path: PathBuf, endpoint: String) -> PyResult<()> {
    py.detach(|| {
        let profile = serde_json::from_str(&profile).map_err(value_error)?;
        let selection = profiles::selection(&profile).map_err(value_error)?;
        Store::new(path).remember_endpoint(&selection, &endpoint).map_err(value_error)
    })
}

/* === Tests === */
