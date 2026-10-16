/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod plots;
mod statistics;
pub(crate) mod telemetry;

/* === Re-Exports === */
pub(super) use plots::materialize;

/* === Imports === */
// Standard Library Imports
use std::num::NonZeroUsize;
// Third Party Imports
use harmos::{
    operations::{Context, StreamSettings},
    storage::{
        arrow::ArrowBatch,
        query::{Query, QueryOutput},
    },
};
// Local Imports
use crate::{Error, Result};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Copy)]
pub(super) struct ReadContext<'a, 'ctx> {
    pub ctx: &'a Context<'ctx>,
    pub settings: &'a crate::config::StorageSettings,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
/// Every Arrow batch one query answers through the storage operation.
pub(super) async fn batches(
    ctx: &Context<'_>,
    settings: &crate::config::StorageSettings,
    query: Query,
) -> Result<Vec<ArrowBatch>> {
    let at = query.at.clone();
    let session = ctx.download::<harmos::storage::operations::Query>(
        query,
        StreamSettings {
            items: NonZeroUsize::MIN,
            // The Transport Claims Bytes With A u32 Semaphore Permit.
            bytes: NonZeroUsize::new(
                settings
                    .budget
                    .read_memory_bytes
                    .get()
                    .min(u32::MAX as usize),
            )
            .expect("nonzero read budget"),
        },
    )?;
    let (_, mut output, completion) = session.split();
    let mut schema = None;
    let mut rows = 0u64;
    let mut batches = Vec::new();
    while let Some(item) = output.next().await? {
        match &*item {
            QueryOutput::Schema(value) => {
                if schema.is_some() || rows != 0 {
                    return Err(invalid("query emitted a repeated or late schema"));
                }
                schema = Some(value.clone());
            }
            QueryOutput::Batch(batch) => {
                let announced = schema
                    .as_ref()
                    .ok_or_else(|| invalid("query emitted data before its schema"))?;
                batch.validate_schema(announced.as_schema())?;
                rows += batch.as_record_batch().num_rows() as u64;
                batches.push(batch.clone());
            }
        }
    }
    let completed = completion.join().await?;
    if schema.is_none() || completed.at != at || completed.rows != rows {
        return Err(invalid(
            "query completion differs from the consumed evidence",
        ));
    }
    Ok(batches)
}

pub(super) fn invalid(reason: &'static str) -> Error {
    Error::Storage(dokime::storage::Error::Invalid(reason))
}

/* === Private Functions === */

/* === Tests === */
