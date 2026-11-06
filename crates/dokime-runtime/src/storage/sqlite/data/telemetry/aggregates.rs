/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Durable Moments And Exact Order Statistics Replayed From Evidence.

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

/* === Constants === */
const EXACT_INTEGER_RANGE: f64 = 9_007_199_254_740_992.0;

pub(in crate::storage::sqlite) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS storage_numeric_summaries (
    execution TEXT NOT NULL, stream TEXT NOT NULL, field TEXT NOT NULL, moments BLOB NOT NULL,
    PRIMARY KEY(execution,stream,field)
) WITHOUT ROWID;
";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
pub(in crate::storage::sqlite) struct Batch {
    fields: BTreeMap<String, BTreeMap<String, BTreeMap<String, Moments>>>,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct Moments {
    pub(crate) count: u64,
    sum: f64,
    mean: f64,
    lowest: f64,
    highest: f64,
    deviations: f64,
    first: f64,
    last: f64,
    max_abs: f64,
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
        &mut self, connection: &Connection, execution: &str, position: u64, time: i128, sample: &crate::telemetry::TelemetrySample,
        previews: &mut super::previews::Batch,
    ) -> Result<(), Error> {
        if !self.fields.contains_key(execution) {
            self.fields.insert(execution.to_owned(), BTreeMap::new());
        }
        let streams = self.fields.get_mut(execution).expect("loaded execution");
        if !streams.contains_key(sample.stream.as_str()) {
            streams.insert(sample.stream.to_string(), BTreeMap::new());
        }
        let fields = streams.get_mut(sample.stream.as_str()).expect("loaded stream");
        let input = super::previews::Input {
            execution,
            stream: sample.stream.as_str(),
            position,
            time,
        };
        for (field, typed) in &sample.values {
            let value = dokime_domain::model::values::numeric_value(typed).filter(|value| value.is_finite());
            if value.is_none() && !matches!(typed, dokime_domain::model::values::Value::Null) {
                continue;
            }
            previews.push(connection, input, field, value, typed)?;
            let Some(value) = value else { continue };
            if !fields.contains_key(field) {
                let existing: Option<Vec<u8>> = connection
                    .query_row(
                        "SELECT moments FROM storage_numeric_summaries WHERE execution=?1 AND stream=?2 AND field=?3",
                        params![execution, sample.stream.as_str(), field],
                        |row| row.get(0),
                    )
                    .optional()?;
                let moments = existing.as_deref().map(Moments::decode).transpose()?.unwrap_or_default();
                fields.insert(field.clone(), moments);
            }
            fields.get_mut(field).expect("loaded moments").push(value)?;
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", name = "ingest.numeric.save", skip_all)]
    pub(in crate::storage::sqlite) fn save(self, connection: &Connection) -> Result<(), Error> {
        let mut fields = connection.prepare_cached(
            "INSERT INTO storage_numeric_summaries VALUES (?1,?2,?3,?4)
             ON CONFLICT(execution,stream,field) DO UPDATE SET moments=excluded.moments",
        )?;
        for (execution, streams) in self.fields {
            for (stream, values) in streams {
                for (field, moments) in values {
                    fields.execute(params![execution, stream, field, moments.encode()])?;
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
            Aggregate::Rms => self.mean.hypot(std),
            _ => return None,
        })
    }

    pub(crate) fn push(&mut self, value: f64) -> Result<(), Error> {
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

    fn encode(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(72);
        bytes.extend_from_slice(&self.count.to_le_bytes());
        for value in [self.sum, self.mean, self.lowest, self.highest, self.deviations, self.first, self.last, self.max_abs] {
            bytes.extend_from_slice(&value.to_le_bytes());
        }
        bytes
    }

    pub(super) fn decode(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() != 72 {
            return Err(Error::Corrupt("numeric summary width differs"));
        }
        let (count, floats) = bytes.split_at(8);
        let count = u64::from_le_bytes(count.try_into().unwrap());
        let values = floats.as_chunks::<8>().0;
        let values: [f64; 8] = std::array::from_fn(|index| f64::from_le_bytes(values[index]));
        let [sum, mean, lowest, highest, deviations, first, last, max_abs] = values;
        if count == 0 {
            return Err(Error::Corrupt("numeric summary has no observations"));
        }
        Ok(Self {
            count,
            sum,
            mean,
            lowest,
            highest,
            deviations,
            first,
            last,
            max_abs,
        })
    }
}

impl crate::storage::sqlite::State {
    pub(crate) async fn document_statistic(
        &self, execution: String, stream: String, field: String, at: harmos::storage::StorageCursor, aggregate: harmos::Aggregate,
        integer: bool,
    ) -> Result<Option<dokime_domain::model::values::Value>, Error> {
        let root = self.0.readers.database.parent().ok_or(Error::Invalid("storage root is absent"))?.to_owned();
        self.read(move |connection| {
            Ok((|| {
                let head = super::super::saved(connection)?;
                if head.history != at.history || head.sequence < at.sequence {
                    return Ok(None);
                }
                let newer: bool = connection.query_row(
                    "SELECT EXISTS(SELECT 1 FROM streams WHERE execution=?1 AND stream=?2 AND last_position>?3)",
                    params![execution, stream, super::super::position(at.sequence)],
                    |row| row.get(0),
                )?;
                if newer {
                    return Ok(None);
                }
                let shared: bool = connection.query_row(
                    "SELECT EXISTS(SELECT 1 FROM streams s WHERE s.stream=?2 AND s.execution IS NOT ?1
                     AND EXISTS(SELECT 1 FROM stream_captures c WHERE c.execution=?1 AND c.file_id=s.file_id))",
                    params![execution, stream],
                    |row| row.get(0),
                )?;
                if shared {
                    return Ok(None);
                }
                let bytes: Option<Vec<u8>> = connection
                    .query_row(
                        "SELECT moments FROM storage_numeric_summaries WHERE execution=?1 AND stream=?2 AND field=?3",
                        params![execution, stream, field],
                        |row| row.get(0),
                    )
                    .optional()?;
                let Some(bytes) = bytes else {
                    return Ok(None);
                };
                let moments = Moments::decode(&bytes)?;
                let captured: i64 = connection.query_row(
                    "SELECT COALESCE(SUM(s.rows),0) FROM streams s WHERE s.execution=?1 AND s.stream=?2",
                    params![execution, stream],
                    |row| row.get(0),
                )?;
                if u64::try_from(captured).ok() != Some(moments.count) {
                    return Ok(None);
                }
                let median = if aggregate == harmos::Aggregate::Median && (!integer || moments.max_abs < EXACT_INTEGER_RANGE) {
                    let mut summaries = [NumericSummary {
                        stream,
                        field,
                        statistics: vec![("count".into(), moments.count as f64)],
                    }];
                    exact_percentiles(connection, &root, &execution, &at, &mut summaries, &Default::default())?;
                    summaries[0].statistics.iter().find(|(name, _)| name == "median").map(|(_, value)| *value)
                } else {
                    None
                };
                document_value(moments, aggregate, integer, median)
            })())
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

/* === Private Functions === */

fn document_value(
    moments: Moments, aggregate: harmos::Aggregate, integer: bool, median: Option<f64>,
) -> Result<Option<dokime_domain::model::values::Value>, Error> {
    use dokime_domain::model::values::Value;
    use harmos::Aggregate;
    if aggregate == Aggregate::Count {
        return Ok(Some(Value::Unsigned(moments.count)));
    }
    // Outside The Exactly Representable Range, Typed Queries Own Integer Statistics.
    if integer {
        let safe = match aggregate {
            Aggregate::Min | Aggregate::Max | Aggregate::Median => moments.max_abs < EXACT_INTEGER_RANGE,
            Aggregate::Mean | Aggregate::Sum => moments.max_abs * (moments.count as f64) < EXACT_INTEGER_RANGE,
            Aggregate::Rms | Aggregate::Std => true,
            _ => false,
        };
        if !safe {
            return Ok(None);
        }
    }
    let value = match aggregate {
        Aggregate::Median => {
            let Some(value) = median else { return Ok(None) };
            value
        }
        Aggregate::Mean if integer => moments.sum / moments.count as f64,
        aggregate => {
            let Some(value) = moments.value(aggregate) else { return Ok(None) };
            value
        }
    };
    if !value.is_finite() {
        return Ok(None);
    }
    Ok(Some(if integer && !matches!(aggregate, Aggregate::Rms | Aggregate::Std) && value.fract() == 0.0 {
        Value::Integer(value as i64)
    } else {
        Value::Number(value)
    }))
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
                Ok(unordered(key))
            })?;
        }
    }
    Ok(())
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

fn unordered(value: u64) -> f64 {
    f64::from_bits(if value >> 63 != 0 { value ^ (1 << 63) } else { !value })
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_integer_index_matches_typed_folds() {
        use dokime_domain::model::values::Value;
        use harmos::Aggregate;
        let rows = [-7, 3, 11, 0, 0, 3, 99];
        let mut moments = Moments::default();
        for value in rows {
            moments.push(value as f64).unwrap();
        }
        let mut ordered = rows;
        ordered.sort_unstable();
        let median = ordered[ordered.len() / 2] as f64;
        for aggregate in
            [Aggregate::Count, Aggregate::Min, Aggregate::Max, Aggregate::Mean, Aggregate::Sum, Aggregate::Rms, Aggregate::Median]
        {
            let mut reference =
                crate::document::frame::Fold::new(&crate::expressions::telemetry("fixture", "value").aggregate(aggregate)).unwrap();
            for (position, value) in rows.iter().enumerate() {
                reference.push_at(position as u64, Some(&Value::Integer(*value))).unwrap();
            }
            assert_eq!(document_value(moments, aggregate, true, Some(median)).unwrap(), reference.finish().unwrap());
        }
    }

    #[test]
    fn document_integer_index_preserves_lossless_fallback() {
        use harmos::Aggregate;
        let mut moments = Moments::default();
        for value in [9_007_199_254_740_993_u64, 9_007_199_254_740_995] {
            moments.push(value as f64).unwrap();
        }
        for aggregate in [Aggregate::Min, Aggregate::Max, Aggregate::Mean, Aggregate::Median] {
            assert!(document_value(moments, aggregate, true, None).unwrap().is_none());
        }
    }

    #[test]
    fn replay_order_preserves_float_bits() {
        let connection = Connection::open_in_memory().unwrap();
        connection.execute("CREATE TABLE values_replay (value BLOB NOT NULL)", []).unwrap();
        let mut values = [7.0, -0.0, 0.0, -1.25, f64::MIN, f64::MAX, 7.0];
        for value in values {
            connection.execute("INSERT INTO values_replay VALUES (?1)", [ordered(value).to_be_bytes().as_slice()]).unwrap();
        }
        values.sort_by(f64::total_cmp);
        let mut query = connection.prepare("SELECT value FROM values_replay ORDER BY value").unwrap();
        let actual: Vec<_> = query
            .query_map([], |row| row.get::<_, Vec<u8>>(0))
            .unwrap()
            .map(|row| unordered(u64::from_be_bytes(row.unwrap().try_into().unwrap())).to_bits())
            .collect();
        assert_eq!(actual, values.map(f64::to_bits));
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
    fn percentile_interpolation() {
        assert_eq!(interpolate(f64::MIN, f64::MAX, 0.5), 0.0);
        assert_eq!(interpolate(2.0, 10.0, 0.25), 4.0);
        assert_eq!(interpolate(-10.0, -2.0, 0.75), -4.0);
    }
}
