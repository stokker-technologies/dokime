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
    Bundle(#[from] harmos::bundle::Error),
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
pub fn guest_refused(path: &str) -> Error {
    Error::Invalid(format!("Dokime admits native sidecar plugins only; `{path}` declares a WebAssembly guest"))
}

/* === Private Functions === */

/* === Tests === */
