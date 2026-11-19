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
use std::{io, result};

// Third Party Imports
use tonic::{Status, transport};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type Result<T> = result::Result<T, Error>;

/* === Structs === */

/* === Enums === */

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Invalid(String),

    #[error(transparent)]
    Harmos(#[from] harmos::Error),
}

/* === Traits === */

/* === Implementations === */

impl Error {
    pub fn failure(
        classification: harmos::Classification, code: impl Into<String>, message: impl Into<String>, action: impl Into<String>,
    ) -> Self {
        harmos::Error::failure(classification, code, message, action).into()
    }
}

impl From<harmos::Failure> for Error {
    fn from(failure: harmos::Failure) -> Self {
        Self::Harmos(failure.into())
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Harmos(error.into())
    }
}

impl From<Status> for Error {
    fn from(error: Status) -> Self {
        Self::Harmos(error.into())
    }
}

impl From<transport::Error> for Error {
    fn from(error: transport::Error) -> Self {
        Self::Harmos(error.into())
    }
}

impl From<Error> for harmos::Error {
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
