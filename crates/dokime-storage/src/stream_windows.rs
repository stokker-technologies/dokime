/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Indexed Stream Windows

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, ReflectMessage, Value};
use rusqlite::{params, params_from_iter};

// Local Imports
use crate::{Error, Execution, Result, Sample, Store, streams};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct SampleWindow<'a> {
    pub execution: &'a str,
    pub streams: &'a [String],
    pub after: &'a [(String, u64)],
    pub through: u64,
    pub start_ns: i64,
    pub end_ns: i64,
    pub latest: bool,
    pub limit: NonZeroUsize,
}

#[derive(Clone, Copy)]
pub struct NumericWindow<'a> {
    pub execution: &'a str,
    pub stream: &'a str,
    pub field: &'a str,
    pub after: u64,
    pub through: u64,
    pub start_ns: i64,
    pub end_ns: i64,
    pub bins: NonZeroUsize,
}

#[derive(Clone, Debug)]
pub struct NumericBin {
    pub start_ns: i64,
    pub end_ns: i64,
    pub count: u64,
    pub first_ns: i64,
    pub last_ns: i64,
    pub min_ns: i64,
    pub max_ns: i64,
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub sum_squares: f64,
    pub first: f64,
    pub last: f64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Store {
    pub fn sample_batch(&self, execution: &str, samples: &[Sample]) -> Result<Vec<u64>> {
        let mut connection = self.lock()?;
        let transaction = connection.transaction()?;
        let (state, payload): (i32, Vec<u8>) =
            transaction.query_row("SELECT state,payload FROM executions WHERE id=?1", [execution], |row| Ok((row.get(0)?, row.get(1)?)))?;
        if state != 0 {
            return Err(Error::Invalid("Publish Samples: Select an Open Execution".into()));
        }
        let definition = Execution::decode(payload.as_slice())?;
        let catalog = harmos_protocol::v1::Catalog::decode(definition.catalog.as_slice())?;
        let pool = catalog
            .types
            .as_ref()
            .map(|types| DescriptorPool::from_file_descriptor_set(types.clone()))
            .transpose()
            .map_err(|error| Error::Invalid(format!("Publish Samples: Supply Valid Frozen Descriptors: {error}")))?;
        let mut position = crate::store::next_position(&transaction, execution)?;
        let mut positions = Vec::with_capacity(samples.len());
        let mut batches = std::collections::BTreeMap::<String, Vec<Sample>>::new();
        {
            let mut insert = transaction.prepare_cached("INSERT INTO samples VALUES (?1,?2,?3,?4,?5,?6)")?;
            let mut insert_field = transaction.prepare_cached("INSERT INTO sample_fields VALUES (?1,?2,?3,?4,?5,?6,?7)")?;
            let mut open = transaction.prepare_cached("SELECT closed FROM streams WHERE execution=?1 AND stream=?2")?;
            for sample in samples {
                let closed: i32 = open.query_row(params![execution, sample.stream], |row| row.get(0))?;
                if closed != 0 {
                    return Err(Error::Invalid("Publish Samples: Open the Execution Stream Before Writing".into()));
                }
                let mut sample = sample.clone();
                sample.position = position;
                insert.execute(params![
                    execution,
                    position as i64,
                    sample.stream,
                    sample.encode_to_vec(),
                    sample.source_ns,
                    sample.sequence as i64
                ])?;
                if let Some(pool) = &pool {
                    let stream = catalog
                        .streams
                        .iter()
                        .find(|stream| stream.id == sample.stream)
                        .ok_or_else(|| Error::Invalid("Publish Samples: Declare the Stream in the Frozen Catalog".into()))?;
                    let descriptor = pool
                        .get_message_by_name(&stream.row)
                        .ok_or_else(|| Error::Invalid("Publish Samples: Include the Frozen Row Descriptor".into()))?;
                    let row = DynamicMessage::decode(descriptor, sample.row.as_slice())?;
                    for (field, number, label) in indexed_fields(&row, "") {
                        insert_field.execute(params![execution, sample.stream, field, position as i64, sample.source_ns, number, label])?;
                    }
                }
                batches.entry(sample.stream.clone()).or_default().push(sample);
                positions.push(position);
                position += 1;
            }
        }
        for (stream, samples) in batches {
            if let Some(first) = samples.first() {
                transaction.execute(
                    "INSERT INTO sample_batches VALUES (?1,?2,?3,?4)",
                    params![execution, first.position as i64, stream, streams::encode_batch(&samples)?],
                )?;
            }
        }
        transaction.commit()?;
        self.changed.send_modify(|revision| *revision += 1);
        Ok(positions)
    }

    pub fn stream_names(&self, execution: &str) -> Result<Vec<String>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT stream FROM streams WHERE execution=?1 ORDER BY stream")?;
        Ok(statement.query_map([execution], |row| row.get(0))?.collect::<std::result::Result<_, _>>()?)
    }

    pub fn sample_window(&self, window: SampleWindow<'_>) -> Result<Vec<Sample>> {
        let names = if window.streams.is_empty() { self.stream_names(window.execution)? } else { window.streams.to_vec() };
        if names.is_empty() {
            return Ok(Vec::new());
        }
        let mut selections = Vec::new();
        let mut values: Vec<rusqlite::types::Value> = Vec::new();
        let order = if window.latest { "DESC" } else { "ASC" };
        let index =
            if window.start_ns == i64::MIN && window.end_ns == i64::MAX { "samples_stream_position" } else { "samples_stream_time" };
        for stream in names {
            let after = window.after.iter().find(|(selected, _)| selected == &stream).map(|(_, after)| *after).unwrap_or(0);
            selections.push(format!("SELECT * FROM (SELECT stream,payload,position FROM samples INDEXED BY {index} WHERE execution=? AND stream=? AND position>? AND position<=? AND source_ns>=? AND source_ns<=? ORDER BY position {order} LIMIT ? )"));
            values.extend([
                window.execution.to_owned().into(),
                stream.into(),
                (after as i64).into(),
                (window.through as i64).into(),
                window.start_ns.into(),
                window.end_ns.into(),
                (window.limit.get() as i64).into(),
            ]);
        }
        let query = format!("SELECT stream,payload FROM ({}) ORDER BY position {order} LIMIT ?", selections.join(" UNION ALL "));
        values.push((window.limit.get() as i64).into());
        let connection = self.lock()?;
        let mut statement = connection.prepare(&query)?;
        let rows = statement.query_map(params_from_iter(values), |row| Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?)))?;
        let mut samples = rows
            .map(|row| {
                let (stream, bytes) = row?;
                streams::decode(stream, bytes)
            })
            .collect::<Result<Vec<_>>>()?;
        if window.latest {
            samples.reverse();
        }
        Ok(samples)
    }
    pub fn numeric_bounds(&self, execution: &str, stream: &str, field: &str, through: u64) -> Result<Option<(i64, i64)>> {
        let connection = self.lock()?;
        let (start, end): (Option<i64>, Option<i64>) = connection.query_row("SELECT (SELECT source_ns FROM sample_fields WHERE execution=?1 AND stream=?2 AND field=?3 AND position<=?4 AND number IS NOT NULL ORDER BY source_ns LIMIT 1),(SELECT source_ns FROM sample_fields WHERE execution=?1 AND stream=?2 AND field=?3 AND position<=?4 AND number IS NOT NULL ORDER BY source_ns DESC LIMIT 1)", params![execution,stream,field,through as i64], |row| Ok((row.get(0)?, row.get(1)?)))?;
        Ok(start.zip(end))
    }

    pub fn numeric_bins(&self, window: NumericWindow<'_>) -> Result<Vec<NumericBin>> {
        if window.end_ns < window.start_ns {
            return Err(Error::Invalid("Read Plot Window: Choose Increasing Bounds".into()));
        }
        let span = (i128::from(window.end_ns) - i128::from(window.start_ns) + 1).max(1);
        let width = ((span + window.bins.get() as i128 - 1) / window.bins.get() as i128).max(1);
        let connection = self.lock()?;
        let mut query=connection.prepare("SELECT position,source_ns,number FROM sample_fields WHERE execution=?1 AND stream=?2 AND field=?3 AND source_ns>=?4 AND source_ns<=?5 AND position>?6 AND position<=?7 AND number IS NOT NULL ORDER BY source_ns,position")?;
        let rows = query.query_map(
            params![
                window.execution,
                window.stream,
                window.field,
                window.start_ns,
                window.end_ns,
                window.after as i64,
                window.through as i64
            ],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, f64>(2)?)),
        )?;
        let mut bins = std::collections::BTreeMap::<usize, (NumericBin, i64, i64)>::new();
        for row in rows {
            let (position, source_ns, value) = row?;
            let index = ((i128::from(source_ns) - i128::from(window.start_ns)) / width) as usize;
            let (bin, first_position, last_position) = bins.entry(index).or_insert_with(|| {
                (
                    NumericBin {
                        start_ns: source_ns,
                        end_ns: source_ns,
                        count: 0,
                        min: value,
                        max: value,
                        mean: 0.0,
                        sum_squares: 0.0,
                        first: value,
                        last: value,
                        first_ns: source_ns,
                        last_ns: source_ns,
                        min_ns: source_ns,
                        max_ns: source_ns,
                    },
                    position,
                    position,
                )
            });
            // Each Retained Input Row Is Examined Once; Only Requested Buckets Survive.
            bin.count += 1;
            bin.mean += (value - bin.mean) / bin.count as f64;
            bin.sum_squares += value * value;
            bin.start_ns = bin.start_ns.min(source_ns);
            bin.end_ns = bin.end_ns.max(source_ns);
            if value < bin.min {
                bin.min = value;
                bin.min_ns = source_ns;
            }
            if value > bin.max {
                bin.max = value;
                bin.max_ns = source_ns;
            }
            if position < *first_position {
                *first_position = position;
                bin.first = value;
                bin.first_ns = source_ns;
            }
            if position > *last_position {
                *last_position = position;
                bin.last = value;
                bin.last_ns = source_ns;
            }
        }
        Ok(bins.into_values().map(|(bin, _, _)| bin).collect())
    }
}

/* === Public Functions === */

/* === Private Functions === */

pub(crate) fn indexed_fields(row: &DynamicMessage, prefix: &str) -> Vec<(String, Option<f64>, Option<String>)> {
    let mut values = Vec::new();
    for field in row.descriptor().fields() {
        if field.is_list() || field.is_map() {
            continue;
        }
        let path = if prefix.is_empty() { field.name().to_owned() } else { format!("{prefix}.{}", field.name()) };
        let value = row.get_field(&field);
        if let Value::Message(message) = value.as_ref() {
            values.extend(indexed_fields(message, &path));
            continue;
        }
        let number = match value.as_ref() {
            Value::F64(value) => Some(*value),
            Value::F32(value) => Some(*value as f64),
            Value::I64(value) => Some(*value as f64),
            Value::U64(value) => Some(*value as f64),
            Value::I32(value) => Some(*value as f64),
            Value::U32(value) => Some(*value as f64),
            Value::Bool(value) => Some(u8::from(*value) as f64),
            _ => None,
        }
        .filter(|value| value.is_finite());
        let label = match value.as_ref() {
            Value::String(value) => Some(value.clone()),
            Value::EnumNumber(value) => Some(value.to_string()),
            _ => None,
        };
        values.push((path, number, label));
    }
    values
}

/* === Tests === */
