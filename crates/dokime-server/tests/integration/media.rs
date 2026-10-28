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

use std::sync::Arc;

use super::content::UploadGate;
use super::*;
use dokime::domain::model;

#[tokio::test]
async fn media_capture_artifacts() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(UploadGate::default());
    let mut owner = Runtime::with_execution_fixture(
        crate::support::budgets::runtime_config(root.path(), support::metadata("grpc.media", "Media")),
        "fixture",
        gate.clone(),
    )
    .await
    .unwrap();
    support::register_fixture_execution(&owner.handle(), support::fixture_suite(&["passed"])).await;
    let server = Server::start(owner.handle(), Config::local().with_max_message_bytes(1024 * 1024)).await.unwrap();
    let client = connect(&server).await;
    let scope = running_execution(&owner, &client, &gate).await;

    // Scope And Limits Are Refused Before Any Device Opens
    let unscoped = client
        .unary::<contracts::media::Screenshot>(authenticated(contracts::media::Screenshot {
            scope: Some(model::content::wire::ArtifactScope { execution_id: None, ..scope.clone() }),
            display: None,
        }))
        .await
        .unwrap_err();
    assert_eq!(unscoped.code(), Code::InvalidArgument);
    let unbounded = recording(
        &client,
        contracts::media::AudioRecording {
            scope: Some(scope.clone()),
            input: None,
            maximum_seconds: Some(0),
        },
    )
    .await
    .unwrap_err();
    assert_eq!(unbounded.code(), Code::InvalidArgument);

    // Stills Land Under Their Media Folders
    let screenshot = client
        .unary::<contracts::media::Screenshot>(authenticated(contracts::media::Screenshot {
            scope: Some(scope.clone()),
            display: None,
        }))
        .await;
    captured(screenshot.map(|response| response.into_inner()), "screenshots/", ".png", "image/png", "display");
    let photo =
        client.unary::<contracts::media::Photo>(authenticated(contracts::media::Photo { scope: Some(scope.clone()), camera: None })).await;
    captured(photo.map(|response| response.into_inner()), "photos/", ".jpg", "image/jpeg", "camera");

    // A Bounded Recording Reports Progress, Then Its Sealed Artifact
    let bounded = recording(
        &client,
        contracts::media::AudioRecording {
            scope: Some(scope.clone()),
            input: None,
            maximum_seconds: Some(1),
        },
    )
    .await;
    let live_audio = match bounded {
        Ok(mut frames) => {
            let mut progress = 0;
            let completed = loop {
                match frames.message().await.unwrap().expect("a completed frame") {
                    model::media::wire::RecordingFrame {
                        payload: Some(model::media::wire::RecordingFramePayload::Progress(_)),
                    } => progress += 1,
                    model::media::wire::RecordingFrame {
                        payload: Some(model::media::wire::RecordingFramePayload::Completed(record)),
                    } => {
                        break *record;
                    }
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
            contracts::media::AudioRecording {
                scope: Some(scope.clone()),
                input: None,
                maximum_seconds: Some(60),
            },
        )
        .await
        .unwrap();
        frames.message().await.unwrap().expect("a progress frame");
        drop(frames);
        // A Cancelled Recording Seals Its Artifact
        support::on_state_change(&owner.handle(), async || {
            let audio = owner
                .handle()
                .artifacts()
                .await
                .expect("artifact query succeeds")
                .iter()
                .filter(|record| record.filename.starts_with("audio/"))
                .filter(|record| record.status == dokime::artifacts::ArtifactStatus::Complete)
                .count();
            (audio == 2).then_some(())
        })
        .await;
    }

    // An Execution Sealing Mid-Recording Seals The Recording Into It
    if live_audio {
        let mut frames = recording(
            &client,
            contracts::media::AudioRecording {
                scope: Some(scope.clone()),
                input: None,
                maximum_seconds: Some(60),
            },
        )
        .await
        .unwrap();
        frames.message().await.unwrap().expect("a progress frame");
        gate.release.notify_one();
        let completed = {
            loop {
                match frames.message().await.unwrap().expect("a completed frame") {
                    model::media::wire::RecordingFrame {
                        payload: Some(model::media::wire::RecordingFramePayload::Completed(record)),
                    } => {
                        break *record;
                    }
                    _ => continue,
                }
            }
        };
        assert_eq!(completed.scope.as_ref().and_then(|scope| scope.execution_id.clone()), scope.execution_id);
        captured(Ok(completed), "audio/", ".wav", "audio/wav", "audio");
    } else {
        gate.release.notify_one();
    }

    // Video Recording Is Declared But Refused Until An Encoder Is Chosen
    let screen = recording_refusal::<contracts::media::ScreenRecording>(
        &client,
        contracts::media::ScreenRecording {
            scope: Some(scope.clone()),
            display: None,
            audio: Some(model::media::wire::RecordingAudio { microphone: true, system: true }),
            maximum_seconds: None,
        },
    )
    .await;
    let camera = recording_refusal::<contracts::media::CameraRecording>(
        &client,
        contracts::media::CameraRecording {
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

pub(super) async fn running_execution(owner: &Runtime, client: &WireClient, gate: &UploadGate) -> model::content::wire::ArtifactScope {
    let cycle_id = support::planned_cycle(
        client,
        authenticated(contracts::execution::CreateCycle {
            metadata: Some(model::identity::wire::Metadata {
                identifier: "media.capture".into(),
                label: "Media".into(),
                description: None,
                ..Default::default()
            }),
            ..Default::default()
        }),
        selection_plan(owner),
    )
    .await
    .unwrap();
    client.start_first(authenticated(contracts::execution::GetCycle { cycle_id: cycle_id.clone() })).await.unwrap();
    gate.started.notified().await;
    let execution_id = client
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
    model::content::wire::ArtifactScope {
        cycle_id: Some(cycle_id),
        case_id: None,
        execution_id: Some(execution_id),
    }
}

async fn recording(
    client: &WireClient, request: contracts::media::AudioRecording,
) -> Result<WireStream<model::media::wire::RecordingFrame>, tonic::Status> {
    let mut frames = client.download::<contracts::media::AudioRecording>(authenticated(request)).await?.into_inner();

    // A Refusal Before Recording Arrives As The First Frame
    use futures_util::StreamExt;
    let first = frames.message().await?;
    let frames = futures_util::stream::iter(first.map(Ok)).chain(frames.frames);
    Ok(WireStream { frames: Box::pin(frames) })
}

async fn recording_refusal<O>(client: &WireClient, request: O) -> tonic::Status
where
    O: contracts::Endpoint<Output = dokime::harmos::operations::Stream<model::media::wire::RecordingFrame>>,
    O: dokime::harmos::domain::Portable + Sync,
    O::Response: dokime::harmos::domain::Portable + dokime::harmos::MemorySize,
{
    match client.download::<O>(authenticated(request)).await {
        Err(status) => status,
        Ok(response) => response.into_inner().message().await.expect_err("video recording is refused"),
    }
}

// Asserts a capture landed, or that the host's missing device was refused.
fn captured(
    result: Result<model::content::wire::ArtifactMetadata, tonic::Status>, folder: &str, extension: &str, content_type: &str, device: &str,
) {
    let record = match result {
        Ok(record) => record,
        Err(status) => return refused(&status, device),
    };
    assert!(record.filename.starts_with(folder), "{}", record.filename);
    assert!(record.filename.ends_with(extension), "{}", record.filename);
    assert_eq!(record.filename.matches('/').count(), 1, "{}", record.filename);
    assert_eq!(record.content_type.as_deref(), Some(content_type));
    assert_eq!(record.state, model::content::wire::ArtifactState::Complete);
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
