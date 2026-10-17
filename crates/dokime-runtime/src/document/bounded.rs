/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod cards;
mod normalization;
mod repetition;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::io::Write;
// Third Party Imports
use harmos::MemorySize;
use serde::Serialize;
// Local Imports
use super::input::BorrowedInput;
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DocumentLimit;

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Default for DocumentLimits {
    fn default() -> Self {
        Self {
            source_bytes: 16 * 1024 * 1024,
            depth: 32,
            expanded_nodes: 4096,
            materialized_bytes: 32 * 1024 * 1024,
            evaluation_work: 256_000_000,
        }
    }
}

impl std::fmt::Display for DocumentLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("document source, expansion, or evaluation budget exceeded")
    }
}

impl std::error::Error for DocumentLimit {}

/* === Public Functions === */
pub fn document_size<T: Serialize + ?Sized>(
    value: &T,
    max_bytes: usize,
    max_depth: usize,
) -> Result<usize, DocumentLimit> {
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
    let mut counter = Counter {
        bytes: 0,
        depth: 0,
        max_bytes,
        max_depth,
        quoted: false,
        escaped: false,
    };
    serde_json::to_writer(&mut counter, value).map_err(|_| DocumentLimit)?;
    Ok(counter.bytes)
}

pub fn preflight_document(
    definition: &DocumentDefinition,
    telemetry_rows: usize,
    limits: DocumentLimits,
) -> Result<usize, DocumentLimit> {
    // Encoded Inspection Bounds Depth And Evaluation Work; Native Graph Admission
    // Accounts Actual Capacities, Typed Slots And Nested Field Allocations.
    let bytes = document_size(definition, limits.source_bytes, limits.depth)?;
    if definition.memory_size() > limits.source_bytes {
        return Err(DocumentLimit);
    }
    fn nodes(
        items: &[DocumentNode],
        samples: usize,
        depth: usize,
        limit: usize,
    ) -> Result<(usize, usize), DocumentLimit> {
        if depth == 0 {
            return Err(DocumentLimit);
        }
        let mut constructed = 0usize;
        let mut declared = 0usize;
        for node in items {
            let (amount, source) = match node {
                DocumentNode::Card(_) => (1, 1),
                DocumentNode::Section(section) => {
                    let (children, descendants) =
                        nodes(&section.children, samples, depth - 1, limit)?;
                    let source = descendants.checked_add(1).ok_or(DocumentLimit)?;
                    let amount = children.checked_add(1).ok_or(DocumentLimit)?;
                    let amount = if section.repeat.is_some() {
                        // Expansion Copies Every Declared Descendant Before Its
                        // Nested Repeats Materialize Their Own Children.
                        amount
                            .checked_add(descendants)
                            .and_then(|nodes| nodes.checked_mul(samples.max(1)))
                            .ok_or(DocumentLimit)?
                    } else {
                        amount
                    };
                    (amount, source)
                }
            };
            constructed = constructed
                .checked_add(amount)
                .filter(|count| *count <= limit)
                .ok_or(DocumentLimit)?;
            declared = declared.checked_add(source).ok_or(DocumentLimit)?;
        }
        Ok((constructed, declared))
    }
    let mut count = definition
        .sections
        .iter()
        .try_fold(definition.cards.len(), |count, section| {
            count.checked_add(section.cards.len()).ok_or(DocumentLimit)
        })?;
    let mut repeats = definition
        .sections
        .iter()
        .any(|section| section.repeat.is_some());
    repeats |= has_repeat(&definition.layout.nodes);
    count = count
        .checked_add(
            nodes(
                &definition.layout.nodes,
                telemetry_rows,
                limits.depth,
                limits.expanded_nodes,
            )?
            .0,
        )
        .ok_or(DocumentLimit)?;
    for section in &definition.sections {
        let multiplier = if section.repeat.is_some() {
            telemetry_rows.max(1)
        } else {
            1
        };
        count = count
            .checked_add(
                section
                    .cards
                    .len()
                    .checked_add(1)
                    .and_then(|nodes| {
                        if section.repeat.is_some() {
                            nodes.checked_add(section.cards.len())
                        } else {
                            Some(nodes)
                        }
                    })
                    .and_then(|nodes| nodes.checked_mul(multiplier))
                    .ok_or(DocumentLimit)?,
            )
            .ok_or(DocumentLimit)?;
    }
    if definition.layout.nodes.is_empty() && definition.sections.is_empty() {
        // Normalization Creates One Implicit Placement Per Card Definition.
        count = count
            .checked_add(definition.cards.len())
            .ok_or(DocumentLimit)?;
    }
    if count > limits.expanded_nodes
        // One normalized graph plus at most one graph-sized clone per expanded
        // node. Label substitution and evidence-dependent payloads are admitted
        // separately below, before materialization.
        || normalization::memory(definition)
            .checked_mul(count.saturating_add(1))
            .is_none_or(|n| n > limits.materialized_bytes)
        || bytes
            .checked_mul(telemetry_rows.max(1))
            .and_then(|work| {
                work.checked_add(super::binding::evaluation_work(
                    definition,
                    telemetry_rows,
                    count,
                ))
            })
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

pub fn resolve_document_bounded(
    input: DocumentResolveInput,
    limits: DocumentLimits,
) -> crate::Result<DocumentPayload> {
    preflight_input(&input.borrowed(), limits, false)?;
    if input.memory_size() > limits.source_bytes {
        return Err(super::DocumentFailure::Limit.into());
    }
    let payload = super::resolve::resolve_current(input)?;
    check_payload(payload, limits)
}

pub(crate) fn capture_document_borrowed(
    input: BorrowedInput<'_>,
    limits: DocumentLimits,
) -> crate::Result<crate::document::capture::Capture> {
    capture_memory(&input, limits)?;
    let captured = super::resolve::capture_borrowed(input)?;
    if captured.memory_size() > limits.materialized_bytes {
        return Err(super::DocumentFailure::Limit.into());
    }
    document_size(&captured, limits.materialized_bytes, limits.depth)
        .map_err(|_| super::DocumentFailure::Limit)?;
    Ok(captured)
}

pub(crate) fn capture_memory(
    input: &BorrowedInput<'_>,
    limits: DocumentLimits,
) -> crate::Result<usize> {
    let construction = preflight_input(input, limits, true)?;
    let count = preflight_document(
        input.definition,
        input
            .data
            .telemetry
            .len()
            .saturating_add(input.data.context.len())
            .saturating_add(input.data.records.len())
            .saturating_add(input.data.artifacts.len()),
        limits,
    )
    .map_err(|_| super::DocumentFailure::Limit)?;
    let declared = input.definition.cards.len().saturating_add(
        input
            .definition
            .sections
            .iter()
            .map(|section| section.cards.len())
            .sum::<usize>(),
    );
    // Construction Bounds Captured Strings, Filters, Results And Declaration.
    // Explicit Registries Include Unplaced Sources And Pruned Repeat Selections.
    let registries =
        harmos::btree_map_memory::<dokime_domain::model::identity::Identifier, dokime_domain::model::identity::Identifier>(
            count.saturating_add(declared),
        )
        .saturating_add(harmos::btree_map_memory::<
            dokime_domain::model::identity::Identifier,
            Vec<crate::document::capture::RepeatInstance>,
        >(count))
        .saturating_add(harmos::btree_map_memory::<
            dokime_domain::model::identity::Identifier,
            super::capture::SectionBinding,
        >(count))
        .saturating_add(
            count.saturating_mul(
                std::mem::size_of::<crate::document::capture::RepeatInstance>()
                    .saturating_add(std::mem::size_of::<crate::document::capture::CardBinding>()),
            ),
        );
    // Construction Already Covers Every Owned Copy The Resolver Makes; Capture
    // Adds Only Its Declaration And Registries, Not A Second Construction.
    construction
        .checked_add(input.definition.memory_size())
        .and_then(|bytes| bytes.checked_add(registries))
        .and_then(|bytes| {
            count
                .checked_mul(super::payload::MAX_UNAVAILABLE_REASON_BYTES)
                .and_then(|sections| bytes.checked_add(sections))
        })
        .and_then(|bytes| {
            bytes.checked_add(super::binding::scratch_memory(
                input.definition,
                input
                    .data
                    .telemetry
                    .len()
                    .saturating_add(input.data.context.len())
                    .saturating_add(input.data.records.len())
                    .saturating_add(input.data.artifacts.len()),
            ))
        })
        .filter(|bytes| *bytes <= limits.materialized_bytes)
        .ok_or(super::DocumentFailure::Limit.into())
}

/* === Private Functions === */
fn preflight_input(
    input: &BorrowedInput<'_>,
    limits: DocumentLimits,
    borrowed: bool,
) -> crate::Result<usize> {
    let source_bytes = document_size(input, limits.source_bytes, limits.depth)
        .map_err(|_| super::DocumentFailure::Limit)?;
    if input.memory_size() > limits.source_bytes {
        return Err(super::DocumentFailure::Limit.into());
    }
    let source_copies = if borrowed {
        // Definitions And Selection Are Owned; Scoped Evidence Stays Borrowed.
        input
            .definition
            .memory_size()
            .saturating_add(input.selection.memory_size())
    } else {
        source_bytes.saturating_mul(3)
    };
    let count = preflight_document(
        input.definition,
        input
            .data
            .telemetry
            .len()
            .saturating_add(input.data.context.len())
            .saturating_add(input.data.records.len())
            .saturating_add(input.data.artifacts.len()),
        limits,
    )
    .map_err(|_| super::DocumentFailure::Limit)?;
    let context_work = document_size(input.definition, limits.source_bytes, limits.depth)
        .map_err(|_| super::DocumentFailure::Limit)?
        .saturating_mul(input.data.context.len())
        .saturating_mul(
            input
                .data
                .telemetry
                .len()
                .saturating_add(input.data.context.len())
                .saturating_add(input.data.records.len())
                .saturating_add(input.data.artifacts.len())
                .max(1),
        );
    if context_work > limits.evaluation_work {
        return Err(super::DocumentFailure::Limit.into());
    }
    // Current Plot Inputs Use An Empty Sample Slice; Summary Values Are Counted Below.
    let normalized_bytes = normalization::memory(input.definition);
    let largest_value = input
        .data
        .telemetry
        .iter()
        .flat_map(|sample| sample.values.values())
        .map(MemorySize::memory_size)
        .max()
        .unwrap_or(std::mem::size_of::<dokime_domain::model::values::Value>())
        .max(input.data.context.largest_value())
        .max(
            input
                .data
                .records
                .iter()
                .map(|record| {
                    record
                        .entry
                        .details
                        .memory_size()
                        .saturating_add(std::mem::size_of::<dokime_domain::model::values::Value>())
                })
                .max()
                .unwrap_or(0),
        )
        .max(
            input
                .data
                .artifacts
                .iter()
                .map(|artifact| {
                    artifact
                        .memory_size()
                        .saturating_add(std::mem::size_of::<dokime_domain::model::values::Value>())
                })
                .max()
                .unwrap_or(0),
        );
    let (largest_card, declared_cards) = input
        .definition
        .cards
        .iter()
        .map(|card| {
            cards::memory(
                &card.metadata,
                &card.filters,
                &card.body,
                &input.selection.scope,
                largest_value,
            )
        })
        .chain(
            input
                .definition
                .sections
                .iter()
                .flat_map(|section| section.cards.iter())
                .map(|card| {
                    cards::memory(
                        &card.metadata,
                        &card.filters,
                        &card.body,
                        &input.selection.scope,
                        largest_value,
                    )
                }),
        )
        .fold((0usize, 0usize), |(largest, total), bytes| {
            (largest.max(bytes), total.saturating_add(bytes))
        });
    // Both Registries Retain At Most One Entry Per Constructed Card.
    let registries =
        harmos::btree_map_memory::<dokime_domain::model::identity::Identifier, super::DocumentCardDefinition>(count)
            .checked_add(harmos::btree_map_memory::<dokime_domain::model::identity::Identifier, usize>(
                count,
            ))
            .ok_or(super::DocumentFailure::Limit)?;
    let repeats = input
        .definition
        .sections
        .iter()
        .any(|section| section.repeat.is_some())
        || has_repeat(&input.definition.layout.nodes);
    // Static Card Payloads Are Cached Once Per ID; Repeats Create New IDs.
    let graph = if repeats {
        normalized_bytes
            .checked_add(largest_card)
            .and_then(|bytes| bytes.checked_mul(count.saturating_add(1)))
    } else {
        normalized_bytes.checked_add(declared_cards)
    };
    let construction = graph
        .and_then(|bytes| bytes.checked_mul(8))
        .and_then(|bytes| bytes.checked_add(source_copies))
        .and_then(|bytes| bytes.checked_add(registries))
        .and_then(|bytes| {
            bytes.checked_add(super::binding::scratch_memory(
                input.definition,
                input
                    .data
                    .telemetry
                    .len()
                    .saturating_add(input.data.context.len())
                    .saturating_add(input.data.records.len())
                    .saturating_add(input.data.artifacts.len()),
            ))
        })
        .and_then(|bytes| {
            bytes.checked_add(repetition::memory(
                input.definition,
                super::binding::Evidence {
                    records: input.data.records,
                    artifacts: input.data.artifacts,
                    telemetry: input.data.telemetry,
                    context: input.data.context,
                    selection: Some(input.selection),
                },
                count,
            ))
        })
        .and_then(|bytes| {
            let markers = if borrowed {
                // Capture Carries Only The Count; Owned Resolution Builds Markers.
                0
            } else {
                super::markers::selection_memory(input.data.records, input.selection)
            };
            bytes.checked_add(markers)
        })
        .filter(|bytes| *bytes <= limits.materialized_bytes)
        .ok_or(super::DocumentFailure::Limit)?;
    if construction > limits.materialized_bytes {
        return Err(super::DocumentFailure::Limit.into());
    }
    Ok(construction)
}

fn check_payload(
    payload: DocumentPayload,
    limits: DocumentLimits,
) -> crate::Result<DocumentPayload> {
    if payload.memory_size() > limits.materialized_bytes {
        return Err(super::DocumentFailure::Limit.into());
    }
    document_size(&payload, limits.materialized_bytes, limits.depth)
        .map_err(|_| super::DocumentFailure::Limit)?;
    Ok(payload)
}

fn has_repeat(nodes: &[DocumentNode]) -> bool {
    nodes.iter().any(|node| match node {
        DocumentNode::Card(_) => false,
        DocumentNode::Section(section) => section.repeat.is_some() || has_repeat(&section.children),
    })
}

/* === Tests === */
