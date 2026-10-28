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
    sync::{Arc, Mutex},
    time::Duration,
};

// Third Party Imports

// Local Imports
use crate::runtime::handle::RuntimeHandle;
use crate::telemetry::{SamplingScope, TelemetryCaptureLease, TelemetrySamplingRequest, TelemetrySourceKind};
use crate::telemetry_runtime::sampling::record_scope_from_sampling_scope;
use crate::transport::DeliverSamplesRequest;
use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

pub(crate) const HOST_PRODUCER_KEY: &str = "host";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct StreamLeases {
    state: Mutex<LeaseState>,
}

#[derive(Default)]
struct LeaseState {
    closed_windows: usize,
    instances: BTreeMap<Identifier, Instance>,
    ledger: Vec<LeaseWindow>,
    held: BTreeMap<String, Hold>,
}

struct PendingLeases {
    table: Arc<StreamLeases>,
    ids: Vec<String>,
}

struct Hold {
    stream: Identifier,
    closing: bool,
}

struct Instance {
    owner: StreamOwner,

    #[allow(dead_code, reason = "the declared cadence the running instance was started at")]
    interval: Duration,

    holders: BTreeSet<String>,
    captures: BTreeMap<String, String>,
    generation: u64,
}

#[derive(Clone, Debug)]
pub struct LeaseFact {
    pub stream: Identifier,
    pub owner: String,
    pub opened_at: Timestamp,
    pub closed_at: Option<Timestamp>,
    pub shared_with: Vec<String>,
    pub gap: bool,
}

pub(crate) struct Released {
    pub(crate) captures: Vec<String>,
}

#[derive(Clone, Debug)]
struct LeaseWindow {
    pub lease_id: String,
    pub stream: Identifier,
    pub owner: StreamOwner,
    pub requester: String,
    pub scope: RecordScope,
    pub opened_at: Timestamp,
    pub closed_at: Option<Timestamp>,
    pub gap: bool,
}

#[derive(Clone, Debug)]
struct Holder {
    pub scope: RecordScope,
    pub requester: String,
}

pub(crate) struct LeasedRows {
    holders: BTreeMap<Identifier, Vec<Holder>>,
}

/* === Enums === */

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum StreamOwner {
    Host,
    Sidecar(String),
}

/* === Traits === */

/* === Implementations === */

impl Drop for PendingLeases {
    fn drop(&mut self) {
        let mut state = self.table.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        for lease_id in &self.ids {
            let attached = state
                .held
                .get(lease_id)
                .is_some_and(|hold| state.instances.get(&hold.stream).is_some_and(|instance| instance.captures.contains_key(lease_id)));
            if attached {
                // Physical Capture Remains With Its Original Settlement Owner.
                continue;
            }
            state.release(lease_id);
        }
    }
}

impl LeasedRows {
    pub(crate) fn attribute(
        owner: harmos::storage::ReadBatch<Self>, state: &crate::state::RuntimeState, samples: &[crate::telemetry::TelemetrySample],
    ) -> crate::runtime::Result<harmos::storage::ReadBatch<Self>> {
        owner.try_map(|mut delivery| {
            for holder in delivery.holders.values_mut().flatten() {
                holder.scope = state.capture_scope(&holder.scope).map_err(crate::runtime::Error::Operation)?;
            }
            for (stream, holders) in &mut delivery.holders {
                let extra = samples
                    .iter()
                    .filter(|sample| {
                        &sample.stream == stream && sample.scope.as_ref().is_some_and(|scope| scope.test_execution_id.is_some())
                    })
                    .count();
                let needed = holders.len().checked_add(extra).ok_or_else(lease_memory_error)?;
                if holders.capacity() < needed {
                    holders.try_reserve_exact(needed - holders.len()).map_err(|_| lease_memory_error())?;
                }
            }
            for sample in samples {
                let Some(scope) = &sample.scope else {
                    continue;
                };
                let scope = state.capture_scope(scope).map_err(crate::runtime::Error::Operation)?;
                let Some(execution) = scope.test_execution_id.as_ref().and_then(|id| state.executions.get(id)) else {
                    continue;
                };
                let Some(plugin) = execution.snapshot.plugin() else {
                    continue;
                };
                let holders = delivery.holders.get_mut(&sample.stream).ok_or_else(lease_memory_error)?;
                if !holders
                    .iter()
                    .any(|holder| holder.scope.test_execution_id == scope.test_execution_id && holder.requester == plugin.source)
                {
                    holders.push(Holder { scope, requester: plugin.source.clone() });
                }
            }
            Ok(delivery)
        })
    }

    pub(crate) fn capture_scopes(&self, stream: &Identifier) -> impl Iterator<Item = &RecordScope> + Clone {
        self.holders.get(stream).into_iter().flatten().map(|holder| &holder.scope)
    }
}

impl LeaseState {
    fn release(&mut self, lease_id: &str) -> Option<Released> {
        let stream = self.held.remove(lease_id)?.stream;
        let closed_at = crate::model::time::now_utc();
        for window in self.ledger.iter_mut().rev() {
            if window.lease_id == lease_id {
                window.closed_at = Some(closed_at);
                break;
            }
        }
        self.bound_ledger();
        let instance = self.instances.get_mut(&stream)?;
        instance.holders.remove(lease_id);
        let last = instance.holders.is_empty();
        // This hold's own demand, always: the reading it paid for ends with it
        // whether or not anyone else is still holding the stream.
        let mut captures = instance.captures.remove(lease_id).into_iter().collect::<Vec<_>>();
        if last {
            captures.extend(
                self.instances.remove(&stream).map(|instance| instance.captures.into_values().collect::<Vec<_>>()).unwrap_or_default(),
            );
        }

        Some(Released { captures })
    }

    fn bound_ledger(&mut self) {
        let closed = self.ledger.iter().filter(|window| window.closed_at.is_some()).count();
        if closed <= self.closed_windows {
            return;
        }
        let mut forget = closed - self.closed_windows;
        self.ledger.retain(|window| {
            if forget > 0 && window.closed_at.is_some() {
                forget -= 1;
                return false;
            }
            true
        });
    }
}

#[derive(Clone, Copy)]
pub(crate) struct LeaseFacts<'a> {
    state: &'a LeaseState,
    scope: &'a RecordScope,
}
#[derive(Clone, Copy)]
pub(crate) struct LeaseFactRef<'a> {
    state: &'a LeaseState,
    window: &'a LeaseWindow,
}
impl<'a> LeaseFacts<'a> {
    pub(crate) fn iter(self) -> impl Iterator<Item = LeaseFactRef<'a>> {
        self.state
            .ledger
            .iter()
            .filter(move |window| window.scope.is_visible_in(self.scope))
            .map(move |window| LeaseFactRef { state: self.state, window })
    }
}
impl<'a> LeaseFactRef<'a> {
    pub(crate) fn stream(self) -> &'a Identifier {
        &self.window.stream
    }
    pub(crate) fn owner(self) -> &'a str {
        match &self.window.owner {
            StreamOwner::Host => "host",
            StreamOwner::Sidecar(source) => source,
        }
    }
    pub(crate) fn opened_at(self) -> &'a Timestamp {
        &self.window.opened_at
    }
    pub(crate) fn closed_at(self) -> Option<&'a Timestamp> {
        self.window.closed_at.as_ref()
    }
    pub(crate) fn gap(self) -> bool {
        self.window.gap
    }
    pub(crate) fn shared_with(self) -> impl Iterator<Item = &'a str> {
        self.state
            .ledger
            .iter()
            .filter(move |other| {
                other.lease_id != self.window.lease_id && other.stream == self.window.stream && overlaps(self.window, other)
            })
            .filter_map(|other| other.scope.test_execution_id.as_ref().map(|id| id.as_str()))
    }
}

impl StreamLeases {
    pub(crate) fn new() -> Self {
        let closed_windows = 4096;
        Self {
            state: Mutex::new(LeaseState { closed_windows, ..LeaseState::default() }),
        }
    }

    fn take(&self, stream: &Identifier, owner: StreamOwner, requester: &str, interval: Duration, scope: RecordScope) -> (String, bool) {
        let lease_id = format!("lease-{}", uuid::Uuid::now_v7());
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let entry = state.instances.entry(stream.clone());
        let started = matches!(entry, std::collections::btree_map::Entry::Vacant(_));
        let instance = entry.or_insert_with(|| Instance {
            owner: owner.clone(),
            interval,
            holders: BTreeSet::new(),
            captures: BTreeMap::new(),
            generation: 0,
        });
        instance.holders.insert(lease_id.clone());
        state.held.insert(lease_id.clone(), Hold { stream: stream.clone(), closing: false });
        state.ledger.push(LeaseWindow {
            lease_id: lease_id.clone(),
            stream: stream.clone(),
            owner,
            requester: requester.to_owned(),
            scope,
            opened_at: crate::model::time::now_utc(),
            closed_at: None,
            gap: false,
        });
        state.bound_ledger();

        (lease_id, started)
    }

    pub(crate) fn release(&self, lease_id: &str) -> Option<Released> {
        self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner).release(lease_id)
    }

    pub(crate) fn fence_release(&self, lease_id: &str) -> Option<Vec<String>> {
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let hold = state.held.get_mut(lease_id)?;
        hold.closing = true;
        let stream = hold.stream.clone();
        let instance = state.instances.get(&stream)?;
        Some(instance.captures.get(lease_id).cloned().into_iter().collect())
    }

    pub(crate) fn attach_captures(&self, captures: &[TelemetryCaptureLease]) -> Result<()> {
        let assignments = captures
            .iter()
            .map(|capture| {
                let stream = Identifier::new(&capture.request.stream).map_err(|error| Error::Operation(error.to_string()))?;
                Ok((stream, &capture.request.request_id, &capture.lease_id))
            })
            .collect::<Result<Vec<_>>>()?;
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        for (stream, lease_id, _) in &assignments {
            let present = state.held.get(*lease_id).is_some_and(|hold| !hold.closing && hold.stream == *stream);
            if !present {
                return Err(Error::Operation("stream hold closed before capture admission".into()));
            }
        }
        for (stream, lease_id, capture) in assignments {
            state
                .instances
                .get_mut(&stream)
                .expect("the admitted hold retains its stream")
                .captures
                .insert(lease_id.clone(), capture.clone());
        }
        Ok(())
    }

    pub(crate) fn mark_gap(&self, stream: &Identifier) {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let Some(instance) = state.instances.get_mut(stream) else {
            return;
        };
        instance.generation = instance.generation.saturating_add(1);
        let open = instance.holders.clone();
        for window in state.ledger.iter_mut() {
            if window.closed_at.is_none() && open.contains(&window.lease_id) {
                window.gap = true;
            }
        }
    }

    fn holders_of(&self, stream: &Identifier) -> Vec<Holder> {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let Some(instance) = state.instances.get(stream) else {
            return Vec::new();
        };
        state
            .ledger
            .iter()
            .filter(|window| window.closed_at.is_none() && instance.holders.contains(&window.lease_id))
            .map(|window| Holder {
                scope: window.scope.clone(),
                requester: window.requester.clone(),
            })
            .collect()
    }

    fn owner_of(&self, stream: &Identifier) -> Option<String> {
        self.state.lock().unwrap_or_else(|error| error.into_inner()).instances.get(stream).map(|instance| match &instance.owner {
            StreamOwner::Host => "host".to_owned(),
            StreamOwner::Sidecar(source) => source.clone(),
        })
    }

    pub(crate) fn is_active(&self, lease_id: &str) -> bool {
        self.state.lock().unwrap_or_else(|error| error.into_inner()).held.contains_key(lease_id)
    }

    pub(crate) fn with_facts<R>(&self, scope: &RecordScope, read: impl FnOnce(LeaseFacts<'_>) -> R) -> R {
        let state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        read(LeaseFacts { state: &state, scope })
    }

    pub(crate) fn facts_for(&self, scope: &RecordScope) -> Vec<LeaseFact> {
        self.with_facts(scope, |facts| {
            facts
                .iter()
                .map(|fact| LeaseFact {
                    stream: fact.stream().clone(),
                    owner: fact.owner().to_owned(),
                    opened_at: fact.opened_at().clone(),
                    closed_at: fact.closed_at().cloned(),
                    shared_with: fact.shared_with().map(str::to_owned).collect(),
                    gap: fact.gap(),
                })
                .collect()
        })
    }
}

impl RuntimeHandle {
    pub async fn lease_streams(&self, scope: SamplingScope, streams: &[Identifier], requester: &str) -> Result<Vec<String>> {
        let record_scope = self.state_read().capture_scope(&record_scope_from_sampling_scope(&scope)).map_err(Error::Operation)?;
        if let Some(id) = &record_scope.test_execution_id
            && self.execution_storage.acquisition(id).await.is_none()
        {
            // A Lease Can Precede Provider Dispatch; Its Durable Capture Owner Must Exist.
            let session = self.storage().clone();
            let cycle = self.execution_storage.report_source(&session, id).await?;
            let execution = cycle
                .test_executions
                .iter()
                .find(|execution| &execution.test_execution_id == id)
                .ok_or_else(|| Error::Operation("lease execution is absent from its cycle".into()))?;
            self.execution_storage.acquire(&cycle, execution).await?;
        }
        let mut pending = PendingLeases {
            table: self.stream_leases(),
            ids: Vec::with_capacity(streams.len()),
        };
        // Every hold on a host stream gets its own demand, because a stored
        // row carries one scope and each run reads its evidence back by its
        // own. They share one producer key, so the stream is still read once
        // per tick. Only a stream this call actually *started* needs its
        // owner reached, which is the sidecar arm below.
        let mut host_holds = Vec::new();
        for stream in streams {
            let (owner, interval) = self.stream_owner_and_cadence(stream)?;
            let (lease_id, _started) = self.stream_leases().take(stream, owner.clone(), requester, interval, record_scope.clone());
            pending.ids.push(lease_id.clone());
            match owner {
                StreamOwner::Host => host_holds.push((stream.clone(), interval, lease_id)),
                StreamOwner::Sidecar(_) => {}
            }
        }
        if !host_holds.is_empty() {
            self.start_host_captures(scope, requester, &host_holds).await?;
        }

        Ok(std::mem::take(&mut pending.ids))
    }

    pub async fn release_stream_leases(&self, leases: Vec<String>) -> Result<()> {
        let sampling = self.sampling();
        let stream_leases = self.stream_leases();
        for lease_id in leases {
            sampling.release_stream_lease(self, &stream_leases, &lease_id).await?;
        }
        Ok(())
    }

    pub async fn stream_lease_is_active(&self, lease_id: &str) -> bool {
        self.stream_leases().is_active(lease_id)
    }

    pub async fn stream_leases_for(&self, scope: &RecordScope) -> Vec<LeaseFact> {
        self.stream_leases().facts_for(scope)
    }

    pub(crate) fn leased_rows(
        &self, samples: &[crate::telemetry::TelemetrySample],
    ) -> crate::runtime::Result<harmos::storage::ReadBatch<LeasedRows>> {
        use super::streams::admission::slots;

        let mut streams = BTreeSet::new();
        for sample in samples {
            if streams.contains(&sample.stream) {
                continue;
            }
            streams.insert(&sample.stream);
        }
        let table = self.stream_leases();
        let state = table.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let matches = |stream: &Identifier, window: &&LeaseWindow| {
            &window.stream == stream
                && window.closed_at.is_none()
                && state.instances.get(stream).is_some_and(|instance| instance.holders.contains(&window.lease_id))
        };
        harmos::storage::ReadBatch::new(()).try_map(|_| {
            let mut holders = BTreeMap::new();
            for stream in streams {
                let windows = state.ledger.iter().filter(|window| matches(stream, window));
                let mut rows = slots(windows.clone().count())?;
                rows.extend(windows.map(|window| Holder {
                    scope: window.scope.clone(),
                    requester: window.requester.clone(),
                }));
                holders.insert(stream.clone(), rows);
            }
            Ok(LeasedRows { holders })
        })
    }

    // Encode Measurement Fields Once; Append Each Leaseholder's Captured Scope To Its Wire Row.
    pub(crate) async fn forward_leased_rows(
        &self, delivery: harmos::storage::ReadBatch<LeasedRows>, samples: &[super::streams::PreparedSample],
        identities: &[super::streams::admission::SampleIdentity<'_>],
    ) {
        if delivery.holders.values().all(Vec::is_empty) {
            return;
        }
        let mut frames = Vec::new();
        if frames.try_reserve_exact(samples.len()).is_err() {
            tracing::warn!(rows = samples.len(), "leased rows could not reserve their delivery frames");
            return;
        }
        frames.resize(samples.len(), None::<String>);
        for (stream, holders) in &delivery.holders {
            for holder in holders {
                if let Some(id) = holder.scope.test_execution_id.as_ref()
                    && self.sampling().measurement_closed(id.as_str()).await
                {
                    continue;
                }
                let delivered = |row: &&super::streams::PreparedSample| {
                    row.original() && &row.sample.stream == stream && lease_delivers(&row.sample, holder, identities)
                };
                let count = samples.iter().filter(delivered).count();
                if count == 0 {
                    continue;
                }
                let request = (|| -> crate::runtime::Result<_> {
                    let mut encoded = super::streams::admission::slots(count)?;
                    for (row, frame) in samples.iter().zip(frames.iter_mut()).filter(|(row, _)| delivered(row)) {
                        let frame = lease_frame(frame, &LeaseSample(&row.sample))?;
                        let source = row.sample.scope.as_ref();
                        let scope = source
                            .filter(|scope| scope.test_execution_id.is_some() && scope.test_execution_id == holder.scope.test_execution_id)
                            .unwrap_or(&holder.scope);
                        let scope = lease_json(scope)?;
                        let mut copy = String::new();
                        copy.try_reserve_exact(
                            frame.len().checked_add(scope.len()).and_then(|size| size.checked_add(10)).ok_or_else(lease_memory_error)?,
                        )
                        .map_err(|_| lease_memory_error())?;
                        copy.push_str(&frame[..frame.len() - 1]);
                        copy.push_str(",\"scope\":");
                        copy.push_str(&scope);
                        copy.push('}');
                        encoded.push(copy);
                    }
                    Ok(DeliverSamplesRequest {
                        samples: encoded,
                        gaps: Vec::new(),
                        scope_json: lease_json(&holder.scope)?,
                    })
                })();
                match request {
                    Ok(request) => self.deliver_to_leaseholder(holder, request).await,
                    Err(error) => {
                        tracing::warn!(stream = stream.as_str(), detail = %error, "a leased row could not be encoded for its leaseholder")
                    }
                }
            }
        }
    }

    pub(crate) async fn forward_lease_gap(&self, stream: &Identifier, reason: &str) {
        let holders = self.stream_leases().holders_of(stream);
        if holders.is_empty() {
            return;
        }
        let owner = self.stream_leases().owner_of(stream).unwrap_or_else(|| "host".to_owned());
        let gap = crate::telemetry::LeaseGap {
            stream: stream.clone(),
            owner,
            observed_at: crate::model::time::now_utc(),
            reason: reason.to_owned(),
        };
        let Ok(encoded) = serde_json::to_string(&gap) else {
            return;
        };
        for holder in holders {
            self.deliver_to_leaseholder(
                &holder,
                DeliverSamplesRequest {
                    samples: Vec::new(),
                    gaps: vec![encoded.clone()],
                    scope_json: serde_json::to_string(&holder.scope).unwrap_or_default(),
                },
            )
            .await;
        }
    }

    pub(crate) async fn refuse_execution_stream(&self, stream: &Identifier, scope: &RecordScope, requester: &str, reason: &str) {
        let leases = {
            let table = self.stream_leases();
            let mut state = table.state.lock().unwrap_or_else(|error| error.into_inner());
            state
                .ledger
                .iter_mut()
                .filter(|window| {
                    window.stream == *stream
                        && window.closed_at.is_none()
                        && window.scope.test_execution_id == scope.test_execution_id
                        && window.requester == requester
                })
                .map(|window| {
                    window.gap = true;
                    window.lease_id.clone()
                })
                .collect::<Vec<_>>()
        };
        let gap = crate::telemetry::LeaseGap {
            stream: stream.clone(),
            owner: requester.to_owned(),
            observed_at: crate::model::time::now_utc(),
            reason: reason.to_owned(),
        };
        if let Ok(encoded) = serde_json::to_string(&gap) {
            self.deliver_to_leaseholder(
                &Holder {
                    scope: scope.clone(),
                    requester: requester.to_owned(),
                },
                DeliverSamplesRequest {
                    samples: Vec::new(),
                    gaps: vec![encoded],
                    scope_json: serde_json::to_string(scope).unwrap_or_default(),
                },
            )
            .await;
        }
        if let Err(error) = self.release_stream_leases(leases).await {
            tracing::error!(%error, "refused capture retains its physical stream hold");
        }
    }

    async fn deliver_to_leaseholder(&self, holder: &Holder, request: DeliverSamplesRequest) {
        // A hold that names no run has nobody to be evidence for. A dashboard
        // viewing a stream is exactly that hold, and delivering it would be
        // worse than useless: the sidecar decides which of its executions a
        // scope covers, and a scope naming nothing covers all of them — every
        // live run in that process would take a viewer's rows as its own.
        let Some(execution) = holder.scope.test_execution_id.clone() else {
            return;
        };
        let Some(template) =
            self.state_read().executions.get(&execution).map(|execution| execution.snapshot.case().metadata.identifier.to_string())
        else {
            return;
        };
        let target = crate::domains::execution::run::Target::of(self, &template);
        if let Err(error) = target
            .signal(crate::transport::ExecutionSignal {
                execution_token: execution.to_string(),
                samples: Some(request),
                ..crate::transport::ExecutionSignal::default()
            })
            .await
        {
            tracing::debug!(
                requester = holder.requester,
                detail = %error.message,
                "a leaseholder's run did not take its stream's rows"
            );
        }
    }

    fn stream_owner_and_cadence(&self, stream: &Identifier) -> Result<(StreamOwner, Duration)> {
        let state = self.state_read();
        let contract = state
            .telemetry_definitions
            .get(stream)
            .ok_or_else(|| Error::Operation(format!("telemetry stream {stream} is not published by any installed plugin")))?;
        let interval = contract
            .interval
            .map(|interval| Duration::from_millis(interval.0.max(1)))
            .ok_or_else(|| Error::Operation(format!("telemetry stream {stream} declares no cadence, so nothing samples it to lease")))?;
        let owner = state
            .installed_plugins
            .values()
            .find(|artifact| artifact.telemetry_contracts.iter().any(|declared| &declared.stream == stream))
            .map(|artifact| artifact.source.clone());

        Ok(match owner {
            Some(source) if source == dokime_host::HOST_PLUGIN_SOURCE => (StreamOwner::Host, interval),
            Some(source) => (StreamOwner::Sidecar(source), interval),
            // No artifact claims the contract, which is two different things
            // and they are told apart by whether the contract names a
            // template. One that does is a suite's own stream, sampled inside
            // the execution that declares it and never by the host: leasing it
            // would ask the host's collector for something it has never heard
            // of, so it is refused in words an author can act on. One that does
            // not is a builtin — a host contract in an embedding that never
            // installed the host as an artifact — and the host is its owner.
            None if contract.template_id.is_some() => {
                return Err(Error::Operation(format!(
                    "telemetry stream {stream} is a template's own, sampled inside the execution that declares it, and cannot be leased"
                )));
            }
            None => (StreamOwner::Host, interval),
        })
    }

    async fn start_host_captures(&self, scope: SamplingScope, requester: &str, holds: &[(Identifier, Duration, String)]) -> Result<()> {
        let requests = holds
            .iter()
            .map(|(stream, interval, lease_id)| TelemetrySamplingRequest {
                // The hold's own id, so the demand that comes back can be put
                // on the hold that paid for it.
                request_id: lease_id.clone(),
                source_kind: TelemetrySourceKind::InstalledPlugin,
                source_id: requester.to_owned(),
                stream: stream.to_string(),
                // The owner's, read off the contract when the lease was taken.
                interval: Some(dokime_domain::model::time::DurationMillis(interval.as_millis().min(u128::from(u64::MAX)) as u64)),
                min_interval: None,
                start_after: None,
            })
            .collect::<Vec<_>>();
        let holds = self.stream_leases();
        self.request_host_sampling_bound(scope, requests, |captures| holds.attach_captures(captures)).await?;

        Ok(())
    }

    pub(crate) fn streams_owned_by(&self, source: &str) -> Vec<Identifier> {
        self.state_read()
            .installed_plugins
            .get(source)
            .map(|artifact| artifact.telemetry_contracts.iter().map(|contract| contract.stream.clone()).collect())
            .unwrap_or_default()
    }

    pub(crate) fn stream_leases(&self) -> Arc<StreamLeases> {
        Arc::clone(&self.stream_leases)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn lease_delivers(
    sample: &crate::telemetry::TelemetrySample, holder: &Holder, identities: &[super::streams::admission::SampleIdentity<'_>],
) -> bool {
    identities.binary_search(&(holder.scope.test_execution_id.as_ref(), &sample.sample_id)).is_ok()
        && super::streams::lease_accepts_sample(sample.scope.as_ref(), &holder.scope)
}

#[derive(Default)]
struct LeaseJson(Vec<u8>);
impl std::io::Write for LeaseJson {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.try_reserve(bytes.len()).map_err(std::io::Error::other)?;
        self.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
struct LeaseSample<'a>(&'a crate::telemetry::TelemetrySample);

impl serde::Serialize for LeaseSample<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let sample = self.0;
        let mut map = serializer.serialize_map(Some(10))?;
        map.serialize_entry("sample_id", &sample.sample_id)?;
        map.serialize_entry("stream", &sample.stream)?;
        map.serialize_entry("schema_version", &sample.schema_version)?;
        map.serialize_entry("sequence", &sample.sequence)?;
        map.serialize_entry("timestamp", &sample.timestamp)?;
        map.serialize_entry("received_at", &sample.received_at)?;
        map.serialize_entry("source", &sample.source)?;
        map.serialize_entry("values", &sample.values)?;
        map.serialize_entry("summary", &sample.summary)?;
        map.serialize_entry("metadata", &sample.metadata)?;
        map.end()
    }
}

fn lease_frame<'a>(frame: &'a mut Option<String>, value: &impl serde::Serialize) -> crate::runtime::Result<&'a str> {
    if frame.is_none() {
        *frame = Some(lease_json(value)?);
    }
    Ok(frame.as_deref().expect("the delivered frame was encoded"))
}
fn lease_json(value: &impl serde::Serialize) -> crate::runtime::Result<String> {
    let mut bytes = LeaseJson::default();
    serde_json::to_writer(&mut bytes, value).map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
    String::from_utf8(bytes.0).map_err(|error| crate::runtime::Error::Operation(error.to_string()))
}
fn lease_memory_error() -> crate::runtime::Error {
    crate::runtime::Error::Operation("leased telemetry staging memory overflows".into())
}

fn overlaps(left: &LeaseWindow, right: &LeaseWindow) -> bool {
    let left_ended = left.closed_at.as_ref();
    let right_ended = right.closed_at.as_ref();

    left_ended.is_none_or(|ended| ended >= &right.opened_at) && right_ended.is_none_or(|ended| ended >= &left.opened_at)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::lease_frame;
    use std::cell::Cell;

    struct Counted(Cell<usize>);

    impl serde::Serialize for Counted {
        fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
            self.0.set(self.0.get() + 1);
            serializer.serialize_str("shared row")
        }
    }

    #[test]
    fn leaseholders_reuse_one_json_encoding() {
        let sample = Counted(Cell::new(0));
        let mut frame = None;
        for _ in 0..64 {
            assert_eq!(lease_frame(&mut frame, &sample).unwrap(), "\"shared row\"");
        }
        assert_eq!(sample.0.get(), 1);
    }
}
