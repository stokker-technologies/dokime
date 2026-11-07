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
use std::collections::{BTreeMap, BTreeSet, VecDeque};

// Third Party Imports
use tonic::Status;

// Local Imports
use super::super::value::{self, canonical};
use dokime::domain::model::documents::wire as contract;
use dokime::domain::model::documents::wire::DocumentFrameFrame as Frame;
use dokime::{CurrentDocumentCapture, CurrentDocumentEvent, document as native};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Frames {
    frames: VecDeque<dokime::domain::model::documents::wire::DocumentFrame>,
    limit: usize,
    budget: Option<native::capture::CardBudget>,
    card: String,
}

pub(super) struct Stream {
    limit: usize,
    sent: u64,
    partial: bool,
    cards: u64,
    layout: u64,
    bindings: u64,
    repeats: BTreeSet<String>,
    sections: BTreeSet<String>,
    failed: BTreeMap<String, native::capture::CardFailure>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Frames {
    fn new(limit: usize) -> Self {
        Self {
            frames: VecDeque::new(),
            limit,
            budget: None,
            card: String::new(),
        }
    }

    fn failure(&mut self, error: Status) -> native::capture::CardFailure {
        let budget = self.budget.take();
        native::capture::CardFailure {
            code: budget.as_ref().map_or("document_card_failed", |budget| budget.name.as_str()).to_owned(),
            message: if budget.is_some() { error.message().to_owned() } else { format!("card {}: {}", self.card, error.message()) },
            budget,
        }
    }

    fn push(&mut self, frame: Frame) -> Result<(), Status> {
        let frame = dokime::domain::model::documents::wire::DocumentFrame { frame: Some(frame) };
        let bytes = crate::grpc::content::output_frame_len(&frame)?;
        if bytes > self.limit {
            self.budget = Some(native::capture::CardBudget {
                name: "frame_bytes".into(),
                limit: self.limit as u64,
                reached: bytes as u64,
                card_id: self.card.clone(),
            });
            return Err(Status::resource_exhausted(format!(
                "card {}: budget frame_bytes exceeded (limit {}, reached {bytes})",
                self.card, self.limit
            )));
        }
        self.frames.try_reserve_exact(1).map_err(|_| Status::resource_exhausted("document response allocation refused"))?;
        self.frames.push_back(frame);
        Ok(())
    }
}

impl Stream {
    pub(super) fn new(limit: usize) -> Self {
        Self {
            limit,
            sent: 0,
            partial: false,
            cards: 0,
            layout: 0,
            bindings: 0,
            repeats: BTreeSet::new(),
            sections: BTreeSet::new(),
            failed: BTreeMap::new(),
        }
    }

    pub(super) fn event(&mut self, event: CurrentDocumentEvent) -> Result<VecDeque<contract::DocumentFrame>, Status> {
        let frames = match event {
            CurrentDocumentEvent::Structure { document, pending_repeats, partial } => {
                self.partial = partial;
                self.structure(document, pending_repeats)?
            }
            CurrentDocumentEvent::Evidence { cut, telemetry, cacheable } => {
                let mut frames = Frames::new(self.limit);
                frames.push(Frame::EvidenceCut(contract::DocumentEvidenceCut {
                    cut: Some(contract::DocumentTelemetryCut {
                        store: cut.history.store.to_string(),
                        epoch: cut.history.epoch.to_string(),
                        position: cut.sequence,
                    }),
                    telemetry,
                    cacheable,
                }))?;
                frames.frames
            }
            CurrentDocumentEvent::Artifacts(digest) => {
                let mut frames = Frames::new(self.limit);
                frames.push(Frame::ArtifactCut(digest))?;
                frames.frames
            }
            CurrentDocumentEvent::Expansion { capture, cacheable } => self.expansion(capture, cacheable)?,
            CurrentDocumentEvent::Page { current, next } => {
                let mut frames = Frames::new(self.limit);
                frames.push(Frame::Page(contract::DocumentPage { current, next }))?;
                frames.frames
            }
            CurrentDocumentEvent::Card(binding) => {
                if self.failed.contains_key(binding.source_id.as_str()) {
                    VecDeque::new()
                } else {
                    self.card(binding)?
                }
            }
        };
        self.sent += frames.len() as u64;
        Ok(frames)
    }

    fn structure(&mut self, captured: CurrentDocumentCapture, pending_repeats: bool) -> Result<VecDeque<contract::DocumentFrame>, Status> {
        let mut output = Frames::new(self.limit);
        let capture = captured.capture;
        let definition = capture.definition;
        let scope = canonical::scope(&capture.selection.scope)?;
        let telemetry = captured.telemetry_cut.map(|cut| dokime::domain::model::documents::wire::DocumentTelemetryCut {
            store: cut.history.store.as_str().to_owned(),
            epoch: cut.history.epoch.as_str().to_owned(),
            position: cut.sequence,
        });
        output.push(Frame::Header(dokime::domain::model::documents::wire::DocumentHeader {
            document_id: definition.document_id.to_string(),
            metadata: Some(canonical::metadata(&definition.metadata)?),
            scope: Some(scope),
            frontiers: Some(dokime::domain::model::documents::wire::DocumentFrontiers {
                process_epoch: captured.process_epoch,
                state_revision: captured.state_revision,
                telemetry,
                artifacts_sha256: None,
                evidence: captured.evidence_cut.map(|cut| contract::DocumentTelemetryCut {
                    store: cut.history.store.to_string(),
                    epoch: cut.history.epoch.to_string(),
                    position: cut.sequence,
                }),
            }),
        }))?;
        let mut ordinal = 0;
        layout_nodes(&mut output, definition.layout.nodes, None, &mut ordinal)?;
        let card_count = definition.cards.len() as u64;

        self.layout = ordinal;
        self.cards = card_count;
        for card in definition.cards {
            let source = card.metadata.identifier.to_string();
            let mut encoded = Frames::new(self.limit);
            encoded.card = source.clone();
            if let Err(error) = declaration(&mut encoded, card.clone(), &capture.selection) {
                self.failed.insert(source.clone(), encoded.failure(error));
                encoded = Frames::new(self.limit);
                let mut metadata = card.metadata.clone();
                metadata.label = source.clone();
                metadata.description = None;
                metadata.tags.clear();
                metadata.attributes.clear();
                encoded.push(Frame::Card(contract::DocumentCard {
                    metadata: Some(canonical::metadata(&metadata)?),
                    filters: Vec::new(),
                    body: Some(contract::DocumentCardBody::Markdown(String::new())),
                }))?;
            }
            output.frames.extend(encoded.frames);
        }
        self.bindings = capture.cards.len() as u64;
        for binding in capture.cards {
            let mut encoded = self.card(binding)?;
            encoded.pop_back();
            output.frames.extend(encoded);
        }
        self.selections(&mut output, capture.repeats, capture.sections, pending_repeats)?;
        for warning in capture.warnings {
            output.push(Frame::Warning(contract::DocumentWarning {
                code: warning.code.to_string(),
                message: warning.message,
            }))?;
        }
        output.push(Frame::StructureComplete(self.counts(self.sent + output.frames.len() as u64 + 1)))?;
        Ok(output.frames)
    }

    fn apply_failure(&self, mut binding: native::capture::CardBinding) -> native::capture::CardBinding {
        if let Some(failure) = self.failed.get(binding.source_id.as_str()) {
            let mut failure = failure.clone();
            if let Some(budget) = &mut failure.budget {
                budget.card_id = binding.metadata.identifier.to_string();
            }
            binding.result = native::capture::CardBindingResult::Failed { failure };
        }
        binding
    }

    fn card(&self, binding: native::capture::CardBinding) -> Result<VecDeque<contract::DocumentFrame>, Status> {
        let mut binding = self.apply_failure(binding);
        let id = binding.metadata.identifier.to_string();
        let mut output = Frames::new(self.limit);
        output.card = id.clone();
        if let Err(error) = append_binding(&mut output, binding.clone()) {
            let failure = output.failure(error);
            output = Frames::new(self.limit);
            binding = failed_binding(binding, failure);
            append_binding(&mut output, binding.clone())?;
        }
        output.push(Frame::CardComplete(id))?;
        let reached = output
            .frames
            .iter()
            .try_fold(0usize, |used, frame| Ok::<_, Status>(used.saturating_add(crate::grpc::content::output_frame_len(frame)?)))?;
        let limit = native::bounded::DocumentLimits::default().materialized_bytes;
        if reached > limit {
            let mut output = Frames::new(self.limit);
            let failure = native::capture::CardFailure {
                code: "materialized_bytes".into(),
                message: format!(
                    "card {}: budget materialized_bytes exceeded (limit {limit}, reached {reached})",
                    binding.metadata.identifier
                ),
                budget: Some(native::capture::CardBudget {
                    name: "materialized_bytes".into(),
                    limit: limit as u64,
                    reached: reached as u64,
                    card_id: binding.metadata.identifier.to_string(),
                }),
            };
            binding = failed_binding(binding, failure);
            append_binding(&mut output, binding.clone())?;
            output.push(Frame::CardComplete(binding.metadata.identifier.to_string()))?;
            return Ok(output.frames);
        }
        Ok(output.frames)
    }

    fn expansion(&mut self, capture: native::capture::Capture, cacheable: bool) -> Result<VecDeque<contract::DocumentFrame>, Status> {
        let mut output = Frames::new(self.limit);
        self.bindings += capture.cards.len() as u64;
        let removed =
            self.sections.iter().filter(|id| !capture.sections.keys().any(|key| key.as_str() == id.as_str())).cloned().collect::<Vec<_>>();
        for id in removed {
            self.sections.remove(&id);
            output.push(Frame::SectionRemoval(id))?;
        }
        for binding in capture.cards {
            let mut encoded = self.card(binding)?;
            encoded.pop_back();
            output.frames.extend(encoded);
        }
        self.selections(&mut output, capture.repeats, capture.sections, false)?;
        output.push(Frame::ExpansionComplete(cacheable))?;
        Ok(output.frames)
    }

    fn selections(
        &mut self, output: &mut Frames,
        repeats: BTreeMap<dokime::domain::model::identity::Identifier, Vec<native::capture::RepeatInstance>>,
        sections: BTreeMap<dokime::domain::model::identity::Identifier, native::capture::SectionBinding>, pending: bool,
    ) -> Result<(), Status> {
        for (id, instances) in repeats {
            self.repeats.insert(id.to_string());
            output.push(Frame::RepeatSelection(contract::DocumentRepeatSelection {
                node_id: id.to_string(),
                instance_count: instances.len() as u64,
                pending,
            }))?;
            for (ordinal, instance) in instances.into_iter().enumerate() {
                output.push(Frame::RepeatInstance(contract::DocumentRepeatInstance {
                    selection_id: id.to_string(),
                    ordinal: ordinal as u64,
                    suffix: instance.suffix,
                    node_id: instance.node_id.to_string(),
                    filter: Some(super::super::plots::plot_filter_wire(&instance.filter)?),
                }))?;
            }
        }
        for (id, section) in sections {
            if !self.sections.insert(id.to_string()) {
                continue;
            }
            output.push(Frame::SectionIdentity(contract::DocumentSectionIdentity {
                node_id: id.to_string(),
                metadata: Some(canonical::metadata(&section.metadata)?),
                unavailable: section.unavailable,
            }))?;
        }
        Ok(())
    }

    fn counts(&self, frame_count: u64) -> contract::DocumentCompletion {
        contract::DocumentCompletion {
            frame_count,
            card_count: self.cards,
            layout_node_count: self.layout,
            binding_count: self.bindings,
            repeat_count: self.repeats.len() as u64,
            section_count: self.sections.len() as u64,
            partial: self.partial,
        }
    }

    pub(super) fn completion(&self) -> contract::DocumentFrame {
        contract::DocumentFrame {
            frame: Some(Frame::Completion(self.counts(self.sent + 1))),
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn declaration(output: &mut Frames, card: native::DocumentCardDefinition, selection: &native::DocumentSelection) -> Result<(), Status> {
    use super::super::{enums::canonical as enums, history::document::canonical as authoring, plots};
    use dokime::domain::model::documents::wire::DocumentCardBody as Wire;
    use native::DocumentCardBody as Body;
    let id = card.metadata.identifier.to_string();
    let body = match &card.body {
        Body::Text { markdown } => Wire::Markdown(markdown.clone()),
        Body::Table { rows } => Wire::Table(dokime::domain::model::documents::wire::DocumentTable { row_count: rows.len() as u64 }),
        Body::TelemetrySummary { summary } => {
            Wire::Summary(dokime::domain::model::documents::wire::DocumentSummary { group_count: summary.groups.len() as u64 })
        }
        body => {
            let kind = match body {
                Body::Plot { .. } => dokime::domain::model::documents::wire::DocumentCardKind::Plot,
                Body::Steps => dokime::domain::model::documents::wire::DocumentCardKind::Steps,
                Body::Criteria => dokime::domain::model::documents::wire::DocumentCardKind::Criteria,
                Body::Traceability => dokime::domain::model::documents::wire::DocumentCardKind::Traceability,
                Body::Notes => dokime::domain::model::documents::wire::DocumentCardKind::Notes,
                Body::Records => dokime::domain::model::documents::wire::DocumentCardKind::Records,
                Body::Artifacts => dokime::domain::model::documents::wire::DocumentCardKind::Artifacts,
                Body::Media => dokime::domain::model::documents::wire::DocumentCardKind::Media,
                Body::Health => dokime::domain::model::documents::wire::DocumentCardKind::Health,
                Body::TelemetryContext => dokime::domain::model::documents::wire::DocumentCardKind::TelemetryContext,
                _ => unreachable!("literal cards handled above"),
            };
            let plot = if let Body::Plot { plot } = body {
                let mut wire = authoring::historical_plot(plot, &selection.scope)?;
                // A Plot Reading One Telemetry Stream Names It, As Live Requests Do.
                if let Some(request) = wire.request.as_mut()
                    && request.stream.is_empty()
                    && let Some(stream) = single_stream(plot)
                {
                    request.stream = stream.to_owned();
                }
                Some(wire)
            } else {
                None
            };
            Wire::Owner(Box::new(dokime::domain::model::documents::wire::DocumentOwnerReference {
                kind,
                plot_id: plot.as_ref().and_then(|plot| plot.request.as_ref()?.plot_id.clone()),
                plot,
            }))
        }
    };
    output.push(Frame::Card(dokime::domain::model::documents::wire::DocumentCard {
        metadata: Some(canonical::metadata(&card.metadata)?),
        filters: card.filters.iter().map(plots::plot_filter_wire).collect::<Result<_, _>>()?,
        body: Some(body),
    }))?;
    if let Body::Table { rows } = card.body {
        for (ordinal, fields) in rows.into_iter().enumerate() {
            let mut budget = value::Budget::new()?;
            output.push(Frame::TableRow(dokime::domain::model::documents::wire::DocumentTableRow {
                card_id: id.clone(),
                ordinal: ordinal as u64,
                values: Some(canonical::object(&fields, &mut budget)?),
            }))?;
        }
    } else if let Body::TelemetrySummary { summary } = card.body {
        for (group_ordinal, group) in summary.groups.into_iter().enumerate() {
            output.push(Frame::GroupDefinition(dokime::domain::model::documents::wire::DocumentSummaryGroupDefinition {
                card_id: id.clone(),
                ordinal: group_ordinal as u64,
                metadata: Some(canonical::metadata(&group.metadata)?),
                tile_count: group.tiles.len() as u64,
            }))?;
            for (ordinal, tile) in group.tiles.into_iter().enumerate() {
                let conditions = tile
                    .conditions
                    .into_iter()
                    .map(|condition| {
                        Ok(dokime::domain::model::documents::wire::DocumentSummaryCondition {
                            operator: enums::filter_operator(&condition.operator),
                            value: Some(canonical::encode(&condition.value, &mut value::Budget::new()?, 0)?),
                            state: summary_state(condition.state),
                        })
                    })
                    .collect::<Result<_, Status>>()?;
                output.push(Frame::TileDefinition(dokime::domain::model::documents::wire::DocumentSummaryTileDefinition {
                    card_id: id.clone(),
                    group_ordinal: group_ordinal as u64,
                    ordinal: ordinal as u64,
                    metadata: Some(canonical::metadata(&tile.metadata)?),
                    value: Some(plots::data_expression_wire(&tile.value)?),
                    filters: tile.filters.iter().map(plots::plot_filter_wire).collect::<Result<_, _>>()?,
                    stat: summary_stat(tile.stat),
                    format: summary_format(tile.format),
                    presentation: summary_presentation(tile.presentation),
                    unit: tile.unit,
                    conditions,
                }))?;
            }
        }
    }
    Ok(())
}

fn append_binding(output: &mut Frames, binding: native::capture::CardBinding) -> Result<(), Status> {
    use super::super::plots;
    use native::capture::CardBindingResult;
    output.card = binding.metadata.identifier.to_string();
    let id = binding.metadata.identifier.to_string();
    let result = match &binding.result {
        CardBindingResult::Pending => contract::DocumentBindingResult::Pending(true),
        CardBindingResult::Failed { failure } => contract::DocumentBindingResult::Failure(card_failure(failure)),
        CardBindingResult::Static => dokime::domain::model::documents::wire::DocumentBindingResult::Literal(true),
        CardBindingResult::Summary { groups } => dokime::domain::model::documents::wire::DocumentBindingResult::Summary(
            dokime::domain::model::documents::wire::DocumentSummary { group_count: groups.len() as u64 },
        ),
        CardBindingResult::Plot { descriptor } => {
            dokime::domain::model::documents::wire::DocumentBindingResult::Plot(Box::new(plots::document_plot_definition(descriptor)?))
        }
    };
    output.push(Frame::Binding(dokime::domain::model::documents::wire::DocumentBinding {
        source_id: binding.source_id.to_string(),
        metadata: Some(canonical::metadata(&binding.metadata)?),
        result: Some(result),
    }))?;
    if let CardBindingResult::Summary { groups } = binding.result {
        for (group_ordinal, group) in groups.into_iter().enumerate() {
            output.push(Frame::SummaryGroup(dokime::domain::model::documents::wire::DocumentSummaryGroup {
                card_id: id.clone(),
                ordinal: group_ordinal as u64,
                metadata: Some(canonical::metadata(&group.metadata)?),
                tile_count: group.tiles.len() as u64,
            }))?;
            for (ordinal, tile) in group.tiles.into_iter().enumerate() {
                if !tile.valid_result() {
                    return Err(Status::internal("invalid summary unavailable result"));
                }
                let encoded = tile.value.as_ref().map(|item| canonical::encode(item, &mut value::Budget::new()?, 0)).transpose()?;
                output.push(Frame::SummaryTile(dokime::domain::model::documents::wire::DocumentSummaryTile {
                    card_id: id.clone(),
                    group_ordinal: group_ordinal as u64,
                    ordinal: ordinal as u64,
                    metadata: Some(canonical::metadata(&tile.metadata)?),
                    value: encoded,
                    stat: summary_stat(tile.stat),
                    format: summary_format(tile.format),
                    presentation: summary_presentation(tile.presentation),
                    unit: tile.unit,
                    state: tile.state.map(summary_state),
                    unavailable: tile.unavailable,
                }))?;
            }
        }
    }
    Ok(())
}

fn failed_binding(mut binding: native::capture::CardBinding, failure: native::capture::CardFailure) -> native::capture::CardBinding {
    binding.result = native::capture::CardBindingResult::Failed { failure };
    binding.metadata.label = binding.metadata.identifier.to_string();
    binding.metadata.description = None;
    binding.metadata.tags.clear();
    binding.metadata.attributes.clear();
    binding
}

fn card_failure(failure: &native::capture::CardFailure) -> contract::DocumentCardFailure {
    contract::DocumentCardFailure {
        code: failure.code.clone(),
        message: failure.message.clone(),
        budget: failure.budget.as_ref().map(|budget| contract::DocumentCardBudget {
            name: budget.name.clone(),
            limit: budget.limit,
            reached: budget.reached,
            card_id: budget.card_id.clone(),
        }),
    }
}

fn single_stream(plot: &native::PlotDefinition) -> Option<&str> {
    use dokime::expressions::DataSource;
    let mut streams = std::collections::BTreeSet::new();
    for expression in plot.y_expressions.iter().chain(&plot.x_expression).chain(&plot.series_by_expression).chain(&plot.z_expression) {
        if let DataSource::Telemetry { stream, .. } = &expression.source {
            streams.insert(stream.as_str());
        }
    }
    let mut streams = streams.into_iter();
    match (streams.next(), streams.next()) {
        (Some(stream), None) => Some(stream),
        _ => None,
    }
}

fn layout_nodes(output: &mut Frames, nodes: Vec<native::DocumentNode>, parent: Option<u64>, ordinal: &mut u64) -> Result<(), Status> {
    for node in nodes {
        let current = *ordinal;
        *ordinal += 1;
        let (node, children) = match node {
            native::DocumentNode::Section(section) => (
                dokime::domain::model::documents::wire::DocumentLayoutNodeNode::Section(Box::new(
                    dokime::domain::model::documents::wire::DocumentSection {
                        node_id: section.node_id.to_string(),
                        metadata: Some(canonical::metadata(&section.metadata)?),
                        repeat: section.repeat.as_ref().map(super::super::history::document::canonical::repeat).transpose()?,
                    },
                )),
                section.children,
            ),
            native::DocumentNode::Card(card) => (
                dokime::domain::model::documents::wire::DocumentLayoutNodeNode::Placement(
                    dokime::domain::model::documents::wire::DocumentPlacement {
                        card_id: card.card_id.to_string(),
                        column_span: Some(dokime::domain::model::documents::wire::DocumentSpan {
                            base: card.layout.col_span.base.into(),
                            sm: card.layout.col_span.sm.map(Into::into),
                            md: card.layout.col_span.md.map(Into::into),
                            lg: card.layout.col_span.lg.map(Into::into),
                            xl: card.layout.col_span.xl.map(Into::into),
                        }),
                        row_span: card.layout.row_span.into(),
                        order: card.layout.order,
                        density: density(card.layout.density),
                        rail: rail(card.layout.rail),
                    },
                ),
                Vec::new(),
            ),
        };
        output.push(Frame::LayoutNode(dokime::domain::model::documents::wire::DocumentLayoutNode {
            ordinal: current,
            parent_ordinal: parent,
            node: Some(node),
        }))?;
        layout_nodes(output, children, Some(current), ordinal)?;
    }
    Ok(())
}

fn density(value: native::DocumentCardDensity) -> dokime::domain::model::documents::wire::DocumentDensity {
    match value {
        native::DocumentCardDensity::Compact => dokime::domain::model::documents::wire::DocumentDensity::Compact,
        native::DocumentCardDensity::Normal => dokime::domain::model::documents::wire::DocumentDensity::Normal,
        native::DocumentCardDensity::Spacious => dokime::domain::model::documents::wire::DocumentDensity::Spacious,
    }
}

fn rail(value: native::DocumentCardRail) -> dokime::domain::model::documents::wire::DocumentRail {
    match value {
        native::DocumentCardRail::Main => dokime::domain::model::documents::wire::DocumentRail::Main,
        native::DocumentCardRail::Right => dokime::domain::model::documents::wire::DocumentRail::Right,
        native::DocumentCardRail::Left => dokime::domain::model::documents::wire::DocumentRail::Left,
    }
}

fn summary_stat(value: native::TelemetrySummaryStat) -> dokime::domain::model::documents::wire::DocumentSummaryStat {
    match value {
        native::TelemetrySummaryStat::Latest => dokime::domain::model::documents::wire::DocumentSummaryStat::Latest,
    }
}

fn summary_format(value: native::TelemetrySummaryFormat) -> dokime::domain::model::documents::wire::DocumentSummaryFormat {
    match value {
        native::TelemetrySummaryFormat::Auto => dokime::domain::model::documents::wire::DocumentSummaryFormat::Auto,
        native::TelemetrySummaryFormat::Number => dokime::domain::model::documents::wire::DocumentSummaryFormat::Number,
        native::TelemetrySummaryFormat::Percent => dokime::domain::model::documents::wire::DocumentSummaryFormat::Percent,
        native::TelemetrySummaryFormat::Duration => dokime::domain::model::documents::wire::DocumentSummaryFormat::Duration,
        native::TelemetrySummaryFormat::Distance => dokime::domain::model::documents::wire::DocumentSummaryFormat::Distance,
        native::TelemetrySummaryFormat::Status => dokime::domain::model::documents::wire::DocumentSummaryFormat::Status,
        native::TelemetrySummaryFormat::Text => dokime::domain::model::documents::wire::DocumentSummaryFormat::Text,
    }
}

fn summary_presentation(
    value: native::TelemetrySummaryPresentation,
) -> dokime::domain::model::documents::wire::DocumentSummaryPresentation {
    match value {
        native::TelemetrySummaryPresentation::Value => dokime::domain::model::documents::wire::DocumentSummaryPresentation::Value,
        native::TelemetrySummaryPresentation::Progress => dokime::domain::model::documents::wire::DocumentSummaryPresentation::Progress,
        native::TelemetrySummaryPresentation::Badge => dokime::domain::model::documents::wire::DocumentSummaryPresentation::Badge,
    }
}

fn summary_state(value: native::TelemetrySummaryState) -> dokime::domain::model::documents::wire::DocumentSummaryState {
    match value {
        native::TelemetrySummaryState::Ok => dokime::domain::model::documents::wire::DocumentSummaryState::Ok,
        native::TelemetrySummaryState::Warning => dokime::domain::model::documents::wire::DocumentSummaryState::Warning,
        native::TelemetrySummaryState::Critical => dokime::domain::model::documents::wire::DocumentSummaryState::Critical,
        native::TelemetrySummaryState::Muted => dokime::domain::model::documents::wire::DocumentSummaryState::Muted,
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;

    fn summary(
        id: &str, value: Option<dokime::domain::model::values::Value>,
    ) -> (native::DocumentCardDefinition, native::capture::CardBinding) {
        let tile = native::TelemetrySummaryTileDefinition::new("Value", dokime::expressions::parameter(id));
        let mut group = native::TelemetrySummaryGroupDefinition::new("fixture.group", "Fixture");
        let result = value
            .map(|value| native::capture::CardBindingResult::Summary {
                groups: vec![native::TelemetrySummaryGroupPayload {
                    metadata: group.metadata.clone(),
                    tiles: vec![native::TelemetrySummaryTilePayload {
                        metadata: tile.metadata.clone(),
                        value: Some(value),
                        unavailable: None,
                        stat: tile.stat.clone(),
                        format: tile.format.clone(),
                        presentation: tile.presentation.clone(),
                        unit: tile.unit.clone(),
                        state: None,
                    }],
                }],
            })
            .unwrap_or(native::capture::CardBindingResult::Pending);
        group.tile(tile);
        let mut definition = native::TelemetrySummaryDefinition::new();
        definition.group(group);
        let mut card = native::DocumentCard::telemetry_summary(definition);
        card.metadata.identifier = dokime::domain::model::identity::Identifier::new(id).unwrap();
        let binding = native::capture::CardBinding {
            source_id: card.metadata.identifier.clone(),
            metadata: card.metadata.clone(),
            result,
        };
        (card.into(), binding)
    }

    #[test]
    fn oversized_frame_fails_only_its_card() {
        use dokime::domain::model::values::Value;
        let (large, pending_large) = summary("fixture.large", None);
        let (healthy, pending_healthy) = summary("fixture.healthy", None);
        let mut definition = native::DocumentDefinition::new("Fixture");
        for card in [large, healthy] {
            definition.layout.nodes.push(native::DocumentNode::Card(native::DocumentCardPlacement {
                card_id: card.metadata.identifier.clone(),
                layout: Default::default(),
            }));
            definition.cards.push(card);
        }
        let capture = native::capture::Capture {
            definition,
            selection: native::DocumentSelection::default(),
            cards: vec![pending_large, pending_healthy],
            repeats: BTreeMap::new(),
            sections: BTreeMap::new(),
            warnings: Vec::new(),
        };
        let document = CurrentDocumentCapture {
            capture,
            process_epoch: "fixture.epoch".into(),
            state_revision: 1,
            telemetry_cut: None,
            evidence_cut: None,
            artifacts_sha256: None,
        };
        let limit = 4096;
        let input = "payload".repeat(4096);
        let (_, healthy) = summary("fixture.healthy", Some(Value::Integer(7)));
        for value in [Value::Text(input.clone()), Value::Array(vec![Value::Null; input.len()])] {
            let mut oversized_metadata = document.clone();
            oversized_metadata.capture.definition.cards[0].metadata.attributes.insert("fixture.payload".into(), value);
            oversized_metadata.capture.cards[0].metadata = oversized_metadata.capture.definition.cards[0].metadata.clone();
            let mut metadata_stream = Stream::new(limit);
            let frames = metadata_stream
                .event(CurrentDocumentEvent::Structure {
                    document: oversized_metadata,
                    pending_repeats: false,
                    partial: false,
                })
                .unwrap();
            assert!(frames.iter().any(|frame| matches!(&frame.frame, Some(Frame::Binding(binding)) if
                binding.metadata.as_ref().is_some_and(|metadata| metadata.identifier == "fixture.large" && metadata.attributes.is_empty()) &&
                matches!(&binding.result, Some(contract::DocumentBindingResult::Failure(_))))));
            let frames = metadata_stream.event(CurrentDocumentEvent::Card(healthy.clone())).unwrap();
            assert!(frames.iter().any(|frame| matches!(&frame.frame, Some(Frame::SummaryTile(tile)) if tile.value.is_some())));
        }
        let mut stream = Stream::new(limit);
        stream
            .event(CurrentDocumentEvent::Structure {
                document,
                pending_repeats: false,
                partial: false,
            })
            .unwrap();
        let (_, large) = summary("fixture.large", Some(Value::Text(input.clone())));
        let frames = stream.event(CurrentDocumentEvent::Card(large)).unwrap();
        let failure = frames
            .iter()
            .find_map(|frame| match &frame.frame {
                Some(Frame::Binding(binding)) => match &binding.result {
                    Some(contract::DocumentBindingResult::Failure(failure)) => Some(failure),
                    _ => None,
                },
                _ => None,
            })
            .expect("oversized card failure");
        let budget = failure.budget.as_ref().unwrap();
        assert_eq!(budget.name, "frame_bytes");
        assert_eq!(budget.limit, limit as u64);
        assert!(budget.reached >= input.len() as u64 && budget.reached > budget.limit);
        assert_eq!(budget.card_id, "fixture.large");
        let frames = stream.event(CurrentDocumentEvent::Card(healthy)).unwrap();
        assert!(frames.iter().any(|frame| matches!(&frame.frame, Some(Frame::Binding(binding)) if matches!(binding.result, Some(contract::DocumentBindingResult::Summary(_))))));
        assert!(frames.iter().any(|frame| matches!(&frame.frame, Some(Frame::SummaryTile(tile)) if tile.value.is_some())));
    }
}
