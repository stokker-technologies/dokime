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
use super::{DocumentDefinition, DocumentPayload, DocumentResolveInput};

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
    pub layout_depth: usize,
    pub unavailable_reason_bytes: usize,
    pub maximum_plot_points: usize,
}

pub(crate) struct SharedBindings<'a> {
    selection: &'a DocumentSelection,
    data: BorrowedData<'a>,
    bytes: usize,
    limits: DocumentLimits,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DocumentLimit {
    Bytes { limit: usize, reached: usize },
    Depth { limit: usize, reached: usize },
    Encoding,
}

/* === Traits === */

/* === Implementations === */

impl Default for DocumentLimits {
    fn default() -> Self {
        Self {
            source_bytes: 16777216,
            depth: 32,
            expanded_nodes: 4096,
            materialized_bytes: 33554432,
            layout_depth: 12,
            unavailable_reason_bytes: 512,
            maximum_plot_points: 100000,
        }
    }
}

impl std::fmt::Display for DocumentLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bytes { limit, reached } => write!(f, "encoded_bytes exceeded (limit {limit}, reached {reached})"),
            Self::Depth { limit, reached } => write!(f, "value_depth exceeded (limit {limit}, reached {reached})"),
            Self::Encoding => f.write_str("document value cannot be encoded"),
        }
    }
}

impl std::error::Error for DocumentLimit {}

impl<'a> SharedBindings<'a> {
    pub(crate) fn new(selection: &'a DocumentSelection, data: BorrowedData<'a>, limits: DocumentLimits) -> crate::Result<Self> {
        let bytes = document_size(selection, limits.source_bytes, limits.depth).map_err(|_| super::DocumentFailure::Limit)?;
        Ok(Self { selection, data, bytes, limits })
    }

    fn validate(&self, definition: &DocumentDefinition) -> crate::Result<()> {
        let bytes = document_size(definition, self.limits.source_bytes, self.limits.depth.saturating_sub(1))
            .map_err(|_| super::DocumentFailure::Limit)?;
        let total = self.bytes.checked_add(b",\"definition\":".len()).and_then(|total| total.checked_add(bytes));
        if total.is_none_or(|total| total > self.limits.source_bytes) {
            return Err(super::DocumentFailure::Limit.into());
        }
        Ok(())
    }

    /// Captures the definition; repeats resolved ahead of it, if given, expand
    /// over those values instead of over this capture's own rows.
    pub(crate) fn capture(
        &self, definition: &'a DocumentDefinition, repeats: Option<&super::resolve::RepeatValues>,
    ) -> crate::Result<crate::document::capture::Capture> {
        self.validate(definition)?;
        let input = BorrowedInput {
            definition,
            selection: self.selection,
            data: self.data,
        };
        let captured = super::resolve::capture_borrowed(input, self.limits, repeats)?;
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
        failure: Option<DocumentLimit>,
    }
    impl Counter {
        fn refuse(&mut self, failure: DocumentLimit) -> std::io::Error {
            self.failure = Some(failure);
            std::io::Error::other(failure)
        }
    }
    impl Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.bytes = self.bytes.checked_add(bytes.len()).ok_or_else(|| {
                self.refuse(DocumentLimit::Bytes {
                    limit: self.max_bytes,
                    reached: usize::MAX,
                })
            })?;
            if self.bytes > self.max_bytes {
                return Err(self.refuse(DocumentLimit::Bytes {
                    limit: self.max_bytes,
                    reached: self.bytes,
                }));
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
                                return Err(self.refuse(DocumentLimit::Depth {
                                    limit: self.max_depth,
                                    reached: self.depth,
                                }));
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
    let mut counter = Counter {
        bytes: 0,
        depth: 0,
        max_bytes,
        max_depth,
        quoted: false,
        escaped: false,
        failure: None,
    };
    if serde_json::to_writer(&mut counter, value).is_err() {
        return Err(counter.failure.unwrap_or(DocumentLimit::Encoding));
    }
    Ok(counter.bytes)
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
    SharedBindings::new(input.selection, input.data, limits)?.capture(input.definition, None)
}

/* === Private Functions === */

pub(crate) fn card_size<T: Serialize + ?Sized>(value: &T, maximum: usize, depth: usize, card: &str) -> crate::Result<()> {
    let reached = document_size(value, usize::MAX, depth).map_err(|error| match error {
        DocumentLimit::Depth { limit, reached } => super::DocumentFailure::Budget {
            budget: "value_depth",
            limit,
            reached,
            card: card.to_owned(),
        },
        DocumentLimit::Bytes { reached, .. } => super::DocumentFailure::Budget {
            budget: "materialized_bytes",
            limit: maximum,
            reached,
            card: card.to_owned(),
        },
        DocumentLimit::Encoding => super::DocumentFailure::Evaluation {
            message: "document card cannot be encoded",
        },
    })?;
    if reached > maximum {
        return Err(super::DocumentFailure::Budget {
            budget: "materialized_bytes",
            limit: maximum,
            reached,
            card: card.to_owned(),
        }
        .into());
    }
    Ok(())
}

pub(crate) fn check_payload(payload: DocumentPayload, limits: DocumentLimits) -> crate::Result<DocumentPayload> {
    document_size(&payload, limits.materialized_bytes, limits.depth).map_err(|_| super::DocumentFailure::Limit)?;
    Ok(payload)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actual_limits() {
        use crate::document::DocumentCard;
        let mut definition = DocumentDefinition::new("bindings.structure", "Structure");
        for index in 0..150 {
            definition.cards.push(DocumentCard::text(format!("card.{index}"), "Card", "Contents").into());
        }
        let input = DocumentResolveInput {
            definition,
            selection: DocumentSelection::default(),
            data: Default::default(),
        };
        let empty = capture_document_borrowed(input.borrowed(), DocumentLimits::default()).unwrap();
        assert_eq!(empty.cards.len(), 150);
        let limits = DocumentLimits {
            expanded_nodes: 149,
            ..Default::default()
        };
        assert!(matches!(
            capture_document_borrowed(input.borrowed(), limits),
            Err(crate::Error::Document(super::super::DocumentFailure::Budget { budget: "expanded_nodes", .. }))
        ));

        let value = ("brackets {} []", vec![1u8, 2]);
        let bytes = serde_json::to_vec(&value).unwrap().len();
        match card_size(&value, bytes - 1, 32, "fixture.output").unwrap_err() {
            crate::Error::Document(super::super::DocumentFailure::Budget { budget, limit, reached, card }) => {
                assert_eq!((budget, limit, reached, card.as_str()), ("materialized_bytes", bytes - 1, bytes, "fixture.output"));
            }
            error => panic!("unexpected output failure {error}"),
        }
        match card_size(&value, bytes, 1, "fixture.output").unwrap_err() {
            crate::Error::Document(super::super::DocumentFailure::Budget { budget, limit, reached, card }) => {
                assert_eq!((budget, limit, reached, card.as_str()), ("value_depth", 1, 2, "fixture.output"));
            }
            error => panic!("unexpected depth failure {error}"),
        }
    }

    #[test]
    fn time_between_repeats_are_refused_naming_their_section() {
        use crate::document::{DocumentFailure, DocumentNode, DocumentSection, DocumentSectionNode, DocumentSectionRepeat};
        use crate::expressions::{record, telemetry};
        use dokime_domain::model::identity::{Identifier, Metadata};

        // Authored Through The Builder: Elapsed Time Is The Repeated Value.
        let mut authored = DocumentDefinition::new("repeats.authored", "Authored");
        let mut laps = DocumentSection::new("Laps");
        laps.id("fixture.laps");
        laps.each(record("fixture.end").time_between("fixture.start"), "Lap {value}");
        authored.section(laps);

        // Stored Or Wire Layout: Elapsed Time Orders A Row-Local Value.
        let section = Identifier::new("fixture.channels").unwrap();
        let mut stored = DocumentDefinition::new("repeats.stored", "Stored");
        stored.layout.nodes.push(DocumentNode::Section(Box::new(DocumentSectionNode {
            node_id: section.clone(),
            metadata: Metadata::new(section, "Channels", None),
            repeat: Some(DocumentSectionRepeat {
                value: telemetry("wifi", "channel"),
                label_template: "Channel {value}".into(),
                id_template: None,
                order_by: Some(record("fixture.end").time_between("fixture.start")),
                filters: Vec::new(),
            }),
            children: Vec::new(),
        })));

        for (definition, section) in [(authored, "fixture.laps"), (stored, "fixture.channels")] {
            let input = DocumentResolveInput {
                definition,
                selection: DocumentSelection::default(),
                data: Default::default(),
            };
            let captured = capture_document_borrowed(input.borrowed(), DocumentLimits::default()).map(|_| ());
            let resolved = crate::document::resolve_document(input, DocumentLimits::default()).map(|_| ());
            for result in [captured, resolved] {
                match result {
                    Err(crate::Error::Document(DocumentFailure::CorrelatedRepeat { section: refused })) => {
                        assert_eq!(refused.as_str(), section)
                    }
                    other => panic!("expected the repeat of {section} to be refused, got {other:?}"),
                }
            }
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
