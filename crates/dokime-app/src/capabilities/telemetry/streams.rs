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
use std::{
    collections::{BTreeMap, BTreeSet},
    pin::Pin,
};
// Third Party Imports
use futures_util::{Stream, StreamExt};
// Local Imports
use crate::evidence::{
    PublishReceipt, TelemetrySelection, retain_running_samples, validation_rejection,
};
use crate::runtime::handle::RuntimeHandle;
use crate::streams::{CycleTrack, ExecutionTrack, TelemetryRow};
use crate::unavailable_rejection;
use dokime::core::{Identifier, RecordScope, Timestamp, Value, scope_visible_in};
use dokime::telemetry::{
    TelemetryBatch, TelemetrySample, ValidationMode, validate_telemetry_sample,
};

/* === Constants === */
const TELEMETRY_STREAM_PREFIX: &str = "dokime.tel";
const TELEMETRY_STREAM_DELIMITER: char = '|';
const UNSCOPED_STREAM_SEGMENT: &str = "-";

/* === Macros === */

/* === Type Aliases === */
pub(crate) type TelemetryWatch = Pin<Box<dyn Stream<Item = u64> + Send>>;
type PreparedSample = (TelemetrySample, dokime::telemetry::TelemetryContract, u64);

/* === Structs === */
pub(crate) struct SampleAdmission {
    _order: tokio::sync::OwnedMutexGuard<()>,
    execution: crate::runtime::storage::telemetry::TelemetryAdmission,
    accepted: Vec<PreparedSample>,
    rows: Vec<TelemetryRow>,
    delivery: super::leases::LeasedRows,
    cursors: BTreeMap<String, u64>,
}

/* === Enums === */
#[derive(Clone, Debug, Eq, PartialEq)]
enum TelemetryRead {
    Unfiltered,
    Cycle(CycleTrack),
    Execution(CycleTrack, ExecutionTrack),
}

/* === Traits === */

/* === Implementations === */
impl RuntimeHandle {
    pub(crate) async fn publish_telemetry_to_streams(
        &self,
        batch: TelemetryBatch,
        received_at: Timestamp,
    ) -> std::result::Result<PublishReceipt, crate::Error> {
        let runtime = self.clone();
        tokio::spawn(async move {
            runtime
                .publish_telemetry_to_streams_inner(batch, received_at)
                .await
        })
        .await
        .map_err(|e| unavailable_rejection(e.to_string()))?
    }
    async fn publish_telemetry_to_streams_inner(
        &self,
        mut batch: TelemetryBatch,
        received_at: Timestamp,
    ) -> std::result::Result<PublishReceipt, crate::Error> {
        let sampling = self.sampling();
        let _publish = sampling.begin_publish().await;
        let closing_executions = sampling.closing_executions().await;
        let state = self.state_read();
        let contract =
            crate::runtime_state::contract_lookup(&state).telemetry_contract(&batch.stream);
        for sample in &batch.samples {
            if let Some(scope) = &sample.scope {
                state.capture_scope(scope).map_err(|reason| {
                    crate::Error::Operation(format!("telemetry acquisition refused: {reason}"))
                })?;
            }
        }
        let dropped = retain_open_execution_samples(&mut batch, &closing_executions)
            + retain_running_samples(&state, &mut batch);
        let revision = self.state_manager.applied().0;
        let Some(contract) = contract else {
            return Err(validation_rejection(
                format!("telemetry stream {} is not registered", batch.stream),
                "stream",
                Some("active telemetry contract".to_owned()),
            ));
        };
        if batch.samples.is_empty() {
            return Ok(PublishReceipt::dropped(
                dropped,
                revision,
                "telemetry sample(s) dropped because the scoped execution or cycle is not running",
            ));
        }

        normalize_batch(&mut batch, &contract, &received_at)?;

        let accepted = batch.samples.len() as u64;
        let stream_cursors = self
            .write_samples_to_streams(&batch.samples)
            .await
            .map_err(|error| {
                unavailable_rejection(format!("telemetry stream write failed: {error}"))
            })?;
        // Admission also delivers the frozen lease batch while this publish
        // guard is held. Quiescence therefore waits for both capture and
        // physical delivery; a later lease never receives an uncaptured row.
        let mut receipt = PublishReceipt::accepted(accepted, stream_cursors, revision);
        receipt.dropped = dropped;

        Ok(receipt)
    }

    pub(crate) async fn publish_captured_telemetry(
        &self,
        completion: crate::services::telemetry::CapturedSamples,
        received_at: Timestamp,
    ) -> crate::Result<PublishReceipt> {
        let sampling = self.sampling();
        let _publish = sampling.begin_publish().await;
        let mut batch = completion.into_batch();
        let contract = crate::runtime_state::contract_lookup(&self.state_read())
            .telemetry_contract(&batch.stream)
            .ok_or_else(|| {
                validation_rejection(
                    format!("telemetry stream {} is not registered", batch.stream),
                    "stream",
                    Some("active telemetry contract".to_owned()),
                )
            })?;
        normalize_batch(&mut batch, &contract, &received_at)?;
        let accepted = batch.samples.len() as u64;
        let revision = self.state_manager.applied().0;
        // The Original Storage Admission Rejects A Closed Acquisition And Retains Capacity.
        let admission = self.prepare_sample_admission(&batch.samples).await?;
        let cursors = self.admit_samples(admission).await?;
        Ok(PublishReceipt::accepted(accepted, cursors, revision))
    }

    async fn write_samples_to_streams(
        &self,
        samples: &[TelemetrySample],
    ) -> crate::Result<BTreeMap<String, u64>> {
        let runtime = self.clone();
        let samples = samples.to_vec();
        tokio::spawn(async move {
            let admission = runtime.prepare_sample_admission(&samples).await?;
            runtime.admit_samples(admission).await
        })
        .await
        .map_err(|error| crate::Error::Operation(error.to_string()))?
    }

    pub(crate) async fn prepare_sample_admission(
        &self,
        samples: &[TelemetrySample],
    ) -> crate::Result<SampleAdmission> {
        if samples.len() > 4096 {
            return Err(crate::Error::overload(
                "telemetry_capture_busy",
                "telemetry capture exceeds 4096 rows; split the batch",
            ));
        }
        let order = self.telemetry_admission.clone().lock_owned().await;
        let mut delivery = self.leased_rows(samples).await;
        let open = self.execution_storage.telemetry_scopes().await?;
        let mut leased = BTreeMap::new();
        for sample in samples {
            if leased.contains_key(&sample.stream) {
                continue;
            }
            let scopes = delivery.capture_scopes(&sample.stream);
            let mut owners = BTreeMap::new();
            for scope in scopes {
                if let Some(id) = scope.test_execution_id.as_ref()
                    && let Some(canonical) = open.get(id)
                {
                    owners.insert(id.clone(), canonical.clone());
                }
            }
            leased.insert(
                sample.stream.clone(),
                owners.into_values().collect::<Vec<_>>(),
            );
        }
        let state = self
            .state_manager
            .state()
            .read(Clone::clone)
            .await
            .map_err(|error| crate::Error::Operation(error.to_string()))?
            .value;
        // Validate The Entire Supplied Batch Before Closed Scope Filtering.
        for sample in samples {
            if let Some(scope) = &sample.scope {
                state.capture_scope(scope).map_err(|reason| {
                    crate::Error::Operation(format!("telemetry acquisition refused: {reason}"))
                })?;
            }
        }
        let mut samples = captured_samples(samples, &leased)?;
        delivery.attribute(&state)?;
        for sample in &mut samples {
            if let Some(scope) = &sample.scope {
                sample.scope = Some(
                    state
                        .capture_scope(scope)
                        .map_err(crate::Error::Operation)?,
                );
            }
        }
        let mut accepted = Vec::with_capacity(samples.len());
        for sample in samples {
            let contract = crate::runtime_state::contract_lookup(&state)
                .telemetry_contract(&sample.stream)
                .ok_or_else(|| {
                    validation_rejection("telemetry contract is unavailable", "stream", None)
                })?;
            validate_telemetry_sample(Some(&contract), &sample, ValidationMode::Strict)
                .map_err(|error| validation_rejection(error.to_string(), "telemetry", None))?;
            accepted.push((sample, contract, 0));
        }
        let (rows, cursors) = {
            let known = self
                .telemetry_cursors
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let mut cursors = BTreeMap::new();
            let mut rows = Vec::with_capacity(accepted.len());
            for (sample, _, ordinal) in &mut accepted {
                let stream = telemetry_stream_id(sample.scope.as_ref(), &sample.stream);
                let next = cursors
                    .entry(stream.clone())
                    .or_insert_with(|| known.get(&stream).copied().unwrap_or(0));
                *next = next
                    .checked_add(1)
                    .ok_or_else(|| crate::Error::Operation("telemetry ordinal exhausted".into()))?;
                *ordinal = *next;
                rows.push(telemetry_row(sample, *next)?);
            }
            (rows, cursors)
        };
        let host_count = accepted
            .iter()
            .filter(|(sample, _, _)| sample.stream.as_str() == "host.system")
            .count() as u64;
        self.host_traceability
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .generation
            .checked_add(host_count)
            .ok_or_else(|| {
                crate::Error::Operation("host traceability generation exhausted".into())
            })?;
        let execution = self.execution_storage.prepare_telemetry(&accepted).await?;
        Ok(SampleAdmission {
            _order: order,
            execution,
            accepted,
            rows,
            cursors,
            delivery,
        })
    }

    pub(crate) async fn admit_samples(
        &self,
        admission: SampleAdmission,
    ) -> crate::Result<BTreeMap<String, u64>> {
        let runtime = self.clone();
        tokio::spawn(async move { runtime.admit_samples_owned(admission).await }).await?
    }

    async fn admit_samples_owned(
        &self,
        admission: SampleAdmission,
    ) -> crate::Result<BTreeMap<String, u64>> {
        let SampleAdmission {
            _order,
            execution,
            accepted,
            rows,
            cursors,
            delivery,
        } = admission;
        execution.commit().await?;
        for row in rows {
            self.streams.publish(row);
        }
        {
            let mut host = self
                .host_traceability
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            for (sample, _, ordinal) in accepted {
                if sample.stream.as_str() != "host.system" {
                    continue;
                }
                let stream = telemetry_stream_id(sample.scope.as_ref(), &sample.stream);
                let mut sample = sample;
                sample.sequence = ordinal;
                // Checked while the same telemetry admission guard was held.
                host.generation += 1;
                host.samples.insert(stream, sample);
            }
        }
        {
            let mut known = self
                .telemetry_cursors
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            for (stream, ordinal) in &cursors {
                known
                    .entry(stream.clone())
                    .and_modify(|known| *known = (*known).max(*ordinal))
                    .or_insert(*ordinal);
            }
        }
        drop(_order);
        self.forward_leased_rows(delivery).await;
        Ok(cursors)
    }

    pub async fn watch_plot_telemetry(
        &self,
        _requests: &[crate::queries::PlotPayloadRequest],
    ) -> crate::Result<Vec<TelemetryWatch>> {
        let watching = self.streams.watch::<TelemetryRow>().map(|item| match item {
            harmos::streams::Item::Row(row) => row.value.ordinal,
            harmos::streams::Item::Gap { missed } => missed,
        });
        Ok(vec![Box::pin(watching)])
    }

    pub async fn collect_telemetry(&self, selection: &TelemetrySelection) -> Vec<TelemetrySample> {
        self.try_collect_telemetry(selection)
            .await
            .unwrap_or_else(|error| {
                tracing::error!(%error, "failed to read Harmos telemetry series");
                Vec::new()
            })
    }

    pub(crate) async fn try_collect_telemetry(
        &self,
        selection: &TelemetrySelection,
    ) -> crate::Result<Vec<TelemetrySample>> {
        self.try_collect_telemetry_bounded(selection, None).await
    }

    pub(crate) async fn try_collect_telemetry_bounded(
        &self,
        selection: &TelemetrySelection,
        scan_limit: Option<usize>,
    ) -> crate::Result<Vec<TelemetrySample>> {
        let cycle = selection.scope.test_cycle_id.clone().or_else(|| {
            let execution_id = selection.scope.test_execution_id.as_ref()?;
            let state = self.state_read();
            state
                .executions
                .get(execution_id)
                .map(|execution| execution.test_cycle_id.clone())
        });
        let read = telemetry_read_plan(selection, cycle);
        let mut rows = match read {
            TelemetryRead::Unfiltered => self.streams.query::<TelemetryRow>(..).retained().await,
            TelemetryRead::Cycle(cycle) => {
                self.streams
                    .query::<TelemetryRow>(..)
                    .retained()
                    .track((cycle,))
                    .await
            }
            TelemetryRead::Execution(cycle, execution) => {
                self.streams
                    .query::<TelemetryRow>(..)
                    .retained()
                    .track((cycle, execution))
                    .await
            }
        }
        .map_err(|error| crate::Error::Operation(error.to_string()))?
        .take(scan_limit.map_or(usize::MAX, |limit| limit.saturating_add(1)))
        .collect::<Vec<_>>();
        if scan_limit.is_some_and(|limit| rows.len() > limit) {
            return Err(crate::Error::Validation {
                code: "analysis_read_budget".into(),
                message: "analysis raw evidence scan exceeds its budget; narrow the scope".into(),
                issues: vec![],
            });
        }

        // Preserve publication order across tracks within the selected prefix.
        rows.sort_by_key(|row| row.position);
        let samples = rows
            .into_iter()
            .map(|row| row.value)
            .map(sample_from_row)
            .collect::<crate::Result<Vec<_>>>()?
            .into_iter()
            .filter(|sample| {
                let stream = telemetry_stream_id(sample.scope.as_ref(), &sample.stream);
                selection
                    .after_stream_cursors
                    .get(&stream)
                    .is_none_or(|after| sample.sequence > *after)
            })
            .collect::<Vec<_>>();
        Ok(selection.apply(&samples))
    }

    pub(crate) async fn telemetry_sample_count(&self) -> usize {
        self.telemetry_cursors
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .values()
            .fold(0_usize, |count, cursor| {
                count.saturating_add(usize::try_from(*cursor).unwrap_or(usize::MAX))
            })
    }

    pub(crate) async fn telemetry_aggregate_outputs(
        &self,
        scope: &dokime::core::RecordScope,
    ) -> crate::Result<Vec<crate::telemetry_runtime::output::AggregateOutput>> {
        let state = self.settled_evidence_state().await;
        let contracts = state.telemetry_contracts();
        let samples = self
            .collect_telemetry(&TelemetrySelection {
                scope: crate::evidence::ScopeSelection {
                    test_cycle_id: scope.test_cycle_id.clone(),
                    test_execution_id: scope.test_execution_id.clone(),
                    test_case_id: scope.test_case_id.clone(),
                    template_id: scope.template_id.clone(),
                },
                ..TelemetrySelection::default()
            })
            .await;
        Ok(crate::telemetry_runtime::output::project_samples(
            &contracts, &samples,
        ))
    }

    pub(crate) async fn enrich_host_traceability(
        &self,
        snapshots: &mut Vec<crate::traceability::TraceabilitySnapshot>,
        scope: Option<&dokime::core::RecordScope>,
    ) {
        let mut host = self
            .collect_telemetry(&TelemetrySelection {
                streams: vec![Identifier::new("host.system").expect("valid stream")],
                ..TelemetrySelection::default()
            })
            .await;
        host.sort_by_key(|sample| sample.sequence);
        if let Some(snapshot) = crate::traceability::host_traceability_snapshot_from_samples(
            host.iter(),
            scope,
            Timestamp::now_utc(),
        ) {
            crate::traceability::replace_snapshots_by_source(
                snapshots,
                "traceability.host",
                snapshot,
            );
        }
    }

    pub(crate) async fn enrich_lease_traceability(
        &self,
        snapshots: &mut Vec<crate::traceability::TraceabilitySnapshot>,
        scope: Option<&dokime::core::RecordScope>,
    ) {
        let selected = scope.cloned().unwrap_or_default();
        let leases = self.stream_leases().facts_for(&selected);
        if let Some(snapshot) = crate::traceability::lease_traceability_snapshot(
            &leases,
            &selected,
            Timestamp::now_utc(),
        ) {
            crate::traceability::replace_snapshots_by_source(
                snapshots,
                crate::traceability::LEASES_TRACEABILITY.source,
                snapshot,
            );
        }
    }

    pub(crate) fn with_latest_host_traceability<T>(
        &self,
        scope: Option<&RecordScope>,
        read: impl FnOnce(Option<&dokime::telemetry::TelemetrySample>, &str) -> crate::Result<T>,
    ) -> crate::Result<T> {
        let host_traceability = self
            .host_traceability
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let fingerprint = format!("host.system:g{}", host_traceability.generation);
        let latest = host_traceability
            .samples
            .values()
            .filter(|sample| {
                scope.is_none_or(|selected| scope_visible_in(sample.scope.as_ref(), selected))
            })
            .max_by(|left, right| {
                left.timestamp
                    .cmp(&right.timestamp)
                    .then_with(|| left.sequence.cmp(&right.sequence))
                    .then_with(|| left.sample_id.cmp(&right.sample_id))
            });
        read(latest, &fingerprint)
    }
}

/* === Public Functions === */
pub fn telemetry_stream_id(
    scope: Option<&dokime::core::RecordScope>,
    stream: &Identifier,
) -> String {
    let (cycle, execution) = telemetry_track(scope);
    telemetry_stream_id_from_track(&cycle, &execution, stream.as_str())
}

/* === Private Functions === */
fn normalize_batch(
    batch: &mut TelemetryBatch,
    contract: &dokime::telemetry::TelemetryContract,
    received_at: &Timestamp,
) -> crate::Result<()> {
    batch.schema_version = contract.schema_version;
    for sample in &mut batch.samples {
        sample.stream = contract.stream.clone();
        sample.schema_version = contract.schema_version;
        sample.received_at = received_at.clone();
        ensure_telemetry_timestamp(sample).map_err(|message| {
            validation_rejection(
                message,
                "telemetry.timestamp",
                Some("an RFC 3339 timestamp at or after the Unix epoch".to_owned()),
            )
        })?;
        validate_telemetry_sample(Some(contract), sample, ValidationMode::Strict).map_err(
            |error| {
                validation_rejection(
                    error.to_string(),
                    format!("telemetry.{}", contract.stream.as_str()),
                    Some(format!("schema_version {}", contract.schema_version)),
                )
            },
        )?;
    }
    Ok(())
}

fn telemetry_row(sample: &TelemetrySample, ordinal: u64) -> crate::Result<TelemetryRow> {
    let timestamp = sample.timestamp.unix_millis().ok_or_else(|| {
        crate::Error::Operation(format!(
            "sample timestamp `{}` is not at or after the Unix epoch",
            sample.timestamp
        ))
    })?;
    let payload = crate::persistence::to_vec(sample)
        .map_err(|error| crate::Error::Operation(error.to_string()))?;
    let (cycle, execution) = telemetry_track(sample.scope.as_ref());
    Ok(TelemetryRow {
        cycle,
        execution,
        ordinal,
        timestamp_millis: timestamp,
        stream: sample.stream.to_string(),
        payload,
    })
}

fn sample_from_row(row: TelemetryRow) -> crate::Result<TelemetrySample> {
    let mut sample: TelemetrySample = crate::persistence::from_slice(&row.payload)
        .map_err(|error| crate::Error::Operation(format!("invalid telemetry row: {error}")))?;
    sample.sequence = row.ordinal;
    Ok(sample)
}

fn telemetry_track(scope: Option<&dokime::core::RecordScope>) -> (CycleTrack, ExecutionTrack) {
    (
        CycleTrack::from_id(scope.and_then(|scope| scope.test_cycle_id.clone())),
        ExecutionTrack::from_id(scope.and_then(|scope| scope.test_execution_id.clone())),
    )
}

fn telemetry_read_plan(
    selection: &TelemetrySelection,
    resolved_cycle: Option<dokime::core::TestCycleId>,
) -> TelemetryRead {
    let cycle = resolved_cycle.map(|cycle| CycleTrack::from_id(Some(cycle)));
    let execution = selection
        .scope
        .test_execution_id
        .clone()
        .map(|execution| ExecutionTrack::from_id(Some(execution)));

    match (cycle, execution) {
        (Some(cycle), Some(execution)) => TelemetryRead::Execution(cycle, execution),
        (Some(cycle), None) => TelemetryRead::Cycle(cycle),
        (None, Some(execution)) => TelemetryRead::Execution(CycleTrack::from_id(None), execution),
        (None, None) => TelemetryRead::Unfiltered,
    }
}

fn telemetry_stream_id_from_track(
    cycle: &CycleTrack,
    execution: &ExecutionTrack,
    stream: &str,
) -> String {
    format!(
        "{TELEMETRY_STREAM_PREFIX}{delimiter}{cycle}{delimiter}{execution}{delimiter}{stream}",
        delimiter = TELEMETRY_STREAM_DELIMITER,
        cycle = cycle.as_str().unwrap_or(UNSCOPED_STREAM_SEGMENT),
        execution = execution.as_str().unwrap_or(UNSCOPED_STREAM_SEGMENT),
        stream = stream,
    )
}

pub(super) fn lease_accepts_sample(source: Option<&RecordScope>, target: &RecordScope) -> bool {
    source.is_none_or(|source| {
        source
            .test_execution_id
            .as_ref()
            .is_none_or(|id| Some(id) == target.test_execution_id.as_ref())
            && source
                .test_cycle_id
                .as_ref()
                .is_none_or(|id| Some(id) == target.test_cycle_id.as_ref())
            && source
                .test_case_id
                .as_ref()
                .is_none_or(|id| Some(id) == target.test_case_id.as_ref())
            && (source.test_execution_id.is_some()
                || (source.step_id.is_none() && source.criterion_id.is_none()))
    })
}

fn captured_samples(
    samples: &[TelemetrySample],
    leased: &BTreeMap<Identifier, Vec<RecordScope>>,
) -> crate::Result<Vec<TelemetrySample>> {
    let mut captured = Vec::new();
    for sample in samples {
        if captured.len() >= 4096 {
            return Err(crate::Error::overload(
                "telemetry_capture_busy",
                "telemetry capture exceeds 4096 rows; split the batch",
            ));
        }
        captured.push(sample.clone());
        if sample
            .scope
            .as_ref()
            .is_some_and(|scope| scope.test_execution_id.is_some())
        {
            continue;
        }
        for scope in leased.get(&sample.stream).into_iter().flatten() {
            if !lease_accepts_sample(sample.scope.as_ref(), scope) {
                continue;
            }
            if captured.len() >= 4096 {
                return Err(crate::Error::overload(
                    "telemetry_capture_busy",
                    "telemetry capture exceeds 4096 rows; split the batch",
                ));
            }
            let mut owned = sample.clone();
            owned.scope = Some(scope.clone());
            captured.push(owned);
        }
    }
    Ok(captured)
}

fn ensure_telemetry_timestamp(sample: &mut TelemetrySample) -> Result<(), String> {
    if sample.values.contains_key("timestamp") {
        return Ok(());
    }
    let timestamp = sample.timestamp.unix_millis().ok_or_else(|| {
        format!(
            "sample timestamp `{}` is not an RFC 3339 instant at or after the Unix epoch",
            sample.timestamp
        )
    })?;
    sample.values.insert(
        "timestamp".to_owned(),
        Value::Number(timestamp as f64 / 1_000.0),
    );
    Ok(())
}

fn retain_open_execution_samples(
    batch: &mut TelemetryBatch,
    closing_executions: &BTreeSet<dokime::core::TestExecutionId>,
) -> u64 {
    let before = batch.samples.len();
    batch.samples.retain(|sample| {
        sample
            .scope
            .as_ref()
            .and_then(|scope| scope.test_execution_id.as_ref())
            .is_none_or(|id| !closing_executions.contains(id))
    });
    (before - batch.samples.len()) as u64
}

/* === Tests === */
