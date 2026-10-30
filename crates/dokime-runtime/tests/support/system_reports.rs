/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Shared System Story Teardown. Integration And Unit Fixtures Keep Their Native Lifecycle.
#![allow(dead_code)]

/* === Modules === */
/* === Re-Exports === */
/* === Imports === */
// Standard Library Imports
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    ops::{Deref, DerefMut},
    path::Path,
};
// Third Party Imports
use sha2::{Digest, Sha256};
// Local Imports
use dokime_runtime::ExecutionOutputContent;
use dokime_runtime::{Runtime as NativeRuntime, RuntimeHandle};

/* === Constants === */
const EXEMPTIONS: &str = include_str!("system_reports.json");

/* === Macros === */
/* === Type Aliases === */
/* === Structs === */
pub struct Artifact {
    pub size_bytes: u64,
    pub sha256: String,
}

pub struct Runtime {
    owner: NativeRuntime,
    finalize: bool,
}

/* === Enums === */
/* === Traits === */
/* === Implementations === */
impl From<NativeRuntime> for Runtime {
    fn from(owner: NativeRuntime) -> Self {
        Self { owner, finalize: required() }
    }
}

impl Deref for Runtime {
    type Target = NativeRuntime;
    fn deref(&self) -> &Self::Target {
        &self.owner
    }
}

impl DerefMut for Runtime {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.owner
    }
}

impl Runtime {
    pub async fn shutdown(&mut self) -> dokime_runtime::runtime::Result<dokime_runtime::runtime::query::RuntimeOverviewSnapshot> {
        if self.finalize {
            let handle = self.owner.handle();
            Box::pin(finalize(&handle)).await;
            self.finalize = false;
        }
        self.owner.shutdown().await
    }
}

/* === Public Functions === */
pub fn required() -> bool {
    let executable = std::env::current_exe().unwrap();
    let name = executable.file_stem().unwrap().to_str().unwrap();
    let suffix = name.strip_prefix("system-");
    let is_system = suffix.is_some_and(system_suffix);
    if name != "system" && !is_system {
        return false;
    }

    let thread = std::thread::current();
    let story = std::env::var("NEXTEST_TEST_NAME").ok();
    let story = story.as_deref().or(thread.name()).expect("system story identity");
    let exemptions: BTreeMap<String, BTreeMap<String, String>> = serde_json::from_str(EXEMPTIONS).unwrap();
    let package = exemptions.get(env!("CARGO_PKG_NAME"));
    let reason = package.and_then(|stories| stories.get(story));
    if let Some(reason) = reason {
        eprintln!("execution report exemption: {story}: {reason}");
        return false;
    }
    true
}

pub fn payload(path: &Path) -> serde_json::Value {
    let html = std::fs::read_to_string(path).unwrap();
    let json = html.split_once(r#"<script id="hub-report-data" type="application/json">"#).unwrap().1.split_once("</script>").unwrap().0;
    serde_json::from_str(json).unwrap()
}

pub fn verify_file(path: &Path, size: u64, sha256: &str) {
    let mut file = std::fs::File::open(path).unwrap();
    assert_eq!(file.metadata().unwrap().len(), size);
    let mut digest = Sha256::new();
    let mut buffer = [0; 64 * 1024];
    let mut length = file.read(&mut buffer).unwrap();
    while length != 0 {
        digest.update(&buffer[..length]);
        length = file.read(&mut buffer).unwrap();
    }
    assert_eq!(hex(digest.finalize().as_slice()), sha256);
}

pub fn reopen(path: &Path, id: &str, expected_artifacts: &BTreeMap<String, Artifact>) {
    let report = payload(path);
    let executions = report["test_executions"].as_array().unwrap();
    assert_eq!(executions.len(), 1);
    assert_eq!(executions[0]["test_execution_id"], id);

    let artifacts = report["snapshot"]["artifacts"].as_array().unwrap();
    assert_eq!(artifacts.len(), expected_artifacts.len());
    let mut seen = std::collections::BTreeSet::new();
    for artifact in artifacts {
        let id = artifact["artifact_id"].as_str().unwrap();
        assert!(seen.insert(id));
        let expected = &expected_artifacts[id];
        assert_eq!(artifact["size_bytes"].as_u64().unwrap(), expected.size_bytes);
        let relative = artifact["path"].as_str().unwrap();
        dokime_runtime::validate_relative_output_path(relative).unwrap();
        let href = artifact["href"].as_str().unwrap();
        assert_eq!(href, format!("outputs/{relative}"));
        assert_eq!(artifact["downloadHref"], href);
        verify_file(&path.parent().unwrap().join(href), expected.size_bytes, &expected.sha256);
    }
}

pub async fn finalize(handle: &RuntimeHandle) {
    let root = tempfile::tempdir().unwrap();
    let cycles = handle.cycles().await;
    let mut saved = 0;
    let mut artifacts_saved = 0;
    for cycle in cycles {
        let (reports, artifacts) = save_cycle(handle, cycle, root.path()).await;
        saved += reports;
        artifacts_saved += artifacts;
    }

    assert!(saved > 0, "report-producing system story has no executions; exempt no-report stories centrally");
    eprintln!("reopened {saved} sealed execution reports with {artifacts_saved} artifacts");
}

/* === Private Functions === */
fn system_suffix(hash: &str) -> bool {
    hash.as_bytes().iter().all(u8::is_ascii_hexdigit)
}

async fn save_cycle(handle: &RuntimeHandle, cycle: dokime_runtime::CycleView, root: &Path) -> (usize, usize) {
    if cycle.test_executions.is_empty() {
        return (0, 0);
    }
    handle.seal_cycle(cycle.test_cycle_id, dokime_runtime::runtime::command::CommandContext::default()).await.unwrap();

    let saved = cycle.test_executions.len();
    let mut artifacts = 0;
    for execution in cycle.test_executions {
        artifacts += save_execution(handle, &execution.test_execution_id, root).await;
    }
    (saved, artifacts)
}

async fn save_execution(handle: &RuntimeHandle, id: &dokime_runtime::TestExecutionId, root: &Path) -> usize {
    let checkpoint = handle.execution_saved_checkpoint(id.clone()).await.unwrap();
    let view = dokime_runtime::runtime::ExecutionView::open_checkpoint(handle.storage(), &checkpoint).await.unwrap().unwrap();
    let seal = view.execution().saved.as_ref().expect("final execution seal").clone();
    let directory = root.join(id.as_str());
    std::fs::create_dir_all(&directory).unwrap();

    let mut artifacts = BTreeMap::new();
    for output in &seal.source.outputs {
        let ExecutionOutputContent::Artifact { record, .. } = &output.content else {
            continue;
        };
        dokime_runtime::validate_relative_output_path(&output.relative_path).unwrap();
        let destination = directory.join("outputs").join(&output.relative_path);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();

        use dokime_runtime::harmos::storage::{ByteRange, FileRead, FileSource};
        let request = FileRead {
            source: FileSource::Stored(output.file.clone()),
            range: ByteRange { offset: 0, limit: None },
        };
        let reader = handle.storage().read_file(request).await.unwrap();
        save_file(reader, &destination, &output.file).await;

        let artifact = Artifact {
            size_bytes: output.file.length,
            sha256: hex(&output.file.sha256),
        };
        assert!(artifacts.insert(record.artifact_id.to_string(), artifact).is_none());
    }

    let path = directory.join("report.html");
    let report = handle.open_execution_report(id).await.unwrap();
    assert_eq!(report.file, seal.report);
    save_file(report.reader, &path, &seal.report).await;
    reopen(&path, id.as_str(), &artifacts);

    let reopened = dokime_runtime::runtime::ExecutionView::open_checkpoint(handle.storage(), &checkpoint).await.unwrap().unwrap();
    assert_eq!(reopened.execution().saved.as_ref(), Some(&seal));
    artifacts.len()
}

async fn save_file(
    mut reader: dokime_runtime::harmos::storage::FileReader, path: &Path, reference: &dokime_runtime::harmos::storage::FileReference,
) {
    let mut file = std::fs::File::create(path).unwrap();
    let mut offset = 0;
    let mut digest = Sha256::new();
    while let Some(batch) = reader.next().await.unwrap() {
        assert_eq!(batch.offset(), offset);
        file.write_all(batch.bytes()).unwrap();
        digest.update(batch.bytes());
        offset += batch.bytes().len() as u64;
    }

    reader.completion().unwrap();
    file.sync_all().unwrap();
    drop(file);
    assert_eq!(offset, reference.length);
    assert_eq!(digest.finalize().as_slice(), reference.sha256.as_slice());
    verify_file(path, reference.length, &hex(&reference.sha256));
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(text, "{byte:02x}").unwrap();
    }
    text
}

/* === Tests === */
