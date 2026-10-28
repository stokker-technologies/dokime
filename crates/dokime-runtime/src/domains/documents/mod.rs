/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod acquire;
mod construction;
mod current;
mod follow;
mod handle;
mod phase;
pub(crate) mod saved;
mod stream;

/* === Re-Exports === */

pub(crate) use crate::transactions::documents::AddExecutionDocumentParts;

pub use current::{CurrentDocumentCapture, PreparedCurrentDocument};
pub use follow::ExecutionFileFollower;
pub use stream::ExecutionFileStream;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::artifacts::ArtifactRecord;
use crate::definition::TestCaseTemplate;
use crate::document::{
    DocumentCard, DocumentCardBody, DocumentCardLayout, DocumentCardRail, DocumentDefinition, DocumentPayload, DocumentSection,
    DocumentSelection, resolve_document_plot_axes,
};
use crate::execution_runtime::registry::SuiteRegistry;
use crate::runtime::error::Error;
use crate::runtime::error::Result;
use crate::telemetry::{TelemetryContract, TelemetrySample};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::result::TestCycle;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::identity::DocumentId;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::records::evidence::RecordEntry;

/* === Constants === */

pub(crate) const DEFAULT_DOCUMENT_ID: &str = "default";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct RuntimeDocumentData {
    pub(crate) records: crate::storage::projection::Pages<RecordEntry>,
    pub(crate) telemetry: harmos::storage::ReadBatch<Vec<TelemetrySample>>,
    pub(crate) artifacts: harmos::storage::ReadBatch<Vec<ArtifactRecord>>,
}

pub(crate) struct RuntimeDocumentInput {
    records: Vec<harmos::storage::ReadBatch<Vec<RecordEntry>>>,
    telemetry: Vec<TelemetrySample>,
    artifacts: Vec<ArtifactRecord>,
}

struct PreparedDocument {
    definition: DocumentDefinition,
    selection: DocumentSelection,
    data: RuntimeDocumentInput,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeDocumentData {
    pub(crate) fn into_owner(self) -> Result<harmos::storage::ReadBatch<RuntimeDocumentInput>> {
        let data = self.telemetry.try_map_with(self.records, |telemetry, records| -> Result<_> { Ok((telemetry, records)) })?;
        data.try_map_with(self.artifacts, |(telemetry, records), artifacts| -> Result<_> {
            Ok(RuntimeDocumentInput { records, telemetry, artifacts })
        })
    }
}

impl RuntimeDocumentInput {
    fn borrowed(&self) -> crate::document::input::BorrowedData<'_> {
        crate::document::input::BorrowedData {
            context: Default::default(),
            records: crate::document::input::Records::Pages(&self.records),
            telemetry: &self.telemetry,
            artifacts: crate::document::input::Artifacts::Native(&self.artifacts),
        }
    }
}

/* === Public Functions === */

pub(crate) fn resolve_cycle_document(
    state: &crate::RuntimeState, cycle: &TestCycle, document_id: &DocumentId, selection: DocumentSelection, data: RuntimeDocumentData,
    limits: crate::document::bounded::DocumentLimits,
) -> Result<harmos::storage::ReadBatch<Result<Option<DocumentPayload>>>> {
    let suites = &state.suites;
    let prepared = data.into_owner()?.try_map(|mut data| -> Result<_> {
        let result = (|| -> Result<Option<PreparedDocument>> {
            let mut selection = selection;
            if selection.scope.test_cycle_id.is_none() {
                selection.scope = selection.scope.with_cycle(cycle.test_cycle_id.clone());
            }
            validate_selection(state, cycle, &selection.scope)?;
            let id = selection
                .scope
                .test_execution_id
                .as_ref()
                .ok_or_else(|| Error::Operation("document selection requires an execution".into()))?;
            let execution = selected_execution(state, cycle, id)?;
            normalize_execution_selection(execution, &mut selection.scope);
            let Some(definition) = captured_authored_document(suites, execution, document_id) else { return Ok(None) };
            data.telemetry.retain(|sample| crate::document::markers::scope_matches(sample.scope.as_ref(), &selection));
            Ok(Some(PreparedDocument { definition, selection, data }))
        })();
        Ok(result)
    })?;
    let prepared = prepared.try_map(|result| -> Result<_> {
        Ok(result.and_then(|prepared| {
            let Some(mut prepared) = prepared else { return Ok(None) };
            let input = crate::document::input::BorrowedInput {
                definition: &prepared.definition,
                selection: &prepared.selection,
                data: prepared.data.borrowed(),
            };
            crate::document::bounded::validate_native_resolution(&input, limits)?;
            prepared.definition = crate::document::resolve::normalize_definition(prepared.definition);
            Ok(Some(prepared))
        }))
    })?;
    prepared.try_map(|result| -> Result<_> {
        Ok(result.and_then(|prepared| {
            let Some(PreparedDocument { definition, selection, data }) = prepared else { return Ok(None) };
            let payload = crate::document::resolve::resolve_normalized(definition, selection, data.borrowed(), true, limits)?;
            crate::document::bounded::check_payload(payload, limits).map(Some).map_err(Into::into)
        }))
    })
}

pub(crate) fn default_document(label: impl Into<String>) -> DocumentDefinition {
    DocumentDefinition::new(DEFAULT_DOCUMENT_ID, label)
}

pub(crate) fn cycle_document(label: impl Into<String>) -> DocumentDefinition {
    let mut definition = default_document(label);
    for section in construction::sections(&construction::CYCLE_SECTIONS) {
        definition.section(section);
    }
    definition
}

pub(crate) fn captured_authored_document(
    suites: &SuiteRegistry, execution: &TestExecution, document_id: &DocumentId,
) -> Option<DocumentDefinition> {
    let definitions = &execution.documents;
    // Instance registration is keyed by the definition identity, not the outer
    // map key. Preserve its first-definition-wins behavior for native snapshots.
    let authored = definitions.values().find(|definition| &definition.document_id == document_id).cloned();
    let completed = complete_execution_document(authored, document_id, || execution_document_label(execution));

    completed.map(|definition| with_resolved_plot_axes(definition, suites))
}

pub(crate) fn with_resolved_plot_axes(mut definition: DocumentDefinition, suites: &SuiteRegistry) -> DocumentDefinition {
    fn unresolved(body: &DocumentCardBody) -> bool {
        matches!(body, DocumentCardBody::Plot { plot } if plot.x_axis.is_none())
    }

    let pending = definition
        .cards
        .iter()
        .map(|card| &card.body)
        .chain(definition.sections.iter().flat_map(|section| section.cards.iter().map(|card| &card.body)))
        .any(unresolved);
    if !pending {
        return definition;
    }

    let count = suites.suites().map(|suite| suite.telemetry_contracts.len()).sum();
    let mut contracts = Vec::with_capacity(count);
    contracts.extend(suites.suites().flat_map(|suite| suite.telemetry_contracts.iter()).map(TelemetryContract::normalized));
    resolve_document_plot_axes(&mut definition, &contracts);

    definition
}

pub(crate) fn selected_execution_template(execution: &TestExecution) -> Option<&TestCaseTemplate> {
    execution.snapshot.template()
}

/* === Private Functions === */

fn execution_document_label(execution: &TestExecution) -> String {
    selected_execution_template(execution)
        .map(|template| template.metadata.label.clone())
        .unwrap_or_else(|| execution.snapshot.case().metadata.label.clone())
}

fn complete_execution_document(
    authored: Option<DocumentDefinition>, document_id: &DocumentId, label: impl FnOnce() -> String,
) -> Option<DocumentDefinition> {
    let is_default = document_id.as_str() == DEFAULT_DOCUMENT_ID;
    let mut document = match authored {
        Some(document) => document,
        None if is_default => default_document(label()),
        None => return None,
    };
    if is_default {
        document.prepend_sections(built_in_execution_result_sections());
        for section in built_in_execution_evidence_sections() {
            document.upsert_section(section);
        }
    }
    Some(document)
}

fn built_in_execution_result_sections() -> Vec<DocumentSection> {
    construction::sections(&construction::RESULT_SECTIONS)
}

fn built_in_execution_evidence_sections() -> Vec<DocumentSection> {
    construction::sections(&construction::EVIDENCE_SECTIONS)
}

fn built_in_card(identifier: &str, label: &str, order: i32, body: DocumentCardBody) -> DocumentCard {
    DocumentCard {
        metadata: metadata(identifier, label, None),
        layout: DocumentCardLayout {
            order,
            rail: DocumentCardRail::Main,
            ..DocumentCardLayout::default()
        },
        filters: Vec::new(),
        body,
    }
}

fn metadata(identifier: &str, label: &str, description: Option<String>) -> Metadata {
    Metadata::new(Identifier::new(identifier).expect("built-in document identifiers are valid"), label, description)
}

fn validate_selection(state: &crate::RuntimeState, cycle: &TestCycle, scope: &RecordScope) -> Result<()> {
    let selected_case_id = if let Some(case_id) = &scope.test_case_id {
        if !cycle.execution_ids.iter().filter_map(|id| state.executions.get(id)).any(|execution| &execution.test_case_id == case_id) {
            return Err(Error::Operation(format!("cycle test case {case_id} not found")));
        }
        Some(case_id)
    } else {
        None
    };

    if let Some(test_execution_id) = &scope.test_execution_id {
        let execution_case_id = &selected_execution(state, cycle, test_execution_id)?.test_case_id;
        if let Some(case_id) = selected_case_id
            && case_id != execution_case_id
        {
            return Err(Error::Operation(format!("cycle execution {test_execution_id} does not belong to test case {case_id}")));
        }
    }

    Ok(())
}

fn normalize_execution_selection(execution: &TestExecution, scope: &mut RecordScope) {
    if scope.test_case_id.is_none() {
        scope.test_case_id = Some(execution.test_case_id.clone());
    }
    if scope.template_id.is_none() {
        scope.template_id = execution.scope.template_id.clone();
    }
}

fn selected_execution<'a>(
    state: &'a crate::RuntimeState, cycle: &TestCycle, test_execution_id: &dokime_domain::model::identity::TestExecutionId,
) -> Result<&'a TestExecution> {
    cycle
        .execution_ids
        .contains(test_execution_id)
        .then(|| state.executions.get(test_execution_id))
        .flatten()
        .ok_or_else(|| Error::Operation(format!("cycle execution {test_execution_id} not found")))
}

/* === Tests === */
