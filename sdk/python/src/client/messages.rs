/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Reflect The Rust Catalog Without A Generated Python Wire Tree

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use pyo3::{prelude::*, types::PyBytes};

// Local Imports
use super::value_error;
use dokime::harmos::{
    grpc,
    operations::{Contract, Contracts},
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
    module.add_function(wrap_pyfunction!(contracts, module)?)?;
    module.add_function(wrap_pyfunction!(descriptors, module)?)?;
    module.add_function(wrap_pyfunction!(preferences, module)?)
}

/* === Private Functions === */

#[pyfunction]
fn contracts(py: Python<'_>) -> PyResult<String> {
    py.detach(|| {
        let contracts = Contracts::of::<dokime::operations::Operations>().map_err(value_error)?;
        serde_json::to_string(&contracts).map_err(value_error)
    })
}

#[pyfunction]
fn descriptors<'py>(py: Python<'py>, contract: String) -> PyResult<Bound<'py, PyBytes>> {
    let bytes = py.detach(|| {
        let contract: Contract = serde_json::from_str(&contract).map_err(value_error)?;
        let binding = contract.grpc().ok_or_else(|| value_error("operation has no gRPC declaration"))?;
        let descriptor = grpc::descriptor(&binding.package, &binding.service, &[(&binding.method, &contract)]).map_err(value_error)?;
        Ok::<_, PyErr>(descriptor.parent_pool().encode_to_vec())
    })?;
    Ok(PyBytes::new(py, &bytes))
}

#[pyfunction]
fn preferences() -> PyResult<String> {
    let settings = dokime::domain::model::settings::application::ClientPreferences::default();
    serde_json::to_string(&settings).map_err(value_error)
}

/* === Tests === */
