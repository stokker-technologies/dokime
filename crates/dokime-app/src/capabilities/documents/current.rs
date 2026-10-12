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
use std::num::NonZeroUsize;

// Third Party Imports

// Local Imports
use super::acquire::{DocumentAcquisitionFailure, acquire};
use crate::document::{
    DocumentCardBody, DocumentDefinition, DocumentResolveInput, DocumentSelection,
    bounded::{DocumentLimits, capture_document_borrowed, document_size},
};
use crate::{RuntimeHandle, queries::RuntimeStateSnapshot};
use dokime::core::{DocumentId, RecordScope, TestCycleId, TestExecutionId};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct PreparedCurrentDocument {
    runtime: RuntimeHandle,
    snapshot: RuntimeStateSnapshot,
    definition: DocumentDefinition,
    selection: DocumentSelection,
}

pub struct CurrentDocumentCapture {
    pub capture: crate::document::capture::Capture,
    pub process_epoch: String,
    pub state_revision: u64,
    pub telemetry_cut: Option<CurrentDocumentTelemetryCut>,
    pub artifacts_sha256: Option<String>,
}

/* === Enums === */
pub enum CurrentDocumentTelemetryCut {
    Window {
        delivered: u64,
    },
    Series {
        epoch: u64,
        delivered: u64,
        ordinal: u64,
    },
}

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub fn prepare_current_cycle_document(
        &self,
        cycle_id: &TestCycleId,
        document_id: &DocumentId,
    ) -> crate::Result<PreparedCurrentDocument> {
        if document_id.as_str() != super::DEFAULT_DOCUMENT_ID {
            return Err(DocumentAcquisitionFailure::Missing.into());
        }
        let snapshot = self.watch_state().latest();
        let cycle = snapshot
            .state()
            .cycles
            .get(cycle_id)
            .ok_or(DocumentAcquisitionFailure::Missing)?;
        if cycle.execution_ids.len() > super::acquire::SOURCE_ROWS {
            return Err(DocumentAcquisitionFailure::Limit.into());
        }
        let limits = DocumentLimits::default();
        document_size(&cycle.metadata, 64 * 1024, limits.depth)
            .map_err(|_| DocumentAcquisitionFailure::Limit)?;
        let scopes = harmos::btree_map_memory::<&TestExecutionId, ()>(cycle.execution_ids.len());
        if scopes > limits.source_bytes {
            return Err(DocumentAcquisitionFailure::Limit.into());
        }
        let _seen = self
            .storage()
            .prepare_production(NonZeroUsize::new(scopes.max(1)).unwrap())?;
        let mut selection = DocumentSelection {
            scope: RecordScope::default().with_cycle(cycle_id.clone()),
        };
        selection.scope.session_id = cycle.session_id.clone();
        crate::document::input::Context::ordered(
            &cycle.execution_ids,
            &snapshot.state().executions,
            &selection.scope,
        )
        .ok_or(DocumentAcquisitionFailure::IncompleteCapture)?;
        let definition = super::cycle_document(cycle.metadata.label.clone());
        Ok(PreparedCurrentDocument {
            runtime: self.clone(),
            snapshot,
            definition,
            selection,
        })
    }

    pub fn prepare_current_document(
        &self,
        cycle_id: &TestCycleId,
        execution_id: &TestExecutionId,
        document_id: &DocumentId,
    ) -> crate::Result<PreparedCurrentDocument> {
        let snapshot = self.watch_state().latest();
        let state = snapshot.state();
        let cycle = state
            .cycles
            .get(cycle_id)
            .ok_or(DocumentAcquisitionFailure::Missing)?;
        let execution = state
            .executions
            .get(execution_id)
            .filter(|execution| &execution.test_cycle_id == cycle_id)
            .ok_or(DocumentAcquisitionFailure::Missing)?;
        let definitions = &execution.documents;
        if definitions.len() > 10_000 {
            return Err(DocumentAcquisitionFailure::Limit.into());
        }
        let limits = DocumentLimits::default();
        if let Some(definition) = definitions
            .values()
            .find(|definition| &definition.document_id == document_id)
        {
            document_size(definition, limits.source_bytes, limits.depth)
                .map_err(|_| DocumentAcquisitionFailure::Limit)?;
        }
        document_size(&cycle.metadata, 64 * 1024, 32)
            .map_err(|_| DocumentAcquisitionFailure::Limit)?;
        {
            if state
                .suites
                .suites()
                .flat_map(|suite| suite.suite.test_case_templates.iter())
                .take(100_001)
                .count()
                > 100_000
            {
                return Err(DocumentAcquisitionFailure::Limit.into());
            }
            document_size(&execution.snapshot.case().metadata, 64 * 1024, 32)
                .map_err(|_| DocumentAcquisitionFailure::Limit)?;
            if let Some(template) = super::selected_execution_template(execution) {
                document_size(&template.metadata, 64 * 1024, 32)
                    .map_err(|_| DocumentAcquisitionFailure::Limit)?;
            }
        }
        let definition = super::captured_authored_document(&state.suites, execution, document_id)
            .ok_or(DocumentAcquisitionFailure::Missing)?;
        let selection = DocumentSelection {
            scope: execution.scope.clone(),
        };
        Ok(PreparedCurrentDocument {
            runtime: self.clone(),
            snapshot,
            definition,
            selection,
        })
    }
}

impl PreparedCurrentDocument {
    pub async fn capture(self) -> crate::Result<CurrentDocumentCapture> {
        let mut artifacts = false;
        for body in self.definition.cards.iter().map(|card| &card.body).chain(
            self.definition
                .sections
                .iter()
                .flat_map(|section| section.cards.iter().map(|card| &card.body)),
        ) {
            match body {
                DocumentCardBody::Artifacts | DocumentCardBody::Media => artifacts = true,
                _ => {}
            }
        }
        let telemetry = {
            let context = binding_context(
                &self.runtime,
                &self.snapshot,
                &self.definition,
                &self.selection,
            )?;
            crate::document::binding::telemetry_needed(&self.definition, context)
        };
        artifacts |= crate::document::binding::artifacts_needed(&self.definition);
        let records = crate::document::binding::records_needed(&self.definition);
        let acquired = acquire(
            &self.runtime,
            &self.selection,
            &self.snapshot,
            records,
            telemetry,
            artifacts,
        )
        .await?;
        let input = DocumentResolveInput {
            definition: self.definition,
            selection: self.selection,
            data: acquired.data,
        };
        let mut borrowed = input.borrowed();
        borrowed.data.context = binding_context(
            &self.runtime,
            &self.snapshot,
            &input.definition,
            &input.selection,
        )?;
        if let Some(id) = &input.selection.scope.test_execution_id {
            let execution = self
                .snapshot
                .state()
                .executions
                .get(id)
                .ok_or(DocumentAcquisitionFailure::Missing)?;
            borrowed.data.records = crate::document::input::Records::Captured {
                rows: &acquired.records,
                scopes: execution.records.subscribed_records(),
            };
        }
        borrowed.data.artifacts = crate::document::input::Artifacts::Native(
            acquired.artifacts.as_deref().map_or(&[], Vec::as_slice),
        );
        let capture =
            capture_document_borrowed(borrowed, DocumentLimits::default()).map_err(|error| {
                match error {
                    dokime::Error::Document(crate::document::DocumentFailure::Limit) => {
                        DocumentAcquisitionFailure::Limit
                    }
                    error => DocumentAcquisitionFailure::Source(error.to_string()),
                }
            })?;
        Ok(CurrentDocumentCapture {
            capture,
            process_epoch: self.snapshot.state().process_epoch().to_string(),
            state_revision: self.snapshot.revision(),
            telemetry_cut: acquired.telemetry_cut.map(|cut| match cut {
                harmos::ReadCut::Window { delivered } => CurrentDocumentTelemetryCut::Window {
                    delivered: delivered.0,
                },
                harmos::ReadCut::Series {
                    epoch,
                    delivered,
                    ordinal,
                } => CurrentDocumentTelemetryCut::Series {
                    epoch,
                    delivered: delivered.0,
                    ordinal,
                },
            }),
            artifacts_sha256: acquired.artifacts_sha256,
        })
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn binding_context<'a>(
    runtime: &RuntimeHandle,
    snapshot: &'a RuntimeStateSnapshot,
    definition: &DocumentDefinition,
    selection: &'a DocumentSelection,
) -> crate::Result<crate::document::input::Context<'a>> {
    use crate::document::input::Context;
    if !crate::document::binding::context_needed(definition) {
        return Ok(Context::default());
    }
    if let Some(id) = &selection.scope.test_execution_id {
        let execution = snapshot
            .state()
            .executions
            .get(id)
            .ok_or(DocumentAcquisitionFailure::Missing)?;
        return Ok(Context::new(
            std::slice::from_ref(execution),
            &selection.scope,
        ));
    }
    let cycle = snapshot
        .state()
        .cycles
        .get(
            selection
                .scope
                .test_cycle_id
                .as_ref()
                .ok_or(DocumentAcquisitionFailure::Missing)?,
        )
        .ok_or(DocumentAcquisitionFailure::Missing)?;
    let bytes = harmos::btree_map_memory::<&TestExecutionId, ()>(cycle.execution_ids.len());
    if bytes > DocumentLimits::default().source_bytes {
        return Err(DocumentAcquisitionFailure::Limit.into());
    }
    let _seen = runtime
        .storage()
        .prepare_production(NonZeroUsize::new(bytes.max(1)).unwrap())?;
    Context::ordered(
        &cycle.execution_ids,
        &snapshot.state().executions,
        &selection.scope,
    )
    .ok_or_else(|| DocumentAcquisitionFailure::IncompleteCapture.into())
}

/* === Tests === */
