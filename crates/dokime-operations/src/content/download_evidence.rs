/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Download One Complete Saved Evidence Archive

use serde::{Deserialize, Serialize};

use super::*;
use dokime_domain::model::exports::{EvidenceExportFrame, EvidenceSubject};

#[harmos::operation(resources = [crate::hosting::Host<DownloadEvidence>], response = (), output = EvidenceExportFrame, id = "dokime.content.download_evidence",
    policy = TRANSFER, visibility = harmos::operations::Visibility::Host,
    grpc(package = "dokime.v1", service = "Content", method = "DownloadEvidence"))]
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DownloadEvidence {
    pub subject: Option<EvidenceSubject>,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(DownloadEvidence);
}
