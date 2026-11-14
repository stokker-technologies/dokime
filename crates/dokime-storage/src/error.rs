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
    #[error("Close Arrow Storage: Join Its Flush Worker: {0}")]
    Worker(#[from] tokio::task::JoinError),

    #[error("{0}")]
    NotFound(String),

    #[error("Read Arrow Evidence: {0}; Restore a Valid Backup")]
    Corrupt(&'static str),

    #[error("Process Arrow Evidence: {0}; Check the Stream Contract")]
    Data(&'static str),

    #[error("Read Arrow Evidence: Reduce the Selected Batch ({budget}: {reached} > {limit})")]
    Budget { budget: &'static str, limit: u64, reached: u64 },

    #[error("{0}")]
    Conflict(String),

    #[error("{0}")]
    AlreadyExists(String),

    #[error("{0}")]
    Permission(String),

    #[error("Cannot Open Evidence Root: {0}. Select a fresh storage root; the old folder is preserved")]
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

    #[error("Evaluate Stored Evidence: {0}")]
    Query(#[from] harmos_storage::Error),

    #[error("Read Stream Summary: Restore a Valid Backup: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Export Evidence: Check the Archive Destination: {0}")]
    Zip(#[from] zip::result::ZipError),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
