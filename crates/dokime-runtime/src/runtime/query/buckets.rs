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
use crate::{
    RuntimeHandle,
    storage::sqlite::{Moments, QueryWork},
};
use dokime_domain::model::{
    identity::wire::Timestamp,
    query::{
        error::{EffectiveLimit, Failure, Reason},
        page::{PageLimits, SamplePage, SampleRow},
        reduction::{Bucket, BucketValue, StatisticValue},
        schema::FieldType,
    },
    values::numeric_value,
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

#[derive(Default)]
struct Sum {
    bins: BTreeMap<i16, i128>,
}

#[derive(Default)]
struct Reduction {
    moments: Moments,
    sum: Sum,
}

struct Group {
    samples: u64,
    source: String,
    versions: BTreeSet<u32>,
    fields: Vec<Reduction>,
}

#[derive(Default)]
struct Buckets {
    groups: BTreeMap<Key, Group>,
    partitions: BTreeSet<String>,
    memory: usize,
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
        let config = self.query_config().clone();
        let selection = plan.selection.clone();
        let execution = request.execution_id.clone();
        let stream = request.stream.clone();
        let (buckets, work, _) = self
            .local_storage
            .query_telemetry(plan.read, QueryWork::new(config.clone(), operation), Buckets::default(), move |state, _, sample| {
                if !samples::matches(&selection, &sample) {
                    return Ok(None);
                }
                let time = sample
                    .timestamp
                    .to_datetime()
                    .map_err(|_| Failure::new(Reason::Corruption, operation, "Invalid bucket input timestamp"))?
                    .unix_timestamp_nanos();
                let start = aligned(time, every);
                let partition = String::from_utf8(tokens::bounded_json(
                    &[execution.as_str(), stream.as_str(), sample.source.as_str()],
                    config.limits().token_bytes,
                    operation,
                )?)
                .expect("UTF-8 partition");
                if !state.partitions.contains(&partition) {
                    config.bound(state.partitions.len() + 1, config.limits().partitions, "partitions", operation)?;
                    state.partitions.insert(partition.clone());
                }
                let key = Key { start, partition };
                if !state.groups.contains_key(&key) {
                    if state.groups.len() >= config.limits().buckets {
                        let mut failure =
                            Failure::new(Reason::BucketLimit, operation, "Bucket state exhausted; narrow the window or increase every");
                        failure.limit = Some(EffectiveLimit {
                            name: "buckets".into(),
                            value: config.limits().buckets as u64,
                        });
                        return Err(failure);
                    }
                    let bytes = selection.schema().fields.len().saturating_mul(std::mem::size_of::<Reduction>())
                        + key.partition.len() * 2
                        + sample.source.len()
                        + selection.schema().versions.len() * 64
                        + 512;
                    config.bound(
                        state.memory.saturating_add(bytes),
                        config.limits().working_bytes as usize / 4,
                        "working_bytes",
                        operation,
                    )?;
                    state.memory += bytes;
                    let fields = selection.schema().fields.iter().map(|_| Reduction::default()).collect();
                    state.groups.insert(
                        key.clone(),
                        Group {
                            samples: 0,
                            source: sample.source.clone(),
                            versions: BTreeSet::new(),
                            fields,
                        },
                    );
                }
                let group = state.groups.get_mut(&key).expect("admitted bucket");
                group.samples += 1;
                group.versions.insert(sample.schema_version);
                for (field, reduction) in selection.schema().fields.iter().zip(&mut group.fields) {
                    let Some(value) = sample.values.get(&field.id).and_then(numeric_value) else {
                        continue;
                    };
                    let value = if value == 0.0 { 0.0 } else { value };
                    reduction.moments.push(value).map_err(|_| Failure::new(Reason::WorkLimit, operation, "Bucket counter overflow"))?;
                    if matches!(aggregate, harmos::Aggregate::Sum | harmos::Aggregate::Mean) {
                        let (exponent, mantissa) = Sum::parts(value);
                        if !reduction.sum.bins.contains_key(&exponent) {
                            config.bound(
                                state.memory.saturating_add(128),
                                config.limits().working_bytes as usize / 4,
                                "working_bytes",
                                operation,
                            )?;
                            state.memory += 128;
                        }
                        reduction.sum.push(exponent, mantissa, operation)?;
                    }
                }
                Ok(None)
            })
            .await?;
        work.check()?;
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
        for key in buckets.groups.keys().filter(|key| after.as_ref().is_none_or(|after| *key > after)) {
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
        for (key, group) in buckets.groups {
            if after.as_ref().is_some_and(|after| key <= *after) {
                continue;
            }
            let (start, end) = bounds(key.start, every, operation)?;
            let mut fields = Vec::new();
            for (field, reduction) in plan.selection.schema().fields.iter().zip(group.fields) {
                let count = reduction.moments.count;
                let value = if aggregate == harmos::Aggregate::Count {
                    Some(StatisticValue::Count(count))
                } else if count == 0 {
                    None
                } else {
                    Some(finite(
                        match aggregate {
                            harmos::Aggregate::Sum => reduction.sum.value(1),
                            harmos::Aggregate::Mean => reduction.sum.value(count),
                            _ => reduction.moments.value(aggregate).expect("validated bucket reducer"),
                        },
                        operation,
                    )?)
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

impl Sum {
    fn parts(value: f64) -> (i16, i128) {
        let bits = value.to_bits();
        let exponent = ((bits >> 52) & 0x7ff) as i16;
        let fraction = bits & ((1u64 << 52) - 1);
        let mantissa = if exponent == 0 { fraction } else { fraction | (1u64 << 52) };
        let mantissa = i128::from(mantissa) * if bits >> 63 == 0 { 1 } else { -1 };
        (if exponent == 0 { -1074 } else { exponent - 1075 }, mantissa)
    }

    fn push(&mut self, exponent: i16, mantissa: i128, operation: &str) -> Result<(), Box<Failure>> {
        let value = self.bins.entry(exponent).or_default();
        *value = value.checked_add(mantissa).ok_or_else(|| Failure::new(Reason::WorkLimit, operation, "Bucket sum counter overflow"))?;
        Ok(())
    }

    fn value(&self, divisor: u64) -> f64 {
        let mut total = 0.0;
        for (exponent, mantissa) in &self.bins {
            let scaled = *mantissa as f64 / divisor as f64;
            let term = if *exponent < -1022 {
                scaled * 2f64.powi(i32::from(*exponent) + 1074) * f64::from_bits(1)
            } else {
                scaled * 2f64.powi(i32::from(*exponent))
            };
            total += term;
        }
        total
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn aligned(timestamp: i128, every: u64) -> i128 {
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
    fn query_bucket_sum() {
        let values = [1e16, 1.0, -1e16, 3.0, f64::from_bits(1)];
        let mut forward = Sum::default();
        let mut reverse = Sum::default();
        for value in values {
            let (exponent, mantissa) = Sum::parts(value);
            forward.push(exponent, mantissa, "test.query").unwrap();
        }
        for value in values.into_iter().rev() {
            let (exponent, mantissa) = Sum::parts(value);
            reverse.push(exponent, mantissa, "test.query").unwrap();
        }
        assert_eq!(forward.value(1), 4.0);
        assert_eq!(forward.value(1).to_bits(), reverse.value(1).to_bits());
        assert_eq!(forward.value(5).to_bits(), reverse.value(5).to_bits());
    }

    #[test]
    fn query_epoch_alignment() {
        let start = aligned(-1, 1_000_000_000);
        let (start, end) = bounds(start, 1_000_000_000, "test.query").unwrap();
        assert_eq!((start.seconds, start.nanos, end.seconds, end.nanos), (-1, 0, 0, 0));
    }
}
