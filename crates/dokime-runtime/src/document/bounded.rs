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
use std::io::Write;

// Third Party Imports
use serde::Serialize;

// Local Imports
use super::input::{BorrowedData, BorrowedInput, DocumentSelection};
use super::{DocumentDefinition, DocumentNode, DocumentPayload, DocumentResolveInput};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Copy, Debug)]
pub struct DocumentLimits {
    pub source_bytes: usize,
    pub depth: usize,
    pub expanded_nodes: usize,
    pub materialized_bytes: usize,
    pub evaluation_work: usize,
    pub layout_depth: usize,
    pub unavailable_reason_bytes: usize,
    pub maximum_plot_points: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DocumentLimit;

pub(crate) struct SharedBindings<'a> {
    selection: &'a DocumentSelection,
    data: BorrowedData<'a>,
    bytes: usize,
    rows: usize,
    limits: DocumentLimits,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Default for DocumentLimits {
    fn default() -> Self {
        Self {
            source_bytes: 16777216,
            depth: 32,
            expanded_nodes: 4096,
            materialized_bytes: 33554432,
            evaluation_work: 256000000,
            layout_depth: 12,
            unavailable_reason_bytes: 512,
            maximum_plot_points: 100000,
        }
    }
}

impl std::fmt::Display for DocumentLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("document source, expansion, or evaluation budget exceeded")
    }
}

impl std::error::Error for DocumentLimit {}

impl<'a> SharedBindings<'a> {
    pub(crate) fn new(selection: &'a DocumentSelection, data: BorrowedData<'a>, limits: DocumentLimits) -> crate::Result<Self> {
        #[derive(Serialize)]
        struct Shared<'a> {
            selection: &'a DocumentSelection,
            data: BorrowedData<'a>,
        }
        let bytes =
            document_size(&Shared { selection, data }, limits.source_bytes, limits.depth).map_err(|_| super::DocumentFailure::Limit)?;
        let rows =
            data.telemetry.len().saturating_add(data.context.len()).saturating_add(data.records.len()).saturating_add(data.artifacts.len());
        Ok(Self { selection, data, bytes, rows, limits })
    }

    pub(crate) fn accounting(&self) -> (usize, usize) {
        (self.bytes, self.rows)
    }

    pub(crate) fn validated(
        selection: &'a DocumentSelection, data: BorrowedData<'a>, limits: DocumentLimits, bytes: usize, rows: usize,
    ) -> Self {
        Self { selection, data, bytes, rows, limits }
    }

    fn validate(&self, definition: &DocumentDefinition) -> crate::Result<()> {
        let bytes = document_size(definition, self.limits.source_bytes, self.limits.depth.saturating_sub(1))
            .map_err(|_| super::DocumentFailure::Limit)?;
        let total = self.bytes.checked_add(b",\"definition\":".len()).and_then(|total| total.checked_add(bytes));
        if total.is_none_or(|total| total > self.limits.source_bytes) {
            return Err(super::DocumentFailure::Limit.into());
        }
        preflight_definition(definition, self.rows, self.limits, bytes).map_err(|_| super::DocumentFailure::Limit)?;
        let context_work = bytes.checked_mul(self.data.context.len()).and_then(|work| work.checked_mul(self.rows.max(1)));
        if context_work.is_none_or(|work| work > self.limits.evaluation_work) {
            return Err(super::DocumentFailure::Limit.into());
        }
        Ok(())
    }

    pub(crate) fn capture(&self, definition: &'a DocumentDefinition) -> crate::Result<crate::document::capture::Capture> {
        self.validate(definition)?;
        let input = BorrowedInput { definition, selection: self.selection, data: self.data };
        let captured = super::resolve::capture_borrowed(input, self.limits)?;
        document_size(&captured, self.limits.materialized_bytes, self.limits.depth).map_err(|_| super::DocumentFailure::Limit)?;
        Ok(captured)
    }
}

/* === Public Functions === */

pub fn document_size<T: Serialize + ?Sized>(value: &T, max_bytes: usize, max_depth: usize) -> Result<usize, DocumentLimit> {
    struct Counter {
        bytes: usize,
        depth: usize,
        max_bytes: usize,
        max_depth: usize,
        quoted: bool,
        escaped: bool,
    }
    impl Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.bytes = self.bytes.checked_add(bytes.len()).ok_or_else(refusal)?;
            if self.bytes > self.max_bytes {
                return Err(refusal());
            }
            for byte in bytes {
                if self.quoted {
                    if self.escaped {
                        self.escaped = false;
                    } else if *byte == b'\\' {
                        self.escaped = true;
                    } else if *byte == b'"' {
                        self.quoted = false;
                    }
                } else {
                    match byte {
                        b'"' => self.quoted = true,
                        b'{' | b'[' => {
                            self.depth += 1;
                            if self.depth > self.max_depth {
                                return Err(refusal());
                            }
                        }
                        b'}' | b']' => self.depth -= 1,
                        _ => {}
                    }
                }
            }
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    fn refusal() -> std::io::Error {
        std::io::Error::other(DocumentLimit)
    }
    let mut counter = Counter { bytes: 0, depth: 0, max_bytes, max_depth, quoted: false, escaped: false };
    serde_json::to_writer(&mut counter, value).map_err(|_| DocumentLimit)?;
    Ok(counter.bytes)
}

pub fn preflight_document(definition: &DocumentDefinition, telemetry_rows: usize, limits: DocumentLimits) -> Result<usize, DocumentLimit> {
    // Encoded Inspection Bounds Depth And Evaluation Work; Native Graph Admission
    // Accounts Actual Capacities, Typed Slots And Nested Field Allocations.
    let bytes = document_size(definition, limits.source_bytes, limits.depth)?;
    preflight_definition(definition, telemetry_rows, limits, bytes)
}

pub(crate) fn validate_native_resolution(input: &BorrowedInput<'_>, limits: DocumentLimits) -> crate::Result<()> {
    SharedBindings::new(input.selection, input.data, limits)?.validate(input.definition)
}

pub(super) fn resolve_bounded(
    input: DocumentResolveInput, limits: DocumentLimits, materialize_plots: bool,
) -> crate::Result<DocumentPayload> {
    validate_native_resolution(&input.borrowed(), limits)?;
    let payload = super::resolve::resolve(input, materialize_plots, limits)?;
    check_payload(payload, limits)
}

pub(crate) fn capture_document_borrowed(
    input: BorrowedInput<'_>, limits: DocumentLimits,
) -> crate::Result<crate::document::capture::Capture> {
    SharedBindings::new(input.selection, input.data, limits)?.capture(input.definition)
}

/* === Private Functions === */

fn preflight_definition(
    definition: &DocumentDefinition, telemetry_rows: usize, limits: DocumentLimits, bytes: usize,
) -> Result<usize, DocumentLimit> {
    fn nodes(items: &[DocumentNode], samples: usize, depth: usize, limit: usize) -> Result<(usize, usize), DocumentLimit> {
        if depth == 0 {
            return Err(DocumentLimit);
        }
        let mut constructed = 0usize;
        let mut declared = 0usize;
        for node in items {
            let (amount, source) = match node {
                DocumentNode::Card(_) => (1, 1),
                DocumentNode::Section(section) => {
                    let (children, descendants) = nodes(&section.children, samples, depth - 1, limit)?;
                    let source = descendants.checked_add(1).ok_or(DocumentLimit)?;
                    let amount = children.checked_add(1).ok_or(DocumentLimit)?;
                    let amount = if section.repeat.is_some() {
                        // Expansion Copies Every Declared Descendant Before Its
                        // Nested Repeats Materialize Their Own Children.
                        amount.checked_add(descendants).and_then(|nodes| nodes.checked_mul(samples.max(1))).ok_or(DocumentLimit)?
                    } else {
                        amount
                    };
                    (amount, source)
                }
            };
            constructed = constructed.checked_add(amount).filter(|count| *count <= limit).ok_or(DocumentLimit)?;
            declared = declared.checked_add(source).ok_or(DocumentLimit)?;
        }
        Ok((constructed, declared))
    }
    let mut count = definition
        .sections
        .iter()
        .try_fold(definition.cards.len(), |count, section| count.checked_add(section.cards.len()).ok_or(DocumentLimit))?;
    let mut repeats = definition.sections.iter().any(|section| section.repeat.is_some());
    repeats |= has_repeat(&definition.layout.nodes);
    count =
        count.checked_add(nodes(&definition.layout.nodes, telemetry_rows, limits.depth, limits.expanded_nodes)?.0).ok_or(DocumentLimit)?;
    for section in &definition.sections {
        let multiplier = if section.repeat.is_some() { telemetry_rows.max(1) } else { 1 };
        count = count
            .checked_add(
                section
                    .cards
                    .len()
                    .checked_add(1)
                    .and_then(|nodes| if section.repeat.is_some() { nodes.checked_add(section.cards.len()) } else { Some(nodes) })
                    .and_then(|nodes| nodes.checked_mul(multiplier))
                    .ok_or(DocumentLimit)?,
            )
            .ok_or(DocumentLimit)?;
    }
    if definition.layout.nodes.is_empty() && definition.sections.is_empty() {
        // Normalization Creates One Implicit Placement Per Card Definition.
        count = count.checked_add(definition.cards.len()).ok_or(DocumentLimit)?;
    }
    if count > limits.expanded_nodes
        || bytes
            .checked_mul(telemetry_rows.max(1))
            .and_then(|work| work.checked_add(super::binding::evaluation_work(definition, telemetry_rows, count)))
            .and_then(|work| work.checked_add(super::binding::correlation_work(definition, telemetry_rows, count)))
            .is_none_or(|n| n > limits.evaluation_work)
        || (repeats
            && telemetry_rows
                .checked_mul(telemetry_rows)
                .and_then(|n| n.checked_mul(count.max(1)))
                .is_none_or(|n| n > limits.evaluation_work))
    {
        return Err(DocumentLimit);
    }
    Ok(count)
}

pub fn resolve_document_bounded(input: DocumentResolveInput, limits: DocumentLimits) -> crate::Result<DocumentPayload> {
    resolve_bounded(input, limits, false)
}

pub(crate) fn check_payload(payload: DocumentPayload, limits: DocumentLimits) -> crate::Result<DocumentPayload> {
    document_size(&payload, limits.materialized_bytes, limits.depth).map_err(|_| super::DocumentFailure::Limit)?;
    Ok(payload)
}

fn has_repeat(nodes: &[DocumentNode]) -> bool {
    nodes.iter().any(|node| match node {
        DocumentNode::Card(_) => false,
        DocumentNode::Section(section) => section.repeat.is_some() || has_repeat(&section.children),
    })
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_bindings_preserve_the_exact_source_boundary() {
        let input = DocumentResolveInput {
            definition: DocumentDefinition::new("bindings.boundary", "Boundary"),
            selection: DocumentSelection::default(),
            data: Default::default(),
        };
        let borrowed = input.borrowed();
        let length = document_size(&borrowed, usize::MAX, 32).unwrap();
        for source_bytes in [length - 1, length] {
            let limits = DocumentLimits { source_bytes, ..Default::default() };
            let result =
                SharedBindings::new(borrowed.selection, borrowed.data, limits).and_then(|bindings| bindings.validate(borrowed.definition));
            assert_eq!(result.is_ok(), source_bytes == length);
        }
    }

    #[test]
    fn normalization_preserves_placements() {
        use crate::document::{DocumentCard, DocumentSection};
        let mut definition = DocumentDefinition::new("normalization", "Normalization");
        definition.cards.push(DocumentCard::text("shared", "Original", "before").into());
        let mut section = DocumentSection::new("Legacy");
        section.cards.push(DocumentCard::text("shared", "Replacement", "after"));
        section.cards.push(DocumentCard::text("added", "Added", "body"));
        let replacement = crate::document::DocumentCardDefinition::from(section.cards[0].clone());
        definition.sections.push(section);

        let (cards, layout) = definition.normalized_parts();
        assert_eq!(cards.len(), 2);
        assert_eq!(cards[0], replacement);
        assert_eq!(cards[1].metadata.identifier.as_str(), "added");
        assert_eq!(layout.nodes.len(), 1);

        definition.sections.clear();
        let (cards, layout) = definition.normalized_parts();
        assert_eq!(layout.nodes.len(), cards.len());
    }
}
