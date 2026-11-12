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
    Harmos(#[from] harmos::Error),
}

/* === Traits === */

/* === Implementations === */

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
