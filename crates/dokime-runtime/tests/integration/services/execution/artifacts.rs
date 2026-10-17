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
use std::sync::Arc;
use std::path::Path;

// Third Party Imports

// Local Imports
use super::smoke::{Behaviour, Smoke, SOURCE, SUITE};
use crate::service_support::prelude::*;
use dokime_runtime::execution::{Caller, ExecutionHost, Runnable, SampleOutput, SidecarTemplate, Workflow};
use dokime_runtime::transport::{EvidenceBatch, EvidenceReceipt, ExecutionSignal, LeaseAsk, LeaseResolution, OperatorAnswer, OperatorQuestion, RunExecution, SidecarExecutionOutcome, SignalReply, TransportFuture};
use dokime_runtime::Refusal;
use dokime_runtime::runtime::command::{CommandContext, CommandStatus};

/* === Constants === */
const WATCHED_CONTENT: &[u8] = b"the chamber log, rewritten in place\n";
const SETTLED_CONTENT: &[u8] = b"produced after the abandoned transfers\n";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(crate) struct Observed<T: SidecarTemplate> {
    pub(crate) workflow: Workflow<T>,
    pub(crate) watched: tokio::sync::watch::Sender<bool>,
}

struct Admitted {
    host: Arc<dyn ExecutionHost>,
    watched: tokio::sync::watch::Sender<bool>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ExecutionHost for Admitted {
    fn push(&self, batch: EvidenceBatch) -> TransportFuture<'_, Result<EvidenceReceipt, String>> {
        let watched = batch.artifact_chunks.iter().any(|chunk| chunk.name == "capture/chamber.log" && !chunk.last);
        Box::pin(async move {
            let receipt = self.host.push(batch).await?;
            if watched {
                self.watched.send_replace(true);
            }
            Ok(receipt)
        })
    }

    fn ask(&self, question: OperatorQuestion) -> TransportFuture<'_, Result<OperatorAnswer, String>> {
        self.host.ask(question)
    }

    fn lease(&self, ask: LeaseAsk) -> TransportFuture<'_, Result<LeaseResolution, String>> {
        self.host.lease(ask)
    }
}

impl<T: SidecarTemplate> Runnable for Observed<T> {
    fn template_identifier(&self) -> &str {
        self.workflow.template_identifier()
    }

    fn run(&self, request: RunExecution, host: Arc<dyn ExecutionHost>, output: Option<SampleOutput>, caller: Caller) -> TransportFuture<'_, Result<SidecarExecutionOutcome, Refusal>> {
        let admitted = Arc::new(Admitted { host, watched: self.watched.clone() });
        self.workflow.run(request, admitted, output, caller)
    }

    fn signal(&self, signal: ExecutionSignal) -> TransportFuture<'_, Result<SignalReply, Refusal>> {
        self.workflow.signal(signal)
    }

    fn take_local_samples(&self, token: &str) -> Result<Vec<dokime_runtime::telemetry::TelemetrySample>, Refusal> {
        self.workflow.take_local_samples(token)
    }

    fn interrupt(&self) {
        self.workflow.interrupt();
    }

    fn shutdown(&self) -> TransportFuture<'_, ()> {
        self.workflow.shutdown()
    }
}

/* === Public Functions === */

/* === Private Functions === */
async fn execute(root: &Path, abandoned: bool) -> (Runtime, RuntimeHandle, dokime_domain::model::execution::result::CycleView) {
    let provider = Smoke::new(Behaviour::new(0, 0, false).artifacts(abandoned));
    let (runtime, handle) = started(config(root).register(Arc::new(provider))).await;
    let suite = handle.test_suites().await.into_iter().find(|suite| suite.source == SOURCE).unwrap();
    let cycle_id = start_cases(&handle, metadata("artifact.cycle", "Artifact Cycle"), materialize_cases(SUITE, &suite.suite), false).await;
    let receipt = handle.run_cycle_cases(cycle_id.clone(), CommandContext::default()).await;
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

/* === Tests === */
// SDK FrameArtifactSink -> Workflow EvidenceBatch -> host Session/ArtifactBridge.
// IPC encoding/admission stays in the OBC system walkthrough and protocol contracts.
#[tokio::test]
async fn chunked_artifacts_and_rewrite() {
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
    for (name, media, bytes) in [("evidence/pulse.csv", "text/csv", expected.as_bytes()), ("capture/chamber.log", "text/plain", WATCHED_CONTENT)] {
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

#[tokio::test]
async fn abandoned_transfers_release_slots() {
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
        assert!(log.contains(&format!("[WARN] dropped execution artifact `evidence/partial-{index}.bin`: the producer aborted the transfer after {received} bytes")), "{log}");
    }
    assert_eq!(log.matches("dropped execution artifact").count(), 17);
    drop(handle);
    owner.shutdown().await.unwrap();
    spill_removed(root.path());
}
