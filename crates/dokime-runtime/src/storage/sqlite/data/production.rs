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
use harmos::{MemorySize, storage::*};
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
const METADATA_BYTES: usize = 16 * 1024;
const CHUNK_BYTES: usize = 64 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl State {
    pub(super) async fn production_create(&self, root: PathBuf, metadata: FileMetadata) -> Result<StagedFile, Error> {
        self.call(move |connection| Ok(files::create_file(connection, &root, metadata, true))).await?
    }

    pub(super) async fn production_fence(&self, file: StagedFile, expected: u64, next: u64) -> Result<(), Error> {
        if expected == 0 || expected.checked_add(1) != Some(next) {
            return Err(Error::Data("invalid producer generation transition"));
        }
        self.call(move |connection| {
            Ok((|| {
                let (generation, phase, _) = checkpoint(connection, &file)?.ok_or(Error::Data("unknown production"))?;
                if phase != FilePhase::Producing || !matches!(generation, value if value == expected || value == next) {
                    return Err(Error::Conflict("producer generation is fenced or terminal"));
                }
                connection
                    .execute("UPDATE storage_productions SET generation=?1 WHERE id=?2", params![position(next), file.id.as_str()])?;
                Ok(())
            })())
        })
        .await?
    }

    pub(super) async fn production_save(&self, root: PathBuf, requested: FileCheckpoint) -> Result<(), Error> {
        requested.sha256()?;
        let encoded = entry::encode(&requested, METADATA_BYTES)?;
        self.call(move |connection| {
            Ok((|| {
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
                let path = files::staging_path(connection, &root, &file.id)?;
                files::regular(&path)?;
                let mut physical = OpenOptions::new().read(true).write(true).open(&path)?;
                verify_prefix(&mut physical, &requested, Some(&prior))?;
                physical.sync_all()?;
                files::confirm_durability(&path)?;
                connection.execute("UPDATE storage_productions SET checkpoint=?1 WHERE id=?2", params![encoded, file.id.as_str()])?;
                Ok(())
            })())
        })
        .await?
    }

    pub(super) async fn production_recover(&self, root: PathBuf, limits: ReadLimits) -> Result<Vec<FileCheckpoint>, Error> {
        self.call(move |connection| {
            Ok((|| {
                let count: i64 = connection.query_row("SELECT count(*) FROM storage_productions WHERE phase=0", [], |row| row.get(0))?;
                if count < 0 || count as u64 > limits.rows.get() as u64 {
                    return Err(Error::Data("production recovery exceeds count bound"));
                }
                let mut statement = connection.prepare("SELECT id FROM storage_productions WHERE phase=0 ORDER BY id")?;
                let mut rows = statement.query([])?;
                let mut checkpoints = Vec::with_capacity(count as usize);
                let mut bytes = 0usize;
                while let Some(row) = rows.next()? {
                    let id =
                        harmos::Identifier::new(row.get::<_, String>(0)?).map_err(|_| Error::Corrupt("invalid production identifier"))?;
                    let file = StagedFile { id, history: saved(connection)?.history };
                    let (generation, _, mut value) =
                        checkpoint(connection, &file)?.ok_or(Error::Corrupt("missing production checkpoint"))?;
                    bytes = bytes
                        .checked_add(value.memory_size())
                        .filter(|bytes| *bytes <= limits.bytes.get())
                        .ok_or(Error::Data("production recovery exceeds byte bound"))?;
                    value.session.generation = generation;
                    checkpoints.push(value);
                }
                for value in &checkpoints {
                    let path = files::staging_path(connection, &root, &value.session.id)?;
                    if !files::regular(&path)? && value.length == 0 {
                        harmos::filesystem::publication::create_dir_all(path.parent().expect("staging parent"))?;
                        OpenOptions::new().write(true).create_new(true).open(&path)?.sync_all()?;
                        files::confirm_durability(&path)?;
                    }
                    let mut physical = OpenOptions::new().read(true).write(true).open(&path)?;
                    verify_prefix(&mut physical, value, None)?;
                    physical.set_len(value.length)?;
                    physical.sync_all()?;
                }
                Ok(checkpoints)
            })())
        })
        .await?
    }

    pub(super) async fn production_progress(&self, root: PathBuf, file: StagedFile) -> Result<FileProgress, Error> {
        self.call(move |connection| Ok(progress(connection, &root, &file))).await?
    }

    pub(super) async fn production_read(
        &self, root: PathBuf, file: StagedFile, offset: u64, maximum: NonZeroUsize,
    ) -> Result<Vec<u8>, Error> {
        let maximum = maximum.get().min(CHUNK_BYTES);
        self.call(move |connection| {
            Ok((|| {
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
                let mut bytes = vec![0; (progress.length - offset).min(maximum as u64) as usize];
                physical.read_exact(&mut bytes)?;
                Ok(bytes)
            })())
        })
        .await?
    }

    pub(super) async fn production_abandon(&self, root: PathBuf, file: StagedFile) -> Result<(), Error> {
        self.call(move |connection| {
            Ok((|| {
                let current = progress(connection, &root, &file)?;
                if current.phase != FilePhase::Producing {
                    return Ok(());
                }
                let path = files::staging_path(connection, &root, &file.id)?;
                files::confirm_durability(&path)?;
                connection.execute("UPDATE storage_productions SET phase=2 WHERE id=?1", [file.id.as_str()])?;
                Ok(())
            })())
        })
        .await?
    }

    pub(super) async fn production_delete(&self, root: PathBuf, file: StagedFile) -> Result<(), Error> {
        self.call(move |connection| Ok((|| {
            saved(connection)?.history.check(&file.history)?;
            let active: bool = connection.query_row("SELECT EXISTS(SELECT 1 FROM storage_productions p JOIN storage_file_stages s ON p.id=s.id WHERE p.id=?1 AND p.phase=0 AND s.finished IS NULL)",[file.id.as_str()],|row|row.get(0))?;
            if active { return Err(Error::Conflict("active production cannot be deleted")); }
            let reference: Option<Vec<u8>> = connection.query_row("SELECT coalesce(finished,finish_intent) FROM storage_file_stages WHERE id=?1",[file.id.as_str()],|row| row.get(0)).optional()?.flatten();
            let reference = reference.map(|body| {
                if body.len()>METADATA_BYTES { return Err(Error::Corrupt("deleted reference exceeds bound")); }
                serde_json::from_slice::<FileReference>(&body).map_err(Error::from)
            }).transpose()?;
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
        })())).await?
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
    if body.len() > METADATA_BYTES {
        return Err(Error::Corrupt("checkpoint exceeds byte bound"));
    }
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
    let mut buffer = [0; CHUNK_BYTES];
    while remaining > 0 {
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
