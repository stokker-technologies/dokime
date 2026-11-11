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
use std::io::Write;

// Third Party Imports
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

// Local Imports
use crate::storage::Error;

/* === Constants === */

pub(in crate::storage::sqlite) const SCHEMA: &str = "CREATE TABLE IF NOT EXISTS storage_report_previews (
    execution TEXT NOT NULL, stream TEXT NOT NULL, field TEXT NOT NULL, through TEXT NOT NULL, envelope BLOB NOT NULL,
    PRIMARY KEY(execution,stream,field)
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS storage_report_context (
    execution TEXT NOT NULL,stream TEXT NOT NULL,through TEXT NOT NULL,state BLOB NOT NULL,
    PRIMARY KEY(execution,stream)
) WITHOUT ROWID;
CREATE TABLE IF NOT EXISTS storage_plot_buckets (
    execution TEXT NOT NULL, stream TEXT NOT NULL, field TEXT NOT NULL,
    width INTEGER NOT NULL, bucket INTEGER NOT NULL, through TEXT NOT NULL, state BLOB NOT NULL,
    PRIMARY KEY(execution,stream,field,width,bucket)
) WITHOUT ROWID;";

/* === Macros === */

/* === Type Aliases === */

type FieldProjections = BTreeMap<String, BTreeMap<String, BTreeMap<String, (u64, Projection)>>>;
type StreamContexts = BTreeMap<(String, String), (u64, BTreeMap<String, dokime_domain::model::values::Value>)>;

type WindowBuckets = BTreeMap<(String, String, String, i64, i64), (u64, Bucket)>;

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
    pub(crate) scale: f64,
    pub(crate) sum: f64,
    pub(crate) squares: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Envelope {
    pub(crate) origin: i128,
    pub(crate) width: i128,
    pub(crate) buckets: BTreeMap<i128, Bucket>,
}

#[derive(Clone)]
pub(crate) struct FieldPreview {
    pub(crate) stream: String,
    pub(crate) field: String,
    pub(crate) envelope: Envelope,
    pub(crate) exact: Vec<(Point, dokime_domain::model::values::Value)>,
    pub(crate) total: u64,
    pub(crate) last: Option<Point>,
}

#[derive(Clone, Copy)]
pub(in crate::storage::sqlite) struct Input<'a> {
    pub execution: &'a str,
    pub stream: &'a str,
    pub position: u64,
    pub time: i128,
}

#[derive(Serialize, Deserialize)]
struct Projection {
    envelope: Envelope,
    exact: Vec<(Point, NumericValue)>,
    count: u64,
    last: Option<Point>,
}

pub(in crate::storage::sqlite) struct Batch {
    settings: crate::storage::sqlite::PreviewSettings,
    fields: FieldProjections,
    contexts: StreamContexts,
    windows: WindowBuckets,
}

/* === Enums === */

#[derive(Serialize, Deserialize)]
enum NumericValue {
    Null,
    Integer(i64),
    Unsigned(u64),
    Number(f64),
}

/* === Traits === */

/* === Implementations === */

impl NumericValue {
    fn new(value: &dokime_domain::model::values::Value) -> Result<Self, Error> {
        use dokime_domain::model::values::Value;
        Ok(match value {
            Value::Null => Self::Null,
            Value::Integer(value) => Self::Integer(*value),
            Value::Unsigned(value) => Self::Unsigned(*value),
            Value::Number(value) => Self::Number(*value),
            _ => return Err(Error::Invalid("numeric preview contains a nonnumeric value")),
        })
    }

    fn value(&self) -> dokime_domain::model::values::Value {
        use dokime_domain::model::values::Value;
        match self {
            Self::Null => Value::Null,
            Self::Integer(value) => Value::Integer(*value),
            Self::Unsigned(value) => Value::Unsigned(*value),
            Self::Number(value) => Value::Number(*value),
        }
    }
}

impl Projection {
    fn encode(&self) -> Result<Vec<u8>, Error> {
        postcard::to_allocvec(self).map_err(|_| Error::Data("numeric preview encoding failed"))
    }

    fn decode(bytes: &[u8]) -> Result<Self, Error> {
        postcard::from_bytes(bytes).map_err(|_| {
            Error::Incompatible("Numeric preview layout is incompatible; preserve existing evidence and use a fresh storage root".into())
        })
    }
}

impl Bucket {
    pub(crate) fn new(point: Point) -> Self {
        Self {
            first: point,
            last: point,
            min: point,
            max: point,
            count: 1,
            scale: point.value.abs(),
            sum: if point.value == 0.0 { 0.0 } else { point.value.signum() },
            squares: if point.value == 0.0 { 0.0 } else { 1.0 },
        }
    }

    pub(crate) fn merge(&mut self, other: Self) -> Result<(), Error> {
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
        let scale = self.scale.max(other.scale);
        if scale > 0.0 {
            let left = self.scale / scale;
            let right = other.scale / scale;
            self.sum = self.sum * left + other.sum * right;
            self.squares = self.squares * left * left + other.squares * right * right;
        }
        self.scale = scale;
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
        Self {
            settings,
            fields: BTreeMap::new(),
            contexts: BTreeMap::new(),
            windows: BTreeMap::new(),
        }
    }

    pub(in crate::storage::sqlite) fn push(
        &mut self, connection: &Connection, input: Input<'_>, field: &str, value: Option<f64>, typed: &dokime_domain::model::values::Value,
    ) -> Result<(), Error> {
        let Input { execution, stream, position, time } = input;
        if !self.fields.contains_key(execution) {
            self.fields.insert(execution.to_owned(), BTreeMap::new());
        }
        let streams = self.fields.get_mut(execution).expect("loaded execution");
        if !streams.contains_key(stream) {
            streams.insert(stream.to_owned(), BTreeMap::new());
        }
        let fields = streams.get_mut(stream).expect("loaded stream");
        if !fields.contains_key(field) {
            let existing: Option<Vec<u8>> = connection
                .query_row(
                    "SELECT envelope FROM storage_report_previews WHERE execution=?1 AND stream=?2 AND field=?3",
                    params![execution, stream, field],
                    |row| row.get(0),
                )
                .optional()?;
            let projection = match existing {
                Some(bytes) => Projection::decode(&bytes)?,
                None => Projection {
                    envelope: Envelope {
                        origin: 0,
                        width: i128::try_from(self.settings.bucket_interval.as_nanos())
                            .map_err(|_| Error::Data("preview interval overflow"))?,
                        buckets: BTreeMap::new(),
                    },
                    exact: Vec::new(),
                    count: 0,
                    last: None,
                },
            };
            fields.insert(field.to_owned(), (0, projection));
        }
        let (through, projection) = fields.get_mut(field).expect("loaded preview");
        *through = position;
        let point = Point {
            time,
            position,
            value: value.unwrap_or_default(),
        };
        projection.last = Some(point);
        let before = projection.count;
        projection.count = before.checked_add(1).ok_or(Error::Data("preview count overflow"))?;
        if before < self.settings.exact_points.get() as u64 && projection.exact.len() as u64 == before {
            projection.exact.push((point, NumericValue::new(typed)?));
        } else {
            projection.exact.clear();
        }
        if value.is_some() {
            let width = i64::try_from(self.settings.bucket_interval.as_nanos()).map_err(|_| Error::Data("preview interval overflow"))?;
            let index = i64::try_from(time.div_euclid(i128::from(width))).map_err(|_| Error::Data("preview bucket overflow"))?;
            let key = (execution.to_owned(), stream.to_owned(), field.to_owned(), width, index);
            if let std::collections::btree_map::Entry::Vacant(entry) = self.windows.entry(key.clone()) {
                let previous: Option<Vec<u8>> = connection
                    .query_row(
                        "SELECT state FROM storage_plot_buckets WHERE execution=?1 AND stream=?2 AND field=?3 AND width=?4 AND bucket=?5",
                        params![execution, stream, field, width, index],
                        |row| row.get(0),
                    )
                    .optional()?;
                let bucket = previous.as_deref().map(decode).transpose()?;
                entry.insert((position, bucket.unwrap_or_else(|| Bucket::new(point))));
                if previous.is_none() {
                    projection.envelope.push(point, self.settings.bucket_count.get())?;
                    return Ok(());
                }
            }
            let (through, bucket) = self.windows.get_mut(&key).expect("loaded time bucket");
            *through = position;
            bucket.merge(Bucket::new(point))?;
            projection.envelope.push(point, self.settings.bucket_count.get())?;
        }
        Ok(())
    }

    pub(in crate::storage::sqlite) fn context<'a>(
        &mut self, connection: &Connection, execution: &str, position: u64, stream: &str,
        values: impl IntoIterator<Item = (&'a str, &'a dokime_domain::model::values::Value)>,
    ) -> Result<(), Error> {
        let key = (execution.to_owned(), stream.to_owned());
        if let std::collections::btree_map::Entry::Vacant(entry) = self.contexts.entry(key.clone()) {
            let bytes: Option<Vec<u8>> = connection
                .query_row("SELECT state FROM storage_report_context WHERE execution=?1 AND stream=?2", params![key.0, key.1], |row| {
                    row.get(0)
                })
                .optional()?;
            let previous = bytes.as_deref().map(decode).transpose()?.unwrap_or_default();
            entry.insert((0, previous));
        }
        let (through, context) = self.contexts.get_mut(&key).expect("loaded stream context");
        *through = position;
        for (field, value) in values {
            context.insert(field.to_owned(), value.clone());
        }
        Ok(())
    }

    pub(in crate::storage::sqlite) fn save(self, connection: &Connection) -> Result<(), Error> {
        let mut query = connection.prepare_cached(
            "INSERT INTO storage_report_previews VALUES (?1,?2,?3,?4,?5)
            ON CONFLICT(execution,stream,field) DO UPDATE SET through=excluded.through,envelope=excluded.envelope",
        )?;
        for (execution, streams) in self.fields {
            for (stream, fields) in streams {
                for (field, (through, projection)) in fields {
                    query.execute(params![execution, stream, field, super::super::position(through), projection.encode()?])?;
                }
            }
        }
        let mut buckets = connection.prepare_cached(
            "INSERT INTO storage_plot_buckets VALUES (?1,?2,?3,?4,?5,?6,?7)
             ON CONFLICT(execution,stream,field,width,bucket) DO UPDATE SET through=excluded.through,state=excluded.state",
        )?;
        for ((execution, stream, field, width, index), (through, bucket)) in self.windows {
            buckets.execute(params![execution, stream, field, width, index, super::super::position(through), encode(&bucket)?])?;
        }
        let mut query = connection.prepare_cached(
            "INSERT INTO storage_report_context VALUES (?1,?2,?3,?4)
            ON CONFLICT(execution,stream) DO UPDATE SET through=excluded.through,state=excluded.state",
        )?;
        for ((execution, stream), (through, values)) in self.contexts {
            query.execute(params![execution, stream, super::super::position(through), encode(&values)?])?;
        }
        Ok(())
    }
}

/* === Public Functions === */

pub(crate) fn latest_time(
    connection: &Connection, execution: &str, stream: &str, fields: &[String], at: u64,
) -> Result<Option<i128>, Error> {
    let mut query =
        connection.prepare("SELECT envelope,through FROM storage_report_previews WHERE execution=?1 AND stream=?2 AND field=?3")?;
    let mut latest = None;
    for field in fields {
        let row: Option<(Vec<u8>, String)> =
            query.query_row(params![execution, stream, field], |row| Ok((row.get(0)?, row.get(1)?))).optional()?;
        let Some((bytes, through)) = row else { continue };
        if through > super::super::position(at) {
            return Ok(None);
        }
        let projection = Projection::decode(&bytes)?;
        for bucket in projection.envelope.buckets.values() {
            latest = Some(latest.map_or(bucket.last.time, |time: i128| time.max(bucket.last.time)));
        }
    }
    Ok(latest)
}

// Indexed Empty Intervals Need No Raw Read; Only Partial Or Newer Buckets Do.
pub(crate) fn window_raw_ranges(
    connection: &Connection, execution: &str, stream: &str, field: &str, range: std::ops::RangeInclusive<i128>, at: u64, target: i128,
) -> Result<Vec<(i128, i128)>, Error> {
    let (from, to) = range.into_inner();
    let mut widths = connection.prepare("SELECT DISTINCT width FROM storage_plot_buckets WHERE execution=?1 AND stream=?2 AND field=?3")?;
    let widths = widths.query_map(params![execution, stream, field], |row| row.get::<_, i64>(0))?.collect::<Result<Vec<_>, _>>()?;
    let mut query = connection.prepare("SELECT bucket,through FROM storage_plot_buckets WHERE execution=?1 AND stream=?2 AND field=?3 AND width=?4 AND bucket>=?5 AND bucket<=?6 ORDER BY bucket")?;
    let mut ranges = Vec::new();
    for width in widths {
        let span = i128::from(width);
        let first = i64::try_from(from.div_euclid(span)).map_err(|_| Error::Data("preview range overflow"))?;
        let last = i64::try_from(to.div_euclid(span)).map_err(|_| Error::Data("preview range overflow"))?;
        let mut rows = query.query(params![execution, stream, field, width, first, last])?;
        while let Some(row) = rows.next()? {
            let start = i128::from(row.get::<_, i64>(0)?) * span;
            let end = start + span;
            let through: String = row.get(1)?;
            if start < from
                || end > to
                || span > target
                || start.div_euclid(target) != (end - 1).div_euclid(target)
                || through > super::super::position(at)
            {
                ranges.push((start.max(from), end.min(to + 1)));
            }
        }
    }
    Ok(ranges)
}

// Only Complete Buckets Inside The Window Are Reused; Callers Read Its Edges.
pub(crate) fn window_buckets(
    connection: &Connection, execution: &str, stream: &str, field: &str, from: i128, to: i128, at: u64,
) -> Result<Vec<(i128, i128, Bucket)>, Error> {
    let mut widths = connection.prepare("SELECT DISTINCT width FROM storage_plot_buckets WHERE execution=?1 AND stream=?2 AND field=?3")?;
    let widths = widths.query_map(params![execution, stream, field], |row| row.get::<_, i64>(0))?.collect::<Result<Vec<_>, _>>()?;
    let mut query = connection.prepare(
        "SELECT bucket,through,state FROM storage_plot_buckets
         WHERE execution=?1 AND stream=?2 AND field=?3 AND width=?4 AND bucket>=?5 AND bucket<?6 ORDER BY bucket",
    )?;
    let mut buckets = Vec::new();
    for width in widths {
        let span = i128::from(width);
        let first = i64::try_from(from.div_euclid(span) + i128::from(from.rem_euclid(span) != 0))
            .map_err(|_| Error::Data("preview range overflow"))?;
        let last = i64::try_from(to.div_euclid(span)).map_err(|_| Error::Data("preview range overflow"))?;
        let mut rows = query.query(params![execution, stream, field, width, first, last])?;
        while let Some(row) = rows.next()? {
            let start = i128::from(row.get::<_, i64>(0)?) * span;
            let through: String = row.get(1)?;
            if through > super::super::position(at) {
                continue;
            }
            buckets.push((start, start + span, decode(super::super::blob(row, 2)?)?));
        }
    }
    Ok(buckets)
}

pub(crate) fn report_context(
    connection: &Connection, execution: &str, stream: &str, at: u64,
) -> Result<BTreeMap<String, dokime_domain::model::values::Value>, Error> {
    let mut query =
        connection.prepare("SELECT stream,state FROM storage_report_context WHERE execution=?1 AND through<=?2 ORDER BY through")?;
    let mut rows = query.query(params![execution, super::super::position(at)])?;
    let mut values = BTreeMap::new();
    while let Some(row) = rows.next()? {
        let selected: String = row.get(0)?;
        if crate::plot::stream_matches_alias(&selected, stream) {
            values.extend(decode::<BTreeMap<String, dokime_domain::model::values::Value>>(super::super::blob(row, 1)?)?);
        }
    }
    Ok(values)
}

pub(super) fn report_sample_rows(
    connection: &Connection, root: &std::path::Path, execution: String, stream: String, at: harmos::storage::StorageCursor, limit: usize,
) -> Result<Vec<harmos::storage::StoredRow>, Error> {
    let Some(rows) = std::num::NonZeroUsize::new(limit) else { return Ok(Vec::new()) };
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
        .filter("execution", Some(execution.clone()))
        .filter("stream", Some(stream.clone()))
    };
    let mut selected = BTreeMap::new();
    for row in super::reader::stored(connection, root, &scan(0))? {
        selected.insert(row.record.position.sequence, row);
    }
    // The Batch Index Locates The Final Rows Without Reading Earlier Flushed Batches.
    let mut batches = connection.prepare(
        "SELECT CASE WHEN s.execution=?1 THEN b.rows ELSE (
            SELECT sum(rows) FROM stream_captures c WHERE c.execution=?1 AND c.file_id=b.file_id AND c.batch=b.batch)
            END,b.first_position FROM stream_batches b JOIN streams s ON s.file_id=b.file_id
         WHERE (s.execution=?1 OR EXISTS(SELECT 1 FROM stream_captures c
            WHERE c.execution=?1 AND c.file_id=b.file_id AND c.batch=b.batch))
         AND s.stream=?2 AND b.first_position<=?3 ORDER BY b.last_position DESC",
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
        let page = super::reader::stored(connection, root, &scan(after))?;
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
}

pub(super) fn report_record_rows(
    connection: &Connection, execution: String, cycle: String, at: harmos::storage::StorageCursor, limit: usize,
) -> Result<Vec<dokime_domain::model::records::evidence::RecordEntry>, Error> {
    let head = super::super::saved(connection)?;
    head.history.check(&at.history)?;
    let mut selected = BTreeMap::new();
    for order in ["ASC", "DESC"] {
        // Each Branch Seeks Its Own Owner Index Before The Bounded Union.
        for selection in [
            "SELECT position FROM storage_record_visibility WHERE execution=?1",
            "SELECT position FROM storage_record_visibility WHERE cycle=?2 AND execution IS NULL",
            "SELECT position FROM storage_record_visibility WHERE cycle IS NULL AND execution IS NULL
                AND json_extract(scope,'$.test_case_id') IS NULL
                AND json_extract(scope,'$.template_id') IS NULL
                AND json_extract(scope,'$.suite_id') IS NULL
                AND json_extract(scope,'$.step_id') IS NULL
                AND json_extract(scope,'$.criterion_id') IS NULL",
            "SELECT position FROM storage_record_captures WHERE execution=?1",
        ] {
            let sql = format!(
                "SELECT e.body,e.digest FROM ({selection} AND position<=?3 ORDER BY position {order} LIMIT ?4) i JOIN storage_data_entries e ON e.position=i.position"
            );
            let mut query = connection.prepare(&sql)?;
            let mut rows = query.query(params![execution, cycle, super::super::position(at.sequence), limit as i64])?;
            while let Some(row) = rows.next()? {
                let body = super::super::blob(row, 0)?;
                super::super::check_digest(body, super::super::blob(row, 1)?)?;
                let super::super::Entry::Record(row) = super::super::Entry::decode("record", body)? else { unreachable!() };
                let mut record = crate::storage::records::RecordRow::decode_entry(&row.payload)?;
                if let Some(scope) =
                    record.captured_scopes.iter().find(|scope| scope.test_execution_id.as_ref().is_some_and(|id| id.as_str() == execution))
                {
                    record.scope = Some(scope.clone());
                }
                selected.insert(row.position.sequence, record);
            }
        }
    }
    let mut rows: Vec<_> = selected.into_values().collect();
    if rows.len() > limit * 2 {
        rows.drain(limit..rows.len() - limit);
    }
    Ok(rows)
}

pub(crate) fn report_previews(
    connection: &Connection, execution: String, at: harmos::storage::StorageCursor, window: Option<(i128, i128)>,
) -> Result<Vec<FieldPreview>, Error> {
    let head = super::super::saved(connection)?;
    head.history.check(&at.history)?;
    let mut query =
        connection.prepare("SELECT stream,field,envelope,through FROM storage_report_previews WHERE execution=?1 ORDER BY stream,field")?;
    let mut rows = query.query([&execution])?;
    let mut fields = Vec::new();
    while let Some(row) = rows.next()? {
        let through: String = row.get(3)?;
        if through > super::super::position(at.sequence) {
            return Err(Error::Data("preview has advanced beyond requested evidence; request a current snapshot"));
        }
        let mut projection = Projection::decode(super::super::blob(row, 2)?)?;
        if let Some((from, to)) = window {
            let stream: String = row.get(0)?;
            let field: String = row.get(1)?;
            let buckets = window_buckets(connection, &execution, &stream, &field, from, to, at.sequence)?;
            let width = buckets.first().map(|(start, end, _)| end - start).unwrap_or(1);
            projection.envelope = Envelope {
                origin: 0,
                width,
                buckets: buckets.into_iter().map(|(start, _, bucket)| (start.div_euclid(width), bucket)).collect(),
            };
            projection.exact.retain(|(point, _)| point.time >= from && point.time < to);
            projection.count = projection.envelope.buckets.values().map(|bucket| bucket.count).sum();
        }
        fields.push(FieldPreview {
            stream: row.get(0)?,
            field: row.get(1)?,
            envelope: projection.envelope,
            exact: projection.exact.iter().map(|(point, value)| (*point, value.value())).collect(),
            total: projection.count,
            last: projection.last,
        });
    }
    Ok(fields)
}

/* === Private Functions === */

pub(crate) fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, Error> {
    let mut encoder = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::fast());
    serde_json::to_writer(&mut encoder, value)?;
    encoder.flush()?;
    Ok(encoder.finish()?)
}

pub(crate) fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, Error> {
    let decoder = flate2::read::ZlibDecoder::new(bytes);
    serde_json::from_reader(decoder).map_err(|error| {
        Error::Incompatible(format!("Preview layout is incompatible; use a fresh storage root and preserve existing evidence: {error}"))
    })
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_merges_fields_across_flushes() {
        use dokime_domain::model::values::Value;
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch(SCHEMA).unwrap();
        let settings = crate::config::local_storage_defaults().previews;
        let first = Value::from(17u64);
        let second = Value::from(29u64);
        let mut batch = Batch::new(settings);
        batch.context(&connection, "execution", 1, "stream", [("first", &first)]).unwrap();
        batch.save(&connection).unwrap();
        let mut batch = Batch::new(settings);
        batch.context(&connection, "execution", 2, "stream", [("second", &second)]).unwrap();
        batch.save(&connection).unwrap();
        let context = report_context(&connection, "execution", "stream", 2).unwrap();
        assert_eq!(context, BTreeMap::from([("first".into(), first), ("second".into(), second)]));
        assert!(report_context(&connection, "execution", "stream", 1).unwrap().is_empty());
    }

    #[test]
    fn increasing_capacity_cannot_restore_a_discarded_prefix() {
        use dokime_domain::model::values::Value;
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch(SCHEMA).unwrap();
        let mut settings = crate::config::local_storage_defaults().previews;
        settings.exact_points = 1.try_into().unwrap();
        for position in 1..=3 {
            if position == 3 {
                settings.exact_points = 8.try_into().unwrap();
            }
            let mut batch = Batch::new(settings);
            batch
                .push(
                    &connection,
                    Input {
                        execution: "execution",
                        stream: "stream",
                        position,
                        time: position.into(),
                    },
                    "value",
                    None,
                    &Value::Null,
                )
                .unwrap();
            batch.save(&connection).unwrap();
        }
        let bytes: Vec<u8> = connection.query_row("SELECT envelope FROM storage_report_previews", [], |row| row.get(0)).unwrap();
        let projection = Projection::decode(&bytes).unwrap();
        assert_eq!(projection.count, 3);
        assert!(projection.exact.is_empty());
        assert!(projection.envelope.buckets.is_empty());
        assert_eq!(projection.last.unwrap().position, 3);
    }

    #[test]
    fn exact_prefix_survives_compaction_and_yields_to_envelope() {
        use dokime_domain::model::values::{Value, numeric_value};
        for capacity in [1usize, 3, 8] {
            let connection = Connection::open_in_memory().unwrap();
            connection.execute_batch(SCHEMA).unwrap();
            let mut settings = crate::config::local_storage_defaults().previews;
            settings.exact_points = capacity.try_into().unwrap();
            settings.bucket_count = 2.try_into().unwrap();
            let values: Vec<_> = (0..capacity)
                .map(|index| if index % 3 == 1 { Value::Null } else { Value::Unsigned((1u64 << 53) + index as u64 + 1) })
                .collect();
            let mut batch = Batch::new(settings);
            for (index, value) in values.iter().enumerate() {
                batch
                    .push(
                        &connection,
                        Input {
                            execution: "execution",
                            stream: "stream",
                            position: index as u64 + 1,
                            time: index as i128 * 17 - 23,
                        },
                        "value",
                        numeric_value(value),
                        value,
                    )
                    .unwrap();
            }
            batch.save(&connection).unwrap();
            let saved: Vec<u8> = connection.query_row("SELECT envelope FROM storage_report_previews", [], |row| row.get(0)).unwrap();
            let projection = Projection::decode(&saved).unwrap();
            assert_eq!(projection.exact.iter().map(|(_, value)| value.value()).collect::<Vec<_>>(), values);
            let mut batch = Batch::new(settings);
            let spike = Value::Number(-1e12);
            batch
                .push(
                    &connection,
                    Input {
                        execution: "execution",
                        stream: "stream",
                        position: capacity as u64 + 1,
                        time: 101,
                    },
                    "value",
                    Some(-1e12),
                    &spike,
                )
                .unwrap();
            batch.save(&connection).unwrap();
            let saved: Vec<u8> = connection.query_row("SELECT envelope FROM storage_report_previews", [], |row| row.get(0)).unwrap();
            let projection = Projection::decode(&saved).unwrap();
            assert!(projection.exact.is_empty());
            assert_eq!(projection.count, values.len() as u64 + 1);
            assert_eq!(
                projection.envelope.buckets.values().map(|bucket| bucket.count).sum::<u64>(),
                values.iter().filter(|value| numeric_value(value).is_some()).count() as u64 + 1
            );
            assert_eq!(projection.envelope.buckets.values().map(|bucket| bucket.min.value).fold(f64::INFINITY, f64::min), -1e12);
            assert!(projection.envelope.buckets.len() <= settings.bucket_count.get());
        }
    }

    #[test]
    fn window_buckets_reopen_and_include_late_samples() {
        use dokime_domain::model::values::Value;
        let connection = Connection::open_in_memory().unwrap();
        connection.execute_batch(SCHEMA).unwrap();
        let mut settings = crate::config::local_storage_defaults().previews;
        settings.bucket_interval = std::time::Duration::from_secs(1);
        for (position, time, value) in [(1, 100_000_000, 2.0), (2, 900_000_000, 4.0), (3, 2_100_000_000, 9.0)] {
            let mut batch = Batch::new(settings);
            batch
                .push(
                    &connection,
                    Input {
                        execution: "execution",
                        stream: "stream",
                        position,
                        time,
                    },
                    "value",
                    Some(value),
                    &Value::Number(value),
                )
                .unwrap();
            batch.save(&connection).unwrap();
        }
        let buckets = window_buckets(&connection, "execution", "stream", "value", 0, 2_000_000_000, 3).unwrap();
        assert_eq!(buckets.len(), 1);
        let bucket = &buckets[0].2;
        assert_eq!(bucket.count, 2);
        assert_eq!(bucket.scale * bucket.sum / bucket.count as f64, 3.0);
        assert_eq!(bucket.scale.powi(2) * bucket.squares / bucket.count as f64, 10.0);
        assert!(window_buckets(&connection, "execution", "stream", "value", 1, 999_999_999, 3).unwrap().is_empty());

        let mut batch = Batch::new(settings);
        batch
            .push(
                &connection,
                Input {
                    execution: "execution",
                    stream: "stream",
                    position: 4,
                    time: 500_000_000,
                },
                "value",
                Some(-10.0),
                &Value::Number(-10.0),
            )
            .unwrap();
        batch.save(&connection).unwrap();
        let buckets = window_buckets(&connection, "execution", "stream", "value", 0, 2_000_000_000, 4).unwrap();
        assert_eq!(buckets[0].2.count, 3);
        assert_eq!(buckets[0].2.min.value, -10.0);
        assert_eq!(buckets[0].2.first.position, 1);
        assert_eq!(buckets[0].2.last.position, 2);
        assert!(window_buckets(&connection, "execution", "stream", "value", 0, 2_000_000_000, 3).unwrap().is_empty());
    }

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
                    let mean = members.iter().map(|point| point.value).sum::<f64>() / members.len() as f64;
                    assert!(
                        (bucket.scale * bucket.sum / bucket.count as f64 - mean).abs() <= 1e-12 * mean.abs().max(bucket.scale).max(1.0)
                    );
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
