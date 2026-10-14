/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod download_appearance_asset;
mod download_artifact;
mod download_execution_file;
mod follow_execution_file;
mod list_artifacts;
mod list_execution_files;
mod read_execution_file;
mod upload_artifact;

/* === Re-Exports === */
pub use download_appearance_asset::DownloadAppearanceAsset;
pub use download_artifact::DownloadArtifact;
pub use download_execution_file::DownloadExecutionFile;
pub use follow_execution_file::FollowExecutionFile;
pub use list_artifacts::ListArtifacts;
pub use list_execution_files::ListExecutionFiles;
pub use read_execution_file::ReadExecutionFile;
pub use upload_artifact::UploadArtifact;

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
#[allow(unused_imports)]
use super::appearance::*;
#[allow(unused_imports)]
use super::common::*;
use super::{Access, Endpoint, FOLLOW, ORDINARY, TRANSFER};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Metadata-only inventory for operator views. Entries are ordered by artifact
// ID. A continuation is valid only while the runtime epoch and complete
// metadata fingerprint remain unchanged.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListArtifactsRequest {
    pub limit: u32,
    pub after_artifact_id: String,
    pub cursor: Option<ArtifactCursor>,
}

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
pub struct DownloadArtifactRequest {
    pub artifact_id: String,
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

// Files currently owned by an execution in this runtime. These logical names
// select registered outputs; they never resolve arbitrary filesystem paths.
// Listings are ordered by relative_path. A continuation binds to execution,
// runtime epoch and metadata fingerprint; changed listings require a restart.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ListExecutionFilesRequest {
    pub execution_id: String,
    pub limit: u32,
    pub after_path: String,
    pub cursor: Option<ExecutionFilesCursor>,
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

// A bounded read for live previews/tailing, not an immutable full-file download.
// A native producer may rewrite/truncate a streaming file between or during
// reads. Each window reports its actual offset and observed length. An offset
// past the current end is clamped to that end. max_bytes defaults to 64 KiB and
// must not exceed 64 KiB. No EOF/finality is implied by an empty streaming window.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ReadExecutionFileRequest {
    pub execution_id: String,
    pub relative_path: String,
    pub offset: u64,
    pub max_bytes: u32,
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

// Opens one admitted output on its original saved view or active producer inode.
// Saved reads verify the frozen descriptor length and digest. Live reads stop at
// the observed prefix; producer rewrites may make that read fail. Each original
// charged chunk is encoded before the next read. Completion describes bytes
// actually delivered. Consumers verify offsets, size, digest and final OK status.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadExecutionFileRequest {
    pub execution_id: String,
    pub relative_path: String,
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

// Follow the original registered production from a locally committed offset.
// A nonzero offset requires the identity returned by the first metadata frame.
// Reader cancellation never cancels the producer. Producer session generations
// may change without changing this retained-content identity.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct FollowExecutionFileRequest {
    pub execution_id: String,
    pub relative_path: String,
    pub identity: Option<ExecutionFileIdentity>,
    pub offset: u64,
    // Defaults to 64 KiB; must not exceed 64 KiB.
    #[row(description = "Defaults to 64 KiB; must not exceed 64 KiB.")]
    pub max_bytes: u32,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionFileIdentity {
    pub history: Option<super::checkpoint::StoreHistory>,
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
    pub position: Option<super::checkpoint::StorageCursor>,
    pub length: u64,
    // Exactly 64 lowercase hexadecimal characters, over the entire file.
    #[row(description = "Exactly 64 lowercase hexadecimal characters, over the entire file.")]
    pub sha256: String,
}

// At most 8 MiB per registered appearance asset. Capture and verify before
// streaming: one metadata frame, contiguous <=64 KiB chunks, completion and OK
// final status. A changed/missing reference requires refreshing GetAppearance.
#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct DownloadAppearanceAssetRequest {
    pub reference: Option<AppearanceAssetReference>,
}

#[harmos::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct AppearanceAssetFrame {
    pub payload: Option<AppearanceAssetFramePayload>,
}

/* === Enums === */
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

#[harmos::catalog]
pub enum Operations {
    DownloadAppearanceAsset(DownloadAppearanceAsset),
    ListExecutionFiles(ListExecutionFiles),
    ReadExecutionFile(ReadExecutionFile),
    DownloadExecutionFile(DownloadExecutionFile),
    FollowExecutionFile(FollowExecutionFile),
    ListArtifacts(ListArtifacts),
    DownloadArtifact(DownloadArtifact),
    UploadArtifact(UploadArtifact),
}

/* === Traits === */

/* === Implementations === */
impl Endpoint for DownloadAppearanceAsset {
    const PATH: &'static str = "/dokime.v1.Content/DownloadAppearanceAsset";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ListExecutionFiles {
    const PATH: &'static str = "/dokime.v1.Content/ListExecutionFiles";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ReadExecutionFile {
    const PATH: &'static str = "/dokime.v1.Content/ReadExecutionFile";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for DownloadExecutionFile {
    const PATH: &'static str = "/dokime.v1.Content/DownloadExecutionFile";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for FollowExecutionFile {
    const PATH: &'static str = "/dokime.v1.Content/FollowExecutionFile";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for ListArtifacts {
    const PATH: &'static str = "/dokime.v1.Content/ListArtifacts";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for DownloadArtifact {
    const PATH: &'static str = "/dokime.v1.Content/DownloadArtifact";
    const ACCESS: Access = Access::Read;
}

impl Endpoint for UploadArtifact {
    const PATH: &'static str = "/dokime.v1.Content/UploadArtifact";
    const ACCESS: Access = Access::Control;
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
