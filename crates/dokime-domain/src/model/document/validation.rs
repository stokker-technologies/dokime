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

// Local Imports
use crate::model::document::definition::DocumentCardBody;
use crate::model::document::definition::DocumentCardDefinition;
use crate::model::document::definition::DocumentCardLayout;
use crate::model::document::definition::DocumentDefinition;
use crate::model::document::definition::DocumentLayout;
use crate::model::document::definition::DocumentNode;
use crate::model::document::definition::{plot_validation_issue_bytes, validate_plot_definition};
use crate::model::identity::Identifier;
use crate::model::identity::Metadata;
use crate::model::validation::diagnostic::{size as diagnostic_size, text as diagnostic};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
struct ValidationMemory {
    bytes: usize,
    temporary: usize,
    issues: usize,
    sections: usize,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn validate_document_definition(
    definition: &DocumentDefinition, maximum_depth: usize, maximum_plot_points: usize,
) -> Result<(), Vec<String>> {
    let (cards, layout) = definition.normalized_parts();
    validate_normalized_parts(&cards, &layout, maximum_depth, maximum_plot_points)
}

pub fn validate_normalized_parts(
    cards: &[DocumentCardDefinition], layout: &DocumentLayout, maximum_depth: usize, maximum_plot_points: usize,
) -> Result<(), Vec<String>> {
    let memory = validation_memory(cards, layout, maximum_depth, maximum_plot_points);
    let mut issues = Vec::with_capacity(memory.issues);
    let mut card_ids = cards.iter().enumerate().map(|(index, card)| (&card.metadata.identifier, index)).collect::<Vec<_>>();
    card_ids.sort_unstable();
    let mut node_ids = Vec::with_capacity(memory.sections);
    index_nodes(&layout.nodes, 1, maximum_depth, &mut node_ids);
    node_ids.sort_unstable();

    for (index, card) in cards.iter().enumerate() {
        if matches!(&card.body, DocumentCardBody::Plot { plot } if plot.display_points.is_some_and(|points| points as usize > maximum_plot_points))
        {
            issues.push("plot display points exceed the configured maximum".into());
        }
        if let DocumentCardBody::Plot { plot } = &card.body
            && let Err(issue) = validate_plot_definition(plot)
        {
            issues.push(diagnostic(format_args!("card {}: {issue}", card.metadata.identifier.as_str())));
        }

        if first(&card_ids, &card.metadata.identifier) != Some(index) {
            issues.push(diagnostic(format_args!("duplicate card id {}", card.metadata.identifier.as_str())));
        }
    }

    validate_nodes(&layout.nodes, &card_ids, &node_ids, &mut 0, 1, &mut issues, maximum_depth);

    if issues.is_empty() { Ok(()) } else { Err(issues) }
}

/// Peak Native Storage Of Validation Indexes And Every Possible Diagnostic.
pub fn normalized_validation_memory(
    cards: &[DocumentCardDefinition], layout: &DocumentLayout, maximum_depth: usize, maximum_plot_points: usize,
) -> usize {
    let memory = validation_memory(cards, layout, maximum_depth, maximum_plot_points);
    memory
        .bytes
        .saturating_add(memory.temporary)
        .saturating_add(memory.issues.saturating_mul(std::mem::size_of::<String>()))
        .saturating_add(cards.len().saturating_add(memory.sections).saturating_mul(std::mem::size_of::<(&Identifier, usize)>()))
}

/* === Private Functions === */

fn first(index: &[(&Identifier, usize)], id: &Identifier) -> Option<usize> {
    let start = index.partition_point(|(candidate, _)| *candidate < id);
    index.get(start).filter(|(candidate, _)| *candidate == id).map(|(_, position)| *position)
}

fn index_nodes<'a>(nodes: &'a [DocumentNode], depth: usize, maximum_depth: usize, index: &mut Vec<(&'a Identifier, usize)>) {
    if depth > maximum_depth {
        return;
    }
    for node in nodes {
        let DocumentNode::Section(section) = node else { continue };
        index.push((&section.node_id, index.len()));
        index_nodes(&section.children, depth + 1, maximum_depth, index);
    }
}

fn validate_nodes(
    nodes: &[DocumentNode], card_ids: &[(&Identifier, usize)], node_ids: &[(&Identifier, usize)], position: &mut usize, depth: usize,
    issues: &mut Vec<String>, maximum_depth: usize,
) {
    if depth > maximum_depth {
        issues.push(diagnostic(format_args!("document layout exceeds maximum depth of {maximum_depth}")));
        return;
    }

    for node in nodes {
        match node {
            DocumentNode::Section(section) => {
                let current = *position;
                *position += 1;
                if first(node_ids, &section.node_id) != Some(current) {
                    issues.push(diagnostic(format_args!("duplicate section node id {}", section.node_id.as_str())));
                }
                validate_nodes(&section.children, card_ids, node_ids, position, depth + 1, issues, maximum_depth);
            }
            DocumentNode::Card(card) => {
                if first(card_ids, &card.card_id).is_none() {
                    issues.push(diagnostic(format_args!("missing card definition {}", card.card_id.as_str())));
                }
                validate_card_layout(&card.layout, issues);
            }
        }
    }
}

fn validation_memory(
    cards: &[DocumentCardDefinition], layout: &DocumentLayout, maximum_depth: usize, maximum_plot_points: usize,
) -> ValidationMemory {
    let mut memory = ValidationMemory::default();
    for card in cards {
        memory.card(&card.metadata, &card.body, maximum_plot_points);
    }
    memory.nodes(&layout.nodes, 1, maximum_depth);
    memory
}

impl ValidationMemory {
    fn card(&mut self, metadata: &Metadata, body: &DocumentCardBody, maximum_plot_points: usize) {
        if let DocumentCardBody::Plot { plot } = body {
            if plot.display_points.is_some_and(|points| points as usize > maximum_plot_points) {
                self.issue(format_args!("plot display points exceed the configured maximum"));
            }
            if let Some(bytes) = plot_validation_issue_bytes(plot) {
                self.issues = self.issues.saturating_add(1);
                self.bytes = self
                    .bytes
                    .saturating_add(diagnostic_size(format_args!("card {}: ", metadata.identifier.as_str())))
                    .saturating_add(bytes);
                self.temporary = self.temporary.max(bytes);
            }
        }
        self.issue(format_args!("duplicate card id {}", metadata.identifier.as_str()));
    }

    fn issue(&mut self, arguments: std::fmt::Arguments<'_>) {
        self.issues = self.issues.saturating_add(1);
        self.bytes = self.bytes.saturating_add(diagnostic_size(arguments));
    }

    fn nodes(&mut self, nodes: &[DocumentNode], depth: usize, maximum_depth: usize) {
        if depth > maximum_depth {
            self.issue(format_args!("document layout exceeds maximum depth of {maximum_depth}"));
            return;
        }
        for node in nodes {
            match node {
                DocumentNode::Section(section) => {
                    self.sections = self.sections.saturating_add(1);
                    self.issue(format_args!("duplicate section node id {}", section.node_id.as_str()));
                    self.nodes(&section.children, depth + 1, maximum_depth);
                }
                DocumentNode::Card(card) => {
                    self.issue(format_args!("missing card definition {}", card.card_id.as_str()));
                    for (label, span) in [
                        ("base", Some(card.layout.col_span.base)),
                        ("sm", card.layout.col_span.sm),
                        ("md", card.layout.col_span.md),
                        ("lg", card.layout.col_span.lg),
                        ("xl", card.layout.col_span.xl),
                    ] {
                        if span.is_some_and(|span| !(1..=12).contains(&span)) {
                            self.issue(format_args!("card layout {label} span must be between 1 and 12"));
                        }
                    }
                }
            }
        }
    }
}

fn validate_card_layout(layout: &DocumentCardLayout, issues: &mut Vec<String>) {
    for (label, span) in [
        ("base", Some(layout.col_span.base)),
        ("sm", layout.col_span.sm),
        ("md", layout.col_span.md),
        ("lg", layout.col_span.lg),
        ("xl", layout.col_span.xl),
    ] {
        if let Some(span) = span
            && !(1..=12).contains(&span)
        {
            issues.push(diagnostic(format_args!("card layout {label} span must be between 1 and 12")));
        }
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::document::definition::{DocumentCard, DocumentCardPlacement, DocumentSectionNode};
    use crate::model::identity::Metadata;

    #[test]
    fn sorted_indexes_preserve_source_diagnostic_order() {
        let card = DocumentCardDefinition::from(DocumentCard::text("duplicate.card", "Card", "body"));
        let cards = vec![card.clone(), card];
        let section = DocumentSectionNode {
            node_id: Identifier::new("duplicate.section").unwrap(),
            metadata: Metadata::new(Identifier::new("section").unwrap(), "Section", None),
            repeat: None,
            children: vec![DocumentNode::Card(DocumentCardPlacement {
                card_id: Identifier::new("missing.card").unwrap(),
                layout: Default::default(),
            })],
        };
        let mut duplicate = section.clone();
        duplicate.children.clear();
        let layout = DocumentLayout {
            nodes: vec![DocumentNode::Section(Box::new(section)), DocumentNode::Section(Box::new(duplicate))],
        };
        let estimate = normalized_validation_memory(&cards, &layout, 8, 100);
        let issues = validate_normalized_parts(&cards, &layout, 8, 100).unwrap_err();
        assert_eq!(issues.len(), 3);
        for (issue, identifier) in issues.iter().zip(["duplicate.card", "missing.card", "duplicate.section"]) {
            assert!(issue.contains(identifier));
        }
        let retained = issues.capacity() * std::mem::size_of::<String>() + issues.iter().map(String::capacity).sum::<usize>();
        assert!(estimate >= retained);
    }
}
