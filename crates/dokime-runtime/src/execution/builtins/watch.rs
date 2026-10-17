/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// `WatchFile` without a host, as a harness serves it: one local file polled
// in this process. Stream captures only what is written after the start into
// the produced artifact and offers each line to the request's matchers and
// markers; Mirror and the snapshots copy the whole file once. Like the
// host's, a cancelled watch seals what it captured and settles as cancelled,
// without an outcome; only a bounded watch answers with one.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::Duration;

// Third Party Imports
use tokio::sync::watch;
use tokio::time::Instant;

// Local Imports
use super::Binding;
use crate::artifacts::ArtifactWriter;
use dokime_domain::model::identity::Identifier;
use crate::files::{LineBuffer, LineMatch, LineMatchers};
use dokime_domain::model::files::wire::FileWatchGap;
use dokime_domain::model::files::wire::FileWatchGapReason;
use dokime_domain::model::files::wire::FileWatchPolicy;
use dokime_domain::model::files::wire::FileWatchSourcePayload;
use dokime_domain::model::files::wire::WatchFileOutcome;
use dokime_domain::operations::files::WatchFile;
use dokime_domain::model::files::wire::WatchedArtifact;
use crate::records::{RecordPublishRequest, RecordSeverity};

/* === Constants === */
const DEFAULT_POLL: Duration = Duration::from_millis(1000);

const READ_BYTES: usize = 64 * 1024;

const CANCELLED: &str = "the file watch was cancelled; it sealed what it captured";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Tail {
    path: PathBuf,
    artifact: String,
    offset: u64,
    written: u64,
    writer: ArtifactWriter,
    lines: LineBuffer,
    matchers: LineMatchers,
    gaps: Vec<FileWatchGap>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Tail {
    // Captures what the file gained since the last pass.
    async fn pass(&mut self, binding: &Binding) -> Result<(), String> {
        let length = std::fs::metadata(&self.path).map_or(0, |metadata| metadata.len());
        if length < self.offset {
            self.gaps.push(FileWatchGap {
                path: self.artifact.clone(),
                offset: self.written,
                reason: FileWatchGapReason::Truncated,
            });
            self.offset = 0;
            self.lines.reset();
        }
        if length == self.offset {
            return Ok(());
        }
        let bytes = read(&self.path, self.offset, length - self.offset)
            .map_err(|error| format!("{}: {error}", self.path.display()))?;
        self.offset += bytes.len() as u64;
        self.written += bytes.len() as u64;
        self.writer
            .write_all(&bytes)
            .and_then(|()| self.writer.flush())
            .map_err(|error| format!("artifact {:?}: {error}", self.artifact))?;
        let lines = self.lines.push(&bytes);
        publish(binding, &self.matchers, lines).await;
        Ok(())
    }

    async fn finish(mut self, binding: &Binding) -> Result<WatchFileOutcome, String> {
        let last = self.lines.finish();
        publish(binding, &self.matchers, last).await;
        self.writer
            .finish()
            .map_err(|error| format!("artifact {:?}: {error}", self.artifact))?;
        Ok(WatchFileOutcome {
            artifacts: vec![WatchedArtifact {
                artifact: self.artifact,
                size_bytes: self.written,
            }],
            gaps: self.gaps,
        })
    }
}

/* === Public Functions === */
pub(super) async fn run(
    binding: Binding,
    request: WatchFile,
    mut cancel: watch::Receiver<bool>,
) -> Result<WatchFileOutcome, String> {
    // Resolve The File, Its Artifact And Its Matchers
    let path = match request.source.and_then(|source| source.payload) {
        Some(FileWatchSourcePayload::Local(local)) => PathBuf::from(local.root),
        Some(FileWatchSourcePayload::Ftp(_)) => {
            return Err("watching an FTP source needs the Dokime host".into());
        }
        None => return Err("a file watch needs a local or FTP source".into()),
    };
    let artifact = artifact_name(&path, &request.output, request.artifact.as_deref())?;
    let matchers = LineMatchers::compile(&request.matchers, &request.markers)
        .map_err(|error| error.to_string())?;
    let poll = request
        .poll_interval_ms
        .map_or(DEFAULT_POLL, |milliseconds| {
            Duration::from_millis(milliseconds.into())
        });
    let bound = request
        .maximum_seconds
        .map(|seconds| Duration::from_secs(seconds.into()));
    let started = Instant::now();
    let policy = request.policy;
    if !matches!(policy, FileWatchPolicy::Stream) {
        if policy == FileWatchPolicy::Unspecified {
            return Err("a file watch needs a policy".into());
        }
        let cancelled =
            policy != FileWatchPolicy::SnapshotAtStart && wait(&mut cancel, bound, started).await;
        let outcome = snapshot(&binding, &path, artifact)?;
        return if cancelled {
            Err(CANCELLED.into())
        } else {
            Ok(outcome)
        };
    }

    // Stream From The Length At The Start
    let writer = binding
        .artifacts
        .create(&artifact)
        .map_err(|error| format!("artifact {artifact:?}: {error}"))?;
    let mut tail = Tail {
        offset: std::fs::metadata(&path).map_or(0, |metadata| metadata.len()),
        path,
        artifact,
        written: 0,
        writer,
        lines: LineBuffer::default(),
        matchers,
        gaps: Vec::new(),
    };
    let cancelled = loop {
        tail.pass(&binding).await?;
        let pause = bound.map_or(poll, |bound| {
            bound.saturating_sub(started.elapsed()).min(poll)
        });
        if bound.is_some_and(|bound| started.elapsed() >= bound) {
            break false;
        }
        tokio::select! {
            _ = cancel.wait_for(|cancelled| *cancelled) => break true,
            () = tokio::time::sleep(pause) => {}
        }
    };
    tail.pass(&binding).await?;
    let outcome = tail.finish(&binding).await?;
    if cancelled {
        return Err(CANCELLED.into());
    }
    Ok(outcome)
}

/* === Private Functions === */
fn artifact_name(path: &Path, output: &str, artifact: Option<&str>) -> Result<String, String> {
    if let Some(artifact) = artifact {
        return Ok(artifact.to_owned());
    }
    let file = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| format!("{} names no file", path.display()))?;
    Ok(if output.is_empty() {
        file.to_owned()
    } else {
        format!("{output}/{file}")
    })
}

// Whether the watch was cancelled rather than bounded.
async fn wait(
    cancel: &mut watch::Receiver<bool>,
    bound: Option<Duration>,
    started: Instant,
) -> bool {
    let until = async {
        match bound {
            Some(bound) => tokio::time::sleep(bound.saturating_sub(started.elapsed())).await,
            None => std::future::pending().await,
        }
    };
    tokio::select! {
        changed = cancel.wait_for(|cancelled| *cancelled) => changed.is_ok(),
        () = until => false,
    }
}

fn snapshot(binding: &Binding, path: &Path, artifact: String) -> Result<WatchFileOutcome, String> {
    let size_bytes = std::fs::metadata(path)
        .map_err(|error| format!("{}: {error}", path.display()))?
        .len();
    binding
        .artifacts
        .import(path, &artifact)
        .map_err(|error| format!("artifact {artifact:?}: {error}"))?;
    Ok(WatchFileOutcome {
        artifacts: vec![WatchedArtifact {
            artifact,
            size_bytes,
        }],
        gaps: Vec::new(),
    })
}

fn read(path: &Path, from: u64, length: u64) -> std::io::Result<Vec<u8>> {
    let mut file = std::fs::File::open(path)?;
    file.seek(SeekFrom::Start(from))?;
    let mut bytes = Vec::with_capacity(usize::try_from(length).unwrap_or(READ_BYTES));
    file.take(length).read_to_end(&mut bytes)?;
    Ok(bytes)
}

// Rows reach the execution's streams; markers its records.
async fn publish(
    binding: &Binding,
    matchers: &LineMatchers,
    lines: impl IntoIterator<Item = String>,
) {
    if matchers.is_empty() {
        return;
    }
    for line in lines {
        for found in matchers.matches(&line) {
            match found {
                LineMatch::Row {
                    stream,
                    values,
                    storage_schema,
                } => match Identifier::new(&stream) {
                    Ok(stream) => match crate::telemetry::capture_payload(&storage_schema, &values)
                    {
                        Ok(payload) => binding.streams.published(&stream, values, payload).await,
                        Err(error) => binding
                            .journal
                            .log(crate::execution::log::Level::Warn, error),
                    },
                    Err(error) => binding.journal.log(
                        crate::execution::log::Level::Warn,
                        format!("matcher stream {stream:?}: {error}"),
                    ),
                },
                LineMatch::Marker {
                    record_type,
                    details,
                    line,
                } => match Identifier::new(&record_type) {
                    Ok(record_type) => {
                        let mut record =
                            RecordPublishRequest::new(record_type, RecordSeverity::Info);
                        record.message = Some(line);
                        record.details = details;
                        binding.journal.record(record);
                    }
                    Err(error) => binding.journal.log(
                        crate::execution::log::Level::Warn,
                        format!("marker record type {record_type:?}: {error}"),
                    ),
                },
            }
        }
    }
}

/* === Tests === */
