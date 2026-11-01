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
use dokime::harmos::{Context, Response};
use tonic::Status;

// Local Imports
use super::content::{artifact_scope, content_failure, metadata};
use crate::grpc::transcode::Transcode;
use dokime::app::LiveRuntime;
use dokime::artifacts::ArtifactRecord;
use dokime::domain::model::content::wire::ArtifactMetadata;
use dokime::domain::model::diagnostics::admission::AdmissionFailure;
use dokime::domain::model::execution::RecordScope;
use dokime::domains::media::AudioRecording as Recording;
use dokime::domains::media::NOT_COMPILED;
use dokime::domains::media::UNAVAILABLE;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::operations::media as contract;
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
    runtime: LiveRuntime,
    maximum: usize,
}

struct Unsupported;

// A Cancelled Or Failed Stream Still Seals What Was Recorded.
struct Unsealed(Option<Recording>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Serve<contract::Screenshot> for Resources {
    fn serve<'a>(&'a self, request: contract::Screenshot, _: Context<'a, contract::Screenshot>) -> Served<'a, ArtifactMetadata> {
        Box::pin(async move {
            let scope = capture_scope(request.scope.as_ref())?;
            let record =
                dokime::app::live_runtime(&self.runtime).await?.capture_screenshot(scope, request.display).await.map_err(failure)?;
            self.respond(&record)
        })
    }
}

impl Serve<contract::Photo> for Resources {
    fn serve<'a>(&'a self, request: contract::Photo, _: Context<'a, contract::Photo>) -> Served<'a, ArtifactMetadata> {
        Box::pin(async move {
            let scope = capture_scope(request.scope.as_ref())?;
            let record = dokime::app::live_runtime(&self.runtime).await?.capture_photo(scope, request.camera).await.map_err(failure)?;
            self.respond(&record)
        })
    }
}

impl Serve<contract::AudioRecording> for Resources {
    fn serve<'a>(&'a self, request: contract::AudioRecording, ctx: Context<'a, contract::AudioRecording>) -> Served<'a, ()> {
        Box::pin(async move {
            let call_context = ctx.call();
            let output = ctx.streams();
            let revoked = crate::grpc::access::revocation(call_context);
            crate::grpc::access::during(revoked, async {
                let limit = recording_limit(request.maximum_seconds)?;
                let scope = capture_scope(request.scope.as_ref())?;
                let started =
                    dokime::app::live_runtime(&self.runtime).await?.start_audio_recording(scope, request.input).await.map_err(failure)?;
                let mut unsealed = Unsealed(None);
                let recording = unsealed.0.insert(started);

                // Report Progress Until The Limit Or Until The Execution Seals It;
                // Cancellation Seals Through `Unsealed`
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
                        .map_err(crate::grpc::chunked::output_failure)?;
                    tokio::select! {
                        sealed = recording.settled() => break sealed,
                        () = tokio::time::sleep(Duration::from_millis(SERVER_MEDIA_PROGRESS_MS).min(limit - elapsed)) => {}
                    }
                };

                unsealed.0 = None;
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
                    .map_err(crate::grpc::chunked::output_failure)?;
                Ok::<_, Status>(().into())
            })
            .await
        })
    }
}

impl Serve<contract::ScreenRecording> for Unsupported {
    fn serve<'a>(&'a self, _: contract::ScreenRecording, ctx: Context<'a, contract::ScreenRecording>) -> Served<'a, ()> {
        Box::pin(async move {
            let revoked = crate::grpc::access::revocation(ctx.call());
            crate::grpc::access::during(revoked, async { Err(Status::failed_precondition(NO_VIDEO_ENCODER)) }).await
        })
    }
}

impl Serve<contract::CameraRecording> for Unsupported {
    fn serve<'a>(&'a self, _: contract::CameraRecording, ctx: Context<'a, contract::CameraRecording>) -> Served<'a, ()> {
        Box::pin(async move {
            let revoked = crate::grpc::access::revocation(ctx.call());
            crate::grpc::access::during(revoked, async { Err(Status::failed_precondition(NO_VIDEO_ENCODER)) }).await
        })
    }
}

impl Drop for Unsealed {
    fn drop(&mut self) {
        let Some(mut recording) = self.0.take() else {
            return;
        };
        tokio::spawn(async move {
            let filename = recording.filename().to_owned();
            match recording.finish().await {
                Ok(record) => tracing::info!(artifact = %record.artifact_id, %filename, "cancelled audio recording sealed"),
                Err(error) => tracing::warn!(%filename, %error, "cancelled audio recording could not be sealed"),
            }
        });
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

pub(super) fn prepare(
    setup: &mut dokime::harmos::Setup<dokime::app::Dokime>, runtime: &LiveRuntime, maximum: usize,
) -> Result<(), crate::grpc::Error> {
    let served = Arc::new(Resources { runtime: runtime.clone(), maximum });
    let unsupported = Arc::new(Unsupported);

    setup.resource::<Host<contract::Screenshot>>(Host::<contract::Screenshot>::new(served.clone()));
    setup.resource::<Host<contract::Photo>>(Host::<contract::Photo>::new(served.clone()));
    setup.resource::<Host<contract::AudioRecording>>(Host::<contract::AudioRecording>::new(served));
    setup.resource::<Host<contract::ScreenRecording>>(Host::<contract::ScreenRecording>::new(unsupported.clone()));
    setup.resource::<Host<contract::CameraRecording>>(Host::<contract::CameraRecording>::new(unsupported));

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

/* === Tests === */
