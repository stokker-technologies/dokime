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
use std::sync::Arc;

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
    ) -> crate::runtime::Result<harmos::storage::ReadBatch<EvidenceSnapshot>> {
        let state = snapshot.state();
        let at = self.saved_through(snapshot.revision()).await?;
        let session = self.storage().clone();
        let telemetry =
            self.collect_telemetry_at_in(&session, &crate::evidence_runtime::TelemetrySelection::default(), None, None, &at).await?;
        let records: crate::storage::ReadRows<RecordEntry> = self.record_pages_at_in(&session, snapshot, &at).await?.into();
        let artifacts = self.artifacts_at_in(&session, &at).await?;
        let summaries = EvidenceSummaries::retained(&records, &telemetry)?;
        let traceability = self.retained_traceability_in(state, None, &telemetry, true, &crate::model::time::now_utc())?;
        let result = telemetry.try_map_with(summaries, |telemetry, summaries| {
            Ok::<_, crate::runtime::Error>(EvidenceSnapshot {
                record_contracts: evidence_clone_rows(state.record_definitions.values().count(), state.record_definitions.values())?,
                telemetry_contracts: evidence_clone_rows(
                    state.telemetry_definitions.values().count(),
                    state.telemetry_definitions.values(),
                )?,
                summaries,
                records,
                telemetry,
                prompts: evidence_clone_rows(state.open_prompts().count(), state.open_prompts())?,
                artifacts: Vec::new(),
                health: evidence_clone_rows(state.iter_health_items().count(), state.iter_health_items())?,
                traceability: Vec::new(),
                notes: evidence_clone_rows(state.iter_notes().count(), state.iter_notes())?,
            })
        })?;
        let result = result.try_map_with(artifacts, |mut snapshot, artifacts| {
            snapshot.artifacts = artifacts;
            Ok::<_, crate::runtime::Error>(snapshot)
        })?;
        result.try_map_with(traceability, |mut snapshot, traceability| {
            snapshot.traceability = traceability;
            Ok(snapshot)
        })
    }

    // A Read Sees Its Own Writes: Records Through The Revision And Every Accepted Row Are Saved Before The Cut Is Taken.
    pub(crate) async fn saved_through(&self, revision: u64) -> crate::runtime::Result<harmos::storage::StorageCursor> {
        self.record_source.wait_through(&self.state_manager, revision).await?;
        Ok(self.storage().flush_until(&self.storage().cursor_accepted()).await?)
    }

    pub(crate) fn state_read(&self) -> Arc<crate::runtime::state::RuntimeState> {
        Arc::clone(&self.state.read().expect("runtime state read-model lock is healthy"))
    }

    pub(crate) async fn refresh_state(&self) -> crate::runtime::Result<Arc<crate::runtime::state::RuntimeState>> {
        let revision = self.state_manager.state().read(|_| ()).await?.position.0;
        if self.state_watch.borrow().revision() >= revision {
            return Ok(self.state_read());
        }
        // Clone Shared Roots At One Committed Revision; Readers Retain Immutable Entries.
        let state =
            self.state_manager.state().read(Clone::clone).await.map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
        let frontier = state.position.0;
        let state = Arc::new(state.value);
        self.state_watch.send_if_modified(|snapshot| {
            if snapshot.revision() >= frontier {
                return false;
            }
            *self.state.write().expect("runtime state read-model lock is healthy") = Arc::clone(&state);
            *snapshot = crate::queries::RuntimeStateSnapshot::new(Arc::clone(&state), frontier);
            true
        });
        Ok(self.state_read())
    }

    pub(crate) async fn capture_state(&self) -> crate::runtime::Result<crate::queries::RuntimeStateSnapshot> {
        self.refresh_state().await?;
        Ok(self.state_watch.borrow().clone())
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

    pub fn storage_read_counters(&self) -> crate::storage::sqlite::ReadCounters {
        self.local_storage.read_counters()
    }

    pub fn application_root(&self) -> std::path::PathBuf {
        self.config().application.root.clone()
    }

    pub fn application_paths(&self) -> crate::paths::ApplicationPaths {
        self.config().application.clone()
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
        let traceability = crate::traceability::count(snapshot.state())?;
        Ok(crate::runtime_state::evidence_counts(snapshot.state(), self.execution_storage.artifact_count().await?, traceability))
    }
}

impl RuntimeHandle {
    pub async fn evidence_snapshot(&self) -> crate::runtime::Result<harmos::storage::ReadBatch<crate::evidence_runtime::EvidenceSnapshot>> {
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

    pub async fn telemetry_stream_summaries(
        &self,
    ) -> crate::runtime::Result<harmos::storage::ReadBatch<Vec<crate::evidence_runtime::TelemetryStreamSummary>>> {
        let session = self.storage();
        let at = session.cursor_saved();
        let codecs = crate::runtime::storage::telemetry::saved_codecs(session, &at, None, self.schema_settings(), |id, _| {
            crate::storage::telemetry::selected_schema(id, &[])
        })
        .await?;
        let mut summaries = EvidenceSummaries::default();
        let mut after = 0;
        loop {
            let page = self.local_storage.telemetry_page(at.clone(), after, None, None).await?;
            if page.is_empty() {
                break;
            }
            for row in page {
                after = row.record.position.sequence;
                let codec = codecs
                    .iter()
                    .find(|codec| codec.schema() == &row.record.schema)
                    .ok_or_else(|| crate::runtime::Error::Operation("saved telemetry codec is missing".into()))?;
                let mut sample = codec.decode_stored(&row)?.sample;
                sample.sequence = after;
                summaries.record_telemetry_sample(&sample);
            }
            tokio::task::yield_now().await;
        }
        Ok(harmos::storage::ReadBatch::new(summaries.telemetry_streams))
    }

    pub async fn selected_records(&self, selection: crate::evidence_runtime::RecordSelection) -> crate::runtime::Result<Vec<RecordEntry>> {
        let snapshot = self.capture_state().await?;
        let captured =
            crate::queries::records::execution_capture(snapshot.state().executions.values(), selection.scope.test_execution_id.as_ref());
        Ok(selection.apply_with_subscriptions(&self.records_at(&snapshot).await?, |record| {
            captured.and_then(|captures| captures.captured_scope(record))
        }))
    }

    pub(crate) async fn record_pages_at_in(
        &self, session: &crate::storage::Manager, snapshot: &crate::queries::RuntimeStateSnapshot, at: &harmos::storage::StorageCursor,
    ) -> crate::runtime::Result<crate::storage::projection::Pages<RecordEntry>> {
        let state = snapshot.state();
        self.record_source
            .saved_filtered_records_at_in(session, snapshot.revision(), at, |record| {
                record.scope.as_ref().is_none_or(|scope| scope.test_cycle_id.as_ref().is_none_or(|id| state.cycles.contains_key(id)))
                    || state.executions.values().any(|execution| execution.records.captured_scope(record).is_some())
            })
            .await
    }

    pub(crate) async fn records_at(&self, snapshot: &crate::queries::RuntimeStateSnapshot) -> crate::runtime::Result<Vec<RecordEntry>> {
        let session = self.storage().clone();
        let at = self.saved_through(snapshot.revision()).await?;
        let pages = self.record_pages_at_in(&session, snapshot, &at).await?;
        Ok(pages.iter().flat_map(|page| page.iter().cloned()).collect())
    }

    pub(crate) async fn current_journal_records(&self) -> crate::runtime::Result<Vec<RecordEntry>> {
        let snapshot = self.capture_state().await?;
        self.records_at(&snapshot).await
    }

    pub async fn selected_telemetry(
        &self, selection: crate::evidence_runtime::TelemetrySelection,
    ) -> crate::runtime::Result<harmos::storage::ReadBatch<Vec<TelemetrySample>>> {
        self.collect_telemetry(&selection).await
    }

    // Reads One Storage-Ordered Page After The Position `after`; Each Page Or Wake Scans Only Rows Past It.
    pub async fn telemetry_page_at(
        &self, session: &crate::storage::Manager, selection: &crate::evidence_runtime::TelemetrySelection,
        at: &harmos::storage::StorageCursor, after: u64,
    ) -> crate::runtime::Result<(crate::runtime::TelemetryPage, bool)> {
        use harmos::storage::{ObjectRead, query::ObjectVersions};
        let mut sealed = false;
        if let Some(id) = &selection.scope.test_execution_id {
            let intent = self.state_read().report_finalizations.get(id).filter(|intent| intent.lifecycle.is_some()).cloned();
            if let Some(intent) = intent {
                sealed = at.covers(&intent.evidence_at)?;
            }

            let mut rows = session
                .read_objects::<crate::storage::objects::execution::Execution>(ObjectRead {
                    at: at.clone(),
                    key: Some(id.as_str()),
                    versions: ObjectVersions::Latest,
                })
                .await?;
            while let Some(page) = rows.next().await? {
                sealed |= page.iter().any(|row| row.value.is_sealed());
            }
        }
        let options = crate::runtime::storage::telemetry::ReadOptions { after, ..Default::default() };
        let page =
            crate::runtime::storage::telemetry::collect_page(session, at, selection, self.schema_settings(), options, &self.local_storage)
                .await?;
        Ok((page, sealed))
    }

    pub async fn analysis_definitions(&self) -> crate::runtime::Result<Vec<crate::analysis::Definition>> {
        let snapshot = self.capture_state().await?;
        super::plots::analysis_definitions(self, &snapshot)
    }

    pub async fn query_plot_payloads(
        &self, requests: Vec<crate::queries::PlotPayloadRequest>, mode: crate::queries::RuntimePlotQueryMode,
    ) -> crate::runtime::Result<Vec<harmos::storage::ReadBatch<crate::queries::PlotPayloadEnvelope>>> {
        crate::queries::plots::query_plot_payloads(self, requests, mode).await
    }

    pub async fn query_plot_payloads_at(
        &self, requests: Vec<crate::queries::PlotPayloadRequest>, mode: crate::queries::RuntimePlotQueryMode,
        at: &harmos::storage::StorageCursor,
    ) -> crate::runtime::Result<Vec<harmos::storage::ReadBatch<crate::queries::PlotPayloadEnvelope>>> {
        crate::queries::plots::query_plot_payloads_at(self, requests, mode, at).await
    }

    pub async fn query_plot_stream_batches(
        &self, requests: &[crate::queries::PlotPayloadRequest],
    ) -> crate::runtime::Result<Vec<crate::queries::RuntimePlotStreamBatch>> {
        crate::queries::plots::query_plot_stream_batches(self, requests).await
    }

    pub async fn artifacts(&self) -> crate::runtime::Result<Vec<ArtifactRecord>> {
        self.execution_storage.artifact_records().await
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
        .await
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

fn evidence_memory() -> crate::runtime::Error {
    harmos::Error::ResourceExhausted {
        resource: "evidence snapshot construction",
    }
    .into()
}

fn evidence_clone_rows<'a, T: Clone + 'a>(count: usize, rows: impl Iterator<Item = &'a T>) -> crate::runtime::Result<Vec<T>> {
    let mut result = Vec::new();
    result.try_reserve_exact(count).map_err(|_| evidence_memory())?;
    result.extend(rows.cloned());
    Ok(result)
}

/* === Tests === */
