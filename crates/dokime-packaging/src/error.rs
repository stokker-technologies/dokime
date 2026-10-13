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
    #[error("{0} exceeded its execution timeout")]
    Timeout(String),
    #[error("{0} exceeded the {1} byte limit")]
    Output(String, usize),
    #[error("{program} failed ({status}): {stderr}")]
    Process {
        program: String,
        status: std::process::ExitStatus,
        stderr: String,
    },
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/* === Traits === */

/* === Implementations === */
impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self::Invalid(error.to_string())
    }
}

impl From<harmos::plugin::bundle::Error> for Error {
    fn from(error: harmos::plugin::bundle::Error) -> Self {
        Self::Invalid(error.to_string())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
