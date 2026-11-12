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
use crate::{Error, Result, Store, Sample, Execution, streams};


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
        let (state, payload): (i32, Vec<u8>) = transaction.query_row("SELECT state,payload FROM executions WHERE id=?1", [execution], |row| Ok((row.get(0)?, row.get(1)?)))?;
        if state != 0 { return Err(Error::Invalid("Publish Samples: Select an Open Execution".into())); }
        let definition = Execution::decode(payload.as_slice())?;
        let catalog = harmos_protocol::v1::Catalog::decode(definition.catalog.as_slice())?;
        let pool = catalog.types.as_ref().map(|types| DescriptorPool::from_file_descriptor_set(types.clone()))
            .transpose().map_err(|error| Error::Invalid(format!("Publish Samples: Supply Valid Frozen Descriptors: {error}")))?;
        let mut position = crate::store::next_position(&transaction, execution)?;
        let mut positions = Vec::with_capacity(samples.len());
        {
            let mut insert = transaction.prepare_cached("INSERT INTO samples VALUES (?1,?2,?3,?4,?5,?6)")?;
            let mut insert_field = transaction.prepare_cached("INSERT INTO sample_fields VALUES (?1,?2,?3,?4,?5,?6,?7)")?;
            let mut open = transaction.prepare_cached("SELECT closed FROM streams WHERE execution=?1 AND stream=?2")?;
            for sample in samples {
                let closed: i32 = open.query_row(params![execution, sample.stream], |row| row.get(0))?;
                if closed != 0 { return Err(Error::Invalid("Publish Samples: Open the Execution Stream Before Writing".into())); }
                let mut sample = sample.clone();
                sample.position = position;
                insert.execute(params![execution, position as i64, sample.stream, streams::encode(&sample)?, sample.source_ns, sample.sequence as i64])?;
                if let Some(pool) = &pool {
                    let stream = catalog.streams.iter().find(|stream| stream.id == sample.stream)
                        .ok_or_else(|| Error::Invalid("Publish Samples: Declare the Stream in the Frozen Catalog".into()))?;
                    let descriptor = pool.get_message_by_name(&stream.row).ok_or_else(|| Error::Invalid("Publish Samples: Include the Frozen Row Descriptor".into()))?;
                    let row = DynamicMessage::decode(descriptor, sample.row.as_slice())?;
                    for field in row.descriptor().fields() {
                        let value = row.get_field(&field);
                        let number = match value.as_ref() {
                            Value::F64(value) => Some(*value), Value::F32(value) => Some(f64::from(*value)),
                            Value::I64(value) => Some(*value as f64), Value::U64(value) => Some(*value as f64),
                            Value::I32(value) => Some(f64::from(*value)), Value::U32(value) => Some(f64::from(*value)),
                            Value::Bool(value) => Some(u8::from(*value) as f64), _ => None,
                        }.filter(|value| value.is_finite());
                        let label = match value.as_ref() { Value::String(value) => Some(value.clone()), Value::EnumNumber(value) => Some(value.to_string()), _ => None };
                        insert_field.execute(params![execution, sample.stream, field.name(), position as i64, sample.source_ns, number, label])?;
                    }
                }
                positions.push(position);
                position += 1;
            }
        }
        transaction.commit()?;
        self.changed.send_modify(|revision| *revision += 1);
        Ok(positions)
    }
    pub fn stream_names(&self, execution: &str) -> Result<Vec<String>> {
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT stream FROM streams WHERE execution=?1 ORDER BY stream")?;
        Ok(statement.query_map([execution], |row| row.get(0))?.collect::<std::result::Result<_,_>>()?)
    }
    pub fn sample_window(&self, window: SampleWindow<'_>) -> Result<Vec<Sample>> {
        let mut sql = String::from("SELECT stream,payload FROM samples WHERE execution=? AND position<=? AND source_ns>=? AND source_ns<=?");
        let mut values: Vec<rusqlite::types::Value> = vec![window.execution.to_owned().into(), (window.through as i64).into(), window.start_ns.into(), window.end_ns.into()];
        if !window.streams.is_empty() {
            sql.push_str(" AND stream IN (");
            sql.push_str(&vec!["?"; window.streams.len()].join(","));
            sql.push(')');
            values.extend(window.streams.iter().cloned().map(Into::into));
        }
        for (stream, after) in window.after {
            sql.push_str(" AND (stream!=? OR position>?)");
            values.extend([stream.clone().into(), (*after as i64).into()]);
        }
        sql.push_str(if window.latest { " ORDER BY position DESC LIMIT ?" } else { " ORDER BY position LIMIT ?" });
        values.push((window.limit.get() as i64).into());
        let connection = self.lock()?;
        let mut statement = connection.prepare(&sql)?;
        let rows = statement.query_map(params_from_iter(values), |row| Ok((row.get::<_, String>(0)?, row.get::<_, Vec<u8>>(1)?)))?;
        let mut samples = rows.map(|row| { let (stream, bytes) = row?; streams::decode(stream, bytes) }).collect::<Result<Vec<_>>>()?;
        if window.latest { samples.reverse(); }
        Ok(samples)
    }
    pub fn numeric_bounds(&self, execution: &str, stream: &str, field: &str, through: u64) -> Result<Option<(i64, i64)>> {
        let connection = self.lock()?;
        let (start, end): (Option<i64>, Option<i64>) = connection.query_row("SELECT MIN(source_ns),MAX(source_ns) FROM sample_fields WHERE execution=?1 AND stream=?2 AND field=?3 AND position<=?4 AND number IS NOT NULL", params![execution,stream,field,through as i64], |row| Ok((row.get(0)?, row.get(1)?)))?;
        Ok(start.zip(end))
    }
    pub fn numeric_bins(&self, window: NumericWindow<'_>) -> Result<Vec<NumericBin>> {
        if window.end_ns < window.start_ns { return Err(Error::Invalid("Read Plot Window: Choose Increasing Bounds".into())); }
        let span = (i128::from(window.end_ns) - i128::from(window.start_ns) + 1).max(1);
        let width = ((span + window.bins.get() as i128 - 1) / window.bins.get() as i128).max(1).min(i64::MAX as i128) as i64;
        let connection = self.lock()?;
        let mut statement = connection.prepare("SELECT (source_ns-?4)/?6 AS bucket,COUNT(*),MIN(number),MAX(number),AVG(number),SUM(number*number),MIN(position),MAX(position),MIN(source_ns),MAX(source_ns) FROM sample_fields WHERE execution=?1 AND stream=?2 AND field=?3 AND source_ns>=?4 AND source_ns<=?5 AND position>?7 AND position<=?8 AND number IS NOT NULL GROUP BY bucket ORDER BY bucket")?;
        let rows = statement.query_map(params![window.execution,window.stream,window.field,window.start_ns,window.end_ns,width,window.after as i64,window.through as i64], |row| {
            Ok((row.get::<_,i64>(1)? as u64,row.get::<_,f64>(2)?,row.get::<_,f64>(3)?,row.get::<_,f64>(4)?,row.get::<_,f64>(5)?,row.get::<_,i64>(6)?,row.get::<_,i64>(7)?,row.get::<_,i64>(8)?,row.get::<_,i64>(9)?))
        })?;
        let mut result = Vec::new();
        let mut point = connection.prepare_cached("SELECT number FROM sample_fields WHERE execution=?1 AND stream=?2 AND field=?3 AND position=?4")?;
        for row in rows {
            let (count,min,max,mean,sum_squares,first,last,start_ns,end_ns) = row?;
            let first = point.query_row(params![window.execution,window.stream,window.field,first], |row| row.get(0))?;
            let last = point.query_row(params![window.execution,window.stream,window.field,last], |row| row.get(0))?;
            result.push(NumericBin { start_ns,end_ns,count,min,max,mean,sum_squares,first,last });
        }
        Ok(result)
    }
}


/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
