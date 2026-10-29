/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(crate) mod plots;
pub(crate) mod telemetry;

pub(super) use plots::{ReadCache, ReadPlan, materialize, read_plan};

use harmos::storage::{Query, arrow::ArrowBatch, query::QueryArrow};

use crate::{runtime::Error, runtime::Result};

pub(super) async fn visit_batches(
    session: &crate::storage::Manager, at: &harmos::storage::StorageCursor, query: Query, mut visit: impl FnMut(ArrowBatch) -> Result<()>,
) -> Result<()> {
    let mut reader = session.query(query).await?;
    let mut schema = None;
    let mut rows = 0u64;
    while let Some(item) = reader.next().await? {
        match item {
            QueryArrow::Schema(value) => {
                if schema.is_some() || rows != 0 {
                    return Err(invalid("query emitted a repeated or late schema"));
                }
                schema = Some(value);
            }
            QueryArrow::Batch(batch) => {
                let announced = schema.as_ref().ok_or_else(|| invalid("query emitted data before its schema"))?;
                batch.validate_schema(announced.as_schema())?;
                rows = rows.checked_add(batch.as_record_batch().num_rows() as u64).ok_or_else(|| invalid("query row count overflows"))?;
                visit(batch)?;
            }
        }
    }
    let completed = reader.completion()?;
    if schema.is_none() || completed.at != *at || completed.rows != rows {
        return Err(invalid("query completion differs from the consumed evidence"));
    }
    Ok(())
}

pub(super) fn invalid(reason: &'static str) -> Error {
    Error::Storage(crate::storage::Error::Invalid(reason))
}
