/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Services;
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

#[tonic::async_trait]
impl v1::media_server::Media for Services {
    type AudioRecordingStream = dokime_operations_next::client::Output<v1::MediaAudioRecordingOutput>;
    async fn audio_recording(
        &self, request: tonic::Request<v1::MediaAudioRecordingRequest>,
    ) -> Result<tonic::Response<Self::AudioRecordingStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Media.AudioRecording: Held for the Client API Capture Decision"))
    }
    type RunAudioRecordingStream = dokime_operations_next::client::Output<v1::MediaAudioRecordingRunOutput>;
    async fn run_audio_recording(
        &self, request: tonic::Request<tonic::Streaming<v1::MediaAudioRecordingRunInput>>,
    ) -> Result<tonic::Response<Self::RunAudioRecordingStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Media.RunAudioRecording: Held for the Client API Capture Decision"))
    }
    type CameraRecordingStream = dokime_operations_next::client::Output<v1::MediaCameraRecordingOutput>;
    async fn camera_recording(
        &self, request: tonic::Request<v1::MediaCameraRecordingRequest>,
    ) -> Result<tonic::Response<Self::CameraRecordingStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Media.CameraRecording: Held for the Client API Capture Decision"))
    }
    type RunCameraRecordingStream = dokime_operations_next::client::Output<v1::MediaCameraRecordingRunOutput>;
    async fn run_camera_recording(
        &self, request: tonic::Request<tonic::Streaming<v1::MediaCameraRecordingRunInput>>,
    ) -> Result<tonic::Response<Self::RunCameraRecordingStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Media.RunCameraRecording: Held for the Client API Capture Decision"))
    }
    async fn photo(&self, request: tonic::Request<v1::MediaPhotoRequest>) -> Result<tonic::Response<v1::ArtifactMetadata>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Media.Photo: Held for the Client API Capture Decision"))
    }
    type RunPhotoStream = dokime_operations_next::client::Output<v1::MediaPhotoRunOutput>;
    async fn run_photo(
        &self, request: tonic::Request<tonic::Streaming<v1::MediaPhotoRunInput>>,
    ) -> Result<tonic::Response<Self::RunPhotoStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Media.RunPhoto: Held for the Client API Capture Decision"))
    }
    type ScreenRecordingStream = dokime_operations_next::client::Output<v1::MediaScreenRecordingOutput>;
    async fn screen_recording(
        &self, request: tonic::Request<v1::MediaScreenRecordingRequest>,
    ) -> Result<tonic::Response<Self::ScreenRecordingStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Media.ScreenRecording: Held for the Client API Capture Decision"))
    }
    type RunScreenRecordingStream = dokime_operations_next::client::Output<v1::MediaScreenRecordingRunOutput>;
    async fn run_screen_recording(
        &self, request: tonic::Request<tonic::Streaming<v1::MediaScreenRecordingRunInput>>,
    ) -> Result<tonic::Response<Self::RunScreenRecordingStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Media.RunScreenRecording: Held for the Client API Capture Decision"))
    }
    async fn screenshot(
        &self, request: tonic::Request<v1::MediaScreenshotRequest>,
    ) -> Result<tonic::Response<v1::ArtifactMetadata>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Media.Screenshot: Held for the Client API Capture Decision"))
    }
    type RunScreenshotStream = dokime_operations_next::client::Output<v1::MediaScreenshotRunOutput>;
    async fn run_screenshot(
        &self, request: tonic::Request<tonic::Streaming<v1::MediaScreenshotRunInput>>,
    ) -> Result<tonic::Response<Self::RunScreenshotStream>, tonic::Status> {
        let _ = request;
        Err(tonic::Status::unimplemented("Media.RunScreenshot: Held for the Client API Capture Decision"))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
