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
mod handle;
mod phase;
pub(crate) mod saved;
mod stream;

/* === Re-Exports === */
pub(crate) use crate::transactions::documents::AddExecutionDocumentParts;
pub(crate) use construction::{construction_memory, cycle_memory};
pub use current::{CurrentDocumentCapture, CurrentDocumentTelemetryCut, PreparedCurrentDocument};
pub use stream::ExecutionFileStream;

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::definition::{TestCaseTemplate, TestCycleInstance};
use crate::document::{
    DocumentCard, DocumentCardBody, DocumentCardLayout, DocumentCardRail, DocumentDataInput,
    DocumentDefinition, DocumentPayload, DocumentResolveInput, DocumentSection, DocumentSelection,
    resolve_document_plot_axes,
};
use crate::execution::TestExecution;
use crate::records::RecordEntry;
use crate::{
    error::{Error, Result},
    execution::CycleView,
    execution_runtime::registry::SuiteRegistry,
};
use dokime::artifacts::ArtifactRecord;
use dokime::core::{CaseId, DocumentId, Identifier, Metadata, RecordScope};
use dokime::telemetry::{TelemetryContract, TelemetrySample};

/* === Constants === */
pub(crate) const DEFAULT_DOCUMENT_ID: &str = "default";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug, Default)]
pub(crate) struct RuntimeDocumentData {
    pub(crate) records: Vec<RecordEntry>,
    pub(crate) telemetry: Vec<TelemetrySample>,
    pub(crate) artifacts: Vec<ArtifactRecord>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeDocumentData {
    pub(crate) fn into_input(self) -> DocumentDataInput {
        DocumentDataInput {
            records: self.records,
            telemetry: self.telemetry,
            artifacts: self.artifacts,
        }
    }
}

/* === Public Functions === */
pub(crate) fn resolve_cycle_document(
    suites: &SuiteRegistry,
    cycle: &TestCycleInstance,
    document_id: &DocumentId,
    mut selection: DocumentSelection,
    data: RuntimeDocumentData,
) -> Result<Option<DocumentPayload>> {
    let cycle_snapshot = &cycle.cycle;
    if selection.scope.test_cycle_id.is_none() {
        selection.scope = selection
            .scope
            .with_cycle(cycle_snapshot.test_cycle_id.clone());
    }
    validate_selection(cycle_snapshot, &selection.scope)?;
    normalize_execution_selection(cycle_snapshot, &mut selection.scope)?;

    let Some(definition) = authored_document(cycle, document_id, &selection.scope)? else {
        return Ok(None);
    };
    let definition = with_resolved_plot_axes(definition, suites);
    let input = DocumentResolveInput {
        definition,
        selection,
        data: data.into_input(),
    };

    crate::document::resolve_document(input)
        .map(Some)
        .map_err(Into::into)
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
    suites: &SuiteRegistry,
    execution: &TestExecution,
    document_id: &DocumentId,
) -> Option<DocumentDefinition> {
    let definitions = &execution.documents;
    // Instance registration is keyed by the definition identity, not the outer
    // map key. Preserve its first-definition-wins behavior for native snapshots.
    let authored = definitions
        .values()
        .find(|definition| &definition.document_id == document_id)
        .cloned();
    let completed = complete_execution_document(authored, document_id, || {
        execution_document_label(execution)
    });

    completed.map(|definition| with_resolved_plot_axes(definition, suites))
}

pub(crate) fn with_resolved_plot_axes(
    mut definition: DocumentDefinition,
    suites: &SuiteRegistry,
) -> DocumentDefinition {
    fn unresolved(body: &DocumentCardBody) -> bool {
        matches!(body, DocumentCardBody::Plot { plot } if plot.x_axis.is_none())
    }

    let pending = definition
        .cards
        .iter()
        .map(|card| &card.body)
        .chain(
            definition
                .sections
                .iter()
                .flat_map(|section| section.cards.iter().map(|card| &card.body)),
        )
        .any(unresolved);
    if !pending {
        return definition;
    }

    let count = suites
        .suites()
        .map(|suite| suite.telemetry_contracts.len())
        .sum();
    let mut contracts = Vec::with_capacity(count);
    contracts.extend(
        suites
            .suites()
            .flat_map(|suite| suite.telemetry_contracts.iter())
            .map(TelemetryContract::normalized),
    );
    resolve_document_plot_axes(&mut definition, &contracts);

    definition
}

pub(crate) fn selected_execution_template(execution: &TestExecution) -> Option<&TestCaseTemplate> {
    execution.snapshot.template()
}

/* === Private Functions === */
fn authored_document(
    cycle: &TestCycleInstance,
    document_id: &DocumentId,
    scope: &RecordScope,
) -> Result<Option<DocumentDefinition>> {
    let id = scope
        .test_execution_id
        .as_ref()
        .ok_or_else(|| Error::Operation("document selection requires an execution".into()))?;
    let execution = selected_execution(&cycle.cycle, id)?;
    Ok(authored_execution_document(cycle, execution, document_id))
}

fn authored_execution_document(
    cycle: &TestCycleInstance,
    execution: &TestExecution,
    document_id: &DocumentId,
) -> Option<DocumentDefinition> {
    let instance = cycle.get_execution(&execution.test_execution_id)?;
    complete_execution_document(
        instance
            .documents
            .get_instance(document_id)
            .map(|instance| instance.definition.clone()),
        document_id,
        || execution_document_label(execution),
    )
}

fn execution_document_label(execution: &TestExecution) -> String {
    selected_execution_template(execution)
        .map(|template| template.metadata.label.clone())
        .unwrap_or_else(|| execution.snapshot.case().metadata.label.clone())
}

fn complete_execution_document(
    authored: Option<DocumentDefinition>,
    document_id: &DocumentId,
    label: impl FnOnce() -> String,
) -> Option<DocumentDefinition> {
    let is_default = document_id.as_str() == DEFAULT_DOCUMENT_ID;
    let mut document = match authored {
        Some(document) => document,
        None if is_default => default_document(label()),
        None => return None,
    };
    if is_default {
        for section in built_in_execution_result_sections().into_iter().rev() {
            document.prepend_section(section);
        }
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

fn built_in_card(
    identifier: &str,
    label: &str,
    order: i32,
    body: DocumentCardBody,
) -> DocumentCard {
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
    Metadata::new(
        Identifier::new(identifier).expect("built-in document identifiers are valid"),
        label,
        description,
    )
}

fn validate_selection(cycle: &CycleView, scope: &RecordScope) -> Result<()> {
    let selected_case_id = if let Some(case_id) = &scope.test_case_id {
        if !cycle
            .cases()
            .any(|test_case| &test_case.test_case_id == case_id)
        {
            return Err(Error::Operation(format!(
                "cycle test case {case_id} not found"
            )));
        }
        Some(case_id.clone())
    } else {
        None
    };

    if let Some(test_execution_id) = &scope.test_execution_id {
        let execution_case_id = selected_execution_case_id(cycle, test_execution_id)?;
        if let Some(case_id) = selected_case_id
            && case_id != execution_case_id
        {
            return Err(Error::Operation(format!(
                "cycle execution {test_execution_id} does not belong to test case {case_id}"
            )));
        }
    }

    Ok(())
}

fn normalize_execution_selection(cycle: &CycleView, scope: &mut RecordScope) -> Result<()> {
    let test_execution_id = scope
        .test_execution_id
        .clone()
        .ok_or_else(|| Error::Operation("document selection requires an execution".into()))?;
    let execution = selected_execution(cycle, &test_execution_id)?;
    if scope.test_case_id.is_none() {
        scope.test_case_id = Some(execution.test_case_id.clone());
    }
    if scope.template_id.is_none() {
        scope.template_id = execution.scope.template_id.clone();
    }
    Ok(())
}

fn selected_execution_case_id(
    cycle: &CycleView,
    test_execution_id: &dokime::core::TestExecutionId,
) -> Result<CaseId> {
    Ok(selected_execution(cycle, test_execution_id)?
        .test_case_id
        .clone())
}

fn selected_execution<'a>(
    cycle: &'a CycleView,
    test_execution_id: &dokime::core::TestExecutionId,
) -> Result<&'a TestExecution> {
    cycle
        .test_executions
        .iter()
        .find(|execution| &execution.test_execution_id == test_execution_id)
        .ok_or_else(|| Error::Operation(format!("cycle execution {test_execution_id} not found")))
}

/* === Tests === */
