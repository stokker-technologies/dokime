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
use std::{num::NonZeroUsize, path::Path};

// Third Party Imports
use harmos::storage::*;
use rusqlite::{Connection, params};

// Local Imports
use super::super::{Error, query};
use super::{Entry, SavedData, blob, check_digest, files, position, sequence};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn verify(
    connection: &Connection,
    root: &Path,
    cut: &SavedData,
    maximum: NonZeroUsize,
) -> Result<(), Error> {
    if cut.retention_generation.checked_add(1).is_none() {
        return Err(Error::Data("retained snapshot generation exhausted"));
    }
    if cut.lost_objects_through > cut.sequence {
        return Err(Error::Data("retained cut precedes completeness floor"));
    }
    let mut statement = connection.prepare("SELECT position,kind,body,digest,object_fields FROM storage_data_entries WHERE position<=?1 AND NOT EXISTS(SELECT 1 FROM storage_deleted_files WHERE kind='file' AND storage_deleted_files.id=storage_data_entries.item) AND (retired_at IS NULL OR retired_at>?2) ORDER BY position")?;
    let mut rows = statement.query(params![
        position(cut.sequence),
        position(cut.retention_generation)
    ])?;
    let mut complete = cut.lost_objects_through;
    while let Some(row) = rows.next()? {
        query::checkpoint()?;
        let index = row.get_ref(0)?.as_str().map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                0,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?;
        let kind = row.get_ref(1)?.as_str().map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                1,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?;
        let body = blob(row, 2)?;
        if body.len() > maximum.get() {
            return Err(Error::Corrupt("retained entry exceeds backend bound"));
        }
        check_digest(body, blob(row, 3)?)?;
        let entry = Entry::decode(kind, body)?;
        let cursor = entry.position();
        cursor.history.check(&cut.history)?;
        let index_sequence = sequence(index)?;
        if index_sequence == 0
            || index_sequence != cursor.sequence
            || index != position(cursor.sequence)
        {
            return Err(Error::Corrupt("retained position index differs"));
        }
        if cursor.sequence > cut.lost_objects_through {
            if complete.checked_add(1) != Some(cursor.sequence) {
                return Err(Error::Corrupt("retained completeness suffix has a gap"));
            }
            complete = cursor.sequence;
        }
        match &entry {
            Entry::Schema(schema) => validate(schema, cut)?,
            Entry::Object(object) => {
                let fields = blob(row, 4)?;
                if fields.len() > maximum.get() {
                    return Err(Error::Corrupt(
                        "retained object fields exceed backend bound",
                    ));
                }
                let fields: Vec<harmos::schema::graph::Field> = serde_json::from_slice(fields)?;
                let schema = dependency(connection, &object.value.schema, cursor, cut, maximum)?;
                if !fields.iter().eq(schema
                    .schema
                    .root()
                    .map_err(|_| Error::Corrupt("invalid retained schema root"))?
                    .fields()
                    .iter()
                    .filter(|field| super::index::keys::supported(field)))
                {
                    return Err(Error::Corrupt(
                        "retained object descriptors differ from saved schema",
                    ));
                }
            }
            Entry::Record(record) => {
                dependency(connection, &record.schema, cursor, cut, maximum)?;
            }
            Entry::Stream(stream) => {
                dependency(connection, &stream.record.schema, cursor, cut, maximum)?;
            }
            Entry::File(file) => files::verify_saved(connection, root, file)?,
        }
    }
    if complete != cut.sequence {
        return Err(Error::Corrupt(
            "retained completeness suffix ends before cut",
        ));
    }
    Ok(())
}

/* === Private Functions === */
fn validate(schema: &StorageSchema, cut: &SavedData) -> Result<(), Error> {
    schema.position.history.check(&cut.history)?;
    let failure = |error| {
        Error::Runtime(harmos::Error::backend(
            "validate retained schema",
            harmos::BackendFailure::Corrupt,
            error,
        ))
    };
    let settings = harmos::config::SchemaSettings::default();
    schema.schema.prepare(&settings).map_err(failure)?;
    Ok(())
}

fn dependency(
    connection: &Connection,
    key: &SchemaKey,
    entry: &StorageCursor,
    cut: &SavedData,
    maximum: NonZeroUsize,
) -> Result<StorageSchema, Error> {
    let mut statement = connection.prepare("SELECT body,digest FROM storage_data_entries WHERE kind='schema' AND schema_key=?1 AND position<?2 AND (retired_at IS NULL OR retired_at>?3) ORDER BY position DESC LIMIT 1")?;
    let mut rows = statement.query(params![
        serde_json::to_string(key)?,
        position(entry.sequence),
        position(cut.retention_generation)
    ])?;
    let row = rows.next()?.ok_or(Error::Corrupt(
        "retained entry has no preceding saved schema",
    ))?;
    let body = blob(row, 0)?;
    if body.len() > maximum.get() {
        return Err(Error::Corrupt("retained schema exceeds backend bound"));
    }
    check_digest(body, blob(row, 1)?)?;
    let schema: StorageSchema = serde_json::from_slice(body)?;
    if schema.key != *key || schema.position.sequence >= entry.sequence {
        return Err(Error::Corrupt("retained schema index differs"));
    }
    validate(&schema, cut)?;
    Ok(schema)
}

/* === Tests === */
