/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Operator media captured on the backend host: each capture becomes one
// execution artifact under its media folder, sealed through the upload path.

/* === Modules === */
mod audio;
mod camera;
mod screen;

/* === Re-Exports === */
pub use audio::AudioRecording;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::artifacts::{ArtifactRecord, ArtifactUploadInput};
use crate::{RuntimeHandle, runtime::Error, runtime::Result};
use dokime_domain::model::diagnostics::admission::AdmissionFailure;
use dokime_domain::model::execution::RecordScope;

/* === Constants === */
// Error code of a capture refused because the host has no usable device.
pub const UNAVAILABLE: &str = "media_unavailable";

// Error code of a capture refused because this build has no native capture.
pub const NOT_COMPILED: &str = "media_not_compiled";

// Recordings stop and seal after this long unless the caller asks otherwise.
pub const DEFAULT_RECORDING_SECONDS: u32 = 600;

// The longest recording one request may ask for.
pub const MAXIMUM_RECORDING_SECONDS: u32 = 3600;

const CHUNK_BYTES: usize = 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    // Captures one display as `screenshots/<UTC timestamp>.png`.
    pub async fn capture_screenshot(&self, scope: RecordScope, display: Option<u32>) -> Result<ArtifactRecord> {
        require_execution(&scope)?;
        let png = screen::capture_png(display).await?;
        self.store_capture(scope, capture_name("screenshots", "png"), "image/png", png).await
    }

    // Captures one camera frame as `photos/<UTC timestamp>.jpg`.
    pub async fn capture_photo(&self, scope: RecordScope, camera: Option<u32>) -> Result<ArtifactRecord> {
        require_execution(&scope)?;
        let jpeg = tokio::task::spawn_blocking(move || camera::capture_jpeg(camera)).await??;
        self.store_capture(scope, capture_name("photos", "jpg"), "image/jpeg", jpeg).await
    }

    // Starts recording one audio input into `audio/<UTC timestamp>.wav`.
    pub async fn start_audio_recording(&self, scope: RecordScope, input: Option<u32>) -> Result<AudioRecording> {
        require_execution(&scope)?;
        let filename = capture_name("audio", "wav");
        AudioRecording::start(self.clone(), scope, filename, input).await
    }

    async fn store_capture(&self, scope: RecordScope, filename: String, content_type: &str, bytes: Vec<u8>) -> Result<ArtifactRecord> {
        let input = ArtifactUploadInput {
            scope,
            filename,
            content_type: Some(content_type.to_owned()),
            size_bytes: bytes.len() as u64,
            sha256: None,
        };
        let upload = self.begin_artifact_upload(input).await?;

        // Send Bounded Chunks, Abandoning A Partial Capture
        for chunk in bytes.chunks(CHUNK_BYTES) {
            if let Err(source) = upload.send(chunk).await {
                if let Err(cleanup) = upload.abandon().await {
                    tracing::warn!(error = %cleanup, "media capture abandonment failed");
                }
                return Err(source);
            }
        }
        upload.finish().await
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn require_execution(scope: &RecordScope) -> Result<()> {
    if scope.test_cycle_id.is_none() || scope.test_execution_id.is_none() {
        return Err(Error::Admission(AdmissionFailure::Validation {
            code: "media_scope".into(),
            message: "media capture requires a cycle and execution scope".into(),
            issues: Vec::new(),
        }));
    }
    Ok(())
}

fn capture_name(folder: &str, extension: &str) -> String {
    let now = time::OffsetDateTime::now_utc();
    format!(
        "{folder}/{:04}-{:02}-{:02}_{:02}-{:02}-{:02}-{:03}.{extension}",
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
        now.millisecond(),
    )
}

#[cfg_attr(not(feature = "media-native"), allow(dead_code))]
fn unavailable(message: impl std::fmt::Display) -> Error {
    Error::Admission(AdmissionFailure::Unavailable { code: UNAVAILABLE.into(), message: message.to_string() })
}

#[cfg_attr(feature = "media-native", allow(dead_code))]
fn not_compiled() -> Error {
    Error::Admission(AdmissionFailure::Unavailable { code: NOT_COMPILED.into(), message: "capture not available in this build".into() })
}

/* === Tests === */
