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
};

// Third Party Imports

// Local Imports
use crate::runtime::handle::RuntimeHandle;
use crate::telemetry::SamplingScope;
use crate::telemetry_runtime::sampling::record_scope_from_sampling_scope;
use crate::transport::DeliverSamplesRequest;
use crate::{runtime::Error, runtime::Result};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

type StreamDemand = std::pin::Pin<Box<dyn futures_util::Stream<Item = harmos::streams::Item<harmos::streams::PublicationRow>> + Send>>;

/* === Structs === */

pub(crate) struct StreamLeases {
    state: Mutex<LeaseState>,
    // One FIFO Per Leaseholder Run: Admission Queues Rows Without Awaiting Any Plugin Call.
    deliveries: Mutex<BTreeMap<String, tokio::sync::mpsc::UnboundedSender<Delivery>>>,
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
}

struct Instance {
    owner: String,

    // The Publication Watch That Wants This Stream From Its Owner While The Stream Is Held.
    demand: Option<StreamDemand>,

    holders: BTreeSet<String>,
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

#[derive(Clone, Debug)]
struct LeaseWindow {
    pub lease_id: String,
    pub stream: Identifier,
    pub owner: String,
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

// Each Declared Stream Leases On Its Own: A Sampled Stream Gets A Lease, A
// Plugin-Published Stream Needs None, And A Stream That Cannot Be Leased Is
// Refused Without Costing The Others Theirs.
#[derive(Clone, Debug, Default)]
pub struct DeclaredLeases {
    pub leases: Vec<String>,
    pub published: Vec<Identifier>,
    pub refused: Vec<(Identifier, String)>,
}

// A Viewer's Holds On The Streams It Watches; Dropping It Releases Them, So A Closed View
// Stops The Streams Nobody Else Holds.
#[must_use = "the viewed streams run only while their holds live"]
pub struct ViewerLeases {
    table: Arc<StreamLeases>,
    ids: Vec<String>,
}

pub(crate) struct LeasedRows {
    holders: BTreeMap<Identifier, Vec<Holder>>,
}

struct Delivery {
    requester: String,
    // `None` Is A Barrier: It Signals Nothing And Settles After Every Earlier Delivery.
    request: Option<DeliverSamplesRequest>,
    delivered: tokio::sync::oneshot::Sender<()>,
}

/* === Enums === */

// What A Stream Needs Before A Run Reads It.
enum LeaseTarget {
    Sampled { owner: String },
    // Its Owner Publishes It Without A Sampling Declaration Or Observer.
    Published,
}

/* === Traits === */

/* === Implementations === */

impl Drop for PendingLeases {
    fn drop(&mut self) {
        let mut state = self.table.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        for lease_id in &self.ids {
            state.release(lease_id);
        }
    }
}

impl Drop for ViewerLeases {
    fn drop(&mut self) {
        for lease_id in &self.ids {
            self.table.release(lease_id);
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
    fn release(&mut self, lease_id: &str) -> Option<()> {
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
        if last {
            self.instances.remove(&stream);
        }
        Some(())
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
        &self.window.owner
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
            deliveries: Mutex::default(),
        }
    }

    fn take(&self, stream: &Identifier, owner: String, requester: &str, scope: RecordScope) -> (String, bool) {
        let lease_id = format!("lease-{}", uuid::Uuid::now_v7());
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        let entry = state.instances.entry(stream.clone());
        let started = matches!(entry, std::collections::btree_map::Entry::Vacant(_));
        let instance = entry.or_insert_with(|| Instance {
            owner: owner.clone(),
            demand: None,
            holders: BTreeSet::new(),
            generation: 0,
        });
        instance.holders.insert(lease_id.clone());
        state.held.insert(lease_id.clone(), Hold { stream: stream.clone() });
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

    pub(crate) fn release(&self, lease_id: &str) -> Option<()> {
        self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner).release(lease_id)
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
        self.state.lock().unwrap_or_else(|error| error.into_inner()).instances.get(stream).map(|instance| instance.owner.clone())
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
        let record_scope = self.lease_scope(&scope).await?;
        let mut pending = PendingLeases {
            table: self.stream_leases(),
            ids: Vec::with_capacity(streams.len()),
        };
        for stream in streams {
            let LeaseTarget::Sampled { owner } = self.lease_target(stream)? else {
                return Err(Error::Operation(format!(
                    "telemetry stream {stream} declares no sampling or observer, so nothing samples it to lease"
                )));
            };
            let (lease_id, started) = self.stream_leases().take(stream, owner, requester, record_scope.clone());
            if started {
                self.engage_stream_demand(stream);
            }
            pending.ids.push(lease_id);
        }

        Ok(std::mem::take(&mut pending.ids))
    }

    // Leases Every Sampled Stream Of A Declared Set Independently; Releasing
    // `leases` Releases Exactly The Streams That Were Granted.
    pub async fn lease_declared_streams(&self, scope: SamplingScope, streams: &[Identifier], requester: &str) -> Result<DeclaredLeases> {
        let record_scope = self.lease_scope(&scope).await?;
        let mut declared = DeclaredLeases::default();
        for stream in streams {
            match self.lease_target(stream) {
                Ok(LeaseTarget::Sampled { owner }) => {
                    let (lease_id, started) = self.stream_leases().take(stream, owner, requester, record_scope.clone());
                    if started {
                        self.engage_stream_demand(stream);
                    }
                    declared.leases.push(lease_id);
                }
                Ok(LeaseTarget::Published) => declared.published.push(stream.clone()),
                Err(error) => declared.refused.push((stream.clone(), error.to_string())),
            }
        }
        Ok(declared)
    }

    async fn lease_scope(&self, scope: &SamplingScope) -> Result<RecordScope> {
        let record_scope = self.state_read().capture_scope(&record_scope_from_sampling_scope(scope)).map_err(Error::Operation)?;
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
        Ok(record_scope)
    }

    // Holds Every Sampled Stream The Plots Read For As Long As The Viewer Keeps The Holds.
    pub async fn view_plot_streams(&self, requests: &[crate::plot::PlotPayloadRequest], viewer: &str) -> ViewerLeases {
        let requested = requests.iter().flat_map(crate::plot::plot_streams);
        let mut streams = requested.filter_map(|stream| Identifier::new(stream).ok()).collect::<Vec<_>>();
        streams.sort();
        streams.dedup();
        let scope = SamplingScope {
            test_cycle_id: None,
            test_case_id: None,
            test_execution_id: None,
        };
        // A Stream That Cannot Be Leased Is Simply Not Viewed Live; Its Stored Rows Still Plot.
        let declared = self.lease_declared_streams(scope, &streams, viewer).await;
        let ids = declared.map(|declared| declared.leases).unwrap_or_default();
        ViewerLeases { table: self.stream_leases(), ids }
    }

    pub async fn release_stream_leases(&self, leases: Vec<String>) -> Result<()> {
        let stream_leases = self.stream_leases();
        for lease_id in leases {
            stream_leases.release(&lease_id);
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
    // Rows Are Only Queued Here; Each Returned Receipt Settles Once Its Run Took Its Rows.
    pub(crate) async fn queue_leased_rows(
        &self, delivery: harmos::storage::ReadBatch<LeasedRows>, samples: &[super::streams::PreparedSample],
        identities: &[super::streams::admission::SampleIdentity<'_>],
    ) -> Vec<tokio::sync::oneshot::Receiver<()>> {
        let mut queued = Vec::new();
        if delivery.holders.values().all(Vec::is_empty) {
            return queued;
        }
        let mut frames = Vec::new();
        if frames.try_reserve_exact(samples.len()).is_err() {
            tracing::warn!(rows = samples.len(), "leased rows could not reserve their delivery frames");
            return queued;
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
                    for (index, frame) in frames.iter_mut().enumerate().filter(|(index, _)| delivered(&&samples[*index])) {
                        let row = &samples[index];
                        // The Holder Reads The Ordinal Its Own Captured Copy Was Saved Under, As Its Replay Does.
                        let Some(sequence) = holder_ordinal(samples, index, holder) else {
                            return Err(crate::runtime::Error::Operation("a leased row has no captured copy for its leaseholder".into()));
                        };
                        let frame = lease_frame(frame, &LeaseSample(&row.sample))?;
                        let source = row.sample.scope.as_ref();
                        let scope = source
                            .filter(|scope| scope.test_execution_id.is_some() && scope.test_execution_id == holder.scope.test_execution_id)
                            .unwrap_or(&holder.scope);
                        let scope = lease_json(scope)?;
                        let sequence = sequence.to_string();
                        let mut copy = String::new();
                        copy.try_reserve_exact(
                            [frame.len(), scope.len(), sequence.len(), 22]
                                .into_iter()
                                .try_fold(0_usize, usize::checked_add)
                                .ok_or_else(lease_memory_error)?,
                        )
                        .map_err(|_| lease_memory_error())?;
                        copy.push_str(&frame[..frame.len() - 1]);
                        copy.push_str(",\"sequence\":");
                        copy.push_str(&sequence);
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
                    Ok(request) => queued.extend(self.queue_for_leaseholder(holder, request)),
                    Err(error) => {
                        tracing::warn!(stream = stream.as_str(), detail = %error, "a leased row could not be encoded for its leaseholder")
                    }
                }
            }
        }
        queued
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
        if let Some(delivered) = self.queue_for_leaseholder(holder, request) {
            let _ = delivered.await;
        }
    }

    // Queues Behind Every Earlier Delivery To The Same Run Without Awaiting Its Plugin, So A
    // Caller Holding Admission Order Never Waits On A Leaseholder That Itself Waits On Admission.
    fn queue_for_leaseholder(&self, holder: &Holder, request: DeliverSamplesRequest) -> Option<tokio::sync::oneshot::Receiver<()>> {
        // A hold that names no run has nobody to be evidence for. A dashboard
        // viewing a stream is exactly that hold, and delivering it would be
        // worse than useless: the sidecar decides which of its executions a
        // scope covers, and a scope naming nothing covers all of them — every
        // live run in that process would take a viewer's rows as its own.
        let execution = holder.scope.test_execution_id.clone()?;
        let rows = !request.samples.is_empty();
        self.enqueue_delivery(execution, holder.requester.clone(), Some(request), rows)
    }

    // Settles Once Every Delivery Queued Before It Reached Its Run.
    pub(crate) async fn settle_leaseholder_deliveries(&self, execution: &dokime_domain::model::identity::TestExecutionId) {
        if let Some(settled) = self.enqueue_delivery(execution.clone(), String::new(), None, false) {
            let _ = settled.await;
        }
    }

    fn enqueue_delivery(
        &self, execution: dokime_domain::model::identity::TestExecutionId, requester: String, request: Option<DeliverSamplesRequest>,
        rows: bool,
    ) -> Option<tokio::sync::oneshot::Receiver<()>> {
        let (delivered, receipt) = tokio::sync::oneshot::channel();
        let delivery = Delivery { requester, request, delivered };

        // A Live Queue Takes It In Order; Otherwise This Delivery Starts The Run's Queue.
        let leases = self.stream_leases();
        let mut queues = leases.deliveries.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        // Rows Read The Measurement Fence Under The Queue Lock: A Settlement Barrier Queued After
        // The Fence Therefore Follows Every Row That Could Still Reach The Run.
        if rows && self.sampling().measurement_closed_now(execution.as_str()) {
            return None;
        }
        let delivery = match queues.get(execution.as_str()) {
            Some(queue) => match queue.send(delivery) {
                Ok(()) => return Some(receipt),
                Err(closed) => closed.0,
            },
            None => delivery,
        };
        let (queue, pending) = tokio::sync::mpsc::unbounded_channel();
        let _ = queue.send(delivery);
        queues.insert(execution.to_string(), queue);
        drop(queues);
        let runtime = self.clone();
        tokio::spawn(async move { runtime.drain_leaseholder(execution, pending).await });
        Some(receipt)
    }

    // Signals One Run's Queued Deliveries In Order; Retires Its Queue Under The Table Lock Once Empty.
    async fn drain_leaseholder(
        &self, execution: dokime_domain::model::identity::TestExecutionId, mut pending: tokio::sync::mpsc::UnboundedReceiver<Delivery>,
    ) {
        while let Some(delivery) = self.next_delivery(execution.as_str(), &mut pending) {
            if let Some(request) = delivery.request {
                self.signal_leaseholder(&execution, &delivery.requester, request).await;
            }
            let _ = delivery.delivered.send(());
        }
    }

    fn next_delivery(&self, execution: &str, pending: &mut tokio::sync::mpsc::UnboundedReceiver<Delivery>) -> Option<Delivery> {
        if let Ok(delivery) = pending.try_recv() {
            return Some(delivery);
        }
        // Senders Queue Under This Lock, So An Empty Check Here Cannot Strand A Delivery.
        let leases = self.stream_leases();
        let mut queues = leases.deliveries.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let delivery = pending.try_recv().ok();
        if delivery.is_none() {
            queues.remove(execution);
        }
        delivery
    }

    async fn signal_leaseholder(
        &self, execution: &dokime_domain::model::identity::TestExecutionId, requester: &str, request: DeliverSamplesRequest,
    ) {
        let template =
            self.state_read().executions.get(execution).map(|execution| execution.snapshot.case().metadata.identifier.to_string());
        let Some(template) = template else {
            return;
        };
        let target = crate::domains::execution::run::Target::of(self, &template);
        let signal = crate::transport::ExecutionSignal {
            execution_token: execution.to_string(),
            samples: Some(request),
            ..crate::transport::ExecutionSignal::default()
        };
        if let Err(error) = target.signal(signal).await {
            tracing::debug!(requester, detail = %error.message, "a leaseholder's run did not take its stream's rows");
        }
    }

    fn lease_target(&self, stream: &Identifier) -> Result<LeaseTarget> {
        let state = self.state_read();
        let contract = state
            .telemetry_definitions
            .get(stream)
            .ok_or_else(|| Error::Operation(format!("telemetry stream {stream} is not published by any installed plugin")))?;
        let owner = state
            .installed_plugins
            .values()
            .find(|artifact| artifact.telemetry_contracts.iter().any(|declared| &declared.stream == stream))
            .map(|artifact| artifact.source.clone())
            .ok_or_else(|| {
                Error::Operation(format!("telemetry stream {stream} is not owned by an installed plugin and cannot be leased"))
            })?;
        let observed = self.routing.get().is_some_and(|router| {
            router.graph().ordered().iter().any(|provider| {
                provider.id == owner
                    && provider.contributions.observers.iter().any(|observer| observer.streams.iter().any(|id| id == stream.as_str()))
            })
        });
        if contract.interval.is_none() && !observed {
            return Ok(LeaseTarget::Published);
        }
        Ok(LeaseTarget::Sampled { owner })
    }

    // Wants A Held Stream From Its Owner's Current Publication; A Stream Whose Owner Is Not
    // Serving Yet Is Engaged When The Owner Is Exposed.
    fn engage_stream_demand(&self, stream: &Identifier) {
        let version = self.state_read().telemetry_definitions.get(stream).map(|contract| contract.schema_version);
        let Some(publication) = version.and_then(|version| self.streams().publication(stream.as_str(), version)) else {
            return;
        };
        let demand: StreamDemand = Box::pin(publication.watch());
        let table = self.stream_leases();
        let mut state = table.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(instance) = state.instances.get_mut(stream) {
            // The New Watch Is Engaged Before The Previous One Drops, So A Held Stream Never Pauses.
            instance.demand = Some(demand);
        }
    }

    // A Newly Serving Owner Takes Over Demand For Every Stream Of Its That Is Already Held.
    pub(crate) fn engage_owned_demand(&self, owner: &str) {
        for stream in self.streams_owned_by(owner) {
            let table = self.stream_leases();
            let held = table.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner).instances.contains_key(&stream);
            if held {
                self.engage_stream_demand(&stream);
            }
        }
    }

    pub(crate) fn release_owned_demand(&self, owner: &str) {
        let table = self.stream_leases();
        let mut state = table.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        for instance in state.instances.values_mut().filter(|instance| instance.owner == owner) {
            instance.demand.take();
        }
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

// The Ordinal Of The Row Saved For This Holder: The Original Row Or One Of Its Contiguous Lease Copies.
fn holder_ordinal(samples: &[super::streams::PreparedSample], index: usize, holder: &Holder) -> Option<u64> {
    let original = &samples.get(index)?.sample;
    let execution = holder.scope.test_execution_id.as_ref()?;
    let saved_for_holder =
        |row: &&super::streams::PreparedSample| row.scope().and_then(|scope| scope.test_execution_id.as_ref()) == Some(execution);
    let mut copies = samples[index..].iter().take_while(|row| std::sync::Arc::ptr_eq(&row.sample, original));
    copies.find(saved_for_holder).map(|row| row.ordinal)
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
        // The Sequence Is Each Holder's Own Stream Ordinal, Appended Per Holder Like Its Scope.
        let mut map = serializer.serialize_map(Some(9))?;
        map.serialize_entry("sample_id", &sample.sample_id)?;
        map.serialize_entry("stream", &sample.stream)?;
        map.serialize_entry("schema_version", &sample.schema_version)?;
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
