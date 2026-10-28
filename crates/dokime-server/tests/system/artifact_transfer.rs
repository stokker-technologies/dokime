/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Artifact Transfer At Full Scale: A 300 MiB Artifact Uploads Through The Native File
// Contract And Downloads In Bounded Frames; Integration Keeps The 2 MiB Guard
// Of The Same Path At Two Frame Limits (`integration/content.rs`).

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use futures_util::FutureExt;
use sha2::{Digest, Sha256};
use tonic::transport::Endpoint;

// Local Imports
use crate::support::{self, wire::*};
use dokime::artifacts::ArtifactUploadInput;
use dokime::client::contracts;
use dokime::domain::model;
use dokime::domain::model::execution::result::CycleView;
use dokime::domains::ExecutionProviderTarget;
use dokime::{ExecutionFixture, ExecutionFixtureContext, ExecutionFixtureFuture, RecordScope, Runtime};
use dokime_server::grpc::Server;

/* === Constants === */

const ARTIFACT_BYTES: u64 = 300 * 1024 * 1024;
const UPLOAD_CHUNK_BYTES: usize = 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct LargeArtifact;

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ExecutionFixture for LargeArtifact {
    fn run_cycle<'a>(
        &'a self, context: ExecutionFixtureContext, cycle: &'a CycleView, _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            let execution = &cycle.test_executions[0];
            let scope = RecordScope::default()
                .with_cycle(cycle.test_cycle_id.clone())
                .with_test_case(execution.test_case_id.clone())
                .with_test_execution(execution.test_execution_id.clone());
            let input = ArtifactUploadInput {
                scope,
                filename: "large.bin".into(),
                content_type: Some("application/octet-stream".into()),
                size_bytes: ARTIFACT_BYTES,
                sha256: None,
            };
            let upload = context.begin_artifact_upload(input).await.unwrap();
            let chunk = vec![0_u8; UPLOAD_CHUNK_BYTES];
            let mut sent = 0_u64;
            while sent < ARTIFACT_BYTES {
                let remaining = (ARTIFACT_BYTES - sent).min(chunk.len() as u64) as usize;
                upload.send(chunk[..remaining].to_vec()).await.unwrap();
                sent += remaining as u64;
            }
            upload.finish().await.unwrap();
            Ok(support::fixture_provider_outcome(cycle, false))
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn failed(cycle: &model::execution::wire::CycleSummary) -> bool {
    let Some(result) = cycle.result.as_ref() else {
        return false;
    };
    matches!(result.status, model::execution::wire::ResultStatus::Failed | model::execution::wire::ResultStatus::Blocked)
}

/* === Tests === */

#[tokio::test]
async fn artifact_transfer() {
    let root = tempfile::tempdir().unwrap();
    let config = crate::support::budgets::runtime_config(root.path(), support::metadata("grpc.content.bundle", "Content"));
    let mut owner = Runtime::with_execution_fixture(config, "fixture", Arc::new(LargeArtifact)).boxed_local().await.unwrap();
    support::register_fixture_execution(&owner.handle(), support::fixture_suite(&["passed"])).await;
    let server = Server::start(owner.handle(), keyed()).await.unwrap();
    let stream = Endpoint::from_shared(server.local_endpoint()).unwrap().connect().await.unwrap();
    let execution = WireClient::new(stream.clone());
    let create = contracts::execution::CreateCycle {
        metadata: Some(model::identity::wire::Metadata {
            identifier: "content.bundle".into(),
            label: "Bundle".into(),
            ..Default::default()
        }),
        ..Default::default()
    };
    let cycle_id = support::planned_cycle(&execution, authenticated(create), selection_plan(&owner)).await.unwrap();

    // Stage: The Execution Uploads The Whole Artifact And Passes.
    execution.start_first(authenticated(contracts::execution::GetCycle { cycle_id: cycle_id.clone() })).await.unwrap();
    support::on_state_change(&owner.handle(), async || {
        let request = authenticated(contracts::execution::GetCycle { cycle_id: cycle_id.clone() });
        let cycle = execution.unary::<contracts::execution::GetCycle>(request).await.unwrap().into_inner();
        assert!(!failed(&cycle), "the upload execution failed: {cycle:?}");
        support::cycle_settled(&cycle, model::execution::wire::ResultStatus::Passed).then_some(())
    })
    .await;

    // Stage: The Download Streams Metadata Once, Then Contiguous Chunks, Then Completion.
    let content = WireClient::new(stream);
    let artifacts = owner.handle().artifacts().await.unwrap();
    let artifact = artifacts.iter().find(|artifact| artifact.filename == "large.bin").unwrap();
    let request = authenticated(contracts::content::DownloadArtifact {
        artifact_id: artifact.artifact_id.to_string(),
    });
    let mut download = content.download::<contracts::content::DownloadArtifact>(request).await.unwrap().into_inner();
    let metadata = match download.message().await.unwrap().unwrap().payload {
        Some(model::content::wire::ArtifactFramePayload::Metadata(metadata)) => metadata,
        payload => panic!("unexpected artifact header: {payload:?}"),
    };
    assert_eq!(metadata.size_bytes, ARTIFACT_BYTES);
    let mut downloaded = 0_u64;
    let mut sizes = Vec::new();
    let mut digest = Sha256::new();
    let completion = loop {
        let payload = download.message().await.unwrap().expect("artifact completion").payload.unwrap();
        let chunk = match payload {
            model::content::wire::ArtifactFramePayload::Chunk(chunk) => chunk,
            model::content::wire::ArtifactFramePayload::Complete(done) => break done,
            model::content::wire::ArtifactFramePayload::Metadata(_) => panic!("artifact metadata must be emitted once"),
        };
        assert_eq!(chunk.offset, downloaded);
        downloaded += chunk.data.len() as u64;
        sizes.push(chunk.data.len());
        digest.update(&chunk.data);
    };
    assert!(download.message().await.unwrap().is_none());

    // Invariants: Exact Bytes, Each Chunk Within One Message, And At Most Twice The Minimal Chunk Count.
    assert_eq!(completion.size_bytes, downloaded);
    assert_eq!(metadata.size_bytes, downloaded);
    assert_eq!(completion.sha256, digest.finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>());
    assert_eq!(metadata.sha256, completion.sha256);
    let frame = *sizes.iter().max().unwrap();
    assert!(sizes.iter().all(|size| *size > 0), "no empty chunk");
    assert!(frame <= keyed().max_message_bytes(), "a chunk fits one message: {frame}");
    let full = ARTIFACT_BYTES.div_ceil(frame as u64);
    assert!(sizes.len() as u64 <= 2 * full, "bounded fragmentation: {} chunks of at most {frame} bytes", sizes.len());

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}
