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
use serde::{Deserialize, Serialize};

// Local Imports
use crate::definition::TestCycleInstance;
use crate::document::DocumentInstance;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::result::ExecutionStatus;
use dokime_domain::model::identity::DocumentId;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RuntimeDocumentFrame {
    pub documents: Vec<RuntimeDocumentRecord>,
    pub total_documents: usize,
    pub returned_documents: usize,
    pub truncated: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RuntimeDocumentRecord {
    pub document_id: DocumentId,
    pub scope: RecordScope,
    pub metadata: Metadata,
    pub sections: usize,
    pub cards: usize,
    pub has_payload: bool,
    pub updated_at: Timestamp,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeDocumentRecord {
    fn from_instance(instance: &DocumentInstance, scope: RecordScope, has_payload: bool) -> Self {
        let sections = instance.definition.section_count();
        let cards = instance.definition.card_count();

        Self {
            document_id: instance.document_id().clone(),
            scope,
            metadata: instance.definition.metadata.clone(),
            sections,
            cards,
            has_payload,
            updated_at: instance.updated_at.clone(),
        }
    }
}

/* === Public Functions === */

pub(crate) fn document_frame<'a>(cycles: impl Iterator<Item = &'a TestCycleInstance>, scope: Option<&RecordScope>) -> RuntimeDocumentFrame {
    let documents = cycles.flat_map(|cycle| cycle_document_records(cycle, scope)).collect::<Vec<_>>();

    RuntimeDocumentFrame {
        total_documents: documents.len(),
        returned_documents: documents.len(),
        documents,
        truncated: false,
    }
}

/* === Private Functions === */

fn cycle_document_records(cycle: &TestCycleInstance, selected_scope: Option<&RecordScope>) -> Vec<RuntimeDocumentRecord> {
    let mut records = Vec::new();
    for execution in cycle.executions() {
        let execution_scope = execution.execution.scope.clone();
        if selected_scope.is_some_and(|selected| !execution_scope.is_visible_in(selected)) {
            continue;
        }

        records.extend(execution.documents.instances().map(|document| {
            RuntimeDocumentRecord::from_instance(document, execution_scope.clone(), execution_has_payload(&execution.execution.status))
        }));
    }

    records
}

fn execution_has_payload(status: &ExecutionStatus) -> bool {
    matches!(
        status,
        ExecutionStatus::Invalidated
            | ExecutionStatus::Passed
            | ExecutionStatus::Failed
            | ExecutionStatus::Blocked
            | ExecutionStatus::Cancelled
            | ExecutionStatus::Interrupted
    )
}

/* === Tests === */
