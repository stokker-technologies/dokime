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
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

// Third Party Imports
use tokio::sync::watch;
use tokio::time::Instant;

// Local Imports
use super::Binding;
use crate::files::{LineBuffer, LineMatch, LineMatchers};
use crate::records::RecordSeverity;
use dokime_domain::model::files::wire::FileWatchGap;
use dokime_domain::model::files::wire::FileWatchGapReason;
use dokime_domain::model::files::wire::FileWatchPolicy;
use dokime_domain::model::files::wire::FileWatchSourcePayload;
use dokime_domain::model::files::wire::WatchFileOutcome;
use dokime_domain::model::files::wire::WatchedArtifact;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::records::publication::RecordPublishRequest;
use dokime_domain::operations::files::WatchFile;

/* === Constants === */

const CANCELLED: &str = "the file watch was cancelled; it sealed what it captured";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Tail {
    path: PathBuf,
    artifact: String,
    offset: u64,
    written: u64,
    spool: tempfile::NamedTempFile,
    active: bool,
    mirror: bool,
    modified: Option<std::time::SystemTime>,
    lines: LineBuffer,
    matchers: LineMatchers,
    gaps: Vec<FileWatchGap>,
    identity: Option<(u64, u64)>,
    artifacts: Vec<WatchedArtifact>,
    base: String,
    maximum_bytes: u64,
    captured: u64,
    read_bytes: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Tail {
    // Captures what the file gained since the last pass.
    async fn pass(&mut self, binding: &Binding) -> Result<(), String> {
        let metadata = match std::fs::metadata(&self.path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                if self.active {
                    self.gap(FileWatchGapReason::Vanished);
                    self.seal(binding).await?;
                }
                return Ok(());
            }
            Err(error) => return Err(format!("{}: {error}", self.path.display())),
        };
        let identity = crate::files::identity(&self.path, &metadata);
        let replaced = self.identity.is_some() && self.identity != identity;
        let truncated = metadata.len() < self.offset;
        if self.active && (replaced || truncated) {
            self.gap(if replaced { FileWatchGapReason::Replaced } else { FileWatchGapReason::Truncated });
            self.seal(binding).await?;
        }
        if !self.active {
            self.artifact = crate::files::segment_name(&self.base, self.artifacts.len() as u64 + 1);
            self.spool = tempfile::NamedTempFile::new().map_err(|error| error.to_string())?;
            self.offset = 0;
            self.written = 0;
            self.lines.reset();
            self.modified = None;
            self.active = true;
        }
        self.identity = identity;
        let length = metadata.len();
        if self.mirror && self.offset == length && self.modified == metadata.modified().ok() {
            return Ok(());
        }
        if self.mirror {
            let retained = self.captured - self.written;
            if retained.saturating_add(length) > self.maximum_bytes {
                return Err(format!("file watch exceeds its {} byte capture cap", self.maximum_bytes));
            }
            self.spool.as_file_mut().set_len(0).map_err(|error| error.to_string())?;
            self.spool.as_file_mut().seek(SeekFrom::Start(0)).map_err(|error| error.to_string())?;
            self.offset = 0;
            self.written = 0;
            self.captured = retained;
        }
        self.modified = metadata.modified().ok();
        if length == self.offset {
            return Ok(());
        }
        if self.captured.saturating_add(length - self.offset) > self.maximum_bytes {
            return Err(format!("file watch exceeds its {} byte capture cap", self.maximum_bytes));
        }
        let mut file = std::fs::File::open(&self.path).map_err(|error| error.to_string())?;
        file.seek(SeekFrom::Start(self.offset)).map_err(|error| error.to_string())?;
        let mut source = file.take(length - self.offset);
        let mut bytes = vec![0; self.read_bytes];
        loop {
            let count = source.read(&mut bytes).map_err(|error| error.to_string())?;
            if count == 0 {
                break;
            }
            self.spool.write_all(&bytes[..count]).map_err(|error| error.to_string())?;
            self.offset += count as u64;
            self.written += count as u64;
            self.captured += count as u64;
            if !self.mirror {
                let lines = self.lines.push(&bytes[..count]);
                publish(binding, &self.matchers, lines).await;
            }
        }
        Ok(())
    }

    async fn finish(mut self, binding: &Binding) -> Result<WatchFileOutcome, String> {
        self.seal(binding).await?;
        Ok(WatchFileOutcome { artifacts: self.artifacts, gaps: self.gaps })
    }

    async fn seal(&mut self, binding: &Binding) -> Result<(), String> {
        if !self.active {
            return Ok(());
        }
        let last = self.lines.finish();
        publish(binding, &self.matchers, last).await;
        self.spool.flush().map_err(|error| error.to_string())?;
        binding.artifacts.import(self.spool.path(), &self.artifact).map_err(|error| error.to_string())?;
        self.artifacts.push(WatchedArtifact { artifact: self.artifact.clone(), size_bytes: self.written });
        self.active = false;
        Ok(())
    }

    fn gap(&mut self, reason: FileWatchGapReason) {
        let path = self.path.file_name().expect("watch source names a file").to_string_lossy().into_owned();
        self.gaps.push(FileWatchGap { path, offset: self.written, reason });
    }
}

/* === Public Functions === */

pub(super) async fn run(
    binding: Binding, request: WatchFile, mut cancel: watch::Receiver<bool>, ready: super::Ready,
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
    dokime_domain::model::artifacts::name::validate(&artifact).map_err(|error| error.to_string())?;
    crate::artifacts::stream::inferred_mime(&artifact).map_err(|error| error.to_string())?;
    if !path.is_absolute() {
        return Err("a local file watch source must be absolute".into());
    }
    let matchers = LineMatchers::compile(&request.matchers, &request.markers, 32, 1024, 1 << 20, &Default::default())
        .map_err(|error| error.to_string())?;
    let policy = request.policy;
    if policy == FileWatchPolicy::Unspecified {
        return Err("a file watch needs a policy".into());
    }
    if !matchers.is_empty() && policy != FileWatchPolicy::Stream {
        return Err("line matchers and markers need the Stream policy".into());
    }
    let poll = match request.poll_interval_ms {
        Some(milliseconds) => Duration::from_millis(milliseconds.into()),
        None => Duration::from_secs(1),
    };
    let bound = request.maximum_seconds.map(|seconds| Duration::from_secs(seconds.into()));
    let valid_poll = (Duration::from_millis(50)..=Duration::from_millis(60_000)).contains(&poll);
    let valid_bound = bound.is_none_or(|duration| !duration.is_zero() && duration <= Duration::from_millis(86_400_000));
    let maximum_bytes = request.maximum_bytes.unwrap_or(256 * 1024 * 1024);
    if !valid_poll || !valid_bound || maximum_bytes == 0 || maximum_bytes > 4 * 1024 * 1024 * 1024 {
        return Err("invalid file watch poll interval, duration or byte cap".into());
    }
    let source_started = Instant::now();
    let source_wait = request.wait_for_source_seconds.map(|seconds| Duration::from_secs(seconds.into()));
    if source_wait.is_some_and(|duration| duration.is_zero() || duration > Duration::from_millis(86_400_000)) {
        return Err("source wait exceeds its configured duration bound".into());
    }
    let metadata = loop {
        if source_wait.is_some_and(|wait| source_started.elapsed() >= wait) {
            return Err("source wait deadline elapsed before the file appeared".into());
        }
        match std::fs::metadata(&path) {
            Ok(metadata) if metadata.is_file() => break metadata,
            Ok(_) => return Err("a WatchFile source must name a regular file".into()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && source_wait.is_some() => {}
            Err(error) => return Err(format!("{}: {error}", path.display())),
        }
        let remaining = source_wait.expect("waiting opted in").saturating_sub(source_started.elapsed());
        if remaining.is_zero() {
            return Err("source wait deadline elapsed before the file appeared".into());
        }
        tokio::select! {
            _ = cancel.wait_for(|cancelled| *cancelled) => return Err(CANCELLED.into()),
            () = tokio::time::sleep(poll.min(remaining)) => {}
        }
    };
    std::fs::File::open(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let started = Instant::now();
    let baseline = vec![path.file_name().unwrap().to_string_lossy().into_owned()];
    if request.selection == dokime_domain::model::files::wire::FileWatchSelection::New {
        ready.send_replace(Some(Ok(dokime_domain::model::files::wire::FileWatchArmed { baseline })));
        let cancelled = wait(&mut cancel, bound, started).await;
        return if cancelled { Err(CANCELLED.into()) } else { Ok(WatchFileOutcome { artifacts: Vec::new(), gaps: Vec::new() }) };
    }
    if !matches!(policy, FileWatchPolicy::Stream | FileWatchPolicy::Mirror) {
        if policy == FileWatchPolicy::SnapshotAtStart {
            let outcome = snapshot(&binding, &path, artifact, maximum_bytes)?;
            ready.send_replace(Some(Ok(dokime_domain::model::files::wire::FileWatchArmed { baseline })));
            return Ok(outcome);
        }
        ready.send_replace(Some(Ok(dokime_domain::model::files::wire::FileWatchArmed { baseline })));
        let cancelled = wait(&mut cancel, bound, started).await;
        let outcome = snapshot(&binding, &path, artifact, maximum_bytes)?;
        return if cancelled { Err(CANCELLED.into()) } else { Ok(outcome) };
    }

    // Stream From The Length At The Start
    let spool = tempfile::NamedTempFile::new().map_err(|error| error.to_string())?;
    let mut tail = Tail {
        offset: if policy == FileWatchPolicy::Stream { metadata.len() } else { 0 },
        mirror: policy == FileWatchPolicy::Mirror,
        modified: None,
        active: true,
        identity: crate::files::identity(&path, &metadata),
        base: artifact.clone(),
        artifacts: Vec::new(),
        maximum_bytes,
        captured: 0,
        read_bytes: 64 * 1024,
        path,
        artifact,
        written: 0,
        spool,
        lines: LineBuffer::new(std::num::NonZeroUsize::new(64 * 1024).ok_or_else(|| "line capacity must be positive".to_owned())?),
        matchers,
        gaps: Vec::new(),
    };
    tail.pass(&binding).await?;
    let started = Instant::now();
    ready.send_replace(Some(Ok(dokime_domain::model::files::wire::FileWatchArmed { baseline })));
    let cancelled = loop {
        if let Err(error) = tail.pass(&binding).await {
            tail.seal(&binding).await?;
            return Err(error);
        }
        let pause = bound.map_or(poll, |bound| bound.saturating_sub(started.elapsed()).min(poll));
        if bound.is_some_and(|bound| started.elapsed() >= bound) {
            break false;
        }
        tokio::select! {
            _ = cancel.wait_for(|cancelled| *cancelled) => break true,
            () = tokio::time::sleep(pause) => {}
        }
    };
    let final_pass = tail.pass(&binding).await;
    let outcome = tail.finish(&binding).await?;
    final_pass?;
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
    let file = path.file_name().and_then(|name| name.to_str()).ok_or_else(|| format!("{} names no file", path.display()))?;
    Ok(if output.is_empty() { file.to_owned() } else { format!("{output}/{file}") })
}

// Whether the watch was cancelled rather than bounded.
async fn wait(cancel: &mut watch::Receiver<bool>, bound: Option<Duration>, started: Instant) -> bool {
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

fn snapshot(binding: &Binding, path: &Path, artifact: String, maximum_bytes: u64) -> Result<WatchFileOutcome, String> {
    let size_bytes = std::fs::metadata(path).map_err(|error| format!("{}: {error}", path.display()))?.len();
    if size_bytes > maximum_bytes {
        return Err(format!("file watch exceeds its {maximum_bytes} byte capture cap"));
    }
    binding.artifacts.import(path, &artifact).map_err(|error| format!("artifact {artifact:?}: {error}"))?;
    Ok(WatchFileOutcome { artifacts: vec![WatchedArtifact { artifact, size_bytes }], gaps: Vec::new() })
}

// Rows reach the execution's streams; markers its records.
async fn publish(binding: &Binding, matchers: &LineMatchers, lines: impl IntoIterator<Item = String>) {
    if matchers.is_empty() {
        return;
    }
    for line in lines {
        for found in matchers.matches(&line) {
            match found {
                LineMatch::Row { stream, values, storage_schema } => match Identifier::new(&stream) {
                    Ok(stream) => match crate::telemetry::capture_payload(&storage_schema, &values, &Default::default()) {
                        Ok(payload) => binding.streams.published(&stream, values, payload).await,
                        Err(error) => binding.journal.log(crate::execution::log::Level::Warn, error),
                    },
                    Err(error) => binding.journal.log(crate::execution::log::Level::Warn, format!("matcher stream {stream:?}: {error}")),
                },
                LineMatch::Marker { record_type, details, line } => match Identifier::new(&record_type) {
                    Ok(record_type) => {
                        let mut record = RecordPublishRequest::new(record_type, RecordSeverity::Info);
                        record.message = Some(line);
                        record.details = details;
                        binding.journal.record(record);
                    }
                    Err(error) => {
                        binding.journal.log(crate::execution::log::Level::Warn, format!("marker record type {record_type:?}: {error}"))
                    }
                },
            }
        }
    }
}

/* === Tests === */
