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
use std::{num::NonZeroUsize, sync::Arc};

// Third Party Imports
use harmos::{MemorySize, storage::StorageSchema};
use rusqlite::{Connection, params};

// Local Imports
use super::super::{Error, memory};
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
    _decoded: memory::Reservation,
    _encoded: memory::Reservation,
    _decoder: memory::Reservation,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
pub(super) fn write(
    connection: &Connection,
    entry: &Encoded,
    memory: &Arc<memory::Budget>,
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
        let bytes = Entry::decoding_bytes("schema", body.len())?
            .checked_add(schema_key.len() + std::mem::size_of::<IndexSchema>())
            .ok_or(Error::Data("scalar schema memory overflow"))?;
        let mut decoded = memory.reserve_owned(bytes, usize::MAX)?;
        let schema: StorageSchema = serde_json::from_slice(body)?;
        let retained = schema
            .memory_size()
            .checked_add(schema_key.len())
            .and_then(|bytes| {
                bytes.checked_add(
                    std::mem::size_of::<IndexSchema>() - std::mem::size_of::<StorageSchema>(),
                )
            })
            .ok_or(Error::Data("retained scalar schema memory overflow"))?;
        decoded.shrink(retained)?;
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
        let validation = schema
            .schema
            .validation_memory_bytes(&settings)
            .map_err(failure)?;
        let validation = memory.reserve_owned(validation, usize::MAX)?;
        schema.schema.prepare(&settings).map_err(failure)?;
        drop(validation);
        // Scalar Browsing Needs Only The Supported Top-Level Descriptors.
        // The Complete Immutable Schema Remains In Its Original Schema Entry.
        let root = schema.schema.root().map_err(failure)?;
        let slots = root
            .fields()
            .len()
            .checked_mul(std::mem::size_of::<&harmos::schema::graph::Field>())
            .ok_or(Error::Data("scalar descriptor slots overflow"))?;
        let _slots = memory.reserve_owned(slots, usize::MAX)?;
        let selected = root
            .fields()
            .iter()
            .filter(|field| keys::supported(field))
            .collect::<Vec<_>>();
        let bytes = entry::check_size(&selected, maximum.get())?;
        let encoded = memory.reserve_owned(bytes, usize::MAX)?;
        let fields = entry::encode(&selected, maximum.get())?;
        drop(selected);
        let bytes = keys::Decoder::construction_bytes(&schema.schema, schema.key.payload_encoding)?;
        let decoder_memory = memory.reserve_owned(bytes, usize::MAX)?;
        let decoder = keys::Decoder::new(&schema.schema, schema.key.payload_encoding)?;
        *cache = Some(IndexSchema {
            key: schema_key.clone(),
            schema,
            fields,
            decoder,
            _decoded: decoded,
            _encoded: encoded,
            _decoder: decoder_memory,
        });
    }
    let cached = cache.as_ref().expect("original descriptor is retained");
    let schema = &cached.schema;
    let decoded = Entry::decoding_bytes(entry.kind, entry.body.len())?;
    let _decoded = memory.reserve_owned(decoded, usize::MAX)?;
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
    let scalar_bytes = keys::allocation_bytes(root.fields(), payload.len(), &cached.decoder)?;
    let _scalars = memory.reserve_owned(scalar_bytes, usize::MAX)?;
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
