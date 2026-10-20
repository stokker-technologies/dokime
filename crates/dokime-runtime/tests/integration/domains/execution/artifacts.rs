/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::path::Path;
use std::sync::Arc;

// Third Party Imports

// Local Imports
use crate::service_support::prelude::*;
use crate::service_support::smoke::{Behaviour, SOURCE, SUITE, Smoke};
use dokime_runtime::runtime::command::{CommandContext, CommandStatus};

/* === Constants === */
const WATCHED_CONTENT: &[u8] = b"the chamber log, rewritten in place\n";
const SETTLED_CONTENT: &[u8] = b"produced after the abandoned transfers\n";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/* === Enums === */

/* === Traits === */

/* === Implementations === */
/* === Public Functions === */

/* === Private Functions === */
async fn execute(root: &Path, abandoned: bool) -> (Runtime, RuntimeHandle, dokime_domain::model::execution::result::CycleView) {
    let provider = Smoke::new(Behaviour::new(777, 0, false).artifacts(abandoned));
    let (runtime, handle) = started(config(root).register(Arc::new(provider))).await;
    let suite = handle.test_suites().await.into_iter().find(|suite| suite.source == SOURCE).unwrap();
    let cycle_id = start_cases(&handle, metadata("artifact.cycle", "Artifact Cycle"), materialize_cases(SUITE, &suite.suite)).await;
    let receipt = handle.start_next_execution(cycle_id.clone(), CommandContext::default()).await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");
    wait_for_run_settled(&handle, &receipt.command_id).await;
    let cycle = cycle_snapshot(&handle, &cycle_id).await;
    assert_eq!(cycle.test_executions.len(), 1);
    assert_eq!(cycle.test_executions[0].status, dokime_domain::model::execution::result::ExecutionStatus::Passed);
    (runtime, handle, cycle)
}

fn spill_removed(root: &Path) {
    for entry in std::fs::read_dir(root).unwrap() {
        assert!(!entry.unwrap().file_name().to_string_lossy().starts_with("dokime-sidecar-artifact-"));
    }
}

async fn export_zip(
    runtime: &RuntimeHandle, subject: dokime_domain::model::exports::EvidenceSubject, destination: &Path,
) -> serde_json::Value {
    use sha2::{Digest, Sha256};
    use std::io::{Read, Write};
    let mut bundle = runtime.export_evidence(subject.clone()).await.unwrap();
    assert_eq!(bundle.metadata().subject.as_ref(), Some(&subject));
    let metadata = bundle.metadata().clone();
    assert!(matches!(runtime.export_evidence(subject).await, Err(dokime_runtime::domains::exports::evidence::Error::Busy)));
    let mut output = std::fs::File::create_new(destination).unwrap();
    let mut digest = Sha256::new();
    let mut offset = 0;
    while let Some(chunk) = bundle.next_chunk(std::num::NonZeroUsize::new(32_768).unwrap()).await.unwrap() {
        assert_eq!(chunk.offset, offset);
        offset += chunk.data.len() as u64;
        digest.update(&chunk.data);
        output.write_all(&chunk.data).unwrap();
    }
    assert_eq!(offset, metadata.size_bytes);
    assert_eq!(digest.finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>(), metadata.sha256);
    drop(output);
    drop(bundle);

    let mut zip = zip::ZipArchive::new(std::fs::File::open(destination).unwrap()).unwrap();
    let manifest: serde_json::Value = serde_json::from_reader(zip.by_name("manifest.json").unwrap()).unwrap();
    assert_eq!(manifest["schema_version"], 1);
    assert_eq!(manifest["complete"], true);
    for payload in manifest["payloads"].as_array().unwrap() {
        let mut entry = zip.by_name(payload["path"].as_str().unwrap()).unwrap();
        assert_eq!(entry.size(), payload["size_bytes"].as_u64().unwrap());
        let mut hasher = Sha256::new();
        let mut buffer = [0_u8; 65_536];
        loop {
            let read = entry.read(&mut buffer).unwrap();
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        let sha256 = hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        assert_eq!(sha256, payload["sha256"].as_str().unwrap());
    }
    manifest
}

/* === Tests === */
#[tokio::test]
async fn saved_evidence_export() {
    use dokime_domain::model::exports::EvidenceSubject;
    let root = tempfile::tempdir().unwrap();
    let (mut owner, runtime, cycle) = execute(root.path(), false).await;
    let execution = &cycle.test_executions[0];
    let execution_subject = EvidenceSubject::Execution(execution.test_execution_id.to_string());
    let cycle_subject = EvidenceSubject::Cycle(cycle.test_cycle_id.to_string());
    assert!(matches!(
        runtime.export_evidence(cycle_subject.clone()).await,
        Err(dokime_runtime::domains::exports::evidence::Error::Unsealed("cycle"))
    ));
    let path = root.path().join("execution.zip");
    let manifest = export_zip(&runtime, execution_subject.clone(), &path).await;
    assert_eq!(manifest["executions"].as_array().unwrap().len(), 1);
    assert_eq!(manifest["executions"][0]["counts"]["samples"], 777);
    assert!(manifest["executions"][0]["counts"]["records"].as_u64().unwrap() > 0);
    let database_path = root.path().join("inspection.sqlite");
    let mut zip = zip::ZipArchive::new(std::fs::File::open(&path).unwrap()).unwrap();
    let mut database_file = std::fs::File::create_new(&database_path).unwrap();
    std::io::copy(&mut zip.by_name("evidence.sqlite").unwrap(), &mut database_file).unwrap();
    drop(database_file);
    let database = rusqlite::Connection::open(&database_path).unwrap();
    assert_eq!(database.query_row("SELECT count(*) FROM executions", [], |row| row.get::<_, i64>(0)).unwrap(), 1);
    assert_eq!(database.query_row("SELECT phase FROM cycles", [], |row| row.get::<_, String>(0)).unwrap(), "active");
    assert_eq!(database.query_row("SELECT sum(sample_count) FROM streams", [], |row| row.get::<_, i64>(0)).unwrap(), 777);
    assert!(database.query_row("SELECT count(*) FROM outputs", [], |row| row.get::<_, i64>(0)).unwrap() >= 2);
    let table = database.query_row("SELECT table_name FROM streams WHERE sample_count=777", [], |row| row.get::<_, String>(0)).unwrap();
    assert_eq!(database.query_row(&format!("SELECT count(*) FROM {table}"), [], |row| row.get::<_, i64>(0)).unwrap(), 777);
    assert!(!manifest.to_string().contains("PRIVATE KEY"));
    drop(database);
    drop(zip);

    runtime
        .append_fixture_cases(cycle.test_cycle_id.clone(), vec![execution.snapshot.case().clone()], CommandContext::default())
        .await
        .unwrap();
    runtime.seal_cycle(cycle.test_cycle_id.clone(), CommandContext::default()).await.unwrap();
    let manifest = export_zip(&runtime, cycle_subject.clone(), &root.path().join("cycle.zip")).await;
    assert!(manifest["cycle_revision"].is_object());
    assert_eq!(manifest["executions"].as_array().unwrap().len(), 2);
    assert_eq!(manifest["executions"][1]["counts"]["samples"], 0);
    assert!(manifest["payloads"].as_array().unwrap().iter().any(|entry| entry["path"].as_str().unwrap().starts_with("reports/cycle-")));
    owner.shutdown().await.unwrap();
    drop(runtime);
    let (mut owner, runtime) = started(config(root.path())).await;
    let historical = export_zip(&runtime, execution_subject, &root.path().join("historical.zip")).await;
    assert_eq!(historical["executions"].as_array().unwrap().len(), 1);
    assert_eq!(historical["executions"][0], manifest["executions"][0]);
    let historical_cycle = export_zip(&runtime, cycle_subject, &root.path().join("historical-cycle.zip")).await;
    assert_eq!(historical_cycle["executions"], manifest["executions"]);
    assert_eq!(historical_cycle["cycle_revision"], manifest["cycle_revision"]);
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn required_evidence_files() {
    use dokime_domain::model::exports::EvidenceSubject;

    let root = tempfile::tempdir().unwrap();
    let (mut owner, runtime, cycle) = execute(root.path(), false).await;
    let execution = &cycle.test_executions[0];
    for (subject, filename) in [
        (EvidenceSubject::Execution(execution.test_execution_id.to_string()), "verified-execution.zip"),
        (EvidenceSubject::Cycle(cycle.test_cycle_id.to_string()), "verified-cycle.zip"),
    ] {
        if let EvidenceSubject::Cycle(_) = &subject {
            runtime.seal_cycle(cycle.test_cycle_id.clone(), CommandContext::default()).await.unwrap();
        }
        let destination = root.path().join(filename);
        let manifest = export_zip(&runtime, subject.clone(), &destination).await;
        let payloads = manifest["payloads"].as_array().unwrap();
        let output = match &subject {
            EvidenceSubject::Execution(_) => {
                payloads.iter().find(|entry| entry["relative_path"].as_str().is_some_and(|path| path.ends_with("/capture/chamber.log")))
            }
            EvidenceSubject::Cycle(_) => {
                let report = format!("reports/cycle-{}.html", cycle.test_cycle_id);
                payloads.iter().find(|entry| entry["path"] == report)
            }
        }
        .expect("the selected captured log or sealed-cycle report exists");
        let path = runtime.storage_root().join("files").join(output["sha256"].as_str().unwrap());

        for missing in [true, false] {
            let path = &path;
            let original = std::fs::read(path).unwrap();
            assert!(!original.is_empty());
            let backup = path.with_extension("held");
            if missing {
                std::fs::rename(path, &backup).unwrap();
            } else {
                let mut corrupt = original.clone();
                corrupt[0] ^= 0x55;
                std::fs::write(path, corrupt).unwrap();
            }

            let result = runtime.export_evidence(subject.clone()).await;
            if missing {
                std::fs::rename(&backup, path).unwrap();
            } else {
                std::fs::write(path, &original).unwrap();
            }
            let error = result.err().expect("missing or corrupt required bytes must refuse the complete archive");
            assert!(
                matches!(
                    error,
                    dokime_runtime::domains::exports::evidence::Error::Dependency(_)
                        | dokime_runtime::domains::exports::evidence::Error::Invalid(_)
                ),
                "unexpected export failure: {error}"
            );
            let cache = runtime.application_root().join("cache/exports");
            assert_eq!(std::fs::read_dir(&cache).unwrap().count(), 0, "failed export must remove its private database and ZIP");

            // Retry The Repaired Download Without New Execution Or Retained Admission.
            let bundle = runtime.export_evidence(subject.clone()).await.unwrap();
            assert_eq!(bundle.metadata().subject.as_ref(), Some(&subject));
            drop(bundle);
            assert_eq!(std::fs::read_dir(&cache).unwrap().count(), 0);
            assert_eq!(std::fs::read(path).unwrap(), original);
        }
    }
    let saved = cycle_snapshot(&runtime, &cycle.test_cycle_id).await;
    assert_eq!(saved.test_executions.len(), 1, "download retries must never append or rerun executions");
    assert_eq!(saved.test_executions[0], *execution);
    owner.shutdown().await.unwrap();
}

// SDK FrameArtifactSink -> Workflow EvidenceBatch -> host Session/ArtifactBridge.
// IPC encoding/admission stays in the OBC system walkthrough and protocol contracts.
#[tokio::test]
async fn chunked_artifact_rewrite() {
    use std::fmt::Write;
    let root = tempfile::tempdir().unwrap();
    let (mut owner, handle, cycle) = execute(root.path(), false).await;
    let execution = &cycle.test_executions[0];
    let mut expected = String::from("sample,value\n");
    for index in 0..262_144 {
        writeln!(expected, "{index:08},{:03}", index % 251).unwrap();
    }
    assert!(expected.len() > dokime_runtime::transport::MAX_ARTIFACT_CHUNK_BYTES * 3);
    let artifacts = handle.artifacts().await.unwrap();
    for (name, media, bytes) in
        [("evidence/pulse.csv", "text/csv", expected.as_bytes()), ("capture/chamber.log", "text/plain", WATCHED_CONTENT)]
    {
        let record = artifacts.iter().find(|record| record.filename == name).unwrap();
        assert_eq!(record.status, dokime_runtime::artifacts::ArtifactStatus::Complete);
        assert_eq!(record.scope.test_execution_id.as_ref(), Some(&execution.test_execution_id));
        assert_eq!(record.content_type.as_deref(), Some(media));
        assert_eq!(record.size_bytes, bytes.len() as u64);
        let content = handle.read_artifact_content(&record.artifact_id, bytes.len()).await.unwrap().unwrap();
        assert_eq!(content.bytes, bytes);
    }
    drop(handle);
    owner.shutdown().await.unwrap();
    spill_removed(root.path());
}

// The abandoned transfers outgrow the artifact backlog, so the step parks
// until the host drains it. Product runtimes are multi-threaded; on a single
// thread the parked step would hold the worker the host drains on.
#[tokio::test(flavor = "multi_thread")]
async fn abandoned_transfer_slots() {
    let root = tempfile::tempdir().unwrap();
    let (mut owner, handle, cycle) = execute(root.path(), true).await;
    let execution = &cycle.test_executions[0];
    let stored = handle.artifacts().await.unwrap();
    assert_eq!(stored.iter().map(|record| record.filename.as_str()).collect::<Vec<_>>(), ["evidence/settled.txt"]);
    let content = handle.read_artifact_content(&stored[0].artifact_id, SETTLED_CONTENT.len()).await.unwrap().unwrap();
    assert_eq!(content.bytes, SETTLED_CONTENT);
    let files = handle.execution_files(&execution.test_execution_id).await.unwrap();
    let log = files.iter().find(|file| file.relative_path.ends_with("stream.log")).unwrap();
    let log = handle.execution_file_content(&execution.test_execution_id, log.relative_path.clone(), 0, 16_384).await.unwrap().unwrap();
    let log = String::from_utf8(log.bytes).unwrap();
    let received = dokime_runtime::transport::MAX_ARTIFACT_CHUNK_BYTES;
    for index in 0..17 {
        assert!(
            log.contains(&format!(
                "[WARN] dropped execution artifact `evidence/partial-{index}.bin`: the producer aborted the transfer after {received} bytes"
            )),
            "{log}"
        );
    }
    assert_eq!(log.matches("dropped execution artifact").count(), 17);
    drop(handle);
    owner.shutdown().await.unwrap();
    spill_removed(root.path());
}
