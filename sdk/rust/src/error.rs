/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// SDK Authoring and Transport Failures

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use harmos::Error as HarmosError;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type Result<T> = std::result::Result<T, Error>;

/* === Structs === */

/* === Enums === */

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Invalid(String),

    #[error(transparent)]
    Harmos(#[from] HarmosError),
}

/* === Traits === */

/* === Implementations === */

impl Error {
    pub fn failure(
        classification: harmos::Classification, code: impl Into<String>, message: impl Into<String>, action: impl Into<String>,
    ) -> Self {
        HarmosError::failure(classification, code, message, action).into()
    }
}

impl From<harmos::Failure> for Error {
    fn from(failure: harmos::Failure) -> Self {
        Self::Harmos(failure.into())
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Harmos(error.into())
    }
}

impl From<tonic::Status> for Error {
    fn from(error: tonic::Status) -> Self {
        Self::Harmos(error.into())
    }
}

impl From<tonic::transport::Error> for Error {
    fn from(error: tonic::transport::Error) -> Self {
        Self::Harmos(error.into())
    }
}

impl From<Error> for HarmosError {
    fn from(error: Error) -> Self {
        match error {
            Error::Invalid(message) => Self::Invalid(message),
            Error::Harmos(error) => error,
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
