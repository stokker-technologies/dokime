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
use std::collections::BTreeMap;

// Third Party Imports

// Local Imports
use crate::records::BuiltinRecordType;
use crate::records::RecordSeverity;
use crate::state::RuntimeState;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::RecordId;
use dokime_domain::model::records::evidence::Record;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::values::Value;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct BuiltinDraft {
    pub record_type: BuiltinRecordType,
    pub source: String,
    pub severity: RecordSeverity,
    pub message: Option<String>,
    pub details: BTreeMap<String, Value>,
    pub scope: Option<RecordScope>,
}

pub(crate) struct Drafts {
    session: dokime_domain::model::identity::SessionId,
    first: u64,
    records: Vec<(Record, Option<String>)>,
    captures: Vec<(dokime_domain::model::identity::TestExecutionId, Vec<dokime_domain::model::identity::Identifier>, RecordScope)>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Drafts {
    pub(crate) fn new(state: &RuntimeState) -> Self {
        Self {
            session: state.record_namespace.clone(),
            first: state.record_count,
            records: Vec::new(),
            captures: state
                .executions
                .values()
                .filter(|execution| {
                    !execution.records.record_subscriptions_closed()
                        && execution.status == dokime_domain::model::execution::result::ExecutionStatus::Running
                })
                .map(|execution| {
                    (
                        execution.test_execution_id.clone(),
                        execution
                            .snapshot
                            .template()
                            .map(|template| {
                                template.subscriptions.iter().filter_map(crate::definition::Subscription::record).cloned().collect()
                            })
                            .unwrap_or_default(),
                        execution.capture_scope(),
                    )
                })
                .collect(),
        }
    }

    pub(crate) fn capture_scope(&mut self, scope: RecordScope) {
        if let Some(execution) = scope.test_execution_id.clone()
            && !self.captures.iter().any(|(owner, _, _)| owner == &execution)
        {
            self.captures.push((execution, Vec::new(), scope));
        }
    }

    pub(crate) fn following(&self) -> Self {
        Self {
            session: self.session.clone(),
            first: self.first + self.records.len() as u64,
            records: Vec::new(),
            captures: self.captures.clone(),
        }
    }

    pub(crate) fn next_id(&self) -> RecordId {
        RecordId::from_seed(format!("dokime-record:{}:{}", self.session, self.first + self.records.len() as u64))
    }

    pub(crate) fn push(&mut self, record: Record) {
        self.records.push((record, None));
    }

    pub(crate) fn push_sidecar(&mut self, mut record: Record, owner: &str) {
        record.source = owner.to_owned();
        self.records.push((record, Some(owner.to_owned())));
    }

    pub(crate) fn draft(&mut self, draft: BuiltinDraft, at: Timestamp) {
        self.push(Record {
            record_id: self.next_id(),
            received_at: at.clone(),
            timestamp: at,
            record_type: draft.record_type.identifier(),
            schema_version: draft.record_type.schema_version(),
            source: draft.source,
            severity: draft.severity,
            message: draft.message,
            error: None,
            details: draft.details,
            scope: draft.scope,
            related_sample_id: None,
            related_stream_id: None,
            related_artifact_id: None,
        });
    }

    pub(crate) fn retain(&mut self, mut keep: impl FnMut(&Record) -> bool) {
        self.records.retain(|(record, _)| keep(record));
    }

    pub(crate) fn collect(&self, out: &mut harmos::state::Records) {
        for (index, (record, native_owner)) in self.records.iter().enumerate() {
            let captured = self.captured(record, index);
            if let Some(owner) = native_owner {
                let payload = crate::persistence::to_vec(&captured).expect("Record contains only JSON-compatible fields");
                let _ = out.push_system(harmos::state::SystemFact {
                    source: harmos::state::FactSource::Sidecar(owner.clone()),
                    name: record.record_type.to_string(),
                    version: record.schema_version,
                    payload,
                });
            } else {
                let _ = out.push(captured);
            }
        }
    }

    fn captured(&self, record: &Record, index: usize) -> dokime_domain::state::records::CapturedRecord {
        let scopes = self
            .captures
            .iter()
            .filter(|(owner, kinds, _)| {
                record.scope.as_ref().and_then(|scope| scope.test_execution_id.as_ref()) == Some(owner)
                    || kinds.contains(&record.record_type)
            })
            .map(|(_, _, scope)| scope.clone())
            .collect();
        dokime_domain::state::records::CapturedRecord {
            record: record.clone(),
            ordinal: self.first + index as u64 + 1,
            scopes,
        }
    }

    pub(crate) fn apply(self, state: &mut RuntimeState) {
        for (index, (record, _)) in self.records.iter().enumerate() {
            state.commit_record(self.captured(record, index));
        }
    }
}

impl IntoIterator for Drafts {
    type Item = Record;
    type IntoIter = std::iter::Map<std::vec::IntoIter<(Record, Option<String>)>, fn((Record, Option<String>)) -> Record>;

    fn into_iter(self) -> Self::IntoIter {
        self.records.into_iter().map(|(record, _)| record)
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
