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
use std::{collections::BTreeSet, sync::Arc};
// Third Party Imports

// Local Imports
use crate::artifacts::ArtifactRecord;
use crate::telemetry::TelemetrySample;
use crate::{
    evidence_runtime::{EvidenceSnapshot, EvidenceSummaries},
    execution_runtime::SuiteSnapshot,
    queries::RuntimeStateSnapshot,
    runtime::handle::RuntimeHandle,
};
use dokime_domain::model::execution::health::HealthItem;
use dokime_domain::model::execution::notes::OperatorNote;
use dokime_domain::model::identity::Metadata;
use dokime_domain::model::identity::SessionId;
use dokime_domain::model::records::evidence::RecordEntry;
use dokime_domain::model::traceability::TraceabilitySnapshot;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub(crate) async fn projected_evidence_snapshot(
        &self, snapshot: &crate::queries::RuntimeStateSnapshot,
    ) -> crate::runtime::Result<EvidenceSnapshot> {
        let state = snapshot.state();
        let telemetry = self.collect_telemetry(&crate::evidence_runtime::TelemetrySelection::default()).await;
        let records = self.records_at(snapshot).await?;
        let (prompts, health, mut traceability, notes, artifacts, record_contracts, telemetry_contracts) = (
            state.pending_prompt_projection(),
            state.health_items(),
            crate::evidence_runtime::state::traceability_snapshots(state),
            state.notes(),
            self.artifacts().await?,
            state.record_contracts(),
            state.telemetry_contracts(),
        );
        self.enrich_host_traceability(&mut traceability, None).await;
        self.enrich_lease_traceability(&mut traceability, None).await;
        let summaries = EvidenceSummaries::from_records(&records, &telemetry);

        Ok(EvidenceSnapshot {
            record_contracts,
            telemetry_contracts,
            summaries,
            records,
            telemetry,
            prompts,
            artifacts,
            health,
            traceability,
            notes,
        })
    }

    pub(crate) fn state_read(&self) -> Arc<crate::runtime::state::RuntimeState> {
        Arc::clone(&self.state.read().expect("runtime state read-model lock is healthy"))
    }

    pub(crate) async fn refresh_state(&self) -> crate::runtime::Result<Arc<crate::runtime::state::RuntimeState>> {
        let state =
            self.state_manager.state().read(Clone::clone).await.map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
        let frontier = state.position.0;
        let state = Arc::new(state.value);
        *self.state.write().expect("runtime state read-model lock is healthy") = Arc::clone(&state);
        self.state_watch.send_replace(crate::queries::RuntimeStateSnapshot::new(Arc::clone(&state), frontier));
        Ok(state)
    }

    pub(crate) async fn capture_state(&self) -> crate::runtime::Result<crate::queries::RuntimeStateSnapshot> {
        let state =
            self.state_manager.state().read(Clone::clone).await.map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
        Ok(crate::queries::RuntimeStateSnapshot::new(Arc::new(state.value), state.position.0))
    }

    pub(crate) async fn settled_evidence_state(&self) -> Arc<crate::runtime::state::RuntimeState> {
        self.refresh_state().await.unwrap_or_else(|error| {
            tracing::error!(%error, "failed to refresh Harmos state for runtime read");
            self.state_read()
        })
    }

    pub fn storage_root(&self) -> std::path::PathBuf {
        self.config().storage_root.clone()
    }

    pub fn application_root(&self) -> std::path::PathBuf {
        self.config().application_root.clone()
    }

    pub fn hub_metadata(&self) -> Metadata {
        self.config().hub_metadata.clone()
    }

    pub fn identity(&self) -> dokime_domain::model::identity::DokimeIdentity {
        self.state_read().identity.clone()
    }

    pub fn is_shutting_down(&self) -> bool {
        *self.shutdown_requested.borrow() || self.state_read().shutting_down
    }

    pub async fn session_context(&self) -> (SessionId, String) {
        let state = self.state_read();

        (state.session_id.clone(), self.config().hub_metadata.label.clone())
    }

    pub async fn evidence_counts_at(
        &self, snapshot: &crate::queries::RuntimeStateSnapshot,
    ) -> crate::runtime::Result<crate::evidence_runtime::EvidenceCounts> {
        Ok(crate::runtime_state::evidence_counts(snapshot.state(), self.execution_storage.artifact_count().await?))
    }
}

/* === Read Dispatch === */

impl RuntimeHandle {
    pub async fn evidence_snapshot(&self) -> crate::runtime::Result<crate::evidence_runtime::EvidenceSnapshot> {
        let snapshot = self.capture_state().await?;
        self.projected_evidence_snapshot(&snapshot).await
    }

    pub async fn cycles(&self) -> Vec<dokime_domain::model::execution::result::CycleView> {
        self.state_read().cycle_views().collect()
    }

    pub async fn record_contracts(&self) -> Vec<crate::records::RecordContract> {
        self.state_read().record_contracts()
    }

    pub async fn telemetry_contracts(&self) -> Vec<crate::telemetry::TelemetryContract> {
        self.state_read().telemetry_contracts()
    }

    pub async fn telemetry_stream_summaries(&self) -> Vec<crate::evidence_runtime::TelemetryStreamSummary> {
        let telemetry = self.collect_telemetry(&crate::evidence_runtime::TelemetrySelection::default()).await;

        crate::evidence_runtime::EvidenceSummaries::from_records(std::iter::empty::<&RecordEntry>(), &telemetry).telemetry_streams
    }

    pub async fn selected_records(&self, selection: crate::evidence_runtime::RecordSelection) -> crate::runtime::Result<Vec<RecordEntry>> {
        let snapshot = self.capture_state().await?;
        let captured =
            crate::queries::records::execution_capture(snapshot.state().executions.values(), selection.scope.test_execution_id.as_ref());
        Ok(selection.apply_with_subscriptions(&self.records_at(&snapshot).await?, |record| {
            captured.and_then(|captures| captures.subscribed_records().get(&record.record_id))
        }))
    }

    pub(crate) async fn journal_records_through(&self, frontier: harmos::state::Position) -> crate::runtime::Result<Vec<RecordEntry>> {
        validate_frontier(frontier, self.state_manager.applied())?;
        self.record_source.wait_through(&self.state_manager, frontier.0).await?;
        let mut records = Vec::new();
        let mut reader = self.record_source.history(&self.state_manager, 0, frontier.0).await?;
        while let Some(record) = reader.next().await? {
            records.push((*record).clone());
        }
        Ok(records)
    }

    pub(crate) async fn records_at(&self, snapshot: &crate::queries::RuntimeStateSnapshot) -> crate::runtime::Result<Vec<RecordEntry>> {
        let visible = snapshot
            .state()
            .records
            .record_visibility
            .iter()
            .map(|record| record.record_id.clone())
            .chain(snapshot.state().executions.values().flat_map(|execution| execution.records.subscribed_records().keys().cloned()))
            .collect::<BTreeSet<_>>();
        Ok(self
            .journal_records_through(harmos::state::Position(snapshot.revision()))
            .await?
            .into_iter()
            .filter(|record| {
                visible.contains(&record.record_id)
                    || record.scope.as_ref().is_none_or(|scope| scope.test_cycle_id.is_none() && scope.test_execution_id.is_none())
            })
            .collect())
    }

    pub(crate) async fn current_journal_records(&self) -> crate::runtime::Result<Vec<RecordEntry>> {
        let snapshot = self.capture_state().await?;
        self.records_at(&snapshot).await
    }

    pub async fn selected_telemetry(&self, selection: crate::evidence_runtime::TelemetrySelection) -> Vec<TelemetrySample> {
        self.collect_telemetry(&selection).await
    }

    pub async fn try_selected_telemetry(
        &self, selection: crate::evidence_runtime::TelemetrySelection,
    ) -> crate::runtime::Result<Vec<TelemetrySample>> {
        self.try_collect_telemetry(&selection).await
    }

    pub async fn analysis_definitions(&self) -> crate::runtime::Result<Vec<crate::analysis::Definition>> {
        let snapshot = self.capture_state().await?;
        super::plots::analysis_definitions(self, &snapshot)
    }

    pub async fn query_plot_payloads(
        &self, requests: Vec<crate::queries::PlotPayloadRequest>, mode: crate::queries::RuntimePlotQueryMode,
    ) -> crate::runtime::Result<Vec<crate::queries::PlotPayloadEnvelope>> {
        crate::queries::plots::query_plot_payloads(self, requests, mode).await
    }

    pub async fn query_plot_stream_batches(
        &self, requests: &[crate::queries::PlotPayloadRequest],
    ) -> crate::runtime::Result<Vec<crate::queries::RuntimePlotStreamBatch>> {
        crate::queries::plots::query_plot_stream_batches(self, requests).await
    }

    pub async fn artifacts(&self) -> crate::runtime::Result<Vec<ArtifactRecord>> {
        self.execution_storage.artifact_records().await
    }

    pub async fn health_items(&self) -> Vec<HealthItem> {
        self.state_read().health_items()
    }

    pub async fn traceability_snapshots(&self) -> Vec<TraceabilitySnapshot> {
        let state = self.state_read();
        let mut snapshots = crate::evidence_runtime::state::traceability_snapshots(&state);
        self.enrich_host_traceability(&mut snapshots, None).await;
        self.enrich_lease_traceability(&mut snapshots, None).await;
        snapshots
    }

    pub async fn traceability_snapshots_page_at(
        &self, snapshot: &RuntimeStateSnapshot, scope: Option<&dokime_domain::model::execution::RecordScope>, after: &str, limit: usize,
        max_entries: usize, max_bytes: usize,
    ) -> crate::runtime::Result<(Vec<TraceabilitySnapshot>, bool, String)> {
        let created_at = crate::model::time::now_utc();
        self.with_latest_host_traceability(scope, |host, host_fingerprint| {
            let host_candidate = host.and_then(|sample| crate::traceability::host_traceability_page_candidate(sample, scope));
            let (mut snapshots, has_more, host_selected) = crate::traceability::bounded_snapshots_from_state(
                snapshot.state(),
                scope,
                after,
                limit,
                crate::traceability::TraceabilityPageBudget { max_entries, max_bytes },
                host_candidate.as_ref(),
                created_at.clone(),
            )
            .ok_or_else(|| crate::runtime::Error::Operation("traceability page exceeds the bounded page budget".into()))?;
            if host_selected {
                let host = host.expect("host candidate requires a selected host sample");
                let host =
                    crate::traceability::bounded_host_traceability_snapshot_from_sample(host, scope, max_entries, max_bytes, created_at)
                        .map_err(crate::runtime::Error::Operation)?
                        .expect("host candidate has visible entries");
                snapshots.push(host);
                snapshots.sort_by_key(|item| item.traceability_id.to_string());
            }
            Ok((snapshots, has_more, host_fingerprint.to_owned()))
        })
    }

    pub async fn notes(&self) -> Vec<OperatorNote> {
        self.state_read().notes()
    }

    pub async fn test_suites(&self) -> Vec<SuiteSnapshot> {
        self.state_read().test_suites()
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn validate_frontier(frontier: harmos::state::Position, applied: harmos::state::Position) -> crate::runtime::Result<()> {
    if frontier > applied {
        return Err(crate::runtime::Error::Operation("record frontier exceeds applied state position".into()));
    }
    Ok(())
}

/* === Tests === */
