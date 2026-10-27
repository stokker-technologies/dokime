/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Portable Saved Evidence Export Subject And Transfer Inventory

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::model::content::wire::{ArtifactChunk, ArtifactCompletion};

/* === Constants === */

pub const EVIDENCE_EXPORT_SCHEMA_VERSION: u32 = 1;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

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

/* === Enums === */

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

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
