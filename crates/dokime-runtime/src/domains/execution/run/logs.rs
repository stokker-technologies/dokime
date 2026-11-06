/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Persist Sidecar Diagnostics In Bounded Ordered Segments.

/* === Modules === */

#[cfg(feature = "testing")]
pub(crate) mod testing;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports

// Local Imports
use super::{ExecutionFixtureContext, TestExecution, artifacts::DiscardedArtifact};

/* === Constants === */

const SEGMENT_BYTES: u64 = 64 * 1024 * 1024;
const EXECUTION_BYTES: u64 = 1024 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
pub(super) struct StreamLog {
    failure: Option<String>,
    segment: u32,
    segment_bytes: u64,
    total_bytes: u64,
}

struct StreamLogWriter<'a> {
    storage: &'a Arc<crate::runtime::storage::executions::ExecutionStorage>,
    execution: &'a dokime_domain::model::identity::TestExecutionId,
    path: String,
    capacity: usize,
    buffer: Option<Vec<u8>>,
    log: &'a mut StreamLog,
}

/* === Enums === */

#[derive(Clone, Copy, Debug)]
pub(super) enum DropReason {
    Abandoned,
    Incomplete,
}

/* === Traits === */

/* === Implementations === */

impl DropReason {
    fn trace(self) -> &'static str {
        match self {
            Self::Abandoned => "dropped an execution artifact its producer abandoned",
            Self::Incomplete => "discarded an execution artifact that never completed",
        }
    }
}

impl StreamLog {
    pub(super) async fn append(
        &mut self, context: &ExecutionFixtureContext, execution: &TestExecution, source: &str, entries: &[crate::execution::log::Entry],
    ) {
        if entries.is_empty() {
            return;
        }
        use crate::execution::log::Level;
        for entry in entries {
            macro_rules! mirror {
                ($level:expr) => {
                    tracing::event!(target: "sidecar", $level,
                        plugin_id = source,
                        execution_id = %execution.test_execution_id,
                        cycle_id = %execution.test_cycle_id,
                        line = %entry.message,
                        timestamp = %entry.timestamp,
                        "sidecar execution log"
                    )
                };
            }
            match Level::try_from(entry.level) {
                Ok(Level::Trace) => mirror!(tracing::Level::TRACE),
                Ok(Level::Debug) => mirror!(tracing::Level::DEBUG),
                Ok(Level::Info) => mirror!(tracing::Level::INFO),
                Ok(Level::Warn) => mirror!(tracing::Level::WARN),
                Ok(Level::Error) => mirror!(tracing::Level::ERROR),
                Err(_) => tracing::warn!(
                    plugin_id = source,
                    execution_id = %execution.test_execution_id,
                    cycle_id = %execution.test_cycle_id,
                    level = entry.level,
                    line = %entry.message,
                    "sidecar execution diagnostic has unknown severity"
                ),
            }
        }
        if self.failure.is_some() {
            return;
        }
        let persisted = async {
            let mut writer = StreamLogWriter::new(context, execution, self)?;
            writer.storage.reserve_output(writer.execution, &writer.path).await?;
            for entry in entries {
                writer.entry(entry).await?;
            }
            writer.flush().await
        }
        .await;
        if let Err(error) = persisted {
            self.failure = Some(error.to_string());
            tracing::warn!(plugin_id = source, %error, "failed to persist sidecar stream log");
        }
    }
}

impl StreamLog {
    pub(super) fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }
}

impl<'a> StreamLogWriter<'a> {
    fn new(context: &'a ExecutionFixtureContext, execution: &'a TestExecution, log: &'a mut StreamLog) -> crate::runtime::Result<Self> {
        let storage = &context.runtime.execution_storage;
        let capacity = 64 * 1024;

        let path = segment_path(log.segment);
        Ok(Self {
            storage,
            execution: &execution.test_execution_id,
            path,
            capacity,
            buffer: None,
            log,
        })
    }

    async fn entry(&mut self, entry: &crate::execution::log::Entry) -> crate::runtime::Result<()> {
        use crate::execution::log::Level;
        use std::io::Write;

        // UNKNOWN() Plus The Maximum Signed Decimal i32 Representation.
        let mut unknown = [0; "UNKNOWN()".len() + i32::BITS as usize / 3 + 1];
        let level = match Level::try_from(entry.level) {
            Ok(Level::Trace) => "TRACE",
            Ok(Level::Debug) => "DEBUG",
            Ok(Level::Info) => "INFO",
            Ok(Level::Warn) => "WARN",
            Ok(Level::Error) => "ERROR",
            Err(_) => {
                let mut output = std::io::Cursor::new(unknown.as_mut_slice());
                write!(output, "UNKNOWN({})", entry.level)?;
                let length = output.position() as usize;
                std::str::from_utf8(&unknown[..length]).expect("ASCII severity")
            }
        };
        self.message(&entry.timestamp, level, &[entry.message.as_str()]).await
    }

    async fn artifact(&mut self, artifact: &DiscardedArtifact, reason: DropReason) -> crate::runtime::Result<()> {
        use std::io::Write;

        let mut count = [0; u64::MAX.ilog10() as usize + 1];
        let mut output = std::io::Cursor::new(count.as_mut_slice());
        write!(output, "{}", artifact.received)?;
        let length = output.position() as usize;
        let received = std::str::from_utf8(&count[..length]).expect("ASCII byte count");
        let fragments = match reason {
            DropReason::Abandoned => {
                ["dropped execution artifact `", artifact.name.as_str(), "`: the producer aborted the transfer after ", received, " bytes"]
            }
            DropReason::Incomplete => [
                "discarded incomplete execution artifact `",
                artifact.name.as_str(),
                "`: ",
                received,
                " bytes arrived before the execution ended",
            ],
        };
        self.message("", "WARN", &fragments).await
    }

    async fn message(&mut self, timestamp: &str, level: &str, fragments: &[&str]) -> crate::runtime::Result<()> {
        self.prefix(timestamp, level).await?;
        for fragment in fragments {
            self.fragment(timestamp, level, fragment).await?;
        }
        self.push(b"\n").await
    }

    async fn fragment(&mut self, timestamp: &str, level: &str, fragment: &str) -> crate::runtime::Result<()> {
        let mut lines = fragment.split('\n');
        self.push(lines.next().expect("split retains an empty part").as_bytes()).await?;
        for line in lines {
            self.push(b"\n").await?;
            self.prefix(timestamp, level).await?;
            self.push(line.as_bytes()).await?;
        }
        Ok(())
    }

    async fn prefix(&mut self, timestamp: &str, level: &str) -> crate::runtime::Result<()> {
        use std::io::Write;

        // The Existing UTC Millisecond Timestamp Has Exactly 24 ASCII Bytes.
        let mut fallback = [0; "0000-00-00T00:00:00.000Z".len()];
        let timestamp = if timestamp.is_empty() {
            let now = time::OffsetDateTime::now_utc();
            let mut output = fallback.as_mut_slice();
            write!(
                output,
                "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
                now.year(),
                u8::from(now.month()),
                now.day(),
                now.hour(),
                now.minute(),
                now.second(),
                now.millisecond()
            )?;
            std::str::from_utf8(&fallback).expect("ASCII UTC timestamp")
        } else {
            timestamp
        };
        for fragment in [timestamp, " [", level, "] "] {
            self.push(fragment.as_bytes()).await?;
        }
        Ok(())
    }

    async fn push(&mut self, mut remaining: &[u8]) -> crate::runtime::Result<()> {
        while !remaining.is_empty() {
            let capacity = self.capacity;
            let bytes = self.writable()?;
            let count = remaining.len().min(capacity - bytes.len());
            bytes.extend_from_slice(&remaining[..count]);
            remaining = &remaining[count..];
            self.flush_full().await?;
        }
        Ok(())
    }

    fn writable(&mut self) -> crate::runtime::Result<&mut Vec<u8>> {
        if self.buffer.is_none() {
            let buffer = self.storage.output_buffer(self.capacity)?;
            self.buffer = Some(buffer);
        }
        let bytes = self.buffer.as_mut().expect("admitted log buffer");
        Ok(bytes)
    }

    async fn flush_full(&mut self) -> crate::runtime::Result<()> {
        let Some(bytes) = self.buffer.as_ref() else {
            return Ok(());
        };
        if bytes.len() != self.capacity {
            return Ok(());
        }
        self.flush().await
    }

    async fn flush(&mut self) -> crate::runtime::Result<()> {
        let Some(bytes) = self.buffer.take() else {
            return Ok(());
        };
        let next = self
            .log
            .total_bytes
            .checked_add(bytes.len() as u64)
            .filter(|next| *next <= EXECUTION_BYTES)
            .ok_or_else(|| log_error("execution log reached its 1 GiB capture limit"))?;
        let mut offset = 0;
        while offset < bytes.len() {
            if self.log.segment_bytes == SEGMENT_BYTES {
                self.storage.close_output(self.execution, &self.path, false).await?;
                let next = self.log.segment.checked_add(1).ok_or_else(|| log_error("log segment identity exhausted"))?;
                self.log.segment = self.storage.reserve_execution_log_segment(self.execution, next).await?;
                self.log.segment_bytes = 0;
                self.path = segment_path(self.log.segment);
            }
            let count = (SEGMENT_BYTES - self.log.segment_bytes).min((bytes.len() - offset) as u64) as usize;
            let page = bytes[offset..offset + count].to_vec();
            self.storage.append_output(self.execution, &self.path, page).await?;
            self.log.segment_bytes += count as u64;
            self.log.total_bytes += count as u64;
            offset += count;
        }
        debug_assert_eq!(self.log.total_bytes, next);
        Ok(())
    }
}

/* === Public Functions === */

pub(super) async fn report_dropped(
    context: &ExecutionFixtureContext, execution: &TestExecution, source: &str, stream_log: &mut StreamLog, dropped: &[DiscardedArtifact],
    reason: DropReason,
) {
    if dropped.is_empty() {
        return;
    }
    for artifact in dropped {
        tracing::warn!(
            plugin_id = source,
            transfer_id = artifact.transfer_id,
            name = artifact.name,
            received_bytes = artifact.received,
            "{}",
            reason.trace()
        );
    }
    if stream_log.failure.is_some() {
        return;
    }
    let persisted = async {
        let mut writer = StreamLogWriter::new(context, execution, stream_log)?;
        writer.storage.reserve_output(writer.execution, &writer.path).await?;
        for artifact in dropped {
            writer.artifact(artifact, reason).await?;
        }
        writer.flush().await
    }
    .await;
    if let Err(error) = persisted {
        stream_log.failure = Some(error.to_string());
        tracing::warn!(plugin_id = source, %error, "failed to persist sidecar stream log");
    }
}

/* === Private Functions === */

fn segment_path(segment: u32) -> String {
    let name = if segment == 0 {
        "execution.log".to_owned()
    } else {
        crate::artifacts::name::collision_name("execution.log", u64::from(segment) + 1)
    };
    format!("artifacts/{name}")
}

fn log_error(error: impl ToString) -> crate::runtime::Error {
    crate::runtime::Error::Operation(error.to_string())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn execution_log_names_rollover_and_capture_limit() {
        let fixture: serde_json::Value =
            serde_json::from_str(include_str!("../../../report/fixtures/execution-report-input.json")).unwrap();
        let mut cycle: dokime_domain::model::execution::result::CycleView =
            serde_json::from_value(fixture["input"]["cycle"].clone()).unwrap();
        cycle.test_executions[0].saved = None;
        let execution = &cycle.test_executions[0];
        let root = tempfile::tempdir().unwrap();
        let mut owner = crate::Runtime::start(crate::RuntimeConfig::new(root.path(), cycle.metadata.clone())).await.unwrap();
        let runtime = owner.handle();
        let storage = &runtime.execution_storage;
        storage.acquire(&cycle, execution).await.unwrap();
        let id = &execution.test_execution_id;
        let path = segment_path(0);
        storage.reserve_output(id, &path).await.unwrap();
        let mut log = StreamLog {
            segment_bytes: SEGMENT_BYTES - 1,
            total_bytes: SEGMENT_BYTES - 1,
            ..Default::default()
        };
        let mut writer = StreamLogWriter {
            storage,
            execution: id,
            path,
            capacity: 4,
            buffer: Some(vec![1, 2, 3]),
            log: &mut log,
        };
        writer.flush().await.unwrap();
        let entries = storage.native_entries(id).await.unwrap();
        assert!(entries.iter().any(|entry| entry.relative_path == "artifacts/execution.log" && entry.size_bytes == 1));
        assert_eq!(writer.path, "artifacts/execution.2.log");
        assert_eq!(storage.log_prefix(id, &writer.path).await.unwrap(), 2);
        assert_eq!(writer.log.segment_bytes, 2);
        storage.native_target(id.clone(), "artifacts/execution.3.log".into(), true, false).await.unwrap();
        writer.log.segment_bytes = SEGMENT_BYTES;
        writer.buffer = Some(vec![4]);
        writer.flush().await.unwrap();
        assert_eq!(writer.path, "artifacts/execution.4.log");
        assert_eq!(storage.log_prefix(id, &writer.path).await.unwrap(), 1);
        writer.log.total_bytes = EXECUTION_BYTES - 1;
        writer.buffer = Some(vec![5]);
        writer.flush().await.unwrap();
        writer.buffer = Some(vec![6]);
        assert!(writer.flush().await.is_err());
        assert_eq!(storage.log_prefix(id, &writer.path).await.unwrap(), 2);
        drop(writer);
        drop(runtime);
        owner.shutdown().await.unwrap();
    }
}
