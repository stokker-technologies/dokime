/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod native;
mod projection;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{
    mem::size_of,
    num::NonZeroUsize,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};
// Third Party Imports
use harmos::{
    MemorySize,
    storage::{
        frame::{ProjectedRow, Scan, ScanCursor, ScanOrder, ScanPage},
        reduce::{ScalarKind, ScalarValue},
        *,
    },
};
use rusqlite::{Connection, types::Value as SqlValue};
// Local Imports
use super::super::{Error, memory, query};
use super::{Entry, SavedData, blob, check_digest, entry, kind, position};
use projection::Projection;

/* === Constants === */
const SCHEMA: &str = "
CREATE TEMP TABLE IF NOT EXISTS storage_frame_rows (
    run INTEGER NOT NULL, key BLOB NOT NULL, body BLOB NOT NULL,
    PRIMARY KEY(run, key)
) WITHOUT ROWID;
";
const CAPTURE: u8 = 0;
const KEYED: u8 = 1;
const CAPTURE_TOKEN: usize = 1 + size_of::<u64>();
static RUNS: AtomicU64 = AtomicU64::new(1);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Read {
    pub scan: Scan,
    pub order: ScanOrder,
    pub cursor: Option<ScanCursor>,
    pub limits: ReadLimits,
    pub maximum: NonZeroUsize,
}
#[derive(Default)]
pub(super) struct Runs(Mutex<Vec<Run>>);
struct Run {
    id: u64,
    scan: Scan,
    order: ScanOrder,
}
struct Window {
    filter: String,
    arguments: Vec<SqlValue>,
}
struct Decoder<'a> {
    memory: &'a Arc<memory::Budget>,
    scanned: &'a AtomicU64,
    cut: &'a SavedData,
    read: &'a Read,
    schema: &'a StorageSchema,
    projection: Projection<'a>,
    active: Option<&'a StorageViewComputation>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Runs {
    fn owns(&self, id: u64, scan: &Scan, order: &ScanOrder) -> bool {
        self.0
            .lock()
            .unwrap()
            .iter()
            .any(|run| run.id == id && run.scan == *scan && run.order == *order)
    }
    fn insert(&self, id: u64, scan: &Scan, order: &ScanOrder) {
        self.0.lock().unwrap().push(Run {
            id,
            scan: scan.clone(),
            order: order.clone(),
        });
    }
    fn remove(&self, id: u64) {
        self.0.lock().unwrap().retain(|run| run.id != id);
    }
    pub(super) fn take(&self) -> Vec<u64> {
        std::mem::take(&mut *self.0.lock().unwrap())
            .into_iter()
            .map(|run| run.id)
            .collect()
    }
}

impl Window {
    fn new(cut: &SavedData, selection: &ReadSelection) -> Result<Self, Error> {
        let schema = selection
            .schema
            .as_ref()
            .ok_or(Error::Data("scan schema is absent"))?;
        let mut arguments = vec![
            SqlValue::Text(kind(selection.kind).into()),
            SqlValue::Text(serde_json::to_string(schema)?),
            SqlValue::Text(position(selection.after)),
            SqlValue::Text(position(selection.through.unwrap_or(cut.sequence))),
            SqlValue::Text(position(cut.retention_generation)),
        ];
        let mut filter = String::from(
            "kind=?1 AND schema_key=?2 AND position>?3 AND position<=?4 AND (retired_at IS NULL OR retired_at>?5)",
        );
        for (name, value) in [
            ("scope", selection.scope.as_ref()),
            ("stream", selection.stream.as_ref()),
            ("item", selection.item.as_ref()),
        ] {
            if let Some(value) = value {
                arguments.push(SqlValue::Text(value.as_str().into()));
                filter.push_str(&format!(" AND {name}=?{}", arguments.len()));
            }
        }
        if let Some(range) = selection.source_time {
            arguments.push(SqlValue::Blob(entry::timestamp_key(range.start).to_vec()));
            filter.push_str(&format!(" AND source_time>=?{}", arguments.len()));
            arguments.push(SqlValue::Blob(entry::timestamp_key(range.end).to_vec()));
            filter.push_str(&format!(" AND source_time<?{}", arguments.len()));
        }
        Ok(Self { filter, arguments })
    }
}

impl Decoder<'_> {
    fn decode(&self, row: &rusqlite::Row<'_>) -> Result<ProjectedRow, Error> {
        checkpoint(self.active)?;
        self.scanned.fetch_add(1, Ordering::AcqRel);
        let indexed = super::sequence(
            row.get_ref(0)?
                .as_str()
                .map_err(|_| Error::Corrupt("scan position is not text"))?,
        )?;
        let body = blob(row, 1)?;
        if body.len() > self.read.maximum.get() {
            return Err(Error::Corrupt("scan row exceeds backend bound"));
        }
        check_digest(body, blob(row, 2)?)?;
        let selection = &self.read.scan.selection;
        let _decoded = self.memory.reserve_owned(
            Entry::decoding_bytes(kind(selection.kind), body.len())?,
            usize::MAX,
        )?;
        let entry = Entry::decode(kind(selection.kind), body)?;
        let (record, stream) = match &entry {
            Entry::Record(record) => (record, None),
            Entry::Stream(row) => (&row.record, Some(&row.stream)),
            _ => return Err(Error::Corrupt("scan row collection differs")),
        };
        check_record(record, stream, indexed, self.schema, selection, self.cut)?;
        let bytes = self.projection.output_memory(record, stream)?;
        let _charge = self.memory.reserve_owned(bytes, usize::MAX)?;
        let values = self.projection.project(
            record,
            stream,
            self.memory,
            bound(bytes)?,
            self.read.maximum,
        )?;
        if values.memory_size() > bytes {
            return Err(Error::Corrupt("scan projection exceeded preclaim"));
        }
        Ok(ProjectedRow {
            capture_order: indexed,
            values,
        })
    }
}

/* === Public Functions === */
pub(super) fn admission(scan: &Scan, order: &ScanOrder) -> harmos::Result<()> {
    let selection = &scan.selection;
    if !matches!(selection.kind, ReadKind::Records | ReadKind::Stream)
        || selection.schema.is_none()
        || selection.object_key.is_some()
        || selection.object_versions != ObjectVersions::Revisions
        || selection.order != ReadOrder::Ascending
        || (selection.kind == ReadKind::Records && selection.stream.is_some())
    {
        return Err(invalid(
            "frame scans require an explicit record/stream schema and capture order",
        ));
    }
    projection::admission(selection, &scan.columns)?;
    if let ScanOrder::Keyed { by, time } = order {
        for column in by.iter().chain(time) {
            let field = scan
                .columns
                .get(*column)
                .ok_or_else(|| invalid("keyed column is out of range"))?;
            if field.kind == ScalarKind::Exact {
                return Err(invalid("keyed scans cannot order exact rationals"));
            }
        }
    }
    Ok(())
}

pub(super) fn planning_bytes(scan: &Scan) -> Result<usize, Error> {
    scan.memory_size()
        .checked_mul(8)
        .and_then(|bytes| bytes.checked_add(64 * 1024))
        .ok_or(Error::Data("scan planning memory overflows"))
}

pub(super) fn page(
    connection: &Connection,
    memory: &Arc<memory::Budget>,
    scanned: &AtomicU64,
    cut: &SavedData,
    read: &Read,
    runs: &Runs,
    active: Option<&StorageViewComputation>,
) -> Result<ScanPage, Error> {
    admission(&read.scan, &read.order)?;
    checkpoint(active)?;
    let head = super::saved(connection)?;
    head.history.check(&cut.history)?;
    if head.sequence < cut.sequence {
        return Err(Error::Corrupt("scan view exceeds saved frontier"));
    }
    let selection = &read.scan.selection;
    if selection
        .through
        .is_some_and(|through| through > cut.sequence)
    {
        return Err(Error::Data("scan cut exceeds the original saved view"));
    }
    let window = Window::new(cut, selection)?;
    let schema = schema(connection, memory, cut, selection, read.maximum, active)?;
    let construction = projection::construction_memory(&schema, &read.scan.columns)?;
    let _projection = memory.reserve_owned(construction, usize::MAX)?;
    let decoder = Decoder {
        memory,
        scanned,
        cut,
        read,
        schema: &schema,
        projection: Projection::new(&schema, &read.scan.columns, construction)?,
        active,
    };
    match &read.order {
        ScanOrder::Capture => capture(connection, &window, &decoder),
        ScanOrder::Keyed { .. } => keyed(connection, &window, &decoder, runs),
    }
}

pub(super) fn release(connection: &Connection, runs: &[u64]) -> Result<(), Error> {
    connection.execute_batch(SCHEMA)?;
    let mut statement = connection.prepare("DELETE FROM temp.storage_frame_rows WHERE run=?1")?;
    for run in runs {
        statement.execute([run_value(*run)?])?;
    }
    Ok(())
}

/* === Private Functions === */
fn capture(
    connection: &Connection,
    window: &Window,
    decoder: &Decoder<'_>,
) -> Result<ScanPage, Error> {
    let limits = decoder.read.limits;
    let selection = &decoder.read.scan.selection;
    let after = match &decoder.read.cursor {
        None => selection.after,
        Some(cursor) => {
            let token = cursor.token.as_slice();
            if token.len() != CAPTURE_TOKEN || token[0] != CAPTURE {
                return Err(invalid("foreign capture continuation").into());
            }
            let after = u64::from_be_bytes(token[1..].try_into().expect("checked token length"));
            if after < selection.after {
                return Err(invalid("continuation precedes its scan window").into());
            }
            after
        }
    };
    // The Continuation Replaces The Window's Lower Bound So The Index Seeks To It.
    let mut arguments = window.arguments.clone();
    arguments[2] = SqlValue::Text(position(after));
    let probe = limits.rows.get().saturating_add(1);
    let mut statement = connection.prepare(&format!(
        "SELECT position,body,digest FROM storage_data_entries WHERE {} ORDER BY position ASC LIMIT {probe}",
        window.filter
    ))?;
    let mut sql = statement.query(rusqlite::params_from_iter(&arguments))?;
    let mut rows = slots(limits.rows.get())?;
    let mut bytes = CAPTURE_TOKEN;
    let mut more = false;
    while let Some(row) = sql.next()? {
        if rows.len() == limits.rows.get() {
            more = true;
            break;
        }
        let projected = decoder.decode(row)?;
        let text = text_bytes(&projected);
        if bytes.saturating_add(text) > limits.bytes.get() {
            more = true;
            break;
        }
        bytes += text;
        rows.push(projected);
    }
    finish(rows, more, |last| {
        let mut token = Vec::with_capacity(CAPTURE_TOKEN);
        token.push(CAPTURE);
        token.extend_from_slice(&last.capture_order.to_be_bytes());
        token
    })
}

fn keyed(
    connection: &Connection,
    window: &Window,
    decoder: &Decoder<'_>,
    runs: &Runs,
) -> Result<ScanPage, Error> {
    let read = decoder.read;
    let ScanOrder::Keyed { by, time } = &read.order else {
        unreachable!("keyed page requires a keyed order");
    };
    connection.execute_batch(SCHEMA)?;
    let (run, after) = match &read.cursor {
        Some(cursor) => {
            let token = cursor.token.as_slice();
            if token.len() < CAPTURE_TOKEN || token[0] != KEYED {
                return Err(invalid("foreign keyed continuation").into());
            }
            let run = u64::from_be_bytes(token[1..CAPTURE_TOKEN].try_into().expect("checked"));
            if !runs.owns(run, &read.scan, &read.order) {
                return Err(harmos::Error::ViewExpired.into());
            }
            (run, token[CAPTURE_TOKEN..].to_vec())
        }
        // Every Pass Sorts Its Own Copy: Two Streams Of One Pass May Share An Order.
        None => {
            let run = RUNS.fetch_add(1, Ordering::Relaxed);
            if let Err(error) = sort(connection, window, decoder, run, by, *time) {
                release(connection, &[run])?;
                return Err(error);
            }
            runs.insert(run, &read.scan, &read.order);
            (run, Vec::new())
        }
    };

    // A Finished Or Failed Pass Frees Its Copy For Evidence Writes Sharing The Allocator.
    let page = sorted(connection, decoder, run, after);
    if !matches!(&page, Ok(page) if page.next.is_some()) {
        runs.remove(run);
        release(connection, &[run])?;
    }
    page
}

fn sorted(
    connection: &Connection,
    decoder: &Decoder<'_>,
    run: u64,
    after: Vec<u8>,
) -> Result<ScanPage, Error> {
    let read = decoder.read;
    let limits = read.limits;
    let probe = limits.rows.get().saturating_add(1);
    let mut statement = connection.prepare(&format!(
        "SELECT key,body FROM temp.storage_frame_rows WHERE run=?1 AND key>?2 ORDER BY key LIMIT {probe}"
    ))?;
    let mut sql = statement.query(rusqlite::params![run_value(run)?, after])?;
    let mut rows = slots(limits.rows.get())?;
    let mut last = Vec::new();
    let mut text = 0usize;
    let mut more = false;
    while let Some(row) = sql.next()? {
        checkpoint(decoder.active)?;
        if rows.len() == limits.rows.get() {
            more = true;
            break;
        }
        let key = blob(row, 0)?;
        let body = blob(row, 1)?;
        if key.len() < size_of::<u64>() || body.len() > read.maximum.get() {
            return Err(Error::Corrupt("sorted scan row is malformed"));
        }
        let _decoded = decoder
            .memory
            .reserve_owned(body.len().saturating_mul(4), usize::MAX)?;
        let values: Vec<ScalarValue> = serde_json::from_slice(body)?;
        if values.len() != read.scan.columns.len() {
            return Err(Error::Corrupt("sorted scan row width differs"));
        }
        let capture = u64::from_be_bytes(
            key[key.len() - size_of::<u64>()..]
                .try_into()
                .expect("checked key length"),
        );
        let projected = ProjectedRow {
            capture_order: capture,
            values,
        };
        let next = text.saturating_add(text_bytes(&projected));
        if next.saturating_add(CAPTURE_TOKEN + key.len()) > limits.bytes.get() {
            more = true;
            break;
        }
        text = next;
        last.clear();
        last.extend_from_slice(key);
        rows.push(projected);
    }
    finish(rows, more, |_| {
        let mut token = Vec::with_capacity(CAPTURE_TOKEN + last.len());
        token.push(KEYED);
        token.extend_from_slice(&run.to_be_bytes());
        token.extend_from_slice(&last);
        token
    })
}

fn sort(
    connection: &Connection,
    window: &Window,
    decoder: &Decoder<'_>,
    run: u64,
    by: &[usize],
    time: Option<usize>,
) -> Result<(), Error> {
    let mut select = connection.prepare(&format!(
        "SELECT position,body,digest FROM storage_data_entries WHERE {} ORDER BY position ASC",
        window.filter
    ))?;
    let mut insert = connection
        .prepare("INSERT INTO temp.storage_frame_rows(run,key,body) VALUES (?1,?2,?3)")?;
    let mut sql = select.query(rusqlite::params_from_iter(&window.arguments))?;
    let run = run_value(run)?;
    while let Some(row) = sql.next()? {
        let projected = decoder.decode(row)?;
        let mut key = Vec::new();
        for column in by.iter().copied().chain(time) {
            encode_key(&projected.values[column], &mut key)?;
        }
        key.extend_from_slice(&projected.capture_order.to_be_bytes());
        let body = serde_json::to_vec(&projected.values)?;
        insert.execute(rusqlite::params![run, key, body])?;
    }
    Ok(())
}

fn finish(
    rows: Vec<ProjectedRow>,
    more: bool,
    token: impl FnOnce(&ProjectedRow) -> Vec<u8>,
) -> Result<ScanPage, Error> {
    let next = match (rows.last(), more) {
        (None, true) => {
            return Err(harmos::Error::ResourceExhausted {
                resource: "SQLite scan row text",
            }
            .into());
        }
        (Some(last), true) => Some(ScanCursor { token: token(last) }),
        _ => None,
    };
    Ok(ScanPage { rows, next })
}

fn encode_key(value: &ScalarValue, key: &mut Vec<u8>) -> Result<(), Error> {
    value.check()?;
    match value {
        ScalarValue::Missing => key.push(0),
        ScalarValue::Null => key.push(1),
        value => {
            key.push(2);
            match value {
                ScalarValue::Bool(value) => key.push(u8::from(*value)),
                ScalarValue::Signed(value) => {
                    key.extend_from_slice(&((*value as u64) ^ (1 << 63)).to_be_bytes());
                }
                ScalarValue::Unsigned(value) => key.extend_from_slice(&value.to_be_bytes()),
                ScalarValue::Float(value) => {
                    // Negative Zero Compares Equal To Zero And Shares Its Key.
                    let bits = if *value == 0.0 { 0.0f64 } else { *value }.to_bits();
                    let ordered = if bits >> 63 == 1 {
                        !bits
                    } else {
                        bits ^ (1 << 63)
                    };
                    key.extend_from_slice(&ordered.to_be_bytes());
                }
                ScalarValue::Datetime(value) => {
                    key.extend_from_slice(&((*value as u128) ^ (1 << 127)).to_be_bytes());
                }
                ScalarValue::Text(text) => {
                    for byte in text.bytes() {
                        key.push(byte);
                        if byte == 0 {
                            key.push(0xff);
                        }
                    }
                    key.extend_from_slice(&[0, 0]);
                }
                _ => return Err(invalid("keyed scans cannot order exact rationals").into()),
            }
        }
    }
    Ok(())
}

fn schema(
    connection: &Connection,
    memory: &Arc<memory::Budget>,
    cut: &SavedData,
    selection: &ReadSelection,
    maximum: NonZeroUsize,
    active: Option<&StorageViewComputation>,
) -> Result<StorageSchema, Error> {
    let key = selection.schema.as_ref().expect("admitted schema");
    let through = selection.through.unwrap_or(cut.sequence);
    let mut statement = connection.prepare("SELECT position,body,digest FROM storage_data_entries WHERE kind='schema' AND schema_key=?1 AND position<=?2 AND (retired_at IS NULL OR retired_at>?3) ORDER BY position ASC")?;
    let mut rows = statement.query(rusqlite::params![
        serde_json::to_string(key)?,
        position(through),
        position(cut.retention_generation)
    ])?;
    let mut original: Option<StorageSchema> = None;
    while let Some(row) = rows.next()? {
        checkpoint(active)?;
        let indexed = super::sequence(
            row.get_ref(0)?
                .as_str()
                .map_err(|_| Error::Corrupt("scan schema position is not text"))?,
        )?;
        let body = blob(row, 1)?;
        if body.len() > maximum.get() {
            return Err(Error::Corrupt("scan schema exceeds backend bound"));
        }
        check_digest(body, blob(row, 2)?)?;
        let _decoded =
            memory.reserve_owned(Entry::decoding_bytes("schema", body.len())?, usize::MAX)?;
        let schema: StorageSchema = serde_json::from_slice(body)?;
        schema.position.history.check(&cut.history)?;
        if schema.key != *key || schema.position.sequence != indexed || indexed > through {
            return Err(Error::Corrupt("scan schema index differs"));
        }
        match &original {
            None => original = Some(schema),
            // Identical Re-Admissions Do Not Change The Codec Of Earlier Rows.
            Some(original) if original.fields != schema.fields => {
                return Err(Error::Data(
                    "scan source contains incompatible schema declarations",
                ));
            }
            Some(_) => {}
        }
    }
    original.ok_or(Error::Data("scan schema is absent from saved view"))
}

fn check_record(
    record: &StoredRecord,
    stream: Option<&harmos::Identifier>,
    indexed: u64,
    schema: &StorageSchema,
    selection: &ReadSelection,
    cut: &SavedData,
) -> Result<(), Error> {
    if record.position.sequence != indexed
        || record.schema != schema.key
        || schema.position.sequence >= indexed
        || indexed <= selection.after
        || indexed > selection.through.unwrap_or(cut.sequence)
        || selection
            .scope
            .as_ref()
            .is_some_and(|scope| record.scope.as_ref() != Some(scope))
        || selection
            .item
            .as_ref()
            .is_some_and(|item| &record.item != item)
        || selection
            .stream
            .as_ref()
            .is_some_and(|selected| stream != Some(selected))
        || selection
            .source_time
            .is_some_and(|range| !range.contains(record.source_time))
    {
        return Err(Error::Corrupt("scan row differs from selected index"));
    }
    record.position.history.check(&cut.history)?;
    Ok(())
}

fn text_bytes(row: &ProjectedRow) -> usize {
    row.values
        .iter()
        .map(|value| match value {
            ScalarValue::Text(text) => text.capacity(),
            _ => 0,
        })
        .sum()
}

fn run_value(run: u64) -> Result<i64, Error> {
    i64::try_from(run).map_err(|_| Error::Data("sorted scan run identity overflows"))
}

fn checkpoint(active: Option<&StorageViewComputation>) -> Result<(), Error> {
    query::checkpoint()?;
    if let Some(active) = active {
        active.check()?;
    }
    Ok(())
}

fn slots<T>(count: usize) -> Result<Vec<T>, Error> {
    let mut values = Vec::new();
    values
        .try_reserve_exact(count)
        .map_err(|_| exhausted("SQLite scan slots"))?;
    Ok(values)
}

fn bound(bytes: usize) -> Result<NonZeroUsize, Error> {
    NonZeroUsize::new(bytes)
        .filter(|bytes| bytes.get() != usize::MAX)
        .ok_or_else(|| exhausted("SQLite scan memory"))
}

fn exhausted(resource: &'static str) -> Error {
    harmos::Error::ResourceExhausted { resource }.into()
}

fn invalid(reason: &'static str) -> harmos::Error {
    harmos::Error::InvalidInput {
        field: "SQLite frame scan",
        reason,
    }
}

/* === Tests === */
