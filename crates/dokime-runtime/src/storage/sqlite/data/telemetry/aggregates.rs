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
use harmos::storage::StoredRow;
use rusqlite::{Connection, OptionalExtension, params};

// Local Imports
use super::super::blob;
use crate::storage::Error;

/* === Constants === */
pub(in crate::storage::sqlite) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS storage_numeric_summaries (
    execution TEXT NOT NULL, stream TEXT NOT NULL, field TEXT NOT NULL, moments BLOB NOT NULL,
    PRIMARY KEY(execution,stream,field)
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS storage_numeric_ranks (
    execution TEXT NOT NULL, stream TEXT NOT NULL, field TEXT NOT NULL,
    parent BLOB NOT NULL, child INTEGER NOT NULL, count INTEGER NOT NULL CHECK(count>0),
    PRIMARY KEY(execution,stream,field,parent,child)
) WITHOUT ROWID;
";

/* === Macros === */

/* === Type Aliases === */
type Key = (String, String, String);

/* === Structs === */

#[derive(Default)]
pub(in crate::storage::sqlite) struct Batch {
    fields: BTreeMap<Key, Moments>,
    ranks: BTreeMap<(Key, Vec<u8>, u8), u64>,
}

#[derive(Clone, Copy, Default)]
struct Moments {
    count: u64,
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
    pub(in crate::storage::sqlite) fn push(
        &mut self, connection: &Connection, row: &StoredRow, previews: &mut super::previews::Batch,
    ) -> Result<(), Error> {
        let Some(execution) = row.record.scope.as_ref() else {
            return Ok(());
        };
        if !row.record.schema.id.as_str().starts_with("dokime.telemetry.") {
            return Ok(());
        }
        for (field, value) in crate::storage::telemetry::numeric_metadata(&row.record.payload)? {
            previews.push(connection, row, &field, value)?;
            let key = (execution.to_string(), row.stream.to_string(), field);
            if let std::collections::btree_map::Entry::Vacant(entry) = self.fields.entry(key.clone()) {
                let existing: Option<Vec<u8>> = connection
                    .query_row(
                        "SELECT moments FROM storage_numeric_summaries WHERE execution=?1 AND stream=?2 AND field=?3",
                        params![key.0, key.1, key.2],
                        |row| row.get(0),
                    )
                    .optional()?;
                let moments = existing.as_deref().map(Moments::decode).transpose()?.unwrap_or_default();
                entry.insert(moments);
            }
            self.fields.get_mut(&key).expect("loaded moments").push(value)?;
            let ordered = ordered(value).to_be_bytes();
            for depth in 0..8 {
                let count = self.ranks.entry((key.clone(), ordered[..depth].to_vec(), ordered[depth])).or_default();
                *count = count.checked_add(1).ok_or(Error::Data("numeric rank count overflow"))?;
            }
        }
        Ok(())
    }

    pub(in crate::storage::sqlite) fn save(self, connection: &Connection) -> Result<(), Error> {
        let mut fields = connection.prepare_cached(
            "INSERT INTO storage_numeric_summaries VALUES (?1,?2,?3,?4)
             ON CONFLICT(execution,stream,field) DO UPDATE SET moments=excluded.moments",
        )?;
        for (key, moments) in self.fields {
            fields.execute(params![key.0, key.1, key.2, moments.encode()])?;
        }
        let mut ranks = connection.prepare_cached(
            "INSERT INTO storage_numeric_ranks VALUES (?1,?2,?3,?4,?5,?6)
             ON CONFLICT(execution,stream,field,parent,child) DO UPDATE SET count=count+excluded.count",
        )?;
        for ((key, parent, child), count) in self.ranks {
            let count = i64::try_from(count).map_err(|_| Error::Data("numeric rank count exceeds SQLite range"))?;
            ranks.execute(params![key.0, key.1, key.2, parent, child, count])?;
        }
        Ok(())
    }
}

impl Moments {
    fn push(&mut self, value: f64) -> Result<(), Error> {
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

    fn decode(bytes: &[u8]) -> Result<Self, Error> {
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
    pub(crate) async fn numeric_summaries(&self, execution: String) -> Result<Vec<NumericSummary>, Error> {
        self.read(move |connection| Ok(numeric_summaries(connection, execution))).await?
    }
}

/* === Public Functions === */

pub(super) fn numeric_percentile(
    connection: &Connection, execution: String, stream: String, field: String, rank: u8,
) -> Result<Option<f64>, Error> {
    if rank > 100 {
        return Err(Error::Data("invalid percentile rank"));
    }

    let moments: Option<Vec<u8>> = connection
        .query_row(
            "SELECT moments FROM storage_numeric_summaries WHERE execution=?1 AND stream=?2 AND field=?3",
            params![execution, stream, field],
            |row| row.get(0),
        )
        .optional()?;
    moments.map(|moments| percentile(connection, &(execution, stream, field), Moments::decode(&moments)?.count, rank)).transpose()
}

pub(super) fn numeric_summaries(connection: &Connection, execution: String) -> Result<Vec<NumericSummary>, Error> {
    let mut query =
        connection.prepare("SELECT stream,field,moments FROM storage_numeric_summaries WHERE execution=?1 ORDER BY stream,field")?;
    let mut rows = query.query([&execution])?;
    let mut summaries = Vec::new();
    while let Some(row) = rows.next()? {
        let stream: String = row.get(0)?;
        let field: String = row.get(1)?;
        let moments = Moments::decode(blob(row, 2)?)?;
        let std = (moments.deviations / moments.count as f64).max(0.0).sqrt();
        let key = (execution.clone(), stream.clone(), field.clone());
        let statistics: Vec<_> = vec![
            ("count", moments.count as f64),
            ("first", moments.first),
            ("last", moments.last),
            ("min", moments.lowest),
            ("max", moments.highest),
            ("mean", moments.mean),
            ("median", percentile(connection, &key, moments.count, 50)?),
            ("std", std),
            ("rms", moments.mean.hypot(std)),
            ("max_abs", moments.max_abs),
            ("sum", moments.sum),
            ("p95", percentile(connection, &key, moments.count, 95)?),
            ("p99", percentile(connection, &key, moments.count, 99)?),
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

fn ordered(value: f64) -> u64 {
    let bits = value.to_bits();
    if bits >> 63 != 0 { !bits } else { bits ^ (1 << 63) }
}

fn rank(connection: &Connection, key: &Key, mut ordinal: u64) -> Result<f64, Error> {
    let mut prefix = Vec::with_capacity(8);
    let mut query = connection.prepare_cached(
        "SELECT child,count FROM storage_numeric_ranks WHERE execution=?1 AND stream=?2 AND field=?3 AND parent=?4 ORDER BY child",
    )?;
    for _ in 0..8 {
        let mut rows = query.query(params![key.0, key.1, key.2, prefix])?;
        let mut selected = None;
        while let Some(row) = rows.next()? {
            let child: u8 = row.get(0)?;
            let count = u64::try_from(row.get::<_, i64>(1)?).map_err(|_| Error::Corrupt("numeric rank count is negative"))?;
            if ordinal < count {
                selected = Some(child);
                break;
            }
            ordinal -= count;
        }
        prefix.push(selected.ok_or(Error::Corrupt("numeric rank is absent"))?);
    }
    let ordered = u64::from_be_bytes(prefix.try_into().unwrap());
    let bits = if ordered >> 63 != 0 { ordered ^ (1 << 63) } else { !ordered };
    Ok(f64::from_bits(bits))
}

fn percentile(connection: &Connection, key: &Key, count: u64, percentile: u8) -> Result<f64, Error> {
    let position = f64::from(percentile) / 100.0 * (count - 1) as f64;
    let low = rank(connection, key, position.floor() as u64)?;
    let fraction = position - position.floor();
    if fraction == 0.0 {
        return Ok(low);
    }
    let high = rank(connection, key, position.ceil() as u64)?;
    Ok(if low == high {
        low
    } else if low.signum() == high.signum() {
        low + (high - low) * fraction
    } else {
        low * (1.0 - fraction) + high * fraction
    })
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

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
