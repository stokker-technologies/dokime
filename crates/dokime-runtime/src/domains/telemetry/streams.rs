/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

pub(super) mod admission;

/* === Re-Exports === */

pub(crate) use admission::{PreparedSample, TelemetryCursors};

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::evidence_runtime::{PublishReceipt, TelemetrySelection, retain_running_samples, validation_rejection};
use crate::runtime::handle::RuntimeHandle;
use crate::telemetry::{TelemetryBatch, TelemetrySample};
use crate::unavailable_rejection;
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::scope_visible_in;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::time::Timestamp;
use dokime_domain::model::values::Value;

/* === Constants === */

const TELEMETRY_STREAM_PREFIX: &str = "dokime.tel";
const TELEMETRY_STREAM_DELIMITER: char = '|';
const UNSCOPED_STREAM_SEGMENT: &str = "-";

/* === Macros === */

/* === Type Aliases === */

pub(crate) struct CapturedPublication {
    pub receipt: harmos::storage::ReadBatch<PublishReceipt>,
    pub failure: Option<crate::runtime::Error>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl RuntimeHandle {
    pub(crate) async fn publish_telemetry_to_streams(
        &self, batch: TelemetryBatch, received_at: Timestamp,
    ) -> std::result::Result<harmos::storage::ReadBatch<PublishReceipt>, crate::runtime::Error> {
        let runtime = self.clone();
        tokio::spawn(async move { runtime.publish_telemetry_to_streams_inner(batch, received_at).await })
            .await
            .map_err(|e| unavailable_rejection(e.to_string()))?
    }
    async fn publish_telemetry_to_streams_inner(
        &self, mut batch: TelemetryBatch, received_at: Timestamp,
    ) -> std::result::Result<harmos::storage::ReadBatch<PublishReceipt>, crate::runtime::Error> {
        let sampling = self.sampling();
        let _publish = sampling.begin_publish().await;
        let state = self.state_read();
        let contract = state.telemetry_definitions.get(&batch.stream);
        for sample in &batch.samples {
            if let Some(scope) = &sample.scope {
                state
                    .capture_scope(scope)
                    .map_err(|reason| crate::runtime::Error::Operation(format!("telemetry acquisition refused: {reason}")))?;
            }
        }
        let dropped = sampling.retain_open_samples(&mut batch).await + retain_running_samples(&state, &mut batch);
        let revision = self.state_manager.applied().0;
        let Some(contract) = contract else {
            return Err(validation_rejection(
                format!("telemetry stream {} is not registered", batch.stream),
                "stream",
                Some("active telemetry contract".to_owned()),
            ));
        };
        if batch.samples.is_empty() {
            drop(batch);
            return admission::dropped_receipt(
                dropped,
                revision,
                "telemetry sample(s) dropped because the scoped execution or cycle is not running",
            );
        }

        normalize_batch(&mut batch, contract, &received_at)?;

        let receipt = self
            .write_samples_to_streams(batch.samples)
            .await
            .map_err(|error| unavailable_rejection(format!("telemetry stream write failed: {error}")))?;
        // Admission also delivers the frozen lease batch while this publish
        // guard is held. Quiescence therefore waits for both capture and
        // physical delivery; a later lease never receives an uncaptured row.
        receipt.try_map(|mut receipt| {
            receipt.dropped += dropped;
            Ok::<_, crate::runtime::Error>(receipt)
        })
    }

    pub(crate) async fn publish_captured_telemetry(
        &self, completion: crate::services::telemetry::CapturedSamples, received_at: Timestamp,
    ) -> crate::runtime::Result<CapturedPublication> {
        let sampling = self.sampling();
        let _publish = sampling.begin_publish().await;
        let mut batch = completion.into_batch();
        let dropped = sampling.retain_open_samples(&mut batch).await;
        let revision = self.state_manager.applied().0;
        if batch.samples.is_empty() {
            drop(batch);
            return Ok(CapturedPublication {
                receipt: admission::dropped_receipt(dropped, revision, "unsent captured samples discarded after execution stop")?,
                failure: None,
            });
        }
        let state = self.state_read();
        let contract = state.telemetry_definitions.get(&batch.stream).ok_or_else(|| {
            validation_rejection(
                format!("telemetry stream {} is not registered", batch.stream),
                "stream",
                Some("active telemetry contract".to_owned()),
            )
        })?;
        normalize_batch(&mut batch, contract, &received_at)?;
        // The Original Storage Admission Rejects A Closed Acquisition And Retains Capacity.
        let admission = self.prepare_sample_admission(batch.samples).await?;
        let runtime = self.clone();
        let mut published = tokio::spawn(async move { runtime.admit_samples_owned(admission).await }).await??;
        published.receipt = published.receipt.try_map(|mut receipt| {
            receipt.dropped += dropped;
            Ok::<_, crate::runtime::Error>(receipt)
        })?;
        Ok(published)
    }

    async fn write_samples_to_streams(
        &self, samples: Vec<TelemetrySample>,
    ) -> crate::runtime::Result<harmos::storage::ReadBatch<PublishReceipt>> {
        let runtime = self.clone();
        tokio::spawn(async move {
            let admission = runtime.prepare_sample_admission(samples).await?;
            runtime.admit_samples(admission).await
        })
        .await
        .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?
    }

    pub async fn collect_telemetry(
        &self, selection: &TelemetrySelection,
    ) -> crate::runtime::Result<harmos::storage::ReadBatch<Vec<TelemetrySample>>> {
        crate::runtime::storage::telemetry::collect_saved(
            self.storage(),
            &self.storage().cursor_saved(),
            selection,
            self.schema_settings(),
            crate::runtime::storage::telemetry::ReadOptions {
                sequence: crate::runtime::storage::telemetry::Sequence::StoragePosition,
                ..Default::default()
            },
        )
        .await
    }

    pub(crate) async fn collect_telemetry_in(
        &self, session: &crate::storage::Manager, selection: &TelemetrySelection,
    ) -> crate::runtime::Result<harmos::storage::ReadBatch<Vec<TelemetrySample>>> {
        self.collect_telemetry_at_in(session, selection, None, None, &self.storage().cursor_saved()).await
    }

    pub(crate) async fn collect_telemetry_at_in(
        &self, session: &crate::storage::Manager, selection: &TelemetrySelection, scan_limit: Option<usize>, window: Option<(&str, f64)>,
        at: &harmos::storage::StorageCursor,
    ) -> crate::runtime::Result<harmos::storage::ReadBatch<Vec<TelemetrySample>>> {
        crate::runtime::storage::telemetry::collect_saved(
            session,
            at,
            selection,
            self.schema_settings(),
            crate::runtime::storage::telemetry::ReadOptions { scan_limit, window, ..Default::default() },
        )
        .await
    }

    pub(crate) async fn telemetry_sample_count(&self) -> usize {
        self.telemetry_cursors
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .values
            .values()
            .fold(0_usize, |count, cursor| count.saturating_add(usize::try_from(*cursor).unwrap_or(usize::MAX)))
    }

    pub(crate) async fn telemetry_aggregate_outputs(
        &self, scope: &dokime_domain::model::execution::RecordScope,
    ) -> crate::runtime::Result<Vec<crate::telemetry_runtime::output::AggregateOutput>> {
        let state = self.settled_evidence_state().await;
        let contracts = state.telemetry_contracts();
        let samples = self
            .collect_telemetry(&TelemetrySelection {
                scope: crate::evidence_runtime::ScopeSelection {
                    test_cycle_id: scope.test_cycle_id.clone(),
                    test_execution_id: scope.test_execution_id.clone(),
                    test_case_id: scope.test_case_id.clone(),
                    template_id: scope.template_id.clone(),
                },
                ..TelemetrySelection::default()
            })
            .await?;
        Ok(crate::telemetry_runtime::output::project_samples(&contracts, &samples))
    }

    pub(crate) fn retained_traceability_in(
        &self, state: &crate::state::RuntimeState, scope: Option<&RecordScope>, samples: &[TelemetrySample], include_leases: bool,
        created_at: &Timestamp,
    ) -> crate::runtime::Result<harmos::storage::ReadBatch<Vec<dokime_domain::model::traceability::TraceabilitySnapshot>>> {
        let snapshots = crate::traceability::retained_state(state, scope, created_at)?;
        self.enrich_retained_traceability(snapshots, scope, samples, include_leases)
    }

    fn enrich_retained_traceability(
        &self, mut snapshots: harmos::storage::ReadBatch<Vec<dokime_domain::model::traceability::TraceabilitySnapshot>>,
        scope: Option<&RecordScope>, samples: &[TelemetrySample], include_leases: bool,
    ) -> crate::runtime::Result<harmos::storage::ReadBatch<Vec<dokime_domain::model::traceability::TraceabilitySnapshot>>> {
        let host = samples
            .iter()
            .filter(|sample| {
                sample.stream.as_str() == "host.system"
                    && scope.is_none_or(|scope| dokime_domain::model::execution::scope_visible_in(sample.scope.as_ref(), scope))
            })
            .max_by(|left, right| left.timestamp.cmp(&right.timestamp).then(left.sequence.cmp(&right.sequence)));
        if let Some(host) = host
            && let Some(host) = crate::traceability::retained_host(host, &crate::model::time::now_utc())?
        {
            snapshots = crate::traceability::replace_source(snapshots, host, "traceability.host")?;
        }
        if !include_leases {
            return Ok(snapshots);
        }
        let empty = RecordScope::default();
        self.stream_leases().with_facts(scope.unwrap_or(&empty), |facts| {
            let leases = crate::traceability::retained_leases(scope, facts, &crate::model::time::now_utc())?;
            match leases {
                Some(leases) => {
                    crate::traceability::replace_source(snapshots, leases, dokime_domain::model::traceability::LEASES_TRACEABILITY.source)
                }
                None => Ok(snapshots),
            }
        })
    }

    pub(crate) async fn retained_traceability_at_in(
        &self, session: &crate::storage::Manager, state: &crate::state::RuntimeState, scope: Option<&RecordScope>,
        at: &harmos::storage::StorageCursor, include_leases: bool,
    ) -> crate::runtime::Result<harmos::storage::ReadBatch<Vec<dokime_domain::model::traceability::TraceabilitySnapshot>>> {
        let snapshots = crate::traceability::retained_state(state, scope, &crate::model::time::now_utc())?;
        let mut streams = Vec::new();
        streams.try_reserve_exact(1).map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
        streams.push(Identifier::new("host.system").expect("valid stream").into());
        let host = crate::runtime::storage::telemetry::collect_saved(
            session,
            at,
            &TelemetrySelection { streams, ..Default::default() },
            self.schema_settings(),
            crate::runtime::storage::telemetry::ReadOptions { window: Some(("host.system", 0.0)), visible: scope, ..Default::default() },
        )
        .await?;
        self.enrich_retained_traceability(snapshots, scope, &host, include_leases)
    }

    pub(crate) async fn with_latest_host_traceability<T>(
        &self, scope: Option<&RecordScope>, read: impl FnOnce(Option<&TelemetrySample>, &str) -> crate::runtime::Result<T>,
    ) -> crate::runtime::Result<T> {
        self.with_host_traceability_at(scope, &self.storage().cursor_saved(), read).await
    }

    async fn with_host_traceability_at<T>(
        &self, scope: Option<&RecordScope>, at: &harmos::storage::StorageCursor,
        read: impl FnOnce(Option<&TelemetrySample>, &str) -> crate::runtime::Result<T>,
    ) -> crate::runtime::Result<T> {
        let host = crate::runtime::storage::telemetry::collect_saved(
            &self.storage().clone(),
            at,
            &TelemetrySelection { streams: vec![Identifier::new("host.system").expect("valid stream").into()], ..Default::default() },
            self.schema_settings(),
            crate::runtime::storage::telemetry::ReadOptions { window: Some(("host.system", 0.0)), visible: scope, ..Default::default() },
        )
        .await?;
        let latest = host
            .iter()
            .filter(|sample| scope.is_none_or(|selected| scope_visible_in(sample.scope.as_ref(), selected)))
            .max_by(|left, right| left.timestamp.cmp(&right.timestamp).then_with(|| left.sequence.cmp(&right.sequence)));
        let fingerprint = format!("host.system:{}:{}:{}", at.history.store, at.history.epoch, at.sequence);
        read(latest, &fingerprint)
    }
}

/* === Public Functions === */

pub fn telemetry_stream_id_len(scope: Option<&RecordScope>, stream: &Identifier) -> Option<usize> {
    let cycle = scope.and_then(|scope| scope.test_cycle_id.as_ref()).map_or(UNSCOPED_STREAM_SEGMENT, |id| id.as_str());
    let execution = scope.and_then(|scope| scope.test_execution_id.as_ref()).map_or(UNSCOPED_STREAM_SEGMENT, |id| id.as_str());
    [TELEMETRY_STREAM_PREFIX.len(), 3 * TELEMETRY_STREAM_DELIMITER.len_utf8(), cycle.len(), execution.len(), stream.as_str().len()]
        .into_iter()
        .try_fold(0usize, |bytes, next| bytes.checked_add(next))
}

pub fn telemetry_stream_id(scope: Option<&RecordScope>, stream: &Identifier) -> String {
    let cycle = scope.and_then(|scope| scope.test_cycle_id.as_ref()).map_or(UNSCOPED_STREAM_SEGMENT, |id| id.as_str());
    let execution = scope.and_then(|scope| scope.test_execution_id.as_ref()).map_or(UNSCOPED_STREAM_SEGMENT, |id| id.as_str());
    let mut id = String::with_capacity(telemetry_stream_id_len(scope, stream).expect("telemetry key length fits address space"));
    id.push_str(TELEMETRY_STREAM_PREFIX);
    for segment in [cycle, execution, stream.as_str()] {
        id.push(TELEMETRY_STREAM_DELIMITER);
        id.push_str(segment);
    }
    id
}
pub(super) fn lease_accepts_sample(source: Option<&RecordScope>, target: &RecordScope) -> bool {
    source.is_none_or(|source| {
        source.test_execution_id.as_ref().is_none_or(|id| Some(id) == target.test_execution_id.as_ref())
            && source.test_cycle_id.as_ref().is_none_or(|id| Some(id) == target.test_cycle_id.as_ref())
            && source.test_case_id.as_ref().is_none_or(|id| Some(id) == target.test_case_id.as_ref())
            && (source.test_execution_id.is_some() || (source.step_id.is_none() && source.criterion_id.is_none()))
    })
}

/* === Private Functions === */

fn normalize_batch(
    batch: &mut TelemetryBatch, contract: &crate::telemetry::TelemetryContract, received_at: &Timestamp,
) -> crate::runtime::Result<()> {
    batch.schema_version = contract.schema_version;
    for sample in &mut batch.samples {
        sample.stream = contract.stream.clone();
        sample.schema_version = contract.schema_version;
        sample.received_at = received_at.clone();
        if sample.storage_payload.is_none()
            && let Ok(host) = crate::host::HostTelemetryStream::try_from(contract.stream.as_str())
        {
            use dokime_domain::model::telemetry::host::prepare_host_telemetry_payload;
            let row = prepare_host_telemetry_payload(host, &sample.values)
                .map_err(|error| validation_rejection(error.to_string(), "telemetry.payload", None))?;
            sample.storage_payload = Some(row.into_bytes());
        }
        if contract.storage_schema.is_none() || sample.storage_payload.is_none() {
            return Err(validation_rejection(
                "telemetry storage requires an explicit row graph and original canonical payload bytes",
                "telemetry.payload",
                None,
            ));
        }
        ensure_telemetry_timestamp(sample).map_err(|message| {
            validation_rejection(message, "telemetry.timestamp", Some("an RFC 3339 timestamp at or after the Unix epoch".to_owned()))
        })?;
    }
    // The Owned Admission Performs The Semantic Check Once After Attribution.
    Ok(())
}

fn ensure_telemetry_timestamp(sample: &mut TelemetrySample) -> Result<(), String> {
    if sample.values.contains_key("timestamp") {
        return Ok(());
    }
    let timestamp = sample
        .timestamp
        .unix_millis()
        .ok_or_else(|| format!("sample timestamp `{}` is not an RFC 3339 instant at or after the Unix epoch", sample.timestamp))?;
    sample.values.insert("timestamp".to_owned(), Value::Number(timestamp as f64 / 1_000.0));
    Ok(())
}

/* === Tests === */
