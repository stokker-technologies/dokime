/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Operator media operations: host captures sealed as execution artifacts.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{sync::Arc, time::Duration};

// Third Party Imports
use dokime::harmos::operations::{Context, Operation, Response, StreamOutput};
use tonic::Status;

// Local Imports
use super::content::{artifact_scope, content_failure, metadata};
use crate::grpc::transcode::Transcode;
use dokime::RuntimeHandle;
use dokime::artifacts::ArtifactRecord;
use dokime::domain::model::content::wire::ArtifactMetadata;
use dokime::domain::model::diagnostics::admission::AdmissionFailure;
use dokime::domain::model::execution::RecordScope;
use dokime::domain::operations::media as contract;
use dokime::domains::media::AudioRecording as Recording;
use dokime::domains::media::NOT_COMPILED;
use dokime::domains::media::UNAVAILABLE;
use dokime::runtime::Error;

/* === Constants === */

const SERVER_MEDIA_DEFAULT_RECORDING_MS: u64 = 600_000;
const SERVER_MEDIA_PROGRESS_MS: u64 = 1_000;
const SERVER_MEDIA_RECORDING_MS: u64 = 3_600_000;

// Video capture is declared but refused until an encoder is chosen: no ffmpeg,
// GStreamer or platform encoder is linked yet. The desktop's MediaRecorder sink
// remains the video path until that decision lands.
const NO_VIDEO_ENCODER: &str = "no video encoder available in this build";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Resources {
    runtime: RuntimeHandle,
    maximum: usize,
}

struct Screenshot {
    resources: Arc<Resources>,
}

struct Photo {
    resources: Arc<Resources>,
}

struct AudioRecording {
    resources: Arc<Resources>,
    recording: Option<Recording>,
}

struct ScreenRecording;

struct CameraRecording;

/* === Enums === */

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum Operations {
    Screenshot(Screenshot),
    Photo(Photo),
    AudioRecording(AudioRecording),
    ScreenRecording(ScreenRecording),
    CameraRecording(CameraRecording),
}

/* === Traits === */

/* === Implementations === */

impl Operation for Screenshot {
    type Declaration = contract::Screenshot;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }

    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut contract::Screenshot, _: &mut (),
    ) -> Result<Response<ArtifactMetadata>, Status> {
        let scope = capture_scope(request.scope.as_ref())?;
        let record = self.resources.runtime.capture_screenshot(scope, request.display).await.map_err(failure)?;
        self.resources.respond(&record)
    }
}

impl Operation for Photo {
    type Declaration = contract::Photo;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources })
    }

    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut contract::Photo, _: &mut (),
    ) -> Result<Response<ArtifactMetadata>, Status> {
        let scope = capture_scope(request.scope.as_ref())?;
        let record = self.resources.runtime.capture_photo(scope, request.camera).await.map_err(failure)?;
        self.resources.respond(&record)
    }
}

impl Operation for AudioRecording {
    type Declaration = contract::AudioRecording;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<dokime::domain::model::media::wire::RecordingFrame>;

    async fn initialize<'ctx>(resources: Arc<Resources>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self { resources, recording: None })
    }

    async fn run<'a>(
        &mut self, call_context: Self::Context<'a>, request: &mut contract::AudioRecording, output: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        let revoked = crate::grpc::access::revocation(&call_context);
        crate::grpc::access::during(revoked, async {
            let limit = recording_limit(request.maximum_seconds)?;
            let scope = capture_scope(request.scope.as_ref())?;
            let recording = self.resources.runtime.start_audio_recording(scope, request.input).await.map_err(failure)?;
            let recording = self.recording.insert(recording);

            // Report Progress Until The Limit Or Until The Execution Seals It;
            // Cancellation Seals Through `stop`
            let sealed = loop {
                let elapsed = recording.elapsed();
                if elapsed >= limit {
                    break recording.finish().await;
                }
                let progress = dokime::domain::model::media::wire::RecordingProgress {
                    elapsed_ms: elapsed.as_millis().try_into().unwrap_or(u64::MAX),
                    size_bytes: recording.size_bytes(),
                };
                output
                    .send_owned(
                        (dokime::domain::model::media::wire::RecordingFrame {
                            payload: Some(dokime::domain::model::media::wire::RecordingFramePayload::Progress(progress)),
                        })
                        .into(),
                    )
                    .await
                    .map_err(stream_failure)?;
                tokio::select! {
                    sealed = recording.settled() => break sealed,
                    () = tokio::time::sleep(Duration::from_millis(SERVER_MEDIA_PROGRESS_MS).min(limit - elapsed)) => {}
                }
            };

            // Announce The Sealed Artifact
            self.recording = None;
            let record = sealed.map_err(failure)?;
            let completed = metadata(&record)?;
            output
                .send_owned(
                    (dokime::domain::model::media::wire::RecordingFrame {
                        payload: Some(dokime::domain::model::media::wire::RecordingFramePayload::Completed(Box::new(completed))),
                    })
                    .into(),
                )
                .await
                .map_err(stream_failure)?;
            Ok(().into())
        })
        .await
    }

    // A cancelled or failed stream still seals what was recorded.
    async fn stop(&mut self) -> Result<(), Status> {
        let Some(mut recording) = self.recording.take() else {
            return Ok(());
        };
        let filename = recording.filename().to_owned();
        let record = recording.finish().await.map_err(failure)?;
        tracing::info!(artifact = %record.artifact_id, %filename, "cancelled audio recording sealed");
        Ok(())
    }
}

impl Operation for ScreenRecording {
    type Declaration = contract::ScreenRecording;
    type Resources = ();
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<dokime::domain::model::media::wire::RecordingFrame>;

    async fn initialize<'ctx>(_: Arc<()>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self)
    }

    async fn run<'a>(
        &mut self, call_context: Self::Context<'a>, _: &mut contract::ScreenRecording, _: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        let revoked = crate::grpc::access::revocation(&call_context);
        crate::grpc::access::during(revoked, async { Err(Status::failed_precondition(NO_VIDEO_ENCODER)) }).await
    }
}

impl Operation for CameraRecording {
    type Declaration = contract::CameraRecording;
    type Resources = ();
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = StreamOutput<dokime::domain::model::media::wire::RecordingFrame>;

    async fn initialize<'ctx>(_: Arc<()>, _: Self::Context<'ctx>) -> Result<Self, Status> {
        Ok(Self)
    }

    async fn run<'a>(
        &mut self, call_context: Self::Context<'a>, _: &mut contract::CameraRecording, _: &mut Self::Endpoints,
    ) -> Result<Response<()>, Status> {
        let revoked = crate::grpc::access::revocation(&call_context);
        crate::grpc::access::during(revoked, async { Err(Status::failed_precondition(NO_VIDEO_ENCODER)) }).await
    }
}

impl Resources {
    fn respond(&self, record: &ArtifactRecord) -> Result<Response<ArtifactMetadata>, Status> {
        let output = metadata(record)?;
        if output.wire_len()? > self.maximum {
            return Err(Status::resource_exhausted("artifact metadata exceeds response budget"));
        }
        Ok(output.into())
    }
}

/* === Public Functions === */

pub(super) fn register(runtime: &RuntimeHandle, maximum: usize) -> Result<(), crate::grpc::Error> {
    let resources = Arc::new(Resources { runtime: runtime.clone(), maximum });
    let operations = runtime.operations();
    let lane = dokime::app::Lanes::Open;
    operations.register_wire_operation::<Screenshot>("dokime", lane, resources.clone())?;
    operations.register_wire_operation::<Photo>("dokime", lane, resources.clone())?;
    operations.register_wire_operation::<AudioRecording>("dokime", lane, resources)?;
    operations.register_wire_operation::<ScreenRecording>("dokime", lane, Arc::new(()))?;
    operations.register_wire_operation::<CameraRecording>("dokime", lane, Arc::new(()))?;
    Ok(())
}

/* === Private Functions === */

fn capture_scope(scope: Option<&dokime::domain::model::content::wire::ArtifactScope>) -> Result<RecordScope, Status> {
    let scope = artifact_scope(scope)?;
    if scope.test_cycle_id.is_none() || scope.test_execution_id.is_none() {
        return Err(Status::invalid_argument("media capture requires a cycle and execution scope"));
    }
    Ok(scope)
}

fn recording_limit(seconds: Option<u32>) -> Result<Duration, Status> {
    let seconds = seconds.map_or(SERVER_MEDIA_DEFAULT_RECORDING_MS / 1000, u64::from);
    if seconds == 0 || seconds.saturating_mul(1000) > SERVER_MEDIA_RECORDING_MS {
        return Err(Status::invalid_argument("recording maximum exceeds configured bounds"));
    }
    Ok(Duration::from_secs(seconds))
}

fn failure(error: Error) -> Status {
    match &error {
        Error::Admission(AdmissionFailure::Unavailable { code, message }) if code == UNAVAILABLE || code == NOT_COMPILED => {
            Status::failed_precondition(message.clone())
        }
        Error::Admission(AdmissionFailure::Conflict { message, .. }) => Status::failed_precondition(message.clone()),
        _ => content_failure(error),
    }
}

fn stream_failure(error: dokime::harmos::Error) -> Status {
    match error {
        dokime::harmos::Error::ResourceExhausted { .. } => Status::resource_exhausted(error.to_string()),
        dokime::harmos::Error::NotReady { .. } => Status::unavailable(error.to_string()),
        error => Status::internal(error.to_string()),
    }
}

/* === Tests === */
