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
use crate::telemetry_runtime::sampling::record_scope_from_sampling_scope;
use crate::{Error, Result};
use dokime::core::{Identifier, RecordScope, Timestamp};
use dokime::telemetry::{
    SamplingScope, TelemetryCaptureLease, TelemetrySamplingRequest, TelemetrySourceKind,
};
use dokime::transport::DeliverSamplesRequest;

/* === Constants === */
pub(crate) const HOST_PRODUCER_KEY: &str = "host";

const MAX_CLOSED_WINDOWS: usize = 4_096;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Default)]
pub(crate) struct StreamLeases {
    state: Mutex<LeaseState>,
}

#[derive(Default)]
struct LeaseState {
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
    #[allow(
        dead_code,
        reason = "the declared cadence the running instance was started at"
    )]
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
    samples: Vec<dokime::telemetry::TelemetrySample>,
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
        let mut state = self
            .table
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for lease_id in &self.ids {
            let attached = state.held.get(lease_id).is_some_and(|hold| {
                state
                    .instances
                    .get(&hold.stream)
                    .is_some_and(|instance| instance.captures.contains_key(lease_id))
            });
            if attached {
                // Physical Capture Remains With Its Original Settlement Owner.
                continue;
            }
            state.release(lease_id);
        }
    }
}

impl LeasedRows {
    pub(crate) fn attribute(&mut self, state: &crate::state::RuntimeState) -> crate::Result<()> {
        for holder in self.holders.values_mut().flatten() {
            holder.scope = state
                .capture_scope(&holder.scope)
                .map_err(crate::Error::Operation)?;
        }
        for sample in &mut self.samples {
            let Some(scope) = &sample.scope else {
                continue;
            };
            let scope = state
                .capture_scope(scope)
                .map_err(crate::Error::Operation)?;
            let Some(execution) = scope
                .test_execution_id
                .as_ref()
                .and_then(|id| state.executions.get(id))
            else {
                continue;
            };
            let Some(plugin) = execution.snapshot.plugin() else {
                continue;
            };
            sample.scope = Some(scope.clone());
            let holders = self.holders.entry(sample.stream.clone()).or_default();
            if !holders.iter().any(|holder| {
                holder.scope.test_execution_id == scope.test_execution_id
                    && holder.requester == plugin.source
            }) {
                holders.push(Holder {
                    scope,
                    requester: plugin.source.clone(),
                });
            }
        }
        Ok(())
    }

    pub(crate) fn capture_scopes(&self, stream: &Identifier) -> Vec<RecordScope> {
        self.holders
            .get(stream)
            .into_iter()
            .flatten()
            .map(|holder| holder.scope.clone())
            .collect()
    }
}

impl LeaseState {
    fn release(&mut self, lease_id: &str) -> Option<Released> {
        let stream = self.held.remove(lease_id)?.stream;
        let closed_at = Timestamp::now_utc();
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
        let mut captures = instance
            .captures
            .remove(lease_id)
            .into_iter()
            .collect::<Vec<_>>();
        if last {
            captures.extend(
                self.instances
                    .remove(&stream)
                    .map(|instance| instance.captures.into_values().collect::<Vec<_>>())
                    .unwrap_or_default(),
            );
        }

        Some(Released { captures })
    }

    fn bound_ledger(&mut self) {
        let closed = self
            .ledger
            .iter()
            .filter(|window| window.closed_at.is_some())
            .count();
        if closed <= MAX_CLOSED_WINDOWS {
            return;
        }
        let mut forget = closed - MAX_CLOSED_WINDOWS;
        self.ledger.retain(|window| {
            if forget > 0 && window.closed_at.is_some() {
                forget -= 1;
                return false;
            }
            true
        });
    }
}

impl StreamLeases {
    fn take(
        &self,
        stream: &Identifier,
        owner: StreamOwner,
        requester: &str,
        interval: Duration,
        scope: RecordScope,
    ) -> (String, bool) {
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
        state.held.insert(
            lease_id.clone(),
            Hold {
                stream: stream.clone(),
                closing: false,
            },
        );
        state.ledger.push(LeaseWindow {
            lease_id: lease_id.clone(),
            stream: stream.clone(),
            owner,
            requester: requester.to_owned(),
            scope,
            opened_at: Timestamp::now_utc(),
            closed_at: None,
            gap: false,
        });
        state.bound_ledger();

        (lease_id, started)
    }

    pub(crate) fn release(&self, lease_id: &str) -> Option<Released> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .release(lease_id)
    }

    pub(crate) fn fence_release(&self, lease_id: &str) -> Option<Vec<String>> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let hold = state.held.get_mut(lease_id)?;
        hold.closing = true;
        let stream = hold.stream.clone();
        let instance = state.instances.get(&stream)?;
        Some(
            instance
                .captures
                .get(lease_id)
                .cloned()
                .into_iter()
                .collect(),
        )
    }

    pub(crate) fn attach_captures(&self, captures: &[TelemetryCaptureLease]) -> Result<()> {
        let assignments = captures
            .iter()
            .map(|capture| {
                let stream = Identifier::new(&capture.request.stream)
                    .map_err(|error| Error::Operation(error.to_string()))?;
                Ok((stream, &capture.request.request_id, &capture.lease_id))
            })
            .collect::<Result<Vec<_>>>()?;
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for (stream, lease_id, _) in &assignments {
            let present = state
                .held
                .get(*lease_id)
                .is_some_and(|hold| !hold.closing && hold.stream == *stream);
            if !present {
                return Err(Error::Operation(
                    "stream hold closed before capture admission".into(),
                ));
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
            .filter(|window| {
                window.closed_at.is_none() && instance.holders.contains(&window.lease_id)
            })
            .map(|window| Holder {
                scope: window.scope.clone(),
                requester: window.requester.clone(),
            })
            .collect()
    }

    fn owner_of(&self, stream: &Identifier) -> Option<String> {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .instances
            .get(stream)
            .map(|instance| match &instance.owner {
                StreamOwner::Host => "host".to_owned(),
                StreamOwner::Sidecar(source) => source.clone(),
            })
    }

    pub(crate) fn is_active(&self, lease_id: &str) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .held
            .contains_key(lease_id)
    }

    pub(crate) fn facts_for(&self, scope: &RecordScope) -> Vec<LeaseFact> {
        let state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        state
            .ledger
            .iter()
            .filter(|window| window.scope.is_visible_in(scope))
            .map(|window| LeaseFact {
                stream: window.stream.clone(),
                owner: match &window.owner {
                    StreamOwner::Host => "host".to_owned(),
                    StreamOwner::Sidecar(source) => source.clone(),
                },
                opened_at: window.opened_at.clone(),
                closed_at: window.closed_at.clone(),
                shared_with: state
                    .ledger
                    .iter()
                    .filter(|other| {
                        other.lease_id != window.lease_id
                            && other.stream == window.stream
                            && overlaps(window, other)
                    })
                    .filter_map(|other| {
                        other
                            .scope
                            .test_execution_id
                            .as_ref()
                            .map(ToString::to_string)
                    })
                    .collect(),
                gap: window.gap,
            })
            .collect()
    }
}

impl RuntimeHandle {
    pub async fn lease_streams(
        &self,
        scope: SamplingScope,
        streams: &[Identifier],
        requester: &str,
    ) -> Result<Vec<String>> {
        let record_scope = self
            .state_read()
            .capture_scope(&record_scope_from_sampling_scope(&scope))
            .map_err(Error::Operation)?;
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
            let (lease_id, _started) = self.stream_leases().take(
                stream,
                owner.clone(),
                requester,
                interval,
                record_scope.clone(),
            );
            pending.ids.push(lease_id.clone());
            match owner {
                StreamOwner::Host => host_holds.push((stream.clone(), interval, lease_id)),
                StreamOwner::Sidecar(_) => {}
            }
        }
        if !host_holds.is_empty() {
            self.start_host_captures(scope, requester, &host_holds)
                .await?;
        }

        Ok(std::mem::take(&mut pending.ids))
    }

    pub async fn release_stream_leases(&self, leases: Vec<String>) -> Result<()> {
        let sampling = self.sampling();
        let stream_leases = self.stream_leases();
        for lease_id in leases {
            sampling
                .release_stream_lease(self, &stream_leases, &lease_id)
                .await?;
        }
        Ok(())
    }

    pub async fn stream_lease_is_active(&self, lease_id: &str) -> bool {
        self.stream_leases().is_active(lease_id)
    }

    pub async fn stream_leases_for(&self, scope: &RecordScope) -> Vec<LeaseFact> {
        self.stream_leases().facts_for(scope)
    }

    pub(crate) async fn leased_rows(
        &self,
        samples: &[dokime::telemetry::TelemetrySample],
    ) -> LeasedRows {
        let mut holders = BTreeMap::new();
        for sample in samples {
            if !holders.contains_key(&sample.stream) {
                holders.insert(
                    sample.stream.clone(),
                    self.stream_leases().holders_of(&sample.stream),
                );
            }
        }
        LeasedRows {
            samples: samples.to_vec(),
            holders,
        }
    }

    pub(crate) async fn forward_leased_rows(&self, delivery: LeasedRows) {
        for (stream, holders) in delivery.holders {
            for holder in holders {
                let samples = delivery
                    .samples
                    .iter()
                    .filter(|sample| sample.stream == stream)
                    .filter(|sample| {
                        super::streams::lease_accepts_sample(sample.scope.as_ref(), &holder.scope)
                    })
                    .map(|sample| {
                        let mut sample = sample.clone();
                        sample.scope = Some(holder.scope.clone());
                        serde_json::to_string(&sample)
                    })
                    .collect::<std::result::Result<Vec<_>, _>>();
                let Ok(samples) = samples else {
                    tracing::warn!(
                        stream = stream.as_str(),
                        "a leased row could not be encoded for its leaseholder"
                    );
                    continue;
                };
                if samples.is_empty() {
                    continue;
                }
                self.deliver_to_leaseholder(
                    &holder,
                    DeliverSamplesRequest {
                        samples,
                        gaps: Vec::new(),
                        scope_json: serde_json::to_string(&holder.scope).unwrap_or_default(),
                    },
                )
                .await;
            }
        }
    }

    pub(crate) async fn forward_lease_gap(&self, stream: &Identifier, reason: &str) {
        let holders = self.stream_leases().holders_of(stream);
        if holders.is_empty() {
            return;
        }
        let owner = self
            .stream_leases()
            .owner_of(stream)
            .unwrap_or_else(|| "host".to_owned());
        let gap = dokime::telemetry::LeaseGap {
            stream: stream.clone(),
            owner,
            observed_at: Timestamp::now_utc(),
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

    pub(crate) async fn refuse_execution_stream(
        &self,
        stream: &Identifier,
        scope: &RecordScope,
        requester: &str,
        reason: &str,
    ) {
        let leases = {
            let table = self.stream_leases();
            let mut state = table
                .state
                .lock()
                .unwrap_or_else(|error| error.into_inner());
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
        let gap = dokime::telemetry::LeaseGap {
            stream: stream.clone(),
            owner: requester.to_owned(),
            observed_at: Timestamp::now_utc(),
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
        let Some(template) = self
            .state_read()
            .executions
            .get(&execution)
            .map(|execution| execution.snapshot.case().metadata.identifier.to_string())
        else {
            return;
        };
        let target = crate::domains::execution::run::Target::of(self.operations.clone(), &template);
        if let Err(error) = target
            .signal(dokime::transport::ExecutionSignal {
                execution_token: execution.to_string(),
                samples: Some(request),
                ..dokime::transport::ExecutionSignal::default()
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
        let contract = crate::runtime_state::contract_lookup(&state)
            .telemetry_contract(stream)
            .ok_or_else(|| {
                Error::Operation(format!(
                    "telemetry stream {stream} is not published by any installed plugin"
                ))
            })?;
        let interval = contract
            .interval
            .map(|interval| Duration::from_millis(interval.0.max(1)))
            .ok_or_else(|| {
                Error::Operation(format!(
                    "telemetry stream {stream} declares no cadence, so nothing samples it to lease"
                ))
            })?;
        let owner = state
            .installed_plugins
            .values()
            .find(|artifact| {
                artifact
                    .telemetry_contracts
                    .iter()
                    .any(|declared| &declared.stream == stream)
            })
            .map(|artifact| artifact.source.clone());

        Ok(match owner {
            Some(source) if source == dokime_host::installed_plugin().source => {
                (StreamOwner::Host, interval)
            }
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

    async fn start_host_captures(
        &self,
        scope: SamplingScope,
        requester: &str,
        holds: &[(Identifier, Duration, String)],
    ) -> Result<()> {
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
                interval: Some(dokime::core::DurationMillis(
                    interval.as_millis().min(u128::from(u64::MAX)) as u64,
                )),
                min_interval: None,
                start_after: None,
            })
            .collect::<Vec<_>>();
        let holds = self.stream_leases();
        self.request_host_sampling_bound(scope, requests, |captures| {
            holds.attach_captures(captures)
        })
        .await?;

        Ok(())
    }

    pub(crate) fn streams_owned_by(&self, source: &str) -> Vec<Identifier> {
        self.state_read()
            .installed_plugins
            .get(source)
            .map(|artifact| {
                artifact
                    .telemetry_contracts
                    .iter()
                    .map(|contract| contract.stream.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub(crate) fn stream_leases(&self) -> Arc<StreamLeases> {
        Arc::clone(&self.stream_leases)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn overlaps(left: &LeaseWindow, right: &LeaseWindow) -> bool {
    let left_ended = left.closed_at.as_ref();
    let right_ended = right.closed_at.as_ref();

    left_ended.is_none_or(|ended| ended >= &right.opened_at)
        && right_ended.is_none_or(|ended| ended >= &left.opened_at)
}

/* === Tests === */
