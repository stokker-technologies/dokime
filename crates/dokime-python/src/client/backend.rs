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
use pyo3::{prelude::*, types::PyTuple};

// Local Imports
use dokime::client::{Backend, Limits};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[pyclass(name = "Backend", module = "dokime._native.client", frozen)]
pub(crate) struct PyBackend {
    pub(super) inner: Backend,
}

#[pyclass(name = "Limits", module = "dokime._native.client", frozen)]
pub(crate) struct PyLimits {
    inner: Limits,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[pymethods]
impl PyBackend {
    #[getter]
    fn backend_id(&self) -> &str {
        &self.inner.backend_id
    }

    #[getter]
    fn epoch(&self) -> &str {
        &self.inner.epoch
    }

    #[getter]
    fn display_name(&self) -> &str {
        &self.inner.display_name
    }

    #[getter]
    fn product_version(&self) -> &str {
        &self.inner.product_version
    }

    #[getter]
    fn api_version(&self) -> &str {
        &self.inner.api_version
    }

    #[getter]
    fn descriptor_sha256(&self) -> &str {
        &self.inner.descriptor_sha256
    }

    #[getter]
    fn methods<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyTuple>> {
        PyTuple::new(py, &self.inner.methods)
    }

    #[getter]
    fn limits(&self) -> PyLimits {
        PyLimits {
            inner: self.inner.limits.clone(),
        }
    }
}

#[pymethods]
impl PyLimits {
    #[getter]
    fn max_request_bytes(&self) -> u64 {
        self.inner.max_request_bytes
    }

    #[getter]
    fn max_response_bytes(&self) -> u64 {
        self.inner.max_response_bytes
    }

    #[getter]
    fn max_content_bytes(&self) -> Option<u64> {
        self.inner.max_content_bytes
    }

    #[getter]
    fn max_chunk_bytes(&self) -> Option<u32> {
        self.inner.max_chunk_bytes
    }

    #[getter]
    fn max_page_items(&self) -> Option<u32> {
        self.inner.max_page_items
    }

    #[getter]
    fn max_stream_buffer_items(&self) -> Option<u32> {
        self.inner.max_stream_buffer_items
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
