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
    Self::InvalidDefinition(message) => formatter.write_str(message),
    Self::DuplicateDefinition { identifier, first_label, second_label } => write!(formatter, "documents {first_label:?} and {second_label:?} share identifier {identifier}; set .id(...) to distinct identifiers"),
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
    InvalidDefinition(String),
    DuplicateDefinition { identifier: DocumentId, first_label: String, second_label: String },
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

    pub fn open_instance(&mut self, definition: DocumentDefinition, updated_at: Timestamp) -> Result<&mut DocumentInstance, DocumentRegistryError> {
        definition.validate_identity().map_err(DocumentRegistryError::InvalidDefinition)?;
        let document_id = definition.document_id.clone();
        if let Some(existing) = self.definitions.get(&document_id) {
            return Err(DocumentRegistryError::DuplicateDefinition {
                identifier: document_id,
                first_label: existing.metadata.label.clone(),
                second_label: definition.metadata.label.clone(),
            });
        }
        self.definitions.entry(document_id.clone()).or_insert_with(|| definition.clone());
        Ok(self.instances.entry(document_id).or_insert_with(|| DocumentInstance::new(definition, updated_at)))
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::DocumentSection;

    #[test]
    fn document_label_derivation_override_and_collision_refusal() {
        let mut registry = DocumentRegistry::default();
        let now = Timestamp::new("2026-01-01T00:00:00Z");
        for label in ["Hold at Temperature", "Mixed__Separators / Values", "Étage 42"] {
            let document = DocumentDefinition::new(label);
            assert_eq!(document.metadata.identifier, DocumentSection::new(label).metadata.identifier);
            assert_eq!(document.document_id.as_str(), document.metadata.identifier.as_str());
        }
        let document = DocumentDefinition::new("Hold at Temperature");
        assert_eq!(document.document_id.as_str(), "hold.at.temperature");
        registry.open_instance(document, now.clone()).unwrap();
        let collision = DocumentDefinition::new("Hold / at Temperature");
        assert!(matches!(
            registry.open_instance(collision.clone(), now.clone()),
            Err(DocumentRegistryError::DuplicateDefinition { first_label, second_label, .. })
                if first_label == "Hold at Temperature" && second_label == "Hold / at Temperature"
        ));
        assert_eq!(registry.definitions().count(), 1);
        let mut overridden = collision;
        overridden.id("hold");
        assert_eq!(overridden.metadata.identifier.as_str(), "hold");
        assert_eq!(overridden.document_id.as_str(), "hold");
        registry.open_instance(overridden, now.clone()).unwrap();
        for label in ["", "---", "温度"] {
            let mut invalid = DocumentDefinition::new(label);
            assert!(matches!(registry.open_instance(invalid.clone(), now.clone()), Err(DocumentRegistryError::InvalidDefinition(_))));
            invalid.id("explicit");
            assert!(invalid.validate_identity().is_ok());
        }
    }
}
