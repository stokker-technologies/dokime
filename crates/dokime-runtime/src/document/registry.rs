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
use std::collections::BTreeMap;

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use crate::document::{DocumentDefinition, DocumentInstance};
use dokime_domain::model::identity::DocumentId;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

product_error!(DocumentRegistryError, formatter {
    Self::DuplicateDefinition(id) => write!(formatter, "document definition {id} is already registered"),
    Self::DuplicateInstance(id) => write!(formatter, "document instance {id} is already registered"),
});

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DocumentRegistry {
    definitions: BTreeMap<DocumentId, DocumentDefinition>,
    instances: BTreeMap<DocumentId, DocumentInstance>,
}

/* === Enums === */

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DocumentRegistryError {
    DuplicateDefinition(DocumentId),
    DuplicateInstance(DocumentId),
}

/* === Traits === */

/* === Implementations === */

impl DocumentRegistry {
    pub fn definitions(&self) -> impl Iterator<Item = &DocumentDefinition> {
        self.definitions.values()
    }

    pub fn get_definition(&self, document_id: &DocumentId) -> Option<&DocumentDefinition> {
        self.definitions.get(document_id)
    }

    pub fn instances(&self) -> impl Iterator<Item = &DocumentInstance> {
        self.instances.values()
    }

    pub fn open_instance(&mut self, definition: DocumentDefinition, updated_at: Timestamp) -> &mut DocumentInstance {
        let document_id = definition.document_id.clone();
        self.definitions.entry(document_id.clone()).or_insert_with(|| definition.clone());
        self.instances.entry(document_id).or_insert_with(|| DocumentInstance::new(definition, updated_at))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
