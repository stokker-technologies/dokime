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
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::Arc,
};
// Third Party Imports
use harmos::storage::*;
use rusqlite::{Connection, params};
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
// Local Imports
use super::super::{Error, State, memory, query};
use super::{blob, bootstrap, entry, files};

/* === Constants === */
pub(super) const METADATA_BYTES: usize = bootstrap::METADATA_BYTES;
pub(super) const CHUNK_BYTES: usize = 64 * 1024;
pub(super) const WORKING_BYTES: usize = 12 * METADATA_BYTES + 2 * CHUNK_BYTES;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Chunk<'a> {
    pub offset: u64,
    pub bytes: &'a [u8],
}

pub(super) struct Finish {
    pub position: StorageCursor,
    pub length: u64,
    pub sha256: [u8; 32],
}

struct Stage {
    metadata: FileMetadata,
    intent: Option<FileReference>,
    finished: Option<FileReference>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl State {
    pub(super) async fn data_create_bootstrap_file(
        &self,
        root: PathBuf,
        identity: BootstrapId,
        metadata: FileMetadata,
        execution: Arc<query::Budget>,
        reservation: memory::Reservation,
    ) -> Result<StagedFile, Error> {
        let limits = execution.limits;
        self.call_read(Some(execution), move |connection| {
            let _reservation = reservation;
            Ok((|| {
                matching(connection, &identity, Some(limits))?;
                entry::key(metadata.id.as_str())?;
                if let Some(scope) = &metadata.scope {
                    entry::key(scope.as_str())?;
                }
                let owner = entry::encode(&identity, METADATA_BYTES)?;
                let encoded = entry::encode(&metadata, METADATA_BYTES)?;
                let staged = StagedFile {
                    history: identity.cut().history.clone(),
                    id: metadata.id.clone(),
                };
                let stage = match load(connection, &identity, &staged)? {
                    Some(stage) => {
                        if stage.metadata != metadata {
                            return Err(Error::Conflict("private file metadata differs"));
                        }
                        stage
                    }
                    None => {
                        // Persist Ownership Before Any Filesystem Mutation
                        connection.execute(
                            "INSERT INTO storage_bootstrap_files(id,owner,metadata) VALUES (?1,?2,?3)",
                            params![metadata.id.as_str(), owner, encoded],
                        )?;
                        Stage { metadata, intent: None, finished: None }
                    }
                };
                if stage.intent.is_none() && stage.finished.is_none() {
                    let path = path(connection, &root, &identity, &staged.id, false)?;
                    dokime_filesystem::publication::create_dir_all(path.parent().expect("private file parent"))?;
                    files::regular(&path)?;
                    let file = OpenOptions::new().create(true).truncate(false).write(true).open(&path)?;
                    file.sync_all()?;
                    files::confirm_durability(&path)?;
                }
                Ok(staged)
            })())
        }).await?
    }

    pub(super) async fn data_write_bootstrap_file(
        &self,
        root: PathBuf,
        identity: BootstrapId,
        staged: StagedFile,
        chunk: Chunk<'_>,
        execution: Arc<query::Budget>,
        preparation: memory::Reservation,
    ) -> Result<(), Error> {
        let Chunk { offset, bytes } = chunk;
        if bytes.is_empty() || bytes.len() > CHUNK_BYTES {
            return Err(Error::Data(
                "private file chunk must contain 1 to 65536 bytes",
            ));
        }
        let (reservation, bytes) = execution.prepare(|| {
            let reservation = self.reserve_query(2 * bytes.len(), execution.limits.memory_bytes)?;
            Ok((reservation, bytes.to_vec()))
        })?;
        let limits = execution.limits;
        self.call_read(Some(execution), move |connection| {
            let _preparation = preparation;
            let _reservation = reservation;
            Ok((|| {
                matching(connection, &identity, Some(limits))?;
                let stage = load(connection, &identity, &staged)?
                    .ok_or(Error::Data("unknown private file staging"))?;
                let end = offset
                    .checked_add(bytes.len() as u64)
                    .ok_or(Error::Data("private file offset overflow"))?;
                if stage
                    .metadata
                    .expected_length
                    .is_some_and(|length| end > length)
                {
                    return Err(Error::Data("private file chunk exceeds expected length"));
                }
                let sealed = stage.finished.as_ref().or(stage.intent.as_ref());
                let installed = path(connection, &root, &identity, &staged.id, true)?;
                let target = if sealed.is_some() && files::regular(&installed)? {
                    installed
                } else {
                    path(connection, &root, &identity, &staged.id, false)?
                };
                files::regular(&target)?;
                let mut file = OpenOptions::new()
                    .read(true)
                    .write(sealed.is_none())
                    .open(target)?;
                let length = file.metadata()?.len();
                if offset > length || sealed.is_some() && end > length {
                    return Err(Error::Data(
                        "private file write has a gap or grows a sealed file",
                    ));
                }
                let overlap = (length - offset).min(bytes.len() as u64) as usize;
                file.seek(SeekFrom::Start(offset))?;
                let mut existing = vec![0; overlap];
                file.read_exact(&mut existing)?;
                if existing != bytes[..overlap] {
                    return Err(Error::Conflict("private file retry bytes differ"));
                }
                if sealed.is_none() {
                    file.write_all(&bytes[overlap..])?;
                    file.sync_all()?;
                }
                query::checkpoint()?;
                Ok(())
            })())
        })
        .await?
    }

    pub(super) async fn data_finish_bootstrap_file(
        &self,
        root: PathBuf,
        identity: BootstrapId,
        staged: StagedFile,
        finish: Finish,
        execution: Arc<query::Budget>,
        reservation: memory::Reservation,
    ) -> Result<FileReference, Error> {
        let Finish {
            position,
            length,
            sha256,
        } = finish;
        let limits = execution.limits;
        self.call_read(Some(execution), move |connection| {
            let _reservation = reservation;
            Ok((|| {
                matching(connection, &identity, Some(limits))?;
                identity.cut().history.check(&position.history)?;
                if position.sequence == 0 || !identity.cut().covers(&position)? {
                    return Err(Error::Data(
                        "private file position lies outside its import cut",
                    ));
                }
                let stage = load(connection, &identity, &staged)?
                    .ok_or(Error::Data("unknown private file staging"))?;
                let reference = FileReference {
                    metadata: stage.metadata,
                    position,
                    length,
                    sha256,
                };
                if reference
                    .metadata
                    .expected_length
                    .is_some_and(|expected| expected != length)
                    || reference
                        .metadata
                        .expected_sha256
                        .is_some_and(|expected| expected != sha256)
                    || stage
                        .intent
                        .as_ref()
                        .is_some_and(|intent| intent != &reference)
                    || stage
                        .finished
                        .as_ref()
                        .is_some_and(|finished| finished != &reference)
                {
                    return Err(Error::Conflict(
                        "private file finish differs from metadata or prior finish",
                    ));
                }
                let encoded = entry::encode(&reference, METADATA_BYTES)?;
                let source = path(connection, &root, &identity, &staged.id, false)?;
                let destination = path(connection, &root, &identity, &staged.id, true)?;
                let present = files::regular(&destination)?;
                // Refused Bytes Do Not Poison The Original Finish Intent
                verify(if present { &destination } else { &source }, &reference)?;
                connection.execute(
                    "UPDATE storage_bootstrap_files SET finish_intent=?1 WHERE id=?2",
                    params![&encoded, staged.id.as_str()],
                )?;
                dokime_filesystem::publication::create_dir_all(
                    destination.parent().expect("private file parent"),
                )?;
                if !present {
                    dokime_filesystem::publication::install(&source, &destination)?;
                }
                files::confirm_durability(&destination)?;
                query::checkpoint()?;
                connection.execute(
                    "UPDATE storage_bootstrap_files SET finished=?1 WHERE id=?2",
                    params![encoded, staged.id.as_str()],
                )?;
                remove(&source)?;
                Ok(reference)
            })())
        })
        .await?
    }

    pub(super) async fn data_abort_bootstrap_file(
        &self,
        root: PathBuf,
        identity: BootstrapId,
        staged: StagedFile,
        reservation: memory::Reservation,
    ) -> Result<(), Error> {
        self.call(move |connection| {
            let _reservation = reservation;
            Ok((|| {
                matching(connection, &identity, None)?;
                abort_file(connection, &root, &identity, &staged, true)
            })())
        })
        .await?
    }
}

/* === Public Functions === */
pub(super) fn install(
    connection: &Connection,
    root: &Path,
    identity: &BootstrapId,
) -> Result<(), Error> {
    matching(connection, identity, None)?;
    let mut after = String::new();
    loop {
        query::checkpoint()?;
        let mut statement = connection.prepare(
            "SELECT id,finished FROM storage_bootstrap_files WHERE id>?1 ORDER BY id LIMIT 1",
        )?;
        let mut rows = statement.query([&after])?;
        let Some(row) = rows.next()? else {
            return Ok(());
        };
        let reference: FileReference = decode(blob(row, 1)?)?;
        let indexed = row
            .get_ref(0)?
            .as_str()
            .map_err(|_| Error::Corrupt("private file ID is not text"))?;
        if indexed != reference.metadata.id.as_str() {
            return Err(Error::Corrupt("private install file index differs"));
        }
        after = indexed.to_owned();
        drop(rows);
        drop(statement);

        confirm(connection, root, identity, &reference)?;
        if let Some((metadata, finished)) = files::metadata_for(connection, &reference.metadata.id)?
        {
            if metadata != reference.metadata || finished.as_ref() != Some(&reference) {
                return Err(Error::Conflict(
                    "private import differs from a prior file owner",
                ));
            }
            files::confirm(connection, root, &reference)?;
        }
        if files::finish_intent(connection, &reference.metadata.id)?
            .as_ref()
            .is_some_and(|value| value != &reference)
        {
            return Err(Error::Conflict(
                "private import differs from a prior file finish",
            ));
        }
        let destination = files::final_path(root, &reference)?;
        let present = files::regular(&destination)?;
        if present {
            files::verify(&destination, &reference)?;
        }
        let encoded = entry::encode(&reference, METADATA_BYTES)?;
        let mut statement = connection
            .prepare("SELECT reference,created FROM storage_bootstrap_installs WHERE id=?1")?;
        let mut rows = statement.query([reference.metadata.id.as_str()])?;
        if let Some(row) = rows.next()? {
            let original: FileReference = decode(blob(row, 0)?)?;
            if original != reference {
                return Err(Error::Conflict("private file install retry differs"));
            }
            if !row.get::<_, bool>(1)? && !present {
                return Err(Error::Corrupt(
                    "prior canonical file disappeared during private import",
                ));
            }
        } else {
            // Retain Prior Canonical Files Across Abort
            connection.execute(
                "INSERT INTO storage_bootstrap_installs VALUES (?1,?2,?3)",
                params![reference.metadata.id.as_str(), encoded, !present],
            )?;
        }
        drop(rows);
        drop(statement);

        if !present {
            let source = path(connection, root, identity, &reference.metadata.id, true)?;
            let copy = install_path(connection, root, identity, &reference.metadata.id)?;
            files::regular(&copy)?;
            let mut input = File::open(&source)?;
            let mut output = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&copy)?;
            let mut buffer = [0; CHUNK_BYTES];
            loop {
                query::checkpoint()?;
                let count = input.read(&mut buffer)?;
                if count == 0 {
                    break;
                }
                output.write_all(&buffer[..count])?;
            }
            output.sync_all()?;
            drop(output);
            files::verify(&copy, &reference)?;
            dokime_filesystem::publication::create_dir_all(
                destination.parent().expect("canonical file parent"),
            )?;
            // Windows Consumes The Copy; The Original Private File Remains Retryable
            dokime_filesystem::publication::install(&copy, &destination)?;
        }
        files::verify(&destination, &reference)?;
        files::confirm_durability(&destination)?;
        remove(&install_path(
            connection,
            root,
            identity,
            &reference.metadata.id,
        )?)?;
    }
}

pub(super) fn working_bytes(root_bytes: usize) -> Result<usize, Error> {
    root_bytes
        .checked_mul(8)
        .and_then(|bytes| bytes.checked_add(WORKING_BYTES))
        .ok_or(Error::Data("private file working allocation overflow"))
}

pub(super) fn cleanup(
    connection: &Connection,
    root: &Path,
    identity: &BootstrapId,
    remove_installed: bool,
) -> Result<(), Error> {
    bootstrap::matching_owner(connection, identity)?;
    loop {
        let mut statement =
            connection.prepare("SELECT id FROM storage_bootstrap_files ORDER BY id LIMIT 1")?;
        let mut rows = statement.query([])?;
        let Some(row) = rows.next()? else {
            let orphaned: bool = connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM storage_bootstrap_installs)",
                [],
                |row| row.get(0),
            )?;
            if orphaned {
                return Err(Error::Corrupt(
                    "private install intent lost its staged owner",
                ));
            }
            return Ok(());
        };
        let raw = row
            .get_ref(0)?
            .as_str()
            .map_err(|_| Error::Corrupt("private file ID is not text"))?;
        entry::key(raw)?;
        let id = raw.to_owned();
        let staged = StagedFile {
            history: identity.cut().history.clone(),
            id: harmos::Identifier::new(id)
                .map_err(|_| Error::Corrupt("private file ID is invalid"))?,
        };
        drop(rows);
        drop(statement);
        abort_file(connection, root, identity, &staged, remove_installed)?;
    }
}

pub(super) fn confirm(
    connection: &Connection,
    root: &Path,
    identity: &BootstrapId,
    reference: &FileReference,
) -> Result<(), Error> {
    matching(connection, identity, None)?;
    let staged = StagedFile {
        history: reference.position.history.clone(),
        id: reference.metadata.id.clone(),
    };
    let stage = load(connection, identity, &staged)?
        .ok_or(Error::Data("private file reference has no staged bytes"))?;
    if stage.finished.as_ref() != Some(reference) {
        return Err(Error::Conflict(
            "private file reference differs from finished bytes",
        ));
    }
    let destination = path(connection, root, identity, &staged.id, true)?;
    verify(&destination, reference)?;
    files::confirm_durability(&destination)?;
    Ok(())
}

/* === Private Functions === */
fn matching(
    connection: &Connection,
    identity: &BootstrapId,
    limits: Option<QueryLimits>,
) -> Result<(), Error> {
    let import = bootstrap::matching(connection, identity)?;
    if limits.is_some_and(|limits| limits != import.limits) {
        return Err(Error::Corrupt(
            "private file policy changed after admission",
        ));
    }
    Ok(())
}

fn load(
    connection: &Connection,
    identity: &BootstrapId,
    staged: &StagedFile,
) -> Result<Option<Stage>, Error> {
    identity.cut().history.check(&staged.history)?;
    super::production::check_available(connection, staged.id.as_str())?;
    entry::key(staged.id.as_str())?;
    let mut statement = connection.prepare(
        "SELECT owner,metadata,finish_intent,finished FROM storage_bootstrap_files WHERE id=?1",
    )?;
    let mut rows = statement.query([staged.id.as_str()])?;
    let Some(row) = rows.next()? else {
        return Ok(None);
    };
    let owner: BootstrapId = decode(blob(row, 0)?)?;
    if owner != *identity {
        return Err(Error::Conflict("private file belongs to another import"));
    }
    let metadata: FileMetadata = decode(blob(row, 1)?)?;
    if metadata.id != staged.id {
        return Err(Error::Corrupt("private file metadata identity differs"));
    }
    let reference = |column| -> Result<Option<FileReference>, Error> {
        if row.get_ref(column)?.data_type() == rusqlite::types::Type::Null {
            return Ok(None);
        }
        let reference: FileReference = decode(blob(row, column)?)?;
        if reference.metadata != metadata
            || reference.position.sequence == 0
            || !identity.cut().covers(&reference.position)?
        {
            return Err(Error::Corrupt(
                "private finished file identity or metadata differs",
            ));
        }
        Ok(Some(reference))
    };
    let intent = reference(2)?;
    let finished = reference(3)?;
    if finished.is_some() && intent != finished {
        return Err(Error::Corrupt(
            "private file finish differs from durable intent",
        ));
    }
    Ok(Some(Stage {
        metadata,
        intent,
        finished,
    }))
}

fn decode<T: DeserializeOwned + Serialize>(body: &[u8]) -> Result<T, Error> {
    if body.len() > METADATA_BYTES {
        return Err(Error::Corrupt("private file metadata exceeds its bound"));
    }
    let decoded: T = serde_json::from_slice(body)?;
    if entry::encode(&decoded, METADATA_BYTES)? != body {
        return Err(Error::Corrupt("private file metadata is not canonical"));
    }
    Ok(decoded)
}

fn path(
    connection: &Connection,
    root: &Path,
    identity: &BootstrapId,
    id: &harmos::Identifier,
    finished: bool,
) -> Result<PathBuf, Error> {
    let owner = entry::encode(identity, METADATA_BYTES)?;
    let digest = Sha256::digest(owner);
    let namespace: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    files::stage_path(
        connection,
        root,
        &namespace,
        id.as_str(),
        if finished { "sealed" } else { "write" },
    )
}

fn verify(path: &Path, reference: &FileReference) -> Result<(), Error> {
    files::regular(path)?;
    let mut file = File::open(path)?;
    if file.metadata()?.len() != reference.length {
        return Err(Error::Data("private file finish length differs"));
    }
    let mut digest = Sha256::new();
    let mut buffer = [0; CHUNK_BYTES];
    loop {
        query::checkpoint()?;
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    if digest.finalize().as_slice() != reference.sha256.as_slice() {
        return Err(Error::Data("private file finish hash differs"));
    }
    Ok(())
}

fn abort_file(
    connection: &Connection,
    root: &Path,
    identity: &BootstrapId,
    staged: &StagedFile,
    remove_installed: bool,
) -> Result<(), Error> {
    // Reconcile The Exact Owner Even For Already-Finished Private Bytes
    let Some(stage) = load(connection, identity, staged)? else {
        return Ok(());
    };
    let mut statement = connection
        .prepare("SELECT reference,created FROM storage_bootstrap_installs WHERE id=?1")?;
    let mut rows = statement.query([staged.id.as_str()])?;
    if let Some(row) = rows.next()? {
        let reference: FileReference = decode(blob(row, 0)?)?;
        if reference.metadata.id != staged.id
            || !identity.cut().covers(&reference.position)?
            || stage.finished.as_ref() != Some(&reference)
        {
            return Err(Error::Corrupt("private install intent identity differs"));
        }
        if remove_installed && row.get::<_, bool>(1)? {
            // Transfer Abort Ownership Before Removing An Install's Intent.
            // Identical Private Files Share One Newly Created Canonical Blob.
            connection.execute("UPDATE storage_bootstrap_installs SET created=1 WHERE id<>?1 AND json_extract(reference,'$.sha256')=?2",
                params![staged.id.as_str(),serde_json::to_string(&reference.sha256)?])?;
        }
        if remove_installed
            && row.get::<_, bool>(1)?
            && !files::shared_bytes(
                connection,
                &reference.sha256,
                None,
                Some(staged.id.as_str()),
            )?
        {
            if files::metadata_for(connection, &staged.id)?
                .is_some_and(|(_, finished)| finished.is_some())
            {
                return Err(Error::Conflict(
                    "published file cannot be removed by private import abort",
                ));
            }
            let destination = files::final_path(root, &reference)?;
            if files::regular(&destination)? {
                files::verify(&destination, &reference)?;
            }
            remove(&destination)?;
        }
    }
    drop(rows);
    drop(statement);
    remove(&install_path(connection, root, identity, &staged.id)?)?;
    remove(&path(connection, root, identity, &staged.id, true)?)?;
    remove(&path(connection, root, identity, &staged.id, false)?)?;
    connection.execute(
        "DELETE FROM storage_bootstrap_installs WHERE id=?1",
        [staged.id.as_str()],
    )?;
    connection.execute(
        "DELETE FROM storage_bootstrap_files WHERE id=?1",
        [staged.id.as_str()],
    )?;
    let owner = entry::encode(identity, METADATA_BYTES)?;
    let namespace: String = Sha256::digest(owner)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    connection.execute(
        "DELETE FROM storage_staging WHERE owner=?1 AND id=?2",
        params![namespace, staged.id.as_str()],
    )?;
    Ok(())
}

fn install_path(
    connection: &Connection,
    root: &Path,
    identity: &BootstrapId,
    id: &harmos::Identifier,
) -> Result<PathBuf, Error> {
    let owner = entry::encode(identity, METADATA_BYTES)?;
    let namespace: String = Sha256::digest(owner)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    files::stage_path(connection, root, &namespace, id.as_str(), "install")
}

fn remove(path: &Path) -> Result<(), Error> {
    files::regular(path)?;
    dokime_filesystem::publication::remove(path)?;
    Ok(())
}

/* === Tests === */
