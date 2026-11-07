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
use futures_util::{FutureExt, StreamExt, future::LocalBoxFuture, stream::FuturesUnordered};
use tokio::sync::mpsc;

// Local Imports
use crate::document::query::complete_card;
use crate::document::{
    DocumentCardDefinition, DocumentDefinition, DocumentSelection,
    bounded::DocumentLimits,
    capture::{Capture, CardBinding, SectionBinding},
};
use crate::{RuntimeHandle, queries::RuntimeStateSnapshot};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::DocumentId;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
pub struct CurrentDocumentQuery {
    pub structure_only: bool,
    pub cards: Vec<dokime_domain::model::identity::Identifier>,
    pub page: Option<String>,
}

pub struct PreparedCurrentDocument {
    runtime: RuntimeHandle,
    snapshot: RuntimeStateSnapshot,
    definition: DocumentDefinition,
    selection: DocumentSelection,
    continuation: Option<crate::document::page::Resume>,
}

#[derive(Clone)]
pub struct CurrentDocumentCapture {
    pub capture: crate::document::capture::Capture,
    pub process_epoch: String,
    pub state_revision: u64,
    pub telemetry_cut: Option<harmos::storage::StorageCursor>,
    pub evidence_cut: Option<harmos::storage::StorageCursor>,
    pub artifacts_sha256: Option<String>,
}

/* === Enums === */

pub enum CurrentDocumentEvent {
    Structure { document: CurrentDocumentCapture, pending_repeats: bool, partial: bool },
    Evidence { cut: harmos::storage::StorageCursor, telemetry: bool, cacheable: bool },
    Artifacts(String),
    Expansion { capture: Capture, cacheable: bool },
    Card(CardBinding),
    Page { current: String, next: Option<String> },
}

#[derive(Debug, thiserror::Error)]
enum PreparationFailure {
    #[error("document selection not found")]
    Missing,
    #[error("selected evidence capture is incomplete")]
    IncompleteCapture,
}

enum DocumentWork {
    Card { binding: CardBinding, result: crate::runtime::Result<CardBinding> },
    Expansion(crate::runtime::Result<crate::document::page::Expansion>),
}

/* === Traits === */

/* === Implementations === */

impl From<PreparationFailure> for crate::runtime::Error {
    fn from(error: PreparationFailure) -> Self {
        match error {
            PreparationFailure::Missing => Self::not_found("document_not_found", error.to_string()),
            PreparationFailure::IncompleteCapture => Self::conflict("document_capture_incomplete", error.to_string()),
        }
    }
}

impl RuntimeHandle {
    pub fn prepare_current_cycle_document(
        &self, cycle_id: &TestCycleId, document_id: &DocumentId,
    ) -> crate::runtime::Result<PreparedCurrentDocument> {
        if document_id.as_str() != super::DEFAULT_DOCUMENT_ID {
            return Err(PreparationFailure::Missing.into());
        }
        let snapshot = self.watch_state().latest();
        let cycle = snapshot.state().cycles.get(cycle_id).ok_or(PreparationFailure::Missing)?;
        let mut selection = DocumentSelection {
            scope: RecordScope::default().with_cycle(cycle_id.clone()),
        };
        selection.scope.session_id = cycle.session_id.clone();
        crate::document::input::Context::ordered(&cycle.execution_ids, &snapshot.state().executions, &selection.scope)
            .ok_or(PreparationFailure::IncompleteCapture)?;
        let definition = super::cycle_document(cycle.metadata.label.clone());
        Ok(PreparedCurrentDocument {
            runtime: self.clone(),
            snapshot,
            definition,
            selection,
            continuation: None,
        })
    }

    pub fn prepare_current_document(
        &self, cycle_id: &TestCycleId, execution_id: &TestExecutionId, document_id: &DocumentId,
    ) -> crate::runtime::Result<PreparedCurrentDocument> {
        let snapshot = self.watch_state().latest();
        let state = snapshot.state();
        state.cycles.get(cycle_id).ok_or(PreparationFailure::Missing)?;
        let execution = state
            .executions
            .get(execution_id)
            .filter(|execution| &execution.test_cycle_id == cycle_id)
            .ok_or(PreparationFailure::Missing)?;
        let definition = super::captured_authored_document(&state.suites, execution, document_id)?.ok_or(PreparationFailure::Missing)?;
        let selection = DocumentSelection { scope: execution.scope.clone() };
        Ok(PreparedCurrentDocument {
            runtime: self.clone(),
            snapshot,
            definition,
            selection,
            continuation: None,
        })
    }
}

impl PreparedCurrentDocument {
    pub async fn capture(self) -> crate::runtime::Result<CurrentDocumentCapture> {
        self.capture_inner(None, CurrentDocumentQuery::default()).await
    }

    pub async fn capture_stream(self, sender: mpsc::Sender<CurrentDocumentEvent>) -> crate::runtime::Result<CurrentDocumentCapture> {
        self.capture_inner(Some(sender), CurrentDocumentQuery::default()).await
    }

    pub async fn query_stream(
        self, query: CurrentDocumentQuery, sender: mpsc::Sender<CurrentDocumentEvent>,
    ) -> crate::runtime::Result<CurrentDocumentCapture> {
        self.capture_inner(Some(sender), query).await
    }

    async fn capture_inner(
        mut self, sender: Option<mpsc::Sender<CurrentDocumentEvent>>, query: CurrentDocumentQuery,
    ) -> crate::runtime::Result<CurrentDocumentCapture> {
        if let Some(token) = &query.page {
            let resumed = crate::document::page::resume(&self.runtime, token, &self.selection, &self.definition.document_id).await?;
            self.definition = resumed.saved.definition.clone();
            self.continuation = Some(resumed);
        }
        if query.structure_only && !query.cards.is_empty() {
            return Err(crate::runtime::Error::conflict("document_query_mode", "structure requests cannot select card results"));
        }
        let requested =
            |id: &dokime_domain::model::identity::Identifier| !query.structure_only && (query.cards.is_empty() || query.cards.contains(id));
        let limits = DocumentLimits::default();
        let input = crate::document::input::BorrowedInput {
            definition: &self.definition,
            selection: &self.selection,
            data: self.query_data()?,
        };
        let scan = crate::document::resolve::RepeatDefinitions::new([&self.definition])?;
        let pending_repeats = !scan.is_empty();
        let empty_repeats = scan.finish();
        let mut queries = Vec::new();
        let mut capture = crate::document::resolve::capture_with_queries(input, limits, Some(&empty_repeats), Some(&mut queries))?;
        retain_pending_sections(&capture.definition.layout.nodes, &capture.repeats, &mut capture.sections);
        let telemetry = crate::document::binding::telemetry_needed(&self.definition, self.query_data()?.context);
        let missing = query.cards.iter().any(|id| !capture.cards.iter().any(|card| &card.metadata.identifier == id));
        let expand = pending_repeats && (query.cards.is_empty() || missing);
        let artifact_input =
            queries.iter().filter(|card| requested(&card.metadata.identifier)).any(crate::document::binding::card_artifacts_needed)
                || (expand && crate::document::binding::repeat_artifacts_needed(&self.definition));
        let mut document = CurrentDocumentCapture {
            capture,
            process_epoch: self
                .continuation
                .as_ref()
                .map_or_else(|| self.snapshot.state().process_epoch().to_string(), |resume| resume.saved.process_epoch.clone()),
            state_revision: self.continuation.as_ref().map_or(self.snapshot.revision(), |resume| resume.saved.state_revision),
            telemetry_cut: None,
            evidence_cut: None,
            artifacts_sha256: None,
        };
        if let Some(sender) = &sender {
            sender
                .send(CurrentDocumentEvent::Structure {
                    document: document.clone(),
                    pending_repeats,
                    partial: query.structure_only || !query.cards.is_empty(),
                })
                .await
                .map_err(|_| crate::runtime::Error::cancelled("document_cancelled", "document receiver closed"))?;
        }
        let at = match &self.continuation {
            Some(resumed) => resumed.cut.at.clone(),
            None => self.runtime.saved_through(self.snapshot.revision()).await?,
        };
        document.evidence_cut = Some(at.clone());
        document.telemetry_cut = telemetry.then(|| at.clone());
        if let Some(sender) = &sender {
            sender
                .send(CurrentDocumentEvent::Evidence {
                    cut: at.clone(),
                    telemetry,
                    cacheable: !artifact_input && !missing,
                })
                .await
                .map_err(|_| crate::runtime::Error::cancelled("document_cancelled", "document receiver closed"))?;
        }
        let mut context = crate::document::query::Context::new(&self.runtime, &self.snapshot, &self.selection, &at);
        if let Some(resume) = &self.continuation {
            context.environment.journal = Some((&resume.saved.process_epoch, resume.saved.state_revision));
        }
        context.validate_membership()?;
        let pending = FuturesUnordered::<LocalBoxFuture<'_, DocumentWork>>::new();
        for card in queries {
            if !requested(&card.metadata.identifier) {
                continue;
            }
            let binding = document
                .capture
                .cards
                .iter()
                .find(|binding| binding.metadata.identifier == card.metadata.identifier)
                .expect("query binding exists")
                .clone();
            pending.push(self.card_work(card, binding, &context));
        }
        if expand {
            pending.push(async { DocumentWork::Expansion(self.expand(&context).await) }.boxed_local());
        }
        let mut pending = pending;
        while let Some(work) = pending.next().await {
            if document.artifacts_sha256.is_none()
                && let Some(digest) = context.artifact_digest()
            {
                document.artifacts_sha256 = Some(digest.to_owned());
                if let Some(sender) = &sender {
                    sender
                        .send(CurrentDocumentEvent::Artifacts(digest.to_owned()))
                        .await
                        .map_err(|_| crate::runtime::Error::cancelled("document_cancelled", "document receiver closed"))?;
                }
            }
            match work {
                DocumentWork::Card { binding, result } => {
                    let completed = complete_card(binding, result, limits);
                    let target = document
                        .capture
                        .cards
                        .iter_mut()
                        .find(|card| card.metadata.identifier == completed.metadata.identifier)
                        .expect("pending card remains in capture");
                    *target = completed.clone();
                    if let Some(sender) = &sender {
                        sender
                            .send(CurrentDocumentEvent::Card(completed))
                            .await
                            .map_err(|_| crate::runtime::Error::cancelled("document_cancelled", "document receiver closed"))?;
                    }
                }
                DocumentWork::Expansion(result) => {
                    let (mut expanded, mut queries, next) = match result {
                        Ok(value) => (value.capture, value.queries, value.next),
                        Err(error) => {
                            let mut failed = document.capture.clone();
                            fail_repeat_sections(
                                &failed.definition.layout.nodes,
                                &failed.repeats,
                                &mut failed.sections,
                                &error.to_string(),
                            );
                            (failed, Vec::new(), None)
                        }
                    };
                    let mut saved = crate::document::page::Saved {
                        definition: self.definition.clone(),
                        selection: self.selection.clone(),
                        context: self.query_data()?.context.capture(),
                        cursor: self.continuation.as_ref().map_or_else(Default::default, |resume| resume.saved.cursor.clone()),
                        process_epoch: document.process_epoch.clone(),
                        state_revision: document.state_revision,
                    };
                    let cut = self.continuation.as_ref().map(|resume| &resume.cut);
                    let tokens = async {
                        let current = crate::document::page::token(&self.runtime, &saved, &at, cut).await?;
                        let next = if let Some(cursor) = next {
                            saved.cursor = cursor;
                            Some(crate::document::page::token(&self.runtime, &saved, &at, cut).await?)
                        } else {
                            None
                        };
                        Ok::<_, crate::runtime::Error>((current, next))
                    }
                    .await;
                    match tokens {
                        Ok((current, next)) => {
                            if let Some(sender) = &sender {
                                sender
                                    .send(CurrentDocumentEvent::Page { current, next })
                                    .await
                                    .map_err(|_| crate::runtime::Error::cancelled("document_cancelled", "document receiver closed"))?;
                            }
                        }
                        Err(error) => {
                            expanded = document.capture.clone();
                            fail_repeat_sections(
                                &expanded.definition.layout.nodes,
                                &expanded.repeats,
                                &mut expanded.sections,
                                &error.to_string(),
                            );
                            queries.clear();
                        }
                    }
                    expanded
                        .cards
                        .retain(|card| !document.capture.cards.iter().any(|old| old.metadata.identifier == card.metadata.identifier));
                    let cacheable = context.artifact_digest().is_some()
                        || queries
                            .iter()
                            .filter(|card| requested(&card.metadata.identifier))
                            .all(|card| !crate::document::binding::card_artifacts_needed(card));
                    for card in queries {
                        if !requested(&card.metadata.identifier) {
                            continue;
                        }
                        if let Some(binding) = expanded.cards.iter().find(|binding| binding.metadata.identifier == card.metadata.identifier)
                        {
                            pending.push(self.card_work(card, binding.clone(), &context));
                        }
                    }
                    document.capture.repeats = expanded.repeats.clone();
                    document.capture.sections = expanded.sections.clone();
                    document.capture.cards.extend(expanded.cards.iter().cloned());
                    if let Some(sender) = &sender {
                        sender
                            .send(CurrentDocumentEvent::Expansion { capture: expanded, cacheable })
                            .await
                            .map_err(|_| crate::runtime::Error::cancelled("document_cancelled", "document receiver closed"))?;
                    }
                }
            }
        }
        for id in &query.cards {
            if !document.capture.cards.iter().any(|card| &card.metadata.identifier == id) {
                return Err(crate::runtime::Error::not_found("document_card_not_found", format!("document card {id} was not found")));
            }
        }
        Ok(document)
    }

    fn query_data(&self) -> crate::runtime::Result<crate::document::input::BorrowedData<'_>> {
        Ok(crate::document::input::BorrowedData {
            context: match &self.continuation {
                Some(resume) => crate::document::input::Context::captured(&resume.saved.context, &self.selection.scope),
                None => binding_context(&self.snapshot, &self.definition, &self.selection)?,
            },
            records: Default::default(),
            telemetry: &[],
            artifacts: crate::document::input::Artifacts::Native(&[]),
        })
    }

    fn card_work<'a>(
        &'a self, card: DocumentCardDefinition, binding: CardBinding, context: &'a crate::document::query::Context<'a>,
    ) -> LocalBoxFuture<'a, DocumentWork> {
        async move {
            let result = match self.query_data() {
                Ok(data) => context.card(card, binding.clone(), data).await,
                Err(error) => Err(error),
            };
            DocumentWork::Card { binding, result }
        }
        .boxed_local()
    }

    async fn expand(&self, context: &crate::document::query::Context<'_>) -> crate::runtime::Result<crate::document::page::Expansion> {
        let at = context.at;
        let artifacts = crate::document::binding::repeat_artifacts_needed(&self.definition);
        let mut data = self.query_data()?;
        data.artifacts = crate::document::input::Artifacts::Native(if artifacts { context.artifacts().await? } else { &[] });
        let input = crate::document::input::BorrowedInput {
            definition: &self.definition,
            selection: &self.selection,
            data,
        };
        let cursor = self.continuation.as_ref().map_or_else(Default::default, |resume| resume.saved.cursor.clone());
        crate::document::page::expand(context.environment, input, at, cursor).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn retain_pending_sections(
    nodes: &[crate::document::DocumentNode],
    repeats: &std::collections::BTreeMap<dokime_domain::model::identity::Identifier, Vec<crate::document::capture::RepeatInstance>>,
    sections: &mut std::collections::BTreeMap<dokime_domain::model::identity::Identifier, SectionBinding>,
) -> bool {
    let mut pending = false;
    for node in nodes {
        if let crate::document::DocumentNode::Section(section) = node {
            if repeats.contains_key(&section.node_id) {
                pending = true;
            } else if retain_pending_sections(&section.children, repeats, sections) {
                sections.insert(
                    section.node_id.clone(),
                    SectionBinding {
                        metadata: section.metadata.clone(),
                        unavailable: None,
                    },
                );
                pending = true;
            }
        }
    }
    pending
}

fn fail_repeat_sections(
    nodes: &[crate::document::DocumentNode],
    repeats: &std::collections::BTreeMap<dokime_domain::model::identity::Identifier, Vec<crate::document::capture::RepeatInstance>>,
    sections: &mut std::collections::BTreeMap<dokime_domain::model::identity::Identifier, SectionBinding>, message: &str,
) {
    let maximum = DocumentLimits::default().unavailable_reason_bytes.min(message.len());
    let mut end = maximum;
    while !message.is_char_boundary(end) {
        end -= 1;
    }
    let message = &message[..end];
    for node in nodes {
        if let crate::document::DocumentNode::Section(section) = node {
            if repeats.contains_key(&section.node_id) {
                sections.insert(
                    section.node_id.clone(),
                    SectionBinding {
                        metadata: section.metadata.clone(),
                        unavailable: Some(message.to_owned()),
                    },
                );
            } else {
                fail_repeat_sections(&section.children, repeats, sections, message);
            }
        }
    }
}

fn binding_context<'a>(
    snapshot: &'a RuntimeStateSnapshot, definition: &DocumentDefinition, selection: &'a DocumentSelection,
) -> crate::runtime::Result<crate::document::input::Context<'a>> {
    use crate::document::input::Context;
    if !crate::document::binding::context_needed(definition) {
        return Ok(Context::default());
    }
    if let Some(id) = &selection.scope.test_execution_id {
        let execution = snapshot.state().executions.get(id).ok_or(PreparationFailure::Missing)?;
        return Ok(Context::new(std::slice::from_ref(execution), &selection.scope));
    }
    let cycle = snapshot
        .state()
        .cycles
        .get(selection.scope.test_cycle_id.as_ref().ok_or(PreparationFailure::Missing)?)
        .ok_or(PreparationFailure::Missing)?;
    Context::ordered(&cycle.execution_ids, &snapshot.state().executions, &selection.scope)
        .ok_or_else(|| PreparationFailure::IncompleteCapture.into())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::capture::CardBindingResult;

    #[test]
    fn output_budget_is_local_to_its_card() {
        let make = |id: &str| {
            let card = crate::document::DocumentCard::text(id, "Fixture", "");
            CardBinding {
                source_id: card.metadata.identifier.clone(),
                metadata: card.metadata,
                result: CardBindingResult::Static,
            }
        };
        let healthy = make("fixture.healthy");
        let mut oversized = make("fixture.large");
        oversized.result = CardBindingResult::Summary {
            groups: (0..64)
                .map(|index| crate::document::TelemetrySummaryGroupPayload {
                    metadata: crate::document::DocumentCard::text(format!("fixture.group.{index}"), "Fixture", "").metadata,
                    tiles: Vec::new(),
                })
                .collect(),
        };
        let limit = serde_json::to_vec(&healthy).unwrap().len() * 4;
        let reached = serde_json::to_vec(&oversized).unwrap().len();
        let limits = DocumentLimits {
            materialized_bytes: limit,
            ..Default::default()
        };
        let failed = complete_card(oversized.clone(), Ok(oversized.clone()), limits);
        assert!(serde_json::to_vec(&failed).unwrap().len() <= limit);
        let CardBindingResult::Failed { failure } = failed.result else { panic!("expected a failed card") };
        let budget = failure.budget.unwrap();
        assert_eq!(failure.code, "materialized_bytes");
        assert_eq!((budget.limit, budget.reached), (limit as u64, reached as u64));
        assert_eq!(budget.card_id, oversized.metadata.identifier.as_str());
        assert_eq!(complete_card(healthy.clone(), Ok(healthy.clone()), limits), healthy);
    }
}
