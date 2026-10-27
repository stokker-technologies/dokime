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
use std::result::Result;
use std::{
    fs::OpenOptions,
    io::{Read, Seek, SeekFrom},
    num::NonZeroUsize,
    path::{Path, PathBuf},
};

// Third Party Imports
use harmos::storage::*;
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{
    Digest, Sha256,
    digest::common::hazmat::{SerializableState, SerializedState},
};

// Local Imports
use super::super::{Error, State};
use super::{entry, files, position, saved, sequence};

/* === Constants === */

pub(super) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS storage_productions (
    id TEXT PRIMARY KEY, generation TEXT NOT NULL CHECK(length(generation)=16),
    phase INTEGER NOT NULL CHECK(phase IN (0,1,2)), checkpoint BLOB NOT NULL
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS storage_deleted_files (
    id TEXT PRIMARY KEY
) WITHOUT ROWID;
";
const CHUNK_BYTES: usize = 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct RecoveryCancellation(std::sync::Arc<std::sync::atomic::AtomicBool>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Drop for RecoveryCancellation {
    fn drop(&mut self) {
        self.0.store(true, std::sync::atomic::Ordering::Release);
    }
}

impl State {
    pub(super) async fn production_create(&self, root: PathBuf, metadata: FileMetadata) -> Result<StagedFile, Error> {
        self.call(move |connection| {
            let result = files::create_file(connection, &root, metadata, true);
            Ok(result)
        })
        .await?
    }

    pub(super) async fn production_fence(&self, file: StagedFile, expected: u64, next: u64) -> Result<(), Error> {
        if expected == 0 || expected.checked_add(1) != Some(next) {
            return Err(Error::Data("invalid producer generation transition"));
        }

        self.call(move |connection| {
            let result = (|| {
                let (generation, phase, _) = checkpoint(connection, &file)?.ok_or(Error::Data("unknown production"))?;
                if phase != FilePhase::Producing || !matches!(generation, value if value == expected || value == next) {
                    return Err(Error::Conflict("producer generation is fenced or terminal"));
                }
                connection
                    .execute("UPDATE storage_productions SET generation=?1 WHERE id=?2", params![position(next), file.id.as_str()])?;
                Ok(())
            })();
            Ok(result)
        })
        .await?
    }

    pub(super) async fn production_save(&self, root: PathBuf, requested: StorageBatch<FileCheckpoint>) -> Result<(), Error> {
        self.call(move |connection| {
            Ok((|| {
                for checkpoint in requested.as_slice() {
                    entry::check_size(checkpoint, usize::MAX)?;
                }
                let transaction = connection.transaction()?;
                for requested in requested.as_slice() {
                    save_checkpoint(&transaction, &root, requested)?;
                }
                super::super::commit(transaction)?;
                Ok(())
            })())
        })
        .await?
    }

    pub(super) async fn production_recover(&self, root: PathBuf, limits: ReadLimits) -> Result<Vec<FileCheckpoint>, Error> {
        let mut checkpoints = Vec::new();
        let mut after = String::new();
        let mut bytes = 0usize;
        loop {
            let cursor = after.clone();
            let root = root.clone();
            let page = self
                .read(move |connection| {
                    Ok((|| -> Result<Vec<_>, Error> {
                        let count: i64 =
                            connection.query_row("SELECT count(*) FROM storage_productions WHERE phase=0", [], |row| row.get(0))?;
                        if count < 0 || count as u64 > limits.rows.get() as u64 {
                            return Err(Error::Data("production recovery exceeds count bound"));
                        }
                        let mut statement =
                            connection.prepare("SELECT id FROM storage_productions WHERE phase=0 AND id>?1 ORDER BY id LIMIT 16")?;
                        let mut rows = statement.query([cursor])?;
                        let mut page = Vec::new();
                        while let Some(row) = rows.next()? {
                            let id = harmos::Identifier::new(entry::key(super::text(row, 0)?)?)
                                .map_err(|_| Error::Corrupt("invalid production identifier"))?;
                            let file = StagedFile { id, history: saved(connection)?.history };
                            let (generation, _, mut value) =
                                checkpoint(connection, &file)?.ok_or(Error::Corrupt("missing production checkpoint"))?;
                            value.session.generation = generation;
                            let path = files::staging_path(connection, &root, &value.session.id)?;
                            page.push((value, path));
                        }
                        Ok(page)
                    })())
                })
                .await??;
            if page.is_empty() {
                break;
            }
            for (value, path) in page {
                bytes = bytes
                    .checked_add(entry::check_size(&value, usize::MAX)?)
                    .filter(|bytes| *bytes <= limits.bytes.get())
                    .ok_or(Error::Data("production recovery exceeds byte bound"))?;
                if checkpoints.len() >= limits.rows.get() {
                    return Err(Error::Data("production recovery exceeds count bound"));
                }
                after = value.session.id.to_string();
                let cancelled = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
                let cancellation = RecoveryCancellation(cancelled.clone());
                let value = tokio::task::spawn_blocking(move || {
                    if !files::regular(&path)? && value.length == 0 {
                        harmos::filesystem::publication::create_dir_all(path.parent().expect("staging parent"))?;
                        OpenOptions::new().write(true).create_new(true).open(&path)?.sync_all()?;
                        files::confirm_durability(&path)?;
                    }
                    let mut physical = OpenOptions::new().read(true).write(true).open(&path)?;
                    verify_prefix_cancellable(&mut physical, &value, None, Some(&cancelled))?;
                    physical.set_len(value.length)?;
                    physical.sync_all()?;
                    Ok::<_, Error>(value)
                })
                .await??;
                drop(cancellation);
                checkpoints.push(value);
                tokio::task::yield_now().await;
            }
        }
        Ok(checkpoints)
    }

    pub(super) async fn production_progress(&self, root: PathBuf, file: StagedFile) -> Result<FileProgress, Error> {
        self.call(move |connection| {
            let result = progress(connection, &root, &file);
            Ok(result)
        })
        .await?
    }

    pub(super) async fn production_read(
        &self, root: PathBuf, file: StagedFile, offset: u64, maximum: NonZeroUsize,
    ) -> Result<Vec<u8>, Error> {
        let maximum = maximum.get();

        self.call(move |connection| {
            let result = (|| {
                let progress = progress(connection, &root, &file)?;
                if offset >= progress.length {
                    return Err(Error::Data("production read is outside its prefix"));
                }
                let path = match files::metadata_for(connection, &file.id)?.and_then(|(_, finished)| finished) {
                    Some(reference) => files::final_path(&root, &reference)?,
                    None => files::staging_path(connection, &root, &file.id)?,
                };
                files::regular(&path)?;
                let mut physical = OpenOptions::new().read(true).open(path)?;
                physical.seek(SeekFrom::Start(offset))?;
                let length = (progress.length - offset).min(maximum as u64) as usize;
                let mut bytes = Vec::new();
                bytes.try_reserve_exact(length).map_err(std::io::Error::other)?;
                bytes.resize(length, 0);
                physical.read_exact(&mut bytes)?;
                Ok(bytes)
            })();
            Ok(result)
        })
        .await?
    }

    pub(super) async fn production_abandon(&self, root: PathBuf, file: StagedFile) -> Result<(), Error> {
        self.call(move |connection| {
            let result = (|| {
                let current = progress(connection, &root, &file)?;
                if current.phase != FilePhase::Producing {
                    return Ok(());
                }
                let path = files::staging_path(connection, &root, &file.id)?;
                files::confirm_durability(&path)?;
                connection.execute("UPDATE storage_productions SET phase=2 WHERE id=?1", [file.id.as_str()])?;
                Ok(())
            })();
            Ok(result)
        })
        .await?
    }
}

fn save_checkpoint(connection: &Connection, root: &Path, requested: &FileCheckpoint) -> Result<(), Error> {
    requested.sha256()?;
    let file = StagedFile { history: requested.session.history.clone(), id: requested.session.id.clone() };
    // A Snapshot Captured While Producing Can Persist After A Longer
    // Save, Completion, Abandonment Or Deletion. Durable State Already
    // Covers That Prefix, So The Late Checkpoint Is A No-Op.
    saved(connection)?.history.check(&file.history)?;
    if deleted(connection, file.id.as_str())? {
        return Ok(());
    }
    let (generation, phase, prior) = stored_checkpoint(connection, &file)?.ok_or(Error::Data("unknown production"))?;
    if requested.metadata != prior.metadata || requested.session.generation != generation {
        return Err(Error::Conflict("checkpoint identity differs or is fenced"));
    }
    match phase {
        FilePhase::Producing if requested.length < prior.length => return Ok(()),
        FilePhase::Producing => {}
        FilePhase::Finished => {
            let finished = files::stored_metadata(connection, &file.id)?
                .and_then(|(_, reference)| reference)
                .ok_or(Error::Corrupt("finished production has no reference"))?;
            if requested.length > finished.length {
                return Err(Error::Conflict("checkpoint exceeds finished production"));
            }
            return Ok(());
        }
        FilePhase::Abandoned => return Ok(()),
    }
    if requested.length == prior.length && requested.hash_state != prior.hash_state {
        return Err(Error::Conflict("checkpoint retry differs"));
    }
    let path = files::staging_path(connection, root, &file.id)?;
    files::regular(&path)?;
    let mut physical = OpenOptions::new().read(true).write(true).open(&path)?;
    verify_prefix(&mut physical, requested, Some(&prior))?;
    // The Identical Prefix Is Already Durable. Preserve Corruption Checks Without
    // Reflushing Every Other Active File On Each Checkpoint Batch.
    if requested == &prior {
        return Ok(());
    }
    let encoded = entry::encode(requested)?;
    physical.sync_all()?;
    files::confirm_durability(&path)?;
    connection.execute("UPDATE storage_productions SET checkpoint=?1 WHERE id=?2", params![encoded, file.id.as_str()])?;
    Ok(())
}

impl State {
    pub(super) async fn production_delete(&self, root: PathBuf, file: StagedFile) -> Result<(), Error> {
        self.call(move |connection| {
            let result = (|| {
            saved(connection)?.history.check(&file.history)?;
            let active: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM storage_productions p JOIN storage_file_stages s ON p.id=s.id WHERE p.id=?1 AND p.phase=0 AND s.finished IS NULL)",[file.id.as_str()],|row|row.get(0))?;
            if active { return Err(Error::Conflict("active production cannot be deleted")); }
            let mut statement = connection.prepare("SELECT coalesce(finished,finish_intent) FROM storage_file_stages WHERE id=?1")?;
            let mut rows = statement.query([file.id.as_str()])?;
            let reference = if let Some(row) = rows.next()? {
                if row.get_ref(0)?.data_type() == rusqlite::types::Type::Null {
                    None
                } else {
                    Some(serde_json::from_slice::<FileReference>(super::blob(row, 0)?)?)
                }
            } else {
                None
            };
            drop(rows);
            drop(statement);
            let transaction = connection.transaction()?;
            let added = transaction.execute("INSERT OR IGNORE INTO storage_deleted_files VALUES (?1)",[file.id.as_str()])?;
            if added!=0 {
                let mut head=saved(&transaction)?;
                head.retention_generation=head.retention_generation.checked_add(1).ok_or(Error::Data("retention generation exhausted"))?;
                transaction.execute("UPDATE storage_data_head SET retention_generation=?1 WHERE singleton=1",params![position(head.retention_generation)])?;
            }
            super::super::commit(transaction)?;
            // Tombstones Fence New Reads Before Physical Reclamation. The
            // Serialized Owner Drains Every Previously Accepted Bounded Read.
            if let Some(reference)=reference
                && !files::shared_bytes(connection,&reference.sha256,Some(file.id.as_str()))? {
                    let path=files::final_path(&root,&reference)?;
                    files::remove(&path)?;
                    #[cfg(unix)]
                if let Some(parent)=path.parent().filter(|path|path.exists()) { std::fs::File::open(parent)?.sync_all()?; }
            }
            let staged: Option<String> = connection.query_row("SELECT uuid FROM storage_staging WHERE owner='local' AND id=?1 AND phase='write'",[file.id.as_str()],|row|row.get(0)).optional()?;
            if let Some(name)=staged {
                let uuid=uuid::Uuid::parse_str(&name).map_err(|_|Error::Corrupt("invalid staging UUID"))?;
                let path=root.join("staging").join(uuid.to_string());
                files::ancestors(&root,&path)?;
                files::remove(&path)?;
                #[cfg(unix)]
                if let Some(parent)=path.parent().filter(|path|path.exists()) { std::fs::File::open(parent)?.sync_all()?; }
            }
            connection.execute("DELETE FROM storage_staging WHERE owner='local' AND id=?1",[file.id.as_str()])?;
            Ok(())
            })();
            Ok(result)
        }).await?
    }
}

/* === Public Functions === */

pub(super) fn check_available(connection: &Connection, id: &str) -> Result<(), Error> {
    if deleted(connection, id)? {
        return Err(Error::Conflict("file identity has been deleted"));
    }
    Ok(())
}

pub(super) fn check_writable(connection: &Connection, id: &str) -> Result<(), Error> {
    check_available(connection, id)?;
    let abandoned: bool =
        connection.query_row("SELECT EXISTS(SELECT 1 FROM storage_productions WHERE id=?1 AND phase=2)", [id], |row| row.get(0))?;
    if abandoned {
        return Err(Error::Conflict("file identity has been abandoned"));
    }
    Ok(())
}

pub(super) fn checkpoint(connection: &Connection, file: &StagedFile) -> Result<Option<(u64, FilePhase, FileCheckpoint)>, Error> {
    check_available(connection, file.id.as_str())?;
    stored_checkpoint(connection, file)
}

pub(super) fn stored_checkpoint(connection: &Connection, file: &StagedFile) -> Result<Option<(u64, FilePhase, FileCheckpoint)>, Error> {
    saved(connection)?.history.check(&file.history)?;
    let mut statement = connection.prepare("SELECT generation,phase,checkpoint FROM storage_productions WHERE id=?1")?;
    let mut rows = statement.query([file.id.as_str()])?;
    let Some(row) = rows.next()? else {
        return Ok(None);
    };
    let generation = sequence(row.get_ref(0)?.as_str().map_err(|_| Error::Corrupt("invalid producer generation"))?)?;
    let phase: u8 = row.get(1)?;
    let body = super::blob(row, 2)?;
    let value: FileCheckpoint = serde_json::from_slice(body)?;
    value.sha256()?;
    value.session.history.check(&file.history)?;
    if value.session.id != file.id || value.session.generation > generation {
        return Err(Error::Corrupt("checkpoint identity or generation differs"));
    }
    let phase = match phase {
        0 => FilePhase::Producing,
        1 => FilePhase::Finished,
        2 => FilePhase::Abandoned,
        _ => return Err(Error::Corrupt("invalid production phase")),
    };
    Ok(Some((generation, phase, value)))
}

/* === Private Functions === */

fn deleted(connection: &Connection, id: &str) -> Result<bool, Error> {
    Ok(connection.query_row("SELECT EXISTS(SELECT 1 FROM storage_deleted_files WHERE id=?1)", [id], |row| row.get(0))?)
}

fn progress(connection: &Connection, root: &Path, file: &StagedFile) -> Result<FileProgress, Error> {
    saved(connection)?.history.check(&file.history)?;
    check_available(connection, file.id.as_str())?;
    if let Some((_, Some(reference))) = files::metadata_for(connection, &file.id)? {
        return Ok(FileProgress { length: reference.length, phase: FilePhase::Finished });
    }
    let (_, phase, _) = checkpoint(connection, file)?.ok_or(Error::Data("unknown production"))?;
    let path = files::staging_path(connection, root, &file.id)?;
    files::regular(&path)?;
    let length = std::fs::metadata(path)?.len();
    Ok(FileProgress { length, phase })
}

fn verify_prefix(file: &mut std::fs::File, checkpoint: &FileCheckpoint, previous: Option<&FileCheckpoint>) -> Result<(), Error> {
    verify_prefix_cancellable(file, checkpoint, previous, None)
}

fn verify_prefix_cancellable(
    file: &mut std::fs::File, checkpoint: &FileCheckpoint, previous: Option<&FileCheckpoint>,
    cancelled: Option<&std::sync::atomic::AtomicBool>,
) -> Result<(), Error> {
    if file.metadata()?.len() < checkpoint.length {
        return Err(Error::Corrupt("saved production prefix is missing"));
    }
    // The Exclusive Writer Only Appends Or Verifies Identical Retry Bytes.
    // Check New Bytes From The Durable State; Recovery Verifies The Full Prefix.
    let offset = previous.map_or(0, |value| value.length);
    let mut digest = match previous {
        Some(value) => {
            let encoded = <&SerializedState<Sha256>>::try_from(value.hash_state.as_slice())
                .map_err(|_| Error::Corrupt("saved checkpoint state width differs"))?;
            Sha256::deserialize(encoded).map_err(|_| Error::Corrupt("saved checkpoint hash state differs"))?
        }
        None => Sha256::new(),
    };
    file.seek(SeekFrom::Start(offset))?;
    let mut remaining = checkpoint.length - offset;
    let mut buffer = vec![0; CHUNK_BYTES];
    while remaining > 0 {
        if cancelled.is_some_and(|cancelled| cancelled.load(std::sync::atomic::Ordering::Acquire)) {
            return Err(Error::Closed);
        }
        let count = remaining.min(CHUNK_BYTES as u64) as usize;
        file.read_exact(&mut buffer[..count])?;
        digest.update(&buffer[..count]);
        remaining -= count as u64;
    }
    if digest.serialize().as_slice() != checkpoint.hash_state.as_slice() {
        return Err(Error::Corrupt("production checkpoint digest differs"));
    }
    Ok(())
}

/* === Tests === */
