/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Download One Complete Saved Evidence Archive

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::*;
use crate::model::exports::{EvidenceExportFrame, EvidenceSubject};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos_domain::message]
#[harmos_domain::operation(response = (), output = EvidenceExportFrame, id = "dokime.content.download_evidence", version = 1,
    policy = TRANSFER, visibility = harmos_domain::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Content", method = "DownloadEvidence"))]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadEvidence {
    pub subject: Option<EvidenceSubject>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
