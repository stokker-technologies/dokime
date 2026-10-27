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
use crate::document::{DocumentInstance, DocumentPayload, DocumentSelection};
use crate::documents::RuntimeDocumentData;
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

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RuntimeResolvedDocumentFrame {
    pub document: Option<DocumentPayload>,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct RuntimeResolvedDocumentRead(harmos::storage::ReadBatch<RuntimeResolvedDocumentFrame>);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl std::ops::Deref for RuntimeResolvedDocumentRead {
    type Target = RuntimeResolvedDocumentFrame;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::fmt::Debug for RuntimeResolvedDocumentRead {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&*self.0, f)
    }
}
impl PartialEq for RuntimeResolvedDocumentRead {
    fn eq(&self, other: &Self) -> bool {
        *self.0 == *other.0
    }
}
impl Serialize for RuntimeResolvedDocumentRead {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

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

    RuntimeDocumentFrame { total_documents: documents.len(), returned_documents: documents.len(), documents, truncated: false }
}

pub(crate) fn resolved_document_frame(
    state: &crate::RuntimeState, document_id: &DocumentId, selection: DocumentSelection, data: RuntimeDocumentData,
    limits: crate::document::bounded::DocumentLimits,
) -> crate::runtime::Result<RuntimeResolvedDocumentRead> {
    let Some(test_cycle_id) = selection.scope.test_cycle_id.as_ref() else {
        return failure(data, format_args!("document query scope requires test_cycle_id"));
    };

    let Some(cycle) = state.cycles.get(test_cycle_id) else {
        return failure(data, format_args!("cycle {test_cycle_id} not found"));
    };

    let resolved = crate::documents::resolve_cycle_document(state, cycle, document_id, selection, data, limits)?;
    let message = match &*resolved {
        Err(error) => formatted_size(format_args!("{error}"))?,
        _ => 0,
    };
    let frame = resolved.try_map(|result| -> crate::runtime::Result<_> {
        Ok(match result {
            Ok(document) => RuntimeResolvedDocumentFrame { document, error: None },
            Err(error) => RuntimeResolvedDocumentFrame { document: None, error: Some(formatted(format_args!("{error}"), message)) },
        })
    })?;
    Ok(RuntimeResolvedDocumentRead(frame))
}

/* === Private Functions === */

fn failure(data: RuntimeDocumentData, arguments: std::fmt::Arguments<'_>) -> crate::runtime::Result<RuntimeResolvedDocumentRead> {
    let bytes = formatted_size(arguments)?;
    let frame = data.into_owner()?.try_map(|_| -> crate::runtime::Result<_> {
        Ok(RuntimeResolvedDocumentFrame { document: None, error: Some(formatted(arguments, bytes)) })
    })?;
    Ok(RuntimeResolvedDocumentRead(frame))
}

fn formatted_size(arguments: std::fmt::Arguments<'_>) -> crate::runtime::Result<usize> {
    use std::fmt::Write;
    struct Count(usize);
    impl Write for Count {
        fn write_str(&mut self, value: &str) -> std::fmt::Result {
            self.0 = self.0.checked_add(value.len()).ok_or(std::fmt::Error)?;
            Ok(())
        }
    }
    let mut count = Count(0);
    count.write_fmt(arguments).map_err(|_| crate::runtime::Error::Operation("document diagnostic size overflows".into()))?;
    Ok(count.0)
}

fn formatted(arguments: std::fmt::Arguments<'_>, bytes: usize) -> String {
    use std::fmt::Write;
    let mut message = String::with_capacity(bytes);
    message.write_fmt(arguments).expect("document diagnostic writes into a reserved String");
    message
}

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
