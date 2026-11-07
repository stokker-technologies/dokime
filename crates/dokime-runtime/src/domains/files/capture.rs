/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// One running file capture. Each tracked file owns a private spool that holds
// exactly its captured bytes. Stream captures publish committed ranges into
// one live artifact; sealing finishes that same identity. Mirror and copy
// captures upload their final successful spool under `<output>/<relative path>`.
// The caller drives `poll` on changes or at its interval, and `seal` at the end.
// Closing the execution asks the capture to seal, and dropping an unsealed
// capture seals it in the background. A Stream capture with line matchers offers
// each captured line to them, publishing rows to the matched streams and markers
// as records in the execution's scope. Gaps become records the same way.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::io::Seek;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    time::Duration,
};

// Third Party Imports
use tempfile::TempPath;
use tokio::{io::AsyncSeekExt, sync::oneshot};

// Local Imports
use super::source::{Entry, Listing, ReadFailure, Source};
#[cfg(feature = "ftp")]
use super::unreachable_source;
use super::{GapReason, Settings, capped, invalid};
use crate::artifacts::{ArtifactRecord, ArtifactUploadInput};
use crate::execution_runtime::outputs::media_type_for_path;
use crate::files::{FileMatcher, LineBuffer, LineMatch, LineMatchers};
use crate::records::RecordSeverity;
use crate::runtime::storage::artifacts::ArtifactUpload;
use crate::runtime::storage::captures::SealRequest;
use crate::telemetry::{TelemetryBatch, TelemetrySample};
use crate::{RuntimeHandle, runtime::Error, runtime::Result};
use dokime_domain::model::artifacts::name;
use dokime_domain::model::diagnostics::admission::AdmissionFailure;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::records::BuiltinRecordType;
use dokime_domain::model::records::publication::RecordPublishRequest;
use dokime_domain::model::values::Value;

/* === Constants === */

// The producer named on the rows and records line matchers publish.
const SOURCE: &str = "dokime.files";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct FileCapture {
    // Taken by `Drop` to seal an abandoned capture in the background.
    state: Option<State>,
    changes: Option<harmos::storage::DirectoryWatcher>,
}

struct State {
    runtime: RuntimeHandle,
    scope: RecordScope,
    execution: TestExecutionId,
    output: String,
    artifact: Option<String>,
    mode: Mode,
    newest: bool,
    baseline: Option<BTreeSet<String>>,
    selected: Option<String>,
    single: bool,
    segments: BTreeMap<String, u64>,
    names: BTreeSet<String>,
    matcher: FileMatcher,
    lines: LineMatchers,
    sequences: BTreeMap<String, u64>,
    source: Source,
    poll_interval: Duration,
    maximum_files: usize,
    maximum_bytes: u64,
    maximum_failed_passes: u64,
    line_bytes: std::num::NonZeroUsize,
    spools: tempfile::TempDir,
    tracked: BTreeMap<String, Tracked>,
    // Bytes held by live spools and sealed artifacts, charged to the byte cap.
    captured: u64,
    sealed: Vec<ArtifactRecord>,
    passes: u64,
    failed_passes: u64,
    complete: bool,
    // A refused pass ends capture; sealing keeps only what was captured.
    failed: bool,
    registration: u64,
    seal_request: oneshot::Receiver<oneshot::Sender<()>>,
    seal_answer: Option<oneshot::Sender<()>>,
}

struct Tracked {
    upload: Option<ArtifactUpload>,
    artifact: String,
    spool: TempPath,
    // Bytes in the spool; anything past it is an interrupted write.
    written: u64,
    // Stream: the next source byte to capture.
    offset: u64,
    entry: Entry,
    // Mirror policy, or a Stream file whose source cannot resume.
    mirrored: bool,
    synced: bool,
    // A captured line not yet ended, for the line matchers.
    pending: LineBuffer,
    // The spool holds a whole copy or a captured prefix worth sealing; a
    // mirrored file never fetched holds nothing.
    sealable: bool,
}

/* === Enums === */

// What a capture does to a selected file's bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Stream,
    Mirror,
    // Each selected file once, as it is at the first pass.
    Copy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    // Discovered; capture starts from the baseline.
    Baseline,
    Capturing,
    Synced,
}

#[derive(Debug)]
pub enum CaptureEvent {
    Armed { baseline: Vec<String> },
    Progress { path: String, offset: u64, phase: Phase },
    Gap { path: String, offset: u64, reason: GapReason },
    Sealed { path: String, record: Box<ArtifactRecord> },
}

/* === Traits === */

/* === Implementations === */

impl FileCapture {
    pub(super) async fn open(runtime: RuntimeHandle, settings: Settings, source: Source) -> Result<Self> {
        let execution = settings.scope.test_execution_id.clone().ok_or_else(|| invalid("a file capture requires an execution"))?;
        let spools = tempfile::Builder::new()
            .prefix("dokime-capture-")
            .tempdir()
            .map_err(|source| Error::Operation(format!("file capture spool: {source}")))?;

        // Subscribe Before The Baseline So A Racing Write Remains Observable.
        // A Mirror Checks Once Per Interval, However Often Its Source Changes.
        let changes = if settings.mode == Mode::Stream { source.changes(runtime.storage()).await } else { None };

        // Seal into the Execution Before It Stops Admitting Artifacts
        let (seal_sender, seal_request) = oneshot::channel();
        let registration = runtime.execution_storage.captures.register(&execution, seal_sender as SealRequest)?;
        Ok(Self {
            state: Some(State {
                runtime,
                scope: settings.scope,
                execution,
                output: settings.output,
                artifact: settings.artifact,
                mode: settings.mode,
                newest: settings.newest,
                baseline: None,
                selected: None,
                single: settings.single,
                segments: BTreeMap::new(),
                names: BTreeSet::new(),
                matcher: settings.matcher,
                lines: settings.lines,
                sequences: BTreeMap::new(),
                source,
                poll_interval: settings.poll_interval,
                maximum_files: settings.maximum_files,
                maximum_bytes: settings.maximum_bytes,
                maximum_failed_passes: settings.maximum_failed_passes,
                line_bytes: settings.line_bytes,
                spools,
                tracked: BTreeMap::new(),
                captured: 0,
                sealed: Vec::new(),
                passes: 0,
                failed_passes: 0,
                complete: false,
                failed: false,
                registration,
                seal_request,
                seal_answer: None,
            }),
            changes,
        })
    }

    pub fn poll_interval(&self) -> Duration {
        self.state().poll_interval
    }

    // Whether a change of the source can wake the next pass early.
    pub fn follows_changes(&self) -> bool {
        self.changes.is_some()
    }

    // Whether the capture has sealed every file it will capture.
    pub fn is_complete(&self) -> bool {
        self.state().complete
    }

    // The artifacts sealed so far, in sealing order.
    pub fn sealed(&self) -> &[ArtifactRecord] {
        &self.state().sealed
    }

    // One discovery and capture pass. Events are pushed even when the pass
    // fails, so frames already earned reach the caller.
    pub async fn poll(&mut self, events: &mut Vec<CaptureEvent>) -> Result<()> {
        let result = self.state_mut().poll(events).await;
        if self.state().complete || self.state().failed {
            self.changes = None;
        }
        result
    }

    // Captures what remains, then seals every tracked file. Idempotent.
    pub async fn seal(&mut self, events: &mut Vec<CaptureEvent>) -> Result<()> {
        self.changes = None;
        self.state_mut().seal(events).await
    }

    // Resolves when the execution asks its running captures to seal.
    // Cancel-safe; it stays pending once answered or when no request can come.
    pub async fn seal_requested(&mut self) {
        self.state_mut().seal_requested().await;
    }

    // Waits for the next pass; true means the execution asked for the seal.
    pub async fn wait_for_poll(&mut self) -> bool {
        let pause = self.poll_interval();
        let state = self.state.as_mut().expect("file capture state lives until drop");
        tokio::select! {
            () = state.seal_requested() => true,
            () = source_changed(&mut self.changes) => false,
            () = tokio::time::sleep(pause) => false,
        }
    }

    // Records a capture that ended early, in the evidence of its execution.
    pub(super) async fn record_failure(&self, failure: &Error) {
        let mut record = RecordPublishRequest::new(BuiltinRecordType::FileCaptureFailed.identifier(), RecordSeverity::Warning);
        record.details.insert("reason".to_owned(), Value::Text(failure.to_string()));
        record.scope = Some(self.state().scope.clone());
        if let Err(error) = self.state().runtime.publish_record(record).await {
            tracing::warn!(%error, "file capture failure record was refused");
        }
    }

    fn state(&self) -> &State {
        self.state.as_ref().expect("file capture state lives until drop")
    }

    fn state_mut(&mut self) -> &mut State {
        self.state.as_mut().expect("file capture state lives until drop")
    }
}

impl Drop for FileCapture {
    fn drop(&mut self) {
        let Some(mut state) = self.state.take() else {
            return;
        };
        if state.complete {
            return;
        }
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            tracing::warn!(execution = %state.execution, "file capture dropped outside a runtime; its spools are discarded");
            state.release();
            return;
        };
        runtime.spawn(async move {
            let mut events = Vec::new();
            if let Err(source) = state.seal(&mut events).await {
                tracing::warn!(error = %source, "abandoned file capture failed to seal");
            }
        });
    }
}

impl State {
    async fn seal_requested(&mut self) {
        if self.complete || self.seal_answer.is_some() {
            return std::future::pending().await;
        }
        match (&mut self.seal_request).await {
            Ok(answer) => self.seal_answer = Some(answer),
            Err(_) => std::future::pending().await,
        }
    }

    async fn poll(&mut self, events: &mut Vec<CaptureEvent>) -> Result<()> {
        if self.complete || self.failed {
            return Ok(());
        }
        let first = self.baseline.is_none();
        let beginning = events.len();
        self.passes += 1;
        let polled = match self.mode {
            Mode::Copy if first => self.copy(events).await,
            Mode::Copy => Ok(()),
            Mode::Stream | Mode::Mirror => self.track(first, events).await,
        };
        if polled.is_ok()
            && first
            && let Some(baseline) = &self.baseline
        {
            events.insert(
                beginning,
                CaptureEvent::Armed {
                    baseline: baseline.iter().cloned().collect(),
                },
            );
        }
        self.failed |= polled.is_err();
        polled
    }

    #[tracing::instrument(name = "files.capture.seal", level = "debug", skip_all)]
    async fn seal(&mut self, events: &mut Vec<CaptureEvent>) -> Result<()> {
        if self.complete {
            return Ok(());
        }

        // Capture The Last Bytes; A Failing Source Still Seals What It Gave
        let captured = match self.mode {
            _ if self.failed => Ok(()),
            Mode::Copy if self.passes == 0 => self.copy(events).await,
            Mode::Copy => Ok(()),
            Mode::Stream | Mode::Mirror => {
                let first = self.passes == 0;
                self.track(first, events).await
            }
        };
        if let Err(source) = &captured {
            tracing::warn!(error = %source, execution = %self.execution, "file capture sealed after a failed final pass");
        }
        let sealed = self.finish(events).await;
        captured.and(sealed)
    }

    // Seals every tracked file, then answers the execution's seal request.
    #[tracing::instrument(name = "files.capture.finish", level = "debug", skip_all)]
    async fn finish(&mut self, events: &mut Vec<CaptureEvent>) -> Result<()> {
        #[cfg(feature = "testing")]
        crate::domains::execution::run::testing::before_capture_seal(&self.execution).await;
        let mut outcome = Ok(());
        let paths = self.tracked.keys().cloned().collect::<Vec<_>>();
        for path in paths {
            if let Err(source) = self.seal_file(&path, events).await {
                tracing::warn!(error = %source, %path, "file capture failed to seal a file");
                if outcome.is_ok() {
                    outcome = Err(source);
                }
            }
        }
        self.complete = true;
        self.release();
        outcome
    }

    fn release(&mut self) {
        self.runtime.execution_storage.captures.release(&self.execution, self.registration);
        if let Some(answer) = self.seal_answer.take() {
            let _ = answer.send(());
        }
    }

    // Stream and Mirror: discover, capture, and seal files that vanished.
    async fn track(&mut self, first: bool, events: &mut Vec<CaptureEvent>) -> Result<()> {
        let Some(listing) = self.listing().await? else {
            return Ok(());
        };

        // Release Simultaneous Slots Before Admitting New Paths
        let vanished = self.tracked.keys().filter(|path| !listing.contains_key(*path)).cloned().collect::<Vec<_>>();
        for path in vanished {
            let offset = self.tracked[&path].written;
            self.gap(&path, offset, GapReason::Vanished, events).await;
            self.seal_file(&path, events).await?;
        }

        // Capture Each Matched File from Its Baseline
        for (path, entry) in &listing {
            // A Replaced Source Seals Its Segment; The New File Starts Afresh
            if let Some(tracked) = self.tracked.get(path) {
                let replaced = matches!(
                    (tracked.entry.identity, entry.identity),
                    (Some(before), Some(after)) if before != after
                );
                let truncated = entry.size < tracked.offset || (tracked.mirrored && entry.size < tracked.entry.size);
                if replaced || truncated {
                    let offset = tracked.written;
                    let reason = if replaced { GapReason::Replaced } else { GapReason::Truncated };
                    self.gap(path, offset, reason, events).await;
                    self.seal_file(path, events).await?;
                }
            }
            if !self.tracked.contains_key(path) {
                let baseline = if first && self.mode == Mode::Stream { entry.size } else { 0 };
                self.discover(path, *entry, baseline, !first, events).await?;
            }
            match self.capture(path, *entry, events).await {
                Ok(()) => {}
                Err(source @ Error::Admission(AdmissionFailure::Unavailable { .. })) => return self.failed_pass(source),
                Err(source) => return Err(source),
            }
            self.publish(path).await?;
        }
        Ok(())
    }

    async fn listing(&mut self) -> Result<Option<Listing>> {
        let first = self.baseline.is_none();
        let mut listing = match self.source.scan(&self.matcher).await {
            Ok(listing) => {
                self.failed_passes = 0;
                listing
            }
            Err(source @ Error::Admission(AdmissionFailure::Unavailable { .. })) => {
                self.failed_pass(source)?;
                return Ok(None);
            }
            Err(source) => return Err(source),
        };

        if first && listing.is_empty() && (self.single || self.newest) {
            return Err(Error::not_found(super::MISSING, "the source has no matching file"));
        }
        self.select(&mut listing);

        Ok(Some(listing))
    }

    // A source that keeps failing ends the capture; a passing glitch does not.
    fn failed_pass(&mut self, source: Error) -> Result<()> {
        self.failed_passes += 1;
        tracing::warn!(error = %source, attempt = self.failed_passes, "file capture pass failed");
        if self.failed_passes >= self.maximum_failed_passes {
            return Err(source);
        }
        Ok(())
    }

    // Starts tracking one file, refusing it explicitly past the caps.
    async fn discover(&mut self, path: &str, entry: Entry, baseline: u64, new_source: bool, events: &mut Vec<CaptureEvent>) -> Result<()> {
        if self.artifact.is_some() && !self.tracked.is_empty() {
            return Err(capped(format!("a capture into one named artifact tracks one file; {path:?} would be a second")));
        }
        if self.tracked.len() >= self.maximum_files {
            return Err(capped(format!("the capture tracks at most {} files; {path:?} would exceed it", self.maximum_files)));
        }
        let mut artifact = self.artifact_name(path);
        while self.names.contains(&artifact) {
            *self.segments.entry(path.to_owned()).or_default() += 1;
            artifact = self.artifact_name(path);
        }
        name::validate(&artifact).map_err(|source| invalid(format!("artifact {artifact:?}: {source}")))?;
        self.names.insert(artifact.clone());
        let spool = tempfile::NamedTempFile::new_in(self.spools.path())
            .map_err(|source| Error::Operation(format!("file capture spool: {source}")))?
            .into_temp_path();
        self.tracked.insert(
            path.to_owned(),
            Tracked {
                upload: None,
                artifact,
                spool,
                written: 0,
                offset: baseline,
                entry,
                mirrored: self.mode == Mode::Mirror,
                synced: false,
                pending: LineBuffer::new(self.line_bytes),
                sealable: self.mode == Mode::Stream && new_source,
            },
        );
        events.push(CaptureEvent::Progress {
            path: path.to_owned(),
            offset: 0,
            phase: Phase::Baseline,
        });
        Ok(())
    }

    async fn capture(&mut self, path: &str, entry: Entry, events: &mut Vec<CaptureEvent>) -> Result<()> {
        let tracked = self.tracked.get_mut(path).expect("captured files are tracked");
        if tracked.mirrored {
            return self.mirror(path, entry, events).await;
        }

        tracked.entry = entry;
        if entry.size <= tracked.offset {
            return Ok(());
        }

        // Append Only Bytes the Cap Admits
        let length = entry.size - tracked.offset;
        if self.captured.saturating_add(length) > self.maximum_bytes {
            return Err(over_bytes(self.maximum_bytes, path));
        }
        let mut spool = open_spool(&tracked.spool, tracked.written, false).await?;
        match self.source.read_range(path, tracked.offset, length, &mut spool).await {
            Ok(copied) => {
                tracked.offset += copied;
                tracked.written += copied;
                self.captured += copied;
                if copied == 0 {
                    return Ok(());
                }
                tracked.sealable = true;
                events.push(CaptureEvent::Progress {
                    path: path.to_owned(),
                    offset: tracked.written,
                    phase: Phase::Capturing,
                });
                drop(spool);
                if self.lines.is_empty() {
                    return Ok(());
                }
                // The Captured Bytes Are Read Back from the Spool as Lines
                let captured = read_back(&tracked.spool, tracked.written - copied, copied)?;
                let lines = tracked.pending.push(&captured);
                self.offer(lines).await;
                Ok(())
            }
            // A Source That Cannot Resume Mirrors This File from Now On
            #[cfg(feature = "ftp")]
            Err(ReadFailure::ResumeUnsupported) => {
                let offset = tracked.written;
                self.gap(path, offset, GapReason::ResumeUnsupported, events).await;
                let tracked = self.tracked.get(path).expect("captured files are tracked");
                if !self.lines.is_empty() {
                    return Err(Error::conflict(
                        super::UNSUPPORTED,
                        "FTP server refused REST resume; line extraction cannot continue. Enable REST on the server or use a resumable/local source; captured evidence will be sealed",
                    ));
                }
                if tracked.upload.is_some() {
                    // A Visible Stream Prefix Keeps Its Identity When FTP Falls Back.
                    self.seal_file(path, events).await?;
                    self.discover(path, entry, 0, true, events).await?;
                }
                self.tracked.get_mut(path).expect("fallback file is tracked").mirrored = true;
                self.mirror(path, entry, events).await
            }
            Err(ReadFailure::Failed(source)) => Err(source),
        }
    }

    // Refetches the whole file when its size or modification time changed.
    async fn mirror(&mut self, path: &str, entry: Entry, events: &mut Vec<CaptureEvent>) -> Result<()> {
        let tracked = self.tracked.get_mut(path).expect("mirrored files are tracked");
        if tracked.synced
            && tracked.entry.size == entry.size
            && tracked.entry.modified == entry.modified
            && tracked.entry.identity == entry.identity
        {
            return Ok(());
        }
        let held = self.captured - tracked.written;
        if held.saturating_add(entry.size) > self.maximum_bytes {
            return Err(over_bytes(self.maximum_bytes, path));
        }

        // Fetch Into A Fresh Spool; The Previous Copy Stays Until It Succeeds
        let fresh = tempfile::NamedTempFile::new_in(self.spools.path())
            .map_err(|source| Error::Operation(format!("file capture spool: {source}")))?
            .into_temp_path();
        let mut spool = open_spool(&fresh, 0, true).await?;
        let copied = match self.source.read_range(path, 0, entry.size, &mut spool).await {
            Ok(copied) => copied,
            Err(ReadFailure::Failed(source)) => return Err(source),
            #[cfg(feature = "ftp")]
            Err(ReadFailure::ResumeUnsupported) => {
                return Err(unreachable_source("a whole-file retrieval was refused"));
            }
        };
        drop(spool);
        tracked.spool = fresh;
        tracked.written = copied;
        tracked.entry = entry;
        tracked.synced = true;
        tracked.sealable = true;
        self.captured = held + copied;
        events.push(CaptureEvent::Progress {
            path: path.to_owned(),
            offset: copied,
            phase: Phase::Synced,
        });
        Ok(())
    }

    // Copies every selected file once, as it is now.
    async fn copy(&mut self, events: &mut Vec<CaptureEvent>) -> Result<()> {
        let Some(listing) = self.listing().await? else {
            return Ok(());
        };
        for (path, entry) in listing {
            self.discover(&path, entry, 0, true, events).await?;
            self.tracked.get_mut(&path).expect("discovered files are tracked").mirrored = true;
            self.mirror(&path, entry, events).await?;
        }
        Ok(())
    }

    // Publish Committed Stream Bytes Without Waiting For Any Viewer.
    async fn publish(&mut self, path: &str) -> Result<()> {
        let tracked = self.tracked.get_mut(path).expect("published file is tracked");
        if !tracked.sealable || tracked.mirrored {
            return Ok(());
        }
        if tracked.upload.is_none() {
            tracked.upload = Some(
                self.runtime
                    .begin_streamed_artifact(ArtifactUploadInput {
                        scope: self.scope.clone(),
                        filename: tracked.artifact.clone(),
                        content_type: Some(media_type_for_path(path).unwrap_or("application/octet-stream").to_owned()),
                        size_bytes: 0,
                        sha256: None,
                    })
                    .await?,
            );
        }
        let upload = tracked.upload.as_ref().expect("stream artifact admitted");
        upload.send_file_range(&tracked.spool, tracked.written).await?;
        Ok(())
    }

    // Uploads one spool as the artifact `<output>/<path>` and stops tracking it.
    async fn seal_file(&mut self, path: &str, events: &mut Vec<CaptureEvent>) -> Result<()> {
        self.publish(path).await?;
        let Some(mut tracked) = self.tracked.remove(path) else {
            return Ok(());
        };
        if let Some(last) = tracked.pending.finish() {
            self.offer(vec![last]).await;
        }
        if !tracked.sealable {
            tracing::debug!(%path, "file capture never copied this file; nothing is sealed");
            return Ok(());
        }
        // Drop Any Interrupted Write Past the Committed Length
        open_spool(&tracked.spool, tracked.written, false).await?;
        let artifact = tracked.artifact;
        let record = if let Some(upload) = tracked.upload {
            upload.finish().await?
        } else {
            self.runtime
                .upload_artifact_file(
                    self.scope.clone(),
                    &tracked.spool,
                    artifact,
                    Some(media_type_for_path(path).unwrap_or("application/octet-stream").to_owned()),
                )
                .await?
        };
        self.sealed.push(record.clone());
        *self.segments.entry(path.to_owned()).or_default() += 1;
        events.push(CaptureEvent::Sealed {
            path: path.to_owned(),
            record: Box::new(record),
        });
        Ok(())
    }

    fn artifact_name(&self, path: &str) -> String {
        let base = match &self.artifact {
            Some(artifact) => artifact.clone(),
            None if self.output.is_empty() => path.to_owned(),
            None => format!("{}/{path}", self.output),
        };
        let base = if self.single { crate::exports::storage::paths::safe_artifact_path(&base) } else { base };
        match self.segments.get(path).copied().unwrap_or_default() {
            0 => base,
            segment => crate::files::segment_name(&base, segment + 1),
        }
    }

    fn select(&mut self, listing: &mut Listing) {
        if self.baseline.is_none() {
            let baseline = listing.keys().cloned().collect::<BTreeSet<_>>();
            if self.newest {
                self.selected = listing
                    .iter()
                    .max_by(|(left_path, left), (right_path, right)| {
                        left.modified.cmp(&right.modified).then_with(|| right_path.cmp(left_path))
                    })
                    .map(|(path, _)| path.clone());
            }
            self.baseline = Some(baseline);
        }
        if self.newest {
            listing.retain(|path, _| self.selected.as_ref() == Some(path));
        }
    }

    // Reports one gap to the caller and into the execution's evidence.
    async fn gap(&self, path: &str, offset: u64, reason: GapReason, events: &mut Vec<CaptureEvent>) {
        events.push(CaptureEvent::Gap { path: path.to_owned(), offset, reason });
        let mut record = RecordPublishRequest::new(BuiltinRecordType::FileCaptureGap.identifier(), RecordSeverity::Warning);
        record.details.insert("path".to_owned(), Value::Text(path.to_owned()));
        record.details.insert("offset".to_owned(), Value::Integer(offset.try_into().unwrap_or(i64::MAX)));
        record.details.insert("reason".to_owned(), Value::Text(super::gap_reason(reason).to_owned()));
        record.scope = Some(self.scope.clone());
        if let Err(error) = self.runtime.publish_record(record).await {
            tracing::warn!(%error, %path, "file capture gap record was refused");
        }
    }

    // Publishes what the line matchers find in these lines: rows to their
    // streams, markers as records, both in the captured execution's scope. A
    // refused publication is logged; it never ends the capture.
    async fn offer(&mut self, lines: Vec<String>) {
        let mut rows = BTreeMap::<String, Vec<TelemetrySample>>::new();
        let mut records = Vec::new();
        let now = crate::model::time::now_utc();
        for line in lines {
            for found in self.lines.matches(&line) {
                match found {
                    LineMatch::Row { stream, values, storage_schema } => {
                        let payload = match crate::telemetry::capture_payload(&storage_schema, &values, self.runtime.schema_settings()) {
                            Ok(payload) => payload,
                            Err(error) => {
                                tracing::warn!(%error, %stream, "file matcher row was rejected");
                                continue;
                            }
                        };
                        let Ok(identifier) = Identifier::new(&stream) else {
                            continue;
                        };
                        let sequence = self.sequences.entry(stream.clone()).or_default();
                        let mut sample = TelemetrySample::new(crate::model::identity::new_id(), identifier, *sequence, now.clone(), SOURCE);
                        *sequence += 1;
                        sample.scope = Some(self.scope.clone());
                        sample.values.extend(values);
                        sample.storage_payload = Some(payload);
                        rows.entry(stream).or_default().push(sample);
                    }
                    LineMatch::Marker { record_type, details, line } => {
                        let Ok(record_type) = Identifier::new(&record_type) else {
                            continue;
                        };
                        let mut record = RecordPublishRequest::new(record_type, RecordSeverity::Info);
                        record.message = Some(line);
                        record.details = details;
                        record.scope = Some(self.scope.clone());
                        records.push(record);
                    }
                }
            }
        }
        for (stream, samples) in rows {
            let Ok(identifier) = Identifier::new(&stream) else {
                continue;
            };
            if let Err(error) =
                self.runtime.publish_telemetry_batch(TelemetryBatch::new(crate::model::identity::new_id(), identifier, samples)).await
            {
                tracing::warn!(%error, %stream, "file capture rows were refused");
            }
        }
        for record in records {
            let record_type = record.record_type.clone();
            if let Err(error) = self.runtime.publish_record(record).await {
                tracing::warn!(%error, %record_type, "file capture marker was refused");
            }
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

async fn source_changed(changes: &mut Option<harmos::storage::DirectoryWatcher>) {
    let Some(watcher) = changes else { return std::future::pending().await };
    match watcher.next().await {
        Ok(Some(_)) => {}
        result => {
            if let Err(error) = result {
                tracing::warn!(%error, "native file notifications stopped; configured polling remains active");
            }
            *changes = None;
            std::future::pending().await
        }
    }
}

fn over_bytes(maximum: u64, path: &str) -> Error {
    capped(format!("the capture holds at most {maximum} bytes; {path:?} would exceed it"))
}

// The `length` bytes a capture appended at `from`.
fn read_back(spool: &TempPath, from: u64, length: u64) -> Result<Vec<u8>> {
    let failed = |source: std::io::Error| Error::Operation(format!("file capture spool: {source}"));
    let mut file = std::fs::File::open(spool).map_err(failed)?;
    file.seek(std::io::SeekFrom::Start(from)).map_err(failed)?;
    let mut bytes = Vec::new();
    file.take(length).read_to_end(&mut bytes).map_err(failed)?;
    Ok(bytes)
}

// Opens a spool for appending at `length`, or empty when `truncate` is set.
async fn open_spool(path: &TempPath, length: u64, truncate: bool) -> Result<tokio::fs::File> {
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .await
        .map_err(|source| Error::Operation(format!("file capture spool: {source}")))?;
    let length = if truncate { 0 } else { length };
    file.set_len(length).await.map_err(|source| Error::Operation(format!("file capture spool: {source}")))?;
    file.seek(std::io::SeekFrom::Start(length)).await.map_err(|source| Error::Operation(format!("file capture spool: {source}")))?;
    Ok(file)
}

/* === Tests === */
