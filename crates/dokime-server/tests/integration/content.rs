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
use clap::Parser;
use futures_util::FutureExt;
use sha2::{Digest, Sha256};
use tokio::sync::Notify;
// Local Imports
use super::*;
use dokime::RecordScope;
use dokime::artifacts::{ArtifactRecord, ArtifactUploadInput};
use dokime::domain::model;
use dokime::domain::model::execution::result::CycleView;
use dokime::domains::ExecutionProviderTarget;
use dokime::{ExecutionFixture, ExecutionFixtureContext, ExecutionFixtureFuture};

/* === Constants === */

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

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl ExecutionFixture for Artifacts {
    fn run_cycle<'a>(
        &'a self, context: ExecutionFixtureContext, cycle: &'a CycleView, _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            let execution = &cycle.test_executions[0];
            for (name, size, cancelled) in
                [("full.bin", 150_000, false), ("empty.bin", 0, false), ("partial.bin", 8_000, true), ("large.bin", 2 * 1024 * 1024, false)]
            {
                let bytes = (0..size).map(|index| (index % 251) as u8).collect::<Vec<_>>();
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
                    upload.send(chunk.to_vec()).await.unwrap();
                }
                let record = if cancelled { upload.abandon().await } else { upload.finish().await }.unwrap();
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
        &'a self, _context: ExecutionFixtureContext, cycle: &'a CycleView, _target: &'a ExecutionProviderTarget,
    ) -> ExecutionFixtureFuture<'a> {
        Box::pin(async move {
            self.started.notify_one();
            self.release.notified().await;
            Ok(support::fixture_provider_outcome(cycle, false))
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */

// A 4 KiB Frame Limit Pages Listings And Chunks Downloads Into Small Frames
#[tokio::test]
async fn small_frame_transfer() {
    artifact_transfer(4096).boxed_local().await;
}

#[tokio::test]
async fn large_frame_transfer() {
    artifact_transfer(1024 * 1024).boxed_local().await;
}

async fn artifact_transfer(message_bytes: usize) {
    let root = tempfile::tempdir().unwrap();
    let artifacts = Arc::new(Artifacts::default());
    let prepared_owner = dokime_server::grpc::Service::prepare(keyed().with_max_message_bytes(message_bytes)).unwrap();
    let mut owner = Runtime::with_execution_fixture(
        prepared_owner.configuration(crate::support::budgets::runtime_config(root.path(), support::metadata("grpc.content", "Content"))),
        "fixture",
        artifacts.clone(),
    )
    .boxed_local()
    .await
    .unwrap();
    support::register_fixture_execution(&owner.handle(), support::fixture_suite(&["passed"])).await;
    let service = prepared_owner.ready().await.unwrap();
    let server = service.listen().await.unwrap();
    let stream = Endpoint::from_shared(server.local_endpoint()).unwrap().initial_stream_window_size(4096).connect().await.unwrap();
    let execution = WireClient::new(stream.clone());
    let cycle_id = support::planned_cycle(
        &execution,
        authenticated(contracts::execution::CreateCycle {
            metadata: Some(model::identity::wire::Metadata {
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
    let reply =
        execution.start_first(authenticated(contracts::execution::GetCycle { cycle_id: cycle_id.clone() })).await.unwrap().into_inner();
    assert!(matches!(reply.outcome, Some(model::execution::wire::RunReplyOutcome::Accepted(_))));
    artifacts.ready.notified().await;
    let records = artifacts.records.lock().unwrap().clone();
    assert_eq!(records.len(), 4);
    let content = WireClient::new(stream).max_decoding_message_size(message_bytes);
    let first_page = content
        .unary::<contracts::content::ListArtifacts>(authenticated(contracts::content::ListArtifacts { limit: 2, ..Default::default() }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(first_page.artifacts.len(), 2);
    let next_after = first_page.next_after_artifact_id.clone().unwrap();
    let cursor = first_page.cursor.clone().unwrap();
    let second_page = content
        .unary::<contracts::content::ListArtifacts>(authenticated(contracts::content::ListArtifacts {
            limit: 2,
            after_artifact_id: next_after,
            cursor: Some(cursor.clone()),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(second_page.artifacts.len(), 2);
    assert!(second_page.next_after_artifact_id.is_none());
    let mut invalid_cursor = cursor;
    invalid_cursor.metadata_sha256[0] ^= 1;
    assert_eq!(
        content
            .unary::<contracts::content::ListArtifacts>(authenticated(contracts::content::ListArtifacts {
                limit: 2,
                after_artifact_id: first_page.next_after_artifact_id.clone().unwrap(),
                cursor: Some(invalid_cursor),
            }))
            .await
            .unwrap_err()
            .code(),
        Code::Aborted
    );
    let first_id = records[0].0.artifact_id.to_string();
    assert_eq!(
        content
            .download::<contracts::content::DownloadArtifact>(contracts::content::DownloadArtifact { artifact_id: first_id.clone() })
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    assert_eq!(
        content
            .download::<contracts::content::DownloadArtifact>(authenticated(contracts::content::DownloadArtifact {
                artifact_id: "../file".into()
            }))
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
            .download::<contracts::content::DownloadArtifact>(authenticated(contracts::content::DownloadArtifact {
                artifact_id: dokime::model::identity::new_id::<dokime::ArtifactId>().to_string(),
            }))
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
        let native = dokime::client::Client::connect(dokime::client::Target::local(server.local_endpoint()).unwrap(), Some(TOKEN), None)
            .await
            .unwrap();
        let destination = root.path().join("verified.bin");
        let request = contracts::content::DownloadArtifact { artifact_id: first_id.clone() };
        let verified = native.download_artifact(request.clone(), destination.clone()).await.unwrap();
        assert_eq!(verified.metadata.artifact_id, first_id);
        assert_eq!(std::fs::read(&destination).unwrap(), records[0].1);
        assert_eq!(verified.size_bytes, records[0].1.len() as u64);
        assert!(!root.path().join("verified.bin.part").exists());
        // Invoke The Live Catalog's Streaming Operation.
        let request_json = serde_json::to_vec(&serde_json::json!({"artifactId": first_id})).unwrap();
        let dynamic = native
            .call_operation(
                <contracts::content::DownloadArtifact as dokime::harmos::operations::OperationDeclaration>::ID,
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
                    dokime::client::dynamic::Event::Item(value) => {
                        assert!(value.as_object().is_some_and(|fields| !fields.is_empty()));
                        item_count += 1;
                    }
                    dokime::client::dynamic::Event::Completion(_) => {
                        assert!(item_count >= 3, "metadata, content and artifact completion precede operation completion");
                        completed = true;
                    }
                }
            }
        }
        assert!(completed, "dynamic stream must report completion after final OK status");
        dynamic.close().await;

        // The CLI Saves The Same Streaming Protocol As A Complete JSON Document.
        let cli_server = service.listen_with(Config::local().with_max_message_bytes(message_bytes)).await.unwrap();
        let response_file = root.path().join("cli-response.json");
        let endpoint = cli_server.local_endpoint();
        let cli_request = std::str::from_utf8(&request_json).unwrap();
        let command = [
            "dokime",
            "operation",
            "call",
            "dokime.content.download_artifact",
            "--endpoint",
            &endpoint,
            "--json",
            cli_request,
            "--output",
            response_file.to_str().unwrap(),
        ];
        dokime_cli::Cli::try_parse_from(command).unwrap().run(dokime::config::Config::local(root.path())).await.unwrap();
        let bytes = std::fs::read(&response_file).unwrap();
        let response: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        let events = response.as_array().unwrap();
        assert_eq!(events.len(), item_count + 1);
        assert!(events[..item_count].iter().all(|event| event["event"] == "item"));
        assert_eq!(events[item_count]["event"], "completion");

        artifacts.release.notify_one();
        support::wait_for_cycle_outcome(
            &owner.handle(),
            records[0].0.scope.test_cycle_id.as_ref().unwrap(),
            dokime::domain::model::execution::result::ResultStatus::Passed,
        )
        .await;
        // Export Sealed Evidence Through The Same Runtime And Shared Client.
        let execution_id = records[0].0.scope.test_execution_id.as_ref().unwrap().clone();
        let execution_handle = native.execution(execution_id.clone());
        owner
            .handle()
            .watch_state()
            .wait_until(|state| state.state().executions.get(&execution_id).is_some_and(|item| item.saved.is_some()))
            .await
            .unwrap();
        let evidence = root.path().join("execution.zip");
        let downloaded = execution_handle.export(evidence.clone(), false).await.unwrap();
        assert_eq!(downloaded.metadata.subject, Some(model::exports::EvidenceSubject::Execution(execution_id.to_string())));
        let previous = std::fs::read(&evidence).unwrap();
        assert_eq!(previous.len() as u64, downloaded.metadata.size_bytes);
        assert_eq!(Sha256::digest(&previous).iter().map(|byte| format!("{byte:02x}")).collect::<String>(), downloaded.metadata.sha256);
        assert!(execution_handle.export(evidence.clone(), false).await.is_err());
        assert_eq!(std::fs::read(&evidence).unwrap(), previous);
        execution_handle.export(evidence.clone(), true).await.unwrap();
        let mut archive = zip::ZipArchive::new(std::fs::File::open(&evidence).unwrap()).unwrap();
        let manifest: serde_json::Value = serde_json::from_reader(archive.by_name("manifest.json").unwrap()).unwrap();
        assert_eq!(manifest["executions"].as_array().unwrap().len(), 1);
        assert!(manifest["payloads"].as_array().unwrap().iter().any(|entry| entry["path"] == "evidence.sqlite"));
        drop(archive);
        let cycle_id = records[0].0.scope.test_cycle_id.as_ref().unwrap().clone();
        native.cycle(cycle_id.clone()).seal().await.unwrap();
        let cycle_export = root.path().join("cycle.zip");
        let endpoint = cli_server.local_endpoint();
        let output = cycle_export.to_str().unwrap();
        let cycle_id = cycle_id.to_string();
        let command = ["dokime", "cycle", "export", &cycle_id, "--endpoint", &endpoint, "--output", output];
        dokime_cli::Cli::try_parse_from(command).unwrap().run(dokime::config::Config::local(root.path())).await.unwrap();
        cli_server.shutdown().await.unwrap();
        assert!(cycle_export.is_file());
        assert!(!root.path().join("cycle.zip.part").exists());
        // A sealed execution cannot acquire another mutable artifact producer.
        assert!(
            native
                .upload_artifact_file(
                    &destination,
                    model::content::wire::UploadArtifactHeader {
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
        assert!(matches!(native.download_artifact(request.clone(), directory).await, Err(dokime::client::DownloadError::Publication(_))));
        assert!(!root.path().join("publication-directory.part").exists());

        let partial = root.path().join("verified.bin.part");
        std::fs::write(&partial, b"another owner").unwrap();
        assert!(matches!(native.download_artifact(request, destination).await, Err(dokime::client::DownloadError::Io(_))));
        assert_eq!(std::fs::read(&partial).unwrap(), b"another owner");
        native.close_and_wait().await;
    } else {
        let failure = dokime::client::Client::connect(dokime::client::Target::local(server.local_endpoint()).unwrap(), Some(TOKEN), None)
            .await
            .err()
            .expect("the full availability inventory exceeds the small message budget");
        assert!(matches!(
            failure,
            dokime::client::Error::Rpc(status) if status.code() == Code::ResourceExhausted
        ));
        artifacts.release.notify_one();
        support::wait_for_cycle_outcome(
            &owner.handle(),
            records[0].0.scope.test_cycle_id.as_ref().unwrap(),
            dokime::domain::model::execution::result::ResultStatus::Passed,
        )
        .await;
    }

    {
        for (record, expected) in records.iter().take(3) {
            let id = record.artifact_id.to_string();
            let mut stream = content
                .download::<contracts::content::DownloadArtifact>(authenticated(contracts::content::DownloadArtifact { artifact_id: id }))
                .await
                .unwrap()
                .into_inner();
            let Some(model::content::wire::ArtifactFramePayload::Metadata(metadata)) = stream.message().await.unwrap().unwrap().payload
            else {
                panic!("an artifact download opens with its metadata");
            };
            assert_eq!(metadata.size_bytes, expected.len() as u64);
            assert_eq!(metadata.sha256, Sha256::digest(expected).iter().map(|byte| format!("{byte:02x}")).collect::<String>());
            assert_eq!(
                metadata.state,
                if record.filename == "partial.bin" {
                    model::content::wire::ArtifactState::Cancelled
                } else {
                    model::content::wire::ArtifactState::Complete
                }
            );
            let mut actual = Vec::new();
            let mut complete = false;
            while let Some(frame) = stream.message().await.unwrap() {
                assert!(!complete, "completion must be the last frame");
                match frame.payload.unwrap() {
                    model::content::wire::ArtifactFramePayload::Chunk(chunk) => {
                        assert_eq!(chunk.offset, actual.len() as u64);
                        assert!(!chunk.data.is_empty());
                        assert!(chunk.data.len() <= message_bytes - 32);
                        actual.extend(chunk.data);
                    }
                    model::content::wire::ArtifactFramePayload::Complete(done) => {
                        assert_eq!(done.size_bytes, expected.len() as u64);
                        assert_eq!(done.sha256, Sha256::digest(&actual).iter().map(|byte| format!("{byte:02x}")).collect::<String>());
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
        .download::<contracts::content::DownloadArtifact>(authenticated(contracts::content::DownloadArtifact {
            artifact_id: records[3].0.artifact_id.to_string(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert!(matches!(stalled.message().await.unwrap().unwrap().payload, Some(model::content::wire::ArtifactFramePayload::Metadata(_))));
    // The server grants existing connections a five-second graceful drain,
    // then disconnects a stalled peer.
    server.shutdown().await.unwrap();
    let interrupted = async {
        loop {
            match stalled.message().await {
                Err(_) => return true,
                Ok(None) => return false,
                Ok(Some(frame)) => assert!(!matches!(frame.payload, Some(model::content::wire::ArtifactFramePayload::Complete(_)))),
            }
        }
    }
    .await;
    assert!(interrupted, "shutdown must not report a truncated transfer as successful");
    drop(stalled);
    drop(content);
    drop(execution);
    let mut reader = owner.handle().open_artifact_stream(&records[0].0.artifact_id).await.unwrap().unwrap();
    assert_eq!(reader.record().sha256, records[0].0.sha256);
    assert!(reader.read(&mut [0; 16]).await.unwrap() > 0);
    drop(reader);
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn artifact_upload_owned() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(UploadGate::default());
    let prepared_upload = dokime_server::grpc::Service::prepare(keyed().with_max_message_bytes(1024 * 1024)).unwrap();
    let mut owner = Runtime::with_execution_fixture(
        prepared_upload
            .configuration(crate::support::budgets::runtime_config(root.path(), support::metadata("grpc.content.upload", "Content"))),
        "fixture",
        gate.clone(),
    )
    .boxed_local()
    .await
    .unwrap();
    support::register_fixture_execution(&owner.handle(), support::fixture_suite(&["passed"])).await;
    let server = prepared_upload.ready().await.unwrap().listen().await.unwrap();
    let stream = Endpoint::from_shared(server.local_endpoint()).unwrap().connect().await.unwrap();
    let execution = WireClient::new(stream.clone());
    let cycle_id = support::planned_cycle(
        &execution,
        authenticated(contracts::execution::CreateCycle {
            metadata: Some(model::identity::wire::Metadata {
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
    execution.start_first(authenticated(contracts::execution::GetCycle { cycle_id: cycle_id.clone() })).await.unwrap();
    gate.started.notified().await;
    let execution_id = execution
        .unary::<contracts::execution::ListExecutions>(authenticated(contracts::execution::ListExecutions {
            cycle_id: cycle_id.clone(),
            limit: 100,
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner()
        .executions
        .remove(0)
        .execution_id;
    let scope = || model::content::wire::ArtifactScope {
        cycle_id: Some(cycle_id.clone()),
        case_id: None,
        execution_id: Some(execution_id.clone()),
    };
    let bytes = b"native artifact upload".to_vec();
    let digest = Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    // Upload frames stay small; listings are paged to the larger server budget.
    let content = WireClient::new(stream).max_decoding_message_size(1024 * 1024).max_encoding_message_size(4096);
    let uploaded = content
        .upload::<contracts::content::UploadArtifact>(authenticated(tokio_stream::iter(vec![
            model::content::wire::UploadArtifactFrame {
                payload: Some(model::content::wire::UploadArtifactFramePayload::Header(model::content::wire::UploadArtifactHeader {
                    scope: Some(scope()),
                    filename: "native.bin".into(),
                    content_type: Some("application/octet-stream".into()),
                    size_bytes: bytes.len() as u64,
                    sha256: Some(digest.clone()),
                })),
            },
            model::content::wire::UploadArtifactFrame {
                payload: Some(model::content::wire::UploadArtifactFramePayload::Chunk(model::content::wire::ArtifactChunk {
                    offset: 0,
                    data: bytes.clone(),
                })),
            },
        ])))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(uploaded.filename, "native.bin");
    assert_eq!(uploaded.state, model::content::wire::ArtifactState::Complete);
    assert_eq!(uploaded.sha256, digest);

    // Operator Uploads Land Under Their Folder; Escaping Names Are Refused
    let upload_named = |filename: &str| {
        tokio_stream::iter(vec![
            model::content::wire::UploadArtifactFrame {
                payload: Some(model::content::wire::UploadArtifactFramePayload::Header(model::content::wire::UploadArtifactHeader {
                    scope: Some(scope()),
                    filename: filename.into(),
                    content_type: Some("text/plain".into()),
                    size_bytes: 5,
                    sha256: None,
                })),
            },
            model::content::wire::UploadArtifactFrame {
                payload: Some(model::content::wire::UploadArtifactFramePayload::Chunk(model::content::wire::ArtifactChunk {
                    offset: 0,
                    data: b"notes".to_vec(),
                })),
            },
        ])
    };
    let foldered = content
        .upload::<contracts::content::UploadArtifact>(authenticated(upload_named("upload/operator-notes.txt")))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(foldered.filename, "upload/operator-notes.txt");
    assert_eq!(foldered.state, model::content::wire::ArtifactState::Complete);
    for escaping in ["../escape.txt", "/absolute.txt", "upload//empty.txt"] {
        let refused = content.upload::<contracts::content::UploadArtifact>(authenticated(upload_named(escaping))).await.unwrap_err();
        assert_eq!(refused.code(), Code::InvalidArgument, "{escaping}");
    }

    let native =
        dokime::client::Client::connect(dokime::client::Target::local(server.local_endpoint()).unwrap(), Some(TOKEN), None).await.unwrap();
    let source = root.path().join("native-upload.bin");
    std::fs::write(&source, &bytes).unwrap();
    let uploaded = native
        .upload_artifact_file(
            &source,
            model::content::wire::UploadArtifactHeader {
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
    assert_eq!(uploaded.state, model::content::wire::ArtifactState::Complete);
    native.close_and_wait().await;

    let error = content
        .upload::<contracts::content::UploadArtifact>(authenticated(tokio_stream::iter(vec![
            model::content::wire::UploadArtifactFrame {
                payload: Some(model::content::wire::UploadArtifactFramePayload::Header(model::content::wire::UploadArtifactHeader {
                    scope: Some(scope()),
                    filename: "bad-offset.bin".into(),
                    content_type: None,
                    size_bytes: 3,
                    sha256: None,
                })),
            },
            model::content::wire::UploadArtifactFrame {
                payload: Some(model::content::wire::UploadArtifactFramePayload::Chunk(model::content::wire::ArtifactChunk {
                    offset: 1,
                    data: b"bad".to_vec(),
                })),
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
        dokime::artifacts::ArtifactStatus::Cancelled
    );

    let error = content
        .upload::<contracts::content::UploadArtifact>(authenticated(tokio_stream::iter(vec![
            model::content::wire::UploadArtifactFrame {
                payload: Some(model::content::wire::UploadArtifactFramePayload::Header(model::content::wire::UploadArtifactHeader {
                    scope: Some(scope()),
                    filename: "bad-size.bin".into(),
                    content_type: None,
                    size_bytes: 4,
                    sha256: None,
                })),
            },
            model::content::wire::UploadArtifactFrame {
                payload: Some(model::content::wire::UploadArtifactFramePayload::Chunk(model::content::wire::ArtifactChunk {
                    offset: 0,
                    data: b"bad".to_vec(),
                })),
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
        dokime::artifacts::ArtifactStatus::Rejected
    );

    let error = content
        .upload::<contracts::content::UploadArtifact>(authenticated(tokio_stream::iter(vec![
            model::content::wire::UploadArtifactFrame {
                payload: Some(model::content::wire::UploadArtifactFramePayload::Header(model::content::wire::UploadArtifactHeader {
                    scope: Some(scope()),
                    filename: "bad-digest.bin".into(),
                    content_type: None,
                    size_bytes: 3,
                    sha256: Some("0".repeat(64)),
                })),
            },
            model::content::wire::UploadArtifactFrame {
                payload: Some(model::content::wire::UploadArtifactFramePayload::Chunk(model::content::wire::ArtifactChunk {
                    offset: 0,
                    data: b"bad".to_vec(),
                })),
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
        dokime::artifacts::ArtifactStatus::Rejected
    );

    let listed = content
        .unary::<contracts::content::ListArtifacts>(authenticated(contracts::content::ListArtifacts::default()))
        .await
        .unwrap()
        .into_inner()
        .artifacts;
    for (filename, state) in [
        ("native.bin", model::content::wire::ArtifactState::Complete),
        ("bad-offset.bin", model::content::wire::ArtifactState::Cancelled),
        ("bad-size.bin", model::content::wire::ArtifactState::Rejected),
        ("bad-digest.bin", model::content::wire::ArtifactState::Rejected),
    ] {
        assert_eq!(
            listed.iter().find(|artifact| artifact.filename == filename).map(|artifact| artifact.state),
            Some(state),
            "list must preserve {filename} state",
        );
    }

    gate.release.notify_one();
    support::on_state_change(&owner.handle(), async || {
        let cycle = execution
            .unary::<contracts::execution::GetCycle>(authenticated(contracts::execution::GetCycle { cycle_id: cycle_id.clone() }))
            .await
            .unwrap()
            .into_inner();
        if support::cycle_settled(&cycle, model::execution::wire::ResultStatus::Passed) {
            return Some(());
        }
        None
    })
    .await;
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

async fn completion_harness()
-> (tempfile::TempDir, Runtime, Server, WireClient, WireClient, Arc<UploadGate>, String, model::content::wire::ArtifactScope) {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(UploadGate::default());
    let prepared_owner = dokime_server::grpc::Service::prepare(keyed().with_max_message_bytes(4096)).unwrap();
    let owner = Runtime::with_execution_fixture(
        prepared_owner
            .configuration(crate::support::budgets::runtime_config(root.path(), support::metadata("grpc.content.completion", "Content"))),
        "fixture",
        gate.clone(),
    )
    .boxed_local()
    .await
    .unwrap();
    support::register_fixture_execution(&owner.handle(), support::fixture_suite(&["passed"])).await;
    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
    let stream = Endpoint::from_shared(server.local_endpoint()).unwrap().connect().await.unwrap();
    let execution = WireClient::new(stream.clone());
    let cycle_id = support::planned_cycle(
        &execution,
        authenticated(contracts::execution::CreateCycle {
            metadata: Some(model::identity::wire::Metadata {
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
    execution.start_first(authenticated(contracts::execution::GetCycle { cycle_id: cycle_id.clone() })).await.unwrap();
    gate.started.notified().await;
    let execution_id = execution
        .unary::<contracts::execution::ListExecutions>(authenticated(contracts::execution::ListExecutions {
            cycle_id: cycle_id.clone(),
            limit: 100,
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner()
        .executions
        .remove(0)
        .execution_id;
    let scope = model::content::wire::ArtifactScope {
        cycle_id: Some(cycle_id.clone()),
        case_id: None,
        execution_id: Some(execution_id),
    };
    let content = WireClient::new(stream).max_decoding_message_size(4096).max_encoding_message_size(4096);
    (root, owner, server, execution, content, gate, cycle_id, scope)
}

async fn wait_for_watched_lifecycle(
    owner: &Runtime, cycle: &dokime::TestCycleId, lifecycle: dokime::domain::model::execution::result::CycleStatus,
) {
    owner
        .handle()
        .watch_state()
        .wait_until(|snapshot| snapshot.state().cycles.get(cycle).map(|cycle| cycle.lifecycle) == Some(lifecycle))
        .await
        .expect("the runtime state watch stays open");
}

async fn finish_completion_harness(execution: WireClient, cycle_id: String, gate: Arc<UploadGate>, mut owner: Runtime, server: Server) {
    gate.release.notify_one();
    support::on_state_change(&owner.handle(), async || {
        let cycle = execution
            .unary::<contracts::execution::GetCycle>(authenticated(contracts::execution::GetCycle { cycle_id: cycle_id.clone() }))
            .await
            .unwrap()
            .into_inner();
        if support::cycle_settled(&cycle, model::execution::wire::ResultStatus::Passed) {
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
    let (_root, owner, server, execution, content, gate, cycle_id, scope) = completion_harness().await;
    let bytes = b"trailing completion digest upload".to_vec();
    let digest = Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let uploaded = content
        .upload::<contracts::content::UploadArtifact>(authenticated(tokio_stream::iter(vec![
            model::content::wire::UploadArtifactFrame {
                payload: Some(model::content::wire::UploadArtifactFramePayload::Header(model::content::wire::UploadArtifactHeader {
                    scope: Some(scope.clone()),
                    filename: "trailing.bin".into(),
                    content_type: Some("application/octet-stream".into()),
                    size_bytes: bytes.len() as u64,
                    sha256: None,
                })),
            },
            model::content::wire::UploadArtifactFrame {
                payload: Some(model::content::wire::UploadArtifactFramePayload::Chunk(model::content::wire::ArtifactChunk {
                    offset: 0,
                    data: bytes.clone(),
                })),
            },
            model::content::wire::UploadArtifactFrame {
                payload: Some(model::content::wire::UploadArtifactFramePayload::Complete(model::content::wire::ArtifactCompletion {
                    size_bytes: bytes.len() as u64,
                    sha256: digest.clone(),
                })),
            },
        ])))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(uploaded.filename, "trailing.bin");
    assert_eq!(uploaded.state, model::content::wire::ArtifactState::Complete);
    assert_eq!(uploaded.sha256, digest);
    finish_completion_harness(execution, cycle_id, gate, owner, server).await;
}

#[tokio::test]
async fn artifact_upload_match() {
    let (_root, owner, server, execution, content, gate, cycle_id, scope) = completion_harness().await;
    let bytes = b"mismatched trailing digest upload".to_vec();
    let wrong_digest = Sha256::digest(b"not the uploaded bytes").iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let error = content
        .upload::<contracts::content::UploadArtifact>(authenticated(tokio_stream::iter(vec![
            model::content::wire::UploadArtifactFrame {
                payload: Some(model::content::wire::UploadArtifactFramePayload::Header(model::content::wire::UploadArtifactHeader {
                    scope: Some(scope.clone()),
                    filename: "mismatched.bin".into(),
                    content_type: None,
                    size_bytes: bytes.len() as u64,
                    sha256: None,
                })),
            },
            model::content::wire::UploadArtifactFrame {
                payload: Some(model::content::wire::UploadArtifactFramePayload::Chunk(model::content::wire::ArtifactChunk {
                    offset: 0,
                    data: bytes.clone(),
                })),
            },
            model::content::wire::UploadArtifactFrame {
                payload: Some(model::content::wire::UploadArtifactFramePayload::Complete(model::content::wire::ArtifactCompletion {
                    size_bytes: bytes.len() as u64,
                    sha256: wrong_digest,
                })),
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
        dokime::artifacts::ArtifactStatus::Rejected
    );
    finish_completion_harness(execution, cycle_id, gate, owner, server).await;
}

#[tokio::test]
async fn artifact_upload_completion() {
    let (_root, owner, server, execution, content, gate, cycle_id, scope) = completion_harness().await;
    let bytes = b"frame after completion".to_vec();
    let digest = Sha256::digest(&bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let error = content
        .upload::<contracts::content::UploadArtifact>(authenticated(tokio_stream::iter(vec![
            model::content::wire::UploadArtifactFrame {
                payload: Some(model::content::wire::UploadArtifactFramePayload::Header(model::content::wire::UploadArtifactHeader {
                    scope: Some(scope.clone()),
                    filename: "trailing-extra.bin".into(),
                    content_type: None,
                    size_bytes: bytes.len() as u64,
                    sha256: None,
                })),
            },
            model::content::wire::UploadArtifactFrame {
                payload: Some(model::content::wire::UploadArtifactFramePayload::Chunk(model::content::wire::ArtifactChunk {
                    offset: 0,
                    data: bytes.clone(),
                })),
            },
            model::content::wire::UploadArtifactFrame {
                payload: Some(model::content::wire::UploadArtifactFramePayload::Complete(model::content::wire::ArtifactCompletion {
                    size_bytes: bytes.len() as u64,
                    sha256: digest,
                })),
            },
            model::content::wire::UploadArtifactFrame {
                payload: Some(model::content::wire::UploadArtifactFramePayload::Chunk(model::content::wire::ArtifactChunk {
                    offset: bytes.len() as u64,
                    data: b"x".to_vec(),
                })),
            },
        ])))
        .await
        .unwrap_err();
    assert_eq!(error.code(), Code::InvalidArgument);
    finish_completion_harness(execution, cycle_id, gate, owner, server).await;
}

#[tokio::test]
async fn open_report_states() {
    let (_root, mut owner, server, execution, _content, gate, cycle_id, scope) = completion_harness().await;
    let cycle = dokime::TestCycleId::from_uuid(cycle_id.clone()).unwrap();
    let execution_id = scope.execution_id.clone().expect("captured execution scope");
    wait_for_watched_lifecycle(&owner, &cycle, dokime::domain::model::execution::result::CycleStatus::Running).await;

    // Running: Both Reports Are Rendered From The Current Evidence
    let (metadata, html) = cycle_report(&execution, &cycle_id).await;
    assert_eq!(metadata.origin, model::reports::wire::ReportOrigin::Current);
    assert_eq!(metadata.test_execution_id, None);
    assert!(String::from_utf8_lossy(&html).contains("<!doctype html>"));
    let (metadata, html) = execution_report(&execution, &execution_id).await;
    assert_eq!(metadata.origin, model::reports::wire::ReportOrigin::Current);
    assert_eq!(metadata.test_cycle_id, cycle_id);
    assert_eq!(metadata.test_execution_id.as_deref(), Some(execution_id.as_str()));
    assert!(String::from_utf8_lossy(&html).contains("<!doctype html>"));

    // Completed Executions Leave Their Cycle Active Until Explicitly Sealed.
    gate.release.notify_one();
    support::wait_for_cycle_outcome(&owner.handle(), &cycle, dokime::domain::model::execution::result::ResultStatus::Passed).await;
    let (metadata, _) = execution_report(&execution, &execution_id).await;
    assert_eq!(metadata.origin, model::reports::wire::ReportOrigin::Sealed);
    let (metadata, _) = cycle_report(&execution, &cycle_id).await;
    assert_eq!(metadata.origin, model::reports::wire::ReportOrigin::Current);
    execution
        .unary::<contracts::execution::SealCycle>(authenticated(contracts::execution::SealCycle {
            cycle_id: cycle_id.clone(),
            context: None,
        }))
        .await
        .unwrap();

    // Explicit Seal Persists Both Reports Unchanged Across Reads.
    let (metadata, first) = cycle_report(&execution, &cycle_id).await;
    assert_eq!(metadata.origin, model::reports::wire::ReportOrigin::Sealed);
    let (_, second) = cycle_report(&execution, &cycle_id).await;
    assert_eq!(first, second, "a sealed cycle report must read back unchanged");
    let (metadata, _) = execution_report(&execution, &execution_id).await;
    assert_eq!(metadata.origin, model::reports::wire::ReportOrigin::Sealed);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

async fn cycle_report(client: &WireClient, cycle_id: &str) -> (model::reports::wire::ReportMetadata, Vec<u8>) {
    let stream = client
        .download::<contracts::reports::DownloadCycleReport>(authenticated(contracts::reports::DownloadCycleReport {
            cycle_id: cycle_id.to_owned(),
            identifier: String::new(),
        }))
        .await
        .unwrap()
        .into_inner();
    report_frames(stream).await
}

async fn execution_report(client: &WireClient, execution_id: &str) -> (model::reports::wire::ReportMetadata, Vec<u8>) {
    let stream = client
        .download::<contracts::reports::DownloadExecutionReport>(authenticated(contracts::reports::DownloadExecutionReport {
            execution_id: execution_id.to_owned(),
        }))
        .await
        .unwrap()
        .into_inner();
    report_frames(stream).await
}

async fn report_frames(mut stream: WireStream<model::reports::wire::ReportHtmlFrame>) -> (model::reports::wire::ReportMetadata, Vec<u8>) {
    let mut metadata = None;
    let mut html = Vec::new();
    let mut complete = false;
    while let Some(frame) = stream.message().await.unwrap() {
        match frame.payload.expect("report payload") {
            model::reports::wire::ReportHtmlFramePayload::Report(report) => {
                assert!(metadata.replace(*report).is_none(), "one metadata frame");
            }
            model::reports::wire::ReportHtmlFramePayload::Chunk(chunk) => {
                assert_eq!(chunk.offset, html.len() as u64);
                html.extend_from_slice(&chunk.data);
            }
            model::reports::wire::ReportHtmlFramePayload::Complete(done) => {
                assert_eq!(done.size_bytes, html.len() as u64);
                complete = true;
            }
        }
    }
    assert!(complete, "report stream must complete");
    (metadata.expect("report metadata"), html)
}

/* === Tests === */
