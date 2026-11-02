/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// `WatchFile` without a host, as a harness serves it: local files polled
// in this process. Stream captures only what is written after the start into
// the produced artifact and offers each line to the request's matchers and
// markers; Mirror keeps the latest bytes and snapshots copy each file once. Like the
// host's, cancellation seals captured evidence and returns its outcome.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::time::Duration;

// Third Party Imports
use tokio::sync::watch;
use tokio::time::Instant;

// Local Imports
use super::Binding;
use crate::domains::files::{local::LocalSource, watch_file_source};
use crate::files::{FileMatcher, LineBuffer, LineMatch, LineMatchers};
use crate::records::RecordSeverity;
use dokime_domain::model::files::wire::FileWatchGap;
use dokime_domain::model::files::wire::FileWatchGapReason;
use dokime_domain::model::files::wire::FileWatchSourcePayload;
use dokime_domain::model::files::wire::WatchFileOutcome;
use dokime_domain::model::files::wire::WatchedArtifact;
use dokime_domain::model::files::wire::{FileWatchArmed, FileWatchPolicy, FileWatchSelection};
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::records::publication::RecordPublishRequest;
use dokime_operations::files::WatchFile;

/* === Constants === */

const CANCELLED: &str = "the file watch was cancelled; it sealed what it captured";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Tail {
    path: PathBuf,
    relative: String,
    directory: bool,
    artifact: String,
    offset: u64,
    written: u64,
    spool: tempfile::NamedTempFile,
    active: bool,
    sealable: bool,
    segment: u64,
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

struct Capture {
    root: PathBuf,
    source: LocalSource,
    matcher: FileMatcher,
    request: WatchFile,
    matchers: LineMatchers,
    baseline: BTreeSet<String>,
    selected: Option<String>,
    tails: BTreeMap<String, Tail>,
    names: BTreeSet<String>,
    segments: BTreeMap<String, u64>,
    maximum_files: usize,
    maximum_bytes: u64,
    captured: u64,
    outcome: WatchFileOutcome,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Capture {
    async fn pass(&mut self, binding: &Binding, mut listing: crate::domains::files::source::Listing, first: bool) -> Result<(), String> {
        match self.request.selection {
            FileWatchSelection::All => {}
            FileWatchSelection::Latest => listing.retain(|path, _| Some(path) == self.selected.as_ref()),
            FileWatchSelection::New => listing.retain(|path, _| !self.baseline.contains(path)),
        }
        let vanished = self.tails.keys().filter(|path| !listing.contains_key(*path)).cloned().collect::<Vec<_>>();
        for path in vanished {
            let mut tail = self.tails.remove(&path).expect("listed tail");
            tail.gap(FileWatchGapReason::Vanished);
            let outcome = tail.finish(binding).await?;
            self.outcome.artifacts.extend(outcome.artifacts);
            self.outcome.gaps.extend(outcome.gaps);
        }
        for (path, entry) in listing {
            if !self.tails.contains_key(&path) {
                if self.tails.len() >= self.maximum_files {
                    return Err(format!("file watch exceeds its {} file cap", self.maximum_files));
                }
                let base = self.request.artifact.clone().unwrap_or_else(|| {
                    if self.request.output.is_empty() { path.clone() } else { format!("{}/{path}", self.request.output) }
                });
                let segment = self.segments.entry(path.clone()).or_default();
                let mut artifact = if *segment == 0 { base.clone() } else { crate::files::segment_name(&base, *segment + 1) };
                while self.names.contains(&artifact) {
                    *segment += 1;
                    artifact = crate::files::segment_name(&base, *segment + 1);
                }
                dokime_domain::model::artifacts::name::validate(&artifact).map_err(|error| error.to_string())?;
                self.names.insert(artifact.clone());
                self.tails.insert(
                    path.clone(),
                    Tail {
                        path: self.root.join(&path),
                        relative: path.clone(),
                        directory: self.request.recursive || !self.request.rules.is_empty(),
                        artifact,
                        base,
                        offset: if first && self.request.policy == FileWatchPolicy::Stream { entry.size } else { 0 },
                        written: 0,
                        spool: tempfile::NamedTempFile::new().map_err(|error| error.to_string())?,
                        active: true,
                        sealable: self.request.policy == FileWatchPolicy::Stream,
                        segment: *segment,
                        mirror: self.request.policy != FileWatchPolicy::Stream,
                        modified: None,
                        identity: entry.identity,
                        lines: LineBuffer::new(std::num::NonZeroUsize::new(64 * 1024).expect("positive capacity")),
                        matchers: self.matchers.clone(),
                        gaps: Vec::new(),
                        artifacts: Vec::new(),
                        maximum_bytes: self.maximum_bytes,
                        captured: 0,
                        read_bytes: 64 * 1024,
                    },
                );
            }
            let tail = self.tails.get_mut(&path).expect("discovered tail");
            let retained = self.captured - tail.captured;
            tail.maximum_bytes = self.maximum_bytes.saturating_sub(retained);
            let result = tail.pass(binding, &mut self.names).await;
            self.captured = retained + tail.captured;
            self.names.insert(tail.artifact.clone());
            *self.segments.entry(path).or_default() = tail.segment;
            result?;
        }
        Ok(())
    }

    async fn finish(mut self, binding: &Binding) -> Result<WatchFileOutcome, String> {
        for (_, tail) in self.tails {
            let outcome = tail.finish(binding).await?;
            self.outcome.artifacts.extend(outcome.artifacts);
            self.outcome.gaps.extend(outcome.gaps);
        }
        Ok(self.outcome)
    }
}

impl Tail {
    // Captures what the file gained since the last pass.
    async fn pass(&mut self, binding: &Binding, names: &mut BTreeSet<String>) -> Result<(), String> {
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
        let replaced = matches!((self.identity, identity), (Some(before), Some(after)) if before != after);
        let truncated = metadata.len() < self.offset;
        if self.active && (replaced || truncated) {
            self.gap(if replaced { FileWatchGapReason::Replaced } else { FileWatchGapReason::Truncated });
            self.seal(binding).await?;
        }
        if !self.active {
            loop {
                self.segment += 1;
                self.artifact = crate::files::segment_name(&self.base, self.segment + 1);
                if names.insert(self.artifact.clone()) {
                    break;
                }
            }
            dokime_domain::model::artifacts::name::validate(&self.artifact).map_err(|error| error.to_string())?;
            self.spool = tempfile::NamedTempFile::new().map_err(|error| error.to_string())?;
            self.offset = 0;
            self.written = 0;
            self.lines.reset();
            self.modified = None;
            self.active = true;
            self.sealable = !self.mirror;
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
        self.sealable = true;
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
        Ok(WatchFileOutcome {
            artifacts: self.artifacts,
            gaps: self.gaps,
        })
    }

    async fn seal(&mut self, binding: &Binding) -> Result<(), String> {
        if !self.active || !self.sealable {
            return Ok(());
        }
        let last = self.lines.finish();
        publish(binding, &self.matchers, last).await;
        self.spool.flush().map_err(|error| error.to_string())?;
        if self.directory {
            let mime = crate::execution_runtime::outputs::media_type_for_path(&self.relative).unwrap_or("application/octet-stream");
            binding.artifacts.import_with_mime(self.spool.path(), &self.artifact, mime.to_owned()).map_err(|error| error.to_string())?;
        } else {
            binding.artifacts.import(self.spool.path(), &self.artifact).map_err(|error| error.to_string())?;
        }
        self.artifacts.push(WatchedArtifact {
            artifact: self.artifact.clone(),
            size_bytes: self.written,
        });
        self.active = false;
        Ok(())
    }

    fn gap(&mut self, reason: FileWatchGapReason) {
        let path = self.relative.clone();
        self.gaps.push(FileWatchGap { path, offset: self.written, reason });
    }
}

/* === Public Functions === */

pub(super) async fn run(
    binding: Binding, request: WatchFile, mut cancel: watch::Receiver<bool>, ready: super::Ready,
) -> Result<WatchFileOutcome, String> {
    let (source, rules, single) = watch_file_source(&request).map_err(|error| error.to_string())?;
    let root = match source.payload {
        Some(FileWatchSourcePayload::Local(local)) => PathBuf::from(local.root),
        Some(FileWatchSourcePayload::Ftp(_)) => return Err("watching an FTP source needs the Dokime host".into()),
        None => return Err("a file watch needs a local or FTP source".into()),
    };
    let matcher = FileMatcher::compile(&rules).map_err(|error| error.to_string())?;
    let matchers = LineMatchers::compile(&request.matchers, &request.markers, 32, 1024, 1 << 20, &Default::default())
        .map_err(|error| error.to_string())?;
    let policy = request.policy;
    if policy == FileWatchPolicy::Unspecified {
        return Err("a file watch needs a policy".into());
    }
    if !matchers.is_empty() && policy != FileWatchPolicy::Stream {
        return Err("line matchers and markers need the Stream policy".into());
    }
    if !request.output.is_empty() && !dokime_domain::model::artifacts::name::is_relative_path(&request.output) {
        return Err("the output folder must be a relative path with forward slashes".into());
    }
    if let Some(artifact) = &request.artifact {
        dokime_domain::model::artifacts::name::validate(artifact).map_err(|error| error.to_string())?;
    }
    let poll = request.poll_interval_ms.map_or(Duration::from_secs(1), |ms| Duration::from_millis(ms.into()));
    let bound = request.maximum_seconds.map(|seconds| Duration::from_secs(seconds.into()));
    let source_wait = request.wait_for_source_seconds.map(|seconds| Duration::from_secs(seconds.into()));
    let maximum_bytes = request.maximum_bytes.unwrap_or(256 * 1024 * 1024);
    let maximum_files = if single { 1 } else { request.maximum_files.unwrap_or(64) as usize };
    let valid_duration = |duration: Duration| !duration.is_zero() && duration <= Duration::from_secs(86_400);
    if !(Duration::from_millis(50)..=Duration::from_secs(60)).contains(&poll)
        || bound.is_some_and(|duration| !valid_duration(duration))
        || source_wait.is_some_and(|duration| !valid_duration(duration))
        || maximum_bytes == 0
        || maximum_bytes > 4 * 1024 * 1024 * 1024
        || maximum_files == 0
        || maximum_files > 256
    {
        return Err("invalid file watch poll interval, duration, file or byte cap".into());
    }
    let source = LocalSource::open(root.to_str().ok_or("watch root must be UTF-8")?, source_wait.is_some())
        .await
        .map_err(|error| error.to_string())?;
    let source = if request.recursive { source } else { source.top_level() };
    let waiting = Instant::now();
    let listing = loop {
        if source_wait.is_some_and(|wait| waiting.elapsed() >= wait) {
            return Err("source wait deadline elapsed before a matching file appeared".into());
        }
        match source.scan(&matcher).await {
            Ok(listing)
                if !listing.is_empty() || (!single && !(request.selection == FileWatchSelection::Latest && source_wait.is_some())) =>
            {
                break listing;
            }
            Ok(_) if source_wait.is_none() => return Err("the watched source has no matching file".into()),
            Ok(_) => {}
            Err(error) if error.code() == Some("file_watch_missing") && source_wait.is_some() => {}
            Err(error) => return Err(error.to_string()),
        }
        let pause = poll.min(source_wait.expect("waiting opted in").saturating_sub(waiting.elapsed()));
        tokio::select! {
            _ = cancel.wait_for(|cancelled| *cancelled) => return Err(CANCELLED.into()),
            () = tokio::time::sleep(pause) => {}
        }
    };
    let baseline = listing.keys().cloned().collect::<BTreeSet<_>>();
    let selected = listing.iter().max_by(|(lp, l), (rp, r)| l.modified.cmp(&r.modified).then_with(|| rp.cmp(lp))).map(|(p, _)| p.clone());
    let mut capture = Capture {
        root,
        source,
        matcher,
        request,
        matchers,
        baseline,
        selected,
        tails: BTreeMap::new(),
        names: BTreeSet::new(),
        segments: BTreeMap::new(),
        maximum_files,
        maximum_bytes,
        captured: 0,
        outcome: WatchFileOutcome { artifacts: Vec::new(), gaps: Vec::new() },
    };
    if policy != FileWatchPolicy::SnapshotAtStop
        && let Err(error) = capture.pass(&binding, listing, true).await
    {
        capture.finish(&binding).await?;
        return Err(error);
    }
    ready.send_replace(Some(Ok(FileWatchArmed {
        baseline: capture.baseline.iter().cloned().collect(),
    })));
    if policy == FileWatchPolicy::SnapshotAtStart {
        return capture.finish(&binding).await;
    }
    let started = Instant::now();
    let result = loop {
        let pause = bound.map_or(poll, |bound| bound.saturating_sub(started.elapsed()).min(poll));
        if bound.is_some_and(|bound| started.elapsed() >= bound) || *cancel.borrow() {
            break Ok(());
        }
        tokio::select! {
            _ = cancel.wait_for(|cancelled| *cancelled) => break Ok(()),
            () = tokio::time::sleep(pause) => {}
        }
        if policy != FileWatchPolicy::SnapshotAtStop {
            let pass = match capture.source.scan(&capture.matcher).await {
                Ok(listing) => capture.pass(&binding, listing, false).await,
                Err(error) => Err(error.to_string()),
            };
            if let Err(error) = pass {
                break Err(error);
            }
        }
    };
    let final_pass = if result.is_ok() {
        match capture.source.scan(&capture.matcher).await {
            Ok(listing) => capture.pass(&binding, listing, false).await,
            Err(error) => Err(error.to_string()),
        }
    } else {
        result
    };
    let outcome = capture.finish(&binding).await?;
    final_pass?;
    Ok(outcome)
}

/* === Private Functions === */

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
