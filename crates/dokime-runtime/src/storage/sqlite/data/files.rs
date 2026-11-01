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
    path::{Path, PathBuf},
};

// Third Party Imports
use harmos::storage::*;
use rusqlite::{Connection, OptionalExtension, params};
use sha2::{Digest, Sha256, digest::common::hazmat::SerializableState};

// Local Imports
use super::super::{Error, State};
use super::{blob, entry, saved};

/* === Constants === */

const CHUNK_BYTES: usize = 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// One Exact First Block And Its Digest State; At Most CHUNK_BYTES Per Store.
#[derive(Default)]
pub(in crate::storage::sqlite) struct VerifiedPrefix {
    bytes: Vec<u8>,
    digest: Sha256,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl VerifiedPrefix {
    fn begin(&mut self, bytes: &[u8]) -> Sha256 {
        // Reuse Hash Work Only After Comparing The Bytes Actually Read.
        if self.bytes != bytes {
            self.bytes.clear();
            self.bytes.extend_from_slice(bytes);
            self.digest = Sha256::new();
            self.digest.update(bytes);
        }
        self.digest.clone()
    }
}

impl State {
    pub(crate) async fn data_create_file(&self, root: PathBuf, metadata: FileMetadata) -> Result<StagedFile, Error> {
        let owner = self.0.clone();
        self.call(move |connection| {
            let result = (|| {
                let staging = create_file(connection, &root, metadata, false)?;
                if metadata_for(connection, &staging.id)?.is_some_and(|(_, finished)| finished.is_none()) {
                    let path = staging_path(connection, &root, &staging.id)?;
                    let file = OpenOptions::new().read(true).write(true).open(path)?;
                    owner.files.lock().unwrap().insert(staging.id.clone(), file);
                }
                Ok(staging)
            })();
            Ok(result)
        })
        .await?
    }

    pub(crate) async fn data_write_file(&self, root: PathBuf, staging: StagedFile, offset: u64, bytes: FileChunk) -> Result<(), Error> {
        if bytes.is_empty() {
            return Err(Error::Data("file batch must not be empty"));
        }
        let owner = self.0.clone();
        self.call(move |connection| {
            Ok((|| {
                let bytes = bytes.as_slice();
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
                let mut files = owner.files.lock().unwrap();
                let mut opened;
                let file = if let Some(file) = files.get_mut(&staging.id) {
                    file
                } else {
                    opened = OpenOptions::new().read(true).write(sealed.is_none()).open(path)?;
                    &mut opened
                };
                let length = file.metadata()?.len();
                if offset > length || sealed.is_some() && end > length {
                    return Err(Error::Data("file write has a gap or grows a sealed file"));
                }
                let overlap = (length - offset).min(bytes.len() as u64) as usize;
                file.seek(SeekFrom::Start(offset))?;
                if overlap > 0 {
                    let mut existing = vec![0; CHUNK_BYTES];
                    for expected in bytes[..overlap].chunks(CHUNK_BYTES) {
                        let actual = &mut existing[..expected.len()];
                        file.read_exact(actual)?;
                        if actual != expected {
                            return Err(Error::Conflict("file retry bytes differ"));
                        }
                    }
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
        let owner = self.0.clone();
        self.call(move |connection| {
            let result = (|| {
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
                let encoded = entry::encode(&reference)?;
                if finish_intent(connection, &staging.id)?.as_ref().is_some_and(|intent| intent != &reference) {
                    return Err(Error::Conflict("file finish intent has different contents"));
                }
                let source = staging_path(connection, &root, &staging.id)?;
                let destination = final_path(&root, &reference)?;
                let present = regular(&destination)?;
                let mut retained = owner.files.lock().unwrap();
                if present {
                    retained.remove(&staging.id);
                    verify(&destination, &reference)?;
                } else if let Some(file) = retained.get_mut(&staging.id) {
                    let mut prefix = owner.verification_prefix.lock().unwrap();
                    verify_opened(file, &reference, &mut prefix)?;
                } else {
                    verify(&source, &reference)?;
                }
                // Invalid length/hash claims do not bind a finish identity.
                // Persist the verified intent before changing file visibility.
                connection
                    .execute("UPDATE storage_file_stages SET finish_intent=?1 WHERE id=?2", params![&encoded, staging.id.as_str()])?;
                harmos::filesystem::publication::create_dir_all(destination.parent().expect("immutable file parent"))?;
                if !present {
                    if let Some(file) = retained.get(&staging.id) {
                        harmos::filesystem::publication::install_opened(file, &source, &destination)?;
                    } else {
                        harmos::filesystem::publication::install(&source, &destination)?;
                    }
                    super::super::DURABLE_FILES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                } else if finished.is_none() {
                    // An Interrupted Install May Have Published Its Name Before Its Flush.
                    confirm_durability(&destination)?;
                }
                // The Finished Row Is Committed Only After Durable Publication.
                // Retries Still Verify Content And The Exact Finish Identity.
                let transaction = connection.transaction()?;
                transaction.execute("UPDATE storage_file_stages SET finished=?1 WHERE id=?2", params![encoded, staging.id.as_str()])?;
                transaction.execute("UPDATE storage_productions SET phase=1 WHERE id=?1", [staging.id.as_str()])?;
                super::super::commit(transaction)?;
                // Installation May Move Or Link The Source. Remove A Remaining
                // Link; Reused Content Also Confirms Any Earlier Cleanup Retry.
                if present || regular(&source)? {
                    remove(&source)?;
                }
                Ok(reference)
            })();
            Ok(result)
        })
        .await?
    }

    pub(crate) async fn data_abort_file(&self, root: PathBuf, staging: StagedFile) -> Result<(), Error> {
        let owner = self.0.clone();
        self.call(move |connection| {
            owner.files.lock().unwrap().remove(&staging.id);
            let result = (|| {
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
            })();
            Ok(result)
        })
        .await?
    }

    pub(super) async fn data_open_file(
        &self, root: PathBuf, reference: FileReference, through: u64, lease: impl Send + 'static,
    ) -> Result<File, Error> {
        let expected_length = reference.length;
        let checked_root = root.clone();
        let path = self
            .read(move |connection| {
                let result = (|| {
                    // Both Stored Metadata And A Finished Reference May Coexist During Validation.
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
                    Ok(root.join(storage_key(&reference)))
                })();
                Ok(result)
            })
            .await??;
        tokio::task::spawn_blocking(move || {
            let _lease = lease;
            ancestors(&checked_root, &path)?;
            regular(&path)?;
            let file = File::open(path)?;
            if file.metadata()?.len() != expected_length {
                return Err(Error::Corrupt("immutable file length differs"));
            }
            Ok(file)
        })
        .await
        .map_err(|error| Error::Io(std::io::Error::other(error)))?
    }
}

/* === Public Functions === */

pub(super) fn create_file(connection: &mut Connection, root: &Path, metadata: FileMetadata, production: bool) -> Result<StagedFile, Error> {
    entry::key(metadata.id.as_str())?;
    if let Some(scope) = &metadata.scope {
        entry::key(scope.as_str())?;
    }
    let encoded = entry::encode(&metadata)?;
    super::production::check_writable(connection, metadata.id.as_str())?;
    let head = saved(connection)?;
    let active = super::production::checkpoint(
        connection,
        &StagedFile {
            history: head.history.clone(),
            id: metadata.id.clone(),
        },
    )?;
    if production {
        if let Some((_, phase, value)) = &active {
            if *phase != FilePhase::Producing || value.metadata != metadata {
                return Err(Error::Conflict("production creation differs or is terminal"));
            }
        } else if metadata_for(connection, &metadata.id)?.is_some() {
            return Err(Error::Conflict("private staging cannot become public production"));
        }
    }
    let staging = StagedFile {
        history: head.history,
        id: metadata.id.clone(),
    };
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
            session: FileSession {
                history: staging.history.clone(),
                id: staging.id.clone(),
                generation: 1,
            },
            length: 0,
            hash_format: 1,
            hash_state: Sha256::new().serialize().to_vec(),
        };
        transaction.execute(
            "INSERT INTO storage_productions VALUES (?1,?2,0,?3)",
            params![staging.id.as_str(), super::position(1), entry::encode(&checkpoint)?],
        )?;
    }
    let (_, finished) = metadata_for(&transaction, &staging.id)?.ok_or(Error::Data("file staging disappeared"))?;
    // The Staging Identity Belongs To The Same Creation Transaction.
    let path = if finished.is_none() && finish_intent(&transaction, &staging.id)?.is_none() {
        Some(staging_path(&transaction, root, &staging.id)?)
    } else {
        None
    };
    super::super::commit(transaction)?;
    if let Some(path) = path {
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
    harmos::filesystem::publication::confirm(path)?;
    super::super::DURABLE_FILES.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    Ok(())
}

pub(super) fn confirm(
    connection: &Connection, root: &Path, reference: &FileReference, retained: Option<&mut File>, prefix: &mut VerifiedPrefix,
) -> Result<(), Error> {
    let (_, finished) = metadata_for(connection, &reference.metadata.id)?.ok_or(Error::Data("file reference has no finished bytes"))?;
    if finished.as_ref() != Some(reference) {
        return Err(Error::Conflict("file publication differs from finished bytes"));
    }
    let path = final_path(root, reference)?;
    regular(&path)?;
    // The Finished Row Proves Publication Completed Durably. Verify The Bytes
    // Again Before Admission, Without Copying And Replacing The Immutable File.
    if let Some(file) = retained {
        verify_opened(file, reference, prefix)?;
    } else {
        let mut file = File::open(&path)?;
        verify_opened(&mut file, reference, prefix)?;
    }
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
    let metadata_body = blob(row, 0)?;
    let finished_body = if row.get_ref(1)?.data_type() == rusqlite::types::Type::Null { None } else { Some(blob(row, 1)?) };
    let metadata: FileMetadata = serde_json::from_slice(metadata_body)?;
    if metadata.id != *id {
        return Err(Error::Corrupt("file metadata identity differs"));
    }
    let finished = if let Some(body) = finished_body {
        let reference: FileReference = serde_json::from_slice(body)?;
        if reference.metadata != metadata {
            return Err(Error::Corrupt("sealed file metadata differs"));
        }
        saved(connection)?.history.check(&reference.position.history)?;
        Some(reference)
    } else {
        None
    };
    Ok(Some((metadata, finished)))
}

pub(super) fn staging_path(connection: &Connection, root: &Path, id: &harmos::Identifier) -> Result<PathBuf, Error> {
    stage_path(connection, root, "local", id.as_str(), "write")
}

pub(super) fn stage_path(connection: &Connection, root: &Path, owner: &str, id: &str, phase: &str) -> Result<PathBuf, Error> {
    let name: Option<String> = connection
        .query_row("SELECT uuid FROM storage_staging WHERE owner=?1 AND id=?2 AND phase=?3", params![owner, id, phase], |row| row.get(0))
        .optional()?;
    let name = match name {
        Some(name) => name,
        None => {
            let name = uuid::Uuid::now_v7().to_string();
            connection.execute("INSERT INTO storage_staging VALUES (?1,?2,?3,?4)", params![owner, id, phase, name])?;
            name
        }
    };
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
    verify_opened(&mut file, reference, &mut VerifiedPrefix::default())
}

fn verify_opened(file: &mut File, reference: &FileReference, prefix: &mut VerifiedPrefix) -> Result<(), Error> {
    if file.metadata()?.len() != reference.length {
        return Err(Error::Data("file finish length differs"));
    }
    file.seek(SeekFrom::Start(0))?;
    let mut buffer = vec![0; CHUNK_BYTES];
    let mut count = file.read(&mut buffer)?;
    let mut digest = if count == CHUNK_BYTES { prefix.begin(&buffer) } else { Sha256::new_with_prefix(&buffer[..count]) };
    while count != 0 {
        count = file.read(&mut buffer)?;
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

/* === Private Functions === */

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cached_verification_rejects_changed_prefix_and_suffix() {
        let mut file = tempfile::tempfile().unwrap();
        let bytes = vec![0x5a; CHUNK_BYTES + 32];
        file.write_all(&bytes).unwrap();

        let reference = FileReference {
            position: StorageCursor {
                history: StoreHistory {
                    store: harmos::Identifier::new("fixture.files").unwrap(),
                    epoch: harmos::Identifier::new("fixture.epoch").unwrap(),
                },
                sequence: 1,
            },
            metadata: FileMetadata {
                id: harmos::Identifier::new("fixture.file").unwrap(),
                scope: None,
                media_type: "application/octet-stream".into(),
                expected_length: Some(bytes.len() as u64),
                expected_sha256: None,
            },
            length: bytes.len() as u64,
            sha256: Sha256::digest(&bytes).into(),
        };

        let mut prefix = VerifiedPrefix::default();
        verify_opened(&mut file, &reference, &mut prefix).unwrap();
        verify_opened(&mut file, &reference, &mut prefix).unwrap();

        file.seek(SeekFrom::Start(0)).unwrap();
        file.write_all(&[0]).unwrap();
        assert!(verify_opened(&mut file, &reference, &mut prefix).is_err());

        file.seek(SeekFrom::Start(0)).unwrap();
        file.write_all(&[0x5a]).unwrap();
        file.seek(SeekFrom::End(-1)).unwrap();
        file.write_all(&[0]).unwrap();
        assert!(verify_opened(&mut file, &reference, &mut prefix).is_err());

        file.seek(SeekFrom::End(-1)).unwrap();
        file.write_all(&[0x5a]).unwrap();
        verify_opened(&mut file, &reference, &mut prefix).unwrap();
    }
}
