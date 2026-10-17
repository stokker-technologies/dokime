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
use serde::{Deserialize, Serialize};
// Local Imports
use crate::core::{DocumentId, Timestamp};
use crate::document::{DocumentDefinition, DocumentPayload};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentInstance {
    pub definition: DocumentDefinition,
    pub payload: Option<DocumentPayload>,
    pub updated_at: Timestamp,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl DocumentInstance {
    pub fn new(definition: DocumentDefinition, updated_at: Timestamp) -> Self {
        Self {
            definition,
            payload: None,
            updated_at,
        }
    }

    pub fn document_id(&self) -> &DocumentId {
        &self.definition.document_id
    }

    pub fn apply_payload(&mut self, payload: DocumentPayload, updated_at: Timestamp) {
        self.payload = Some(payload);
        self.updated_at = updated_at;
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
