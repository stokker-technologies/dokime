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
mod download_evidence;
mod download_execution_file;
mod follow_execution_file;
mod list_artifacts;
mod list_execution_files;
mod read_execution_file;
mod upload_artifact;

/* === Re-Exports === */

pub use download_appearance_asset::DownloadAppearanceAsset;
pub use download_artifact::DownloadArtifact;
pub use download_evidence::DownloadEvidence;
pub use download_execution_file::DownloadExecutionFile;
pub use follow_execution_file::FollowExecutionFile;
pub use list_artifacts::ListArtifacts;
pub use list_execution_files::ListExecutionFiles;
pub use read_execution_file::ReadExecutionFile;
pub use upload_artifact::UploadArtifact;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::Endpoint;
use super::FOLLOW;
use super::ORDINARY;
use super::TRANSFER;

use dokime_domain::model::appearance::wire::*;

use dokime_domain::model::content::wire::*;

/* === Constants === */

/* === Macros === */

#[cfg(feature = "runtime")]
crate::hosted!(
    DownloadAppearanceAsset,
    DownloadArtifact,
    DownloadEvidence,
    DownloadExecutionFile,
    FollowExecutionFile,
    ListArtifacts,
    ListExecutionFiles,
    ReadExecutionFile,
    UploadArtifact
);

/* === Type Aliases === */

/* === Structs === */

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
    DownloadEvidence(DownloadEvidence),
    UploadArtifact(UploadArtifact),
}

/* === Traits === */

/* === Implementations === */

impl Endpoint for DownloadAppearanceAsset {
    const PATH: &'static str = "/dokime.v1.Content/DownloadAppearanceAsset";
}

impl Endpoint for DownloadEvidence {
    const PATH: &'static str = "/dokime.v1.Content/DownloadEvidence";
}

impl Endpoint for ListExecutionFiles {
    const PATH: &'static str = "/dokime.v1.Content/ListExecutionFiles";
}

impl Endpoint for ReadExecutionFile {
    const PATH: &'static str = "/dokime.v1.Content/ReadExecutionFile";
}

impl Endpoint for DownloadExecutionFile {
    const PATH: &'static str = "/dokime.v1.Content/DownloadExecutionFile";
}

impl Endpoint for FollowExecutionFile {
    const PATH: &'static str = "/dokime.v1.Content/FollowExecutionFile";
}

impl Endpoint for ListArtifacts {
    const PATH: &'static str = "/dokime.v1.Content/ListArtifacts";
}

impl Endpoint for DownloadArtifact {
    const PATH: &'static str = "/dokime.v1.Content/DownloadArtifact";
}

impl Endpoint for UploadArtifact {
    const PATH: &'static str = "/dokime.v1.Content/UploadArtifact";
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
