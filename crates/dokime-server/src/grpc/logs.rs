/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::collections::VecDeque;
use std::path::PathBuf;
use std::time::Duration;
// Third Party Imports
use futures_util::stream;
use prost::Message as _;
use serde_json::Value;
use tokio::sync::watch;
use tonic::Status;
// Local Imports
use super::chunked::FrameStream;
use super::redaction;
use dokime_protocol::v1;

/* === Constants === */
pub(super) const DEFAULT_REPLAY_LINES: u32 = 500;

pub(super) const MAX_REPLAY_LINES: u32 = 5_000;

const READ_WINDOW_BYTES: u64 = 256 * 1024;

const MAX_PENDING_FRAMES: usize = 8_192;

const POLL_INTERVAL: Duration = Duration::from_millis(250);

const MAX_LINE_BYTES: usize = 256 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Watch {
    pub(super) path: PathBuf,
    pub(super) min_level: i32,
    pub(super) replay_lines: u32,
    pub(super) message_limit: usize,
    pub(super) stopping: watch::Receiver<bool>,
}

struct Follow {
    path: PathBuf,
    min_level: i32,
    replay_lines: u32,
    message_limit: usize,
    stopping: watch::Receiver<bool>,
    offset: u64,
    carry: Vec<u8>,
    pending: VecDeque<v1::LogFrame>,
    dropped: u64,
    replayed: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Watch {
    pub(super) fn stream(self) -> FrameStream<v1::LogFrame> {
        let replay_lines = if self.replay_lines == 0 {
            DEFAULT_REPLAY_LINES
        } else {
            self.replay_lines.min(MAX_REPLAY_LINES)
        };
        let follow = Follow {
            path: self.path,
            min_level: self.min_level,
            replay_lines,
            message_limit: self.message_limit,
            stopping: self.stopping,
            offset: 0,
            carry: Vec::new(),
            pending: VecDeque::new(),
            dropped: 0,
            replayed: false,
        };
        Box::pin(stream::try_unfold(follow, Follow::next))
    }
}

impl Follow {
    async fn next(mut self) -> Result<Option<(v1::LogFrame, Self)>, Status> {
        loop {
            // A draining listener ends the stream rather than failing it: a tail
            // has no completion frame to withhold, so there is nothing partial
            // for a consumer to mistake for a whole transfer.
            if *self.stopping.borrow() {
                return Ok(None);
            }
            if !self.replayed {
                self.replay();
                self.replayed = true;
                continue;
            }
            if let Some(mut frame) = self.pending.pop_front() {
                frame.dropped = self.dropped;
                if frame.encoded_len() > self.message_limit {
                    return Err(Status::resource_exhausted(
                        "log frame exceeds the configured message budget",
                    ));
                }
                return Ok(Some((frame, self)));
            }
            if self.advance() == 0 {
                let mut stopping = self.stopping.clone();
                tokio::select! { biased;
                    _ = stopping.changed() => return Ok(None),
                    () = tokio::time::sleep(POLL_INTERVAL) => {}
                }
            }
        }
    }

    fn replay(&mut self) {
        let Ok(bytes) = std::fs::read(&self.path) else {
            return;
        };
        // Follow from the last record boundary, not from the end: a line still
        // being written belongs to the follow, not to the replay.
        let boundary = bytes
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map_or(0, |boundary| boundary + 1);
        self.offset = boundary as u64;
        let mut frames: VecDeque<v1::LogFrame> = VecDeque::new();
        for line in bytes[..boundary].split(|byte| *byte == b'\n') {
            if let Some(frame) = self.frame(line) {
                if frames.len() == self.replay_lines as usize {
                    frames.pop_front();
                }
                frames.push_back(frame);
            }
        }
        self.pending = frames;
    }

    fn advance(&mut self) -> u64 {
        let Ok(metadata) = std::fs::metadata(&self.path) else {
            return 0;
        };
        let length = metadata.len();
        if length < self.offset {
            self.offset = 0;
            self.carry.clear();
        }
        if length == self.offset {
            return 0;
        }

        use std::io::{Read as _, Seek as _, SeekFrom};
        let Ok(mut file) = std::fs::File::open(&self.path) else {
            return 0;
        };
        if file.seek(SeekFrom::Start(self.offset)).is_err() {
            return 0;
        }
        let window = (length - self.offset).min(READ_WINDOW_BYTES);
        let mut bytes = vec![0; usize::try_from(window).unwrap_or_default()];
        let Ok(read) = file.read(&mut bytes) else {
            return 0;
        };
        bytes.truncate(read);
        self.offset += read as u64;

        self.carry.extend_from_slice(&bytes);
        // Everything up to the last newline is whole records; the remainder is a
        // record still being written, unless it has grown past a plausible one.
        let boundary = self.carry.iter().rposition(|byte| *byte == b'\n');
        let complete = match boundary {
            Some(boundary) => self.carry.drain(..=boundary).collect::<Vec<_>>(),
            None if self.carry.len() > MAX_LINE_BYTES => std::mem::take(&mut self.carry),
            None => Vec::new(),
        };
        for line in complete.split(|byte| *byte == b'\n') {
            if let Some(frame) = self.frame(line) {
                if self.pending.len() == MAX_PENDING_FRAMES {
                    self.pending.pop_front();
                    self.dropped += 1;
                }
                self.pending.push_back(frame);
            }
        }
        read as u64
    }

    fn frame(&self, line: &[u8]) -> Option<v1::LogFrame> {
        let line = std::str::from_utf8(line).ok()?.trim();
        if line.is_empty() {
            return None;
        }
        let record: Value = serde_json::from_str(line).ok()?;
        let record = record.as_object()?;
        let level = level(
            record
                .get("level")
                .or_else(|| record.get("severity"))?
                .as_str()?,
        )?;
        if (level as i32) < self.min_level.max(v1::LogLevel::Trace as i32) {
            return None;
        }

        let nested = record.get("fields").and_then(Value::as_object);
        let message = nested
            .and_then(|fields| fields.get("message"))
            .or_else(|| record.get("message"))
            .and_then(Value::as_str)
            .unwrap_or_default();
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

        Some(v1::LogFrame {
            time: record
                .get("timestamp")
                .or_else(|| record.get("time"))
                .and_then(Value::as_str)
                .and_then(timestamp),
            level: level as i32,
            target: record
                .get("target")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            message: redaction::text(message),
            fields,
            dropped: 0,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn level(name: &str) -> Option<v1::LogLevel> {
    match name.trim().to_ascii_uppercase().as_str() {
        "TRACE" => Some(v1::LogLevel::Trace),
        "DEBUG" => Some(v1::LogLevel::Debug),
        "INFO" => Some(v1::LogLevel::Info),
        "WARN" | "WARNING" => Some(v1::LogLevel::Warn),
        "ERROR" => Some(v1::LogLevel::Error),
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

fn timestamp(value: &str) -> Option<prost_types::Timestamp> {
    let moment =
        time::OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339).ok()?;
    Some(prost_types::Timestamp {
        seconds: moment.unix_timestamp(),
        nanos: moment.nanosecond() as i32,
    })
}

/* === Tests === */
