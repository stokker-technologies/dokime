/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Catalog-Pruned Native IPC Query Batches

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::Path;

// Third Party Imports
use arrow_array::{Array, RecordBatch, StructArray, UInt64Array};
use rusqlite::{Connection, types::Value};

// Local Imports
use super::{QueryRead, QueryWork, storage_error};
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

pub(crate) struct QueryProjection {
    pub metadata: Vec<&'static str>,
    pub decoded_bytes: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl State {
    pub(crate) async fn query_native_batches<T: Send + 'static>(
        &self, read: QueryRead, mut work: QueryWork, mut output: T, projection: Option<QueryProjection>,
        mut consume: impl FnMut(&mut T, RecordBatch, TelemetryContract, &QueryWork) -> Result<(), Box<Failure>> + Send + 'static,
    ) -> Result<(T, QueryWork), Box<Failure>> {
        let operation = work.operation;
        let root = self.0.readers.database.parent().expect("storage root").to_owned();
        let mut cancellation = work.cancellation();
        let result = self
            .read(move |connection| {
                Ok((|| {
                    if !crate::storage::sqlite::snapshots::available(connection, &read.cut)
                        .map_err(|error| storage_error(error, operation))?
                    {
                        return Err(Failure::new(Reason::SnapshotUnavailable, operation, "The exact query cut is unavailable"));
                    }
                    scan(connection, &root, &read, &mut work, projection.as_ref(), &mut |batch, contract, work| {
                        consume(&mut output, batch, contract, work)
                    })?;
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
    connection: &Connection, root: &Path, read: &QueryRead, work: &mut QueryWork, projection: Option<&QueryProjection>,
    consume: &mut impl FnMut(RecordBatch, TelemetryContract, &QueryWork) -> Result<(), Box<Failure>>,
) -> Result<(), Box<Failure>> {
    let operation = work.operation;
    let scan = || -> StorageResult<_> {
        let mut sql = String::from(
            "SELECT b.offset,b.metadata_bytes,b.body_bytes,b.rows,b.sha256,s.file_id,s.execution,s.contract,b.batch FROM streams s JOIN stream_batches b ON b.file_id=s.file_id WHERE s.stream=? AND b.last_position>? AND b.first_position<=? AND (b.file_id,b.batch) IN (SELECT file_id,batch FROM stream_captures WHERE execution=?)",
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
            Ok((reader::index(row)?, row.get::<_, String>(5)?, row.get::<_, Option<String>>(6)?, contract, row.get::<_, i64>(8)?))
        };
        let (index, id, execution, contract, batch_number) = catalog().map_err(|e| storage_error(e, operation))?;
        if !read.versions.contains(&contract.schema_version) {
            continue;
        }
        partitions.insert(id.clone());
        work.config.bound(partitions.len(), work.config.limits().partitions, "partitions", operation)?;
        work.charge(index.rows, index.body_bytes.saturating_add(u64::from(index.metadata_bytes)))?;
        let batch = (|| -> StorageResult<_> {
            let schema = ipc::contract_schema(&contract, execution.as_deref())?;
            let metadata = projection
                .map_or(&["position", "timestamp", "source", "sample_id", "received_at"][..], |projection| projection.metadata.as_slice());
            let maximum = projection.map_or(work.config.limits().working_bytes as usize / 2, |projection| projection.decoded_bytes);
            let projection = ipc::Projection::new(schema, metadata, &read.fields)?;
            ipc::read_query_batch(&root.join(format!("telemetry/{id}.arrow")), &index, &projection, maximum)
        })()
        .map_err(|e| storage_error(e, operation))?;
        // Capture Ranges Are Bounded By The Admitted Batch, Never By History Length.
        let ranges = (|| -> StorageResult<_> {
            let mut statement = connection.prepare_cached(
                "SELECT first_row,rows FROM stream_captures WHERE execution=?1 AND file_id=?2 AND batch=?3 ORDER BY first_row",
            )?;
            let mut rows = statement.query(rusqlite::params![read.execution, id, batch_number])?;
            let mut ranges = Vec::new();
            let maximum = work.config.limits().working_bytes as usize / 8 / std::mem::size_of::<(u64, u64)>();
            while let Some(row) = rows.next()? {
                if ranges.len() >= maximum {
                    return Err(Error::Data("query capture ranges exceed working budget"));
                }
                if ranges.len() == ranges.capacity() {
                    let next = ranges.capacity().saturating_mul(2).max(1).min(maximum);
                    ranges.reserve_exact(next - ranges.len());
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
            work.check()?;
            let admitted = row < batch.num_rows() && ranges.iter().any(|(first, end)| *first <= row as u64 && (row as u64) < *end) && {
                if positions.is_null(row) {
                    return Err(storage_error(Error::Corrupt("telemetry position is null"), operation));
                }
                let position = positions.value(row);
                position > read.after && position <= read.cut.at.sequence
            };
            if admitted {
                start.get_or_insert(row);
            } else if let Some(first) = start.take() {
                consume(batch.slice(first, row - first), contract.clone(), work)?;
            }
        }
    }
    Ok(())
}

/* === Tests === */
