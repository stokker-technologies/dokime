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
use std::mem::{size_of, size_of_val};

// Third Party Imports
use harmos::MemorySize;

// Local Imports
use crate::document::binding::Evidence;
use crate::document::resolve::{repeat_filter_memory, repeat_label_bound, repeat_selection_memory, repeat_template_memory};
use crate::document::{DocumentCardBody, DocumentDefinition, DocumentFilter, DocumentNode, DocumentSectionRepeat};
use dokime_domain::model::identity::Metadata;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
struct Selection {
    depth: usize,
    filter: usize,
    suffix: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn memory(definition: &DocumentDefinition, evidence: Evidence<'_>, constructed: usize) -> usize {
    let mut selected = Selection::default();
    inspect(&definition.layout.nodes, evidence, 0, &mut selected);
    for section in &definition.sections {
        if let Some(repeat) = &section.repeat {
            admit_repeat(repeat, evidence, 1, &mut selected);
        }
    }
    if selected.depth == 0 {
        return 0;
    }
    let largest = definition
        .cards
        .iter()
        .map(|card| card_fields(&card.metadata, &card.filters, &card.body, &selected))
        .chain(
            definition
                .sections
                .iter()
                .flat_map(|section| section.cards.iter())
                .map(|card| card_fields(&card.metadata, &card.filters, &card.body, &selected)),
        )
        .max()
        .unwrap_or(0);
    let mut sections = section_fields(&definition.layout.nodes, evidence, &selected);
    for section in &definition.sections {
        sections = sections.max(section_memory(
            &section.metadata,
            section.metadata.identifier.as_str().len(),
            section.repeat.as_ref(),
            evidence,
            &selected,
        ));
    }
    selected.filter.saturating_add(largest.saturating_add(sections).saturating_mul(constructed.saturating_add(1)))
}

/* === Private Functions === */

fn inspect(nodes: &[DocumentNode], evidence: Evidence<'_>, depth: usize, selected: &mut Selection) {
    for node in nodes {
        let DocumentNode::Section(section) = node else {
            continue;
        };
        let next = if let Some(repeat) = &section.repeat {
            let next = depth.saturating_add(1);
            admit_repeat(repeat, evidence, next, selected);
            next
        } else {
            depth
        };
        selected.depth = selected.depth.max(next);
        inspect(&section.children, evidence, next, selected);
    }
}

fn card_fields(metadata: &Metadata, filters: &Vec<DocumentFilter>, body: &DocumentCardBody, selected: &Selection) -> usize {
    let mut bytes = vector(filters, selected).saturating_add(identifier_fields(metadata, selected));
    match body {
        DocumentCardBody::Plot { plot } => {
            bytes = bytes.saturating_add(vector(&plot.filters, selected)).saturating_add(identifier_fields(&plot.metadata, selected))
        }
        DocumentCardBody::TelemetrySummary { summary } => {
            for group in &summary.groups {
                bytes = bytes.saturating_add(identifier_fields(&group.metadata, selected));
                for tile in &group.tiles {
                    bytes =
                        bytes.saturating_add(vector(&tile.filters, selected)).saturating_add(identifier_fields(&tile.metadata, selected));
                }
            }
        }
        _ => {}
    }
    bytes
}

fn admit_repeat(repeat: &DocumentSectionRepeat, evidence: Evidence<'_>, depth: usize, selected: &mut Selection) {
    selected.depth = selected.depth.max(depth);
    selected.filter = selected.filter.max(repeat_filter_memory(repeat, evidence));
    // Sanitization Never Expands The UTF-8 Label; An Empty Segment Becomes "value".
    selected.suffix = selected.suffix.max(repeat_label_bound(repeat, evidence).max("value".len()));
}

fn identifier_fields(metadata: &Metadata, selected: &Selection) -> usize {
    let ancestors = selected.depth.saturating_mul(selected.suffix.saturating_add(1));
    identifier_copies(metadata.identifier.as_str().len().saturating_add(ancestors))
}

fn identifier_copies(bytes: usize) -> usize {
    let bytes = bytes.max("value".len());
    // Generated Text And Sanitization Scratch
    let generated = bytes.saturating_add(bytes);
    // Stored Metadata And Its Temporary Definition Clone
    let definition = bytes.saturating_add(bytes);
    // Placement Identifier And Registry Key
    let placement = bytes.saturating_add(bytes);
    // Payload Metadata And Descriptor Identifier
    let payload = bytes.saturating_add(bytes);
    generated.saturating_add(definition).saturating_add(placement).saturating_add(payload)
}

fn section_fields(nodes: &[DocumentNode], evidence: Evidence<'_>, selected: &Selection) -> usize {
    nodes
        .iter()
        .filter_map(|node| {
            let DocumentNode::Section(section) = node else {
                return None;
            };
            Some(
                section_memory(&section.metadata, section.node_id.as_str().len(), section.repeat.as_ref(), evidence, selected)
                    .max(section_fields(&section.children, evidence, selected)),
            )
        })
        .max()
        .unwrap_or(0)
}

fn section_memory(
    metadata: &Metadata, node_id_bytes: usize, repeat: Option<&DocumentSectionRepeat>, evidence: Evidence<'_>, selected: &Selection,
) -> usize {
    let source = metadata.memory_size();
    if source == usize::MAX {
        return usize::MAX;
    }
    let ancestors = selected.depth.saturating_mul(selected.suffix.saturating_add(1));
    let identifier = node_id_bytes.saturating_add(ancestors);
    let templates = repeat.map_or(0, |repeat| repeat_template_memory(repeat, identifier, evidence));
    let candidates = repeat.map_or(0, |repeat| repeat_selection_memory(repeat, evidence));
    // The Template Count Contains Both Sequential Buffers, So Also Bounds The
    // Final Rendered Identifier And Label Moved Into Generated Metadata.
    let generated = identifier_copies(identifier.max(templates)).saturating_add(templates);
    source.saturating_add(templates).saturating_add(candidates).saturating_add(generated)
}

fn vector(filters: &Vec<DocumentFilter>, selected: &Selection) -> usize {
    let source = filters.memory_size();
    if source == usize::MAX || selected.filter == usize::MAX {
        return usize::MAX;
    }
    let added = selected.depth.saturating_mul(selected.filter);
    let slots = filters.len().saturating_add(selected.depth).saturating_mul(size_of::<DocumentFilter>());
    let source_heap = source.saturating_sub(size_of_val(filters));
    let retained = size_of::<Vec<DocumentFilter>>().saturating_add(source_heap).saturating_add(added);
    // The Stored Definition Coexists With A Repeated Definition Clone And Its
    // Old/New Filter Buffers While reserve_exact Appends An Ancestor Filter.
    let repeating = retained.saturating_add(slots).saturating_add(slots);
    // Plot Resolution Keeps Its Input Alongside The Descriptor's Input Clone.
    // Counting Those Copies For Other Card Kinds Is A Conservative Upper Bound.
    let input = retained;
    let descriptor = retained;
    retained.saturating_add(repeating).saturating_add(input).saturating_add(descriptor)
}

/* === Tests === */
