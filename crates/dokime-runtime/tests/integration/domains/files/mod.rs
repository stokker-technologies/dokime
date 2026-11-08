/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// File captures over a running execution: a local tree that grows, rotates and
// gains files, and an in-process FTP server; and live follows of native outputs.

/* === Modules === */

pub(crate) mod capture;
pub(crate) mod follow;
pub(crate) mod ftp;

/* === Re-Exports === */

pub(crate) use crate::service_support::prelude::*;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use sha2::{Digest, Sha256};
use {
    model::files::wire::CaptureMode, model::files::wire::FileRule, model::files::wire::FileSource, model::files::wire::MirrorCapture,
    model::files::wire::StreamCapture,
};

// Local Imports
use dokime_domain::model;
use dokime_operations::files::StartCapture;
use dokime_runtime::artifacts::{ArtifactRecord, ArtifactStatus};
use dokime_runtime::domains::files::{CaptureEvent, FileCapture};

/* === Constants === */

// The artifact folder every story captures into.
pub(crate) const OUTPUT: &str = "capture";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// A runtime whose only execution stays running until released.
pub(crate) struct Bench {
    pub(crate) owner: crate::service_support::system_reports::Runtime,
    pub(crate) handle: RuntimeHandle,
    pub(crate) executor: Arc<BlockingPublishingExecutor>,
    pub(crate) execution: String,
    pub(crate) _root: tempfile::TempDir,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Bench {
    pub(crate) async fn start(configure: impl FnOnce(RuntimeConfig, &Path) -> RuntimeConfig) -> Self {
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

// A capture over a tree into the output every story reads.
pub(crate) fn request(root: &Path, rules: impl IntoIterator<Item = FileRule>, mode: impl Into<CaptureMode>) -> StartCapture {
    StartCapture::new(FileSource::tree(root, rules), OUTPUT, mode)
}

pub(crate) fn stream() -> StreamCapture {
    StreamCapture::plain()
}

// Stories drive a Mirror's passes themselves.
pub(crate) fn mirror() -> MirrorCapture {
    MirrorCapture::every(3600)
}

pub(crate) async fn poll(capture: &mut FileCapture) -> Vec<CaptureEvent> {
    let mut events = Vec::new();
    capture.poll(&mut events).await.unwrap();
    events
}

pub(crate) async fn seal(capture: &mut FileCapture) -> Vec<CaptureEvent> {
    let mut events = Vec::new();
    capture.seal(&mut events).await.unwrap();
    events
}

// The artifacts sealed by these events, by path relative to the root.
pub(crate) fn sealed(events: &[CaptureEvent]) -> BTreeMap<String, ArtifactRecord> {
    events
        .iter()
        .filter_map(|event| match event {
            CaptureEvent::Sealed { path, record } => Some((path.clone(), (**record).clone())),
            _ => None,
        })
        .collect()
}

// A sealed artifact holds exactly these bytes under `<output>/<path>`.
pub(crate) fn assert_artifact(record: &ArtifactRecord, path: &str, bytes: &[u8]) {
    assert_eq!(record.filename, format!("{OUTPUT}/{path}"));
    assert_eq!(record.status, ArtifactStatus::Complete, "{path}");
    assert_eq!(record.size_bytes, bytes.len() as u64, "{path}");
    let digest = Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
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
