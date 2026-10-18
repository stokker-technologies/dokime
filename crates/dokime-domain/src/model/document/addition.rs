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
use super::definition::DocumentCard;
use super::definition::DocumentDefinition;
use super::definition::DocumentNode;
use super::definition::DocumentSection;
use crate::model::identity::Identifier;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
pub enum DocumentAddition {
    Section(Box<DocumentSection>),
    Card { section: Identifier, card: Box<DocumentCard> },
}

/* === Traits === */

/* === Implementations === */
impl DocumentAddition {
    pub fn section(section: DocumentSection) -> Self {
        Self::Section(Box::new(section))
    }

    pub fn card(section: Identifier, card: DocumentCard) -> Self {
        Self::Card { section, card: Box::new(card) }
    }
}

impl DocumentDefinition {
    pub fn add(&mut self, addition: DocumentAddition) -> Result<(), String> {
        self.fold_authored_sections();

        match addition {
            DocumentAddition::Section(section) => self.add_section(*section),
            DocumentAddition::Card { section, card } => self.add_card(&section, *card),
        }
    }

    pub fn has_section(&self, section_id: &Identifier) -> bool {
        nodes_carry_section(&self.layout.nodes, section_id)
            || self.sections.iter().any(|section| &section.metadata.identifier == section_id)
    }

    pub fn has_card(&self, card_id: &Identifier) -> bool {
        self.cards.iter().any(|card| &card.metadata.identifier == card_id)
            || self.sections.iter().flat_map(|section| section.cards.iter()).any(|card| &card.metadata.identifier == card_id)
    }

    pub fn section_card_count(&self, section_id: &Identifier) -> Option<usize> {
        if let Some(count) = nodes_section_card_count(&self.layout.nodes, section_id) {
            return Some(count);
        }

        self.sections.iter().find(|section| &section.metadata.identifier == section_id).map(|section| section.cards.len())
    }

    fn add_section(&mut self, section: DocumentSection) -> Result<(), String> {
        let section_id = section.metadata.identifier.clone();
        if self.has_section(&section_id) {
            return Err(format!(
                "section \"{section_id}\" is already in this document; a declared part cannot be \
                 replaced from a stage"
            ));
        }
        if let Some(card) = section.cards.iter().find(|card| self.has_card(&card.metadata.identifier)) {
            return Err(format!(
                "card \"{}\" is already in this document; rename the card section \"{section_id}\" \
                 carries",
                card.metadata.identifier
            ));
        }

        self.insert_added_section(section);

        Ok(())
    }

    fn add_card(&mut self, section_id: &Identifier, card: DocumentCard) -> Result<(), String> {
        if !self.has_section(section_id) {
            return Err(format!(
                "section \"{section_id}\" is not in this document; add the section before the \
                 cards it carries"
            ));
        }
        let card_id = card.metadata.identifier.clone();
        if self.has_card(&card_id) {
            return Err(format!(
                "card \"{card_id}\" is already in this document; a declared part cannot be \
                 replaced from a stage"
            ));
        }

        self.append_card(section_id, card);

        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn nodes_carry_section(nodes: &[DocumentNode], section_id: &Identifier) -> bool {
    nodes.iter().any(|node| match node {
        DocumentNode::Section(section) => &section.node_id == section_id || nodes_carry_section(&section.children, section_id),
        DocumentNode::Card(_) => false,
    })
}

fn nodes_section_card_count(nodes: &[DocumentNode], section_id: &Identifier) -> Option<usize> {
    for node in nodes {
        let DocumentNode::Section(section) = node else {
            continue;
        };
        if &section.node_id == section_id {
            return Some(section.children.iter().filter(|child| matches!(child, DocumentNode::Card(_))).count());
        }
        if let Some(count) = nodes_section_card_count(&section.children, section_id) {
            return Some(count);
        }
    }

    None
}

/* === Tests === */
