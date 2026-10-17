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
use serde::Deserialize;
use serde::Serialize;

// Local Imports
#[allow(unused_imports)]
use crate::model::appearance::wire::*;
#[allow(unused_imports)]
use crate::model::identity::wire::*;

/* === Types === */

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactCursor {
    pub epoch: String,
    pub metadata_sha256: Vec<u8>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListArtifactsReply {
    pub artifacts: Vec<ArtifactMetadata>,
    pub next_after_artifact_id: Option<String>,
    pub cursor: Option<ArtifactCursor>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactScope {
    pub cycle_id: Option<String>,
    pub case_id: Option<String>,
    // Captured artifacts carry their owning execution.
    #[row(description = "Captured artifacts carry their owning execution.")]
    pub execution_id: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactMetadata {
    pub artifact_id: String,
    pub upload_id: String,
    pub scope: Option<ArtifactScope>,
    pub filename: String,
    pub content_type: Option<String>,
    pub size_bytes: u64,
    // Exactly 64 lowercase hexadecimal characters.
    #[row(description = "Exactly 64 lowercase hexadecimal characters.")]
    pub sha256: String,
    pub state: ArtifactState,
    pub created_at: Option<Timestamp>,
}

// One metadata frame, contiguous chunks bounded by the listener's configured
// transfer size starting at offset 0, then one completion frame. An immutable
// checkpoint is pinned for the stream.
// Cancelled/rejected artifacts may contain partial evidence; their state remains
// explicit. Completion verifies the stored length and digest, and must be
// followed by an OK gRPC final status before the consumer accepts the transfer.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactFrame {
    pub payload: Option<ArtifactFramePayload>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactChunk {
    pub offset: u64,
    pub data: Vec<u8>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ArtifactCompletion {
    pub size_bytes: u64,
    pub sha256: String,
}

// The first upload frame must carry the header. Every later frame must carry
// either a non-empty chunk with the next expected offset or the terminal
// completion frame; no frame may follow completion. The server caps chunks at
// 64 KiB, further bounded by the listener's configured message limit. The
// header's digest is optional: the terminal completion frame
// carries the authoritative size and sha256, verified against the bytes
// actually received before the upload commits. Uploads have no resumable
// session or idempotency key; callers must not retry a stream automatically
// after an uncertain disconnect.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct UploadArtifactFrame {
    pub payload: Option<UploadArtifactFramePayload>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct UploadArtifactHeader {
    pub scope: Option<ArtifactScope>,
    pub filename: String,
    pub content_type: Option<String>,
    pub size_bytes: u64,
    pub sha256: Option<String>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionFilesCursor {
    pub epoch: String,
    pub execution_id: String,
    pub metadata_sha256: Vec<u8>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListExecutionFilesReply {
    pub files: Vec<ExecutionFileMetadata>,
    pub next_after_path: Option<String>,
    pub cursor: Option<ExecutionFilesCursor>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionFileMetadata {
    pub relative_path: String,
    pub content_type: Option<String>,
    pub kind: ExecutionFileKind,
    pub state: ExecutionFileState,
    pub size_bytes: u64,
    pub created_at: Option<Timestamp>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionFileWindow {
    pub relative_path: String,
    pub content_type: Option<String>,
    pub kind: ExecutionFileKind,
    pub state: ExecutionFileState,
    pub offset: u64,
    pub observed_size_bytes: u64,
    pub data: Vec<u8>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionFileDownloadFrame {
    pub payload: Option<ExecutionFileDownloadFramePayload>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionFileDownloadMetadata {
    pub relative_path: String,
    pub content_type: Option<String>,
    pub kind: ExecutionFileKind,
    pub state: ExecutionFileState,
    pub observed_size_bytes: u64,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionFileIdentity {
    pub history: Option<crate::model::execution::wire_checkpoint::StoreHistory>,
    pub file_id: String,
    pub execution_id: String,
    pub relative_path: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionFileFollowFrame {
    pub payload: Option<ExecutionFileFollowFramePayload>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionFileFollowMetadata {
    pub identity: Option<ExecutionFileIdentity>,
    pub content_type: String,
    pub kind: ExecutionFileKind,
    pub phase: ExecutionFilePhase,
    pub available_length: u64,
    pub offset: u64,
}

// Finished carries the original sealed FileReference, covering the entire
// file including bytes retained before a resumed stream. Abandoned retains
// partial bytes and carries no sealed reference. A terminal frame requires
// final OK transport status before acceptance; stream EOF alone proves nothing.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionFileFollowTerminal {
    pub phase: ExecutionFilePhase,
    pub length: u64,
    pub sealed: Option<ExecutionFileSeal>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionFileSeal {
    pub identity: Option<ExecutionFileIdentity>,
    pub position: Option<crate::model::execution::wire_checkpoint::StorageCursor>,
    pub length: u64,
    // Exactly 64 lowercase hexadecimal characters, over the entire file.
    #[row(description = "Exactly 64 lowercase hexadecimal characters, over the entire file.")]
    pub sha256: String,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AppearanceAssetFrame {
    pub payload: Option<AppearanceAssetFramePayload>,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactState {
    Unspecified,
    Complete,
    Cancelled,
    Rejected,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionFileKind {
    Unspecified,
    Text,
    Binary,
    Media,
    Unknown,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionFileState {
    Unspecified,
    Streaming,
    Finalized,
    Interrupted,
}

#[harmos::message]
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionFilePhase {
    Unspecified,
    Producing,
    Finished,
    Abandoned,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ExecutionFileFollowFramePayload {
    Metadata(ExecutionFileFollowMetadata),
    Chunk(ArtifactChunk),
    Terminal(ExecutionFileFollowTerminal),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ArtifactFramePayload {
    Metadata(ArtifactMetadata),
    Chunk(ArtifactChunk),
    Complete(ArtifactCompletion),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum UploadArtifactFramePayload {
    Header(UploadArtifactHeader),
    Chunk(ArtifactChunk),
    Complete(ArtifactCompletion),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum ExecutionFileDownloadFramePayload {
    Metadata(ExecutionFileDownloadMetadata),
    Chunk(ArtifactChunk),
    Complete(ArtifactCompletion),
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum AppearanceAssetFramePayload {
    Metadata(AppearanceAssetMetadata),
    Chunk(ArtifactChunk),
    Complete(ArtifactCompletion),
}

/* === Public Functions === */

/* === Private Functions === */
