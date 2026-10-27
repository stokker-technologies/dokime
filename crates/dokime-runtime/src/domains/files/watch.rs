/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// One running file watch. Each tracked file owns a private spool that holds
// exactly its captured bytes; sealing uploads the spool as the artifact
// `<output>/<relative path>`, so length and SHA-256 come from the artifact
// path. The caller drives the watch: `poll` once per interval, `seal` at the
// end. Closing the execution asks the watch to seal, and dropping an unsealed
// watch seals it in the background. A watch with line matchers offers each
// line the Stream policy captures to them, publishing rows to the matched
// streams and markers as records in the watched execution's scope.

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
use super::{GapReason, Phase, Policy, Selection, Settings, capped, invalid, unreachable_source};
use crate::artifacts::ArtifactRecord;
use crate::execution_runtime::outputs::media_type_for_path;
use crate::files::{FileMatcher, LineBuffer, LineMatch, LineMatchers};
use crate::records::RecordSeverity;
use crate::runtime::storage::captures::SealRequest;
use crate::telemetry::{TelemetryBatch, TelemetrySample};
use crate::{RuntimeHandle, runtime::Error, runtime::Result};
use dokime_domain::model::artifacts::name;
use dokime_domain::model::diagnostics::admission::AdmissionFailure;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::records::publication::RecordPublishRequest;

/* === Constants === */

// The producer named on the rows and records line matchers publish.
const SOURCE: &str = "dokime.files";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct FileWatch {
    // Taken by `Drop` to seal an abandoned watch in the background.
    state: Option<State>,
}

struct State {
    runtime: RuntimeHandle,
    scope: RecordScope,
    execution: TestExecutionId,
    output: String,
    artifact: Option<String>,
    policy: Policy,
    selection: Selection,
    baseline: Option<BTreeSet<String>>,
    selected: Option<String>,
    wait_for_source: Option<Duration>,
    waiting_since: tokio::time::Instant,
    armed_at: Option<tokio::time::Instant>,
    single: bool,
    segments: BTreeMap<String, u64>,
    names: BTreeSet<String>,
    matcher: FileMatcher,
    lines: LineMatchers,
    sequences: BTreeMap<String, u64>,
    source: Source,
    poll_interval: Duration,
    deadline: Option<Duration>,
    maximum_files: usize,
    maximum_bytes: u64,
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

#[derive(Debug)]
pub enum WatchEvent {
    Armed { baseline: Vec<String> },
    Progress { path: String, offset: u64, phase: Phase },
    Gap { path: String, offset: u64, reason: GapReason },
    Sealed { path: String, record: Box<ArtifactRecord> },
}

/* === Traits === */

/* === Implementations === */

impl FileWatch {
    pub(super) async fn open(runtime: RuntimeHandle, settings: Settings, source: Source) -> Result<Self> {
        let execution = settings.scope.test_execution_id.clone().ok_or_else(|| invalid("a file watch requires an execution"))?;
        let spools = tempfile::Builder::new()
            .prefix("dokime-watch-")
            .tempdir()
            .map_err(|source| Error::Operation(format!("file watch spool: {source}")))?;

        // Seal Into The Execution Before It Stops Admitting Artifacts
        let (seal_sender, seal_request) = oneshot::channel();
        let registration = runtime.execution_storage.captures.register(&execution, seal_sender as SealRequest);
        Ok(Self {
            state: Some(State {
                runtime,
                scope: settings.scope,
                execution,
                output: settings.output,
                artifact: settings.artifact,
                policy: settings.policy,
                selection: settings.selection,
                baseline: None,
                selected: None,
                wait_for_source: settings.wait_for_source,
                waiting_since: tokio::time::Instant::now(),
                armed_at: None,
                single: settings.single,
                segments: BTreeMap::new(),
                names: BTreeSet::new(),
                matcher: settings.matcher,
                lines: settings.lines,
                sequences: BTreeMap::new(),
                source,
                poll_interval: settings.poll_interval,
                deadline: settings.deadline,
                maximum_files: settings.maximum_files,
                maximum_bytes: settings.maximum_bytes,
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
        })
    }

    pub fn policy(&self) -> Policy {
        self.state().policy
    }

    pub fn poll_interval(&self) -> Duration {
        let state = self.state();
        if state.baseline.is_none()
            && let Some(wait) = state.wait_for_source
        {
            return state.poll_interval.min(wait.saturating_sub(state.waiting_since.elapsed()));
        }
        state.poll_interval
    }

    pub fn remaining(&self) -> Option<Duration> {
        let state = self.state();
        state.deadline.zip(state.armed_at).map(|(deadline, armed)| deadline.saturating_sub(armed.elapsed()))
    }

    // Whether the watch has sealed every file it will capture.
    pub fn is_complete(&self) -> bool {
        self.state().complete
    }

    // The artifacts sealed so far, in sealing order.
    pub fn sealed(&self) -> &[ArtifactRecord] {
        &self.state().sealed
    }

    // One discovery and capture pass. Events are pushed even when the pass
    // fails, so frames already earned reach the caller.
    pub async fn poll(&mut self, events: &mut Vec<WatchEvent>) -> Result<()> {
        self.state_mut().poll(events).await
    }

    // Captures what remains, then seals every tracked file. Idempotent.
    pub async fn seal(&mut self, events: &mut Vec<WatchEvent>) -> Result<()> {
        self.state_mut().seal(events).await
    }

    // Resolves when the execution asks its running captures to seal.
    // Cancel-safe; it stays pending once answered or when no request can come.
    pub async fn seal_requested(&mut self) {
        let state = self.state_mut();
        if state.complete || state.seal_answer.is_some() {
            return std::future::pending().await;
        }
        match (&mut state.seal_request).await {
            Ok(answer) => state.seal_answer = Some(answer),
            Err(_) => std::future::pending().await,
        }
    }

    fn state(&self) -> &State {
        self.state.as_ref().expect("file watch state lives until drop")
    }

    fn state_mut(&mut self) -> &mut State {
        self.state.as_mut().expect("file watch state lives until drop")
    }
}

impl Drop for FileWatch {
    fn drop(&mut self) {
        let Some(mut state) = self.state.take() else {
            return;
        };
        if state.complete {
            return;
        }
        let Ok(runtime) = tokio::runtime::Handle::try_current() else {
            tracing::warn!(execution = %state.execution, "file watch dropped outside a runtime; its spools are discarded");
            state.release();
            return;
        };
        runtime.spawn(async move {
            let mut events = Vec::new();
            if let Err(source) = state.seal(&mut events).await {
                tracing::warn!(error = %source, "abandoned file watch failed to seal");
            }
        });
    }
}

impl State {
    async fn poll(&mut self, events: &mut Vec<WatchEvent>) -> Result<()> {
        if self.complete || self.failed {
            return Ok(());
        }
        let first = self.baseline.is_none();
        let beginning = events.len();
        self.passes += 1;
        let polled = match self.policy {
            Policy::SnapshotAtStart if first => match self.snapshot(events).await {
                Ok(()) if self.baseline.is_some() => self.finish(events).await,
                Ok(()) => Ok(()),
                Err(source) => Err(source),
            },
            Policy::SnapshotAtStop if first => self.listing().await.map(|_| ()),
            Policy::SnapshotAtStart | Policy::SnapshotAtStop | Policy::Unspecified => Ok(()),
            Policy::Stream | Policy::Mirror => self.track(first, events).await,
        };
        if polled.is_ok()
            && first
            && let Some(baseline) = &self.baseline
        {
            self.armed_at = Some(tokio::time::Instant::now());
            events.insert(beginning, WatchEvent::Armed { baseline: baseline.iter().cloned().collect() });
        }
        self.failed |= polled.is_err();
        polled
    }

    async fn seal(&mut self, events: &mut Vec<WatchEvent>) -> Result<()> {
        if self.complete {
            return Ok(());
        }

        // Capture The Last Bytes; A Failing Source Still Seals What It Gave
        let captured = match self.policy {
            _ if self.failed => Ok(()),
            _ if self.baseline.is_none() && self.wait_for_source.is_some() => Ok(()),
            Policy::SnapshotAtStart if self.passes == 0 => self.snapshot(events).await,
            Policy::SnapshotAtStop => self.snapshot(events).await,
            Policy::Stream | Policy::Mirror => {
                let first = self.passes == 0;
                self.track(first, events).await
            }
            Policy::SnapshotAtStart | Policy::Unspecified => Ok(()),
        };
        if let Err(source) = &captured {
            tracing::warn!(error = %source, execution = %self.execution, "file watch sealed after a failed final pass");
        }
        let sealed = self.finish(events).await;
        captured.and(sealed)
    }

    // Seals every tracked file, then answers the execution's seal request.
    async fn finish(&mut self, events: &mut Vec<WatchEvent>) -> Result<()> {
        let mut outcome = Ok(());
        let paths = self.tracked.keys().cloned().collect::<Vec<_>>();
        for path in paths {
            if let Err(source) = self.seal_file(&path, events).await {
                tracing::warn!(error = %source, %path, "file watch failed to seal a file");
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
    async fn track(&mut self, first: bool, events: &mut Vec<WatchEvent>) -> Result<()> {
        let Some(listing) = self.listing().await? else {
            return Ok(());
        };

        // Release Simultaneous Slots Before Admitting New Paths
        let vanished = self.tracked.keys().filter(|path| !listing.contains_key(*path)).cloned().collect::<Vec<_>>();
        for path in vanished {
            let offset = self.tracked[&path].written;
            events.push(WatchEvent::Gap { path: path.clone(), offset, reason: GapReason::Vanished });
            self.seal_file(&path, events).await?;
        }

        // Capture Each Matched File From Its Baseline
        for (path, entry) in &listing {
            // A Replaced Source Seals Its Segment; The New File Starts Afresh
            if let Some(tracked) = self.tracked.get(path) {
                let replaced = matches!(
                    (tracked.entry.identity, entry.identity),
                    (Some(before), Some(after)) if before != after
                );
                let truncated = entry.size < tracked.offset || (tracked.mirrored && entry.size < tracked.entry.size);
                if replaced || truncated {
                    let reason = if replaced { GapReason::Replaced } else { GapReason::Truncated };
                    events.push(WatchEvent::Gap { path: path.clone(), offset: tracked.written, reason });
                    self.seal_file(path, events).await?;
                }
            }
            if !self.tracked.contains_key(path) {
                let baseline = if first && self.policy == Policy::Stream && self.selection != Selection::New { entry.size } else { 0 };
                self.discover(path, *entry, baseline, events).await?;
            }
            match self.capture(path, *entry, events).await {
                Ok(()) => {}
                Err(source @ Error::Admission(AdmissionFailure::Unavailable { .. })) => return self.failed_pass(source),
                Err(source) => return Err(source),
            }
        }
        Ok(())
    }

    async fn listing(&mut self) -> Result<Option<Listing>> {
        let first = self.baseline.is_none();
        if first && self.wait_for_source.is_some_and(|wait| self.waiting_since.elapsed() >= wait) {
            return Err(Error::not_found("file_watch_missing", "source wait deadline elapsed before a matching file appeared"));
        }
        let mut listing = match self.source.scan(&self.matcher).await {
            Ok(listing) => {
                self.failed_passes = 0;
                listing
            }
            Err(source) if first && source.code() == Some("file_watch_missing") => {
                self.missing_source(source)?;
                return Ok(None);
            }
            Err(source @ Error::Admission(AdmissionFailure::Unavailable { .. })) => {
                self.failed_pass(source)?;
                return Ok(None);
            }
            Err(source) => return Err(source),
        };

        let requires_file = self.single || (self.selection == Selection::Latest && self.wait_for_source.is_some());
        if first && listing.is_empty() && requires_file {
            let source = Error::not_found("file_watch_missing", "the watched source has no matching file");
            self.missing_source(source)?;
            return Ok(None);
        }
        self.select(&mut listing);

        Ok(Some(listing))
    }

    // A source that keeps failing ends the watch; a passing glitch does not.
    fn failed_pass(&mut self, source: Error) -> Result<()> {
        self.failed_passes += 1;
        tracing::warn!(error = %source, attempt = self.failed_passes, "file watch pass failed");
        if self.failed_passes >= 10 {
            return Err(source);
        }
        Ok(())
    }

    // Starts tracking one file, refusing it explicitly past the caps.
    async fn discover(&mut self, path: &str, entry: Entry, baseline: u64, events: &mut Vec<WatchEvent>) -> Result<()> {
        if self.artifact.is_some() && !self.tracked.is_empty() {
            return Err(capped(format!("a watch into one named artifact tracks one file; {path:?} would be a second")));
        }
        if self.tracked.len() >= self.maximum_files {
            return Err(capped(format!("the watch tracks at most {} files; {path:?} would exceed it", self.maximum_files)));
        }
        let mut artifact = self.artifact_name(path);
        while self.names.contains(&artifact) {
            *self.segments.entry(path.to_owned()).or_default() += 1;
            artifact = self.artifact_name(path);
        }
        name::validate(&artifact).map_err(|source| invalid(format!("artifact {artifact:?}: {source}")))?;
        self.names.insert(artifact.clone());
        let spool = tempfile::NamedTempFile::new_in(self.spools.path())
            .map_err(|source| Error::Operation(format!("file watch spool: {source}")))?
            .into_temp_path();
        let line_bytes = 64 * 1024;
        let line_bytes = std::num::NonZeroUsize::new(line_bytes).ok_or_else(|| invalid("line byte capacity must be nonzero"))?;
        self.tracked.insert(
            path.to_owned(),
            Tracked {
                artifact,
                spool,
                written: 0,
                offset: baseline,
                entry,
                mirrored: self.policy == Policy::Mirror,
                synced: false,
                pending: LineBuffer::new(line_bytes),
                sealable: self.policy == Policy::Stream,
            },
        );
        events.push(WatchEvent::Progress { path: path.to_owned(), offset: 0, phase: Phase::Baseline });
        Ok(())
    }

    async fn capture(&mut self, path: &str, entry: Entry, events: &mut Vec<WatchEvent>) -> Result<()> {
        let tracked = self.tracked.get_mut(path).expect("captured files are tracked");
        if tracked.mirrored {
            return self.mirror(path, entry, events).await;
        }

        tracked.entry = entry;
        if entry.size <= tracked.offset {
            return Ok(());
        }

        // Append Only Bytes The Cap Admits
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
                events.push(WatchEvent::Progress { path: path.to_owned(), offset: tracked.written, phase: Phase::Capturing });
                drop(spool);
                if self.lines.is_empty() {
                    return Ok(());
                }
                // The Captured Bytes Are Read Back From The Spool As Lines
                let captured = read_back(&tracked.spool, tracked.written - copied, copied)?;
                let lines = tracked.pending.push(&captured);
                self.offer(lines).await;
                Ok(())
            }
            // A Source That Cannot Resume Mirrors This File From Now On
            Err(ReadFailure::ResumeUnsupported) => {
                events.push(WatchEvent::Gap { path: path.to_owned(), offset: tracked.written, reason: GapReason::ResumeUnsupported });
                if !self.lines.is_empty() {
                    return Err(Error::conflict(
                        super::UNSUPPORTED,
                        "FTP server refused REST resume; line extraction cannot continue. Enable REST on the server or use a resumable/local source; captured evidence will be sealed",
                    ));
                }
                tracked.mirrored = true;
                self.mirror(path, entry, events).await
            }
            Err(ReadFailure::Failed(source)) => Err(source),
        }
    }

    // Refetches the whole file when its size or modification time changed.
    async fn mirror(&mut self, path: &str, entry: Entry, events: &mut Vec<WatchEvent>) -> Result<()> {
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
            .map_err(|source| Error::Operation(format!("file watch spool: {source}")))?
            .into_temp_path();
        let mut spool = open_spool(&fresh, 0, true).await?;
        let copied = match self.source.read_range(path, 0, entry.size, &mut spool).await {
            Ok(copied) => copied,
            Err(ReadFailure::Failed(source)) => return Err(source),
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
        events.push(WatchEvent::Progress { path: path.to_owned(), offset: copied, phase: Phase::Synced });
        Ok(())
    }

    // Copies every matched file once, as it is now.
    async fn snapshot(&mut self, events: &mut Vec<WatchEvent>) -> Result<()> {
        let Some(listing) = self.listing().await? else {
            return Ok(());
        };
        for (path, entry) in listing {
            self.discover(&path, entry, 0, events).await?;
            self.tracked.get_mut(&path).expect("discovered files are tracked").mirrored = true;
            self.mirror(&path, entry, events).await?;
        }
        Ok(())
    }

    // Uploads one spool as the artifact `<output>/<path>` and stops tracking it.
    async fn seal_file(&mut self, path: &str, events: &mut Vec<WatchEvent>) -> Result<()> {
        let Some(mut tracked) = self.tracked.remove(path) else {
            return Ok(());
        };
        if let Some(last) = tracked.pending.finish() {
            self.offer(vec![last]).await;
        }
        if !tracked.sealable {
            tracing::warn!(%path, "file watch never copied this file; nothing is sealed");
            return Ok(());
        }
        // Drop Any Interrupted Write Past The Committed Length
        open_spool(&tracked.spool, tracked.written, false).await?;
        let artifact = tracked.artifact;
        let record = self
            .runtime
            .upload_artifact_file(self.scope.clone(), &tracked.spool, artifact, media_type_for_path(path).map(str::to_owned))
            .await?;
        self.sealed.push(record.clone());
        *self.segments.entry(path.to_owned()).or_default() += 1;
        events.push(WatchEvent::Sealed { path: path.to_owned(), record: Box::new(record) });
        Ok(())
    }

    fn artifact_name(&self, path: &str) -> String {
        let base = match &self.artifact {
            Some(artifact) => artifact.clone(),
            None if self.output.is_empty() => path.to_owned(),
            None => format!("{}/{path}", self.output),
        };
        let base = crate::exports::storage::paths::safe_artifact_path(&base);
        match self.segments.get(path).copied().unwrap_or_default() {
            0 => base,
            segment => crate::files::segment_name(&base, segment + 1),
        }
    }

    fn missing_source(&self, source: Error) -> Result<()> {
        match self.wait_for_source {
            Some(wait) if self.waiting_since.elapsed() < wait => Ok(()),
            Some(_) => Err(Error::not_found("file_watch_missing", "source wait deadline elapsed before a matching file appeared")),
            None => Err(source),
        }
    }

    fn select(&mut self, listing: &mut Listing) {
        if self.baseline.is_none() {
            let baseline = listing.keys().cloned().collect::<BTreeSet<_>>();
            if self.selection == Selection::Latest {
                self.selected = listing
                    .iter()
                    .max_by(|(left_path, left), (right_path, right)| {
                        left.modified.cmp(&right.modified).then_with(|| right_path.cmp(left_path))
                    })
                    .map(|(path, _)| path.clone());
            }
            self.baseline = Some(baseline);
        }
        match self.selection {
            Selection::All => {}
            Selection::Latest => listing.retain(|path, _| self.selected.as_ref() == Some(path)),
            Selection::New => listing.retain(|path, _| !self.baseline.as_ref().expect("baseline established").contains(path)),
        }
    }

    // Publishes what the line matchers find in these lines: rows to their
    // streams, markers as records, both in the watched execution's scope. A
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
                tracing::warn!(%error, %stream, "file watch rows were refused");
            }
        }
        for record in records {
            let record_type = record.record_type.clone();
            if let Err(error) = self.runtime.publish_record(record).await {
                tracing::warn!(%error, %record_type, "file watch marker was refused");
            }
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn over_bytes(maximum: u64, path: &str) -> Error {
    capped(format!("the watch captures at most {maximum} bytes; {path:?} would exceed it"))
}

// The `length` bytes a capture appended at `from`.
fn read_back(spool: &TempPath, from: u64, length: u64) -> Result<Vec<u8>> {
    let failed = |source: std::io::Error| Error::Operation(format!("file watch spool: {source}"));
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
        .map_err(|source| Error::Operation(format!("file watch spool: {source}")))?;
    let length = if truncate { 0 } else { length };
    file.set_len(length).await.map_err(|source| Error::Operation(format!("file watch spool: {source}")))?;
    file.seek(std::io::SeekFrom::Start(length)).await.map_err(|source| Error::Operation(format!("file watch spool: {source}")))?;
    Ok(file)
}

/* === Tests === */
