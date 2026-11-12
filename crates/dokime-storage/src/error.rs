/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

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
    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Conflict(String),

    #[error("{0}")]
    AlreadyExists(String),

    #[error("{0}")]
    Permission(String),

    #[error("Cannot Open Evidence Root: {0}. Make a backup and choose a fresh root or explicitly reset it")]
    Incompatible(String),

    #[error("Evidence storage failed; check the storage root and retry: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Cannot access evidence files; check permissions and available space: {0}")]
    Io(#[from] std::io::Error),

    #[error("Cannot decode evidence; back up the root and restore a valid copy: {0}")]
    Decode(#[from] prost::DecodeError),

    #[error("Cannot read stream evidence; restore a valid evidence root: {0}")]
    Arrow(#[from] arrow_schema::ArrowError),

    #[error("{0}")]
    Invalid(String),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
