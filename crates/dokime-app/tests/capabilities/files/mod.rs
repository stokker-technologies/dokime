/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// File watches over a running execution: a local tree that grows, rotates and
// gains files, and an in-process FTP server.

/* === Modules === */
pub(crate) mod ftp;
pub(crate) mod watch;

/* === Re-Exports === */
pub(crate) use crate::support::prelude::*;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use sha2::{Digest, Sha256};

// Local Imports
use dokime::artifacts::{ArtifactRecord, ArtifactStatus};
use dokime_app::capabilities::files::{FileWatch, WatchEvent};
use dokime_contract::operations::files::{
    FileRule, FileRuleAction, FileWatchPolicy, FileWatchRequest, FileWatchSource,
    FileWatchSourcePayload, LocalWatchSource,
};

/* === Constants === */
// The artifact folder every story captures into.
pub(crate) const OUTPUT: &str = "capture";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// A runtime whose only execution stays running until released.
pub(crate) struct Bench {
    pub(crate) _root: tempfile::TempDir,
    pub(crate) owner: Runtime,
    pub(crate) handle: RuntimeHandle,
    pub(crate) executor: Arc<BlockingPublishingExecutor>,
    pub(crate) execution: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Bench {
    pub(crate) async fn start(
        configure: impl FnOnce(RuntimeConfig, &Path) -> RuntimeConfig,
    ) -> Self {
        let root = tempdir().unwrap();
        let settings = configure(config(root.path()), root.path());
        let (owner, handle, cycle_id, executor) = blocked_cycle_with(settings).await;
        let cycle = wait_for_cycle_lifecycle(&handle, &cycle_id, CycleStatus::Running).await;
        let execution = cycle.test_executions[0].test_execution_id.to_string();
        Self {
            _root: root,
            owner,
            handle,
            executor,
            execution,
        }
    }

    pub(crate) async fn stop(mut self) {
        self.executor.release();
        self.owner.shutdown().await.unwrap();
    }
}

/* === Public Functions === */
pub(crate) fn request(
    execution: &str,
    source: FileWatchSourcePayload,
    rules: Vec<FileRule>,
    policy: FileWatchPolicy,
) -> FileWatchRequest {
    FileWatchRequest {
        execution_id: Some(execution.to_owned()),
        source: Some(FileWatchSource {
            payload: Some(source),
        }),
        rules,
        output: OUTPUT.to_owned(),
        policy,
        poll_interval_ms: Some(50),
        maximum_seconds: None,
        maximum_files: None,
        maximum_bytes: None,
    }
}

pub(crate) fn local(root: &Path) -> FileWatchSourcePayload {
    FileWatchSourcePayload::Local(LocalWatchSource {
        root: root.to_str().unwrap().to_owned(),
    })
}

pub(crate) fn include(glob: &str) -> FileRule {
    FileRule {
        glob: glob.to_owned(),
        action: FileRuleAction::Include,
    }
}

pub(crate) fn exclude(glob: &str) -> FileRule {
    FileRule {
        glob: glob.to_owned(),
        action: FileRuleAction::Exclude,
    }
}

pub(crate) async fn poll(watch: &mut FileWatch) -> Vec<WatchEvent> {
    let mut events = Vec::new();
    watch.poll(&mut events).await.unwrap();
    events
}

pub(crate) async fn seal(watch: &mut FileWatch) -> Vec<WatchEvent> {
    let mut events = Vec::new();
    watch.seal(&mut events).await.unwrap();
    events
}

// The artifacts sealed by these events, by path relative to the root.
pub(crate) fn sealed(events: &[WatchEvent]) -> BTreeMap<String, ArtifactRecord> {
    events
        .iter()
        .filter_map(|event| match event {
            WatchEvent::Sealed { path, record } => Some((path.clone(), (**record).clone())),
            _ => None,
        })
        .collect()
}

// A sealed artifact holds exactly these bytes under `<output>/<path>`.
pub(crate) fn assert_artifact(record: &ArtifactRecord, path: &str, bytes: &[u8]) {
    assert_eq!(record.filename, format!("{OUTPUT}/{path}"));
    assert_eq!(record.status, ArtifactStatus::Complete, "{path}");
    assert_eq!(record.size_bytes, bytes.len() as u64, "{path}");
    let digest = Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(record.sha256.as_deref(), Some(digest.as_str()), "{path}");
}

pub(crate) fn write(path: &Path, bytes: &[u8]) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, bytes).unwrap();
}

pub(crate) fn append(path: &Path, bytes: &[u8]) {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new().append(true).open(path).unwrap();
    file.write_all(bytes).unwrap();
}

/* === Private Functions === */

/* === Tests === */
