/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub mod error;
pub mod objects;
pub mod records;
pub mod reports;
pub mod requests;
pub mod sqlite;
pub mod telemetry;
mod values;

/* === Re-Exports === */
// `plugins/store.rs` belongs to the parallel settings consolidation and still
// names the SQLite owner by its former module path.
pub use sqlite as local;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
pub type Manager = harmos::storage::StorageManager<objects::Objects>;
pub type Backend = sqlite::Backend<objects::Objects>;

pub type Result<T> = std::result::Result<T, Error>;

/* === Structs === */

/* === Enums === */
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Runtime(#[from] harmos::Error),
    #[error(transparent)]
    Saved(#[from] crate::execution::saved::SavedEvidenceError),
    #[error("invalid persisted evidence: {0}")]
    Invalid(&'static str),
    #[error(transparent)]
    Decode(#[from] harmos::MessageError),
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    #[error(transparent)]
    Timestamp(#[from] time::error::Parse),
    #[error(transparent)]
    DatetimeFormat(#[from] time::error::Format),
    #[error(transparent)]
    Identifier(#[from] harmos::IdentifierError),
    #[error(transparent)]
    Contract(#[from] dokime_contract::Error),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
