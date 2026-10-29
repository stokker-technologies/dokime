/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use serde::{Deserialize, Serialize};

use crate::document::{DocumentDefinition, DocumentPayload};
use dokime_domain::model::identity::DocumentId;
use dokime_domain::model::time::Timestamp;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocumentInstance {
    pub definition: DocumentDefinition,
    pub payload: Option<DocumentPayload>,
    pub updated_at: Timestamp,
}

impl DocumentInstance {
    pub fn new(definition: DocumentDefinition, updated_at: Timestamp) -> Self {
        Self { definition, payload: None, updated_at }
    }

    pub fn document_id(&self) -> &DocumentId {
        &self.definition.document_id
    }
}
