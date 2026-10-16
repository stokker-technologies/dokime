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
pub mod rows;
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
/// Encoded runtime-state snapshot payload, including its embedded journal tail.
pub const STATE_SNAPSHOT_BYTES: usize = 1024 * 1024;
// Integrity envelope, positions and histories around one encoded state record.
const STATE_ENVELOPE_BYTES: usize = 64 * 1024;

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
/// Durable state limits matching what the manager admits: one recovery entry
/// carries at most a quarter of the pending write bytes and one checkpoint the
/// runtime-state snapshot, each base64 encoded inside its envelope.
pub fn state_settings(
    settings: &harmos::storage::StorageSettings,
) -> harmos::config::StorageStateSettings {
    let encoded = |payload: usize| {
        std::num::NonZeroUsize::new(
            payload
                .div_ceil(3)
                .saturating_mul(4)
                .saturating_add(STATE_ENVELOPE_BYTES),
        )
        .expect("the envelope keeps encoded records nonempty")
    };
    harmos::config::StorageStateSettings {
        max_entry_bytes: encoded(settings.request.max_pending_write_bytes.get() / 4),
        max_checkpoint_bytes: encoded(STATE_SNAPSHOT_BYTES),
    }
}

/// Read a finished file whole; the reader verifies its length and digest.
pub async fn read_file_bytes(
    storage: &Manager,
    file: &harmos::storage::FileReference,
) -> Result<Vec<u8>> {
    let mut reader = storage
        .read_file(harmos::storage::ReadFile {
            file: file.clone(),
            range: harmos::storage::ByteRange {
                offset: 0,
                limit: None,
            },
        })
        .await?;
    let mut bytes = Vec::new();
    while let Some(batch) = reader.next().await? {
        bytes.extend_from_slice(&batch.bytes);
    }
    Ok(bytes)
}

/* === Private Functions === */

/* === Tests === */
