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
// watch seals it in the background.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, time::Duration};

// Third Party Imports
use tempfile::TempPath;
use tokio::{io::AsyncSeekExt, sync::oneshot};

// Local Imports
use super::source::{Entry, Listing, ReadFailure, Source};
use super::{GapReason, Phase, Policy, Settings, capped, invalid, unreachable_source};
use crate::execution_runtime::outputs::media_type_for_path;
use crate::runtime::storage::captures::SealRequest;
use crate::{Error, Result, RuntimeHandle};
use dokime::artifacts::ArtifactRecord;
use dokime::core::{RecordScope, TestExecutionId};
use dokime_contract::artifacts::name;
use dokime_contract::operations::files::FileMatcher;

/* === Constants === */
// Consecutive failed passes after which an unreachable source ends the watch.
const FAILED_PASSES: u32 = 10;

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
    policy: Policy,
    matcher: FileMatcher,
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
    files: usize,
    passes: u64,
    failed_passes: u32,
    complete: bool,
    // A refused pass ends capture; sealing keeps only what was captured.
    failed: bool,
    registration: u64,
    seal_request: oneshot::Receiver<oneshot::Sender<()>>,
    seal_answer: Option<oneshot::Sender<()>>,
}

struct Tracked {
    spool: TempPath,
    // Bytes in the spool; anything past it is an interrupted write.
    written: u64,
    // Stream: the next source byte to capture.
    offset: u64,
    entry: Entry,
    // Mirror policy, or a Stream file whose source cannot resume.
    mirrored: bool,
    synced: bool,
    // The spool holds a whole copy or a captured prefix worth sealing; a
    // mirrored file never fetched holds nothing.
    sealable: bool,
}

/* === Enums === */
#[derive(Debug)]
pub enum WatchEvent {
    Progress {
        path: String,
        offset: u64,
        phase: Phase,
    },
    Gap {
        path: String,
        offset: u64,
        reason: GapReason,
    },
    Sealed {
        path: String,
        record: Box<ArtifactRecord>,
    },
}

/* === Traits === */

/* === Implementations === */
impl FileWatch {
    pub(super) async fn open(
        runtime: RuntimeHandle,
        settings: Settings,
        source: Source,
    ) -> Result<Self> {
        let execution = settings
            .scope
            .test_execution_id
            .clone()
            .ok_or_else(|| invalid("a file watch requires an execution"))?;
        let spools = tempfile::Builder::new()
            .prefix("dokime-watch-")
            .tempdir()
            .map_err(|source| Error::Operation(format!("file watch spool: {source}")))?;

        // Seal Into The Execution Before It Stops Admitting Artifacts
        let (seal_sender, seal_request) = oneshot::channel();
        let registration = runtime
            .execution_storage
            .captures
            .register(&execution, seal_sender as SealRequest);
        Ok(Self {
            state: Some(State {
                runtime,
                scope: settings.scope,
                execution,
                output: settings.output,
                policy: settings.policy,
                matcher: settings.matcher,
                source,
                poll_interval: settings.poll_interval,
                deadline: settings.deadline,
                maximum_files: settings.maximum_files,
                maximum_bytes: settings.maximum_bytes,
                spools,
                tracked: BTreeMap::new(),
                captured: 0,
                sealed: Vec::new(),
                files: 0,
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
        self.state().poll_interval
    }

    // The bound after which the caller seals the watch, if the request set one.
    pub fn deadline(&self) -> Option<Duration> {
        self.state().deadline
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
        self.state
            .as_ref()
            .expect("file watch state lives until drop")
    }

    fn state_mut(&mut self) -> &mut State {
        self.state
            .as_mut()
            .expect("file watch state lives until drop")
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
        let first = self.passes == 0;
        self.passes += 1;
        let polled = match self.policy {
            Policy::SnapshotAtStart if first => match self.snapshot(events).await {
                Ok(()) => self.finish(events).await,
                Err(source) => Err(source),
            },
            Policy::SnapshotAtStart | Policy::SnapshotAtStop | Policy::Unspecified => Ok(()),
            Policy::Stream | Policy::Mirror => self.track(first, events).await,
        };
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
        self.runtime
            .execution_storage
            .captures
            .release(&self.execution, self.registration);
        if let Some(answer) = self.seal_answer.take() {
            let _ = answer.send(());
        }
    }

    // Stream and Mirror: discover, capture, and seal files that vanished.
    async fn track(&mut self, first: bool, events: &mut Vec<WatchEvent>) -> Result<()> {
        let listing = match self.source.scan(&self.matcher).await {
            Ok(listing) => {
                self.failed_passes = 0;
                listing
            }
            Err(source) => return self.failed_pass(source),
        };

        // Capture Each Matched File From Its Baseline
        for (path, entry) in &listing {
            // A Replaced Source Seals Its Segment; The New File Starts Afresh
            if let Some(tracked) = self.tracked.get(path)
                && !tracked.mirrored
                && matches!(
                    (tracked.entry.identity, entry.identity),
                    (Some(before), Some(after)) if before != after
                )
            {
                events.push(WatchEvent::Gap {
                    path: path.clone(),
                    offset: tracked.written,
                    reason: GapReason::Replaced,
                });
                self.seal_file(path, events).await?;
            }
            if !self.tracked.contains_key(path) {
                let baseline = if first && self.policy == Policy::Stream {
                    entry.size
                } else {
                    0
                };
                self.discover(path, *entry, baseline, events).await?;
            }
            match self.capture(path, *entry, events).await {
                Ok(()) => {}
                Err(source @ Error::Unavailable { .. }) => return self.failed_pass(source),
                Err(source) => return Err(source),
            }
        }

        // A Vanished File Is Sealed With A Gap
        let vanished = self
            .tracked
            .keys()
            .filter(|path| !listing.contains_key(*path))
            .cloned()
            .collect::<Vec<_>>();
        for path in vanished {
            let offset = self.tracked[&path].written;
            events.push(WatchEvent::Gap {
                path: path.clone(),
                offset,
                reason: GapReason::Vanished,
            });
            self.seal_file(&path, events).await?;
        }
        Ok(())
    }

    // A source that keeps failing ends the watch; a passing glitch does not.
    fn failed_pass(&mut self, source: Error) -> Result<()> {
        self.failed_passes += 1;
        tracing::warn!(error = %source, attempt = self.failed_passes, "file watch pass failed");
        if self.failed_passes >= FAILED_PASSES {
            return Err(source);
        }
        Ok(())
    }

    // Starts tracking one file, refusing it explicitly past the caps.
    async fn discover(
        &mut self,
        path: &str,
        entry: Entry,
        baseline: u64,
        events: &mut Vec<WatchEvent>,
    ) -> Result<()> {
        if self.files >= self.maximum_files {
            return Err(capped(format!(
                "the watch tracks at most {} files; {path:?} would exceed it",
                self.maximum_files
            )));
        }
        let artifact = self.artifact_name(path);
        name::validate(&artifact)
            .map_err(|source| invalid(format!("artifact {artifact:?}: {source}")))?;
        let spool = tempfile::NamedTempFile::new_in(self.spools.path())
            .map_err(|source| Error::Operation(format!("file watch spool: {source}")))?
            .into_temp_path();
        self.files += 1;
        self.tracked.insert(
            path.to_owned(),
            Tracked {
                spool,
                written: 0,
                offset: baseline,
                entry,
                mirrored: self.policy == Policy::Mirror,
                synced: false,
                sealable: self.policy == Policy::Stream,
            },
        );
        events.push(WatchEvent::Progress {
            path: path.to_owned(),
            offset: 0,
            phase: Phase::Baseline,
        });
        Ok(())
    }

    async fn capture(
        &mut self,
        path: &str,
        entry: Entry,
        events: &mut Vec<WatchEvent>,
    ) -> Result<()> {
        let tracked = self
            .tracked
            .get_mut(path)
            .expect("captured files are tracked");
        if tracked.mirrored {
            return self.mirror(path, entry, events).await;
        }

        // A Truncated Source Restarts At Its First Byte
        if entry.size < tracked.offset {
            tracked.offset = 0;
            events.push(WatchEvent::Gap {
                path: path.to_owned(),
                offset: tracked.written,
                reason: GapReason::Truncated,
            });
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
        match self
            .source
            .read_range(path, tracked.offset, length, &mut spool)
            .await
        {
            Ok(copied) => {
                tracked.offset += copied;
                tracked.written += copied;
                self.captured += copied;
                if copied > 0 {
                    events.push(WatchEvent::Progress {
                        path: path.to_owned(),
                        offset: tracked.written,
                        phase: Phase::Capturing,
                    });
                }
                Ok(())
            }
            // A Source That Cannot Resume Mirrors This File From Now On
            Err(ReadFailure::ResumeUnsupported) => {
                tracked.mirrored = true;
                events.push(WatchEvent::Gap {
                    path: path.to_owned(),
                    offset: tracked.written,
                    reason: GapReason::ResumeUnsupported,
                });
                self.mirror(path, entry, events).await
            }
            Err(ReadFailure::Failed(source)) => Err(source),
        }
    }

    // Refetches the whole file when its size or modification time changed.
    async fn mirror(
        &mut self,
        path: &str,
        entry: Entry,
        events: &mut Vec<WatchEvent>,
    ) -> Result<()> {
        let tracked = self
            .tracked
            .get_mut(path)
            .expect("mirrored files are tracked");
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
        let copied = match self
            .source
            .read_range(path, 0, entry.size, &mut spool)
            .await
        {
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
        events.push(WatchEvent::Progress {
            path: path.to_owned(),
            offset: copied,
            phase: Phase::Synced,
        });
        Ok(())
    }

    // Copies every matched file once, as it is now.
    async fn snapshot(&mut self, events: &mut Vec<WatchEvent>) -> Result<()> {
        let listing: Listing = self.source.scan(&self.matcher).await?;
        for (path, entry) in listing {
            self.discover(&path, entry, 0, events).await?;
            self.tracked
                .get_mut(&path)
                .expect("discovered files are tracked")
                .mirrored = true;
            self.mirror(&path, entry, events).await?;
        }
        Ok(())
    }

    // Uploads one spool as the artifact `<output>/<path>` and stops tracking it.
    async fn seal_file(&mut self, path: &str, events: &mut Vec<WatchEvent>) -> Result<()> {
        let Some(tracked) = self.tracked.remove(path) else {
            return Ok(());
        };
        if !tracked.sealable {
            tracing::warn!(%path, "file watch never copied this file; nothing is sealed");
            return Ok(());
        }
        // Drop Any Interrupted Write Past The Committed Length
        open_spool(&tracked.spool, tracked.written, false).await?;
        let artifact = self.artifact_name(path);
        let record = self
            .runtime
            .upload_artifact_file(
                self.scope.clone(),
                &tracked.spool,
                artifact,
                media_type_for_path(path).map(str::to_owned),
            )
            .await?;
        self.sealed.push(record.clone());
        events.push(WatchEvent::Sealed {
            path: path.to_owned(),
            record: Box::new(record),
        });
        Ok(())
    }

    fn artifact_name(&self, path: &str) -> String {
        format!("{}/{path}", self.output)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn over_bytes(maximum: u64, path: &str) -> Error {
    capped(format!(
        "the watch captures at most {maximum} bytes; {path:?} would exceed it"
    ))
}

// Opens a spool for appending at `length`, or empty when `truncate` is set.
async fn open_spool(path: &TempPath, length: u64, truncate: bool) -> Result<tokio::fs::File> {
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .await
        .map_err(|source| Error::Operation(format!("file watch spool: {source}")))?;
    let length = if truncate { 0 } else { length };
    file.set_len(length)
        .await
        .map_err(|source| Error::Operation(format!("file watch spool: {source}")))?;
    file.seek(std::io::SeekFrom::Start(length))
        .await
        .map_err(|source| Error::Operation(format!("file watch spool: {source}")))?;
    Ok(file)
}

/* === Tests === */
