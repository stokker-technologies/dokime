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
use super::{DEFAULT_DOCUMENT_ID, built_in_card, selected_execution_template};
use crate::document::{
    DocumentCard, DocumentCardBody, DocumentCardDefinition, DocumentDefinition, DocumentNode,
    DocumentSection, DocumentSectionNode,
};
use crate::execution::TestExecution;
use crate::execution_runtime::registry::SuiteRegistry;
use dokime::core::DocumentId;
use dokime::telemetry::{TelemetryColumn, TelemetryContract};

/* === Constants === */
/// The result built-ins lead the default document: steps and criteria, then
/// the operator's notes on them, before any authored section.
pub(super) const RESULT_SECTIONS: [Section; 2] = [
    Section {
        id: "execution.result",
        label: "Execution",
        cards: &[
            Card {
                id: "result.steps",
                label: "Steps",
                order: 10,
                body: DocumentCardBody::Steps,
            },
            Card {
                id: "result.criteria",
                label: "Criteria",
                order: 20,
                body: DocumentCardBody::Criteria,
            },
        ],
    },
    Section {
        id: "execution.traceability.notes",
        label: "Operator Notes",
        cards: &[Card {
            id: "traceability.notes",
            label: "Operator Notes",
            order: 10,
            body: DocumentCardBody::Notes,
        }],
    },
];
/// The evidence built-ins close the default document, after authored sections.
pub(super) const EVIDENCE_SECTIONS: [Section; 4] = [
    Section {
        id: "execution.events",
        label: "Records",
        cards: &[Card {
            id: "events.timeline",
            label: "Records",
            order: 10,
            body: DocumentCardBody::Records,
        }],
    },
    Section {
        id: "execution.traceability.artifacts",
        label: "Artifacts",
        cards: &[Card {
            id: "traceability.artifacts",
            label: "Artifacts",
            order: 10,
            body: DocumentCardBody::Artifacts,
        }],
    },
    Section {
        id: "execution.traceability.media",
        label: "Media",
        cards: &[Card {
            id: "traceability.media",
            label: "Media",
            order: 10,
            body: DocumentCardBody::Media,
        }],
    },
    Section {
        id: "execution.traceability.sections",
        label: "Traceability",
        cards: &[Card {
            id: "traceability.sections",
            label: "Traceability",
            order: 10,
            body: DocumentCardBody::Traceability,
        }],
    },
];
pub(super) const CYCLE_SECTIONS: [Section; 3] = [
    Section {
        id: "cycle.events",
        ..EVIDENCE_SECTIONS[0]
    },
    Section {
        id: "cycle.artifacts",
        ..EVIDENCE_SECTIONS[1]
    },
    Section {
        id: "cycle.media",
        ..EVIDENCE_SECTIONS[2]
    },
];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Copy)]
pub(super) struct Section {
    id: &'static str,
    label: &'static str,
    cards: &'static [Card],
}
struct Card {
    id: &'static str,
    label: &'static str,
    order: i32,
    body: DocumentCardBody,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(crate) fn construction_memory(
    suites: &SuiteRegistry,
    execution: &TestExecution,
    document_id: &DocumentId,
) -> Option<usize> {
    let authored = execution
        .documents
        .values()
        .find(|definition| &definition.document_id == document_id);
    let is_default = document_id.as_str() == DEFAULT_DOCUMENT_ID;
    if authored.is_none() && !is_default {
        return None;
    }

    let mut bytes = match authored {
        Some(definition) => definition.memory_size(),
        None => {
            let label = selected_execution_template(execution)
                .map(|template| template.metadata.label.as_str())
                .unwrap_or_else(|| execution.snapshot.case().metadata.label.as_str());
            size_of::<DocumentDefinition>()
                .saturating_add(DEFAULT_DOCUMENT_ID.len())
                .saturating_add(DEFAULT_DOCUMENT_ID.len())
                .saturating_add(label.len())
        }
    };
    if is_default {
        let sections = RESULT_SECTIONS.iter().chain(EVIDENCE_SECTIONS.iter());
        let section_count = sections.clone().count();
        let cards = sections.clone().map(|section| section.cards.len()).sum();
        bytes = bytes.saturating_add(growth_memory::<DocumentCardDefinition>(
            authored.map_or(0, |definition| definition.cards.len()),
            cards,
        ));
        bytes = bytes.saturating_add(growth_memory::<DocumentNode>(
            authored.map_or(0, |definition| definition.layout.nodes.len()),
            section_count,
        ));
        // Both built-in section collections can be admitted together safely.
        bytes = bytes.saturating_add(section_count.saturating_mul(size_of::<DocumentSection>()));
        for section in sections {
            bytes = bytes.saturating_add(section_memory(section));
        }
    }

    let pending = authored.map_or(0, |definition| {
        definition
            .cards
            .iter()
            .map(|card| &card.body)
            .chain(
                definition
                    .sections
                    .iter()
                    .flat_map(|section| section.cards.iter().map(|card| &card.body)),
            )
            .filter(|body| matches!(body, DocumentCardBody::Plot { plot } if plot.x_axis.is_none()))
            .count()
    });
    if pending == 0 {
        return Some(bytes);
    }
    let mut contracts = 0usize;
    let mut largest_unit = 0usize;
    for contract in suites
        .suites()
        .flat_map(|suite| suite.telemetry_contracts.iter())
    {
        contracts = contracts.saturating_add(1);
        bytes = bytes.saturating_add(contract.memory_size());
        if !contract
            .columns
            .iter()
            .any(|column| column.metadata.identifier.as_str() == "timestamp")
        {
            bytes =
                bytes.saturating_add(growth_memory::<TelemetryColumn>(contract.columns.len(), 1));
            bytes = bytes
                .saturating_add(size_of::<TelemetryColumn>())
                .saturating_add("timestamp".len())
                .saturating_add("Timestamp".len())
                .saturating_add("The instant this row was measured.".len());
        }
        if contract.default_x.is_none() {
            bytes = bytes.saturating_add("timestamp".len());
        }
        for column in &contract.columns {
            largest_unit =
                largest_unit.max(column.unit.as_ref().map_or(0, |unit| unit.symbol.len()));
        }
    }
    // The normalized vector coexists with its values and the completed document.
    bytes = bytes.saturating_add(contracts.saturating_mul(size_of::<TelemetryContract>()));
    bytes = bytes.saturating_add(pending.saturating_mul(largest_unit));
    Some(bytes)
}

pub(super) fn sections(descriptors: &[Section]) -> Vec<DocumentSection> {
    descriptors
        .iter()
        .map(|descriptor| {
            let mut section = DocumentSection::new(descriptor.label);
            section.id(descriptor.id);
            section.cards = Vec::with_capacity(descriptor.cards.len());
            for card in descriptor.cards {
                section.card(built_in_card(
                    card.id,
                    card.label,
                    card.order,
                    card.body.clone(),
                ));
            }
            section
        })
        .collect()
}

/* === Private Functions === */
fn section_memory(section: &Section) -> usize {
    // Original section/card values, cloned definitions and converted placements
    // coexist while insert_section_node composes the document.
    let mut bytes = size_of::<DocumentSectionNode>()
        .saturating_add(section.id.len()) // Metadata Identifier
        .saturating_add(section.id.len()) // Node Identifier
        .saturating_add(section.id.len()) // upsert_section Temporary Identifier
        .saturating_add(section.label.len())
        .saturating_add(metadata_attributes_memory().saturating_mul(2));
    bytes = bytes.saturating_add(
        section
            .cards
            .len()
            .saturating_mul(size_of::<DocumentCard>()),
    );
    bytes = bytes.saturating_add(
        section
            .cards
            .len()
            .saturating_mul(size_of::<DocumentNode>()),
    );
    for card in section.cards {
        bytes = bytes
            .saturating_add(size_of::<DocumentCardDefinition>())
            .saturating_add(card.id.len()) // Original Card Identifier
            .saturating_add(card.label.len()) // Original Card Label
            .saturating_add(card.id.len()) // Cloned Definition Identifier
            .saturating_add(card.label.len()) // Cloned Definition Label
            .saturating_add(metadata_attributes_memory().saturating_mul(2));
    }
    bytes
}

fn metadata_attributes_memory() -> usize {
    harmos::btree_map_memory::<String, harmos::Value>(0)
        .saturating_sub(size_of::<std::collections::BTreeMap<String, harmos::Value>>())
}

fn growth_memory<T>(length: usize, added: usize) -> usize {
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

/* === Tests === */
