/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Artifact transfer, upload, report opening and operator media against a running execution.

/* === Modules === */

mod content;
mod media;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[dokime_tracing::trace_test]
#[tokio::test]
async fn execution_artifacts() {
    Box::pin(content::small_frame_transfer()).await;
    Box::pin(content::large_frame_transfer()).await;
    Box::pin(content::artifact_upload_owned()).await;
    Box::pin(content::artifact_upload_large()).await;
    Box::pin(content::artifact_upload_digest()).await;
    Box::pin(content::artifact_upload_match()).await;
    Box::pin(content::artifact_upload_completion()).await;
    Box::pin(content::open_report_states()).await;
    Box::pin(media::media_capture_artifacts()).await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn artifact_upload_idle_deadlines() {
    content::artifact_upload_idle_deadlines().await;
}
