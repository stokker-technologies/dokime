/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Durable Moments And Exact Order Statistics Updated In The Evidence Transaction.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use rusqlite::{Connection, OptionalExtension, params};

// Local Imports
use super::super::blob;
use crate::storage::Error;
use dokime_domain::model::values::Value;

/* === Constants === */

pub(in crate::storage::sqlite) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS storage_numeric_summaries (
    execution TEXT NOT NULL, stream TEXT NOT NULL, field TEXT NOT NULL, moments BLOB NOT NULL,
    PRIMARY KEY(execution,stream,field)
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS storage_numeric_checkpoints (
    execution TEXT NOT NULL, stream TEXT NOT NULL, field TEXT NOT NULL,
    first_position TEXT NOT NULL, through TEXT NOT NULL, moments BLOB NOT NULL,
    PRIMARY KEY(execution,stream,field,through)
) WITHOUT ROWID;
";

/* === Macros === */

/* === Type Aliases === */

#[cfg(feature = "native")]
type Key = (String, String, String);

/* === Structs === */

#[derive(Default)]
pub(in crate::storage::sqlite) struct Batch {
    fields: BTreeMap<String, BTreeMap<String, BTreeMap<String, Prefix>>>,
}

struct Prefix {
    moments: Moments,
    first: u64,
    through: u64,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct Moments {
    pub(crate) count: u64,
    present: u64,
    sum: f64,
    mean: f64,
    lowest: f64,
    highest: f64,
    deviations: f64,
    first: f64,
    last: f64,
    max_abs: f64,
    scale: f64,
    squares: f64,
    integer_low: i128,
    integer_high: i128,
    integer_total: u128,
    negative: bool,
    signed: bool,
    floating: bool,
    integers: bool,
}

pub(crate) struct NumericSummary {
    pub(crate) stream: String,
    pub(crate) field: String,
    pub(crate) statistics: Vec<(String, f64)>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Batch {
    #[tracing::instrument(level = "debug", name = "ingest.numeric", skip_all)]
    pub(in crate::storage::sqlite) fn push(
        &mut self, connection: &Connection, input: super::previews::Input<'_>, sample: &crate::telemetry::TelemetrySample,
        previews: &mut super::previews::Batch,
    ) -> Result<(), Error> {
        let super::previews::Input { execution, stream, position, .. } = input;
        if !self.fields.contains_key(execution) {
            self.fields.insert(execution.to_owned(), BTreeMap::new());
        }
        let streams = self.fields.get_mut(execution).expect("loaded execution");
        if !streams.contains_key(stream) {
            streams.insert(stream.to_owned(), BTreeMap::new());
        }
        let fields = streams.get_mut(stream).expect("loaded stream");
        for (field, typed) in &sample.values {
            if !fields.contains_key(field) {
                let existing: Option<Vec<u8>> = connection.query_row(
                    "SELECT moments FROM storage_numeric_checkpoints WHERE execution=?1 AND stream=?2 AND field=?3 ORDER BY through DESC LIMIT 1",
                    params![execution, stream, field], |row| row.get(0),
                ).optional()?;
                let moments = existing.as_deref().map(Moments::decode).transpose()?.unwrap_or_default();
                fields.insert(
                    field.to_owned(),
                    Prefix {
                        moments,
                        first: position,
                        through: position,
                    },
                );
            }
            let prefix = fields.get_mut(field).expect("loaded moments");
            prefix.first = prefix.first.min(position);
            prefix.through = prefix.through.max(position);
            prefix.moments.push_value(typed)?;
            let numeric = dokime_domain::model::values::numeric_value(typed).filter(|value| value.is_finite());
            if numeric.is_some() || matches!(typed, Value::Null) {
                previews.push(connection, input, field, numeric, typed)?;
            }
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", name = "ingest.numeric.save", skip_all)]
    pub(in crate::storage::sqlite) fn save(self, connection: &Connection) -> Result<(), Error> {
        let mut current = connection.prepare_cached(
            "INSERT INTO storage_numeric_summaries VALUES (?1,?2,?3,?4)
            ON CONFLICT(execution,stream,field) DO UPDATE SET moments=excluded.moments",
        )?;
        let mut checkpoint = connection.prepare_cached("INSERT INTO storage_numeric_checkpoints VALUES (?1,?2,?3,?4,?5,?6)")?;
        for (execution, streams) in self.fields {
            for (stream, fields) in streams {
                for (field, prefix) in fields {
                    let encoded = prefix.moments.encode();
                    if prefix.moments.count != 0 {
                        current.execute(params![execution, stream, field, encoded])?;
                    }
                    checkpoint.execute(params![
                        execution,
                        stream,
                        field,
                        super::super::position(prefix.first),
                        super::super::position(prefix.through),
                        encoded
                    ])?;
                }
            }
        }
        Ok(())
    }
}

impl Moments {
    pub(crate) fn value(&self, method: harmos::Aggregate) -> Option<f64> {
        use harmos::Aggregate;
        if self.count == 0 {
            return None;
        }
        let std = if self.deviations.is_nan() { f64::NAN } else { (self.deviations / self.count as f64).max(0.0).sqrt() };
        Some(match method {
            Aggregate::Count => self.count as f64,
            Aggregate::Sum => self.sum,
            Aggregate::Mean => self.mean,
            Aggregate::Min => self.lowest,
            Aggregate::Max => self.highest,
            Aggregate::Std => std,
            Aggregate::Rms => self.scale * (self.squares / self.count as f64).sqrt(),
            _ => return None,
        })
    }

    pub(crate) fn push(&mut self, value: f64) -> Result<(), Error> {
        if !value.is_finite() {
            return Err(Error::Data("numeric summary contains a nonfinite value"));
        }
        self.push_value(&Value::Number(value))
    }

    fn push_value(&mut self, original: &Value) -> Result<(), Error> {
        if matches!(original, Value::Null) {
            return Ok(());
        }
        self.present = self.present.checked_add(1).ok_or(Error::Data("value summary count overflow"))?;
        let value = match original {
            Value::Number(value) if value.is_finite() => {
                self.floating = true;
                *value
            }
            Value::Integer(value) => {
                self.signed = true;
                self.push_integer(i128::from(*value))?;
                *value as f64
            }
            Value::Unsigned(value) => {
                self.push_integer(i128::from(*value))?;
                *value as f64
            }
            Value::Number(_) => return Ok(()),
            _ => return Ok(()),
        };
        let before = self.mean;
        self.count = self.count.checked_add(1).ok_or(Error::Data("numeric summary count overflow"))?;
        self.sum += value;
        if self.count == 1 {
            self.first = value;
            self.lowest = value;
            self.highest = value;
        }
        self.last = value;
        self.lowest = self.lowest.min(value);
        self.highest = self.highest.max(value);
        self.max_abs = self.max_abs.max(value.abs());
        let magnitude = value.abs();
        if magnitude > self.scale {
            self.squares = 1.0 + self.squares * (self.scale / magnitude).powi(2);
            self.scale = magnitude;
        } else if magnitude != 0.0 {
            self.squares += (magnitude / self.scale).powi(2);
        }
        let n = self.count as f64;
        self.mean = if self.count == 1 {
            value
        } else if before.signum() == value.signum() {
            before + (value - before) / n
        } else {
            before * ((n - 1.0) / n) + value / n
        };
        self.deviations += (value - before) * (value - self.mean);
        Ok(())
    }

    fn push_integer(&mut self, value: i128) -> Result<(), Error> {
        if !self.integers {
            self.integer_low = value;
            self.integer_high = value;
            self.integers = true;
        }
        self.integer_low = self.integer_low.min(value);
        self.integer_high = self.integer_high.max(value);
        let negative = value < 0;
        let magnitude = value.unsigned_abs();
        if self.negative == negative {
            self.integer_total = self.integer_total.checked_add(magnitude).ok_or(Error::Data("integer summary total overflow"))?;
        } else if self.integer_total >= magnitude {
            self.integer_total -= magnitude;
        } else {
            self.integer_total = magnitude - self.integer_total;
            self.negative = negative;
        }
        if self.integer_total == 0 {
            self.negative = false;
        }
        Ok(())
    }

    fn encode(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(149);
        bytes.extend_from_slice(b"DQ1\0");
        bytes.extend_from_slice(&self.count.to_le_bytes());
        for value in
            [self.sum, self.mean, self.lowest, self.highest, self.deviations, self.first, self.last, self.max_abs, self.scale, self.squares]
        {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes.extend_from_slice(&self.integer_low.to_le_bytes());
        bytes.extend_from_slice(&self.integer_high.to_le_bytes());
        bytes.extend_from_slice(&self.integer_total.to_le_bytes());
        bytes
            .push(u8::from(self.negative) | (u8::from(self.signed) << 1) | (u8::from(self.floating) << 2) | (u8::from(self.integers) << 3));
        bytes.extend_from_slice(&self.present.to_le_bytes());
        bytes
    }

    pub(super) fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 149 || &bytes[..4] != b"DQ1\0" {
            return Err(Error::Corrupt("numeric summary format differs; use a fresh storage root"));
        }
        let count = u64::from_le_bytes(bytes[4..12].try_into().unwrap());
        let values = bytes[12..92].as_chunks::<8>().0;
        let values: [f64; 10] = std::array::from_fn(|index| f64::from_le_bytes(values[index]));
        let [sum, mean, lowest, highest, deviations, first, last, max_abs, scale, squares] = values;
        let flags = bytes[140];
        if flags & !15 != 0 {
            return Err(Error::Corrupt("numeric summary flags differ"));
        }
        Ok(Self {
            count,
            present: u64::from_le_bytes(bytes[141..149].try_into().unwrap()),
            sum,
            mean,
            lowest,
            highest,
            deviations,
            first,
            last,
            max_abs,
            scale,
            squares,
            integer_low: i128::from_le_bytes(bytes[92..108].try_into().unwrap()),
            integer_high: i128::from_le_bytes(bytes[108..124].try_into().unwrap()),
            integer_total: u128::from_le_bytes(bytes[124..140].try_into().unwrap()),
            negative: flags & 1 != 0,
            signed: flags & 2 != 0,
            floating: flags & 4 != 0,
            integers: flags & 8 != 0,
        })
    }
}

impl crate::storage::sqlite::State {
    #[cfg(feature = "native")]
    pub(crate) async fn document_statistic(
        &self, execution: String, stream: String, field: String, at: harmos::storage::StorageCursor, aggregate: harmos::Aggregate,
        mut work: crate::storage::sqlite::QueryWork,
    ) -> Result<Option<Option<Value>>, Error> {
        let _cancel = work.cancellation();
        let work = std::sync::Arc::new(work);
        let root = self.0.readers.database.parent().ok_or(Error::Invalid("storage root is absent"))?.to_owned();
        self.read(move |connection| {
            let checking = work.clone();
            connection.create_scalar_function("document_alive", 0, rusqlite::functions::FunctionFlags::SQLITE_UTF8, move |_| {
                checking.check().map(|()| true).map_err(|error| rusqlite::Error::UserFunctionError(error))
            })?;
            let result = (|| {
                work.check().map_err(|error| Error::Sqlite(rusqlite::Error::UserFunctionError(error)))?;
                let head = super::super::saved(connection)?;
                if head.history != at.history || head.sequence < at.sequence {
                    return Ok(None);
                }
                let checkpoint: Option<(String, Vec<u8>)> = connection
                    .query_row(
                        "SELECT through,moments FROM storage_numeric_checkpoints WHERE execution=?1 AND stream=?2 AND field=?3 AND through<=?4 ORDER BY through DESC LIMIT 1",
                        params![execution, stream, field, super::super::position(at.sequence)],
                        |row| Ok((row.get(0)?, row.get(1)?)),
                    )
                    .optional()?;
                let (through, mut moments) = match checkpoint {
                    Some((through, bytes)) => (through, Moments::decode(&bytes)?),
                    None => (super::super::position(0), Moments::default()),
                };
                let path = serde_json::to_string(&[&field])?;
                let indexed: bool = connection.query_row(
                    "SELECT EXISTS(SELECT 1 FROM storage_distinct_values WHERE execution=?1 AND kind='telemetry' AND source=?2 AND path=?3)",
                    params![execution, stream, path], |row| row.get(0),
                )?;
                if !indexed {
                    if aggregate == harmos::Aggregate::Median {
                        return replay_median(connection, &root, &(execution, stream, field), &at, &work).map(Some);
                    }
                    let partial: bool = connection.query_row(
                        "SELECT EXISTS(SELECT 1 FROM streams s JOIN stream_batches b ON b.file_id=s.file_id
                        WHERE s.stream=?2 AND b.last_position>?3 AND b.first_position<=?4
                        AND (b.file_id,b.batch) IN (SELECT file_id,batch FROM stream_captures WHERE execution=?1))",
                        params![execution, stream, through, super::super::position(at.sequence)], |row| row.get(0),
                    )?;
                    if partial {
                        return Ok(None);
                    }
                }
                let mut tail = connection.prepare("SELECT v.value,p.value_type FROM storage_value_postings p JOIN storage_distinct_values v
                    USING(execution,kind,source,path,value_key) WHERE p.execution=?1 AND p.kind='telemetry' AND p.source=?2
                    AND p.path=?3 AND p.position>?4 AND p.position<=?5 ORDER BY p.position")?;
                let mut rows = tail.query(params![execution, stream, path, through, super::super::position(at.sequence)])?;
                while let Some(row) = rows.next()? {
                    work.check().map_err(|error| Error::Sqlite(rusqlite::Error::UserFunctionError(error)))?;
                    let value = super::super::distinct::posted_value(super::super::blob(row, 0)?, row.get(1)?)?;
                    moments.push_value(&value)?;
                }
                if aggregate != harmos::Aggregate::Count && moments.present != moments.count {
                    return Ok(None);
                }
                if aggregate == harmos::Aggregate::Median && moments.count != 0 {
                    return document_median(connection, &(execution, stream, field), &at, moments).map(Some);
                }
                if matches!(aggregate, harmos::Aggregate::Percentile(_)) { return Ok(None); }
                document_value(moments, aggregate).map(Some)
            })();
            connection.remove_function("document_alive", 0)?;
            Ok(result)
        })
        .await?
    }

    pub(crate) async fn numeric_summaries(&self, execution: String) -> Result<Vec<NumericSummary>, Error> {
        let root = self.0.readers.database.parent().ok_or(Error::Invalid("storage root is absent"))?.to_owned();
        self.read(move |connection| {
            Ok((|| {
                let head = super::super::saved(connection)?;
                let through = harmos::storage::StorageCursor {
                    history: head.history,
                    sequence: head.sequence,
                };
                let mut summaries = numeric_summaries(connection, execution.clone())?;
                exact_percentiles(connection, &root, &execution, &through, &mut summaries, &Default::default())?;
                Ok(summaries)
            })())
        })
        .await?
    }
}

/* === Public Functions === */

pub(crate) fn numeric_summaries(connection: &Connection, execution: String) -> Result<Vec<NumericSummary>, Error> {
    let mut query =
        connection.prepare("SELECT stream,field,moments FROM storage_numeric_summaries WHERE execution=?1 ORDER BY stream,field")?;
    let mut rows = query.query([&execution])?;
    let mut summaries = Vec::new();
    while let Some(row) = rows.next()? {
        let stream: String = row.get(0)?;
        let field: String = row.get(1)?;
        let moments = Moments::decode(blob(row, 2)?)?;
        let std = (moments.deviations / moments.count as f64).max(0.0).sqrt();
        let statistics: Vec<_> = vec![
            ("count", moments.count as f64),
            ("first", moments.first),
            ("last", moments.last),
            ("min", moments.lowest),
            ("max", moments.highest),
            ("mean", moments.mean),
            ("std", std),
            ("rms", moments.mean.hypot(std)),
            ("max_abs", moments.max_abs),
            ("sum", moments.sum),
        ]
        .into_iter()
        .map(|(name, value)| (name.to_owned(), value))
        .collect();
        if statistics.iter().any(|(_, value)| !value.is_finite()) {
            return Err(Error::Data("numeric aggregate exceeds finite range"));
        }
        summaries.push(NumericSummary { stream, field, statistics });
    }
    Ok(summaries)
}

pub(super) fn exact_percentiles(
    connection: &Connection, root: &std::path::Path, execution: &str, through: &harmos::storage::StorageCursor,
    summaries: &mut [NumericSummary], requested: &std::collections::BTreeSet<(String, String, u8)>,
) -> Result<(), Error> {
    let fields = super::previews::report_previews(connection, execution.into(), through.clone())?;
    let mut streams = BTreeMap::<String, Vec<usize>>::new();
    for (index, summary) in summaries.iter().enumerate() {
        streams.entry(summary.stream.clone()).or_default().push(index);
    }
    for (stream, indices) in streams {
        let keys: Vec<_> = indices.iter().map(|index| summaries[*index].field.clone()).collect();
        let retained: Vec<_> = keys
            .iter()
            .map(|key| fields.iter().find(|field| field.stream == stream && field.field == *key && field.exact.len() as u64 == field.total))
            .collect();
        if retained.iter().all(Option::is_some) {
            for (index, field) in indices.iter().zip(retained) {
                let mut values: Vec<_> = field
                    .expect("retained field")
                    .exact
                    .iter()
                    .filter_map(|(_, value)| dokime_domain::model::values::numeric_value(value).filter(|value| value.is_finite()))
                    .collect();
                values.sort_by(f64::total_cmp);
                add_percentiles(&mut summaries[*index], requested, values.len() as u64, |ordinal| Ok(values[ordinal as usize]))?;
            }
            continue;
        }
        // The Read Path Owns This Disposable External Sort; Ingestion Retains No Rank History.
        let scratch = tempfile::tempdir()?;
        let mut replay = Connection::open(scratch.path().join("percentiles.sqlite"))?;
        replay.execute_batch(
            "PRAGMA cache_size=-2048; PRAGMA temp_store=FILE; PRAGMA mmap_size=0;
            CREATE TABLE values_replay (field INTEGER NOT NULL,value BLOB NOT NULL);",
        )?;
        let mut counts = vec![0u64; keys.len()];
        {
            let transaction = replay.transaction()?;
            let mut insert = transaction.prepare("INSERT INTO values_replay VALUES (?1,?2)")?;
            let mut after = 0;
            let bytes_per_row = keys.len().saturating_mul(std::mem::size_of::<Option<f64>>()).saturating_add(128);
            let page_rows = (8 * 1024 * 1024 / bytes_per_row).clamp(1, 65_536);
            loop {
                let scan = super::super::Scan::new(
                    "stream",
                    through.sequence,
                    0,
                    harmos::storage::ScanPage {
                        after,
                        rows: page_rows.try_into().unwrap(),
                        bytes: (8 * 1024 * 1024).try_into().unwrap(),
                    },
                )
                .filter("execution", Some(execution.into()))
                .filter("stream", Some(stream.clone()));
                let rows = super::reader::numeric(connection, root, &scan, &keys, &Default::default())?;
                if rows.is_empty() {
                    break;
                }
                for (position, row) in rows {
                    if position <= after {
                        return Err(Error::Corrupt("numeric replay did not advance"));
                    }
                    after = position;
                    for (field, value) in row.values.into_iter().enumerate() {
                        if let Some(value) = value {
                            let key = ordered(value).to_be_bytes();
                            insert.execute(params![field as i64, key.as_slice()])?;
                            counts[field] += 1;
                        }
                    }
                }
            }
            drop(insert);
            transaction.commit()?;
        }
        replay.execute_batch("CREATE INDEX values_order ON values_replay(field,value)")?;
        let mut rank = replay.prepare("SELECT value FROM values_replay WHERE field=?1 ORDER BY value LIMIT 1 OFFSET ?2")?;
        for (field, index) in indices.into_iter().enumerate() {
            add_percentiles(&mut summaries[index], requested, counts[field], |ordinal| {
                let bytes: Vec<u8> = rank
                    .query_row(params![field as i64, i64::try_from(ordinal).map_err(|_| Error::Data("numeric rank overflow"))?], |row| {
                        row.get(0)
                    })?;
                let key = u64::from_be_bytes(bytes.try_into().map_err(|_| Error::Corrupt("numeric replay key width differs"))?);
                Ok(f64::from_bits(if key >> 63 != 0 { key ^ (1 << 63) } else { !key }))
            })?;
        }
    }
    Ok(())
}

/* === Private Functions === */

#[cfg(feature = "native")]
fn replay_median(
    connection: &Connection, root: &std::path::Path, key: &Key, at: &harmos::storage::StorageCursor,
    work: &crate::storage::sqlite::QueryWork,
) -> Result<Option<Value>, Error> {
    let maximum = (work.config.limits().working_bytes as usize / 4 / std::mem::size_of::<Value>()).max(1);
    let value = select_median(maximum, |consume| {
        let mut after = 0;
        let page_rows = (work.config.limits().working_bytes as usize / 2 / std::mem::size_of::<(u64, Value)>())
            .clamp(1, work.config.limits().max_page_rows as usize);
        loop {
            let scan = super::super::Scan::new(
                "stream",
                at.sequence,
                0,
                harmos::storage::ScanPage {
                    after,
                    rows: page_rows.try_into().unwrap(),
                    bytes: (work.config.limits().working_bytes as usize).try_into().unwrap(),
                },
            )
            .filter("execution", Some(key.0.clone()))
            .filter("stream", Some(key.1.clone()));
            let rows = super::reader::typed(connection, root, &scan, &key.2, work)?;
            if rows.is_empty() {
                break;
            }
            for (position, value) in rows {
                work.check().map_err(|error| Error::Sqlite(rusqlite::Error::UserFunctionError(error)))?;
                if position <= after {
                    return Err(Error::Corrupt("median replay did not advance"));
                }
                after = position;
                if !matches!(value, Value::Null) {
                    consume(value)?;
                }
            }
        }
        Ok(())
    })?;
    work.check().map_err(|error| Error::Sqlite(rusqlite::Error::UserFunctionError(error)))?;
    Ok(value)
}

#[cfg(feature = "native")]
fn median_key(value: &Value) -> Result<[u8; 11], Error> {
    if dokime_domain::model::values::numeric_value(value).is_none_or(|value| !value.is_finite()) {
        return Err(Error::Data("median replay is not finite numeric evidence"));
    }
    let encoded = super::super::distinct::sort_key(value)?;
    let mut key = [0; 11];
    key.get_mut(..encoded.len()).ok_or(Error::Corrupt("numeric sort key is too wide"))?.copy_from_slice(&encoded);
    Ok(key)
}

#[cfg(feature = "native")]
fn select_median(
    maximum: usize, mut visit: impl FnMut(&mut dyn FnMut(Value) -> Result<(), Error>) -> Result<(), Error>,
) -> Result<Option<Value>, Error> {
    struct Bucket {
        count: u64,
        low: [u8; 11],
        high: [u8; 11],
        value: Value,
    }
    struct Group {
        bucket: Bucket,
        ranks: [Option<u64>; 2],
    }
    let select = |buckets: BTreeMap<u32, Bucket>, ranks: [Option<u64>; 2]| -> Result<Vec<Group>, Error> {
        let mut offset = 0;
        let mut groups = Vec::new();
        let wanted = ranks.iter().flatten().count();
        for bucket in buckets.into_values() {
            let end = offset + bucket.count;
            let selected = ranks.map(|rank| rank.filter(|rank| *rank >= offset && *rank < end).map(|rank| rank - offset));
            if selected.iter().any(Option::is_some) {
                groups.push(Group { bucket, ranks: selected });
            }
            offset = end;
        }
        if groups.iter().map(|group| group.ranks.iter().flatten().count()).sum::<usize>() != wanted {
            return Err(Error::Corrupt("median ranks are absent from replay"));
        }
        Ok(groups)
    };

    // Partition Exact Numeric Exponents; Their Count Is Independent of History Length.
    let mut buckets = BTreeMap::<u32, Bucket>::new();
    let mut count = 0u64;
    let mut signed = false;
    let mut floating = false;
    visit(&mut |value| {
        let key = median_key(&value)?;
        let prefix = u32::from_be_bytes([0, key[0], key[1], key[2]]);
        let bucket = buckets.entry(prefix).or_insert_with(|| Bucket {
            count: 0,
            low: key,
            high: key,
            value: value.clone(),
        });
        bucket.count += 1;
        bucket.low = bucket.low.min(key);
        bucket.high = bucket.high.max(key);
        signed |= matches!(value, Value::Integer(_));
        floating |= matches!(value, Value::Number(_));
        count += 1;
        Ok(())
    })?;
    if count == 0 {
        return Ok(None);
    }
    let mut pending = select(buckets, [Some((count - 1) / 2), Some(count / 2)])?;
    let mut leaves = Vec::new();
    let mut selected = [None, None];

    // Refine Only Buckets Containing a Requested Rank, Skipping Common Key Bytes.
    while let Some(group) = pending.pop() {
        let bucket = &group.bucket;
        if bucket.low == bucket.high {
            for (index, rank) in group.ranks.into_iter().enumerate() {
                if rank.is_some() {
                    selected[index] = Some(bucket.value.clone());
                }
            }
            continue;
        }
        if bucket.count <= maximum as u64 {
            leaves.push(group);
            continue;
        }
        let byte = bucket
            .low
            .iter()
            .zip(bucket.high)
            .position(|(low, high)| *low != high)
            .ok_or(Error::Corrupt("median bucket does not partition"))?;
        let mut buckets = BTreeMap::<u32, Bucket>::new();
        visit(&mut |value| {
            let key = median_key(&value)?;
            if key < bucket.low || key > bucket.high {
                return Ok(());
            }
            let child = buckets.entry(u32::from(key[byte])).or_insert_with(|| Bucket {
                count: 0,
                low: key,
                high: key,
                value: value.clone(),
            });
            child.count += 1;
            child.low = child.low.min(key);
            child.high = child.high.max(key);
            Ok(())
        })?;
        pending.extend(select(buckets, group.ranks)?);
    }

    // At Most Two Small Buckets Share One Bounded Collection Pass.
    if !leaves.is_empty() {
        let mut values = leaves.iter().map(|group| Vec::with_capacity(group.bucket.count as usize)).collect::<Vec<_>>();
        visit(&mut |value| {
            let key = median_key(&value)?;
            if let Some(index) = leaves.iter().position(|group| key >= group.bucket.low && key <= group.bucket.high) {
                if values[index].len() == maximum {
                    return Err(Error::Corrupt("median bucket grew at a fixed cut"));
                }
                values[index].push(value);
            }
            Ok(())
        })?;
        for (group, mut values) in leaves.into_iter().zip(values) {
            if values.len() as u64 != group.bucket.count {
                return Err(Error::Corrupt("median bucket count changed at a fixed cut"));
            }
            for (index, rank) in group.ranks.into_iter().enumerate() {
                if let Some(rank) = rank {
                    let (_, value, _) = values.select_nth_unstable_by(rank as usize, |left, right| {
                        dokime_domain::model::values::numeric_cmp(left, right).expect("checked finite numeric values")
                    });
                    selected[index] = Some(value.clone());
                }
            }
        }
    }
    let expression = crate::expressions::telemetry("replay", "value").median();
    let mut fold = crate::document::frame::Fold::new(&expression).map_err(|_| Error::Invalid("median fold is invalid"))?;
    for (index, value) in selected.into_iter().enumerate() {
        let mut value = value.ok_or(Error::Corrupt("median rank is absent"))?;
        if floating {
            value = Value::Number(dokime_domain::model::values::numeric_value(&value).ok_or(Error::Corrupt("median rank is not numeric"))?);
        }
        fold.push_at(index as u64, Some(&value)).map_err(|_| Error::Corrupt("median rank cannot be reduced"))?;
    }
    let value = fold.finish_checked(&mut || Ok(())).map_err(|_| Error::Data("median is outside its numeric range"))?;
    Ok(match value {
        Some(Value::Unsigned(value)) if signed && i64::try_from(value).is_ok() => Some(Value::Integer(value as i64)),
        value => value,
    })
}

#[cfg(feature = "native")]
fn document_value(moments: Moments, aggregate: harmos::Aggregate) -> Result<Option<Value>, Error> {
    use harmos::Aggregate;
    if aggregate == Aggregate::Count {
        return Ok(Some(Value::Unsigned(moments.present)));
    }
    if moments.count == 0 {
        return Ok((aggregate == Aggregate::Sum).then_some(Value::Unsigned(0)));
    }
    if !moments.floating {
        let exact = match aggregate {
            Aggregate::Min => Some((moments.integer_low < 0, moments.integer_low.unsigned_abs(), 1)),
            Aggregate::Max => Some((moments.integer_high < 0, moments.integer_high.unsigned_abs(), 1)),
            Aggregate::Sum => Some((moments.negative, moments.integer_total, 1)),
            Aggregate::Mean => Some((moments.negative, moments.integer_total, u128::from(moments.count))),
            _ => None,
        };
        if let Some((negative, magnitude, denominator)) = exact {
            return exact_value(negative, magnitude, denominator, moments.signed).map(Some);
        }
    }
    Ok(moments.value(aggregate).filter(|value| value.is_finite()).map(Value::Number))
}

#[cfg(feature = "native")]
fn exact_value(negative: bool, magnitude: u128, denominator: u128, signed: bool) -> Result<Value, Error> {
    let mut a = magnitude;
    let mut b = denominator;
    while b != 0 {
        (a, b) = (b, a % b);
    }
    let magnitude = magnitude / a;
    let denominator = denominator / a;
    if denominator != 1 {
        let value = magnitude as f64 / denominator as f64;
        return Ok(Value::Number(if negative { -value } else { value }));
    }
    if negative {
        let magnitude = i128::try_from(magnitude).map_err(|_| Error::Data("signed aggregate is out of range"))?;
        return i64::try_from(-magnitude).map(Value::Integer).map_err(|_| Error::Data("signed aggregate is out of range"));
    }
    if signed && let Ok(value) = i64::try_from(magnitude) {
        return Ok(Value::Integer(value));
    }
    u64::try_from(magnitude).map(Value::Unsigned).map_err(|_| Error::Data("unsigned aggregate is out of range"))
}

#[cfg(feature = "native")]
fn document_median(
    connection: &Connection, key: &Key, at: &harmos::storage::StorageCursor, moments: Moments,
) -> Result<Option<Value>, Error> {
    let expression = crate::expressions::telemetry(&key.1, &key.2).median();
    let mut fold = crate::document::frame::Fold::new(&expression).map_err(|_| Error::Invalid("median fold is invalid"))?;
    // Keep The Ordered Dictionary Outer: Reversing This Join Scans Every Value For Each Posting.
    // Equal Numeric Values Need No Position Tie-Breaker, Which Would Sort Large Duplicate Groups.
    let mut statement = connection.prepare(
        "SELECT d.value_key,p.value_type FROM storage_distinct_values d INDEXED BY storage_distinct_order
        CROSS JOIN storage_value_postings p INDEXED BY storage_value_numeric USING(execution,kind,source,path,value_key)
        WHERE d.execution=?1 AND d.kind='telemetry' AND d.source=?2 AND d.path=?3 AND p.position<=?4
        AND p.value_type IN (1,2,3) AND document_alive()
        ORDER BY d.sort_key,d.value_key LIMIT 1 OFFSET ?5",
    )?;
    let path = serde_json::to_string(&[&key.2])?;
    for (index, ordinal) in [(moments.count - 1) / 2, moments.count / 2].into_iter().enumerate() {
        let ordinal = i64::try_from(ordinal).map_err(|_| Error::Data("median rank exceeds SQLite range"))?;
        let (selected, kind): (Vec<u8>, u8) = statement
            .query_row(params![key.0, key.1, path, super::super::position(at.sequence), ordinal], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let bytes: Vec<u8> = connection.query_row(
            "SELECT value FROM storage_distinct_values WHERE execution=?1 AND kind='telemetry' AND source=?2 AND path=?3 AND value_key=?4",
            params![key.0, key.1, path, selected],
            |row| row.get(0),
        )?;
        let mut value = super::super::distinct::posted_value(&bytes, kind)?;
        if moments.floating {
            value =
                Value::Number(dokime_domain::model::values::numeric_value(&value).ok_or(Error::Corrupt("median posting is not numeric"))?);
        }
        fold.push_at(index as u64, Some(&value)).map_err(|_| Error::Corrupt("median posting cannot be reduced"))?;
    }
    let value = fold.finish_checked(&mut || Ok(())).map_err(|_| Error::Data("median is outside its numeric range"))?;
    Ok(match value {
        Some(Value::Unsigned(value)) if moments.signed && i64::try_from(value).is_ok() => Some(Value::Integer(value as i64)),
        value => value,
    })
}

fn add_percentiles(
    summary: &mut NumericSummary, requested: &std::collections::BTreeSet<(String, String, u8)>, count: u64,
    mut rank: impl FnMut(u64) -> Result<f64, Error>,
) -> Result<(), Error> {
    let expected = summary.statistics.iter().find(|(name, _)| name == "count").map(|(_, count)| *count);
    if count == 0 || expected != Some(count as f64) {
        return Err(Error::Corrupt("numeric replay count differs from saved summary"));
    }
    let mut ranks = std::collections::BTreeSet::from([50, 95, 99]);
    ranks.extend(
        requested
            .iter()
            .filter(|(stream, field, _)| crate::plot::stream_matches_alias(&summary.stream, stream) && *field == summary.field)
            .map(|(_, _, rank)| *rank),
    );
    for percentile in ranks {
        if percentile > 100 {
            return Err(Error::Data("invalid percentile rank"));
        }
        let position = f64::from(percentile) / 100.0 * (count - 1) as f64;
        let low = rank(position.floor() as u64)?;
        let high = rank(position.ceil() as u64)?;
        let fraction = position - position.floor();
        let value = interpolate(low, high, fraction);
        if percentile == 50 {
            summary.statistics.push(("median".into(), value));
        }
        if percentile != 50
            || requested.iter().any(|(stream, field, rank)| {
                *rank == 50 && *field == summary.field && crate::plot::stream_matches_alias(&summary.stream, stream)
            })
        {
            summary.statistics.push((format!("p{percentile}"), value));
        }
    }
    Ok(())
}

fn interpolate(low: f64, high: f64, fraction: f64) -> f64 {
    if low == high {
        low
    } else if low.signum() == high.signum() {
        low + (high - low) * fraction
    } else {
        low * (1.0 - fraction) + high * fraction
    }
}

fn ordered(value: f64) -> u64 {
    let bits = value.to_bits();
    if bits >> 63 != 0 { !bits } else { bits ^ (1 << 63) }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn radix_median_matches_typed_folds_across_bucket_boundaries() {
        let cases = [
            vec![],
            vec![Value::Number(3.5); 19],
            (-32..=32).rev().map(Value::Integer).collect(),
            vec![
                Value::Unsigned(9_007_199_254_740_995),
                Value::Unsigned(u64::MAX),
                Value::Unsigned(9_007_199_254_740_993),
                Value::Unsigned(9_007_199_254_740_991),
            ],
            vec![Value::Integer(i64::MIN), Value::Integer(-9), Value::Unsigned(7), Value::Integer(-8), Value::Unsigned(u64::MAX)],
            vec![Value::Number(-0.0), Value::Number(0.0), Value::Number(f64::MIN_POSITIVE), Value::Number(-f64::MIN_POSITIVE)],
            vec![Value::Integer(-4), Value::Unsigned(7), Value::Number(2.5), Value::Number(10.0)],
        ];
        for values in cases {
            let expression = crate::expressions::telemetry("fixture", "value").median();
            let mut reference = crate::document::frame::Fold::new(&expression).unwrap();
            for (index, value) in values.iter().enumerate() {
                reference.push_at(index as u64, Some(value)).unwrap();
            }
            let expected = reference.finish().unwrap();
            for maximum in [1, 2, 64] {
                let actual = select_median(maximum, |consume| {
                    for value in &values {
                        consume(value.clone())?;
                    }
                    Ok(())
                })
                .unwrap();
                assert_eq!(actual, expected, "{values:?} with capacity {maximum}");
            }
        }
    }

    #[tokio::test]
    async fn document_checkpoints_match_nullable_rows_at_every_cut() {
        use dokime_domain::model::{execution::RecordScope, identity::TestExecutionId};
        use harmos::Aggregate;
        let root = tempfile::tempdir().unwrap();
        let state = crate::storage::sqlite::State::open(root.path()).await.unwrap();
        let saved = state.data_open(None).await.unwrap();
        let execution = TestExecutionId::from_seed("typed-prefix-owner");
        let scope = RecordScope::default().with_test_execution(execution.clone());
        let raw = vec![
            Some(Value::Unsigned(9_007_199_254_740_993)),
            Some(Value::Null),
            Some(Value::Integer(-4)),
            Some(Value::Unsigned(7)),
            None,
            Some(Value::Integer(7)),
            Some(Value::Unsigned(9_007_199_254_740_995)),
            Some(Value::Null),
        ];
        let admitted = raw.clone();
        let owner = execution.to_string();
        state
            .call(move |connection| {
                Ok((|| -> Result<(), Error> {
                    let transaction = connection.transaction()?;
                    transaction.execute("INSERT INTO files VALUES ('fixture','telemetry','fixture.arrow',0,zeroblob(32),'fixture')", [])?;
                    let mut moments = Moments::default();
                    for (index, value) in admitted.into_iter().enumerate() {
                        let position = index as u64 + 1;
                        if let Some(value) = value {
                            moments.push_value(&value)?;
                            super::super::super::distinct::insert(
                                &transaction,
                                super::super::super::distinct::Row {
                                    kind: "telemetry",
                                    source: "fixture",
                                    position,
                                    file: Some("fixture"),
                                    record: false,
                                    scope: &scope,
                                },
                                [("value", &value)],
                            )?;
                        }
                        if position == 4 || position == 8 {
                            transaction.execute(
                                "INSERT INTO storage_numeric_checkpoints VALUES (?1,'fixture','value',?2,?3,?4)",
                                params![
                                    owner,
                                    super::super::super::position(position - 3),
                                    super::super::super::position(position),
                                    moments.encode()
                                ],
                            )?;
                        }
                    }
                    transaction.execute("UPDATE storage_data_head SET saved=?1", [super::super::super::position(8)])?;
                    transaction.commit()?;
                    Ok(())
                })())
            })
            .await
            .unwrap()
            .unwrap();
        let config =
            crate::config::query_defaults().compose(&crate::config::storage_defaults(), &crate::config::schema_defaults()).unwrap();
        for through in 0..=raw.len() {
            for aggregate in
                [Aggregate::Count, Aggregate::Min, Aggregate::Max, Aggregate::Mean, Aggregate::Sum, Aggregate::Rms, Aggregate::Median]
            {
                let expression = crate::expressions::telemetry("fixture", "value").aggregate(aggregate);
                let mut reference = crate::document::frame::Fold::new(&expression).unwrap();
                for (index, value) in raw[..through].iter().enumerate() {
                    reference.push_at(index as u64, value.as_ref()).unwrap();
                }
                let expected = reference.finish().unwrap();
                let cut = harmos::storage::StorageCursor {
                    history: saved.history.clone(),
                    sequence: through as u64,
                };
                let actual = state
                    .document_statistic(
                        execution.to_string(),
                        "fixture".into(),
                        "value".into(),
                        cut,
                        aggregate,
                        crate::storage::sqlite::QueryWork::document(config.clone(), "fixture"),
                    )
                    .await
                    .unwrap()
                    .expect("the maintained prefix covers the cut");
                if aggregate == Aggregate::Rms
                    && let (Some(Value::Number(actual)), Some(Value::Number(expected))) = (&actual, &expected)
                {
                    assert!((actual - expected).abs() <= expected.abs() * f64::EPSILON * 4.0);
                } else {
                    assert_eq!(actual, expected, "{aggregate:?} at {through}");
                }
            }
        }
        state.close().await.unwrap();
    }

    #[test]
    fn document_integer_index_matches_typed_folds() {
        use dokime_domain::model::values::Value;
        use harmos::Aggregate;
        let rows = [-7, 3, 11, 0, 0, 3, 99];
        let mut moments = Moments::default();
        for value in rows {
            moments.push_value(&Value::Integer(value)).unwrap();
        }
        for aggregate in [Aggregate::Count, Aggregate::Min, Aggregate::Max, Aggregate::Mean, Aggregate::Sum, Aggregate::Rms] {
            let mut reference =
                crate::document::frame::Fold::new(&crate::expressions::telemetry("fixture", "value").aggregate(aggregate)).unwrap();
            for (position, value) in rows.iter().enumerate() {
                reference.push_at(position as u64, Some(&Value::Integer(*value))).unwrap();
            }
            let actual = document_value(moments, aggregate).unwrap();
            let expected = reference.finish().unwrap();
            if aggregate == Aggregate::Rms {
                let (Some(Value::Number(actual)), Some(Value::Number(expected))) = (actual, expected) else { panic!("numeric RMS") };
                assert!((actual - expected).abs() <= expected.abs() * f64::EPSILON * 4.0);
            } else {
                assert_eq!(actual, expected);
            }
        }
    }

    #[test]
    fn document_integer_summary_preserves_large_values() {
        use harmos::Aggregate;
        let mut moments = Moments::default();
        for value in [9_007_199_254_740_993_u64, 9_007_199_254_740_995] {
            moments.push_value(&Value::Unsigned(value)).unwrap();
            moments = Moments::decode(&moments.encode()).unwrap();
        }
        for (aggregate, expected) in
            [(Aggregate::Min, 9_007_199_254_740_993), (Aggregate::Max, 9_007_199_254_740_995), (Aggregate::Mean, 9_007_199_254_740_994)]
        {
            assert_eq!(document_value(moments, aggregate).unwrap(), Some(Value::Unsigned(expected)));
        }
    }

    #[test]
    fn moments_match_harmos() {
        let mut indexed = Moments::default();
        let mut reference = harmos::signal::Summary::new();
        for value in [-7.0, 3.0, 11.0, -0.0, 0.0, 3.0, 99.0] {
            indexed.push(value).unwrap();
            indexed = Moments::decode(&indexed.encode()).unwrap();
            reference.push(value);
            assert_eq!(indexed.count, reference.count());
            assert_eq!(indexed.sum, reference.sum());
            assert_eq!(indexed.mean, reference.mean().unwrap());
            let std = (indexed.deviations / indexed.count as f64).max(0.0).sqrt();
            assert_eq!(std, reference.std().unwrap());
            assert_eq!(indexed.mean.hypot(std), reference.rms().unwrap());
        }
    }

    #[test]
    fn numeric_key_order() {
        let values = [f64::MIN, -100.0, -f64::MIN_POSITIVE, -0.0, 0.0, f64::MIN_POSITIVE, 100.0, f64::MAX];
        assert!(values.windows(2).all(|pair| ordered(pair[0]) < ordered(pair[1])));
    }
}
