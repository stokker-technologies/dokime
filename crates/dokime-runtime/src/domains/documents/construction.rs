/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use super::built_in_card;
use crate::document::{DocumentCardBody, DocumentSection};

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

pub(super) fn sections(descriptors: &[Section]) -> Vec<DocumentSection> {
    descriptors
        .iter()
        .map(|descriptor| {
            let mut section = DocumentSection::new(descriptor.label);
            section.id(descriptor.id);
            section.cards = Vec::with_capacity(descriptor.cards.len());
            for card in descriptor.cards {
                section.card(built_in_card(card.id, card.label, card.order, card.body.clone()));
            }
            section
        })
        .collect()
}
