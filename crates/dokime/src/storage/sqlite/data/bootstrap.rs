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
    num::NonZeroUsize,
    path::{Path, PathBuf},
    sync::Arc,
};

// Third Party Imports
use harmos::storage::ObjectRetentionRelease;
use harmos::{MemorySize, storage::*};
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

// Local Imports
use super::super::{Error, State, memory, query};
use super::{Entry, blob, check_digest, entry, position, saved, sequence};

/* === Constants === */
pub(super) const METADATA_BYTES: usize = 4 * super::HISTORY_BYTES;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(super) struct Import {
    pub identity: BootstrapId,
    pub saved: StorageCursor,
    pub limits: QueryLimits,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Completed {
    import: Import,
    manifest: BootstrapManifest,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl State {
    pub(super) async fn data_bootstrap_releases(
        &self,
        identity: BootstrapId,
        releases: Vec<ObjectRetentionRelease>,
    ) -> Result<(), Error> {
        self.call(move |connection| Ok((|| {
            let transaction = connection.transaction()?;
            matching(&transaction, &identity)?;
            let mut previous = None;
            for release in releases {
                if !identity.cut().covers(&release.through)? {
                    return Err(Error::Data("private release exceeds import cut"));
                }
                let cursor = release.cursor();
                if previous.as_ref().is_some_and(|previous: &harmos::storage::ObjectReleaseCursor| !previous.precedes(&cursor)) {
                    return Err(Error::Data("private release page is unordered"));
                }
                previous = Some(cursor);
                let body = entry::encode(&release, METADATA_BYTES)?;
                transaction.execute(
                    "INSERT INTO storage_bootstrap_releases VALUES (?1,?2,?3,?4,?5)
                     ON CONFLICT(schema_id,schema_version,encoding,item) DO NOTHING",
                    params![release.schema.id.as_str(), release.schema.version.get(),
                        release.schema.payload_encoding as u8, release.item.as_str(), &body])?;
                let stored: Vec<u8> = transaction.query_row(
                    "SELECT body FROM storage_bootstrap_releases WHERE schema_id=?1 AND schema_version=?2 AND encoding=?3 AND item=?4",
                    params![release.schema.id.as_str(), release.schema.version.get(), release.schema.payload_encoding as u8, release.item.as_str()],
                    |row| row.get(0),
                )?;
                if stored != body { return Err(Error::Conflict("private release retry differs")); }
            }
            transaction.commit()?;
            Ok(())
        })())).await?
    }
    pub(super) async fn data_begin_bootstrap(
        &self,
        import: Import,
        execution: Arc<query::Budget>,
    ) -> Result<(), Error> {
        let owner = self.0.clone();
        self.call_read(Some(execution), move |connection| {
            *owner.index_schema.lock().unwrap() = None;
            Ok(begin(connection, &import))
        })
        .await?
    }

    pub(super) async fn data_bootstrap_limits(&self) -> Result<Option<QueryLimits>, Error> {
        self.call(move |connection| {
            Ok((|| {
                let Some(import) = active(connection)? else {
                    if has_rows(connection)? {
                        return Err(Error::Corrupt("private import rows lost their owner"));
                    }
                    return Ok(completed(connection)?.map(|receipt| receipt.import.limits));
                };
                matching_owner(connection, &import.identity)?;
                Ok(Some(import.limits))
            })())
        })
        .await?
    }

    pub(super) async fn data_write_bootstrap(
        &self,
        identity: BootstrapId,
        entries: Vec<entry::Retained>,
        execution: Arc<query::Budget>,
        maximum: NonZeroUsize,
    ) -> Result<(), Error> {
        let limits = execution.limits;
        self.call_read(Some(execution), move |connection| {
            Ok(write(connection, &identity, &entries, limits, maximum))
        })
        .await?
    }

    pub(super) async fn data_finish_bootstrap(
        &self,
        manifest: BootstrapManifest,
        root: PathBuf,
        execution: Arc<query::Budget>,
        maximum: NonZeroUsize,
    ) -> Result<StorageCursor, Error> {
        let memory = self.0.memory.clone();
        let state = self.clone();
        let limits = execution.limits;
        self.call_read(Some(execution), move |connection| {
            *state.0.index_schema.lock().unwrap() = None;
            Ok((|| {
                if let Some(receipt) = completed(connection)?
                    && receipt.import.identity == *manifest.identity()
                {
                    if receipt.manifest != manifest || receipt.import.limits != limits {
                        return Err(Error::Conflict("completed private import retry differs"));
                    }
                    cleanup_completed(connection, &root, &receipt.import.identity)?;
                    return Ok(receipt.import.identity.cut().clone());
                }
                if state.has_data_pins() {
                    return Err(Error::Conflict(
                        "private import visibility waits for existing data pins",
                    ));
                }
                let import = matching(connection, manifest.identity())?;
                if import.limits != limits {
                    return Err(Error::Corrupt(
                        "private import policy changed after admission",
                    ));
                }
                verify_manifest(connection, &root, &manifest, &memory, maximum)?;
                super::bootstrap_files::install(connection, &root, manifest.identity())?;
                publish(connection, &root, &import, &manifest, &memory, maximum)?;
                cleanup_completed(connection, &root, manifest.identity())?;
                Ok(manifest.identity().cut().clone())
            })())
        })
        .await?
    }

    pub(super) async fn data_abort_bootstrap(
        &self,
        identity: BootstrapId,
        root: PathBuf,
    ) -> Result<(), Error> {
        // Cleanup Retains Physical Ownership Without An Execution Cutoff
        self.call(move |connection| Ok(abort(connection, &root, &identity)))
            .await?
    }
}

/* === Public Functions === */
pub(super) fn check_writes(connection: &Connection) -> Result<(), Error> {
    let active: bool = connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM storage_bootstrap) OR EXISTS(SELECT 1 FROM storage_bootstrap_entries) OR EXISTS(SELECT 1 FROM storage_bootstrap_schemas) OR EXISTS(SELECT 1 FROM storage_bootstrap_files) OR EXISTS(SELECT 1 FROM storage_bootstrap_installs) OR EXISTS(SELECT 1 FROM storage_bootstrap_abort)",
        [],
        |row| row.get(0),
    )?;
    if active {
        return Err(Error::Conflict(
            "private replica import fences ordinary publication",
        ));
    }
    Ok(())
}

/* === Private Functions === */
fn publish(
    connection: &mut Connection,
    root: &Path,
    import: &Import,
    manifest: &BootstrapManifest,
    memory: &Arc<memory::Budget>,
    maximum: NonZeroUsize,
) -> Result<(), Error> {
    let receipt = Completed {
        import: import.clone(),
        manifest: manifest.clone(),
    };
    let encoded_receipt = entry::encode(&receipt, 2 * METADATA_BYTES)?;
    let receipt_digest = Sha256::digest(&encoded_receipt);
    let transaction = connection.transaction()?;
    if matching(&transaction, manifest.identity())? != *import {
        return Err(Error::Conflict(
            "private import owner changed before visibility commit",
        ));
    }
    let prior = saved(&transaction)?;
    let generation = prior
        .retention_generation
        .checked_add(1)
        .ok_or(Error::Data("private import retention generation exhausted"))?
        .max(import.identity.retention_generation().get());

    // Adopt Finished File Receipts In The Same Visibility Transaction
    let mut statement =
        transaction.prepare("SELECT metadata,finished FROM storage_bootstrap_files ORDER BY id")?;
    let mut rows = statement.query([])?;
    while let Some(row) = rows.next()? {
        query::checkpoint()?;
        let metadata = blob(row, 0)?;
        let finished = blob(row, 1)?;
        if metadata.len() > METADATA_BYTES || finished.len() > METADATA_BYTES {
            return Err(Error::Corrupt("private file receipt exceeds its bound"));
        }
        let reference: FileReference = serde_json::from_slice(finished)?;
        let changed = transaction.execute(
            "INSERT INTO storage_file_stages VALUES (?1,?2,?3,?3)
             ON CONFLICT(id) DO UPDATE SET finish_intent=excluded.finish_intent,finished=excluded.finished
             WHERE storage_file_stages.metadata=excluded.metadata
                 AND storage_file_stages.finished=excluded.finished
                 AND storage_file_stages.finish_intent=excluded.finish_intent",
            params![reference.metadata.id.as_str(), metadata, finished],
        )?;
        if changed != 1 {
            return Err(Error::Conflict(
                "private file receipt conflicts with prior owner",
            ));
        }
        super::files::confirm(&transaction, root, &reference)?;
    }
    drop(rows);
    drop(statement);

    transaction.execute("DELETE FROM storage_object_releases", [])?;
    let mut release_statement = transaction.prepare("SELECT body FROM storage_bootstrap_releases ORDER BY schema_id,schema_version,encoding,item")?;
    let mut release_rows = release_statement.query([])?;
    while let Some(row) = release_rows.next()? {
        query::checkpoint()?;
        let body = blob(row, 0)?;
        if body.len() > METADATA_BYTES {
            return Err(Error::Corrupt("private release exceeds its bound"));
        }
        let release: ObjectRetentionRelease = serde_json::from_slice(body)?;
        transaction.execute(
            "INSERT INTO storage_object_releases(schema_id,schema_key,schema_version,encoding,item,through,generation) VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![release.schema.id.as_str(), serde_json::to_string(&release.schema)?, release.schema.version.get(),
                release.schema.payload_encoding as u8, release.item.as_str(), position(release.through.sequence), position(generation)],
        )?;
    }
    drop(release_rows);
    drop(release_statement);
    transaction.execute("DELETE FROM storage_object_scalars", [])?;
    transaction.execute("DELETE FROM storage_data_entries", [])?;
    transaction.execute("DELETE FROM storage_data_inventory", [])?;
    let mut statement = transaction
        .prepare("SELECT kind,body,digest FROM storage_bootstrap_entries ORDER BY position")?;
    let mut rows = statement.query([])?;
    let mut schema = None;
    while let Some(row) = rows.next()? {
        query::checkpoint()?;
        let kind = row
            .get_ref(0)?
            .as_str()
            .map_err(|_| Error::Corrupt("private entry kind is not text"))?;
        let body = blob(row, 1)?;
        if body.len() > maximum.get() {
            return Err(Error::Corrupt("private import entry exceeds backend bound"));
        }
        check_digest(body, blob(row, 2)?)?;
        let _decoded =
            memory.reserve_owned(Entry::decoding_bytes(kind, body.len())?, usize::MAX)?;
        let value = Entry::decode(kind, body)?;
        let typed = value.part().entries().next().expect("one typed entry");
        let length = entry::Encoded::encoded_bytes(typed, maximum.get())?;
        let encoded = entry::Encoded::new(typed, length)?;
        if encoded.body != body {
            return Err(Error::Corrupt(
                "private import visibility bytes are not canonical",
            ));
        }
        super::store_entry(&transaction, &encoded, maximum, &mut schema)?;
    }
    drop(rows);
    drop(statement);
    transaction.execute(
        "UPDATE storage_data_head SET saved=?1,lost_objects_through=?2,retention_generation=?3 WHERE singleton=1",
        params![position(import.identity.cut().sequence), position(import.identity.complete_after()), position(generation)],
    )?;
    transaction.execute(
        "UPDATE storage_data_origin SET complete_after=?1 WHERE singleton=1",
        [position(import.identity.complete_after())],
    )?;
    super::inventory::verify(&transaction, &saved(&transaction)?)?;
    transaction.execute(
        "INSERT INTO storage_bootstrap_completed VALUES (1,?1,?2)
         ON CONFLICT(singleton) DO UPDATE SET receipt=excluded.receipt,digest=excluded.digest",
        params![encoded_receipt, receipt_digest.as_slice()],
    )?;
    query::checkpoint()?;
    transaction.commit()?;
    Ok(())
}

fn cleanup_completed(
    connection: &mut Connection,
    root: &Path,
    identity: &BootstrapId,
) -> Result<(), Error> {
    let receipt = completed(connection)?.ok_or(Error::Corrupt(
        "private cleanup lacks its completion receipt",
    ))?;
    if receipt.import.identity != *identity {
        return Err(Error::Conflict("private cleanup completion owner differs"));
    }
    let Some(import) = active(connection)? else {
        if has_rows(connection)? {
            return Err(Error::Corrupt(
                "completed private rows lost their cleanup owner",
            ));
        }
        return Ok(());
    };
    if import.identity != *identity {
        // An Older Completion Receipt Cannot Remove A New Import's Work
        return Ok(());
    }
    if import != receipt.import || aborting(connection)? {
        return Err(Error::Corrupt("completed private cleanup identity differs"));
    }
    super::bootstrap_files::cleanup(connection, root, identity, false)?;
    let transaction = connection.transaction()?;
    matching_owner(&transaction, identity)?;
    transaction.execute("DELETE FROM storage_bootstrap_releases", [])?;
    transaction.execute("DELETE FROM storage_bootstrap_schemas", [])?;
    transaction.execute("DELETE FROM storage_bootstrap_entries", [])?;
    transaction.execute("DELETE FROM storage_bootstrap WHERE singleton=1", [])?;
    transaction.commit()?;
    Ok(())
}

fn completed(connection: &Connection) -> Result<Option<Completed>, Error> {
    let mut statement = connection
        .prepare("SELECT receipt,digest FROM storage_bootstrap_completed WHERE singleton=1")?;
    let mut rows = statement.query([])?;
    let Some(row) = rows.next()? else {
        return Ok(None);
    };
    let encoded = blob(row, 0)?;
    if encoded.len() > 2 * METADATA_BYTES {
        return Err(Error::Corrupt(
            "private completion receipt exceeds its bound",
        ));
    }
    check_digest(encoded, blob(row, 1)?)?;
    let receipt: Completed = serde_json::from_slice(encoded)?;
    receipt.import.limits.check()?;
    receipt.manifest.check_complete()?;
    if receipt.import.identity != *receipt.manifest.identity()
        || !receipt
            .import
            .identity
            .cut()
            .covers(&receipt.import.saved)?
        || entry::encode(&receipt, 2 * METADATA_BYTES)? != encoded
    {
        return Err(Error::Corrupt(
            "private completion receipt identity differs",
        ));
    }
    let head = saved(connection)?;
    head.history.check(&receipt.import.identity.cut().history)?;
    if head.sequence < receipt.import.identity.cut().sequence
        || head.retention_generation < receipt.import.identity.retention_generation().get()
    {
        return Err(Error::Corrupt(
            "private completion receipt exceeds its committed head",
        ));
    }
    Ok(Some(receipt))
}

fn begin(connection: &mut Connection, import: &Import) -> Result<(), Error> {
    import.limits.check()?;
    if aborting(connection)? {
        return Err(Error::Conflict("private import abort is pending"));
    }
    if let Some(receipt) = completed(connection)?
        && receipt.import.identity == import.identity
    {
        if receipt.import != *import {
            return Err(Error::Conflict(
                "completed private import begin retry differs",
            ));
        }
        if active(connection)?.is_some_and(|owner| owner.identity != import.identity) {
            return Err(Error::Conflict(
                "another private replica import owns the store",
            ));
        }
        return Ok(());
    }
    if !import.identity.cut().covers(&import.saved)? {
        return Err(Error::Conflict(
            "private import cut precedes its prior saved cursor",
        ));
    }
    let encoded = entry::encode(import, METADATA_BYTES)?;
    let transaction = connection.transaction()?;
    let head = saved(&transaction)?;
    head.history.check(&import.saved.history)?;
    if head.sequence != import.saved.sequence {
        return Err(Error::Conflict("private import prior saved cursor changed"));
    }
    if let Some(existing) = active(&transaction)? {
        if existing != *import {
            return Err(Error::Conflict(
                "another private replica import owns the store",
            ));
        }
    } else {
        if has_rows(&transaction)? {
            return Err(Error::Corrupt("private import rows lost their owner"));
        }
        transaction.execute("INSERT INTO storage_bootstrap VALUES (1,?1)", [encoded])?;
    }
    transaction.commit()?;
    Ok(())
}

fn abort(connection: &mut Connection, root: &Path, identity: &BootstrapId) -> Result<(), Error> {
    if completed(connection)?.is_some_and(|receipt| receipt.import.identity == *identity) {
        return cleanup_completed(connection, root, identity);
    }
    let transaction = connection.transaction()?;
    let Some(import) = active(&transaction)? else {
        if has_rows(&transaction)? {
            return Err(Error::Corrupt("private import rows lost their owner"));
        }
        transaction.commit()?;
        return Ok(());
    };
    if import.identity != *identity {
        return Err(Error::Conflict(
            "abort does not own the private replica import",
        ));
    }
    matching_owner(&transaction, identity)?;
    let encoded = entry::encode(identity, METADATA_BYTES)?;
    if aborting(&transaction)? {
        let mut statement = transaction
            .prepare("SELECT identity FROM storage_bootstrap_abort WHERE singleton=1")?;
        let mut rows = statement.query([])?;
        let row = rows
            .next()?
            .ok_or(Error::Corrupt("private abort intent disappeared"))?;
        if blob(row, 0)? != encoded {
            return Err(Error::Corrupt("private abort intent owner differs"));
        }
    } else {
        transaction.execute(
            "INSERT INTO storage_bootstrap_abort VALUES (1,?1)",
            [encoded],
        )?;
    }
    transaction.commit()?;
    // The Durable Intent Fences Further Import Work Across Failed Physical Cleanup
    super::bootstrap_files::cleanup(connection, root, identity, true)?;
    let transaction = connection.transaction()?;
    matching_owner(&transaction, identity)?;
    transaction.execute("DELETE FROM storage_bootstrap_files", [])?;
    transaction.execute("DELETE FROM storage_bootstrap_releases", [])?;
    transaction.execute("DELETE FROM storage_bootstrap_schemas", [])?;
    transaction.execute("DELETE FROM storage_bootstrap_entries", [])?;
    transaction.execute("DELETE FROM storage_bootstrap WHERE singleton=1", [])?;
    transaction.execute("DELETE FROM storage_bootstrap_abort WHERE singleton=1", [])?;
    transaction.commit()?;
    Ok(())
}

fn aborting(connection: &Connection) -> Result<bool, Error> {
    Ok(connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM storage_bootstrap_abort)",
        [],
        |row| row.get(0),
    )?)
}

fn write(
    connection: &mut Connection,
    identity: &BootstrapId,
    entries: &[entry::Retained],
    limits: QueryLimits,
    maximum: NonZeroUsize,
) -> Result<(), Error> {
    let transaction = connection.transaction()?;
    if matching(&transaction, identity)?.limits != limits {
        return Err(Error::Corrupt(
            "private import policy changed after admission",
        ));
    }
    for entry in entries {
        query::checkpoint()?;
        if let Some(key) = &entry.schema_key {
            transaction.execute(
                "INSERT INTO storage_bootstrap_schemas VALUES (?1,?2) ON CONFLICT(schema_key,position) DO NOTHING",
                params![key, position(entry.position)],
            )?;
            let indexed: String = transaction.query_row(
                "SELECT position FROM storage_bootstrap_schemas WHERE schema_key=?1 AND position=?2",
                params![key, position(entry.position)],
                |row| row.get(0),
            )?;
            if indexed != position(entry.position) {
                return Err(Error::Conflict(
                    "private import schema key has another position",
                ));
            }
        }
        let mut statement = transaction
            .prepare("SELECT kind,body,digest FROM storage_bootstrap_entries WHERE position=?1")?;
        let mut rows = statement.query([position(entry.position)])?;
        if let Some(row) = rows.next()? {
            let kind = row.get_ref(0)?.as_str().map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })?;
            let body = blob(row, 1)?;
            if body.len() > maximum.get() {
                return Err(Error::Corrupt("private import entry exceeds backend bound"));
            }
            check_digest(body, blob(row, 2)?)?;
            if kind != entry.kind || body != entry.body || blob(row, 2)? != entry.digest {
                return Err(Error::Conflict("private import position retry differs"));
            }
            continue;
        }
        transaction.execute(
            "INSERT INTO storage_bootstrap_entries VALUES (?1,?2,?3,?4)",
            params![
                position(entry.position),
                entry.kind,
                entry.body,
                entry.digest.as_slice()
            ],
        )?;
    }
    query::checkpoint()?;
    transaction.commit()?;
    Ok(())
}

fn verify_manifest(
    connection: &Connection,
    root: &Path,
    expected: &BootstrapManifest,
    memory: &Arc<memory::Budget>,
    maximum: NonZeroUsize,
) -> Result<(), Error> {
    expected.check_complete()?;
    let counts: (i64, i64) = connection.query_row(
        "SELECT (SELECT COUNT(*) FROM storage_bootstrap_schemas),(SELECT COUNT(*) FROM storage_bootstrap_entries WHERE kind='schema')",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    if counts.0 != counts.1 {
        return Err(Error::Corrupt(
            "private import schema index has extra entries",
        ));
    }
    let mut actual = BootstrapManifest::new(expected.identity().clone());
    for (kind, label) in [
        (ReadKind::Schemas, "schema"),
        (ReadKind::Objects, "object"),
        (ReadKind::Records, "record"),
        (ReadKind::Stream, "stream"),
        (ReadKind::Files, "file"),
    ] {
        let mut statement = connection.prepare(
            "SELECT position,body,digest FROM storage_bootstrap_entries WHERE kind=?1 ORDER BY position",
        )?;
        let mut rows = statement.query([label])?;
        while let Some(row) = rows.next()? {
            query::checkpoint()?;
            let index = row.get_ref(0)?.as_str().map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })?;
            let body = blob(row, 1)?;
            if body.len() > maximum.get() {
                return Err(Error::Corrupt("private import entry exceeds backend bound"));
            }
            check_digest(body, blob(row, 2)?)?;
            let _decoded =
                memory.reserve_owned(Entry::decoding_bytes(label, body.len())?, usize::MAX)?;
            let value = Entry::decode(label, body)?;
            let typed = value.part().entries().next().expect("one typed entry");
            if index != position(value.position().sequence) || sequence(index)? == 0 {
                return Err(Error::Corrupt("private import position index differs"));
            }
            let batch = RetainedBatch::new(
                expected.identity(),
                value.part(),
                ReadLimits {
                    rows: NonZeroUsize::MIN,
                    bytes: maximum,
                },
            )?;
            let canonical_bytes = body
                .len()
                .checked_mul(2)
                .ok_or(Error::Data("private canonical entry size overflow"))?;
            let _canonical = memory.reserve_owned(canonical_bytes, usize::MAX)?;
            let encoded = entry::Retained::new(
                batch.part().entries().next().expect("one typed entry"),
                maximum.get(),
            )?;
            if encoded.body != body {
                return Err(Error::Corrupt("private import entry is not canonical"));
            }
            validate_dependencies(connection, &value, expected.identity(), memory, maximum)?;
            actual.record(batch)?;
        }
        actual.finish_collection(kind);
    }
    let mut releases = connection.prepare("SELECT body FROM storage_bootstrap_releases ORDER BY schema_id,schema_version,encoding,item")?;
    let mut rows = releases.query([])?;
    while let Some(row) = rows.next()? {
        query::checkpoint()?;
        let body = blob(row, 0)?;
        if body.len() > METADATA_BYTES {
            return Err(Error::Corrupt("private release exceeds its bound"));
        }
        let release: ObjectRetentionRelease = serde_json::from_slice(body)?;
        if entry::encode(&release, METADATA_BYTES)? != body {
            return Err(Error::Corrupt("private release is not canonical"));
        }
        actual.record_releases(std::slice::from_ref(&release))?;
    }
    actual.finish_releases();
    if actual != *expected {
        return Err(Error::Conflict(
            "private import collection manifest differs",
        ));
    }
    let files: (i64, i64) = connection.query_row(
        "SELECT (SELECT COUNT(*) FROM storage_bootstrap_files),(SELECT COUNT(*) FROM storage_bootstrap_entries WHERE kind='file')",
        [], |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    if files.0 > files.1 {
        return Err(Error::Conflict("private import has undeclared file stages"));
    }
    let mut statement = connection.prepare(
        "SELECT body FROM storage_bootstrap_entries WHERE kind='file' ORDER BY position",
    )?;
    let mut rows = statement.query([])?;
    while let Some(row) = rows.next()? {
        query::checkpoint()?;
        let body = blob(row, 0)?;
        let _decoded =
            memory.reserve_owned(Entry::decoding_bytes("file", body.len())?, usize::MAX)?;
        let Entry::File(reference) = Entry::decode("file", body)? else {
            unreachable!("file collection")
        };
        super::bootstrap_files::confirm(connection, root, expected.identity(), &reference)?;
    }
    // Verify The Original Certified Completeness Suffix
    let identity = expected.identity();
    let mut complete = identity.complete_after();
    let mut suffix = connection.prepare(
        "SELECT position FROM storage_bootstrap_entries WHERE position>?1 ORDER BY position",
    )?;
    let mut positions = suffix.query([position(complete)])?;
    while let Some(row) = positions.next()? {
        query::checkpoint()?;
        let next = sequence(
            row.get_ref(0)?
                .as_str()
                .map_err(|_| Error::Corrupt("private completeness position is not text"))?,
        )?;
        if complete.checked_add(1) != Some(next) {
            return Err(Error::Corrupt("private completeness suffix has a gap"));
        }
        complete = next;
    }
    if complete != identity.cut().sequence {
        return Err(Error::Corrupt(
            "private completeness suffix ends before cut",
        ));
    }
    drop(positions);
    drop(suffix);
    query::checkpoint()?;
    Ok(())
}

fn validate_schema(
    schema: &StorageSchema,
    identity: &BootstrapId,
    memory: &Arc<memory::Budget>,
) -> Result<(), Error> {
    identity.cut().history.check(&schema.position.history)?;
    if schema.position.sequence == 0 || schema.position.sequence > identity.cut().sequence {
        return Err(Error::Corrupt("private import schema lies outside its cut"));
    }
    let failure = |error| {
        Error::Runtime(harmos::Error::backend(
            "validate private import schema",
            harmos::BackendFailure::Corrupt,
            error,
        ))
    };
    let settings = harmos::config::SchemaSettings::default();
    let bytes = schema
        .schema
        .validation_memory_bytes(&settings)
        .map_err(failure)?;
    let _validation = memory.reserve_owned(bytes, usize::MAX)?;
    schema.schema.prepare(&settings).map_err(failure)?;
    Ok(())
}

fn validate_dependencies(
    connection: &Connection,
    entry: &Entry,
    identity: &BootstrapId,
    memory: &Arc<memory::Budget>,
    maximum: NonZeroUsize,
) -> Result<(), Error> {
    let key = match entry {
        Entry::Schema(schema) => {
            validate_schema(schema, identity, memory)?;
            let _key = memory.reserve_owned(
                schema
                    .key
                    .memory_size()
                    .checked_mul(3)
                    .ok_or(Error::Data("private schema lookup size overflow"))?,
                usize::MAX,
            )?;
            let indexed: Option<String> = connection
                .query_row(
                    "SELECT position FROM storage_bootstrap_schemas WHERE schema_key=?1 AND position=?2",
                    params![serde_json::to_string(&schema.key)?, position(schema.position.sequence)],
                    |row| row.get(0),
                )
                .optional()?;
            if indexed.as_deref() != Some(position(schema.position.sequence).as_str()) {
                return Err(Error::Corrupt("private import schema index differs"));
            }
            return Ok(());
        }
        Entry::Object(value) => &value.value.schema,
        Entry::Record(value) => &value.schema,
        Entry::Stream(value) => &value.record.schema,
        Entry::File(_) => return Ok(()),
    };
    let _key = memory.reserve_owned(
        key.memory_size()
            .checked_mul(3)
            .ok_or(Error::Data("private schema lookup size overflow"))?,
        usize::MAX,
    )?;
    let mut statement = connection.prepare(
        "SELECT e.position,e.body,e.digest FROM storage_bootstrap_schemas s JOIN storage_bootstrap_entries e ON e.position=s.position WHERE s.schema_key=?1 AND e.kind='schema' AND e.position<?2 ORDER BY e.position DESC LIMIT 1",
    )?;
    let mut rows = statement.query(params![
        serde_json::to_string(key)?,
        position(entry.position().sequence)
    ])?;
    let row = rows.next()?.ok_or(Error::Corrupt(
        "private import entry has no preceding saved schema",
    ))?;
    let body = blob(row, 1)?;
    if body.len() > maximum.get() {
        return Err(Error::Corrupt(
            "private import schema exceeds backend bound",
        ));
    }
    check_digest(body, blob(row, 2)?)?;
    let _schema = memory.reserve_owned(Entry::decoding_bytes("schema", body.len())?, usize::MAX)?;
    let schema: StorageSchema = serde_json::from_slice(body)?;
    let indexed: String = row.get(0)?;
    if schema.key != *key
        || indexed != position(schema.position.sequence)
        || schema.position.sequence >= entry.position().sequence
    {
        return Err(Error::Corrupt("private import schema dependency differs"));
    }
    validate_schema(&schema, identity, memory)?;
    query::checkpoint()?;
    Ok(())
}

pub(super) fn matching(connection: &Connection, identity: &BootstrapId) -> Result<Import, Error> {
    if aborting(connection)? {
        return Err(Error::Conflict("private import abort is pending"));
    }
    if completed(connection)?.is_some_and(|receipt| receipt.import.identity == *identity) {
        return Err(Error::Conflict(
            "completed private import no longer accepts mutations",
        ));
    }
    matching_owner(connection, identity)
}

pub(super) fn matching_owner(
    connection: &Connection,
    identity: &BootstrapId,
) -> Result<Import, Error> {
    let import =
        active(connection)?.ok_or(Error::Conflict("private import has no active owner"))?;
    if import.identity != *identity {
        return Err(Error::Conflict(
            "batch does not own the private replica import",
        ));
    }
    let head = saved(connection)?;
    head.history.check(&import.saved.history)?;
    let published = completed(connection)?.is_some_and(|receipt| receipt.import == import);
    if head.sequence != import.saved.sequence && !published {
        return Err(Error::Corrupt(
            "private import fence lost its prior saved cursor",
        ));
    }
    Ok(import)
}

fn has_rows(connection: &Connection) -> Result<bool, Error> {
    Ok(connection.query_row(
        "SELECT EXISTS(SELECT 1 FROM storage_bootstrap_releases) OR EXISTS(SELECT 1 FROM storage_bootstrap_entries) OR EXISTS(SELECT 1 FROM storage_bootstrap_schemas) OR EXISTS(SELECT 1 FROM storage_bootstrap_files) OR EXISTS(SELECT 1 FROM storage_bootstrap_installs) OR EXISTS(SELECT 1 FROM storage_bootstrap_abort)",
        [],
        |row| row.get(0),
    )?)
}

pub(super) fn active(connection: &Connection) -> Result<Option<Import>, Error> {
    let mut statement =
        connection.prepare("SELECT metadata FROM storage_bootstrap WHERE singleton=1")?;
    let mut rows = statement.query([])?;
    let Some(row) = rows.next()? else {
        return Ok(None);
    };
    let encoded = blob(row, 0)?;
    if encoded.len() > METADATA_BYTES {
        return Err(Error::Corrupt("private import metadata exceeds its bound"));
    }
    let import: Import = serde_json::from_slice(encoded)?;
    import.limits.check()?;
    if !import.identity.cut().covers(&import.saved)? {
        return Err(Error::Corrupt(
            "private import cut precedes its prior saved cursor",
        ));
    }
    Ok(Some(import))
}

/* === Tests === */
