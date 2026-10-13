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
use std::{borrow::Cow, cmp::Ordering, collections::BTreeMap};
// Third Party Imports

// Local Imports
use super::binding::{self, Evidence, Row, row as expression_value};
use crate::artifacts::media_kind_for;
use crate::core::{Identifier, Metadata, Value};
use crate::document::capture::{
    Capture, CardBinding, CardBindingResult, RepeatInstance, SectionBinding,
};
use crate::document::{
    definition::{
        DocumentCard, DocumentCardBody, DocumentCardDefinition, DocumentCardPlacement,
        DocumentFilter, DocumentLayout, DocumentNode, DocumentSectionNode, DocumentSectionRepeat,
        FilterOperator, PlotDefinition, TelemetrySummaryCondition, TelemetrySummaryDefinition,
        TelemetrySummaryTileDefinition,
    },
    error::{DocumentFailure, Result},
    input::{Artifacts, BorrowedData, BorrowedInput, DocumentResolveInput},
    markers::scope_matches,
    payload::{
        ArtifactListDescriptor, DocumentCardBodyPayload, DocumentCardPayload,
        DocumentCardPlacementPayload, DocumentLayoutPayload, DocumentNodePayload, DocumentPayload,
        DocumentSectionNodePayload, DocumentWarning, TelemetrySummaryGroupPayload,
        TelemetrySummaryTilePayload,
    },
    plot_inputs::{plot_descriptor_for_definition, plot_input_for_definition},
};
use crate::expressions::{DataExpression, DataSource};
use dokime_contract::document::validation::validate_normalized_parts;

/* === Constants === */
const VALUE_PLACEHOLDER: &str = "{value}";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct DocumentMaterializer<'a> {
    capture: Option<&'a mut CaptureBindings>,
    artifact_lists: &'a mut Vec<ArtifactListDescriptor>,
    card_definitions: BTreeMap<Identifier, DocumentCardDefinition>,
    card_payload_ids: BTreeMap<Identifier, usize>,
    card_payloads: Vec<DocumentCardPayload>,
    input_artifacts: Artifacts<'a>,
    markers_len: usize,
    materialize_plots: bool,
    plot_descriptors: &'a mut Vec<crate::document::definition::PlotDescriptor>,
    plot_inputs: &'a mut Vec<crate::document::definition::PlotInput>,
    selection: &'a crate::document::input::DocumentSelection,
    evidence: Evidence<'a>,
}

#[derive(Clone)]
struct RepeatValue<'a> {
    value: Cow<'a, Value>,
    order: Option<Cow<'a, Value>>,
    first_index: usize,
}

#[derive(Default)]
struct CaptureBindings {
    cards: Vec<CardBinding>,
    identity_collision: bool,
    source_ids: BTreeMap<Identifier, Identifier>,
    repeats: BTreeMap<Identifier, Vec<RepeatInstance>>,
    sections: BTreeMap<Identifier, SectionBinding>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl DocumentMaterializer<'_> {
    fn capture_nodes(&mut self, nodes: Vec<DocumentNode>) -> Result<bool> {
        let mut present = false;
        for node in nodes {
            match node {
                DocumentNode::Card(placement) => {
                    self.ensure_card_payload(&placement.card_id)?;
                    present = true;
                }
                DocumentNode::Section(section) => {
                    if let Some(repeat) = &section.repeat
                        && let Err(error) = binding::validate_repeat(repeat)
                    {
                        let reason = unavailable_reason(error)?;
                        let capture = self.capture.as_mut().expect("capture assembly is active");
                        if capture
                            .repeats
                            .insert(section.node_id.clone(), Vec::new())
                            .is_some()
                            || capture
                                .sections
                                .insert(
                                    section.node_id.clone(),
                                    SectionBinding {
                                        metadata: section.metadata,
                                        unavailable: Some(reason),
                                    },
                                )
                                .is_some()
                        {
                            capture.identity_collision = true;
                        }
                        present = true;
                        continue;
                    }
                    for section in self.expanded_section_nodes(*section) {
                        if !self.capture_nodes(section.children)? {
                            continue;
                        }
                        let capture = self.capture.as_mut().expect("capture assembly is active");
                        if capture
                            .sections
                            .insert(
                                section.node_id,
                                SectionBinding {
                                    metadata: section.metadata,
                                    unavailable: None,
                                },
                            )
                            .is_some()
                        {
                            capture.identity_collision = true;
                        }
                        present = true;
                    }
                }
            }
        }
        Ok(present)
    }

    fn materialize_layout(&mut self, layout: DocumentLayout) -> Result<DocumentLayoutPayload> {
        Ok(DocumentLayoutPayload {
            nodes: self.materialize_nodes(layout.nodes)?,
        })
    }

    fn materialize_nodes(&mut self, nodes: Vec<DocumentNode>) -> Result<Vec<DocumentNodePayload>> {
        let mut output = Vec::new();
        for node in nodes {
            output.extend(self.materialize_node(node)?);
        }
        Ok(output)
    }

    fn materialize_node(&mut self, node: DocumentNode) -> Result<Vec<DocumentNodePayload>> {
        match node {
            DocumentNode::Section(section) => {
                let sections = self.expanded_section_nodes(*section);
                let mut output = Vec::new();
                for section in sections {
                    if let Some(section) = self.materialize_section_node(section)? {
                        output.push(section);
                    }
                }
                Ok(output)
            }
            DocumentNode::Card(placement) => Ok(self
                .materialize_card_placement(placement)?
                .into_iter()
                .collect()),
        }
    }

    fn materialize_section_node(
        &mut self,
        section: DocumentSectionNode,
    ) -> Result<Option<DocumentNodePayload>> {
        let children = self.materialize_nodes(section.children)?;
        if children.is_empty() {
            return Ok(None);
        }
        Ok(Some(DocumentNodePayload::Section(
            DocumentSectionNodePayload {
                node_id: section.node_id,
                metadata: section.metadata,
                children,
            },
        )))
    }

    fn materialize_card_placement(
        &mut self,
        placement: DocumentCardPlacement,
    ) -> Result<Option<DocumentNodePayload>> {
        if !self.card_definitions.contains_key(&placement.card_id) {
            return Ok(None);
        }
        self.ensure_card_payload(&placement.card_id)?;
        Ok(Some(DocumentNodePayload::Card(
            DocumentCardPlacementPayload {
                card_id: placement.card_id,
                layout: placement.layout,
            },
        )))
    }

    fn ensure_card_payload(&mut self, card_id: &Identifier) -> Result<()> {
        if self.card_payload_ids.contains_key(card_id) {
            return Ok(());
        }
        if let Some(capture) = self.capture.as_mut() {
            let card = self
                .card_definitions
                .get(card_id)
                .expect("declared card exists");
            let result = match &card.body {
                DocumentCardBody::TelemetrySummary { summary } => CardBindingResult::Summary {
                    groups: telemetry_summary_payload(summary, &card.filters, self.evidence)?,
                },
                DocumentCardBody::Plot { plot } => {
                    let input =
                        plot_input_for_definition(plot, &[], self.markers_len, self.selection);
                    CardBindingResult::Plot {
                        descriptor: Box::new(plot_descriptor_for_definition(
                            input,
                            plot.metadata.clone(),
                        )),
                    }
                }
                _ => CardBindingResult::Static,
            };
            let source_id = capture
                .source_ids
                .get(card_id)
                .expect("captured card source exists")
                .clone();
            let binding = CardBinding {
                source_id,
                metadata: card.metadata.clone(),
                result,
            };
            self.card_payload_ids
                .insert(card_id.clone(), capture.cards.len());
            capture.cards.push(binding);
            return Ok(());
        }
        let Some(card) = self.card_definitions.get(card_id).cloned() else {
            return Ok(());
        };
        let payload = self.card_payload_for_definition(&card)?;
        self.card_payload_ids
            .insert(card_id.clone(), self.card_payloads.len());
        self.card_payloads.push(payload);
        Ok(())
    }

    fn card_payload_for_definition(
        &mut self,
        card: &DocumentCardDefinition,
    ) -> Result<DocumentCardPayload> {
        let body = match &card.body {
            DocumentCardBody::Text { markdown } => DocumentCardBodyPayload::Text {
                markdown: markdown.clone(),
            },
            DocumentCardBody::TelemetrySummary { summary } => {
                DocumentCardBodyPayload::TelemetrySummary {
                    groups: telemetry_summary_payload(summary, &card.filters, self.evidence)?,
                }
            }
            DocumentCardBody::Plot { plot } => {
                let plot_input = plot_input_for_definition(
                    plot,
                    if self.materialize_plots {
                        self.evidence.telemetry
                    } else {
                        &[]
                    },
                    self.markers_len,
                    self.selection,
                );
                let descriptor =
                    plot_descriptor_for_definition(plot_input.clone(), plot.metadata.clone());
                if self.materialize_plots {
                    self.plot_inputs.push(plot_input);
                    self.plot_descriptors.push(descriptor.clone());
                }

                DocumentCardBodyPayload::Plot {
                    descriptor: Box::new(descriptor),
                }
            }
            DocumentCardBody::Table { rows } => {
                DocumentCardBodyPayload::Table { rows: rows.clone() }
            }
            DocumentCardBody::Steps => DocumentCardBodyPayload::Steps,
            DocumentCardBody::Criteria => DocumentCardBodyPayload::Criteria,
            DocumentCardBody::Traceability => DocumentCardBodyPayload::Traceability,
            DocumentCardBody::Notes => DocumentCardBodyPayload::Notes,
            DocumentCardBody::Records => DocumentCardBodyPayload::Records,
            DocumentCardBody::Artifacts => {
                let descriptor = ArtifactListDescriptor::from_statuses(
                    self.selection.scope.clone(),
                    self.input_artifacts
                        .iter()
                        .filter(|artifact| scope_matches(Some(&artifact.scope), self.selection))
                        .map(|artifact| artifact.status.clone()),
                );
                self.artifact_lists.push(descriptor.clone());

                DocumentCardBodyPayload::Artifacts {
                    descriptor: Box::new(descriptor),
                }
            }
            DocumentCardBody::Media => {
                let descriptor = ArtifactListDescriptor::from_statuses(
                    self.selection.scope.clone(),
                    self.input_artifacts
                        .iter()
                        .filter(|artifact| {
                            scope_matches(Some(&artifact.scope), self.selection)
                                && media_kind_for(
                                    &artifact.filename,
                                    artifact.content_type.as_deref(),
                                )
                                .is_some()
                        })
                        .map(|artifact| artifact.status.clone()),
                );
                self.artifact_lists.push(descriptor.clone());

                DocumentCardBodyPayload::Media {
                    descriptor: Box::new(descriptor),
                }
            }
            DocumentCardBody::Health => DocumentCardBodyPayload::Health,
            DocumentCardBody::TelemetryContext => DocumentCardBodyPayload::TelemetryContext,
        };

        Ok(DocumentCardPayload {
            metadata: card.metadata.clone(),
            body,
        })
    }

    fn expanded_section_nodes(&mut self, section: DocumentSectionNode) -> Vec<DocumentSectionNode> {
        let Some(repeat) = section.repeat.as_ref() else {
            return vec![section];
        };
        let values = repeat_values(repeat, self.evidence);
        if let Some(capture) = self.capture.as_mut()
            && capture
                .repeats
                .insert(section.node_id.clone(), Vec::with_capacity(values.len()))
                .is_some()
        {
            capture.identity_collision = true;
        }
        if values.is_empty() {
            return Vec::new();
        }
        let identifier_template = if let Some(template) = repeat.id_template.as_deref() {
            Cow::Borrowed(template)
        } else {
            let bytes = section
                .node_id
                .as_str()
                .len()
                .checked_add(1 + VALUE_PLACEHOLDER.len())
                .expect("repeat identifier template length overflows");
            let mut template = String::with_capacity(bytes);
            template.push_str(section.node_id.as_str());
            template.push('.');
            template.push_str(VALUE_PLACEHOLDER);
            Cow::Owned(template)
        };

        values
            .into_iter()
            .map(|repeat_value| {
                let suffix = identifier_segment(&value_label(&repeat_value.value));
                let label =
                    render_template(&repeat.label_template, &repeat.value, &repeat_value.value);
                let identifier =
                    render_template(&identifier_template, &repeat.value, &repeat_value.value);
                let repeat_filter = DocumentFilter {
                    expression: repeat.value.clone(),
                    operator: FilterOperator::Eq,
                    value: Some(repeat_value.value.into_owned()),
                };

                let node_id = Identifier::new(&identifier)
                    .unwrap_or_else(|_| identifier_segment_identifier(&identifier));
                if let Some(capture) = self.capture.as_mut() {
                    capture
                        .repeats
                        .get_mut(&section.node_id)
                        .expect("repeat selection exists")
                        .push(RepeatInstance {
                            suffix: suffix.clone(),
                            node_id: node_id.clone(),
                            filter: repeat_filter.clone(),
                        });
                }

                DocumentSectionNode {
                    node_id,
                    metadata: generated_metadata(&section.metadata, &identifier, label),
                    repeat: None,
                    children: section
                        .children
                        .iter()
                        .map(|node| self.repeated_layout_node(node, &repeat_filter, &suffix))
                        .collect(),
                }
            })
            .collect()
    }

    fn repeated_layout_node(
        &mut self,
        node: &DocumentNode,
        repeat_filter: &DocumentFilter,
        suffix: &str,
    ) -> DocumentNode {
        match node {
            DocumentNode::Card(placement) => {
                let Some(definition) = self.card_definitions.get(&placement.card_id).cloned()
                else {
                    return DocumentNode::Card(placement.clone());
                };
                let repeated = repeated_card(
                    DocumentCard {
                        metadata: definition.metadata,
                        layout: placement.layout.clone(),
                        filters: definition.filters,
                        body: definition.body,
                    },
                    repeat_filter,
                    suffix,
                );
                let repeated_id = repeated.metadata.identifier.clone();
                if let Some(capture) = self.capture.as_mut() {
                    let source = capture
                        .source_ids
                        .get(&placement.card_id)
                        .expect("authored card source exists")
                        .clone();
                    if capture
                        .source_ids
                        .insert(repeated_id.clone(), source.clone())
                        .is_some_and(|previous| previous != source)
                    {
                        capture.identity_collision = true;
                    }
                }
                let repeated = DocumentCardDefinition::from(repeated);
                if let Some(capture) = self.capture.as_mut()
                    && self
                        .card_definitions
                        .get(&repeated_id)
                        .is_some_and(|previous| previous != &repeated)
                {
                    capture.identity_collision = true;
                }
                self.card_definitions.insert(repeated_id.clone(), repeated);

                DocumentNode::Card(DocumentCardPlacement {
                    card_id: repeated_id,
                    layout: placement.layout.clone(),
                })
            }
            DocumentNode::Section(section) => {
                let identifier = suffixed_identifier(&section.node_id, suffix);
                DocumentNode::Section(Box::new(DocumentSectionNode {
                    node_id: Identifier::new(&identifier)
                        .unwrap_or_else(|_| identifier_segment_identifier(&identifier)),
                    metadata: generated_metadata(
                        &section.metadata,
                        &identifier,
                        section.metadata.label.clone(),
                    ),
                    repeat: section.repeat.clone(),
                    children: section
                        .children
                        .iter()
                        .map(|node| self.repeated_layout_node(node, repeat_filter, suffix))
                        .collect(),
                }))
            }
        }
    }
}

/* === Public Functions === */
pub fn resolve_document(input: DocumentResolveInput) -> Result<DocumentPayload> {
    resolve(input, true)
}

pub(super) fn resolve_current(input: DocumentResolveInput) -> Result<DocumentPayload> {
    resolve(input, false)
}

pub(crate) fn capture_borrowed(input: BorrowedInput<'_>) -> Result<Capture> {
    let (mut cards, layout) = input.definition.normalized_parts();
    validate_normalized_parts(&cards, &layout)
        .map_err(|issues| DocumentFailure::InvalidDefinition { issues })?;
    // Captures Freeze Stored IDs And Discard Builder-Only Automatic ID State.
    for card in &mut cards {
        if let DocumentCardBody::Plot { plot } = &mut card.body {
            plot.automatic_id = false;
        }
    }
    let definition = crate::document::DocumentDefinition {
        document_id: input.definition.document_id.clone(),
        metadata: input.definition.metadata.clone(),
        cards,
        layout,
        sections: Vec::new(),
    };
    let mut capture = CaptureBindings {
        source_ids: definition
            .cards
            .iter()
            .map(|card| {
                let id = card.metadata.identifier.clone();
                (id.clone(), id)
            })
            .collect(),
        ..Default::default()
    };
    let mut artifact_lists = Vec::new();
    let mut plot_descriptors = Vec::new();
    let mut plot_inputs = Vec::new();
    let mut materializer = DocumentMaterializer {
        capture: Some(&mut capture),
        artifact_lists: &mut artifact_lists,
        card_definitions: definition
            .cards
            .iter()
            .cloned()
            .map(|card| (card.metadata.identifier.clone(), card))
            .collect(),
        card_payload_ids: BTreeMap::new(),
        card_payloads: Vec::new(),
        input_artifacts: input.data.artifacts,
        markers_len: input
            .data
            .records
            .iter()
            .filter(|record| input.selection.matches_scope(record.scope))
            .count(),
        materialize_plots: false,
        plot_descriptors: &mut plot_descriptors,
        plot_inputs: &mut plot_inputs,
        selection: input.selection,
        evidence: Evidence {
            records: input.data.records,
            artifacts: input.data.artifacts,
            telemetry: input.data.telemetry,
            context: input.data.context,
            selection: Some(input.selection),
        },
    };
    materializer.capture_nodes(definition.layout.nodes.clone())?;
    if capture.identity_collision {
        return Err(DocumentFailure::InvalidDefinition {
            issues: vec!["captured repeat identities collide".into()],
        }
        .into());
    }
    Ok(Capture {
        definition,
        selection: input.selection.clone(),
        cards: capture.cards,
        repeats: capture.repeats,
        sections: capture.sections,
        warnings: Vec::new(),
    })
}

pub(super) fn repeat_label_bound(repeat: &DocumentSectionRepeat, evidence: Evidence<'_>) -> usize {
    evidence
        .rows(&repeat.value)
        .filter_map(|row| expression_label_size(row, &repeat.value, evidence))
        .max()
        .unwrap_or(0)
}

/* === Private Functions === */
fn expression_label_size(
    row: Row<'_>,
    expression: &DataExpression,
    evidence: Evidence<'_>,
) -> Option<usize> {
    binding::row_text(row, expression)
        .map(str::len)
        .or_else(|| {
            expression_value(row, expression, evidence).map(|value| value_label_size(&value))
        })
}

fn resolve(input: DocumentResolveInput, materialize_plots: bool) -> Result<DocumentPayload> {
    let selection = input.selection;
    let definition = input.definition;
    let mut telemetry = input.data.telemetry;
    telemetry.retain(|sample| scope_matches(sample.scope.as_ref(), &selection));
    resolve_parts(
        definition,
        selection,
        BorrowedData {
            context: super::input::Context::default(),
            records: super::input::Records::Native(&input.data.records),
            telemetry: &telemetry,
            artifacts: Artifacts::Native(&input.data.artifacts),
        },
        materialize_plots,
    )
}

fn resolve_parts(
    definition: crate::document::DocumentDefinition,
    selection: crate::document::DocumentSelection,
    data: BorrowedData<'_>,
    materialize_plots: bool,
) -> Result<DocumentPayload> {
    let (card_definitions, layout_definition) = definition.normalized_parts();
    validate_normalized_parts(&card_definitions, &layout_definition)
        .map_err(|issues| DocumentFailure::InvalidDefinition { issues })?;
    binding::validate(&card_definitions, &layout_definition.nodes)?;
    let markers = super::markers::selected_markers(data.records, &selection)?;
    let mut plot_descriptors = Vec::new();
    let mut plot_inputs = Vec::new();
    let mut artifact_lists = Vec::new();
    let (cards, layout) = {
        let mut materializer = DocumentMaterializer {
            capture: None,
            artifact_lists: &mut artifact_lists,
            card_payloads: Vec::new(),
            card_payload_ids: BTreeMap::new(),
            card_definitions: card_definitions
                .into_iter()
                .map(|card| (card.metadata.identifier.clone(), card))
                .collect(),
            input_artifacts: data.artifacts,
            markers_len: markers.len(),
            materialize_plots,
            plot_descriptors: &mut plot_descriptors,
            plot_inputs: &mut plot_inputs,
            selection: &selection,
            evidence: Evidence {
                records: data.records,
                artifacts: data.artifacts,
                telemetry: data.telemetry,
                context: data.context,
                selection: Some(&selection),
            },
        };
        let layout = materializer.materialize_layout(layout_definition)?;
        (materializer.card_payloads, layout)
    };
    let warnings = if materialize_plots {
        warnings(&plot_descriptors, data.telemetry)?
    } else {
        Vec::new()
    };

    Ok(DocumentPayload {
        document_id: definition.document_id,
        metadata: definition.metadata,
        selection,
        cards,
        layout,
        plot_descriptors,
        plot_inputs,
        record_markers: markers,
        artifact_lists,
        warnings,
    })
}

fn warnings(
    plot_descriptors: &[crate::document::definition::PlotDescriptor],
    telemetry: &[crate::telemetry::TelemetrySample],
) -> Result<Vec<DocumentWarning>> {
    if plot_descriptors.is_empty() || !telemetry.is_empty() {
        return Ok(Vec::new());
    }

    Ok(vec![DocumentWarning {
        code: Identifier::new("document.plot.no_telemetry")?,
        message: "Document declares plot inputs, but no telemetry samples are available."
            .to_owned(),
    }])
}

fn repeat_values<'a>(
    repeat: &DocumentSectionRepeat,
    evidence: Evidence<'a>,
) -> Vec<RepeatValue<'a>> {
    let mut values = Vec::<RepeatValue<'a>>::with_capacity(evidence.upper_rows());

    for (index, sample) in evidence.rows(&repeat.value).enumerate() {
        if !row_matches_filters(sample, repeat.filters.iter(), evidence) {
            continue;
        }
        let Some(value) = expression_value(sample, &repeat.value, evidence)
            .filter(|value| traceable_value(value))
        else {
            continue;
        };
        let order = repeat
            .order_by
            .as_ref()
            .and_then(|expression| expression_value(sample, expression, evidence));

        if let Some(existing) = values
            .iter_mut()
            .find(|candidate| values_equal(&candidate.value, &value))
        {
            if existing.order.is_none() {
                existing.order = order;
            }
            continue;
        }

        values.push(RepeatValue {
            value,
            order,
            first_index: index,
        });
    }

    // Each Distinct Value Has A Unique First Index, So No Further Tie Is Possible.
    values.sort_unstable_by(|left, right| {
        compare_values(left.order.as_deref(), right.order.as_deref())
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.first_index.cmp(&right.first_index))
    });
    values
}

fn repeated_card(
    mut card: DocumentCard,
    repeat_filter: &DocumentFilter,
    suffix: &str,
) -> DocumentCard {
    let identifier = suffixed_identifier(&card.metadata.identifier, suffix);
    card.metadata = generated_metadata(&card.metadata, &identifier, card.metadata.label.clone());
    card.filters.reserve_exact(1);
    card.filters.push(repeat_filter.clone());
    match &mut card.body {
        DocumentCardBody::Plot { plot } => repeat_plot(plot, repeat_filter, suffix),
        DocumentCardBody::TelemetrySummary { summary } => {
            repeat_telemetry_summary(summary, repeat_filter, suffix);
        }
        DocumentCardBody::Text { .. }
        | DocumentCardBody::Table { .. }
        | DocumentCardBody::Steps
        | DocumentCardBody::Criteria
        | DocumentCardBody::Traceability
        | DocumentCardBody::Notes
        | DocumentCardBody::Records
        | DocumentCardBody::Artifacts
        | DocumentCardBody::Media
        | DocumentCardBody::Health
        | DocumentCardBody::TelemetryContext => {}
    }
    card
}

fn repeat_plot(plot: &mut Box<PlotDefinition>, repeat_filter: &DocumentFilter, suffix: &str) {
    let identifier = suffixed_identifier(&plot.metadata.identifier, suffix);
    plot.metadata = generated_metadata(&plot.metadata, &identifier, plot.metadata.label.clone());
    plot.filters.reserve_exact(1);
    plot.filters.push(repeat_filter.clone());
}

fn repeat_telemetry_summary(
    summary: &mut TelemetrySummaryDefinition,
    repeat_filter: &DocumentFilter,
    suffix: &str,
) {
    for group in &mut summary.groups {
        let identifier = suffixed_identifier(&group.metadata.identifier, suffix);
        group.metadata =
            generated_metadata(&group.metadata, &identifier, group.metadata.label.clone());
        for tile in &mut group.tiles {
            let identifier = suffixed_identifier(&tile.metadata.identifier, suffix);
            tile.metadata =
                generated_metadata(&tile.metadata, &identifier, tile.metadata.label.clone());
            tile.filters.reserve_exact(1);
            tile.filters.push(repeat_filter.clone());
        }
    }
}

fn telemetry_summary_payload(
    summary: &TelemetrySummaryDefinition,
    card_filters: &[DocumentFilter],
    evidence: Evidence<'_>,
) -> Result<Vec<TelemetrySummaryGroupPayload>> {
    summary
        .groups
        .iter()
        .map(|group| {
            Ok(TelemetrySummaryGroupPayload {
                metadata: group.metadata.clone(),
                tiles: group
                    .tiles
                    .iter()
                    .map(|tile| telemetry_summary_tile_payload(tile, card_filters, evidence))
                    .collect::<Result<_>>()?,
            })
        })
        .collect()
}

fn telemetry_summary_tile_payload(
    tile: &TelemetrySummaryTileDefinition,
    card_filters: &[DocumentFilter],
    evidence: Evidence<'_>,
) -> Result<TelemetrySummaryTilePayload> {
    let result = (|| {
        binding::validate_predicates(&tile.value, card_filters.iter().chain(tile.filters.iter()))?;
        for filter in card_filters.iter().chain(tile.filters.iter()) {
            binding::validate_filter(&filter.expression)?;
        }
        binding::evaluate(&tile.value, evidence, |sample| {
            row_matches_filters(
                sample,
                card_filters.iter().chain(tile.filters.iter()),
                evidence,
            )
        })
    })();
    let (value, unavailable) = match result {
        Ok(value) => (value, None),
        Err(error) => (None, Some(unavailable_reason(error)?)),
    };
    let state = value
        .as_ref()
        .and_then(|value| matching_summary_state(value, &tile.conditions));
    Ok(TelemetrySummaryTilePayload {
        metadata: tile.metadata.clone(),
        value,
        unavailable,
        stat: tile.stat.clone(),
        format: tile.format.clone(),
        presentation: tile.presentation.clone(),
        unit: tile.unit.clone(),
        state,
    })
}

fn unavailable_reason(error: dokime::Error) -> Result<String> {
    match error {
        dokime::Error::Document(DocumentFailure::Evaluation { message })
            if super::payload::valid_unavailable_reason(message) =>
        {
            Ok(message.to_owned())
        }
        error => Err(error),
    }
}

fn matching_summary_state(
    value: &Value,
    conditions: &[TelemetrySummaryCondition],
) -> Option<crate::document::definition::TelemetrySummaryState> {
    conditions
        .iter()
        .find(|condition| {
            value_matches_operator(
                Some(value),
                condition.operator.clone(),
                Some(&condition.value),
            )
        })
        .map(|condition| condition.state.clone())
}

fn row_matches_filters<'a>(
    row: Row<'_>,
    filters: impl Iterator<Item = &'a DocumentFilter> + Clone,
    evidence: Evidence<'_>,
) -> bool {
    if let Row::Context(context) = row
        && filters
            .clone()
            .any(|filter| row.needs_sample(&filter.expression))
    {
        return evidence
            .telemetry
            .iter()
            .filter(|sample| evidence.selected(sample) && context.covers(sample.scope.as_ref()))
            .any(|sample| row_matches_filters(evidence.sample(sample), filters.clone(), evidence));
    }
    filters.into_iter().all(|filter| {
        value_matches_operator(
            expression_value(row, &filter.expression, evidence).as_deref(),
            filter.operator.clone(),
            filter.value.as_ref(),
        )
    })
}

fn value_matches_operator(
    actual: Option<&Value>,
    operator: FilterOperator,
    expected: Option<&Value>,
) -> bool {
    match operator {
        FilterOperator::Exists => actual.is_some_and(traceable_value),
        FilterOperator::Eq => expected
            .is_some_and(|expected| actual.is_some_and(|actual| values_equal(actual, expected))),
        FilterOperator::Ne => expected
            .is_none_or(|expected| actual.is_none_or(|actual| !values_equal(actual, expected))),
        FilterOperator::Gt => compare_values(actual, expected)
            .is_some_and(|ordering| matches!(ordering, Ordering::Greater)),
        FilterOperator::Gte => compare_values(actual, expected)
            .is_some_and(|ordering| matches!(ordering, Ordering::Greater | Ordering::Equal)),
        FilterOperator::Lt => compare_values(actual, expected)
            .is_some_and(|ordering| matches!(ordering, Ordering::Less)),
        FilterOperator::Lte => compare_values(actual, expected)
            .is_some_and(|ordering| matches!(ordering, Ordering::Less | Ordering::Equal)),
        FilterOperator::Contains => expected
            .is_some_and(|expected| actual.is_some_and(|actual| value_contains(actual, expected))),
    }
}

fn values_equal(actual: &Value, expected: &Value) -> bool {
    if let Some(ordering) = crate::core::numeric_cmp(actual, expected) {
        return ordering == Ordering::Equal;
    }
    actual == expected
}

fn compare_values(actual: Option<&Value>, expected: Option<&Value>) -> Option<Ordering> {
    let actual = actual?;
    let expected = expected?;
    crate::core::numeric_cmp(actual, expected)
        .or_else(|| Some(value_label(actual).cmp(&value_label(expected))))
}

fn value_contains(actual: &Value, expected: &Value) -> bool {
    match actual {
        Value::Text(actual) => actual.contains(value_label(expected).as_str()),
        Value::Array(values) => values
            .iter()
            .any(|candidate| values_equal(candidate, expected)),
        _ => false,
    }
}

fn traceable_value(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Text(value) => !value.trim().is_empty(),
        _ => true,
    }
}

fn render_template(template: &str, expression: &DataExpression, value: &Value) -> String {
    let label = value_label(value);
    let mut rendered = replace_reserved(template, VALUE_PLACEHOLDER, &label);
    if let Some(field) = expression_field_name(expression) {
        let mut placeholder = String::with_capacity(
            field
                .len()
                .checked_add(2)
                .expect("repeat field placeholder length overflows"),
        );
        placeholder.push('{');
        placeholder.push_str(field);
        placeholder.push('}');
        rendered = replace_reserved(&rendered, &placeholder, &label);
    }
    rendered
}

fn replace_reserved(source: &str, placeholder: &str, label: &str) -> String {
    let count = source.matches(placeholder).count();
    let bytes = replacement_size(source.len(), count, placeholder.len(), label.len())
        .expect("repeat template length overflows");
    let mut result = String::with_capacity(bytes);
    for (index, part) in source.split(placeholder).enumerate() {
        if index != 0 {
            result.push_str(label);
        }
        result.push_str(part);
    }
    result
}

fn replacement_size(
    source: usize,
    count: usize,
    placeholder: usize,
    label: usize,
) -> Option<usize> {
    source
        .checked_sub(count.checked_mul(placeholder)?)?
        .checked_add(count.checked_mul(label)?)
}

fn template_memory(template: &str, expression: &DataExpression, label: usize) -> Option<usize> {
    let first = replacement_size(
        template.len(),
        template.matches(VALUE_PLACEHOLDER).count(),
        VALUE_PLACEHOLDER.len(),
        label,
    )?;
    let second = if let Some(field) = expression_field_name(expression) {
        let placeholder = field.len().checked_add(2)?;
        let growth = (first / placeholder).checked_mul(label.saturating_sub(placeholder))?;
        first.checked_add(growth)?.checked_add(placeholder)?
    } else {
        0
    };
    first.checked_add(second)?.checked_add(label)
}

pub(super) fn repeat_filter_memory(
    repeat: &DocumentSectionRepeat,
    evidence: Evidence<'_>,
) -> usize {
    use harmos::MemorySize;
    let bytes = repeat.value.memory_size();
    if bytes == usize::MAX {
        return usize::MAX;
    }
    let expression = bytes.saturating_sub(std::mem::size_of::<DataExpression>());
    let value = evidence
        .rows(&repeat.value)
        .filter_map(|row| {
            binding::row_text(row, &repeat.value)
                .map(|text| std::mem::size_of::<Value>().saturating_add(text.len()))
                .or_else(|| {
                    expression_value(row, &repeat.value, evidence).map(|value| value.memory_size())
                })
        })
        .max()
        .unwrap_or(std::mem::size_of::<Value>());
    if value == usize::MAX {
        return usize::MAX;
    }
    std::mem::size_of::<DocumentFilter>()
        .saturating_add(expression)
        .saturating_add(value.saturating_sub(std::mem::size_of::<Value>()))
}

pub(super) fn repeat_selection_memory(
    repeat: &DocumentSectionRepeat,
    evidence: Evidence<'_>,
) -> usize {
    let labels = repeat.order_by.as_ref().map_or(0, |order| {
        evidence
            .rows(&repeat.value)
            .filter_map(|row| expression_label_size(row, order, evidence))
            .max()
            .unwrap_or(0)
    });
    let (owned, temporary) =
        evidence
            .rows(&repeat.value)
            .fold((0usize, 0usize), |(total, largest), row| {
                let value = binding::row_text(row, &repeat.value).map_or(0, str::len);
                let order = repeat
                    .order_by
                    .as_ref()
                    .and_then(|order| binding::row_text(row, order))
                    .map_or(0, str::len);
                let heap = value.saturating_add(order);
                (total.saturating_add(heap), largest.max(heap))
            });
    std::mem::size_of::<Vec<RepeatValue<'_>>>()
        .saturating_add(
            evidence
                .upper_rows()
                .saturating_mul(std::mem::size_of::<RepeatValue<'_>>()),
        )
        .saturating_add(std::mem::size_of::<RepeatValue<'_>>())
        .saturating_add(owned)
        .saturating_add(temporary)
        .saturating_add(labels)
        .saturating_add(labels)
}

pub(super) fn repeat_template_memory(
    repeat: &DocumentSectionRepeat,
    node_id_bytes: usize,
    evidence: Evidence<'_>,
) -> usize {
    evidence
        .rows(&repeat.value)
        .filter_map(|row| expression_label_size(row, &repeat.value, evidence))
        .map(|label| {
            let labels = template_memory(&repeat.label_template, &repeat.value, label)?;
            let identifiers = if let Some(template) = &repeat.id_template {
                template_memory(template, &repeat.value, label)?
            } else {
                // Default Identifier Template Is "<Node Id>.{value}".
                let template = node_id_bytes
                    .checked_add(1)?
                    .checked_add(VALUE_PLACEHOLDER.len())?;
                let first = node_id_bytes.checked_add(1)?.checked_add(label)?;
                let field = expression_field_name(&repeat.value)?.len().checked_add(2)?;
                let second =
                    first.checked_add((first / field).checked_mul(label.saturating_sub(field))?)?;
                template
                    .checked_add(first)?
                    .checked_add(second)?
                    .checked_add(field)?
                    .checked_add(label)?
            };
            labels.checked_add(identifiers)
        })
        .map(|memory| memory.unwrap_or(usize::MAX))
        .max()
        .unwrap_or(0)
}

fn expression_field_name(expression: &DataExpression) -> Option<&str> {
    match &expression.source {
        DataSource::Telemetry { field, .. } => Some(field),
        DataSource::Dimension { name } | DataSource::Parameter { name } => Some(name),
        DataSource::Record { record_type } => Some(record_type),
        DataSource::Artifact { artifact } => Some(artifact),
    }
}

fn value_label_size(value: &Value) -> usize {
    struct Counter(usize);
    impl std::fmt::Write for Counter {
        fn write_str(&mut self, value: &str) -> std::fmt::Result {
            self.0 = self.0.checked_add(value.len()).ok_or(std::fmt::Error)?;
            Ok(())
        }
    }
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self
                .0
                .checked_add(bytes.len())
                .ok_or_else(|| std::io::Error::other("label length overflow"))?;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter(0);
    let result = match value {
        Value::Null => return 0,
        Value::Bool(value) => std::fmt::write(&mut counter, format_args!("{value}")),
        Value::Integer(value) => std::fmt::write(&mut counter, format_args!("{value}")),
        Value::Unsigned(value) => std::fmt::write(&mut counter, format_args!("{value}")),
        Value::Number(value) => std::fmt::write(&mut counter, format_args!("{value}")),
        Value::Text(value) => return value.len(),
        Value::Datetime(value) => {
            return value
                .to_offset(time::UtcOffset::UTC)
                .format_into(&mut counter, &time::format_description::well_known::Rfc3339)
                .map_or(usize::MAX, |_| counter.0);
        }
        Value::Array(_) | Value::Object(_) => return "value".len(),
    };
    result.map_or(usize::MAX, |_| counter.0)
}

fn value_label(value: &Value) -> String {
    let bytes = value_label_size(value);
    match value {
        Value::Null => String::new(),
        Value::Text(value) => value.clone(),
        Value::Datetime(timestamp) => {
            let mut text = Vec::with_capacity(bytes);
            timestamp
                .to_offset(time::UtcOffset::UTC)
                .format_into(&mut text, &time::format_description::well_known::Rfc3339)
                .expect("a stored datetime is representable as RFC3339");
            String::from_utf8(text).expect("RFC3339 formatting produces UTF-8")
        }
        Value::Array(_) | Value::Object(_) => "value".to_owned(),
        Value::Bool(_) | Value::Integer(_) | Value::Unsigned(_) | Value::Number(_) => {
            let mut text = String::with_capacity(bytes);
            let result = match value {
                Value::Bool(value) => std::fmt::write(&mut text, format_args!("{value}")),
                Value::Integer(value) => std::fmt::write(&mut text, format_args!("{value}")),
                Value::Unsigned(value) => std::fmt::write(&mut text, format_args!("{value}")),
                Value::Number(value) => std::fmt::write(&mut text, format_args!("{value}")),
                _ => unreachable!("the enclosing match selected a scalar"),
            };
            result.expect("writing a scalar into String succeeds");
            if matches!(value, Value::Number(_)) && text.ends_with(".0") {
                text.truncate(text.len() - 2);
            }
            text
        }
    }
}

fn generated_metadata(source: &Metadata, identifier: &str, label: String) -> Metadata {
    Metadata::definition(
        identifier_from_generated(identifier),
        label,
        source.description.clone(),
        source.tags.clone(),
    )
}

fn identifier_from_generated(value: &str) -> Identifier {
    Identifier::new(value).unwrap_or_else(|_| {
        Identifier::new(identifier_segment(value)).expect("sanitized identifier is valid")
    })
}

fn suffixed_identifier(identifier: &Identifier, suffix: &str) -> String {
    let bytes = identifier
        .as_str()
        .len()
        .checked_add(1)
        .and_then(|bytes| bytes.checked_add(suffix.len()))
        .expect("repeated identifier length overflows");
    let mut value = String::with_capacity(bytes);
    value.push_str(identifier.as_str());
    value.push('.');
    value.push_str(suffix);
    value
}

fn identifier_segment(value: &str) -> String {
    let bytes = identifier_segment_size(value);
    if bytes == 0 {
        return "value".to_owned();
    }
    let mut normalized = String::with_capacity(bytes);
    let mut separator = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            if separator && !normalized.is_empty() {
                normalized.push('.');
            }
            normalized.push(character);
            separator = false;
        } else {
            separator = true;
        }
    }
    normalized
}

fn identifier_segment_size(value: &str) -> usize {
    let mut bytes = 0usize;
    let mut separator = false;
    for character in value.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() {
            if separator && bytes != 0 {
                bytes = bytes.saturating_add(1);
            }
            bytes = bytes.saturating_add(1);
            separator = false;
        } else {
            separator = true;
        }
    }
    bytes
}

fn identifier_segment_identifier(value: &str) -> Identifier {
    identifier_from_generated(value)
}

/* === Tests === */
