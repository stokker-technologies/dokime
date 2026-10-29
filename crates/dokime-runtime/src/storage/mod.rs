/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub mod objects;
pub(crate) mod projection;
mod read_rows;
pub mod records;
pub mod reports;
pub mod requests;
pub mod sqlite;
pub mod telemetry;
mod values;

pub use read_rows::{ReadRows, RowsIter};
pub(crate) use values::{datetime_memory, encode_datetime};

pub type Manager = harmos::storage::StorageManager<objects::Objects>;
pub type Backend = sqlite::Backend<objects::Objects>;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
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

    #[error("storage root is incompatible with this build: {0}; choose a fresh root")]
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
    Publication(#[from] harmos::filesystem::Error),

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
    Timestamp(#[from] time::error::Parse),

    #[error(transparent)]
    DatetimeFormat(#[from] time::error::Format),

    #[error(transparent)]
    Identifier(#[from] harmos::IdentifierError),

    #[error(transparent)]
    Contract(#[from] dokime_domain::Error),
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
