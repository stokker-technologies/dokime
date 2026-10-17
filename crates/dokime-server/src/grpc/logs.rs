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
use harmos::{
    MemorySize, Protobuf,
    operations::{Response, StreamOutput},
};
use serde_json::Value;
use tonic::Status;
// Local Imports
use super::redaction;
use dokime_domain::operations::backend as wire;
use dokime_domain::model::identity::wire::Timestamp;

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
    pub(super) min_level: dokime_domain::model::backend::wire::LogLevel,
    pub(super) replay_lines: u32,
    pub(super) message_limit: usize,
}

struct Follow {
    pending_bytes: usize,
    path: PathBuf,
    min_level: dokime_domain::model::backend::wire::LogLevel,
    replay_lines: u32,
    message_limit: usize,
    offset: u64,
    carry: Vec<u8>,
    discarding_line: bool,
    pending: VecDeque<dokime_domain::model::backend::wire::LogFrame>,
    dropped: u64,
    replayed: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Watch {
    pub(super) async fn run(self, output: &mut StreamOutput<dokime_domain::model::backend::wire::LogFrame>) -> Result<(), Status> {
        let replay_lines = if self.replay_lines == 0 {
            DEFAULT_REPLAY_LINES
        } else {
            self.replay_lines.min(MAX_REPLAY_LINES)
        };
        let mut follow = Follow {
            pending_bytes: 0,
            path: self.path,
            min_level: self.min_level,
            replay_lines,
            message_limit: self.message_limit,
            offset: 0,
            carry: Vec::new(),
            discarding_line: false,
            pending: VecDeque::new(),
            dropped: 0,
            replayed: false,
        };
        while let Some(frame) = follow.next().await? {
            output.send_owned(frame).await.map_err(memory_failure)?;
        }
        Ok(())
    }
}

impl Follow {
    async fn next(&mut self) -> Result<Option<Response<dokime_domain::model::backend::wire::LogFrame>>, Status> {
        loop {
            if !self.replayed {
                self.replay().await?;
                self.replayed = true;
                continue;
            }
            if let Some(mut frame) = self.pop() {
                frame.dropped = self.dropped;
                if frame.encoded_len() > self.message_limit {
                    return Err(Status::resource_exhausted(
                        "log frame exceeds the configured message budget",
                    ));
                }
                return Ok(Some(frame.into()));
            }
            if self.advance(None)? == 0 {
                tokio::time::sleep(POLL_INTERVAL).await;
            }
        }
    }

    async fn replay(&mut self) -> Result<(), Status> {
        let Ok(metadata) = std::fs::metadata(&self.path) else {
            return Ok(());
        };
        let end = metadata.len();
        while self.offset < end {
            if self.advance(Some(end))? == 0 {
                break;
            }
            // Give Shared Cancellation A Boundary Between Fixed Read Windows.
            tokio::task::yield_now().await;
        }
        // A Partial Last Record Belongs To Follow, Not To The Replay Snapshot.
        self.offset = self.offset.saturating_sub(self.carry.len() as u64);
        self.carry.clear();
        Ok(())
    }

    fn advance(&mut self, end: Option<u64>) -> Result<u64, Status> {
        let Ok(metadata) = std::fs::metadata(&self.path) else {
            return Ok(0);
        };
        let length = metadata.len().min(end.unwrap_or(u64::MAX));
        if length < self.offset {
            self.offset = 0;
            self.carry.clear();
            self.discarding_line = false;
        }
        if length == self.offset {
            return Ok(0);
        }

        use std::io::{Read as _, Seek as _, SeekFrom};
        let Ok(mut file) = std::fs::File::open(&self.path) else {
            return Ok(0);
        };
        if file.seek(SeekFrom::Start(self.offset)).is_err() {
            return Ok(0);
        }
        let window = (length - self.offset).min(READ_WINDOW_BYTES);
        let mut bytes = vec![0; usize::try_from(window).unwrap_or_default()];
        let Ok(read) = file.read(&mut bytes) else {
            return Ok(0);
        };
        bytes.truncate(read);
        self.offset += read as u64;

        self.carry.extend_from_slice(&bytes);
        if self.discarding_line {
            let Some(boundary) = self.carry.iter().position(|byte| *byte == b'\n') else {
                self.carry.clear();
                return Ok(read as u64);
            };
            self.carry.drain(..=boundary);
            self.discarding_line = false;
        }
        // Everything up to the last newline is whole records; the remainder is a
        // record still being written, unless it has grown past a plausible one.
        let boundary = self.carry.iter().rposition(|byte| *byte == b'\n');
        let complete = match boundary {
            Some(boundary) => self.carry.drain(..=boundary).collect::<Vec<_>>(),
            None if self.carry.len() > MAX_LINE_BYTES => {
                self.carry.clear();
                self.discarding_line = true;
                Vec::new()
            }
            None => Vec::new(),
        };
        for line in complete.split(|byte| *byte == b'\n') {
            if line.len() > MAX_LINE_BYTES {
                continue;
            }
            if let Some(frame) = self.frame(line)? {
                self.push(frame);
            }
        }
        Ok(read as u64)
    }

    fn pop(&mut self) -> Option<dokime_domain::model::backend::wire::LogFrame> {
        let frame = self.pending.pop_front()?;
        self.pending_bytes = self.pending_bytes.saturating_sub(frame.memory_size());
        Some(frame)
    }
    fn push(&mut self, frame: dokime_domain::model::backend::wire::LogFrame) {
        let bytes = frame.memory_size();
        let count = if self.replayed {
            MAX_PENDING_FRAMES
        } else {
            self.replay_lines as usize
        };
        while !self.pending.is_empty()
            && (self.pending.len() >= count
                || self.pending_bytes.saturating_add(bytes) > self.message_limit)
        {
            let count_drop =
                self.replayed || self.pending_bytes.saturating_add(bytes) > self.message_limit;
            self.pop();
            if count_drop {
                self.dropped = self.dropped.saturating_add(1);
            }
        }
        self.pending_bytes = self.pending_bytes.saturating_add(bytes);
        self.pending.push_back(frame);
    }
    fn frame(&self, line: &[u8]) -> Result<Option<dokime_domain::model::backend::wire::LogFrame>, Status> {
        let Some(frame) = self.frame_wire(line) else {
            return Ok(None);
        };
        if frame.memory_size() > self.message_limit || frame.encoded_len() > self.message_limit {
            return Err(Status::resource_exhausted(
                "log frame exceeds the configured message budget",
            ));
        }
        Ok(Some(frame))
    }
    fn frame_wire(&self, line: &[u8]) -> Option<dokime_domain::model::backend::wire::LogFrame> {
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
        if (level as i32) < self.min_level as i32 {
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

        Some(dokime_domain::model::backend::wire::LogFrame {
            time: record
                .get("timestamp")
                .or_else(|| record.get("time"))
                .and_then(Value::as_str)
                .and_then(timestamp),
            level,
            target: record
                .get("target")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            message: redaction::text(message),
            fields: fields
                .into_iter()
                .map(|(key, value)| dokime_domain::model::backend::wire::LogFrameFieldsEntry { key, value })
                .collect(),
            dropped: 0,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn level(name: &str) -> Option<dokime_domain::model::backend::wire::LogLevel> {
    match name.trim().to_ascii_uppercase().as_str() {
        "TRACE" => Some(dokime_domain::model::backend::wire::LogLevel::Trace),
        "DEBUG" => Some(dokime_domain::model::backend::wire::LogLevel::Debug),
        "INFO" => Some(dokime_domain::model::backend::wire::LogLevel::Info),
        "WARN" | "WARNING" => Some(dokime_domain::model::backend::wire::LogLevel::Warn),
        "ERROR" => Some(dokime_domain::model::backend::wire::LogLevel::Error),
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
    let moment =
        time::OffsetDateTime::parse(value, &time::format_description::well_known::Rfc3339).ok()?;
    Some(Timestamp {
        seconds: moment.unix_timestamp(),
        nanos: moment.nanosecond() as i32,
    })
}

fn memory_failure(source: harmos::Error) -> Status {
    let status = if matches!(source, harmos::Error::ResourceExhausted { .. }) {
        Status::resource_exhausted("log frame admission failed")
    } else {
        Status::internal("log frame admission failed")
    };
    crate::grpc::error::detailed(status, &source.into())
}
/* === Tests === */
