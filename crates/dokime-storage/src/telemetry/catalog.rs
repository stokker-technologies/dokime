/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Arrow File Catalog

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::{Path, PathBuf};

// Third Party Imports
use rusqlite::{Connection, params};
use sha2::{Digest, Sha256};

// Local Imports
use crate::{
    Error, Result,
    ipc::{BatchIndex, Checkpoint},
};

/* === Constants === */

pub(crate) const SCHEMA: &str = "
CREATE TABLE telemetry_files(id TEXT PRIMARY KEY, execution TEXT NOT NULL REFERENCES executions(id), stream TEXT NOT NULL, schema_hash BLOB NOT NULL, end INTEGER NOT NULL, hash_state BLOB NOT NULL, batches INTEGER NOT NULL, finalized INTEGER NOT NULL, length INTEGER NOT NULL, sha256 BLOB NOT NULL, UNIQUE(execution,stream,schema_hash));
CREATE TABLE telemetry_batches(file TEXT NOT NULL REFERENCES telemetry_files(id), offset INTEGER NOT NULL, metadata_bytes INTEGER NOT NULL, body_bytes INTEGER NOT NULL, rows INTEGER NOT NULL, sha256 BLOB NOT NULL, start_ns INTEGER NOT NULL, end_ns INTEGER NOT NULL, first_position INTEGER NOT NULL, last_position INTEGER NOT NULL, PRIMARY KEY(file,offset));
CREATE INDEX telemetry_ranges ON telemetry_batches(file,first_position,last_position,start_ns,end_ns);
CREATE TABLE telemetry_garbage(id TEXT PRIMARY KEY);
";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) fn identifier(execution: &str, stream: &str, schema: &[u8]) -> String {
    let mut hash = Sha256::new();
    for value in [execution.as_bytes(), stream.as_bytes(), schema] {
        hash.update((value.len() as u64).to_le_bytes());
        hash.update(value);
    }
    hash.finalize().iter().map(|byte| format!("{byte:02x}")).collect()
}
pub(crate) fn path(root: &Path, id: &str) -> Result<PathBuf> {
    if id.len() != 64 || !id.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(Error::Corrupt("invalid telemetry file identity"));
    }
    Ok(root.join("telemetry").join(format!("{id}.arrow")))
}
pub(crate) fn indices(connection: &Connection, id: &str) -> Result<Vec<BatchIndex>> {
    let mut query =
        connection.prepare("SELECT offset,metadata_bytes,body_bytes,rows,sha256 FROM telemetry_batches WHERE file=?1 ORDER BY offset")?;
    query.query_map([id], index)?.map(|row| Ok(row?)).collect()
}
pub(crate) fn checkpoint(connection: &Connection, id: &str) -> Result<Checkpoint> {
    Ok(connection.query_row("SELECT end,hash_state,batches FROM telemetry_files WHERE id=?1", [id], |row| {
        Ok(Checkpoint {
            end: row.get::<_, i64>(0)? as u64,
            hash: row.get(1)?,
            batches: row.get::<_, i64>(2)? as usize,
        })
    })?)
}
pub(crate) fn index(row: &rusqlite::Row<'_>) -> rusqlite::Result<BatchIndex> {
    let digest: Vec<u8> = row.get(4)?;
    Ok(BatchIndex {
        offset: row.get::<_, i64>(0)? as u64,
        metadata_bytes: row.get::<_, i64>(1)? as u32,
        body_bytes: row.get::<_, i64>(2)? as u64,
        rows: row.get::<_, i64>(3)? as u64,
        sha256: digest.try_into().map_err(|_| rusqlite::Error::InvalidQuery)?,
    })
}
pub(crate) fn collect_garbage(root: &Path, connection: &Connection) -> Result<()> {
    let mut query = connection.prepare("SELECT id FROM telemetry_garbage ORDER BY id")?;
    let files = query.query_map([], |row| row.get::<_, String>(0))?.collect::<std::result::Result<Vec<_>, _>>()?;
    for id in files {
        match std::fs::remove_file(path(root, &id)?) {
            Ok(()) => (),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(error) => return Err(error.into()),
        }
        connection.execute("DELETE FROM telemetry_garbage WHERE id=?1", params![id])?;
    }
    Ok(())
}

pub(crate) fn verify_files(root: &Path, connection: &Connection, execution: &str) -> Result<()> {
    use std::io::Read;
    let mut query = connection.prepare("SELECT id,length,sha256 FROM telemetry_files WHERE execution=?1 ORDER BY id")?;
    for row in
        query.query_map([execution], |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u64, row.get::<_, Vec<u8>>(2)?)))?
    {
        let (id, length, expected) = row?;
        let file = std::fs::File::open(path(root, &id)?)?;
        if file.metadata()?.len() != length {
            return Err(Error::Corrupt("stream file is shorter than its catalog"));
        }
        let mut file = file.take(length);
        let mut hash = Sha256::new();
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hash.update(&buffer[..count]);
        }
        if hash.finalize().as_slice() != expected {
            return Err(Error::Corrupt("stream file digest differs"));
        }
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
