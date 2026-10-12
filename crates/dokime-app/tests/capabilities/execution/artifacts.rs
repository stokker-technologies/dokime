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
use std::{path::Path, time::Duration};

// Third Party Imports
use tempfile::tempdir;

// Local Imports
use crate::support::plugins::fixture_bundle;
use crate::support::prelude::*;
use dokime::core::{Identifier, Metadata};
use dokime::execution::ExecutionStatus;
use dokime_app::runtime::command::{CommandContext, CommandStatus};
use dokime_app::transactions::CycleDefinition;

/* === Constants === */
const ARTIFACT_EVIDENCE_HEADER: &str = "sample,value\n";
const ARTIFACT_EVIDENCE_ROWS: usize = 262_144;

const ABANDONED_ARTIFACT_COUNT: usize = 17;
const SETTLED_ARTIFACT_CONTENT: &str = "produced after the abandoned transfers\n";

const WATCHED_FILE: &str = "chamber.log";
const WATCHED_CONTENT: &str = "the chamber log, rewritten in place\n";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn artifact_evidence() -> Vec<u8> {
    use std::fmt::Write;
    let mut content = String::from(ARTIFACT_EVIDENCE_HEADER);
    for index in 0..ARTIFACT_EVIDENCE_ROWS {
        writeln!(content, "{index:08},{:03}", index % 251).expect("a string accepts a row");
    }
    content.into_bytes()
}

fn assert_spill_removed(root: &Path) {
    for entry in std::fs::read_dir(root).unwrap() {
        let entry = entry.unwrap();
        assert!(
            !entry
                .file_name()
                .to_string_lossy()
                .starts_with("dokime-sidecar-artifact-"),
            "the finished transfer left its spill directory: {}",
            entry.path().display()
        );
    }
}

/* === Tests === */
// a steps streamed artifact is stored whole with its inferred media type.

pub(crate) async fn steps_streamed_media() {
    let root = tempdir().unwrap();
    let (mut runtime, handle) = started(
        config(root.path()).with_plugins([fixture_bundle(root.path(), "fixture-suites")
            .configured(
                "fixture-suites",
                serde_json::json!({ "artifact_evidence": true }),
            )
            .unwrap()]),
    )
    .await;

    let suite = handle
        .test_suites()
        .await
        .into_iter()
        .find(|suite| suite.source == "fixture-suites")
        .unwrap()
        .suite;
    let cycle_id = handle
        .start_cycle(
            CycleDefinition {
                suite: suite.clone(),
                metadata: Metadata::new(
                    Identifier::new("artifact.cycle").unwrap(),
                    "Artifact Cycle",
                    None,
                ),
                cases: materialize_cases("fixture.checks", &suite),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let receipt = handle
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");

    let completed = tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let cycle = handle
                .cycles()
                .await
                .into_iter()
                .find(|cycle| cycle.test_cycle_id == cycle_id)
                .unwrap();
            if !cycle.test_executions.is_empty()
                && cycle
                    .test_executions
                    .iter()
                    .all(|execution| execution.finished_at.is_some())
            {
                return cycle;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("execution settles");

    let execution = &completed.test_executions[0];
    assert_eq!(execution.status, ExecutionStatus::Passed);

    // `capabilities::exports::storage::names::safe_artifact_filename` sanitizes
    // every stored artifact's name down to a safe basename, so the streamed
    // relative name (`evidence/pulse.csv`) is not what lands in the durable
    // `ArtifactRecord`; only the wire name carries it, and
    // `test::workflow::tests` covers that directly against `FrameTransport`.
    let expected = artifact_evidence();
    assert!(
        expected.len() > dokime::transport::MAX_ARTIFACT_CHUNK_BYTES * 3,
        "the fixture streams past several protocol chunks, exercising the host's spill-to-disk read loop"
    );
    let record = handle
        .artifacts()
        .await
        .expect("artifact query succeeds")
        .iter()
        .find(|record| record.filename == "pulse.csv")
        .cloned()
        .expect("the step's streamed artifact is stored against the completed execution");
    assert_eq!(record.status, dokime::artifacts::ArtifactStatus::Complete);
    assert_eq!(
        record.scope.test_execution_id,
        Some(execution.test_execution_id.clone())
    );
    assert_eq!(record.content_type.as_deref(), Some("text/csv"));
    assert_eq!(record.size_bytes, expected.len() as u64);

    let content = handle
        .read_artifact_content(&record.artifact_id, expected.len())
        .await
        .unwrap()
        .expect("the stored artifact content is readable");
    assert_eq!(content.bytes, expected);

    // A file nobody announced, written into a directory the execution owns and
    // rewritten after the watch had already streamed it. It is stored exactly
    // like one handed over through `create` — same scope, same media type read
    // off the name — and it holds the content the file settled on rather than
    // both versions end to end.
    let watched = handle
        .artifacts()
        .await
        .expect("artifact query succeeds")
        .iter()
        .find(|record| record.filename == WATCHED_FILE)
        .cloned()
        .expect("the watched directory's file is stored against the execution");
    assert_eq!(watched.status, dokime::artifacts::ArtifactStatus::Complete);
    assert_eq!(
        watched.scope.test_execution_id,
        Some(execution.test_execution_id.clone())
    );
    assert_eq!(watched.content_type.as_deref(), Some("text/plain"));
    assert_eq!(watched.size_bytes, WATCHED_CONTENT.len() as u64);
    let watched = handle
        .read_artifact_content(&watched.artifact_id, WATCHED_CONTENT.len())
        .await
        .unwrap()
        .expect("the stored artifact content is readable");
    assert_eq!(
        String::from_utf8(watched.bytes).unwrap(),
        WATCHED_CONTENT,
        "a replaced artifact ends with the new content alone"
    );

    drop(handle);
    runtime.shutdown().await.unwrap();
    assert_spill_removed(root.path());
}

// abandoned artifacts are released so the execution can keep producing.

pub(crate) async fn abandoned_artifacts_execution() {
    let root = tempdir().unwrap();
    let (mut runtime, handle) = started(
        config(root.path()).with_plugins([fixture_bundle(root.path(), "fixture-suites")
            .configured(
                "fixture-suites",
                serde_json::json!({ "abandon_artifact": true }),
            )
            .unwrap()]),
    )
    .await;

    let suite = handle
        .test_suites()
        .await
        .into_iter()
        .find(|suite| suite.source == "fixture-suites")
        .unwrap()
        .suite;
    let cycle_id = handle
        .start_cycle(
            CycleDefinition {
                suite: suite.clone(),
                metadata: Metadata::new(
                    Identifier::new("abandoned.cycle").unwrap(),
                    "Abandoned Artifact Cycle",
                    None,
                ),
                cases: materialize_cases("fixture.checks", &suite),
                continue_on_failure: false,
            },
            CommandContext::default(),
        )
        .await
        .unwrap();
    let receipt = handle
        .run_cycle_cases(cycle_id.clone(), CommandContext::default())
        .await;
    assert_eq!(receipt.status, CommandStatus::Accepted, "{receipt:?}");

    let completed = tokio::time::timeout(Duration::from_secs(15), async {
        loop {
            let cycle = handle
                .cycles()
                .await
                .into_iter()
                .find(|cycle| cycle.test_cycle_id == cycle_id)
                .unwrap();
            if !cycle.test_executions.is_empty()
                && cycle
                    .test_executions
                    .iter()
                    .all(|execution| execution.finished_at.is_some())
            {
                return cycle;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .expect("execution settles");

    // Abandoning an artifact is the step's statement that the content is
    // incomplete, not a transport fault: the run keeps the verdict its steps
    // and criteria earned, and the partial bytes never become evidence.
    let execution = &completed.test_executions[0];
    assert_eq!(execution.status, ExecutionStatus::Passed);

    // Each abandoned transfer gave its slot back as its abort arrived, so the
    // execution could open the next one — and one more after all of them.
    let stored = handle.artifacts().await.expect("artifact query succeeds");
    assert_eq!(
        stored
            .iter()
            .map(|record| record.filename.clone())
            .collect::<Vec<_>>(),
        ["settled.txt"],
        "the abandoned transfers are dropped and the finished one is stored"
    );
    let content = handle
        .read_artifact_content(&stored[0].artifact_id, SETTLED_ARTIFACT_CONTENT.len())
        .await
        .unwrap()
        .expect("the stored artifact content is readable");
    assert_eq!(content.bytes, SETTLED_ARTIFACT_CONTENT.as_bytes());

    // The bytes are gone, so the execution's own log is the only place an
    // operator can learn which files the run gave up on.
    let execution_id = &execution.test_execution_id;
    let files = handle.execution_files(execution_id).await.unwrap();
    let log = files
        .iter()
        .find(|file| file.relative_path.ends_with("stream.log"))
        .expect("the abandonments are recorded in the execution's own log");
    let log = handle
        .execution_file_content(execution_id, log.relative_path.clone(), 0, 16_384)
        .await
        .unwrap()
        .expect("the stream log remains readable");
    let log = String::from_utf8(log.bytes.clone()).unwrap();
    // One whole chunk was on its way before the step gave up; the byte after it
    // was still buffered in the writer and never left the sidecar.
    let handed_over = dokime::transport::MAX_ARTIFACT_CHUNK_BYTES;
    for index in 0..ABANDONED_ARTIFACT_COUNT {
        assert!(
            log.contains(&format!(
                "[WARN] dropped execution artifact `evidence/partial-{index}.bin`: the producer aborted the transfer after {handed_over} bytes"
            )),
            "{log}"
        );
    }
    assert_eq!(
        log.matches("dropped execution artifact").count(),
        ABANDONED_ARTIFACT_COUNT,
        "only the transfers the step gave up on are reported"
    );

    drop(handle);
    runtime.shutdown().await.unwrap();
    assert_spill_removed(root.path());
}
