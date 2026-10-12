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
use pyo3::prelude::*;
// Local Imports
use crate::to_value_error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Invalid(String),
    #[error("prompt was cancelled")]
    PromptCancelled,
    #[error("{0}")]
    Sdk(#[source] dokime::Error),
}

/* === Traits === */

/* === Implementations === */
impl From<String> for Error {
    fn from(message: String) -> Self {
        Self::Invalid(message)
    }
}

impl From<&str> for Error {
    fn from(message: &str) -> Self {
        Self::Invalid(message.into())
    }
}

impl From<dokime::Error> for Error {
    fn from(error: dokime::Error) -> Self {
        match error {
            dokime::Error::PromptCancelled => Self::PromptCancelled,
            error => Self::Sdk(error),
        }
    }
}

impl Error {
    pub(crate) fn into_pyerr(self) -> PyErr {
        match self {
            Self::Sdk(error) => to_value_error(error),
            Self::Invalid(message) => to_value_error(message),
            Self::PromptCancelled => Python::attach(|py| {
                match py
                    .import("dokime.core.authoring.prompts")
                    .and_then(|module| module.getattr("PromptCancelled"))
                    .and_then(|class| class.call1(("prompt was cancelled",)))
                {
                    Ok(exception) => PyErr::from_value(exception),
                    Err(error) => error,
                }
            }),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
