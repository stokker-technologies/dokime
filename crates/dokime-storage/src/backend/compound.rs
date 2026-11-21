/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Commit Atomic Storage Mutations

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeSet, result, slice};

// Third Party Imports
use harmos_query::{
    columnar::{ColumnKind, ColumnSource},
    source,
};
use harmos_storage::{
    CompoundOutcome, CompoundReceipt, CompoundWrite, Identifier, MutationHeader, MutationReceipt, MutationValue, ObjectEntry, ObjectKey,
    ObjectVersionToken, Result, StorageCursor,
};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

// Local Imports
use super::{
    State, columns, conflict, db, decode, encode, files, integer, invalid, meta, rows, save_mutation_receipt, set_meta, text, unsigned,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn commit(
    state: &mut State, position: &StorageCursor, items: &[CompoundWrite], header: &MutationHeader,
) -> Result<MutationReceipt> {
    state.check()?;
    state.history.check(&position.history)?;
    let tx = state.connection.transaction_with_behavior(TransactionBehavior::Immediate).map_err(db)?;
    let prior: Option<Vec<u8>> = tx
        .query_row("SELECT data FROM storage_mutations WHERE id=?1", [text(&header.mutation)?], |row| row.get(0))
        .optional()
        .map_err(db)?;
    if let Some(prior) = prior {
        return decode(&prior);
    }
    let first = match &items[0] {
        CompoundWrite::Object { schema, .. } => schema.position.sequence,
        item => item.position().sequence,
    };
    if first <= meta(&tx, "saved")? {
        return Err(conflict("Commit Mutation: Refresh Its Source Position"));
    }

    // Roll Back Every Effect on a Late Conflict, Keeping the Refusal Receipt.
    tx.execute_batch("SAVEPOINT compound_effects").map_err(db)?;
    let mut outcome = CompoundOutcome::Applied {
        positions: items.iter().map(|item| item.position().clone()).collect(),
    };
    for (index, item) in items.iter().enumerate() {
        match item {
            CompoundWrite::Object { expected, schema, value } => {
                state.catalog.validate_value(&value.value, state.settings.chunk_bytes)?;
                let current = current(&tx, &value.value.schema.id, &value.value.item)?;
                if current.as_ref() != expected.as_ref() {
                    outcome = CompoundOutcome::Conflict { index, current };
                    break;
                }
                rows::put_schema(&tx, &state.history, schema)?;
                rows::insert(
                    &tx,
                    &state.history,
                    rows::WriteRow {
                        position: &value.position,
                        kind: 1,
                        schema: Some(&value.value.schema),
                        item: Some(value.value.item.as_str()),
                        scope: value.value.scope.as_ref().map(Identifier::as_str),
                        stream: None,
                        time: None,
                        data: encode(value)?,
                    },
                )?;
                let batch = source::objects(schema, slice::from_ref(value))?;
                let source = ColumnSource {
                    kind: ColumnKind::Objects,
                    schema: schema.key.clone(),
                };
                columns::store(&tx, &source, &batch, state.settings)?;
            }
            CompoundWrite::Delete { position, schema, item, expected } => {
                let current = current(&tx, schema, item)?;
                if expected.is_some() && current.as_ref() != expected.as_ref() {
                    outcome = CompoundOutcome::Conflict { index, current };
                    break;
                }
                tx.execute(
                    "INSERT INTO storage_object_deletions VALUES(?1,?2,?3)",
                    params![integer(position.sequence)?, schema.as_str(), item.as_str()],
                )
                .map_err(db)?;
            }
            CompoundWrite::Record(record) => {
                rows::insert(
                    &tx,
                    &state.history,
                    rows::WriteRow {
                        position: &record.position,
                        kind: 2,
                        schema: Some(&record.schema),
                        item: Some(record.item.as_str()),
                        scope: record.scope.as_ref().map(Identifier::as_str),
                        stream: None,
                        time: record.source_time,
                        data: encode(record)?,
                    },
                )?;
                let schema = rows::schema(&tx, &record.schema, record.position.sequence)?;
                let batch = source::records(&schema, std::iter::once(record))?;
                let source = ColumnSource {
                    kind: ColumnKind::Records,
                    schema: record.schema.clone(),
                };
                columns::store(&tx, &source, &batch, state.settings)?;
            }
            CompoundWrite::Scope { position, scope } => delete_scope(&tx, position, scope)?,
        }
    }
    if matches!(outcome, CompoundOutcome::Conflict { .. }) {
        tx.execute_batch("ROLLBACK TO compound_effects").map_err(db)?;
    }
    tx.execute_batch("RELEASE compound_effects").map_err(db)?;
    let refused = matches!(outcome, CompoundOutcome::Conflict { .. });
    let receipt = header.complete(MutationValue::Compound(CompoundReceipt { position: position.clone(), outcome }));
    save_mutation_receipt(&tx, &receipt)?;
    rows::save(&tx, position.sequence)?;
    tx.execute("INSERT INTO storage_compound_ranges VALUES(?1,?2)", params![integer(first)?, integer(position.sequence)?]).map_err(db)?;
    tx.commit().map_err(db)?;
    if items.iter().any(|item| matches!(item, CompoundWrite::Scope { .. })) {
        reclaim(state)?;
    } else if refused {
        columns::collect_orphans(&state.connection)?;
    }
    Ok(receipt)
}

pub(super) fn current(connection: &Connection, schema: &Identifier, item: &ObjectKey) -> Result<Option<ObjectVersionToken>> {
    let data: Option<Vec<u8>> = connection
        .query_row(
            "SELECT data FROM storage_rows r WHERE kind=1 AND schema_id=?1 AND item=?2
         AND NOT EXISTS(SELECT 1 FROM storage_object_deletions d WHERE d.schema_id=r.schema_id AND d.item=r.item AND d.position>r.position)
         AND NOT EXISTS(SELECT 1 FROM storage_row_deletions d WHERE d.position=r.position)
         ORDER BY position DESC LIMIT 1",
            params![schema.as_str(), item.as_str()],
            |row| row.get(0),
        )
        .optional()
        .map_err(db)?;
    data.as_deref().map(decode::<ObjectEntry>).transpose().map(|entry| entry.map(|entry| entry.version))
}

pub(super) fn visible_positions(connection: &Connection, source: &ColumnSource, first: u64, last: u64, at: u64) -> Result<BTreeSet<u64>> {
    let kind = match source.kind {
        ColumnKind::Objects => 1,
        ColumnKind::Records => 2,
        ColumnKind::Stream => 3,
    };
    let mut query = connection
        .prepare(
            "SELECT position FROM storage_rows r WHERE position>=?1 AND position<=?2 AND kind=?4 AND schema_key=?5
         AND NOT EXISTS(SELECT 1 FROM storage_row_deletions d WHERE d.position=r.position AND d.deleted_at<=?3)
         AND NOT EXISTS(SELECT 1 FROM storage_object_deletions d WHERE r.kind=1 AND d.schema_id=r.schema_id AND d.item=r.item
                        AND d.position>r.position AND d.position<=?3)",
        )
        .map_err(db)?;
    let mut visible = query
        .query_map(params![integer(first)?, integer(last)?, integer(at)?, kind, text(&source.schema)?], |row| unsigned(row, 0))
        .map_err(db)?
        .collect::<result::Result<BTreeSet<_>, _>>()
        .map_err(db)?;
    if source.kind == ColumnKind::Stream {
        let mut ranges = connection
            .prepare(
                "SELECT first,last FROM storage_sample_ranges r WHERE schema_key=?1 AND first<=?3 AND last>=?2
            AND NOT EXISTS(SELECT 1 FROM storage_scope_deletions d WHERE d.scope=r.scope AND d.position>r.last AND d.position<=?4)",
            )
            .map_err(db)?;
        let ranges = ranges
            .query_map(params![text(&source.schema)?, integer(first)?, integer(last)?, integer(at)?], |row| {
                Ok((unsigned(row, 0)?, unsigned(row, 1)?))
            })
            .map_err(db)?
            .collect::<result::Result<Vec<_>, _>>()
            .map_err(db)?;
        for (start, end) in ranges {
            visible.extend(start.max(first)..=end.min(last).min(at));
        }
    }
    Ok(visible)
}

pub(super) fn reclaim(state: &mut State) -> Result<()> {
    let floor = meta(&state.connection, "floor")?;
    if floor == 0 {
        return files::reclaim(state);
    }
    let oldest = state
        .connection
        .query_row("SELECT coalesce(min(position),9223372036854775807) FROM storage_retained_cuts WHERE scope IS NULL", [], |row| {
            unsigned(row, 0)
        })
        .map_err(db)?;
    let tx = state.connection.transaction().map_err(db)?;
    tx.execute_batch("CREATE TEMP TABLE reclaim_rows(position INTEGER PRIMARY KEY);
        INSERT INTO reclaim_rows SELECT r.position FROM storage_rows r JOIN storage_row_deletions d USING(position)
        WHERE NOT EXISTS(SELECT 1 FROM storage_retained_cuts c WHERE c.position>=r.position AND c.position<d.deleted_at AND (c.scope IS NULL OR c.scope=r.scope));
        DELETE FROM storage_index_entries WHERE position IN(SELECT position FROM reclaim_rows);
        DELETE FROM storage_rows WHERE position IN(SELECT position FROM reclaim_rows);
        DROP TABLE reclaim_rows;").map_err(db)?;
    tx.execute(
        "DELETE FROM storage_sample_ranges AS r WHERE EXISTS(SELECT 1 FROM storage_scope_deletions d
        WHERE d.scope=r.scope AND d.position>r.last AND NOT EXISTS(SELECT 1 FROM storage_retained_cuts c
        WHERE c.position>=r.first AND c.position<d.position AND (c.scope IS NULL OR c.scope=r.scope)))",
        [],
    )
    .map_err(db)?;
    let chunks = {
        let mut query = tx.prepare("SELECT id,source,first,last FROM arrow_chunks c WHERE EXISTS(SELECT 1 FROM storage_row_deletions d WHERE d.position>=c.first AND d.position<=c.last AND d.deleted_at<=?1) OR json_extract(source,'$.kind')='Stream'").map_err(db)?;
        query
            .query_map([integer(oldest)?], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, unsigned(row, 2)?, unsigned(row, 3)?))
            })
            .map_err(db)?
            .collect::<result::Result<Vec<_>, _>>()
            .map_err(db)?
    };
    for (id, source, first, last) in chunks {
        let source: ColumnSource = decode(source.as_bytes())?;
        let kind = match source.kind {
            ColumnKind::Objects => 1,
            ColumnKind::Records => 2,
            ColumnKind::Stream => 3,
        };
        let referenced: bool = tx
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM storage_rows WHERE kind=?1 AND schema_key=?2 AND position>=?3 AND position<=?4) OR (?1=3 AND EXISTS(SELECT 1 FROM storage_sample_ranges WHERE schema_key=?2 AND first<=?4 AND last>=?3))",
                params![kind, text(&source.schema)?, integer(first)?, integer(last)?],
                |row| row.get(0),
            )
            .map_err(db)?;
        if !referenced {
            tx.execute("DELETE FROM arrow_columns WHERE chunk=?1", [id]).map_err(db)?;
            tx.execute("DELETE FROM arrow_chunks WHERE id=?1", [id]).map_err(db)?;
        }
    }
    if floor > 0 && oldest >= floor {
        for table in ["projection_pages", "projection_changes", "projection_cuts", "projection_bucket_outputs", "projection_bucket_cuts"] {
            tx.execute(&format!("DELETE FROM {table} WHERE frontier<?1"), [integer(floor)?]).map_err(db)?;
        }
        let stale: bool = tx
            .query_row("SELECT EXISTS(SELECT 1 FROM projection_bucket_state WHERE frontier<?1)", [integer(floor)?], |row| row.get(0))
            .map_err(db)?;
        if stale {
            tx.execute("DELETE FROM projection_bucket_members WHERE (name,revision) IN(SELECT name,revision FROM projection_bucket_state WHERE frontier<?1)", [integer(floor)?]).map_err(db)?;
            tx.execute("DELETE FROM projection_bucket_state WHERE frontier<?1", [integer(floor)?]).map_err(db)?;
        }
    }
    tx.commit().map_err(db)?;
    files::reclaim(state)?;
    columns::collect_orphans(&state.connection)
}

/* === Private Functions === */

fn delete_scope(connection: &Connection, position: &StorageCursor, scope: &Identifier) -> Result<()> {
    let active: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM storage_files f LEFT JOIN storage_file_sessions s USING(id)
         WHERE f.scope=?1 AND f.finished IS NULL AND coalesce(s.abandoned,0)=0)",
            [scope.as_str()],
            |row| row.get(0),
        )
        .map_err(db)?;
    if active {
        return Err(invalid("Delete Scope: Finish or Abandon Its Active File Productions"));
    }

    // Delete All Revisions of Objects Whose Current Identity Belongs to This Scope.
    connection
        .execute(
            "INSERT OR IGNORE INTO storage_row_deletions
         SELECT position,?2 FROM storage_rows WHERE scope=?1 OR (kind=1 AND (schema_id,item) IN
           (SELECT schema_id,item FROM storage_rows r WHERE kind=1 AND scope=?1
            AND position=(SELECT max(o.position) FROM storage_rows o WHERE o.kind=1 AND o.schema_id=r.schema_id AND o.item=r.item)))",
            params![scope.as_str(), integer(position.sequence)?],
        )
        .map_err(db)?;
    let generation =
        meta(connection, "deletion_generation")?.checked_add(1).ok_or_else(|| invalid("Delete Scope: Generation Exhausted"))?;
    connection
        .execute(
            "INSERT OR IGNORE INTO deleted_files SELECT id,?2 FROM storage_files WHERE scope=?1",
            params![scope.as_str(), integer(generation)?],
        )
        .map_err(db)?;
    connection.execute("DELETE FROM storage_retained_snapshots WHERE owner=?1", [scope.as_str()]).map_err(db)?;
    connection
        .execute("INSERT INTO storage_scope_deletions VALUES(?1,?2)", params![scope.as_str(), integer(position.sequence)?])
        .map_err(db)?;
    set_meta(connection, "deletion_generation", generation)?;
    set_meta(connection, "floor", position.sequence)?;
    Ok(())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use harmos_storage::{FileMetadata, StorageBackend, StoreHistory, execution::Execution};

    use super::*;
    use crate::backend::{Backend, BackendSettings};
    use crate::persisted::Scope;

    #[tokio::test]
    async fn scope_deletion_requires_settled_productions_without_touching_other_scopes() {
        // Open the Native Backend and Create an Unfinished Scoped Production.
        let history = StoreHistory {
            store: Identifier::new("store").unwrap(),
            epoch: Identifier::new("epoch").unwrap(),
        };
        let directory = tempfile::tempdir().unwrap();
        let backend = Backend::new(
            directory.path().join("storage.sqlite3"),
            history.clone(),
            BackendSettings {
                chunk_rows: NonZeroUsize::new(4).unwrap(),
                chunk_bytes: NonZeroUsize::new(4096).unwrap(),
            },
            Execution::default(),
        )
        .unwrap();
        backend.create_session().await.unwrap();
        let scope = Scope::CycleSeal("cycle").identifier().unwrap();
        let metadata = FileMetadata {
            id: Identifier::new("upload").unwrap(),
            scope: Some(scope.clone()),
            media_type: "application/octet-stream".into(),
            expected_length: None,
            expected_sha256: None,
        };
        let file = backend.create_file_production(&metadata).await.unwrap();
        let other = backend
            .create_file_production(&FileMetadata {
                id: Identifier::new("other-upload").unwrap(),
                scope: Some(Scope::CycleSeal("cycle/other").identifier().unwrap()),
                ..metadata
            })
            .await
            .unwrap();
        let at = StorageCursor { history, sequence: 1 };

        // Refuse the Exact Scope Before Writing Any Deletion State.
        {
            let state = backend.state.lock().await;
            assert!(matches!(delete_scope(&state.connection, &at, &scope), Err(harmos_storage::Error::InvalidInput { .. })));
            let deleted: i64 = state.connection.query_row("SELECT count(*) FROM deleted_files", [], |row| row.get(0)).unwrap();
            assert_eq!(deleted, 0);
        }

        // Settlement Unblocks Deletion Without Settling a Prefix-Related Scope.
        backend.delete_file_production(&file).await.unwrap();
        {
            let state = backend.state.lock().await;
            delete_scope(&state.connection, &at, &scope).unwrap();
            let deleted: String = state.connection.query_row("SELECT id FROM deleted_files", [], |row| row.get(0)).unwrap();
            assert_eq!(deleted, file.id.as_str());
        }
        assert_eq!(backend.get_file_production(&other).await.unwrap().phase, harmos_storage::FilePhase::Producing);
        backend.delete_session().await.unwrap();
    }
}
