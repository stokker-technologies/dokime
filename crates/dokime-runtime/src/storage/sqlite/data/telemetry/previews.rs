/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Durable Adaptive Time Envelopes; Raw Samples Remain In Their Original Store.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

// Local Imports
use crate::storage::Error;

/* === Constants === */

pub(in crate::storage::sqlite) const SCHEMA: &str = "CREATE TABLE IF NOT EXISTS storage_report_previews (
    execution TEXT NOT NULL, stream TEXT NOT NULL, field TEXT NOT NULL, through TEXT NOT NULL, envelope BLOB NOT NULL,
    PRIMARY KEY(execution,stream,field)
) WITHOUT ROWID;";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct Point {
    pub(crate) time: i128,
    pub(crate) position: u64,
    pub(crate) value: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Bucket {
    pub(crate) first: Point,
    pub(crate) last: Point,
    pub(crate) min: Point,
    pub(crate) max: Point,
    pub(crate) count: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Envelope {
    pub(crate) origin: i128,
    pub(crate) width: i128,
    pub(crate) buckets: BTreeMap<i128, Bucket>,
}

pub(crate) struct FieldPreview {
    pub(crate) stream: String,
    pub(crate) field: String,
    pub(crate) envelope: Envelope,
}

pub(in crate::storage::sqlite) struct Batch {
    settings: crate::storage::sqlite::PreviewSettings,
    fields: BTreeMap<(String, String, String), (u64, Envelope)>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Bucket {
    fn new(point: Point) -> Self {
        Self {
            first: point,
            last: point,
            min: point,
            max: point,
            count: 1,
        }
    }

    fn merge(&mut self, other: Self) -> Result<(), Error> {
        let order = |point: Point| (point.time, point.position);
        if order(other.first) < order(self.first) {
            self.first = other.first;
        }
        if order(other.last) > order(self.last) {
            self.last = other.last;
        }
        if other.min.value < self.min.value {
            self.min = other.min;
        }
        if other.max.value > self.max.value {
            self.max = other.max;
        }
        self.count = self.count.checked_add(other.count).ok_or(Error::Data("preview count overflow"))?;
        Ok(())
    }

    pub(crate) fn points(&self) -> Vec<Point> {
        let mut points = vec![self.first, self.min, self.max, self.last];
        points.sort_by_key(|point| (point.time, point.position));
        points.dedup_by_key(|point| point.position);
        points
    }
}

impl Envelope {
    pub(crate) fn fit(&mut self, maximum: usize) -> Result<(), Error> {
        while self.buckets.len() > maximum {
            self.coarsen()?;
        }
        Ok(())
    }

    fn coarsen(&mut self) -> Result<(), Error> {
        let first = self.buckets.first_key_value().map(|(key, _)| *key).unwrap_or_default();
        self.origin = self
            .origin
            .checked_add(first.checked_mul(self.width).ok_or(Error::Data("preview origin overflow"))?)
            .ok_or(Error::Data("preview origin overflow"))?;
        self.width = self.width.checked_mul(2).ok_or(Error::Data("preview time range overflow"))?;
        for (key, bucket) in std::mem::take(&mut self.buckets) {
            match self.buckets.entry((key - first).div_euclid(2)) {
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(bucket);
                }
                std::collections::btree_map::Entry::Occupied(mut entry) => entry.get_mut().merge(bucket)?,
            }
        }
        Ok(())
    }

    pub(crate) fn push(&mut self, point: Point, maximum: usize) -> Result<(), Error> {
        if !point.value.is_finite() {
            return Err(Error::Data("preview value is not finite"));
        }
        let key = (point.time - self.origin).div_euclid(self.width);
        match self.buckets.entry(key) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(Bucket::new(point));
            }
            std::collections::btree_map::Entry::Occupied(mut entry) => entry.get_mut().merge(Bucket::new(point))?,
        }
        self.fit(maximum)
    }
}

impl Batch {
    pub(in crate::storage::sqlite) fn new(settings: crate::storage::sqlite::PreviewSettings) -> Self {
        Self { settings, fields: BTreeMap::new() }
    }

    pub(in crate::storage::sqlite) fn push(
        &mut self, connection: &Connection, row: &harmos::storage::StoredRow, field: &str, value: f64,
    ) -> Result<(), Error> {
        let Some(execution) = row.record.scope.as_ref() else { return Ok(()) };
        let time = row.record.source_time.ok_or(Error::Data("preview sample has no time"))?;
        let nanos = match time.duration_since(std::time::UNIX_EPOCH) {
            Ok(value) => i128::try_from(value.as_nanos()).map_err(|_| Error::Data("preview time overflow"))?,
            Err(error) => -i128::try_from(error.duration().as_nanos()).map_err(|_| Error::Data("preview time overflow"))?,
        };
        let time = nanos;
        let key = (execution.to_string(), row.stream.to_string(), field.to_owned());
        if let std::collections::btree_map::Entry::Vacant(entry) = self.fields.entry(key.clone()) {
            let existing: Option<Vec<u8>> = connection
                .query_row(
                    "SELECT envelope FROM storage_report_previews WHERE execution=?1 AND stream=?2 AND field=?3",
                    params![key.0, key.1, key.2],
                    |row| row.get(0),
                )
                .optional()?;
            let envelope = match existing {
                Some(bytes) => serde_json::from_slice(&bytes)?,
                None => Envelope {
                    origin: 0,
                    width: i128::try_from(self.settings.bucket_interval.as_nanos())
                        .map_err(|_| Error::Data("preview interval overflow"))?,
                    buckets: BTreeMap::new(),
                },
            };
            entry.insert((0, envelope));
        }
        let (through, envelope) = self.fields.get_mut(&key).expect("loaded preview");
        *through = row.record.position.sequence;
        envelope.push(Point { time, position: *through, value }, self.settings.bucket_count.get())
    }

    pub(in crate::storage::sqlite) fn save(self, connection: &Connection) -> Result<(), Error> {
        let mut query = connection.prepare_cached(
            "INSERT INTO storage_report_previews VALUES (?1,?2,?3,?4,?5)
            ON CONFLICT(execution,stream,field) DO UPDATE SET through=excluded.through,envelope=excluded.envelope",
        )?;
        for ((execution, stream, field), (through, envelope)) in self.fields {
            query.execute(params![execution, stream, field, super::super::position(through), serde_json::to_vec(&envelope)?])?;
        }
        Ok(())
    }
}

impl crate::storage::sqlite::State {
    pub(crate) async fn report_sample_rows(
        &self, execution: String, stream: String, at: harmos::storage::StorageCursor, limit: usize,
    ) -> Result<Vec<harmos::storage::StoredRow>, Error> {
        let Some(rows) = std::num::NonZeroUsize::new(limit) else { return Ok(Vec::new()) };
        let root = self.0.readers.database.parent().ok_or(Error::Invalid("storage root is absent"))?.to_owned();
        self.read(move |connection| {
            Ok((|| {
                let head = super::super::saved(connection)?;
                head.history.check(&at.history)?;
                let scan = |after: u64| {
                    super::super::Scan::new(
                        "stream",
                        at.sequence,
                        0,
                        harmos::storage::ScanPage {
                            after,
                            rows,
                            bytes: std::num::NonZeroUsize::MAX,
                        },
                    )
                    .filter("scope", Some(execution.clone()))
                    .filter("stream", Some(stream.clone()))
                };
                let mut selected = BTreeMap::new();
                for row in super::reader::stored(connection, &root, &scan(0))? {
                    selected.insert(row.record.position.sequence, row);
                }
                // The Batch Index Locates The Final Rows Without Reading Earlier Flushed Batches.
                let mut batches = connection.prepare(
                    "SELECT b.rows,b.first_position FROM stream_batches b JOIN streams s ON s.file_id=b.file_id
                     WHERE s.execution=?1 AND s.stream=?2 AND b.first_position<=?3 ORDER BY b.last_position DESC",
                )?;
                let mut candidates = batches.query(params![execution, stream, super::super::position(at.sequence)])?;
                let (mut counted, mut first) = (0_u64, None::<String>);
                while let Some(row) = candidates.next()? {
                    counted = counted.saturating_add(u64::try_from(row.get::<_, i64>(0)?).unwrap_or_default());
                    first = Some(row.get(1)?);
                    if counted >= limit as u64 {
                        break;
                    }
                }
                let Some(first) = first else { return Ok(selected.into_values().collect()) };
                let mut after = super::super::sequence(&first)?.saturating_sub(1);
                let mut last = std::collections::VecDeque::with_capacity(limit);
                loop {
                    let page = super::reader::stored(connection, &root, &scan(after))?;
                    let Some(end) = page.last().map(|row| row.record.position.sequence) else { break };
                    for row in page {
                        if last.len() == limit {
                            last.pop_front();
                        }
                        last.push_back(row);
                    }
                    after = end;
                }
                for row in last {
                    selected.insert(row.record.position.sequence, row);
                }
                Ok(selected.into_values().collect())
            })())
        })
        .await?
    }

    pub(crate) async fn report_record_rows(
        &self, execution: String, cycle: String, at: harmos::storage::StorageCursor, limit: usize,
    ) -> Result<Vec<dokime_domain::model::records::evidence::RecordEntry>, Error> {
        self.read(move |connection| {
            Ok((|| {
                let head = super::super::saved(connection)?;
                head.history.check(&at.history)?;
                let mut selected = BTreeMap::new();
                for order in ["ASC", "DESC"] {
                    // Each Branch Seeks Its Own Owner Index Before The Bounded Union.
                    for selection in [
                        "SELECT position FROM storage_record_visibility WHERE execution=?1",
                        "SELECT position FROM storage_record_visibility WHERE cycle=?2 AND execution IS NULL",
                        "SELECT position FROM storage_record_visibility WHERE cycle IS NULL AND execution IS NULL",
                        "SELECT position FROM storage_record_captures WHERE execution=?1",
                    ] {
                        let sql = format!("SELECT e.body,e.digest FROM ({selection} AND position<=?3 ORDER BY position {order} LIMIT ?4) i JOIN storage_data_entries e ON e.position=i.position");
                        let mut query = connection.prepare(&sql)?;
                        let mut rows = query.query(params![execution, cycle, super::super::position(at.sequence), limit as i64])?;
                        while let Some(row) = rows.next()? {
                            let body = super::super::blob(row, 0)?;
                            super::super::check_digest(body, super::super::blob(row, 1)?)?;
                            let super::super::Entry::Record(row) = super::super::Entry::decode("record", body)? else { unreachable!() };
                            let mut record = crate::storage::records::RecordRow::decode_entry(&row.payload)?;
                            if let Some(scope) = record.captured_scopes.iter().find(|scope| scope.test_execution_id.as_ref().is_some_and(|id| id.as_str()==execution)) {
                                record.scope = Some(scope.clone());
                            }
                            selected.insert(row.position.sequence, record);
                        }
                    }
                }
                let mut rows: Vec<_> = selected.into_values().collect();
                if rows.len() > limit * 2 { rows.drain(limit..rows.len()-limit); }
                Ok(rows)
            })())
        }).await?
    }

    pub(crate) async fn report_previews(&self, execution: String, at: harmos::storage::StorageCursor) -> Result<Vec<FieldPreview>, Error> {
        self.read(move |connection| {
            Ok((|| {
                let head = super::super::saved(connection)?;
                head.history.check(&at.history)?;
                let mut query = connection.prepare(
                    "SELECT stream,field,envelope,through FROM storage_report_previews WHERE execution=?1 ORDER BY stream,field",
                )?;
                let mut rows = query.query([execution])?;
                let mut fields = Vec::new();
                while let Some(row) = rows.next()? {
                    let through: String = row.get(3)?;
                    if through > super::super::position(at.sequence) {
                        return Err(Error::Data("preview has advanced beyond requested evidence; request a current snapshot"));
                    }
                    fields.push(FieldPreview {
                        stream: row.get(0)?,
                        field: row.get(1)?,
                        envelope: serde_json::from_slice(super::super::blob(row, 2)?)?,
                    });
                }
                Ok(fields)
            })())
        })
        .await?
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bucket_extrema() {
        for offset in [0, i128::from(u64::MAX) + 4096] {
            for maximum in [1, 8, 32] {
                let mut envelope = Envelope {
                    origin: 0,
                    width: 1,
                    buckets: BTreeMap::new(),
                };
                let mut fixture = Vec::new();
                for position in 0..4096_u64 {
                    // Out-Of-Order Times, Duplicate Times, Positive/Negative Spikes And Epoch Crossing.
                    let time = offset + ((position * 37) % 4096) as i128 - 2048;
                    let value = match position % 127 {
                        0 => 1_000_000.0,
                        1 => -1_000_000.0,
                        _ => (position % 97) as f64,
                    };
                    let point = Point { time, position, value };
                    fixture.push(point);
                    envelope.push(point, maximum).unwrap();
                    assert!(envelope.buckets.len() <= maximum);
                    if position % 64 == 0 {
                        envelope = serde_json::from_slice(&serde_json::to_vec(&envelope).unwrap()).unwrap();
                    }
                }
                assert_eq!(envelope.buckets.values().map(|bucket| bucket.count).sum::<u64>(), fixture.len() as u64);
                for (key, bucket) in &envelope.buckets {
                    let members: Vec<_> =
                        fixture.iter().filter(|point| (point.time - envelope.origin).div_euclid(envelope.width) == *key).collect();
                    assert_eq!(bucket.count as usize, members.len());
                    assert_eq!(bucket.min.value, members.iter().map(|point| point.value).fold(f64::INFINITY, f64::min));
                    assert_eq!(bucket.max.value, members.iter().map(|point| point.value).fold(f64::NEG_INFINITY, f64::max));
                    assert_eq!(bucket.first, **members.iter().min_by_key(|point| (point.time, point.position)).unwrap());
                    assert_eq!(bucket.last, **members.iter().max_by_key(|point| (point.time, point.position)).unwrap());
                    let points = bucket.points();
                    assert!(points.contains(&bucket.min));
                    assert!(points.contains(&bucket.max));
                    assert!(points.len() <= 4);
                }
            }
        }
    }
}
