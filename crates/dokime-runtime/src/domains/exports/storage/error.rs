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
use std::path::PathBuf;
// Third Party Imports
use thiserror::Error;
// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
pub type Result<T> = std::result::Result<T, StorageError>;

/* === Structs === */

/* === Enums === */
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("storage I/O failed: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("storage I/O failed while {operation} {path:?}: {source}")]
    IoContext { operation: &'static str, path: PathBuf, source: std::io::Error },

    #[error("storage JSON failed: {source}")]
    Json {
        #[from]
        source: serde_json::Error,
    },

    #[error("Arrow IPC storage failed: {source}")]
    Arrow {
        #[from]
        source: arrow_schema::ArrowError,
    },

    #[error("ZIP bundle storage failed: {source}")]
    Zip {
        #[from]
        source: zip::result::ZipError,
    },

    #[error("ZIP bundle exceeds the {max_bytes}-byte limit")]
    ArchiveTooLarge { max_bytes: usize },

    #[error("path {path:?} must be relative and stay under its root")]
    InvalidRelativePath { path: PathBuf },

    #[error("path {path:?} must live under root {root:?}")]
    PathOutsideRoot { path: PathBuf, root: PathBuf },

    #[error("archive entry {path:?} is duplicated")]
    DuplicateArchiveEntry { path: String },

    #[error("missing required storage file {path:?}")]
    MissingFile { path: PathBuf },

    #[error("invalid storage layout: {message}")]
    InvalidLayout { message: String },

    #[error("telemetry schema fingerprint mismatch for {stream} v{schema_version}")]
    SchemaFingerprintMismatch { stream: String, schema_version: u32, expected: String, actual: String },

    #[error("telemetry batch for {stream} v{schema_version} has no samples")]
    EmptyTelemetryBatch { stream: String, schema_version: u32 },

    #[error("telemetry sample cannot be stored: {message}")]
    InvalidTelemetrySample { message: String },
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
