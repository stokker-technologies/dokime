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
use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};
// Third Party Imports
use pyo3::{exceptions::PyOSError, exceptions::PyValueError, prelude::*};
use serde_json::Value;
// Local Imports
use crate::{pyserde, to_value_error};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
#[pyfunction]
pub(crate) fn write_json_file(path: String, payload: Bound<'_, PyAny>) -> PyResult<()> {
    let path = PathBuf::from(path);
    create_parent_dir(&path)?;

    let value = pyserde::py_to_json(&payload)?;
    let text = serde_json::to_string_pretty(&value).map_err(to_value_error)?;

    fs::write(path, text).map_err(to_io_error)
}

#[pyfunction]
pub(crate) fn write_json_lines(path: String, payloads: Bound<'_, PyAny>) -> PyResult<()> {
    let path = PathBuf::from(path);
    create_parent_dir(&path)?;

    let value = pyserde::py_to_json(&payloads)?;
    let Value::Array(items) = value else {
        return Err(PyValueError::new_err(
            "JSON lines payload must be a list or tuple",
        ));
    };

    let file = File::create(path).map_err(to_io_error)?;
    let mut writer = BufWriter::new(file);
    for item in items {
        serde_json::to_writer(&mut writer, &item).map_err(to_value_error)?;
        writer.write_all(b"\n").map_err(to_io_error)?;
    }

    writer.flush().map_err(to_io_error)
}

/* === Private Functions === */
fn create_parent_dir(path: &Path) -> PyResult<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent).map_err(to_io_error)?;
    }

    Ok(())
}

fn to_io_error(error: std::io::Error) -> PyErr {
    PyOSError::new_err(error.to_string())
}

/* === Tests === */
