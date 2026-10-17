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
use std::mem::size_of;

// Third Party Imports
use harmos::MemorySize;

// Local Imports
use crate::document::{DocumentCardDefinition, DocumentDefinition, DocumentNode, DocumentSectionNode};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn memory(definition: &DocumentDefinition) -> usize {
    let mut bytes = definition.cards.memory_size().saturating_add(definition.layout.memory_size());
    let mut added_cards = 0usize;
    for section in &definition.sections {
        // merge_authored_sections_into_layout clones every card definition,
        // then clones the section before converting its cards into placements.
        for card in &section.cards {
            bytes = bytes.saturating_add(card.memory_size());
            added_cards = added_cards.saturating_add(1);
        }
        bytes = bytes
            .saturating_add(section.memory_size())
            .saturating_add(size_of::<DocumentSectionNode>())
            .saturating_add(section.metadata.identifier.as_str().len())
            .saturating_add(section.cards.len().saturating_mul(size_of::<DocumentNode>()));
    }
    bytes = bytes.saturating_add(growth::<DocumentCardDefinition>(definition.cards.len(), added_cards));
    bytes = bytes.saturating_add(growth::<DocumentNode>(definition.layout.nodes.len(), definition.sections.len()));

    // An empty authored layout receives one placement for each card definition.
    if definition.layout.nodes.is_empty() && definition.sections.is_empty() {
        bytes = bytes.saturating_add(definition.cards.len().saturating_mul(size_of::<DocumentNode>()));
        for card in &definition.cards {
            bytes = bytes.saturating_add(card.metadata.identifier.as_str().len());
        }
    }
    bytes
}

pub(super) fn growth<T>(length: usize, added: usize) -> usize {
    let needed = length.saturating_add(added);
    let mut capacity = length;
    let mut slots = 0usize;
    while capacity < needed {
        let next = capacity.saturating_mul(2).max(4);
        slots = slots.saturating_add(next);
        if next <= capacity {
            return usize::MAX;
        }
        capacity = next;
    }
    slots.saturating_mul(size_of::<T>())
}

/* === Private Functions === */

/* === Tests === */
