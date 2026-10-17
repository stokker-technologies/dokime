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
use serde::Deserialize;
use serde::Serialize;

// Local Imports
#[allow(unused_imports)]
use crate::model::appearance::wire::*;
#[allow(unused_imports)]
use crate::model::identity::wire::*;
use super::Access;
use super::Endpoint;
use super::FOLLOW;
use super::ORDINARY;
use super::TRANSFER;

use crate::model::content::wire::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
// Metadata-only inventory for operator views. Entries are ordered by artifact
// ID. A continuation is valid only while the runtime epoch and complete
// metadata fingerprint remain unchanged.






















// Files currently owned by an execution in this runtime. These logical names
// select registered outputs; they never resolve arbitrary filesystem paths.
// Listings are ordered by relative_path. A continuation binds to execution,
// runtime epoch and metadata fingerprint; changed listings require a restart.








// A bounded read for live previews/tailing, not an immutable full-file download.
// A native producer may rewrite/truncate a streaming file between or during
// reads. Each window reports its actual offset and observed length. An offset
// past the current end is clamped to that end. max_bytes defaults to 64 KiB and
// must not exceed 64 KiB. No EOF/finality is implied by an empty streaming window.




// Opens one admitted output on its original saved view or active producer inode.
// Saved reads verify the frozen descriptor length and digest. Live reads stop at
// the observed prefix; producer rewrites may make that read fail. Each original
// charged chunk is encoded before the next read. Completion describes bytes
// actually delivered. Consumers verify offsets, size, digest and final OK status.






// Follow the original registered production from a locally committed offset.
// A nonzero offset requires the identity returned by the first metadata frame.
// Reader cancellation never cancels the producer. Producer session generations
// may change without changing this retained-content identity.












// At most 8 MiB per registered appearance asset. Capture and verify before
// streaming: one metadata frame, contiguous <=64 KiB chunks, completion and OK
// final status. A changed/missing reference requires refreshing GetAppearance.




/* === Enums === */

















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
