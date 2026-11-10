/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The file operations without a host, as a harness serves them: local files
// polled in this process. Stream captures only what is written after the start
// into the produced artifact and offers each line to the request's matchers and
// markers; Mirror keeps the latest bytes and a copy takes each file once. The
// binding owns a running capture, which seals on `StopCapture` or at run end.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

// Third Party Imports
use tokio::sync::watch;

// Local Imports
use super::Binding;
use crate::config::FileCaptureSettings;
use crate::domains::files::{Mode, Selected, gap_reason, local::LocalSource, mirror_interval, selected, validate_output};
use crate::files::{FileMatcher, LineBuffer, LineMatch, LineMatchers};
use crate::records::RecordSeverity;
use dokime_domain::model::files::wire::CaptureGap;
use dokime_domain::model::files::wire::CaptureGapReason;
use dokime_domain::model::files::wire::CaptureMode;
use dokime_domain::model::files::wire::CaptureStarted;
use dokime_domain::model::files::wire::CapturedArtifact;
use dokime_domain::model::files::wire::CapturedFiles;
use dokime_domain::model::files::wire::FileLocation;
use dokime_domain::model::files::wire::FileSource;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::records::BuiltinRecordType;
use dokime_domain::model::records::publication::RecordPublishRequest;
use dokime_domain::model::values::Value;
use dokime_operations::files::CopyFiles;
use dokime_operations::files::StartCapture;
use dokime_operations::files::StopCapture;

/* === Constants === */

const CANCELLED: &str = "the file operation was cancelled";
const READ_BYTES: usize = 64 * 1024;

/* === Macros === */

/* === Type Aliases === */

type Outcome = Option<Result<CapturedFiles, String>>;

/* === Structs === */

// The captures one run started; shared by every stage's binding.
#[derive(Clone, Default)]
pub(crate) struct LocalCaptures(Arc<Mutex<BTreeMap<u64, Running>>>);

struct Running {
    stop: watch::Sender<bool>,
    outcome: watch::Receiver<Outcome>,
}

struct Tail {
    writer: Option<crate::artifacts::ArtifactWriter>,
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
    gaps: Vec<CaptureGap>,
    identity: Option<(u64, u64)>,
    artifacts: Vec<CapturedArtifact>,
    base: String,
    maximum_bytes: u64,
    captured: u64,
}

struct Capture {
    root: PathBuf,
    source: LocalSource,
    matcher: FileMatcher,
    selection: Selected,
    output: String,
    mode: Mode,
    matchers: LineMatchers,
    selected: Option<String>,
    tails: BTreeMap<String, Tail>,
    names: BTreeSet<String>,
    segments: BTreeMap<String, u64>,
    maximum_files: usize,
    maximum_bytes: u64,
    line_bytes: std::num::NonZeroUsize,
    captured: u64,
    outcome: CapturedFiles,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl LocalCaptures {
    // Seals every capture still running and waits for it, at run end.
    pub(crate) async fn seal_all(&self) {
        let outcomes = {
            let captures = self.0.lock().unwrap_or_else(PoisonError::into_inner);
            captures
                .values()
                .map(|capture| {
                    capture.stop.send_replace(true);
                    capture.outcome.clone()
                })
                .collect::<Vec<_>>()
        };
        for mut outcome in outcomes {
            let _ = outcome.wait_for(Option::is_some).await;
        }
    }
}

impl Capture {
    async fn pass(&mut self, binding: &Binding, mut listing: crate::domains::files::source::Listing, first: bool) -> Result<(), String> {
        if self.selection.newest {
            listing.retain(|path, _| Some(path) == self.selected.as_ref());
        }
        let vanished = self.tails.keys().filter(|path| !listing.contains_key(*path)).cloned().collect::<Vec<_>>();
        for path in vanished {
            let mut tail = self.tails.remove(&path).expect("listed tail");
            tail.gap(binding, CaptureGapReason::Vanished);
            let outcome = tail.finish(binding).await?;
            self.outcome.artifacts.extend(outcome.artifacts);
            self.outcome.gaps.extend(outcome.gaps);
        }
        for (path, entry) in listing {
            if !self.tails.contains_key(&path) {
                if self.tails.len() >= self.maximum_files {
                    return Err(format!("file capture exceeds its {} file limit", self.maximum_files));
                }
                let base = if self.selection.single {
                    self.output.clone()
                } else if self.output.is_empty() {
                    path.clone()
                } else {
                    format!("{}/{path}", self.output)
                };
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
                        writer: None,
                        path: self.root.join(&path),
                        relative: path.clone(),
                        directory: !self.selection.single,
                        artifact,
                        base,
                        offset: if first && self.mode == Mode::Stream { entry.size } else { 0 },
                        written: 0,
                        spool: tempfile::NamedTempFile::new().map_err(|error| error.to_string())?,
                        active: true,
                        sealable: self.mode == Mode::Stream && !first,
                        segment: *segment,
                        mirror: self.mode != Mode::Stream,
                        modified: None,
                        identity: entry.identity,
                        lines: LineBuffer::new(self.line_bytes),
                        matchers: self.matchers.clone(),
                        gaps: Vec::new(),
                        artifacts: Vec::new(),
                        maximum_bytes: self.maximum_bytes,
                        captured: 0,
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

    async fn scan_pass(&mut self, binding: &Binding) -> Result<(), String> {
        let listing = self.source.scan(&self.matcher).await.map_err(|error| error.to_string())?;
        self.pass(binding, listing, false).await
    }

    async fn finish(mut self, binding: &Binding) -> Result<CapturedFiles, String> {
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
                    self.gap(binding, CaptureGapReason::Vanished);
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
            self.gap(binding, if replaced { CaptureGapReason::Replaced } else { CaptureGapReason::Truncated });
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
                return Err(format!("file capture exceeds its {} byte limit", self.maximum_bytes));
            }
            self.spool.as_file_mut().set_len(0).map_err(|error| error.to_string())?;
            self.spool.as_file_mut().seek(SeekFrom::Start(0)).map_err(|error| error.to_string())?;
            self.offset = 0;
            self.written = 0;
            self.captured = retained;
        }
        self.modified = metadata.modified().ok();
        self.sealable |= self.mirror;
        if self.sealable && !self.mirror && self.writer.is_none() {
            self.writer = Some(binding.artifacts.create(&self.artifact).map_err(|error| error.to_string())?);
            self.artifact = self.writer.as_ref().expect("stream writer created").name().to_owned();
        }
        if length == self.offset {
            return Ok(());
        }
        if self.captured.saturating_add(length - self.offset) > self.maximum_bytes {
            return Err(format!("file capture exceeds its {} byte limit", self.maximum_bytes));
        }
        let mut file = std::fs::File::open(&self.path).map_err(|error| error.to_string())?;
        file.seek(SeekFrom::Start(self.offset)).map_err(|error| error.to_string())?;
        let mut source = file.take(length - self.offset);
        let mut bytes = vec![0; READ_BYTES];
        loop {
            let count = source.read(&mut bytes).map_err(|error| error.to_string())?;
            if count == 0 {
                break;
            }
            self.spool.write_all(&bytes[..count]).map_err(|error| error.to_string())?;
            self.offset += count as u64;
            self.written += count as u64;
            self.captured += count as u64;
            self.sealable = true;
            if !self.mirror {
                if self.writer.is_none() {
                    self.writer = Some(binding.artifacts.create(&self.artifact).map_err(|error| error.to_string())?);
                    self.artifact = self.writer.as_ref().expect("stream writer created").name().to_owned();
                }
                let writer = self.writer.as_mut().expect("stream writer created");
                writer.write_all(&bytes[..count]).map_err(|error| error.to_string())?;
                writer.flush().map_err(|error| error.to_string())?;
                let lines = self.lines.push(&bytes[..count]);
                publish(binding, &self.matchers, lines).await;
            }
        }
        Ok(())
    }

    async fn finish(mut self, binding: &Binding) -> Result<CapturedFiles, String> {
        self.seal(binding).await?;
        Ok(CapturedFiles {
            artifacts: self.artifacts,
            gaps: self.gaps,
        })
    }

    async fn seal(&mut self, binding: &Binding) -> Result<(), String> {
        if !self.active {
            return Ok(());
        }
        if !self.sealable {
            self.active = false;
            return Ok(());
        }
        let last = self.lines.finish();
        publish(binding, &self.matchers, last).await;
        self.spool.flush().map_err(|error| error.to_string())?;
        if let Some(writer) = self.writer.take() {
            writer.finish().map_err(|error| error.to_string())?;
        } else if self.directory {
            let mime = crate::execution_runtime::outputs::media_type_for_path(&self.relative).unwrap_or("application/octet-stream");
            self.artifact = binding
                .artifacts
                .import_with_mime(self.spool.path(), &self.artifact, mime.to_owned())
                .map_err(|error| error.to_string())?;
        } else {
            binding.artifacts.import(self.spool.path(), &self.artifact).map_err(|error| error.to_string())?;
        }
        self.artifacts.push(CapturedArtifact {
            artifact: self.artifact.clone(),
            size_bytes: self.written,
        });
        self.active = false;
        Ok(())
    }

    // Reports one gap to the caller and into the execution's evidence.
    fn gap(&mut self, binding: &Binding, reason: CaptureGapReason) {
        let path = self.relative.clone();
        let mut record = RecordPublishRequest::new(BuiltinRecordType::FileCaptureGap.identifier(), RecordSeverity::Warning);
        record.details.insert("path".to_owned(), Value::Text(path.clone()));
        record.details.insert("offset".to_owned(), Value::Integer(self.written.try_into().unwrap_or(i64::MAX)));
        record.details.insert("reason".to_owned(), Value::Text(gap_reason(reason).to_owned()));
        binding.journal.record(record);
        self.gaps.push(CaptureGap { path, offset: self.written, reason });
    }
}

/* === Public Functions === */

pub(super) async fn copy(binding: Binding, request: CopyFiles, mut cancel: watch::Receiver<bool>) -> Result<CapturedFiles, String> {
    let (mut capture, listing) = prepare(&request.source, &request.output, Mode::Copy, LineMatchers::default()).await?;
    let passed = tokio::select! {
        biased;
        _ = cancel.wait_for(|cancelled| *cancelled) => Err(CANCELLED.to_owned()),
        result = capture.pass(&binding, listing, true) => result,
    };
    let outcome = capture.finish(&binding).await?;
    passed?;
    Ok(outcome)
}

pub(super) async fn start(binding: Binding, request: StartCapture, mut cancel: watch::Receiver<bool>) -> Result<CaptureStarted, String> {
    let limits = FileCaptureSettings::default();
    let (mode, poll, matchers, markers) = match &request.mode {
        CaptureMode::Stream(stream) => (
            Mode::Stream,
            crate::config::storage_defaults().filesystem.poll_interval,
            stream.matchers.as_slice(),
            stream.markers.as_slice(),
        ),
        CaptureMode::Mirror(mirror) => {
            (Mode::Mirror, mirror_interval(mirror.interval_s, &limits).map_err(|error| error.to_string())?, &[][..], &[][..])
        }
    };
    let lines = LineMatchers::compile(
        matchers,
        markers,
        limits.maximum_line_rules,
        limits.maximum_pattern_bytes,
        limits.maximum_regex_bytes,
        &Default::default(),
    )
    .map_err(|error| error.to_string())?;
    let (mut capture, listing) = prepare(&request.source, &request.output, mode, lines).await?;
    let first = tokio::select! {
        biased;
        _ = cancel.wait_for(|cancelled| *cancelled) => Err(CANCELLED.to_owned()),
        result = capture.pass(&binding, listing, true) => result,
    };
    if let Err(error) = first {
        capture.finish(&binding).await?;
        return Err(error);
    }
    let capture_id = uuid::Uuid::now_v7().as_u64_pair().1;
    let (stop, stopping) = watch::channel(false);
    let (finished, outcome) = watch::channel(None);
    let captures = binding.captures.clone();
    tokio::spawn(async move {
        let answer = drive(&binding, capture, stopping, poll).await;
        finished.send_replace(Some(answer));
    });
    captures.0.lock().unwrap_or_else(PoisonError::into_inner).insert(capture_id, Running { stop, outcome });
    Ok(CaptureStarted { capture_id })
}

pub(super) async fn stop(binding: Binding, request: StopCapture, mut cancel: watch::Receiver<bool>) -> Result<CapturedFiles, String> {
    let mut outcome = {
        let captures = binding.captures.0.lock().unwrap_or_else(PoisonError::into_inner);
        let capture = captures.get(&request.capture_id).ok_or("no such file capture")?;
        capture.stop.send_replace(true);
        capture.outcome.clone()
    };
    tokio::select! {
        biased;
        answer = outcome.wait_for(Option::is_some) => match answer {
            Ok(answer) => answer.clone().expect("the wait ends on an outcome"),
            Err(_) => Err("the file capture ended without an outcome".to_owned()),
        },
        _ = cancel.wait_for(|cancelled| *cancelled) => Err(CANCELLED.to_owned()),
    }
}

/* === Private Functions === */

async fn prepare(
    source: &FileSource, output: &str, mode: Mode, matchers: LineMatchers,
) -> Result<(Capture, crate::domains::files::source::Listing), String> {
    let limits = FileCaptureSettings::default();
    let selection = selected(source).map_err(|error| error.to_string())?;
    validate_output(output, selection.single).map_err(|error| error.to_string())?;
    let root = match &selection.location {
        FileLocation::Local(local) => PathBuf::from(&local.path),
        FileLocation::Ftp(_) => return Err("capturing an FTP source needs the application host".into()),
    };
    let matcher = FileMatcher::compile(&selection.rules).map_err(|error| error.to_string())?;
    let source = LocalSource::open(root.to_str().ok_or("capture root must be UTF-8")?).await.map_err(|error| error.to_string())?;
    let source = if selection.top_level { source.top_level() } else { source };
    let listing = source.scan(&matcher).await.map_err(|error| error.to_string())?;
    if listing.is_empty() && (selection.single || selection.newest) {
        return Err("the source has no matching file".into());
    }
    let chosen = listing.iter().max_by(|(lp, l), (rp, r)| l.modified.cmp(&r.modified).then_with(|| rp.cmp(lp))).map(|(p, _)| p.clone());
    let maximum_files = if selection.single { 1 } else { limits.maximum_files };
    let capture = Capture {
        root,
        source,
        matcher,
        selection,
        output: output.to_owned(),
        mode,
        matchers,
        selected: chosen,
        tails: BTreeMap::new(),
        names: BTreeSet::new(),
        segments: BTreeMap::new(),
        maximum_files,
        maximum_bytes: limits.maximum_bytes,
        line_bytes: limits.maximum_line_bytes,
        captured: 0,
        outcome: CapturedFiles { artifacts: Vec::new(), gaps: Vec::new() },
    };
    Ok((capture, listing))
}

// Passes until stopped, then takes the final pass and seals. A capture that
// fails records why, because nothing may ever stop it to ask.
async fn drive(
    binding: &Binding, mut capture: Capture, mut stopping: watch::Receiver<bool>, poll: Duration,
) -> Result<CapturedFiles, String> {
    let failed = loop {
        tokio::select! {
            _ = stopping.wait_for(|stopped| *stopped) => break None,
            () = tokio::time::sleep(poll) => {}
        }
        if let Err(error) = capture.scan_pass(binding).await {
            break Some(error);
        }
    };
    let failed = match failed {
        Some(error) => Some(error),
        None => capture.scan_pass(binding).await.err(),
    };
    let outcome = capture.finish(binding).await;
    let Some(error) = failed.or_else(|| outcome.as_ref().err().cloned()) else {
        return outcome;
    };
    let mut record = RecordPublishRequest::new(BuiltinRecordType::FileCaptureFailed.identifier(), RecordSeverity::Warning);
    record.details.insert("reason".to_owned(), Value::Text(error.clone()));
    binding.journal.record(record);
    Err(error)
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
