/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub mod format;
pub(crate) mod ipc;
pub mod objects;
pub(crate) mod projection;
mod read_rows;
pub mod records;
pub mod reports;
pub mod requests;
pub mod sqlite;
pub mod telemetry;
mod values;

/* === Re-Exports === */

pub use read_rows::{ReadRows, RowsIter};
pub(crate) use values::{datetime_memory, encode_datetime};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

// Object Catalog Validation Reuses Each Built-In Type's Immutable Shape Revision.
macro_rules! object_identity {
    ($name:ty, $id:literal) => {
        impl harmos::Identified for $name {
            const ID: &'static str = $id;

            fn definition() -> (harmos::DefinitionId, u32) {
                static VERSION: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
                let version = *VERSION.get_or_init(|| {
                    let schema = harmos::schema::graph::Schema::from_declaration(
                        <Self as harmos::schema::Described>::DEFINITION,
                        &harmos::config::SchemaSettings::default(),
                    )
                    .expect("valid object data shape");
                    harmos::schema::identity::revision(&schema).expect("valid object data shape revision")
                });
                (harmos::DefinitionId::new(Self::ID), version)
            }
        }
    };
}

pub(crate) use object_identity;

/* === Type Aliases === */

pub type Manager = harmos::storage::StorageManager<objects::Objects>;
pub type Backend = sqlite::Backend<objects::Objects>;
pub type Result<T> = std::result::Result<T, Error>;

/* === Structs === */

/* === Enums === */

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("query snapshot is unavailable")]
    QuerySnapshotUnavailable,

    #[error("Arrow telemetry: {0}")]
    Arrow(#[from] arrow_schema::ArrowError),

    #[error("settings revision conflict: expected {expected}, actual {actual}")]
    SettingsConflict { expected: u64, actual: u64 },

    #[error("invalid application settings: {0}")]
    InvalidSettings(String),

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

    #[error(
        "storage root is incompatible with this build: {0}; back up this root before an explicit reset, or choose a fresh root; existing data has been preserved"
    )]
    Incompatible(String),

    #[error("invalid storage catalog operation: {0}")]
    Catalog(String),

    #[error("storage budget {budget} exceeded (limit {limit}, reached {reached})")]
    Budget { budget: &'static str, limit: u64, reached: u64 },

    #[error("SQLite data: {0}")]
    Data(&'static str),

    #[error("SQLite integrity: {0}")]
    Corrupt(&'static str),

    #[error("SQLite identity conflict: {0}")]
    Conflict(&'static str),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Publication(#[from] harmos::filesystem::Error),

    #[error(transparent)]
    Runtime(#[from] harmos::Error),

    #[error(transparent)]
    HarmosStorage(#[from] harmos::storage::Error),

    #[error(transparent)]
    HarmosDomain(#[from] harmos::domain::Error),

    #[error(transparent)]
    Saved(#[from] dokime_domain::model::execution::saved::SavedEvidenceError),

    #[error(
        "Stream `{stream}` has different stored row definitions under one key. This evidence cannot be repaired. Choose a fresh storage root for new executions; keep this root to preserve the existing evidence."
    )]
    TelemetryIntegrity { stream: String },

    #[error(
        "Stream `{stream}` column `{column}` could not be decoded from saved evidence. Keep this storage root and use Settings → Diagnostics → Support Bundle to send the logs to the plugin developer."
    )]
    TelemetryProjection { stream: String, column: String },

    #[error("invalid persisted evidence: {0}")]
    Invalid(&'static str),

    #[error(transparent)]
    Decode(#[from] harmos::MessageError),

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

pub(crate) fn schema<T: harmos::schema::Described>(id: &str) -> harmos::storage::SchemaKey {
    let graph = harmos::schema::graph::Schema::from_declaration(T::DEFINITION, &harmos::config::SchemaSettings::default())
        .expect("finite stored data schema");
    harmos::storage::SchemaKey {
        payload_encoding: harmos::storage::PayloadEncoding::PostcardV1,
        id: harmos::Identifier::new(id).expect("stored schema identifier"),
        version: harmos::schema::identity::revision(&graph).expect("stored schema revision").try_into().unwrap(),
    }
}

pub async fn read_file_bytes(storage: &Manager, file: &harmos::storage::FileReference) -> Result<harmos::storage::ReadBatch<Vec<u8>>> {
    let mut reader = storage
        .read_file(harmos::storage::FileRead {
            source: harmos::storage::FileSource::Stored(file.clone()),
            range: harmos::storage::ByteRange { offset: 0, limit: None },
        })
        .await?;
    let length = usize::try_from(file.length).map_err(|_| Error::Invalid("file length exceeds native address space"))?;
    let first = reader.next().await?;
    if let Some(first) = first {
        if first.offset() != 0 || first.bytes().len() > length {
            return Err(Error::Invalid("first file page exceeds immutable length"));
        }
        if first.bytes().len() == length {
            let bytes = first.try_map(|batch| Ok::<_, Error>(batch.bytes))?;
            if reader.next().await?.is_some() || reader.completion()?.bytes != file.length {
                return Err(Error::Invalid("file completion differs from immutable length"));
            }
            return Ok(bytes);
        }
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(length).map_err(|_| Error::Invalid("file byte allocation failed"))?;
        bytes.extend_from_slice(first.bytes());
        drop(first);
        while let Some(batch) = reader.next().await? {
            if batch.offset() != u64::try_from(bytes.len()).map_err(|_| Error::Invalid("file byte offset overflows"))? {
                return Err(Error::Invalid("file pages are not contiguous"));
            }
            if bytes.len().checked_add(batch.bytes().len()).is_none_or(|end| end > length) {
                return Err(Error::Invalid("file pages exceed immutable length"));
            }
            bytes.extend_from_slice(batch.bytes());
        }
        let completion = reader.completion()?;
        if completion.bytes != file.length || bytes.len() != length {
            return Err(Error::Invalid("file completion differs from immutable length"));
        }
        return Ok(harmos::storage::ReadBatch::new(bytes));
    }
    if length != 0 || reader.completion()?.bytes != 0 {
        return Err(Error::Invalid("empty file completion differs from immutable length"));
    }
    Ok(harmos::storage::ReadBatch::new(Vec::new()))
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use harmos::{Identified, schema::Described};

    static SHAPE_READS: AtomicUsize = AtomicUsize::new(0);

    struct Counted;

    impl Described for Counted {
        const NAME: &'static str = "Counted";
        const FIELDS: &'static [harmos::schema::declaration::Field] = &[];
        const MARKER: &'static [u8] = b"";
        const DEFINITION: harmos::schema::declaration::Definition = harmos::schema::declaration::Definition {
            fields: || {
                SHAPE_READS.fetch_add(1, Ordering::Relaxed);
                Self::FIELDS
            },
            ..harmos::schema::declaration::Definition::of::<Self>()
        };
    }

    super::object_identity!(Counted, "test.object.counted");

    #[test]
    fn object_identity_reuses_the_derived_revision_across_threads() {
        let schema =
            harmos::schema::graph::Schema::from_declaration(Counted::DEFINITION, &harmos::config::SchemaSettings::default()).unwrap();
        let derived = harmos::schema::identity::revision(&schema).unwrap();
        let identity = Counted::definition();
        assert_eq!(identity.1, derived);
        let reads = SHAPE_READS.load(Ordering::Relaxed);
        assert!(reads > 0);
        std::thread::scope(|scope| {
            for _ in 0..8 {
                let identity = &identity;
                scope.spawn(move || {
                    for _ in 0..16 {
                        assert_eq!(&Counted::definition(), identity);
                    }
                });
            }
        });
        assert_eq!(SHAPE_READS.load(Ordering::Relaxed), reads);
    }
}
