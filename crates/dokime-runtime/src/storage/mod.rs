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
/// Encoded runtime-state snapshot payload, including its embedded journal tail.
const STATE_SNAPSHOT_BYTES: usize = 1024 * 1024;
// Integrity envelope, positions and histories around one encoded state record.
const STATE_ENVELOPE_BYTES: usize = 64 * 1024;
// One recovery entry may take at most this share of a checkpoint: what it
// changes must still fit the snapshot beside the rest of state.
const STATE_ENTRY_SHARE: usize = 2;

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
    HarmosStorage(#[from] harmos::storage::Error),
    #[error(transparent)]
    HarmosDomain(#[from] harmos::domain::Error),
    #[error(transparent)]
    Saved(#[from] dokime_domain::model::execution::saved::SavedEvidenceError),
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
    Contract(#[from] dokime_domain::Error),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
/// Runtime snapshot policy shared by state assembly and its encoded storage limits.
pub fn snapshot_policy() -> harmos::state::SnapshotPolicy {
    harmos::state::SnapshotPolicy {
        schema: harmos::storage::SchemaKey {
            payload_encoding: harmos::storage::PayloadEncoding::ProtobufV1,
            id: harmos::Identifier::new("dokime.runtime.state").unwrap(),
            version: 1.try_into().unwrap(),
        },
        every: 1024.try_into().unwrap(),
        max_bytes: STATE_SNAPSHOT_BYTES.try_into().unwrap(),
    }
}

/// Durable state limits, derived from the state policy alone: one checkpoint
/// holds the runtime-state snapshot base64 encoded inside its envelope, and
/// one recovery entry is a fixed share of that checkpoint.
pub fn state_settings() -> harmos::config::StorageStateSettings {
    let policy = snapshot_policy();
    let checkpoint = policy.max_bytes.get().div_ceil(3).saturating_mul(4).saturating_add(STATE_ENVELOPE_BYTES);
    let bytes = |bytes: usize| std::num::NonZeroUsize::new(bytes).expect("the envelope keeps encoded records nonempty");
    harmos::config::StorageStateSettings { max_entry_bytes: bytes(checkpoint / STATE_ENTRY_SHARE), max_checkpoint_bytes: bytes(checkpoint) }
}

/// Read a finished file whole; the reader verifies its length and digest.
pub async fn read_file_bytes(storage: &Manager, file: &harmos::storage::FileReference) -> Result<Vec<u8>> {
    let mut reader = storage
        .read_file(harmos::storage::operations::ReadFile {
            file: file.clone(),
            range: harmos::storage::ByteRange { offset: 0, limit: None },
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
