/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Catalog-Pruned Native IPC Batches For The Batch Control

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    path::Path,
    sync::{Arc, atomic::AtomicBool},
};

// Third Party Imports
use arrow_array::{Array, RecordBatch, StructArray, UInt64Array};
use rusqlite::{Connection, types::Value};

// Local Imports
use super::{Cancellation, QueryRead, QueryWork, storage_error};
use crate::storage::{
    Error, Result as StorageResult, ipc,
    sqlite::{
        State,
        data::{
            position,
            telemetry::{catalog, reader},
        },
    },
};
use dokime_domain::model::{
    query::error::{Failure, Reason},
    telemetry::TelemetryContract,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl State {
    pub(crate) async fn query_native_batches<T: Send + 'static>(
        &self, read: QueryRead, mut work: QueryWork, mut output: T,
        mut consume: impl FnMut(&mut T, RecordBatch, TelemetryContract) -> Result<(), Box<Failure>> + Send + 'static,
    ) -> Result<(T, QueryWork), Box<Failure>> {
        let operation = work.operation;
        let root = self.0.readers.database.parent().expect("storage root").to_owned();
        work.cancelled = Arc::new(AtomicBool::new(false));
        let mut cancellation = Cancellation(Some(work.cancelled.clone()));
        let result = self
            .read(move |connection| {
                Ok((|| {
                    if !crate::storage::sqlite::snapshots::available(connection, &read.cut)
                        .map_err(|error| storage_error(error, operation))?
                    {
                        return Err(Failure::new(Reason::SnapshotUnavailable, operation, "The exact query cut is unavailable"));
                    }
                    scan(connection, &root, &read, &mut work, &mut |batch, contract| consume(&mut output, batch, contract))?;
                    work.check()?;
                    Ok((output, work))
                })())
            })
            .await
            .map_err(|error| storage_error(error, operation))?;
        cancellation.0 = None;
        result
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn scan(
    connection: &Connection, root: &Path, read: &QueryRead, work: &mut QueryWork,
    consume: &mut impl FnMut(RecordBatch, TelemetryContract) -> Result<(), Box<Failure>>,
) -> Result<(), Box<Failure>> {
    let operation = work.operation;
    let scan = || -> StorageResult<_> {
        let mut sql = String::from(
            "SELECT b.offset,b.metadata_bytes,b.body_bytes,b.rows,b.sha256,s.file_id,s.execution,s.contract,b.batch,b.first_position FROM streams s JOIN stream_batches b ON b.file_id=s.file_id WHERE s.stream=? AND b.last_position>? AND b.first_position<=? AND (b.file_id,b.batch) IN (SELECT file_id,batch FROM stream_captures WHERE execution=?)",
        );
        let mut args = vec![
            Value::Text(read.stream.clone()),
            Value::Text(position(read.after)),
            Value::Text(position(read.cut.at.sequence)),
            Value::Text(read.execution.clone()),
        ];
        if let Some(window) = &read.between {
            for (bound, comparison) in [(window.start, "b.last_at>=?"), (window.end, "b.first_at<=?")] {
                if let Some(bound) = bound {
                    sql.push_str(" AND ");
                    sql.push_str(comparison);
                    args.push(Value::Integer(catalog::micros(
                        bound.to_datetime().map_err(|_| Error::Invalid("invalid query time"))?.into(),
                    )?));
                }
            }
        }
        sql.push_str(" ORDER BY b.first_position,s.file_id,b.batch");
        Ok((sql, args))
    };
    let (sql, args) = scan().map_err(|e| storage_error(e, operation))?;
    let mut statement = connection.prepare(&sql).map_err(|e| storage_error(e.into(), operation))?;
    let mut rows = statement.query(rusqlite::params_from_iter(args)).map_err(|e| storage_error(e.into(), operation))?;
    let mut partitions = std::collections::BTreeSet::new();
    while let Some(row) = rows.next().map_err(|e| storage_error(e.into(), operation))? {
        work.check()?;
        let catalog = || -> StorageResult<_> {
            let encoded = row.get_ref(7)?.as_str().map_err(|_| Error::Corrupt("invalid query contract"))?;
            if encoded.len() > work.config.limits().schema_bytes as usize {
                return Err(Error::Data("query contract exceeds schema budget"));
            }
            let contract: TelemetryContract = serde_json::from_str(encoded)?;
            Ok((
                reader::index(row)?,
                row.get::<_, String>(5)?,
                row.get::<_, Option<String>>(6)?,
                contract,
                row.get::<_, i64>(8)?,
                super::super::super::sequence(&row.get::<_, String>(9)?)?,
            ))
        };
        let (index, id, execution, contract, batch_number, _first) = catalog().map_err(|e| storage_error(e, operation))?;
        if !read.versions.contains(&contract.schema_version) {
            continue;
        }
        partitions.insert(id.clone());
        work.config.bound(partitions.len(), work.config.limits().partitions, "partitions", operation)?;
        work.charge(index.rows, index.body_bytes.saturating_add(u64::from(index.metadata_bytes)))?;
        let batch = (|| -> StorageResult<_> {
            let schema = ipc::contract_schema(&contract, execution.as_deref())?;
            let metadata = ["position", "timestamp", "source", "sample_id", "received_at"];
            let projection = ipc::Projection::new(schema, &metadata, &read.fields)?;
            ipc::read_query_batch(
                &root.join(format!("telemetry/{id}.arrow")),
                &index,
                &projection,
                work.config.limits().working_bytes as usize / 2,
            )
        })()
        .map_err(|e| storage_error(e, operation))?;
        // Capture Ranges Are Bounded By The Admitted Batch, Never By History Length.
        let ranges = (|| -> StorageResult<_> {
            let mut statement = connection.prepare_cached(
                "SELECT first_row,rows FROM stream_captures WHERE execution=?1 AND file_id=?2 AND batch=?3 ORDER BY first_row",
            )?;
            let mut rows = statement.query(rusqlite::params![read.execution, id, batch_number])?;
            let mut ranges = Vec::new();
            while let Some(row) = rows.next()? {
                if ranges.len() >= work.config.limits().working_bytes as usize / 64 {
                    return Err(Error::Data("query capture ranges exceed working budget"));
                }
                let first = catalog::unsigned(row, 0)?;
                let count = catalog::unsigned(row, 1)?;
                ranges.push((first, first.saturating_add(count)));
            }
            Ok(ranges)
        })()
        .map_err(|e| storage_error(e, operation))?;
        let metadata = batch
            .column(0)
            .as_any()
            .downcast_ref::<StructArray>()
            .ok_or_else(|| storage_error(Error::Corrupt("telemetry metadata is not a struct"), operation))?;
        let positions = metadata
            .column_by_name("position")
            .and_then(|array| array.as_any().downcast_ref::<UInt64Array>())
            .ok_or_else(|| storage_error(Error::Corrupt("telemetry position is absent"), operation))?;
        let mut start = None;
        for row in 0..=batch.num_rows() {
            let admitted = row < batch.num_rows() && {
                let position = positions.value(row);
                position > read.after
                    && position <= read.cut.at.sequence
                    && ranges.iter().any(|(first, end)| *first <= row as u64 && (row as u64) < *end)
            };
            if admitted {
                start.get_or_insert(row);
            } else if let Some(first) = start.take() {
                consume(batch.slice(first, row - first), contract.clone())?;
            }
        }
    }
    Ok(())
}

/* === Tests === */
