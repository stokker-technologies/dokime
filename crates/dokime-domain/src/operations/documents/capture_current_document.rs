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
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos_domain::message]
#[harmos_domain::operation(response = (), output = DocumentFrame, id = "dokime.documents.capture_current_document", version = 1,
        policy = TRANSFER, visibility = harmos_domain::operations::Visibility::Host,
        grpc(package = "dokime.v1", service = "Documents", method = "CaptureCurrentDocument"))]
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct CaptureCurrentDocument {
    pub scope: Option<DocumentScope>,
    pub document_id: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
