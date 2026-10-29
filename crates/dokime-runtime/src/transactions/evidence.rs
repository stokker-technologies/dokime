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
use harmos::state::Transaction;
use serde::{Deserialize, Serialize};

// Local Imports
use crate::records::ValidationMode;
use crate::records::validate_record;
use crate::runtime::Error;
use crate::state::RuntimeState;
use dokime_domain::model::diagnostics::ValidationIssue;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::health::HealthItem;
use dokime_domain::model::execution::health::HealthSeverity;
use dokime_domain::model::identity::HealthItemId;
use dokime_domain::model::records::publication::RecordPublishRequest;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[harmos::transaction(id = "dokime.evidence.health.record")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecordHealth {
    pub scope: RecordScope,
    pub severity: HealthSeverity,
    pub message: String,
    pub detail: Option<String>,
    pub health_item_id: HealthItemId,
    pub observed_at: Timestamp,
}

#[harmos::transaction(id = "dokime.evidence.record_subscriptions.close")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CloseRecordSubscriptions {
    pub test_cycle_id: dokime_domain::model::identity::TestCycleId,
    pub test_execution_id: dokime_domain::model::identity::TestExecutionId,
}

#[harmos::transaction(id = "dokime.evidence.event.publish")]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublishRecord {
    pub request: RecordPublishRequest,
    pub occurred_at: Timestamp,
    pub received_at: Timestamp,
    source: PublicationSource,
}

/* === Enums === */

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum PublicationSource {
    Host,
    Sidecar { owner: String },
    Undeclared,
}

/* === Traits === */

/* === Implementations === */

impl Transaction<RuntimeState> for RecordHealth {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        if state.shutting_down {
            return Err("runtime is shutting down".to_owned());
        }
        Ok(())
    }

    fn apply(&self, state: &mut RuntimeState) {
        state.record_health_item(HealthItem {
            health_item_id: self.health_item_id.clone(),
            scope: self.scope.clone(),
            severity: self.severity.clone(),
            message: self.message.clone(),
            detail: self.detail.clone(),
            observed_at: self.observed_at.clone(),
        });
    }
}

impl Transaction<RuntimeState> for CloseRecordSubscriptions {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        state
            .executions
            .get(&self.test_execution_id)
            .filter(|execution| execution.test_cycle_id == self.test_cycle_id)
            .map(|_| ())
            .ok_or_else(|| "record subscription execution is missing".to_owned())
    }

    fn apply(&self, state: &mut RuntimeState) {
        state
            .executions
            .get_mut(&self.test_execution_id)
            .expect("checked record subscription execution")
            .records
            .close_record_subscriptions();
    }
}

impl PublishRecord {
    pub fn host(request: RecordPublishRequest, occurred_at: Timestamp) -> Self {
        Self {
            request,
            received_at: occurred_at.clone(),
            occurred_at,
            source: PublicationSource::Host,
        }
    }

    pub fn undeclared(request: RecordPublishRequest, occurred_at: Timestamp) -> Self {
        Self {
            request,
            received_at: occurred_at.clone(),
            occurred_at,
            source: PublicationSource::Undeclared,
        }
    }

    pub fn sidecar(request: RecordPublishRequest, owner: String, occurred_at: Timestamp) -> Self {
        Self {
            request,
            received_at: occurred_at.clone(),
            occurred_at,
            source: PublicationSource::Sidecar { owner },
        }
    }
}

impl Transaction<RuntimeState> for PublishRecord {
    type Error = String;

    fn check(&self, state: &RuntimeState) -> Result<(), Self::Error> {
        if state.shutting_down {
            return Err("runtime is shutting down".to_owned());
        }
        prepare_record(state, self).map(|_| ()).map_err(|error| error.to_string())
    }

    fn records(&self, state: &RuntimeState, out: &mut harmos::state::Records) {
        prepare_record(state, self).expect("checked record plan").collect(out);
    }
    fn apply(&self, state: &mut RuntimeState) {
        prepare_record(state, self).expect("checked record plan").apply(state);
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn prepare_record(state: &RuntimeState, publish: &PublishRecord) -> Result<crate::transactions::records::draft::Drafts, Error> {
    match &publish.source {
        PublicationSource::Host => {
            crate::transactions::records::admission::host(state, &publish.request.record_type)
                .map_err(|error| validation_rejection(error, "record_type", None))?;
            prepare_record_with_source(state, publish, "runtime", false)
        }
        PublicationSource::Sidecar { owner } => {
            crate::transactions::records::admission::sidecar(state, owner, &publish.request.record_type)
                .map_err(|error| validation_rejection(error, "record_type", None))?;
            prepare_record_with_source(state, publish, owner, true)
        }
        PublicationSource::Undeclared => {
            let mut record = publish.request.clone().into_record(state.next_record_id(), 1, publish.occurred_at.clone(), "runtime");
            record.received_at = publish.received_at.clone();
            let mut records = crate::transactions::records::draft::Drafts::new(state);
            records.push(record);
            Ok(records)
        }
    }
}

fn prepare_record_with_source(
    state: &RuntimeState, publish: &PublishRecord, source: &str, native: bool,
) -> Result<crate::transactions::records::draft::Drafts, Error> {
    let Some(contract) = state.record_definitions.get(&publish.request.record_type).cloned() else {
        let detail = validation_rejection(
            format!("record type {} is not registered", publish.request.record_type),
            "record_type",
            Some("active record contract".to_owned()),
        );
        return Err(detail);
    };
    let mut record =
        publish.request.clone().into_record(state.next_record_id(), contract.schema_version, publish.occurred_at.clone(), source);
    record.received_at = publish.received_at.clone();
    if let Some(scope) = &record.scope {
        record.scope = Some(state.capture_scope(scope).map_err(|error| validation_rejection(error, "scope", None))?);
    }
    if let Err(error) = validate_record(Some(&contract), &record, ValidationMode::Strict) {
        return Err(validation_rejection(
            error.to_string(),
            format!("records.{}", record.record_type.as_str()),
            Some(format!("schema_version {}", contract.schema_version)),
        ));
    }
    let mut records = crate::transactions::records::draft::Drafts::new(state);
    if native {
        records.push_sidecar(record, source);
    } else {
        records.push(record);
    }
    Ok(records)
}

fn validation_rejection(message: impl Into<String>, path: impl Into<String>, expected: Option<String>) -> Error {
    Error::validation(
        "validation_failed",
        message,
        vec![ValidationIssue {
            path: path.into(),
            message: "producer payload failed validation".to_owned(),
            expected,
        }],
    )
}

/* === Tests === */
