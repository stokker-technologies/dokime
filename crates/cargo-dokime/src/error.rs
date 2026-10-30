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

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("development child exited with status {0}")]
    DevelopmentExit(i32),
    #[error("cancelled")]
    Cancelled,

    #[error("{0}")]
    Invalid(String),

    #[error(transparent)]
    Packaging(dokime_packaging::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Bindings(#[from] harmos_packaging::python::Error),

    #[error(transparent)]
    Proto(#[from] harmos_packaging::proto::ExportError),

    #[error(transparent)]
    Contract(#[from] dokime::harmos::operations::ContractError),

    #[error(transparent)]
    Bundle(#[from] harmos_packaging::bundle::Error),
}

/* === Traits === */

/* === Implementations === */

impl From<dokime_packaging::Error> for Error {
    fn from(error: dokime_packaging::Error) -> Self {
        match error {
            dokime_packaging::Error::Cancelled => Self::Cancelled,
            error => Self::Packaging(error),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
