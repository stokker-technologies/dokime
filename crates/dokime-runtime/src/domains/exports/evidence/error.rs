/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Saved Evidence Export Failures

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use thiserror::Error;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type Result<T> = std::result::Result<T, Error>;

/* === Structs === */

/* === Enums === */

#[derive(Debug, Error)]
pub enum Error {
    #[error("another evidence export is active")]
    Busy,

    #[error("evidence export was cancelled")]
    Cancelled,

    #[error("{kind} {id} was not found")]
    Missing { kind: &'static str, id: String },

    #[error("{0} must be durably sealed before export")]
    Unsealed(&'static str),

    #[error("invalid saved evidence: {0}")]
    Invalid(&'static str),

    #[error("evidence archive exceeds its configured limit")]
    TooLarge,

    #[error("evidence export I/O failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("evidence SQLite write failed: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("evidence ZIP write failed: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("evidence JSON encoding failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("saved evidence read failed: {0}")]
    Runtime(#[from] crate::runtime::Error),

    #[error("saved evidence storage failed: {0}")]
    Storage(#[from] crate::storage::Error),

    #[error("saved evidence dependency failed: {0}")]
    Dependency(#[from] harmos::storage::Error),

    #[error("evidence export worker failed: {0}")]
    Task(#[from] tokio::task::JoinError),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
