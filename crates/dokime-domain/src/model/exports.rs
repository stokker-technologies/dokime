/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Portable Saved Evidence Export Subject And Transfer Inventory

use serde::{Deserialize, Serialize};

use crate::model::content::wire::{ArtifactChunk, ArtifactCompletion};

pub const EVIDENCE_EXPORT_SCHEMA_VERSION: u32 = 1;

#[harmos_domain::message]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceExportMetadata {
    pub subject: Option<EvidenceSubject>,
    pub filename: String,
    pub schema_version: u32,
    pub size_bytes: u64,
    pub sha256: String,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceExportFrame {
    pub payload: Option<EvidenceExportPayload>,
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceSubject {
    Execution(String),
    Cycle(String),
}

#[harmos_domain::message]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub enum EvidenceExportPayload {
    Metadata(EvidenceExportMetadata),
    Chunk(ArtifactChunk),
    Complete(ArtifactCompletion),
}
