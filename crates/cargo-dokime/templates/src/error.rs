/* ***
 *
 * {{package}}
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

pub type Result<T, E = Error> = std::result::Result<T, E>;

/* === Structs === */

/* === Enums === */

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("bench")]
    Bench(#[from] crate::bench::BenchError),

    #[error("dokime sdk: {0}")]
    Sdk(#[from] dokime::Error),

    #[error("evidence file")]
    Artifact(#[from] std::io::Error),

    #[error("summary")]
    Summary(#[from] serde_json::Error),

    #[error("a soak sample carries no numeric `{0}`")]
    Unmeasured(&'static str),
}

/* === Traits === */

/* === Implementations === */

impl From<Error> for dokime::domains::ActionDispatcherError {
    fn from(error: Error) -> Self {
        Self::Refused {
            code: -32_000,
            message: error.to_string(),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
