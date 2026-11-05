/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bounded Deterministic Epoch Bucket Pages

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};

// Third Party Imports

// Local Imports
use super::{
    samples::{self, Plan},
    snapshots::{QuerySnapshot, Seek},
    statistics::{finite, numeric_field},
    tokens,
};
use crate::RuntimeHandle;
use dokime_domain::model::{
    identity::wire::Timestamp,
    query::{
        error::{Failure, Reason},
        page::{PageLimits, SamplePage, SampleRow},
        reduction::{Bucket, BucketValue, StatisticValue},
        schema::FieldType,
    },
};
use dokime_operations::query::ListSamples;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Key {
    start: i128,
    partition: String,
}

struct Group {
    samples: u64,
    source: String,
    versions: BTreeSet<u32>,
    fields: Vec<super::datafusion::Reduced>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    pub(super) async fn query_bucket_samples(
        &self, request: ListSamples, snapshot: QuerySnapshot, binding: [u8; 32], limits: PageLimits, contracts: Vec<u8>,
    ) -> Result<SamplePage, Box<Failure>> {
        let operation = "dokime.query.list_samples";
        let bucket = request.bucket.as_ref().expect("bucket dispatch");
        bucket.validate(operation)?;
        let every = bucket.every_nanos;
        let aggregate = bucket.aggregate;
        let plan = Plan::new(&contracts, &request, &snapshot, self.query_config(), operation)?;
        for field in &plan.selection.schema().fields {
            numeric_field(field, operation)?;
        }
        let after = match &snapshot.after {
            None => None,
            Some(Seek::Bucket { start, partition }) => Some(Key {
                start: start
                    .to_datetime()
                    .map_err(|_| Failure::new(Reason::InvalidContinuation, operation, "Invalid bucket continuation bound"))?
                    .unix_timestamp_nanos(),
                partition: partition.clone(),
            }),
            _ => return Err(Failure::new(Reason::InvalidContinuation, operation, "Expected a bucket continuation")),
        };
        let reduced =
            super::datafusion::buckets(&self.local_storage, &plan, self.query_config(), every, &request.execution_id, &request.stream)
                .await?;
        let buckets: BTreeMap<_, _> = reduced
            .into_iter()
            .map(|bucket| {
                (
                    Key {
                        start: bucket.start,
                        partition: bucket.partition,
                    },
                    Group {
                        samples: bucket.samples,
                        source: bucket.source,
                        versions: bucket.versions,
                        fields: bucket.fields,
                    },
                )
            })
            .collect();
        let mut schema = plan.selection.schema().clone();
        for field in &mut schema.fields {
            field.value_type = if aggregate == harmos::Aggregate::Count { FieldType::Uint64 } else { FieldType::Float64 };
            field.nullable = aggregate != harmos::Aggregate::Count;
            if aggregate == harmos::Aggregate::Count {
                field.unit = None;
            }
        }
        let mut response = SamplePage {
            rows: Vec::new(),
            schema: Some(schema),
            snapshot: Some(snapshot.snapshot.clone()),
            limits: Some(limits.clone()),
            next: None,
            complete: false,
        };
        // Reserve The Longest Actual Seek Token Before Admitting Output Rows.
        let mut reserved = None;
        let mut longest = 0;
        for key in buckets.keys().filter(|key| after.as_ref().is_none_or(|after| *key > after)) {
            let seek = seek(key, operation)?;
            let size = tokens::bounded_json(&seek, self.query_config().limits().token_bytes, operation)?.len();
            if size > longest {
                longest = size;
                reserved = Some(seek);
            }
        }
        if let Some(key) = reserved {
            response.next = Some(self.query_continuation(&snapshot, binding, key, operation).await?);
        }
        let admitted = samples::page_budget(&response, &limits, operation)?;
        let mut bytes = 0usize;
        let mut memory = 0usize;
        let mut last = None;
        let mut more = false;
        for (key, group) in buckets {
            if after.as_ref().is_some_and(|after| key <= *after) {
                continue;
            }
            let (start, end) = bounds(key.start, every, operation)?;
            let mut fields = Vec::new();
            for (field, reduction) in plan.selection.schema().fields.iter().zip(group.fields) {
                let count = reduction.count;
                let value = if aggregate == harmos::Aggregate::Count {
                    Some(StatisticValue::Count(count))
                } else if count == 0 {
                    None
                } else {
                    Some(finite(reduction.number(aggregate).expect("validated bucket reducer"), operation)?)
                };
                fields.push(BucketValue {
                    field: field.id.clone(),
                    count,
                    value,
                    unit: if aggregate == harmos::Aggregate::Count { None } else { field.unit.clone() },
                    reducer: aggregate,
                });
            }
            let row = SampleRow::Bucket(Bucket {
                start: Some(start),
                end: Some(end),
                execution_id: request.execution_id.clone(),
                stream: request.stream.clone(),
                source: group.source,
                versions: group.versions.into_iter().collect(),
                sample_count: group.samples,
                fields,
            });
            use harmos::Message;
            let size = tokens::bounded_json(&row, admitted.bytes as usize, operation)?.len().max(row.encoded_size()) + 16;
            let row_memory = harmos::MemorySize::memory_size(&row);
            if response.rows.len() == admitted.rows as usize
                || size > (admitted.bytes as usize).saturating_sub(bytes)
                || row_memory > (self.query_config().limits().working_bytes as usize / 4).saturating_sub(memory)
            {
                if response.rows.is_empty() {
                    return Err(Failure::new(Reason::ResourceLimit, operation, "One complete bucket exceeds page or working-byte limits"));
                }
                more = true;
                break;
            }
            bytes += size;
            memory += row_memory;
            last = Some(key);
            response.rows.push(row);
        }
        response.next = if more {
            Some(self.query_continuation(&snapshot, binding, seek(last.as_ref().expect("delivered bucket"), operation)?, operation).await?)
        } else {
            None
        };
        response.complete = !more;
        samples::check_page(&response, &limits, operation)?;
        Ok(response)
    }
}

/* === Public Functions === */

/* === Private Functions === */

pub(super) fn aligned(timestamp: i128, every: u64) -> i128 {
    timestamp.div_euclid(i128::from(every)) * i128::from(every)
}

fn bounds(start: i128, every: u64, operation: &str) -> Result<(Timestamp, Timestamp), Box<Failure>> {
    let convert = |nanos| {
        time::OffsetDateTime::from_unix_timestamp_nanos(nanos)
            .map(Timestamp::from)
            .map_err(|_| Failure::new(Reason::InvalidWindow, operation, "Bucket bounds exceed the supported UTC range"))
    };
    Ok((convert(start)?, convert(start + i128::from(every))?))
}

fn seek(key: &Key, operation: &str) -> Result<Seek, Box<Failure>> {
    let (start, _) = bounds(key.start, 1, operation)?;
    Ok(Seek::Bucket { start, partition: key.partition.clone() })
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_epoch_alignment() {
        let start = aligned(-1, 1_000_000_000);
        let (start, end) = bounds(start, 1_000_000_000, "test.query").unwrap();
        assert_eq!((start.seconds, start.nanos, end.seconds, end.nanos), (-1, 0, 0, 0));
    }
}
