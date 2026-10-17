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
    fs::{File, OpenOptions},
    io::{Read as _, Seek, SeekFrom, Write},
    num::NonZeroUsize,
    path::{Path, PathBuf},
};
// Third Party Imports
use harmos::storage::*;
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256, digest::common::hazmat::SerializableState};
// Local Imports
use super::super::{Error, State};
use super::{blob, entry, saved};

/* === Constants === */
const METADATA_BYTES: usize = 16 * 1024;
const CHUNK_BYTES: usize = 64 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl State {
    pub(crate) async fn data_create_file(&self, root: PathBuf, metadata: FileMetadata) -> Result<StagedFile, Error> {
        self.call(move |connection| Ok(create_file(connection, &root, metadata, false))).await?
    }

    pub(crate) async fn data_write_file(&self, root: PathBuf, staging: StagedFile, offset: u64, bytes: Vec<u8>) -> Result<(), Error> {
        if bytes.is_empty() {
            return Err(Error::Data("file batch must not be empty"));
        }
        self.call(move |connection| {
            Ok((|| {
                super::production::check_writable(connection, staging.id.as_str())?;
                saved(connection)?.history.check(&staging.history)?;
                let (metadata, finished) = metadata_for(connection, &staging.id)?.ok_or(Error::Data("unknown file staging"))?;
                let end = offset.checked_add(bytes.len() as u64).ok_or(Error::Data("file offset overflow"))?;
                if metadata.expected_length.is_some_and(|length| end > length) {
                    return Err(Error::Data("file chunk exceeds expected length"));
                }
                let intent = finish_intent(connection, &staging.id)?;
                let sealed = finished.as_ref().or(intent.as_ref());
                let path = if let Some(reference) = sealed {
                    let final_path = final_path(&root, reference)?;
                    if regular(&final_path)? { final_path } else { staging_path(connection, &root, &staging.id)? }
                } else {
                    staging_path(connection, &root, &staging.id)?
                };
                regular(&path)?;
                let mut file = OpenOptions::new().read(true).write(sealed.is_none()).open(path)?;
                let length = file.metadata()?.len();
                if offset > length || sealed.is_some() && end > length {
                    return Err(Error::Data("file write has a gap or grows a sealed file"));
                }
                let overlap = (length - offset).min(bytes.len() as u64) as usize;
                file.seek(SeekFrom::Start(offset))?;
                let mut existing = vec![0; overlap];
                file.read_exact(&mut existing)?;
                if existing != bytes[..overlap] {
                    return Err(Error::Conflict("file retry bytes differ"));
                }
                if sealed.is_none() {
                    file.write_all(&bytes[overlap..])?;
                }
                Ok(())
            })())
        })
        .await?
    }

    pub(crate) async fn data_finish_file(
        &self, root: PathBuf, staging: StagedFile, position: StorageCursor, length: u64, sha256: [u8; 32],
    ) -> Result<FileReference, Error> {
        self.call(move |connection| {
            Ok((|| {
                super::production::check_writable(connection, staging.id.as_str())?;
                let head = saved(connection)?;
                head.history.check(&staging.history)?;
                head.history.check(&position.history)?;
                if position.sequence == 0 {
                    return Err(Error::Data("position zero cannot reference a file"));
                }
                let (metadata, finished) = metadata_for(connection, &staging.id)?.ok_or(Error::Data("unknown file staging"))?;
                let reference = FileReference { metadata, position, length, sha256 };
                if reference.metadata.expected_length.is_some_and(|value| value != length)
                    || reference.metadata.expected_sha256.is_some_and(|value| value != sha256)
                    || finished.as_ref().is_some_and(|value| value != &reference)
                {
                    return Err(Error::Conflict("file finish conflicts with metadata or prior finish"));
                }
                let encoded = entry::encode(&reference, METADATA_BYTES)?;
                if finish_intent(connection, &staging.id)?.as_ref().is_some_and(|intent| intent != &reference) {
                    return Err(Error::Conflict("file finish intent has different contents"));
                }
                let source = staging_path(connection, &root, &staging.id)?;
                let destination = final_path(&root, &reference)?;
                let present = regular(&destination)?;
                verify(if present { &destination } else { &source }, &reference)?;
                // Invalid length/hash claims do not bind a finish identity.
                // Persist the verified intent before changing file visibility.
                connection
                    .execute("UPDATE storage_file_stages SET finish_intent=?1 WHERE id=?2", params![&encoded, staging.id.as_str()])?;
                harmos::filesystem::publication::create_dir_all(destination.parent().expect("immutable file parent"))?;
                if !present {
                    harmos::filesystem::publication::install(&source, &destination)?;
                }
                // A failed commit leaves an addressable immutable file. The same
                // identity/length/hash retry verifies it and installs the reference.
                confirm_durability(&destination)?;
                let transaction = connection.transaction()?;
                transaction.execute("UPDATE storage_file_stages SET finished=?1 WHERE id=?2", params![encoded, staging.id.as_str()])?;
                transaction.execute("UPDATE storage_productions SET phase=1 WHERE id=?1", [staging.id.as_str()])?;
                transaction.commit()?;
                remove(&source)?;
                Ok(reference)
            })())
        })
        .await?
    }

    pub(crate) async fn data_abort_file(&self, root: PathBuf, staging: StagedFile) -> Result<(), Error> {
        self.call(move |connection| {
            Ok((|| {
                saved(connection)?.history.check(&staging.history)?;
                if metadata_for(connection, &staging.id)?.is_some_and(|(_, finished)| finished.is_some()) {
                    // Aborting cannot remove a finished file or its retry identity.
                    remove(&staging_path(connection, &root, &staging.id)?)?;
                    return Ok(());
                }
                if super::production::checkpoint(connection, &staging)?.is_some() {
                    return Err(Error::Conflict("public production must be abandoned before deletion"));
                }
                if let Some(intent) = finish_intent(connection, &staging.id)? {
                    if connection.query_row(
                        "SELECT EXISTS(SELECT 1 FROM storage_data_entries WHERE kind='file' AND item=?1)",
                        [staging.id.as_str()],
                        |row| row.get::<_, bool>(0),
                    )? {
                        return Err(Error::Data("cannot abort a published file reference"));
                    }
                    if !shared_bytes(connection, &intent.sha256, Some(staging.id.as_str()))? {
                        remove(&final_path(&root, &intent)?)?;
                    }
                }
                remove(&staging_path(connection, &root, &staging.id)?)?;
                connection.execute("DELETE FROM storage_file_stages WHERE id=?1 AND finished IS NULL", [staging.id.as_str()])?;
                connection.execute("DELETE FROM storage_staging WHERE owner='local' AND id=?1", [staging.id.as_str()])?;
                Ok(())
            })())
        })
        .await?
    }

    pub(super) async fn data_read_file(
        &self, root: PathBuf, reference: FileReference, through: u64, offset: u64, maximum: NonZeroUsize,
    ) -> Result<Vec<u8>, Error> {
        let maximum = maximum.get();
        self.read(move |connection| {
            Ok((|| {
                let head = saved(connection)?;
                head.history.check(&reference.position.history)?;
                if through > head.sequence {
                    return Err(Error::Data("file snapshot exceeds saved frontier"));
                }
                // Bind Published Bytes To The Original Saved View
                if reference.position.sequence <= through {
                    let mut statement =
                        connection.prepare("SELECT body,digest FROM storage_data_entries WHERE kind='file' AND position=?1")?;
                    let mut rows = statement.query([super::position(reference.position.sequence)])?;
                    let row = rows.next()?.ok_or(Error::Corrupt("saved file reference is missing"))?;
                    let body = blob(row, 0)?;
                    if body.len() > METADATA_BYTES {
                        return Err(Error::Corrupt("saved file reference exceeds metadata bound"));
                    }
                    super::check_digest(body, blob(row, 1)?)?;
                    let stored: FileReference = serde_json::from_slice(body)?;
                    if stored != reference {
                        return Err(Error::Conflict("file reference differs from saved snapshot"));
                    }
                }
                let (_, finished) = metadata_for(connection, &reference.metadata.id)?.ok_or(Error::Data("unknown immutable file"))?;
                if finished.as_ref() != Some(&reference) {
                    return Err(Error::Conflict("immutable file reference differs"));
                }
                if offset > reference.length {
                    return Err(Error::Data("file read exceeds its length"));
                }
                let path = final_path(&root, &reference)?;
                regular(&path)?;
                let mut file = File::open(path)?;
                if file.metadata()?.len() != reference.length {
                    return Err(Error::Corrupt("immutable file length differs"));
                }
                file.seek(SeekFrom::Start(offset))?;
                let mut bytes = vec![0; (reference.length - offset).min(maximum as u64) as usize];
                file.read_exact(&mut bytes)?;
                Ok(bytes)
            })())
        })
        .await?
    }
}

/* === Public Functions === */

/* === Private Functions === */
pub(super) fn create_file(connection: &mut Connection, root: &Path, metadata: FileMetadata, production: bool) -> Result<StagedFile, Error> {
    entry::key(metadata.id.as_str())?;
    if let Some(scope) = &metadata.scope {
        entry::key(scope.as_str())?;
    }
    let encoded = entry::encode(&metadata, METADATA_BYTES)?;
    super::production::check_writable(connection, metadata.id.as_str())?;
    let head = saved(connection)?;
    let active = super::production::checkpoint(connection, &StagedFile { history: head.history.clone(), id: metadata.id.clone() })?;
    if production {
        if let Some((_, phase, value)) = &active {
            if *phase != FilePhase::Producing || value.metadata != metadata {
                return Err(Error::Conflict("production creation differs or is terminal"));
            }
        } else if metadata_for(connection, &metadata.id)?.is_some() {
            return Err(Error::Conflict("private staging cannot become public production"));
        }
    }
    let staging = StagedFile { history: head.history, id: metadata.id.clone() };
    let transaction = connection.transaction()?;
    if let Some((existing, _)) = metadata_for(&transaction, &staging.id)? {
        if existing != metadata {
            return Err(Error::Conflict("file staging identity has different metadata"));
        }
    } else {
        transaction.execute("INSERT INTO storage_file_stages(id,metadata) VALUES (?1,?2)", params![metadata.id.as_str(), encoded])?;
    }
    if production && active.is_none() {
        let checkpoint = FileCheckpoint {
            metadata: metadata.clone(),
            session: FileSession { history: staging.history.clone(), id: staging.id.clone(), generation: 1 },
            length: 0,
            hash_format: 1,
            hash_state: Sha256::new().serialize().to_vec(),
        };
        transaction.execute(
            "INSERT INTO storage_productions VALUES (?1,?2,0,?3)",
            params![staging.id.as_str(), super::position(1), entry::encode(&checkpoint, METADATA_BYTES)?],
        )?;
    }
    transaction.commit()?;
    let (_, finished) = metadata_for(connection, &staging.id)?.ok_or(Error::Data("file staging disappeared"))?;
    if finished.is_none() && finish_intent(connection, &staging.id)?.is_none() {
        let path = staging_path(connection, root, &staging.id)?;
        harmos::filesystem::publication::create_dir_all(path.parent().expect("staging parent"))?;
        regular(&path)?;
        OpenOptions::new().create(true).truncate(false).write(true).open(&path)?;
        if production {
            confirm_durability(&path)?;
        }
    }
    Ok(staging)
}

pub(super) fn confirm_durability(path: &Path) -> Result<(), Error> {
    Ok(harmos::filesystem::publication::confirm(path)?)
}

pub(super) fn confirm(connection: &Connection, root: &Path, reference: &FileReference) -> Result<(), Error> {
    let (_, finished) = metadata_for(connection, &reference.metadata.id)?.ok_or(Error::Data("file reference has no finished bytes"))?;
    if finished.as_ref() != Some(reference) {
        return Err(Error::Conflict("file publication differs from finished bytes"));
    }
    let path = final_path(root, reference)?;
    regular(&path)?;
    verify(&path, reference)?;
    confirm_durability(&path)?;
    Ok(())
}

pub(super) fn finish_intent(connection: &Connection, id: &harmos::Identifier) -> Result<Option<FileReference>, Error> {
    let mut statement = connection.prepare("SELECT finish_intent FROM storage_file_stages WHERE id=?1")?;
    let mut rows = statement.query([id.as_str()])?;
    let Some(row) = rows.next()? else {
        return Ok(None);
    };
    if row.get_ref(0)?.data_type() == rusqlite::types::Type::Null {
        return Ok(None);
    }
    let body = blob(row, 0)?;
    if body.len() > METADATA_BYTES {
        return Err(Error::Data("file finish intent exceeds its bound"));
    }
    let reference: FileReference = serde_json::from_slice(body)?;
    if reference.metadata.id != *id {
        return Err(Error::Corrupt("file finish intent identity differs"));
    }
    saved(connection)?.history.check(&reference.position.history)?;
    Ok(Some(reference))
}

pub(super) fn metadata_for(
    connection: &Connection, id: &harmos::Identifier,
) -> Result<Option<(FileMetadata, Option<FileReference>)>, Error> {
    super::production::check_available(connection, id.as_str())?;
    stored_metadata(connection, id)
}

pub(super) fn stored_metadata(
    connection: &Connection, id: &harmos::Identifier,
) -> Result<Option<(FileMetadata, Option<FileReference>)>, Error> {
    let mut statement = connection.prepare("SELECT metadata,finished FROM storage_file_stages WHERE id=?1")?;
    let mut rows = statement.query([id.as_str()])?;
    let Some(row) = rows.next()? else {
        return Ok(None);
    };
    let body = blob(row, 0)?;
    if body.len() > METADATA_BYTES {
        return Err(Error::Data("file metadata exceeds its bound"));
    }
    let metadata: FileMetadata = serde_json::from_slice(body)?;
    if metadata.id != *id {
        return Err(Error::Corrupt("file metadata identity differs"));
    }
    let finished = if row.get_ref(1)?.data_type() == rusqlite::types::Type::Null {
        None
    } else {
        let body = blob(row, 1)?;
        if body.len() > METADATA_BYTES {
            return Err(Error::Data("file reference exceeds its bound"));
        }
        let reference: FileReference = serde_json::from_slice(body)?;
        if reference.metadata != metadata {
            return Err(Error::Corrupt("sealed file metadata differs"));
        }
        saved(connection)?.history.check(&reference.position.history)?;
        Some(reference)
    };
    Ok(Some((metadata, finished)))
}

pub(super) fn staging_path(connection: &Connection, root: &Path, id: &harmos::Identifier) -> Result<PathBuf, Error> {
    stage_path(connection, root, "local", id.as_str(), "write")
}

pub(super) fn stage_path(connection: &Connection, root: &Path, owner: &str, id: &str, phase: &str) -> Result<PathBuf, Error> {
    connection.execute(
        "INSERT OR IGNORE INTO storage_staging VALUES (?1,?2,?3,?4)",
        params![owner, id, phase, uuid::Uuid::now_v7().to_string()],
    )?;
    let name: String = connection.query_row(
        "SELECT uuid FROM storage_staging WHERE owner=?1 AND id=?2 AND phase=?3",
        params![owner, id, phase],
        |row| row.get(0),
    )?;
    let uuid = uuid::Uuid::parse_str(&name).map_err(|_| Error::Corrupt("invalid staging UUID"))?;
    if uuid.to_string() != name {
        return Err(Error::Corrupt("noncanonical staging UUID"));
    }
    let path = root.join("staging").join(name);
    ancestors(root, &path)?;
    Ok(path)
}

pub(crate) fn storage_key(reference: &FileReference) -> String {
    format!("files/{}", hex(&reference.sha256))
}

pub(super) fn shared_bytes(connection: &Connection, sha256: &[u8; 32], local_id: Option<&str>) -> Result<bool, Error> {
    let hash = serde_json::to_string(sha256)?;
    Ok(connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM storage_file_stages WHERE NOT EXISTS(SELECT 1 FROM storage_deleted_files WHERE storage_deleted_files.id=storage_file_stages.id) AND (?2 IS NULL OR id<>?2)
            AND (json_extract(finished,'$.sha256')=?1 OR json_extract(finish_intent,'$.sha256')=?1))",
        params![hash, local_id], |row| row.get(0),
    )?)
}

pub(super) fn final_path(root: &Path, reference: &FileReference) -> Result<PathBuf, Error> {
    let path = root.join(storage_key(reference));
    ancestors(root, &path)?;
    Ok(path)
}
pub(super) fn ancestors(root: &Path, path: &Path) -> Result<(), Error> {
    let relative = path.strip_prefix(root).map_err(|_| Error::Data("file path escapes its store"))?;
    let mut current = root.to_path_buf();
    regular(&current)?;
    for component in relative.components() {
        current.push(component);
        regular(&current)?;
    }
    Ok(())
}
pub(super) fn regular(path: &Path) -> Result<bool, Error> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(Error::Data("store file path contains a symlink")),
        Ok(metadata) if metadata.is_file() || metadata.is_dir() => Ok(true),
        Ok(_) => Err(Error::Data("store file path contains a special file")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}
pub(super) fn verify(path: &Path, reference: &FileReference) -> Result<(), Error> {
    regular(path)?;
    let mut file = File::open(path)?;
    if file.metadata()?.len() != reference.length {
        return Err(Error::Data("file finish length differs"));
    }
    let mut digest = Sha256::new();
    let mut buffer = [0; CHUNK_BYTES];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    if digest.finalize().as_slice() != reference.sha256.as_slice() {
        return Err(Error::Data("file finish hash differs"));
    }
    Ok(())
}
pub(super) fn remove(path: &Path) -> Result<(), Error> {
    regular(path)?;
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/* === Tests === */
