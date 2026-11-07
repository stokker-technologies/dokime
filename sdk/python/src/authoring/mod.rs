/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Plugin Authoring

/* === Modules === */

mod callback;
mod columns;
mod context;
mod provider;
mod schema;
mod template;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, sync::Arc};

// Third Party Imports
use pyo3::{exceptions::PyValueError, prelude::*};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn register(module: &Bound<'_, PyModule>) -> PyResult<()> {
    context::register(module)?;
    module.add_function(wrap_pyfunction!(validate_plugin, module)?)?;
    module.add_function(wrap_pyfunction!(embedded_provider, module)?)
}

/* === Private Functions === */

#[pyfunction]
fn validate_plugin(py: Python<'_>, spec: String) -> PyResult<String> {
    py.detach(|| {
        let compiled = compile(&spec)?;
        serde_json::to_string(&compiled.catalog).map_err(value_error)
    })
}

#[pyfunction]
fn embedded_provider(py: Python<'_>, spec: String, callbacks: BTreeMap<String, Py<PyAny>>) -> PyResult<crate::backend::PyEmbeddedProvider> {
    let compiled = Arc::new(py.detach(|| compile(&spec))?);
    let locals = pyo3_async_runtimes::tokio::get_current_locals(py)?;
    let provider = provider::Provider::new(compiled, callbacks, locals).map_err(value_error)?;
    Ok(crate::backend::PyEmbeddedProvider { inner: Arc::new(provider) })
}

fn compile(spec: &str) -> PyResult<provider::Compiled> {
    let spec: provider::Spec = serde_json::from_str(spec).map_err(value_error)?;
    provider::compile(spec).map_err(value_error)
}

fn value_error(error: impl std::fmt::Display) -> PyErr {
    PyValueError::new_err(error.to_string())
}

/* === Tests === */
