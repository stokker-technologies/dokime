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

// Third Party Imports
use tokio::sync::watch;
use tonic::Status;
// Local Imports
use super::super::chunked::{self, ArtifactFrames};
use dokime_app::runtime::ArtifactReader;
use dokime_protocol::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
pub(super) type ArtifactStream = chunked::FrameStream<v1::ArtifactFrame>;

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn checkpoint_artifact(
    reader: ArtifactReader,
    metadata: v1::ArtifactMetadata,
    max_message_bytes: usize,
    stopping: watch::Receiver<bool>,
) -> Result<ArtifactStream, Status> {
    chunked::Download {
        size_bytes: metadata.size_bytes,
        sha256: Some(metadata.sha256.clone()),
        frames: ArtifactFrames(metadata),
        source: chunked::Source::artifact(reader),
        noun: "artifact",
        // The runtime combines storage and integrity failures; do not claim
        // every read failure is corruption.
        read_failure: "artifact content could not be verified",
        chunk_bytes: (64 * 1024).min(max_message_bytes.saturating_sub(1024)),
        message_limit: max_message_bytes,
        stopping,
    }
    .stream()
}

/* === Private Functions === */

/* === Tests === */
