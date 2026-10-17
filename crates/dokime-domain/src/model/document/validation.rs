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
use std::collections::BTreeSet;
use std::fmt::Write;
// Third Party Imports

// Local Imports
use crate::model::document::definition::DocumentCardBody;
use crate::model::document::definition::DocumentCardDefinition;
use crate::model::document::definition::DocumentCardLayout;
use crate::model::document::definition::DocumentDefinition;
use crate::model::document::definition::DocumentLayout;
use crate::model::document::definition::DocumentNode;
use crate::model::document::definition::validate_plot_definition;
use crate::model::identity::Identifier;

/* === Constants === */
const MAX_LAYOUT_DEPTH: usize = 12;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn validate_document_definition(definition: &DocumentDefinition) -> Result<(), Vec<String>> {
    let (cards, layout) = definition.normalized_parts();
    validate_normalized_parts(&cards, &layout)
}

pub fn validate_normalized_parts(
    cards: &[DocumentCardDefinition],
    layout: &DocumentLayout,
) -> Result<(), Vec<String>> {
    let mut issues = Vec::new();
    let mut card_ids = BTreeSet::<&Identifier>::new();

    for card in cards {
        if let DocumentCardBody::Plot { plot } = &card.body
            && let Err(issue) = validate_plot_definition(plot)
        {
            issues.push(diagnostic(format_args!(
                "card {}: {issue}",
                card.metadata.identifier.as_str()
            )));
        }

        if !card_ids.insert(&card.metadata.identifier) {
            issues.push(diagnostic(format_args!(
                "duplicate card id {}",
                card.metadata.identifier.as_str()
            )));
        }
    }

    validate_layout(layout, &card_ids, &mut issues);

    if issues.is_empty() {
        Ok(())
    } else {
        Err(issues)
    }
}

/* === Private Functions === */
fn validate_layout(
    layout: &DocumentLayout,
    card_ids: &BTreeSet<&Identifier>,
    issues: &mut Vec<String>,
) {
    let mut node_ids = BTreeSet::<&Identifier>::new();

    validate_nodes(&layout.nodes, card_ids, &mut node_ids, 1, issues);
}

fn validate_nodes<'a>(
    nodes: &'a [DocumentNode],
    card_ids: &BTreeSet<&Identifier>,
    node_ids: &mut BTreeSet<&'a Identifier>,
    depth: usize,
    issues: &mut Vec<String>,
) {
    if depth > MAX_LAYOUT_DEPTH {
        issues.push(diagnostic(format_args!(
            "document layout exceeds maximum depth of {MAX_LAYOUT_DEPTH}"
        )));
        return;
    }

    for node in nodes {
        match node {
            DocumentNode::Section(section) => {
                if !node_ids.insert(&section.node_id) {
                    issues.push(diagnostic(format_args!(
                        "duplicate section node id {}",
                        section.node_id.as_str()
                    )));
                }
                validate_nodes(&section.children, card_ids, node_ids, depth + 1, issues);
            }
            DocumentNode::Card(card) => {
                if !card_ids.contains(&card.card_id) {
                    issues.push(diagnostic(format_args!(
                        "missing card definition {}",
                        card.card_id.as_str()
                    )));
                }
                validate_card_layout(&card.layout, issues);
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
            issues.push(diagnostic(format_args!(
                "card layout {label} span must be between 1 and 12"
            )));
        }
    }
}

fn diagnostic(arguments: std::fmt::Arguments<'_>) -> String {
    struct Counter(usize);
    impl Write for Counter {
        fn write_str(&mut self, value: &str) -> std::fmt::Result {
            self.0 = self.0.checked_add(value.len()).ok_or(std::fmt::Error)?;
            Ok(())
        }
    }
    let mut count = Counter(0);
    count
        .write_fmt(arguments)
        .expect("document diagnostic length overflows");
    let mut text = String::with_capacity(count.0);
    text.write_fmt(arguments)
        .expect("document diagnostic formatting failed");
    text
}

/* === Tests === */
