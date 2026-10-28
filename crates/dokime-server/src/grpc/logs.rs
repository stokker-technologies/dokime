/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Tail One Log File For Every Subscriber: One Reader Parses Appended Bytes Once And Fans Records Out.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

// Third Party Imports
use dokime::harmos::operations::{Response, StreamOutput};
use serde_json::Value;
use tokio::io::{AsyncReadExt as _, AsyncSeekExt as _};
use tokio::sync::broadcast;
use tonic::Status;

// Local Imports
use super::redaction;
use dokime::domain::model::backend::wire::{LogFrame, LogFrameFieldsEntry, LogLevel};
use dokime::domain::model::identity::wire::Timestamp;

/* === Constants === */

const SERVER_DEFAULT_REPLAY_LINES: u64 = 500;
const SERVER_LOG_LINE_BYTES: u64 = 262_144;
const SERVER_LOG_PENDING_FRAMES: u64 = 8_192;
const SERVER_LOG_READ_WINDOW_BYTES: u64 = 262_144;
const SERVER_MAXIMUM_REPLAY_LINES: u64 = 5_000;

// One Replay Ring Per Wire Level, Indexed By The Level's Discriminant.
const LEVELS: usize = LogLevel::Error as usize + 1;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// One Log File's Shared Tail: A Single Reader, Bounded Replay Rings And A Bounded Live Fan-Out.
pub(super) struct Tail {
    path: PathBuf,
    poll_interval: Duration,
    message_limit: usize,
    reader: tokio::sync::Mutex<Reader>,
    shared: std::sync::Mutex<Shared>,
}

/// What One Subscriber Asks Of The Shared Tail.
pub(super) struct Watch {
    pub(super) min_level: LogLevel,
    pub(super) replay_lines: u32,
}

// A Parsed Record, Shared By Every Subscriber; Its Encoded Size Is Measured Once.
struct Record {
    sequence: u64,
    bytes: usize,
    frame: LogFrame,
}

// The Read Position Survives Between Subscribers, So A Returning Watcher Reads Only Appended Bytes.
#[derive(Default)]
struct Reader {
    offset: u64,
    carry: Vec<u8>,
    discarding_line: bool,
    bytes_read: u64,
}

struct Shared {
    sequence: u64,
    rings: [Ring; LEVELS],
    live: Ring,
    sender: broadcast::Sender<u64>,
    polling: bool,
}

#[derive(Default)]
struct Ring {
    records: VecDeque<Arc<Record>>,
    bytes: usize,
}

struct Subscription {
    tail: Arc<Tail>,
    replay: VecDeque<Arc<Record>>,
    live: broadcast::Receiver<u64>,
    dropped: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Tail {
    pub(super) fn new(path: PathBuf, poll_interval: Duration, message_limit: usize) -> Arc<Self> {
        let capacity = usize::try_from(SERVER_LOG_PENDING_FRAMES).expect("validated pending log count");
        let (sender, _) = broadcast::channel(capacity);
        let shared = Shared {
            sequence: 0,
            rings: Default::default(),
            live: Ring::default(),
            sender,
            polling: false,
        };
        Arc::new(Self {
            path,
            poll_interval,
            message_limit,
            reader: tokio::sync::Mutex::new(Reader::default()),
            shared: std::sync::Mutex::new(shared),
        })
    }

    pub(super) async fn watch(self: &Arc<Self>, watch: Watch, output: &StreamOutput<LogFrame>) -> Result<(), Status> {
        let replay_lines = if watch.replay_lines == 0 {
            SERVER_DEFAULT_REPLAY_LINES
        } else {
            u64::from(watch.replay_lines).min(SERVER_MAXIMUM_REPLAY_LINES)
        };
        let replay_lines = usize::try_from(replay_lines).expect("validated replay line count");
        let mut subscription = self.subscribe(watch.min_level, replay_lines).await?;
        loop {
            // A Quiet Or Filtered Log Answers With A Keepalive Instead Of Going Silent; Waiting Is Cancel-Safe.
            let frame = match tokio::time::timeout(super::WATCH_HEARTBEAT, subscription.next(watch.min_level, self.message_limit)).await {
                Ok(next) => match next? {
                    Some(frame) => frame,
                    None => return Ok(()),
                },
                Err(_) => subscription.keepalive(),
            };
            output.send_owned(frame).await.map_err(memory_failure)?;
        }
    }

    // Catches The Shared Reader Up To The File's Current End, Then Snapshots Replay And Joins The Live Fan-Out Atomically.
    async fn subscribe(self: &Arc<Self>, min_level: LogLevel, replay_lines: usize) -> Result<Subscription, Status> {
        let mut reader = self.reader.lock().await;
        let metadata = tokio::fs::metadata(&self.path).await;
        let end = metadata.map_or(0, |metadata| metadata.len());
        // Each Fixed Read Window Yields, Giving Shared Cancellation A Boundary.
        while self.advance(&mut reader, Some(end)).await > 0 {
            tokio::task::yield_now().await;
        }

        let mut shared = self.lock();
        let (replay, dropped) = shared.replay(min_level, replay_lines, self.message_limit);
        let live = shared.sender.subscribe();
        if !shared.polling {
            shared.polling = true;
            tokio::spawn(self.clone().poll());
        }
        Ok(Subscription {
            tail: self.clone(),
            replay,
            live,
            dropped,
        })
    }

    // Polls Only While Someone Subscribes; The Last Subscriber Leaving Parks The Reader At Its Offset.
    async fn poll(self: Arc<Self>) {
        loop {
            let mut reader = self.reader.lock().await;
            let read = self.advance(&mut reader, None).await;
            drop(reader);
            if self.park_when_unwatched() {
                return;
            }
            match read {
                0 => tokio::time::sleep(self.poll_interval).await,
                _ => tokio::task::yield_now().await,
            }
        }
    }

    fn park_when_unwatched(&self) -> bool {
        let mut shared = self.lock();
        shared.polling = shared.sender.receiver_count() > 0;
        !shared.polling
    }

    async fn advance(&self, reader: &mut Reader, end: Option<u64>) -> u64 {
        let Ok(metadata) = tokio::fs::metadata(&self.path).await else {
            return 0;
        };
        let length = metadata.len().min(end.unwrap_or(u64::MAX));
        if length < reader.offset {
            // The File Was Replaced, So Its Earlier Records No Longer Replay.
            *reader = Reader {
                bytes_read: reader.bytes_read,
                ..Reader::default()
            };
            self.lock().rings = Default::default();
        }
        if length == reader.offset {
            return 0;
        }
        let Some(bytes) = self.read_window(reader.offset, length).await else {
            return 0;
        };
        let read = bytes.len() as u64;
        reader.offset += read;
        reader.bytes_read += read;
        let complete = reader.complete_lines(&bytes);
        let frames = complete.split(|byte| *byte == b'\n').filter(|line| line.len() as u64 <= SERVER_LOG_LINE_BYTES).filter_map(frame_wire);
        for frame in frames {
            // A Frame Whose Size Cannot Be Measured Cannot Be Sent, So It Never Enters The Fan-Out.
            let Ok(bytes) = super::content::output_frame_len(&frame) else {
                continue;
            };
            self.lock().publish(frame, bytes, self.message_limit);
        }
        read
    }

    async fn read_window(&self, offset: u64, length: u64) -> Option<Vec<u8>> {
        let mut file = tokio::fs::File::open(&self.path).await.ok()?;
        file.seek(std::io::SeekFrom::Start(offset)).await.ok()?;
        let window = (length - offset).min(SERVER_LOG_READ_WINDOW_BYTES);
        let mut bytes = vec![0; usize::try_from(window).unwrap_or_default()];
        let read = file.read(&mut bytes).await.ok()?;
        bytes.truncate(read);
        Some(bytes)
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Shared> {
        self.shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl Reader {
    // Everything Up To The Last Newline Is Whole Records; The Remainder Is A Record Still Being Written,
    // Unless It Has Grown Past A Plausible One.
    fn complete_lines(&mut self, bytes: &[u8]) -> Vec<u8> {
        self.carry.extend_from_slice(bytes);
        if self.discarding_line {
            let Some(boundary) = self.carry.iter().position(|byte| *byte == b'\n') else {
                self.carry.clear();
                return Vec::new();
            };
            self.carry.drain(..=boundary);
            self.discarding_line = false;
        }
        let boundary = self.carry.iter().rposition(|byte| *byte == b'\n');
        match boundary {
            Some(boundary) => self.carry.drain(..=boundary).collect::<Vec<_>>(),
            None if self.carry.len() as u64 > SERVER_LOG_LINE_BYTES => {
                self.carry.clear();
                self.discarding_line = true;
                Vec::new()
            }
            None => Vec::new(),
        }
    }
}

impl Shared {
    fn publish(&mut self, frame: LogFrame, bytes: usize, message_limit: usize) {
        self.sequence += 1;
        let record = Arc::new(Record { sequence: self.sequence, bytes, frame });
        let capacity = usize::try_from(SERVER_MAXIMUM_REPLAY_LINES).expect("validated replay line count");
        self.rings[record.frame.level as usize].push(record.clone(), capacity, message_limit);
        let capacity = usize::try_from(SERVER_LOG_PENDING_FRAMES).expect("validated pending log count");
        self.live.push(record, capacity, message_limit);
        // No Receiver Is Not An Error: The Record Still Serves Later Replays.
        // Notifications Retain Only Sequence Numbers; Live Payloads Share One Byte-Bounded Ring.
        let _ = self.sender.send(self.sequence);
    }

    // The Newest Matching Records Up To The Requested Count; Those Over The Message Budget Count As Dropped.
    fn replay(&self, min_level: LogLevel, replay_lines: usize, message_limit: usize) -> (VecDeque<Arc<Record>>, u64) {
        let rings = self.rings.iter().skip(min_level as usize);
        let mut records = rings.flat_map(|ring| ring.records.iter().cloned()).collect::<Vec<_>>();
        records.sort_unstable_by_key(|record| record.sequence);
        let mut replay = records.into_iter().rev().take(replay_lines).collect::<VecDeque<_>>();
        let mut dropped = 0;
        let mut bytes = 0usize;
        let mut kept = VecDeque::with_capacity(replay.len());
        while let Some(record) = replay.pop_front() {
            if bytes.saturating_add(record.bytes) > message_limit && !kept.is_empty() {
                dropped += 1 + replay.len() as u64;
                break;
            }
            bytes = bytes.saturating_add(record.bytes);
            kept.push_front(record);
        }
        (kept, dropped)
    }
}

impl Ring {
    fn push(&mut self, record: Arc<Record>, capacity: usize, message_limit: usize) {
        while !self.records.is_empty() && (self.records.len() >= capacity || self.bytes.saturating_add(record.bytes) > message_limit) {
            self.pop();
        }
        self.bytes = self.bytes.saturating_add(record.bytes);
        self.records.push_back(record);
    }

    fn get(&self, sequence: u64) -> Option<Arc<Record>> {
        // Live Sequence Numbers Are Contiguous, So Lookup Never Scans The Ring.
        let first = self.records.front()?.sequence;
        let offset = sequence.checked_sub(first)?;
        let index = usize::try_from(offset).ok()?;
        self.records.get(index).cloned()
    }

    fn pop(&mut self) {
        if let Some(record) = self.records.pop_front() {
            self.bytes = self.bytes.saturating_sub(record.bytes);
        }
    }
}

impl Subscription {
    // A Keepalive Has No Time, An Unspecified Level And No Text; It Repeats The Cumulative Drop Count.
    fn keepalive(&self) -> Response<LogFrame> {
        LogFrame {
            time: None,
            level: LogLevel::Unspecified,
            target: String::new(),
            message: String::new(),
            fields: Vec::new(),
            dropped: self.dropped,
        }
        .into()
    }

    async fn next(&mut self, min_level: LogLevel, message_limit: usize) -> Result<Option<Response<LogFrame>>, Status> {
        loop {
            let record = match self.replay.pop_front() {
                Some(record) => record,
                None => match self.live.recv().await {
                    Ok(sequence) => {
                        let record = self.tail.lock().live.get(sequence);
                        let Some(record) = record else {
                            self.dropped = self.dropped.saturating_add(1);
                            continue;
                        };
                        record
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        self.dropped = self.dropped.saturating_add(skipped);
                        continue;
                    }
                    Err(broadcast::error::RecvError::Closed) => return Ok(None),
                },
            };
            if (record.frame.level as i32) < min_level as i32 {
                continue;
            }
            if record.bytes > message_limit {
                return Err(Status::resource_exhausted("log frame exceeds the configured message budget"));
            }
            let frame = LogFrame {
                dropped: self.dropped,
                ..record.frame.clone()
            };
            return Ok(Some(frame.into()));
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn frame_wire(line: &[u8]) -> Option<LogFrame> {
    let line = std::str::from_utf8(line).ok()?.trim();
    if line.is_empty() {
        return None;
    }
    let record: Value = serde_json::from_str(line).ok()?;
    let record = record.as_object()?;
    let level = level(record.get("level").or_else(|| record.get("severity"))?.as_str()?)?;

    let nested = record.get("fields").and_then(Value::as_object);
    let message =
        nested.and_then(|fields| fields.get("message")).or_else(|| record.get("message")).and_then(Value::as_str).unwrap_or_default();
    let mut fields = std::collections::BTreeMap::new();
    for (name, value) in nested.into_iter().flatten() {
        if name == "message" {
            continue;
        }
        fields.insert(name.clone(), field(name, value));
    }
    if let Some(spans) = record.get("spans").filter(|spans| !spans.is_null()) {
        fields.insert("spans".to_owned(), field("spans", spans));
    }

    Some(LogFrame {
        time: record.get("timestamp").or_else(|| record.get("time")).and_then(Value::as_str).and_then(timestamp),
        level,
        target: record.get("target").and_then(Value::as_str).unwrap_or_default().to_owned(),
        message: redaction::text(message),
        fields: fields.into_iter().map(|(key, value)| LogFrameFieldsEntry { key, value }).collect(),
        dropped: 0,
    })
}

fn level(name: &str) -> Option<LogLevel> {
    match name.trim().to_ascii_uppercase().as_str() {
        "TRACE" => Some(LogLevel::Trace),
        "DEBUG" => Some(LogLevel::Debug),
        "INFO" => Some(LogLevel::Info),
        "WARN" | "WARNING" => Some(LogLevel::Warn),
        "ERROR" => Some(LogLevel::Error),
        _ => None,
    }
}

fn field(name: &str, value: &Value) -> String {
    if redaction::is_secret_name(name) {
        return redaction::REDACTED.to_owned();
    }
    match value {
        Value::String(text) => redaction::text(text),
        other => other.to_string(),
    }
}

fn timestamp(value: &str) -> Option<Timestamp> {
    let moment = time::OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339).ok()?;
    Some(Timestamp {
        seconds: moment.unix_timestamp(),
        nanos: moment.nanosecond() as i32,
    })
}

fn memory_failure(source: dokime::harmos::operations::StreamError) -> Status {
    use dokime::harmos::operations::StreamError;
    let status = if matches!(source, StreamError::Capacity | StreamError::Construction) {
        Status::resource_exhausted("log frame admission failed")
    } else {
        Status::internal("log frame admission failed")
    };
    crate::grpc::error::detailed_error(status, &source)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;

    fn line(level: &str, message: &str) -> String {
        format!(r#"{{"level":"{level}","target":"dokime::fixture","fields":{{"message":"{message}"}}}}"#)
    }

    async fn caught_up(tail: &Arc<Tail>) -> u64 {
        let mut reader = tail.reader.lock().await;
        while tail.advance(&mut reader, None).await > 0 {}
        reader.bytes_read
    }

    #[tokio::test]
    async fn tail_reads_only_appended_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("dokime.jsonl");
        let first = format!("{}\n{}\n", line("INFO", "started"), line("WARN", "slow"));
        std::fs::write(&path, &first).unwrap();
        let tail = Tail::new(path.clone(), Duration::from_millis(10), 1 << 20);
        assert_eq!(caught_up(&tail).await, first.len() as u64);
        let mut total = first.len() as u64;
        for index in 0..32 {
            let appended = format!("{}\n", line("ERROR", &format!("failed {index}")));
            std::fs::OpenOptions::new().append(true).open(&path).unwrap().write_all(appended.as_bytes()).unwrap();
            total += appended.len() as u64;
            // Each Poll Reads Exactly The Appended Record, Never The File Again.
            assert_eq!(caught_up(&tail).await, total);
        }
        assert_eq!(tail.lock().sequence, 34);
    }

    #[tokio::test]
    async fn subscribers_share_one_parsed_record() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("dokime.jsonl");
        std::fs::write(&path, format!("{}\n{}\n", line("INFO", "started"), line("WARN", "slow"))).unwrap();
        let tail = Tail::new(path.clone(), Duration::from_millis(10), 1 << 20);
        let warn = tail.subscribe(LogLevel::Warn, 500).await.unwrap();
        let all = tail.subscribe(LogLevel::Trace, 500).await.unwrap();
        assert_eq!(warn.replay.len(), 1);
        assert_eq!(all.replay.len(), 2);
        assert!(Arc::ptr_eq(&warn.replay[0], &all.replay[1]), "one parse serves every subscriber");
        // The Second Subscriber Found The Reader Caught Up, So The File Was Read Once.
        let read = std::fs::metadata(&path).unwrap().len();
        assert_eq!(tail.reader.lock().await.bytes_read, read);
    }

    #[tokio::test]
    async fn partial_lines_resume_without_rereading_or_duplicate_records() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("dokime.jsonl");
        let record = line("INFO", "completed");
        let split = record.len() / 2;
        tokio::fs::write(&path, &record[..split]).await.unwrap();
        let tail = Tail::new(path.clone(), Duration::from_millis(10), 1 << 20);
        assert_eq!(caught_up(&tail).await, split as u64);
        assert_eq!(tail.lock().sequence, 0);

        let mut file = tokio::fs::OpenOptions::new().append(true).open(&path).await.unwrap();
        let appended = format!("{}\n", &record[split..]);
        tokio::io::AsyncWriteExt::write_all(&mut file, appended.as_bytes()).await.unwrap();
        file.sync_all().await.unwrap();
        assert_eq!(caught_up(&tail).await, record.len() as u64 + 1);
        assert_eq!(caught_up(&tail).await, record.len() as u64 + 1);
        assert_eq!(tail.lock().sequence, 1);
        assert!(tail.reader.lock().await.carry.is_empty());
    }

    #[tokio::test]
    async fn subscriber_replays_only_records_from_the_truncated_file() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("dokime.jsonl");
        tokio::fs::write(&path, format!("{}\n", line("INFO", "old record"))).await.unwrap();
        let tail = Tail::new(path.clone(), Duration::from_millis(10), 1 << 20);
        caught_up(&tail).await;
        tokio::fs::write(&path, format!("{}\n", line("INFO", "new"))).await.unwrap();

        let subscription = tail.subscribe(LogLevel::Trace, 500).await.unwrap();
        assert_eq!(subscription.replay.len(), 1);
        assert_eq!(subscription.replay[0].frame.message, "new");
    }

    #[tokio::test]
    async fn slow_subscriber_reports_frames_evicted_by_the_live_byte_budget() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("dokime.jsonl");
        let tail = Tail::new(path, Duration::from_millis(10), 1 << 20);
        let mut subscription = tail.subscribe(LogLevel::Trace, 500).await.unwrap();
        let frame = frame_wire(line("INFO", "live").as_bytes()).unwrap();
        let bytes = super::super::content::output_frame_len(&frame).unwrap();
        for _ in 0..3 {
            tail.lock().publish(frame.clone(), bytes, bytes * 2);
        }

        let delivered = subscription.next(LogLevel::Trace, 1 << 20).await.unwrap().unwrap();
        assert_eq!(delivered.dropped, 1);
        assert_eq!(tail.lock().live.bytes, bytes * 2);
        assert_eq!(tail.lock().live.records.len(), 2);
    }

    #[test]
    fn oversized_partial_line_is_bounded_and_does_not_hide_the_next_record() {
        let mut reader = Reader::default();
        let window = vec![b'x'; SERVER_LOG_READ_WINDOW_BYTES as usize];
        assert!(reader.complete_lines(&window).is_empty());
        assert!(reader.complete_lines(&window).is_empty());
        assert!(reader.carry.is_empty());
        assert!(reader.discarding_line);

        let suffix = format!("ignored\n{}\n", line("WARN", "next"));
        let complete = reader.complete_lines(suffix.as_bytes());
        let frames = complete.split(|byte| *byte == b'\n').filter_map(frame_wire).collect::<Vec<_>>();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].message, "next");
        assert!(reader.carry.is_empty());
        assert!(!reader.discarding_line);
    }

    #[test]
    fn replay_rings_stay_bounded() {
        let mut shared = Shared {
            sequence: 0,
            rings: Default::default(),
            live: Ring::default(),
            sender: broadcast::channel(1).0,
            polling: false,
        };
        let frame = frame_wire(line("INFO", "started").as_bytes()).unwrap();
        let records = SERVER_MAXIMUM_REPLAY_LINES * 3;
        for _ in 0..records {
            shared.publish(frame.clone(), 10, usize::MAX);
        }
        assert_eq!(shared.rings[LogLevel::Info as usize].records.len() as u64, SERVER_MAXIMUM_REPLAY_LINES);
        for _ in 0..records {
            shared.publish(frame.clone(), 10, 1_000);
        }
        assert_eq!(shared.rings[LogLevel::Info as usize].bytes, 1_000);
        assert_eq!(shared.live.bytes, 1_000);
        assert_eq!(shared.live.records.len(), 100);
        let (replay, dropped) = shared.replay(LogLevel::Trace, 500, 500);
        assert_eq!((replay.len(), dropped), (50, 50));
        assert_eq!(replay.back().unwrap().sequence, records * 2);
    }
}
