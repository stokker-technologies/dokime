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
    #[error("local state filesystem: {0}")]
    Io(#[from] std::io::Error),

    #[error("local state is closed")]
    Closed,

    #[error("local state is already owned by another runtime")]
    Owned,

    #[error("local state SQLite: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("local state task: {0}")]
    Task(#[from] tokio::task::JoinError),

    #[error("storage root is incompatible with this build: {0}; delete it or choose a fresh root")]
    Incompatible(String),

    #[error("invalid storage catalog operation: {0}")]
    Catalog(String),

    #[error("SQLite data: {0}")]
    Data(&'static str),

    #[error("SQLite integrity: {0}")]
    Corrupt(&'static str),

    #[error("SQLite identity conflict: {0}")]
    Conflict(&'static str),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Runtime(#[from] harmos::Error),

    #[error(transparent)]
    HarmosStorage(#[from] harmos::storage::Error),

    #[error(transparent)]
    HarmosDomain(#[from] harmos::domain::Error),

    #[error(transparent)]
    Publication(#[from] harmos::filesystem::Error),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
