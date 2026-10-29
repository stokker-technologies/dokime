/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(crate) use crate::transactions::PublishRecord;
pub use dokime_domain::model::records::publication::PublishReceipt;
pub use dokime_domain::model::records::publication::PublishStatus;
pub use dokime_domain::model::records::publication::RecordPublishRequest;

use std::collections::BTreeMap;

use tracing::Instrument;

use crate::telemetry::TelemetryBatch;
use crate::telemetry_runtime::sampling::{SamplingPhaseState, sampling_phase_state};
use crate::transactions::RecordHealth;
use crate::{RuntimeHandle, runtime::Error, state::RuntimeState, unavailable_rejection};
use dokime_domain::model::diagnostics::ValidationIssue;
use dokime_domain::model::execution::health::HealthSeverity;
use dokime_domain::model::identity::HealthItemId;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::records::product::ProductRecordDraft;
use dokime_domain::model::time::Timestamp;

impl RuntimeHandle {
    pub async fn publish_product_record(&self, draft: ProductRecordDraft) -> Result<PublishReceipt, Error> {
        self.publish_record(draft.into()).await
    }

    pub async fn publish_record(&self, request: RecordPublishRequest) -> Result<PublishReceipt, Error> {
        self.commit_record(PublishRecord::host(request, crate::model::time::now_utc())).await
    }

    #[cfg(feature = "fault-injection")]
    pub async fn publish_undeclared_record(&self, request: RecordPublishRequest) -> Result<PublishReceipt, Error> {
        self.commit_record(PublishRecord::undeclared(request, crate::model::time::now_utc())).await
    }

    pub(crate) async fn publish_sidecar_record(&self, request: RecordPublishRequest, owner: &str) -> Result<PublishReceipt, Error> {
        self.publish_sidecar_record_at(request, owner, crate::model::time::now_utc()).await
    }

    pub(crate) async fn publish_sidecar_record_at(
        &self, request: RecordPublishRequest, owner: &str, occurred_at: Timestamp,
    ) -> Result<PublishReceipt, Error> {
        let mut transaction = PublishRecord::sidecar(request, owner.to_owned(), occurred_at);
        transaction.received_at = crate::model::time::now_utc();
        self.commit_record(transaction).await
    }

    async fn commit_record(&self, transaction: PublishRecord) -> Result<PublishReceipt, Error> {
        let request = transaction.request.clone();
        let occurred_at = transaction.occurred_at.clone();
        let receipt = match self
            .state_manager
            .commit(transaction)
            .instrument(tracing::debug_span!(target: "crate::record_admission", "journal_commit"))
            .await
        {
            Ok(receipt) => receipt,
            Err(error) => {
                let detail = error.to_string();
                let warning = RecordHealth {
                    scope: request.scope.clone().unwrap_or_default(),
                    severity: HealthSeverity::Warning,
                    message: "Producer evidence publication warning.".to_owned(),
                    detail: Some(format!("Record producer payload rejected: {detail}.")),
                    health_item_id: HealthItemId::from_seed(format!("event-publish-rejection:{occurred_at}:{}", request.record_type)),
                    observed_at: occurred_at,
                };
                if self.state_manager.commit(warning).await.is_ok() {
                    let _ = self.refresh_state().await;
                }
                return Err(validation_rejection(detail, format!("records.{}", request.record_type), None));
            }
        };
        self.refresh_state()
            .instrument(tracing::debug_span!(target: "crate::record_admission", "state_refresh"))
            .await
            .map_err(|error| unavailable_rejection(error.to_string()))?;
        Ok(PublishReceipt::accepted(1, BTreeMap::new(), receipt.frontier.0))
    }

    pub async fn publish_telemetry_batch(&self, batch: TelemetryBatch) -> Result<harmos::storage::ReadBatch<PublishReceipt>, Error> {
        wait_for_telemetry_batch_cycles(self, &batch).await;
        self.publish_telemetry_to_streams(batch, crate::model::time::now_utc(), crate::domains::telemetry::Payloads::Unchecked).await
    }
}

pub(crate) fn validation_rejection(message: impl Into<String>, path: impl Into<String>, expected: Option<String>) -> Error {
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

pub(crate) fn retain_running_samples(state: &RuntimeState, batch: &mut TelemetryBatch) -> u64 {
    let before = batch.samples.len();
    batch.samples.retain(|sample| {
        sample.scope.as_ref().is_none_or(|scope| {
            scope.test_execution_id.as_ref().is_none_or(|id| !state.report_finalizations.contains_key(id))
                && sampling_phase_state(state, scope, None) == SamplingPhaseState::Running
        })
    });
    before.saturating_sub(batch.samples.len()) as u64
}

async fn wait_for_telemetry_batch_cycles(runtime: &RuntimeHandle, batch: &TelemetryBatch) {
    let mut cycle_ids: Vec<TestCycleId> = Vec::new();
    for sample in &batch.samples {
        let Some(test_cycle_id) = sample.scope.as_ref().and_then(|scope| scope.test_cycle_id.clone()) else {
            continue;
        };
        if !cycle_ids.iter().any(|existing| existing == &test_cycle_id) {
            cycle_ids.push(test_cycle_id);
        }
    }
    for test_cycle_id in cycle_ids {
        runtime.wait_if_paused(&test_cycle_id).await;
    }
}
