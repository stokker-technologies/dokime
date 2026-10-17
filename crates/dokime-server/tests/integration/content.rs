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
use std::sync::{Arc, Mutex};
// Third Party Imports
use sha2::{Digest, Sha256};
use tokio::sync::Notify;
// Local Imports
use super::*;
use dokime_runtime::domains::ExecutionProviderTarget;
use dokime_runtime::artifacts::{ArtifactRecord, ArtifactUploadInput};
use dokime_runtime::core::RecordScope;
use dokime_runtime::execution::CycleView;
use dokime_runtime::{ExecutionFixture, ExecutionFixtureContext, ExecutionFixtureFuture};

/* === Constants === */
const LARGE_ARTIFACT_BYTES: u64 = 300 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Default)]
struct Artifacts {
    records: Mutex<Vec<(ArtifactRecord, Vec<u8>)>>,
    ready: Notify,
    release: Notify,
}

#[derive(Default)]
pub(super) struct UploadGate {
    pub(super) started: Notify,
    pub(super) release: Notify,
}

struct LargeArtifact;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ExecutionFixture for Artifacts {
    fn run_cycle<'a>(
        &'a self,
        context: ExecutionFixtureContext,
        cycle: &'a CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            let execution = &cycle.test_executions[0];
            for (name, size, cancelled) in [
                ("full.bin", 150_000, false),
                ("empty.bin", 0, false),
                ("partial.bin", 8_000, true),
                ("large.bin", 2 * 1024 * 1024, false),
            ] {
                let bytes = (0..size)
                    .map(|index| (index % 251) as u8)
                    .collect::<Vec<_>>();
                let upload = context
                    .begin_artifact_upload(ArtifactUploadInput {
                        scope: RecordScope::default()
                            .with_cycle(cycle.test_cycle_id.clone())
                            .with_test_case(execution.test_case_id.clone())
                            .with_test_execution(execution.test_execution_id.clone()),
                        filename: name.into(),
                        content_type: Some("application/octet-stream".into()),
                        size_bytes: size as u64,
                        sha256: None,
                    })
                    .await
                    .unwrap();
                for chunk in bytes.chunks(64 * 1024) {
                    upload.send(chunk).await.unwrap();
                }
                let record = if cancelled {
                    upload.abandon().await
                } else {
                    upload.finish().await
                }
                .unwrap();
                self.records.lock().unwrap().push((record, bytes));
            }
            self.ready.notify_one();
            self.release.notified().await;
            Ok(support::fixture_provider_outcome(cycle, false))
        })
    }
}

impl ExecutionFixture for UploadGate {
    fn run_cycle<'a>(
        &'a self,
        _context: ExecutionFixtureContext,
        cycle: &'a CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            self.started.notify_one();
            self.release.notified().await;
            Ok(support::fixture_provider_outcome(cycle, false))
        })
    }
}

impl ExecutionFixture for LargeArtifact {
    fn run_cycle<'a>(
        &'a self,
        context: ExecutionFixtureContext,
        cycle: &'a CycleView,
        _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            let execution = &cycle.test_executions[0];
            let upload = context
                .begin_artifact_upload(ArtifactUploadInput {
                    scope: RecordScope::default()
                        .with_cycle(cycle.test_cycle_id.clone())
                        .with_test_case(execution.test_case_id.clone())
                        .with_test_execution(execution.test_execution_id.clone()),
                    filename: "large.bin".into(),
                    content_type: Some("application/octet-stream".into()),
                    size_bytes: LARGE_ARTIFACT_BYTES,
                    sha256: None,
                })
                .await
                .unwrap();
            let chunk = vec![0_u8; 1024 * 1024];
            let mut sent = 0_u64;
            while sent < LARGE_ARTIFACT_BYTES {
                let remaining = (LARGE_ARTIFACT_BYTES - sent).min(chunk.len() as u64) as usize;
                upload.send(&chunk[..remaining]).await.unwrap();
                sent += remaining as u64;
            }
            upload.finish().await.unwrap();
            Ok(support::fixture_provider_outcome(cycle, false))
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

// A 4 KiB Frame Limit Pages Listings And Chunks Downloads Into Small Frames
#[tokio::test]
async fn small_frame_transfer() {
    artifact_transfer(4096).await;
}

#[tokio::test]
async fn large_frame_transfer() {
    artifact_transfer(1024 * 1024).await;
}

async fn artifact_transfer(message_bytes: usize) {
    let root = tempfile::tempdir().unwrap();
    let artifacts = Arc::new(Artifacts::default());
    let mut owner = Runtime::with_execution_fixture(
        crate::support::budgets::runtime_config(
            root.path(),
            support::metadata("grpc.content", "Content"),
        ),
        "fixture",
        artifacts.clone(),
    )
    .await
    .unwrap();
    support::register_fixture_execution(&owner.handle(), support::fixture_suite(&["passed"])).await;
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Control)])
            .unwrap()
            .with_max_message_bytes(message_bytes),
    )
    .await
    .unwrap();
    let stream = Endpoint::from_shared(server.local_endpoint())
        .unwrap()
        .initial_stream_window_size(4096)
        .connect()
        .await
        .unwrap();
    let execution = WireClient::new(stream.clone());
    let cycle_id = support::planned_cycle(
        &execution,
        authenticated(contracts::execution::CreateCycleRequest {
            metadata: Some(contracts::common::Metadata {
                identifier: "content.cycle".into(),
                label: "Content".into(),
                description: None,
                ..Default::default()
            }),
            ..Default::default()
        }),
        selection_plan(&owner),
    )
    .await
    .unwrap();
    let reply = execution
        .unary::<contracts::execution::RunCycle>(authenticated(
            contracts::execution::RunCycleRequest {
                cycle_id: cycle_id.clone(),
                context: None,
                idempotency_key: None,
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert!(matches!(
        reply.outcome,
        Some(contracts::execution::RunReplyOutcome::Accepted(_))
    ));
    artifacts.ready.notified().await;
    let records = artifacts.records.lock().unwrap().clone();
    assert_eq!(records.len(), 4);
    let content = WireClient::new(stream).max_decoding_message_size(message_bytes);
    let first_page = content
        .unary::<contracts::content::ListArtifacts>(authenticated(
            contracts::content::ListArtifactsRequest {
                limit: 2,
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(first_page.artifacts.len(), 2);
    let next_after = first_page.next_after_artifact_id.clone().unwrap();
    let cursor = first_page.cursor.clone().unwrap();
    let second_page = content
        .unary::<contracts::content::ListArtifacts>(authenticated(
            contracts::content::ListArtifactsRequest {
                limit: 2,
                after_artifact_id: next_after,
                cursor: Some(cursor.clone()),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(second_page.artifacts.len(), 2);
    assert!(second_page.next_after_artifact_id.is_none());
    let mut invalid_cursor = cursor;
    invalid_cursor.metadata_sha256[0] ^= 1;
    assert_eq!(
        content
            .unary::<contracts::content::ListArtifacts>(authenticated(
                contracts::content::ListArtifactsRequest {
                    limit: 2,
                    after_artifact_id: first_page.next_after_artifact_id.clone().unwrap(),
                    cursor: Some(invalid_cursor),
                }
            ))
            .await
            .unwrap_err()
            .code(),
        Code::Aborted
    );
    let first_id = records[0].0.artifact_id.to_string();
    assert_eq!(
        content
            .download::<contracts::content::DownloadArtifact>(
                contracts::content::DownloadArtifactRequest {
                    artifact_id: first_id.clone()
                }
            )
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    assert_eq!(
        content
            .download::<contracts::content::DownloadArtifact>(authenticated(
                contracts::content::DownloadArtifactRequest {
                    artifact_id: "../file".into()
                }
            ))
            .await
            .unwrap()
            .into_inner()
            .message()
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    assert_eq!(
        content
            .download::<contracts::content::DownloadArtifact>(authenticated(
                contracts::content::DownloadArtifactRequest {
                    artifact_id: dokime_runtime::core::ArtifactId::new().to_string(),
                }
            ))
            .await
            .unwrap()
            .into_inner()
            .message()
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );

    if message_bytes >= 1024 * 1024 {
        // Shared Native Verification And Publication
        let native = dokime_client::Client::connect(
            dokime_client::Target::local(server.local_endpoint()).unwrap(),
            Some(TOKEN),
            None,
        )
        .await
        .unwrap();
        let destination = root.path().join("verified.bin");
        let request = contracts::content::DownloadArtifactRequest {
            artifact_id: first_id.clone(),
        };
        let verified = native
            .download_artifact(request.clone(), destination.clone())
            .await
            .unwrap();
        assert_eq!(verified.metadata.artifact_id, first_id);
        assert_eq!(std::fs::read(&destination).unwrap(), records[0].1);
        assert_eq!(verified.size_bytes, records[0].1.len() as u64);
        assert!(!root.path().join("verified.bin.part").exists());
        // Invoke The Live Catalog's Streaming Operation.
        let request_json =
            serde_json::to_vec(&serde_json::json!({"artifactId": first_id})).unwrap();
        let dynamic = native
            .call_operation(
                <contracts::content::DownloadArtifact as harmos::operations::OperationDeclaration>::ID,
                None,
                &request_json,
                vec![],
            )
            .await
            .unwrap();
        let mut item_count = 0;
        let mut completed = false;
        {
            while let Some(event) = dynamic.message().await.unwrap() {
                assert!(!completed, "dynamic completion must be the final event");
                match event {
                    dokime_client::dynamic::Event::Item(value) => {
                        assert!(value.as_object().is_some_and(|fields| !fields.is_empty()));
                        item_count += 1;
                    }
                    dokime_client::dynamic::Event::Completion(_) => {
                        assert!(
                            item_count >= 3,
                            "metadata, content and artifact completion precede operation completion"
                        );
                        completed = true;
                    }
                }
            }
        }
        assert!(
            completed,
            "dynamic stream must report completion after final OK status"
        );
        dynamic.close().await;

        artifacts.release.notify_one();
        support::wait_for_cycle_lifecycle(
            &owner.handle(),
            records[0].0.scope.test_cycle_id.as_ref().unwrap(),
            dokime_runtime::execution::CycleStatus::Completed,
        )
        .await;
        // A sealed execution cannot acquire another mutable artifact producer.
        assert!(
            native
                .upload_artifact_file(
                    &destination,
                    contracts::content::UploadArtifactHeader {
                        scope: verified.metadata.scope.clone(),
                        filename: "native-upload.bin".into(),
                        content_type: verified.metadata.content_type.clone(),
                        size_bytes: 0,
                        sha256: None,
                    },
                )
                .await
                .is_err()
        );

        let directory = root.path().join("publication-directory");
        std::fs::create_dir(&directory).unwrap();
        assert!(matches!(
            native.download_artifact(request.clone(), directory).await,
            Err(dokime_client::DownloadError::Publication(_))
        ));
        assert!(root.path().join("publication-directory.part").exists());

        let partial = root.path().join("verified.bin.part");
        std::fs::write(&partial, b"another owner").unwrap();
        assert!(matches!(
            native.download_artifact(request, destination).await,
            Err(dokime_client::DownloadError::Io(_))
        ));
        assert_eq!(std::fs::read(&partial).unwrap(), b"another owner");
        native.close_and_wait().await;
    } else {
        let failure = dokime_client::Client::connect(
            dokime_client::Target::local(server.local_endpoint()).unwrap(),
            Some(TOKEN),
            None,
        )
        .await
        .err()
        .expect("the full availability inventory exceeds the small message budget");
        assert!(matches!(
            failure,
            dokime_client::Error::Rpc(status) if status.code() == Code::ResourceExhausted
        ));
        artifacts.release.notify_one();
        support::wait_for_cycle_lifecycle(
            &owner.handle(),
            records[0].0.scope.test_cycle_id.as_ref().unwrap(),
            dokime_runtime::execution::CycleStatus::Completed,
        )
        .await;
    }

    {
        for (record, expected) in records.iter().take(3) {
            let id = record.artifact_id.to_string();
            let mut stream = content
                .download::<contracts::content::DownloadArtifact>(authenticated(
                    contracts::content::DownloadArtifactRequest { artifact_id: id },
                ))
                .await
                .unwrap()
                .into_inner();
            let Some(contracts::content::ArtifactFramePayload::Metadata(metadata)) =
                stream.message().await.unwrap().unwrap().payload
            else {
                panic!("an artifact download opens with its metadata");
            };
            assert_eq!(metadata.size_bytes, expected.len() as u64);
            assert_eq!(
                metadata.sha256,
                Sha256::digest(expected)
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            );
            assert_eq!(
                metadata.state,
                if record.filename == "partial.bin" {
                    contracts::content::ArtifactState::Cancelled
                } else {
                    contracts::content::ArtifactState::Complete
                }
            );
            let mut actual = Vec::new();
            let mut complete = false;
            while let Some(frame) = stream.message().await.unwrap() {
                assert!(!complete, "completion must be the last frame");
                match frame.payload.unwrap() {
                    contracts::content::ArtifactFramePayload::Chunk(chunk) => {
                        assert_eq!(chunk.offset, actual.len() as u64);
                        assert!(!chunk.data.is_empty());
                        assert!(chunk.data.len() <= message_bytes - 32);
                        actual.extend(chunk.data);
                    }
                    contracts::content::ArtifactFramePayload::Complete(done) => {
                        assert_eq!(done.size_bytes, expected.len() as u64);
                        assert_eq!(
                            done.sha256,
                            Sha256::digest(&actual)
                                .iter()
                                .map(|byte| format!("{byte:02x}"))
                                .collect::<String>()
                        );
                        complete = true;
                    }
                    _ => panic!("duplicate metadata"),
                }
            }
            assert!(complete);
            assert_eq!(&actual, expected);
        }
    }
    let mut stalled = content
        .download::<contracts::content::DownloadArtifact>(authenticated(
            contracts::content::DownloadArtifactRequest {
                artifact_id: records[3].0.artifact_id.to_string(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert!(matches!(
        stalled.message().await.unwrap().unwrap().payload,
        Some(contracts::content::ArtifactFramePayload::Metadata(_))
    ));
    // The server grants existing connections a five-second graceful drain,
    // then disconnects a stalled peer.
    server.shutdown().await.unwrap();
    let interrupted = async {
        loop {
            match stalled.message().await {
                Err(_) => return true,
                Ok(None) => return false,
                Ok(Some(frame)) => assert!(!matches!(
                    frame.payload,
                    Some(contracts::content::ArtifactFramePayload::Complete(_))
                )),
            }
        }
    }
    .await;
    assert!(
        interrupted,
        "shutdown must not report a truncated transfer as successful"
    );
    drop(stalled);
    drop(content);
    drop(execution);
    let mut reader = owner
        .handle()
        .open_artifact_stream(&records[0].0.artifact_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(reader.record().sha256, records[0].0.sha256);
    assert!(reader.read(&mut [0; 16]).await.unwrap() > 0);
    drop(reader);
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn artifact_upload_owned() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(UploadGate::default());
    let mut owner = Runtime::with_execution_fixture(
        crate::support::budgets::runtime_config(
            root.path(),
            support::metadata("grpc.content.upload", "Content"),
        ),
        "fixture",
        gate.clone(),
    )
    .await
    .unwrap();
    support::register_fixture_execution(&owner.handle(), support::fixture_suite(&["passed"])).await;
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Control)])
            .unwrap()
            // The native client needs the full availability inventory, which is charged
            // about 14 KiB for 109 gRPC paths and cannot fit a 4 KiB budget (see
            // `artifact_transfer_budget`).
            .with_max_message_bytes(1024 * 1024),
    )
    .await
    .unwrap();
    let stream = Endpoint::from_shared(server.local_endpoint())
        .unwrap()
        .connect()
        .await
        .unwrap();
    let execution = WireClient::new(stream.clone());
    let cycle_id = support::planned_cycle(
        &execution,
        authenticated(contracts::execution::CreateCycleRequest {
            metadata: Some(contracts::common::Metadata {
                identifier: "content.upload".into(),
                label: "Upload".into(),
                description: None,
                ..Default::default()
            }),
            ..Default::default()
        }),
        selection_plan(&owner),
    )
    .await
    .unwrap();
    execution
        .unary::<contracts::execution::RunCycle>(authenticated(
            contracts::execution::RunCycleRequest {
                cycle_id: cycle_id.clone(),
                context: None,
                idempotency_key: None,
            },
        ))
        .await
        .unwrap();
    gate.started.notified().await;
    let execution_id = execution
        .unary::<contracts::execution::ListExecutions>(authenticated(
            contracts::execution::ListExecutionsRequest {
                cycle_id: cycle_id.clone(),
                limit: 100,
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .executions
        .remove(0)
        .execution_id;
    let scope = || contracts::content::ArtifactScope {
        cycle_id: Some(cycle_id.clone()),
        case_id: None,
        execution_id: Some(execution_id.clone()),
    };
    let bytes = b"native artifact upload".to_vec();
    let digest = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    // Upload frames stay small; listings are paged to the larger server budget.
    let content = WireClient::new(stream)
        .max_decoding_message_size(1024 * 1024)
        .max_encoding_message_size(4096);
    let uploaded = content
        .upload::<contracts::content::UploadArtifact>(authenticated(tokio_stream::iter(vec![
            contracts::content::UploadArtifactFrame {
                payload: Some(contracts::content::UploadArtifactFramePayload::Header(
                    contracts::content::UploadArtifactHeader {
                        scope: Some(scope()),
                        filename: "native.bin".into(),
                        content_type: Some("application/octet-stream".into()),
                        size_bytes: bytes.len() as u64,
                        sha256: Some(digest.clone()),
                    },
                )),
            },
            contracts::content::UploadArtifactFrame {
                payload: Some(contracts::content::UploadArtifactFramePayload::Chunk(
                    contracts::content::ArtifactChunk {
                        offset: 0,
                        data: bytes.clone(),
                    },
                )),
            },
        ])))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(uploaded.filename, "native.bin");
    assert_eq!(uploaded.state, contracts::content::ArtifactState::Complete);
    assert_eq!(uploaded.sha256, digest);

    // Operator Uploads Land Under Their Folder; Escaping Names Are Refused
    let upload_named = |filename: &str| {
        tokio_stream::iter(vec![
            contracts::content::UploadArtifactFrame {
                payload: Some(contracts::content::UploadArtifactFramePayload::Header(
                    contracts::content::UploadArtifactHeader {
                        scope: Some(scope()),
                        filename: filename.into(),
                        content_type: Some("text/plain".into()),
                        size_bytes: 5,
                        sha256: None,
                    },
                )),
            },
            contracts::content::UploadArtifactFrame {
                payload: Some(contracts::content::UploadArtifactFramePayload::Chunk(
                    contracts::content::ArtifactChunk {
                        offset: 0,
                        data: b"notes".to_vec(),
                    },
                )),
            },
        ])
    };
    let foldered = content
        .upload::<contracts::content::UploadArtifact>(authenticated(upload_named(
            "upload/operator-notes.txt",
        )))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(foldered.filename, "upload/operator-notes.txt");
    assert_eq!(foldered.state, contracts::content::ArtifactState::Complete);
    for escaping in ["../escape.txt", "/absolute.txt", "upload//empty.txt"] {
        let refused = content
            .upload::<contracts::content::UploadArtifact>(authenticated(upload_named(escaping)))
            .await
            .unwrap_err();
        assert_eq!(refused.code(), Code::InvalidArgument, "{escaping}");
    }

    let native = dokime_client::Client::connect(
        dokime_client::Target::local(server.local_endpoint()).unwrap(),
        Some(TOKEN),
        None,
    )
    .await
    .unwrap();
    let source = root.path().join("native-upload.bin");
    std::fs::write(&source, &bytes).unwrap();
    let uploaded = native
        .upload_artifact_file(
            &source,
            contracts::content::UploadArtifactHeader {
                scope: Some(scope()),
                filename: "native-upload.bin".into(),
                content_type: None,
                size_bytes: 0,
                sha256: Some("caller digest is ignored".into()),
            },
        )
        .await
        .unwrap();
    assert_eq!(uploaded.size_bytes, bytes.len() as u64);
    assert_eq!(uploaded.sha256, digest);
    assert_eq!(uploaded.state, contracts::content::ArtifactState::Complete);
    native.close_and_wait().await;

    let error = content
        .upload::<contracts::content::UploadArtifact>(authenticated(tokio_stream::iter(vec![
            contracts::content::UploadArtifactFrame {
                payload: Some(contracts::content::UploadArtifactFramePayload::Header(
                    contracts::content::UploadArtifactHeader {
                        scope: Some(scope()),
                        filename: "bad-offset.bin".into(),
                        content_type: None,
                        size_bytes: 3,
                        sha256: None,
                    },
                )),
            },
            contracts::content::UploadArtifactFrame {
                payload: Some(contracts::content::UploadArtifactFramePayload::Chunk(
                    contracts::content::ArtifactChunk {
                        offset: 1,
                        data: b"bad".to_vec(),
                    },
                )),
            },
        ])))
        .await
        .unwrap_err();
    assert_eq!(error.code(), Code::InvalidArgument);
    assert_eq!(
        owner
            .handle()
            .artifacts()
            .await
            .expect("artifact query succeeds")
            .iter()
            .find(|record| record.filename == "bad-offset.bin")
            .unwrap()
            .status,
        dokime_runtime::artifacts::ArtifactStatus::Cancelled
    );

    let error = content
        .upload::<contracts::content::UploadArtifact>(authenticated(tokio_stream::iter(vec![
            contracts::content::UploadArtifactFrame {
                payload: Some(contracts::content::UploadArtifactFramePayload::Header(
                    contracts::content::UploadArtifactHeader {
                        scope: Some(scope()),
                        filename: "bad-size.bin".into(),
                        content_type: None,
                        size_bytes: 4,
                        sha256: None,
                    },
                )),
            },
            contracts::content::UploadArtifactFrame {
                payload: Some(contracts::content::UploadArtifactFramePayload::Chunk(
                    contracts::content::ArtifactChunk {
                        offset: 0,
                        data: b"bad".to_vec(),
                    },
                )),
            },
        ])))
        .await
        .unwrap_err();
    assert_eq!(error.code(), Code::InvalidArgument);
    assert_eq!(
        owner
            .handle()
            .artifacts()
            .await
            .expect("artifact query succeeds")
            .iter()
            .find(|record| record.filename == "bad-size.bin")
            .unwrap()
            .status,
        dokime_runtime::artifacts::ArtifactStatus::Rejected
    );

    let error = content
        .upload::<contracts::content::UploadArtifact>(authenticated(tokio_stream::iter(vec![
            contracts::content::UploadArtifactFrame {
                payload: Some(contracts::content::UploadArtifactFramePayload::Header(
                    contracts::content::UploadArtifactHeader {
                        scope: Some(scope()),
                        filename: "bad-digest.bin".into(),
                        content_type: None,
                        size_bytes: 3,
                        sha256: Some("0".repeat(64)),
                    },
                )),
            },
            contracts::content::UploadArtifactFrame {
                payload: Some(contracts::content::UploadArtifactFramePayload::Chunk(
                    contracts::content::ArtifactChunk {
                        offset: 0,
                        data: b"bad".to_vec(),
                    },
                )),
            },
        ])))
        .await
        .unwrap_err();
    assert_eq!(error.code(), Code::InvalidArgument);
    assert_eq!(
        owner
            .handle()
            .artifacts()
            .await
            .expect("artifact query succeeds")
            .iter()
            .find(|record| record.filename == "bad-digest.bin")
            .unwrap()
            .status,
        dokime_runtime::artifacts::ArtifactStatus::Rejected
    );

    let listed = content
        .unary::<contracts::content::ListArtifacts>(authenticated(
            contracts::content::ListArtifactsRequest::default(),
        ))
        .await
        .unwrap()
        .into_inner()
        .artifacts;
    for (filename, state) in [
        ("native.bin", contracts::content::ArtifactState::Complete),
        (
            "bad-offset.bin",
            contracts::content::ArtifactState::Cancelled,
        ),
        ("bad-size.bin", contracts::content::ArtifactState::Rejected),
        (
            "bad-digest.bin",
            contracts::content::ArtifactState::Rejected,
        ),
    ] {
        assert_eq!(
            listed
                .iter()
                .find(|artifact| artifact.filename == filename)
                .map(|artifact| artifact.state),
            Some(state),
            "list must preserve {filename} state",
        );
    }

    gate.release.notify_one();
    support::on_state_change(&owner.handle(), async || {
        let cycle = execution
            .unary::<contracts::execution::GetCycle>(authenticated(
                contracts::execution::GetCycleRequest {
                    cycle_id: cycle_id.clone(),
                },
            ))
            .await
            .unwrap()
            .into_inner();
        if cycle.status == contracts::execution::CycleStatus::Completed {
            return Some(());
        }
        None
    })
    .await;
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

async fn completion_harness() -> (
    tempfile::TempDir,
    Runtime,
    Server,
    WireClient,
    WireClient,
    Arc<UploadGate>,
    String,
    contracts::content::ArtifactScope,
) {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(UploadGate::default());
    let owner = Runtime::with_execution_fixture(
        crate::support::budgets::runtime_config(
            root.path(),
            support::metadata("grpc.content.completion", "Content"),
        ),
        "fixture",
        gate.clone(),
    )
    .await
    .unwrap();
    support::register_fixture_execution(&owner.handle(), support::fixture_suite(&["passed"])).await;
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Control)])
            .unwrap()
            .with_max_message_bytes(4096),
    )
    .await
    .unwrap();
    let stream = Endpoint::from_shared(server.local_endpoint())
        .unwrap()
        .connect()
        .await
        .unwrap();
    let execution = WireClient::new(stream.clone());
    let cycle_id = support::planned_cycle(
        &execution,
        authenticated(contracts::execution::CreateCycleRequest {
            metadata: Some(contracts::common::Metadata {
                identifier: "content.completion".into(),
                label: "Completion".into(),
                description: None,
                ..Default::default()
            }),
            ..Default::default()
        }),
        selection_plan(&owner),
    )
    .await
    .unwrap();
    execution
        .unary::<contracts::execution::RunCycle>(authenticated(
            contracts::execution::RunCycleRequest {
                cycle_id: cycle_id.clone(),
                context: None,
                idempotency_key: None,
            },
        ))
        .await
        .unwrap();
    gate.started.notified().await;
    let execution_id = execution
        .unary::<contracts::execution::ListExecutions>(authenticated(
            contracts::execution::ListExecutionsRequest {
                cycle_id: cycle_id.clone(),
                limit: 100,
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .executions
        .remove(0)
        .execution_id;
    let scope = contracts::content::ArtifactScope {
        cycle_id: Some(cycle_id.clone()),
        case_id: None,
        execution_id: Some(execution_id),
    };
    let content = WireClient::new(stream)
        .max_decoding_message_size(4096)
        .max_encoding_message_size(4096);
    (
        root, owner, server, execution, content, gate, cycle_id, scope,
    )
}

/// Wait on the live state the settled-evidence refusal reads, not on a sibling source.
async fn wait_for_watched_lifecycle(
    owner: &Runtime,
    cycle: &dokime_runtime::core::TestCycleId,
    lifecycle: dokime_runtime::execution::CycleStatus,
) {
    owner
        .handle()
        .watch_state()
        .wait_until(|snapshot| {
            snapshot
                .state()
                .cycles
                .get(cycle)
                .map(|cycle| cycle.lifecycle)
                == Some(lifecycle)
        })
        .await
        .expect("the runtime state watch stays open");
}

async fn finish_completion_harness(
    execution: WireClient,
    cycle_id: String,
    gate: Arc<UploadGate>,
    mut owner: Runtime,
    server: Server,
) {
    gate.release.notify_one();
    support::on_state_change(&owner.handle(), async || {
        let cycle = execution
            .unary::<contracts::execution::GetCycle>(authenticated(
                contracts::execution::GetCycleRequest {
                    cycle_id: cycle_id.clone(),
                },
            ))
            .await
            .unwrap()
            .into_inner();
        if cycle.status == contracts::execution::CycleStatus::Completed {
            return Some(());
        }
        None
    })
    .await;
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn artifact_upload_digest() {
    let (_root, owner, server, execution, content, gate, cycle_id, scope) =
        completion_harness().await;
    let bytes = b"trailing completion digest upload".to_vec();
    let digest = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let uploaded = content
        .upload::<contracts::content::UploadArtifact>(authenticated(tokio_stream::iter(vec![
            contracts::content::UploadArtifactFrame {
                payload: Some(contracts::content::UploadArtifactFramePayload::Header(
                    contracts::content::UploadArtifactHeader {
                        scope: Some(scope.clone()),
                        filename: "trailing.bin".into(),
                        content_type: Some("application/octet-stream".into()),
                        size_bytes: bytes.len() as u64,
                        sha256: None,
                    },
                )),
            },
            contracts::content::UploadArtifactFrame {
                payload: Some(contracts::content::UploadArtifactFramePayload::Chunk(
                    contracts::content::ArtifactChunk {
                        offset: 0,
                        data: bytes.clone(),
                    },
                )),
            },
            contracts::content::UploadArtifactFrame {
                payload: Some(contracts::content::UploadArtifactFramePayload::Complete(
                    contracts::content::ArtifactCompletion {
                        size_bytes: bytes.len() as u64,
                        sha256: digest.clone(),
                    },
                )),
            },
        ])))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(uploaded.filename, "trailing.bin");
    assert_eq!(uploaded.state, contracts::content::ArtifactState::Complete);
    assert_eq!(uploaded.sha256, digest);
    finish_completion_harness(execution, cycle_id, gate, owner, server).await;
}

#[tokio::test]
async fn artifact_upload_match() {
    let (_root, owner, server, execution, content, gate, cycle_id, scope) =
        completion_harness().await;
    let bytes = b"mismatched trailing digest upload".to_vec();
    let wrong_digest = Sha256::digest(b"not the uploaded bytes")
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let error = content
        .upload::<contracts::content::UploadArtifact>(authenticated(tokio_stream::iter(vec![
            contracts::content::UploadArtifactFrame {
                payload: Some(contracts::content::UploadArtifactFramePayload::Header(
                    contracts::content::UploadArtifactHeader {
                        scope: Some(scope.clone()),
                        filename: "mismatched.bin".into(),
                        content_type: None,
                        size_bytes: bytes.len() as u64,
                        sha256: None,
                    },
                )),
            },
            contracts::content::UploadArtifactFrame {
                payload: Some(contracts::content::UploadArtifactFramePayload::Chunk(
                    contracts::content::ArtifactChunk {
                        offset: 0,
                        data: bytes.clone(),
                    },
                )),
            },
            contracts::content::UploadArtifactFrame {
                payload: Some(contracts::content::UploadArtifactFramePayload::Complete(
                    contracts::content::ArtifactCompletion {
                        size_bytes: bytes.len() as u64,
                        sha256: wrong_digest,
                    },
                )),
            },
        ])))
        .await
        .unwrap_err();
    assert_eq!(error.code(), Code::InvalidArgument);
    assert_eq!(
        owner
            .handle()
            .artifacts()
            .await
            .expect("artifact query succeeds")
            .iter()
            .find(|record| record.filename == "mismatched.bin")
            .unwrap()
            .status,
        dokime_runtime::artifacts::ArtifactStatus::Rejected
    );
    finish_completion_harness(execution, cycle_id, gate, owner, server).await;
}

#[tokio::test]
async fn artifact_upload_completion() {
    let (_root, owner, server, execution, content, gate, cycle_id, scope) =
        completion_harness().await;
    let bytes = b"frame after completion".to_vec();
    let digest = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let error = content
        .upload::<contracts::content::UploadArtifact>(authenticated(tokio_stream::iter(vec![
            contracts::content::UploadArtifactFrame {
                payload: Some(contracts::content::UploadArtifactFramePayload::Header(
                    contracts::content::UploadArtifactHeader {
                        scope: Some(scope.clone()),
                        filename: "trailing-extra.bin".into(),
                        content_type: None,
                        size_bytes: bytes.len() as u64,
                        sha256: None,
                    },
                )),
            },
            contracts::content::UploadArtifactFrame {
                payload: Some(contracts::content::UploadArtifactFramePayload::Chunk(
                    contracts::content::ArtifactChunk {
                        offset: 0,
                        data: bytes.clone(),
                    },
                )),
            },
            contracts::content::UploadArtifactFrame {
                payload: Some(contracts::content::UploadArtifactFramePayload::Complete(
                    contracts::content::ArtifactCompletion {
                        size_bytes: bytes.len() as u64,
                        sha256: digest,
                    },
                )),
            },
            contracts::content::UploadArtifactFrame {
                payload: Some(contracts::content::UploadArtifactFramePayload::Chunk(
                    contracts::content::ArtifactChunk {
                        offset: bytes.len() as u64,
                        data: b"x".to_vec(),
                    },
                )),
            },
        ])))
        .await
        .unwrap_err();
    assert_eq!(error.code(), Code::InvalidArgument);
    finish_completion_harness(execution, cycle_id, gate, owner, server).await;
}

#[tokio::test]
async fn open_report_states() {
    let (_root, mut owner, server, execution, _content, gate, cycle_id, scope) =
        completion_harness().await;
    let cycle = dokime_runtime::core::TestCycleId::from_uuid(cycle_id.clone()).unwrap();
    let execution_id = scope
        .execution_id
        .clone()
        .expect("captured execution scope");
    wait_for_watched_lifecycle(&owner, &cycle, dokime_runtime::execution::CycleStatus::Running).await;

    // Running: Both Reports Are Rendered From The Current Evidence
    let (metadata, html) = cycle_report(&execution, &cycle_id).await;
    assert_eq!(metadata.origin, contracts::reports::ReportOrigin::Current);
    assert_eq!(metadata.test_execution_id, None);
    assert!(String::from_utf8_lossy(&html).contains("<!doctype html>"));
    let (metadata, html) = execution_report(&execution, &execution_id).await;
    assert_eq!(metadata.origin, contracts::reports::ReportOrigin::Current);
    assert_eq!(metadata.test_cycle_id, cycle_id);
    assert_eq!(
        metadata.test_execution_id.as_deref(),
        Some(execution_id.as_str())
    );
    assert!(String::from_utf8_lossy(&html).contains("<!doctype html>"));

    // Terminal: Both Reports Are The Sealed Bytes, Unchanged Across Reads
    gate.release.notify_one();
    wait_for_watched_lifecycle(
        &owner,
        &cycle,
        dokime_runtime::execution::CycleStatus::Completed,
    )
    .await;
    let (metadata, first) = cycle_report(&execution, &cycle_id).await;
    assert_eq!(metadata.origin, contracts::reports::ReportOrigin::Sealed);
    let (_, second) = cycle_report(&execution, &cycle_id).await;
    assert_eq!(
        first, second,
        "a sealed cycle report must read back unchanged"
    );
    let (metadata, _) = execution_report(&execution, &execution_id).await;
    assert_eq!(metadata.origin, contracts::reports::ReportOrigin::Sealed);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

async fn cycle_report(
    client: &WireClient,
    cycle_id: &str,
) -> (contracts::reports::ReportMetadata, Vec<u8>) {
    let stream = client
        .download::<contracts::reports::DownloadCycleReport>(authenticated(
            contracts::reports::DownloadCycleReportRequest {
                cycle_id: cycle_id.to_owned(),
                identifier: String::new(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    report_frames(stream).await
}

async fn execution_report(
    client: &WireClient,
    execution_id: &str,
) -> (contracts::reports::ReportMetadata, Vec<u8>) {
    let stream = client
        .download::<contracts::reports::DownloadExecutionReport>(authenticated(
            contracts::reports::DownloadExecutionReportRequest {
                execution_id: execution_id.to_owned(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    report_frames(stream).await
}

async fn report_frames(
    mut stream: WireStream<contracts::reports::ReportHtmlFrame>,
) -> (contracts::reports::ReportMetadata, Vec<u8>) {
    let mut metadata = None;
    let mut html = Vec::new();
    let mut complete = false;
    while let Some(frame) = stream.message().await.unwrap() {
        match frame.payload.expect("report payload") {
            contracts::reports::ReportHtmlFramePayload::Report(report) => {
                assert!(metadata.replace(*report).is_none(), "one metadata frame");
            }
            contracts::reports::ReportHtmlFramePayload::Chunk(chunk) => {
                assert_eq!(chunk.offset, html.len() as u64);
                html.extend_from_slice(&chunk.data);
            }
            contracts::reports::ReportHtmlFramePayload::Complete(done) => {
                assert_eq!(done.size_bytes, html.len() as u64);
                complete = true;
            }
        }
    }
    assert!(complete, "report stream must complete");
    (metadata.expect("report metadata"), html)
}

#[tokio::test]
async fn large_artifact_buffering() {
    if std::env::var_os("RUST_LOG").is_some() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_test_writer()
            .try_init();
    }
    // Stream 300 MiB through the existing native file contract.
    let root = tempfile::tempdir().unwrap();
    let config = crate::support::budgets::runtime_config(
        root.path(),
        support::metadata("grpc.content.bundle", "Content"),
    );
    let mut owner = Runtime::with_execution_fixture(config, "fixture", Arc::new(LargeArtifact))
        .await
        .unwrap();
    support::register_fixture_execution(&owner.handle(), support::fixture_suite(&["passed"])).await;
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Control)])
            .unwrap(),
    )
    .await
    .unwrap();
    let stream = Endpoint::from_shared(server.local_endpoint())
        .unwrap()
        .connect()
        .await
        .unwrap();
    let execution = WireClient::new(stream.clone());
    let cycle_id = support::planned_cycle(
        &execution,
        authenticated(contracts::execution::CreateCycleRequest {
            metadata: Some(contracts::common::Metadata {
                identifier: "content.bundle".into(),
                label: "Bundle".into(),
                description: None,
                ..Default::default()
            }),
            ..Default::default()
        }),
        selection_plan(&owner),
    )
    .await
    .unwrap();
    execution
        .unary::<contracts::execution::RunCycle>(authenticated(
            contracts::execution::RunCycleRequest {
                cycle_id: cycle_id.clone(),
                context: None,
                idempotency_key: None,
            },
        ))
        .await
        .unwrap();
    support::on_state_change(&owner.handle(), async || {
        let cycle = execution
            .unary::<contracts::execution::GetCycle>(authenticated(
                contracts::execution::GetCycleRequest {
                    cycle_id: cycle_id.clone(),
                },
            ))
            .await
            .unwrap()
            .into_inner();
        assert_ne!(cycle.status, contracts::execution::CycleStatus::Failed);
        (cycle.status == contracts::execution::CycleStatus::Completed).then_some(())
    })
    .await;
    let content = WireClient::new(stream);
    let artifacts = owner.handle().artifacts().await.unwrap();
    let artifact = artifacts
        .iter()
        .find(|artifact| artifact.filename == "large.bin")
        .unwrap();
    let mut download = content
        .download::<contracts::content::DownloadArtifact>(authenticated(
            contracts::content::DownloadArtifactRequest {
                artifact_id: artifact.artifact_id.to_string(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let metadata = match download.message().await.unwrap().unwrap().payload {
        Some(contracts::content::ArtifactFramePayload::Metadata(metadata)) => metadata,
        payload => panic!("unexpected artifact header: {payload:?}"),
    };
    assert_eq!(metadata.size_bytes, LARGE_ARTIFACT_BYTES);
    let mut downloaded = 0_u64;
    let mut digest = Sha256::new();
    let completion = loop {
        let frame = download
            .message()
            .await
            .unwrap()
            .expect("artifact completion");
        match frame.payload.unwrap() {
            contracts::content::ArtifactFramePayload::Chunk(chunk) => {
                assert_eq!(chunk.offset, downloaded);
                assert!(!chunk.data.is_empty());
                downloaded += chunk.data.len() as u64;
                digest.update(&chunk.data);
            }
            contracts::content::ArtifactFramePayload::Complete(done) => break done,
            contracts::content::ArtifactFramePayload::Metadata(_) => {
                panic!("artifact metadata must be emitted once")
            }
        }
    };
    assert_eq!(completion.size_bytes, downloaded);
    assert_eq!(
        completion.sha256,
        digest
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
    assert_eq!(metadata.size_bytes, downloaded);
    assert_eq!(metadata.sha256, completion.sha256);
    assert!(download.message().await.unwrap().is_none());
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */
