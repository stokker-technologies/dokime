/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(super) mod keys;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos::storage::StorageSchema;
use rusqlite::{Connection, params};

// Local Imports
use super::super::Error;
use super::{
    Entry, blob, check_digest,
    entry::{self, Encoded},
    position,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(in crate::storage::sqlite) struct IndexSchema {
    key: String,
    schema: StorageSchema,
    fields: Vec<u8>,
    decoder: keys::Decoder,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
pub(super) fn write(
    connection: &Connection,
    entry: &Encoded,
    maximum: NonZeroUsize,
    cache: &mut Option<IndexSchema>,
) -> Result<(), Error> {
    if entry.kind != "object" {
        return Err(Error::Data("scalar index requires an object row"));
    }
    let schema_key = entry
        .schema_key
        .as_ref()
        .ok_or(Error::Data("indexed row has no schema key"))?;
    if cache
        .as_ref()
        .is_none_or(|cached| &cached.key != schema_key)
    {
        *cache = None;
        let mut statement = connection.prepare(
            "SELECT position,body,digest FROM storage_data_entries WHERE kind='schema' AND schema_key=?1 AND position<?2 ORDER BY position LIMIT 1",
        )?;
        let mut rows = statement.query(params![schema_key, position(entry.position)])?;
        let row = rows.next()?.ok_or(Error::Data(
            "indexed row requires its preceding admitted schema",
        ))?;
        let body = blob(row, 1)?;
        if body.len() > maximum.get() {
            return Err(Error::Corrupt("stored scalar schema exceeds backend bound"));
        }
        check_digest(body, blob(row, 2)?)?;
        let schema: StorageSchema = serde_json::from_slice(body)?;
        if row.get::<_, String>(0)? != position(schema.position.sequence) {
            return Err(Error::Corrupt(
                "scalar schema position differs from its index",
            ));
        }
        let failure = |error| {
            Error::Runtime(harmos::Error::backend(
                "validate persisted scalar schema",
                harmos::BackendFailure::Corrupt,
                error,
            ))
        };
        let settings = harmos::config::SchemaSettings::default();
        schema.schema.prepare(&settings).map_err(failure)?;
        // Scalar Browsing Needs Only The Supported Top-Level Descriptors.
        // The Complete Immutable Schema Remains In Its Original Schema Entry.
        let root = schema.schema.root().map_err(failure)?;
        let selected = root
            .fields()
            .iter()
            .filter(|field| keys::supported(field))
            .collect::<Vec<_>>();
        let fields = entry::encode(&selected, maximum.get())?;
        drop(selected);
        let decoder = keys::Decoder::new(&schema.schema, schema.key.payload_encoding)?;
        *cache = Some(IndexSchema {
            key: schema_key.clone(),
            schema,
            fields,
            decoder,
        });
    }
    let cached = cache.as_ref().expect("original descriptor is retained");
    let schema = &cached.schema;
    let stored = Entry::decode(entry.kind, &entry.body)?;
    let Entry::Object(stored) = &stored else {
        return Err(Error::Corrupt("scalar entry collection differs"));
    };
    let (key, cursor, payload) = (
        &stored.value.schema,
        &stored.position,
        stored.value.payload.as_slice(),
    );
    if schema.key != *key
        || schema.position.sequence >= cursor.sequence
        || cursor.sequence != entry.position
    {
        return Err(Error::Corrupt(
            "scalar schema identity differs from its index",
        ));
    }
    schema.position.history.check(&cursor.history)?;
    let root = schema
        .schema
        .root()
        .map_err(|_| Error::Corrupt("invalid scalar schema root"))?;
    let cells = keys::encode(root.fields(), payload, &cached.decoder)?;
    connection.execute(
        "UPDATE storage_data_entries SET object_fields=?1 WHERE position=?2 AND kind='object'",
        params![cached.fields, position(entry.position)],
    )?;
    let insert = "INSERT INTO storage_object_scalars(position,tag,kind,value) VALUES (?1,?2,?3,?4)";
    for cell in cells {
        connection.execute(
            insert,
            params![position(entry.position), cell.tag, cell.kind, cell.key],
        )?;
    }
    Ok(())
}

/* === Tests === */
