/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod construction;
mod current;
mod follow;
mod handle;
mod phase;
pub(crate) mod saved;
mod stream;

/* === Re-Exports === */

pub(crate) use crate::transactions::documents::AddExecutionDocumentParts;

pub use current::{CurrentDocumentCapture, CurrentDocumentEvent, CurrentDocumentQuery, PreparedCurrentDocument};
pub use follow::ExecutionFileFollower;
pub use stream::ExecutionFileStream;

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::definition::TestCaseTemplate;
use crate::document::{
    DocumentCard, DocumentCardBody, DocumentCardLayout, DocumentCardRail, DocumentDefinition, DocumentSection, resolve_document_plot_axes,
};
use crate::execution_runtime::registry::SuiteRegistry;
use crate::telemetry::TelemetryContract;
use dokime_domain::model::execution::result::TestExecution;
use dokime_domain::model::identity::DocumentId;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::identity::Metadata;

/* === Constants === */

pub(crate) const DEFAULT_DOCUMENT_ID: &str = "default";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

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

/* === Tests === */
