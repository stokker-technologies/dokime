/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Operator media operations against a running execution. Live captures run
// when the host has the device; otherwise the refusal itself is asserted.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports

// Local Imports
use super::content::UploadGate;
use super::*;

/* === Constants === */
// Bounds one live capture, including a desktop portal's consent round trip.
const CAPTURE_DEADLINE: Duration = Duration::from_secs(60);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) async fn media_capture_artifacts() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(UploadGate::default());
    let mut owner = Runtime::with_execution_fixture(
        RuntimeConfig::new(root.path(), common::metadata("grpc.media", "Media")),
        "fixture",
        gate.clone(),
    )
    .await
    .unwrap();
    common::register_fixture_execution(&owner.handle(), common::fixture_suite(&["passed"])).await;
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Control)])
            .unwrap()
            .with_max_message_bytes(1024 * 1024),
    )
    .await
    .unwrap();
    let client = connect(&server).await;
    let scope = running_execution(&owner, &client, &gate).await;

    // Scope And Limits Are Refused Before Any Device Opens
    let unscoped = client
        .unary::<contracts::media::Screenshot>(authenticated(contracts::media::ScreenshotRequest {
            scope: Some(contracts::content::ArtifactScope {
                execution_id: None,
                ..scope.clone()
            }),
            display: None,
        }))
        .await
        .unwrap_err();
    assert_eq!(unscoped.code(), Code::InvalidArgument);
    let unbounded = recording(
        &client,
        contracts::media::AudioRecordingRequest {
            scope: Some(scope.clone()),
            input: None,
            maximum_seconds: Some(0),
        },
    )
    .await
    .unwrap_err();
    assert_eq!(unbounded.code(), Code::InvalidArgument);

    // Stills Land Under Their Media Folders
    let screenshot = tokio::time::timeout(
        CAPTURE_DEADLINE,
        client.unary::<contracts::media::Screenshot>(authenticated(
            contracts::media::ScreenshotRequest {
                scope: Some(scope.clone()),
                display: None,
            },
        )),
    )
    .await
    .expect("screenshot settles within its deadline");
    captured(
        screenshot.map(|response| response.into_inner()),
        "screenshots/",
        ".png",
        "image/png",
        "display",
    );
    let photo = tokio::time::timeout(
        CAPTURE_DEADLINE,
        client.unary::<contracts::media::Photo>(authenticated(contracts::media::PhotoRequest {
            scope: Some(scope.clone()),
            camera: None,
        })),
    )
    .await
    .expect("photo settles within its deadline");
    captured(
        photo.map(|response| response.into_inner()),
        "photos/",
        ".jpg",
        "image/jpeg",
        "camera",
    );

    // A Bounded Recording Reports Progress, Then Its Sealed Artifact
    let bounded = tokio::time::timeout(
        CAPTURE_DEADLINE,
        recording(
            &client,
            contracts::media::AudioRecordingRequest {
                scope: Some(scope.clone()),
                input: None,
                maximum_seconds: Some(1),
            },
        ),
    )
    .await
    .expect("bounded recording settles within its deadline");
    let live_audio = match bounded {
        Ok(mut frames) => {
            let mut progress = 0;
            let completed = loop {
                match frames.message().await.unwrap().expect("a completed frame") {
                    contracts::media::RecordingFrame {
                        payload: Some(contracts::media::RecordingFramePayload::Progress(_)),
                    } => progress += 1,
                    contracts::media::RecordingFrame {
                        payload: Some(contracts::media::RecordingFramePayload::Completed(record)),
                    } => break *record,
                    other => panic!("unexpected recording frame {other:?}"),
                }
            };
            assert!(progress >= 1, "a recording reports progress before sealing");
            assert!(frames.message().await.unwrap().is_none());
            captured(Ok(completed), "audio/", ".wav", "audio/wav", "audio");
            true
        }
        Err(status) => {
            refused(&status, "audio");
            false
        }
    };

    // A Cancelled Recording Is Still Sealed
    if live_audio {
        let mut frames = recording(
            &client,
            contracts::media::AudioRecordingRequest {
                scope: Some(scope.clone()),
                input: None,
                maximum_seconds: Some(60),
            },
        )
        .await
        .unwrap();
        frames.message().await.unwrap().expect("a progress frame");
        drop(frames);
        let sealed = tokio::time::timeout(Duration::from_secs(10), async {
            loop {
                let audio = owner
                    .handle()
                    .artifacts()
                    .await
                    .expect("artifact query succeeds")
                    .iter()
                    .filter(|record| record.filename.starts_with("audio/"))
                    .filter(|record| {
                        record.status == dokime_app::artifacts::ArtifactStatus::Complete
                    })
                    .count();
                if audio == 2 {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        })
        .await;
        assert!(sealed.is_ok(), "a cancelled recording seals its artifact");
    }

    // An Execution Sealing Mid-Recording Seals The Recording Into It
    if live_audio {
        let mut frames = recording(
            &client,
            contracts::media::AudioRecordingRequest {
                scope: Some(scope.clone()),
                input: None,
                maximum_seconds: Some(60),
            },
        )
        .await
        .unwrap();
        frames.message().await.unwrap().expect("a progress frame");
        gate.release.notify_one();
        let completed = tokio::time::timeout(Duration::from_secs(30), async {
            loop {
                match frames.message().await.unwrap().expect("a completed frame") {
                    contracts::media::RecordingFrame {
                        payload: Some(contracts::media::RecordingFramePayload::Completed(record)),
                    } => break *record,
                    _ => continue,
                }
            }
        })
        .await
        .expect("the execution seals its running recording");
        assert_eq!(
            completed
                .scope
                .as_ref()
                .and_then(|scope| scope.execution_id.clone()),
            scope.execution_id
        );
        captured(Ok(completed), "audio/", ".wav", "audio/wav", "audio");
    } else {
        gate.release.notify_one();
    }

    // Video Recording Is Declared But Refused Until An Encoder Is Chosen
    let screen = recording_refusal::<contracts::media::ScreenRecording>(
        &client,
        contracts::media::ScreenRecordingRequest {
            scope: Some(scope.clone()),
            display: None,
            audio: Some(contracts::media::RecordingAudio {
                microphone: true,
                system: true,
            }),
            maximum_seconds: None,
        },
    )
    .await;
    let camera = recording_refusal::<contracts::media::CameraRecording>(
        &client,
        contracts::media::CameraRecordingRequest {
            scope: Some(scope),
            camera: None,
            audio: None,
            maximum_seconds: None,
        },
    )
    .await;
    for status in [screen, camera] {
        assert_eq!(status.code(), Code::FailedPrecondition);
        assert_eq!(status.message(), "no video encoder available in this build");
    }

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Private Functions === */
async fn running_execution(
    owner: &Runtime,
    client: &WireClient,
    gate: &UploadGate,
) -> contracts::content::ArtifactScope {
    let cycle_id = client
        .unary::<contracts::execution::CreateCycle>(authenticated(
            contracts::execution::CreateCycleRequest {
                plan: Some(super::workflow::selection_plan(owner)),
                metadata: Some(contracts::common::Metadata {
                    identifier: "media.capture".into(),
                    label: "Media".into(),
                    description: None,
                    ..Default::default()
                }),
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    client
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
    let execution_id = client
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
    contracts::content::ArtifactScope {
        cycle_id: Some(cycle_id),
        case_id: None,
        execution_id: Some(execution_id),
    }
}

async fn recording(
    client: &WireClient,
    request: contracts::media::AudioRecordingRequest,
) -> Result<WireStream<contracts::media::RecordingFrame>, tonic::Status> {
    let mut frames = client
        .download::<contracts::media::AudioRecording>(authenticated(request))
        .await?
        .into_inner();

    // A Refusal Before Recording Arrives As The First Frame
    use futures_util::StreamExt;
    let first = frames.message().await?;
    let frames = futures_util::stream::iter(first.map(Ok)).chain(frames.frames);
    Ok(WireStream {
        frames: Box::pin(frames),
    })
}

async fn recording_refusal<O>(client: &WireClient, request: O::Request) -> tonic::Status
where
    O: contracts::Endpoint
        + harmos::operations::OutputDeclaration<Item = contracts::media::RecordingFrame>,
    O::Request: harmos::Protobuf + Sync,
    O::Response: harmos::Protobuf + harmos::MemorySize,
{
    match client.download::<O>(authenticated(request)).await {
        Err(status) => status,
        Ok(response) => response
            .into_inner()
            .message()
            .await
            .expect_err("video recording is refused"),
    }
}

// Asserts a capture landed, or that the host's missing device was refused.
fn captured(
    result: Result<contracts::content::ArtifactMetadata, tonic::Status>,
    folder: &str,
    extension: &str,
    content_type: &str,
    device: &str,
) {
    let record = match result {
        Ok(record) => record,
        Err(status) => return refused(&status, device),
    };
    assert!(record.filename.starts_with(folder), "{}", record.filename);
    assert!(record.filename.ends_with(extension), "{}", record.filename);
    assert_eq!(
        record.filename.matches('/').count(),
        1,
        "{}",
        record.filename
    );
    assert_eq!(record.content_type.as_deref(), Some(content_type));
    assert_eq!(record.state, contracts::content::ArtifactState::Complete);
    assert!(record.size_bytes > 0);
}

// A host without the device refuses with FailedPrecondition naming it; the
// skip is reported, never counted as a capture.
fn refused(status: &tonic::Status, device: &str) {
    assert_eq!(status.code(), Code::FailedPrecondition, "{status:?}");
    if cfg!(feature = "media-native") {
        assert!(status.message().contains(device), "{status:?}");
    } else {
        assert_eq!(status.message(), "capture not available in this build");
    }
    eprintln!("Skipped live {device} capture: {}", status.message());
}

/* === Tests === */
