/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::Deserialize;
use serde::Serialize;

use super::*;

#[harmos::operation(resources = [crate::hosting::Host<CaptureCurrentDocument>], response = (), output = DocumentFrame, id = "dokime.documents.capture_current_document",
        policy = TRANSFER, visibility = harmos::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Documents", method = "CaptureCurrentDocument"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureCurrentDocument {
    pub scope: Option<DocumentScope>,
    pub document_id: String,
}

#[cfg(feature = "runtime")]
mod runtime {
    use super::*;

    crate::hosted!(CaptureCurrentDocument);
}
