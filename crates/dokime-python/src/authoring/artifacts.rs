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
use std::path::PathBuf;
// Third Party Imports
use pyo3::prelude::*;
// Local Imports
use crate::to_value_error;
use dokime::artifacts::{ArtifactWriter, Artifacts};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[pyclass(name = "Artifacts", module = "dokime._native")]
pub(crate) struct PyArtifacts {
    pub(super) inner: Artifacts,
}

#[pyclass(name = "ArtifactWriter", module = "dokime._native")]
pub(crate) struct PyArtifactWriter {
    inner: Option<ArtifactWriter>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
#[pymethods]
impl PyArtifacts {
    fn create(&self, name: String) -> PyResult<PyArtifactWriter> {
        let inner = self.inner.create(name).map_err(to_value_error)?;
        Ok(PyArtifactWriter { inner: Some(inner) })
    }

    fn directory(&self, py: Python<'_>, name: String) -> PyResult<PathBuf> {
        py.detach(|| self.inner.directory(name))
            .map_err(to_value_error)
    }

    fn import_file(&self, py: Python<'_>, path: PathBuf, name: String) -> PyResult<()> {
        py.detach(|| self.inner.import(path, name))
            .map_err(to_value_error)
    }
}

#[pymethods]
impl PyArtifactWriter {
    fn write_all(&mut self, py: Python<'_>, data: Vec<u8>) -> PyResult<()> {
        let writer = self
            .inner
            .as_mut()
            .ok_or_else(|| to_value_error("artifact writer is closed"))?;
        py.detach(|| writer.write_all(&data))
            .map_err(to_value_error)
    }

    fn flush(&mut self) -> PyResult<()> {
        self.inner
            .as_mut()
            .ok_or_else(|| to_value_error("artifact writer is closed"))?
            .flush()
            .map_err(to_value_error)
    }

    fn finish(&mut self) -> PyResult<()> {
        if let Some(writer) = self.inner.take() {
            writer.finish().map_err(to_value_error)?;
        }
        Ok(())
    }

    fn abandon(&mut self) {
        if let Some(writer) = self.inner.take() {
            writer.abandon();
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
