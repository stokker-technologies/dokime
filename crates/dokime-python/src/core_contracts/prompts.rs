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
    exceptions::PyValueError,
    prelude::*,
    types::{PyAny, PyDict},
};
// Local Imports
use crate::{pyserde, to_value_error};
use dokime_app::capabilities::prompts::{
    FieldValidation, PromptField, PromptFieldKind, declared_confirmation, declared_form,
};
use dokime_app::core::{Identifier, Metadata, ValueType};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
#[pyfunction]
#[pyo3(signature = (prompt_type, title, message = None, submit = None, cancel = None, fields = None))]
pub(crate) fn prompt_definition_contract(
    py: Python<'_>,
    prompt_type: String,
    title: String,
    message: Option<String>,
    submit: Option<String>,
    cancel: Option<String>,
    fields: Option<Vec<Bound<'_, PyDict>>>,
) -> PyResult<Py<PyAny>> {
    let mut declared = match fields {
        None => {
            if submit.is_some() || cancel.is_some() {
                return Err(PyValueError::new_err(
                    "a confirmation answers yes or no; `submit` and `cancel` label a form's actions",
                ));
            }

            declared_confirmation(&prompt_type, title)
        }
        Some(written) => {
            let mut declared = declared_form(&prompt_type, title);
            if let Some(label) = submit {
                declared.actions[0].label = label;
            }
            if let Some(label) = cancel {
                declared.actions[1].label = label;
            }
            declared.fields = written
                .iter()
                .map(declared_field)
                .collect::<PyResult<Vec<_>>>()?;

            declared
        }
    };
    declared.message = message;

    pyserde::to_py(py, &declared)
}

/* === Private Functions === */
fn declared_field(written: &Bound<'_, PyDict>) -> PyResult<PromptField> {
    let name: String = required(written, "name")?;
    let label: String = required(written, "label")?;
    let description: Option<String> = optional(written, "description")?;
    let kind: String = required(written, "kind")?;
    let (kind, value_type) = match kind.as_str() {
        "text" => (PromptFieldKind::Text, ValueType::Text),
        "boolean" => (PromptFieldKind::Boolean, ValueType::Boolean),
        "int" => (PromptFieldKind::Int, ValueType::Integer),
        "float" => (PromptFieldKind::Float, ValueType::Number),
        "choice" => (PromptFieldKind::Choice, ValueType::Text),
        other => {
            return Err(PyValueError::new_err(format!(
                "`{other}` is not a prompt field kind; a field is text, boolean, int, float or choice"
            )));
        }
    };
    let default = match written.get_item("default")? {
        Some(value) if !value.is_none() => Some(pyserde::value_from_py(&value)?),
        _ => None,
    };
    let validation = match written.get_item("validation")? {
        Some(value) if !value.is_none() => {
            let fields = value.cast::<PyDict>()?;
            FieldValidation {
                min_length: optional(fields, "min_length")?,
                max_length: optional(fields, "max_length")?,
                pattern: optional(fields, "pattern")?,
                minimum: fields
                    .get_item("minimum")?
                    .filter(|value| !value.is_none())
                    .as_ref()
                    .map(pyserde::value_from_py)
                    .transpose()?,
                maximum: fields
                    .get_item("maximum")?
                    .filter(|value| !value.is_none())
                    .as_ref()
                    .map(pyserde::value_from_py)
                    .transpose()?,
                min_items: optional(fields, "min_items")?,
                max_items: optional(fields, "max_items")?,
            }
        }
        _ => FieldValidation::default(),
    };

    Ok(PromptField {
        metadata: Metadata::new(
            Identifier::new(name).map_err(to_value_error)?,
            label,
            description,
        ),
        value_type,
        kind,
        multiple: optional(written, "multiple")?.unwrap_or(false),
        required: optional(written, "required")?.unwrap_or(true),
        default,
        choices: optional(written, "choices")?.unwrap_or_default(),
        validation,
    })
}

fn required(written: &Bound<'_, PyDict>, key: &str) -> PyResult<String> {
    written
        .get_item(key)?
        .ok_or_else(|| PyValueError::new_err(format!("a prompt field declares `{key}`")))?
        .extract::<String>()
}

fn written_value<'py>(
    written: &Bound<'py, PyDict>,
    key: &str,
) -> PyResult<Option<Bound<'py, PyAny>>> {
    Ok(written.get_item(key)?.filter(|value| !value.is_none()))
}

fn optional<'py, T>(written: &Bound<'py, PyDict>, key: &str) -> PyResult<Option<T>>
where
    T: serde::de::DeserializeOwned,
{
    written_value(written, key)?
        .map(|value| pyserde::from_py::<T>(&value))
        .transpose()
}

/* === Tests === */
