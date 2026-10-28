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
    sync::Arc,
    time::Duration,
};

// Third Party Imports
use harmos::{ServiceContext, services::Resident};
use tokio::sync::{Mutex, RwLock};
use tokio::time::Instant;

// Local Imports
use crate::app::Dokime;
use crate::runtime::error::{Error, Result};
use crate::runtime::handle::RuntimeHandle;
use crate::telemetry::{SamplingScope, TelemetryBatch, TelemetryCaptureLease, TelemetrySampleKey, TelemetrySamplingRequest};
use crate::telemetry_runtime::sampling::{SamplingPhaseState, record_scope_from_sampling_scope, sampling_phase_state};
use dokime_domain::model::execution::RecordScope;
use dokime_domain::model::execution::health::HealthSeverity;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::time::Timestamp;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

pub type SettlementError = Box<dyn std::error::Error + Send + Sync>;

/* === Structs === */

struct ReceiptDetail<'a> {
    receipt: &'a crate::evidence_runtime::PublishReceipt,
    expected: u64,
}

#[derive(Clone)]
pub(crate) struct Sampling {
    state: Arc<std::sync::Mutex<SamplingState>>,
    publish_gate: Arc<RwLock<()>>,
    fence_changed: Arc<tokio::sync::Notify>,
    idle_grace: Duration,
}

#[derive(Default)]
struct SamplingState {
    producers: BTreeMap<String, ProducerEntry>,
    execution_fences: BTreeMap<TestExecutionId, ExecutionFence>,
}

#[derive(Default)]
struct ExecutionFence {
    permanent: bool,
    scopes: usize,
    closed: usize,
}

pub(crate) struct SamplingQuiescence {
    sampling: Sampling,
    execution_ids: BTreeSet<TestExecutionId>,
    closed: bool,
    released: std::sync::atomic::AtomicBool,
    publish: std::sync::Mutex<Option<tokio::sync::OwnedRwLockWriteGuard<()>>>,
}

struct ProducerEntry {
    admission: Arc<Mutex<()>>,
    registered: bool,
    sampler: Arc<dyn Sampler>,
    leases: BTreeMap<String, CaptureDemand>,
    service: Option<crate::services::Lease>,
    session: Arc<SamplingSession>,
}

#[derive(Clone)]
struct CaptureDemand<T = Instant> {
    scope: RecordScope,
    request: TelemetrySamplingRequest,
    interval: Duration,
    next_due: T,
    paused: bool,
    closing: bool,
}

struct Producer {
    producer_key: String,
    sampling: Sampling,
    runtime: RuntimeHandle,
    session: Arc<SamplingSession>,
}

#[derive(Default)]
struct SamplingOwner {
    source: Option<Box<dyn SamplerSession>>,
    owed: BTreeMap<String, CaptureDemand>,
    returned: Option<CollectedSamples>,
    publication: Option<CapturePublication>,
}

pub(crate) struct CapturedSamples {
    batch: TelemetryBatch,
}

struct CollectedSamples {
    batches: Vec<TelemetryBatch>,
    next: usize,
    accounted: usize,
    received_at: Timestamp,
}

struct CapturePublication {
    lease_id: String,
    task: Option<tokio::task::JoinHandle<std::result::Result<(), ProduceError>>>,
    failure: Option<Arc<ProduceError>>,
}

struct SamplingSession {
    owner: std::sync::Mutex<Option<SamplingOwner>>,
    available: tokio::sync::Notify,
}

struct SamplingBorrow<'a> {
    shared: &'a SamplingSession,
    owner: Option<SamplingOwner>,
}

/* === Enums === */

#[derive(Debug, thiserror::Error)]
enum ProduceError {
    #[error("sampling preferences could not be read: {0}")]
    Preferences(#[from] crate::runtime::Error),
    #[error("sampling source `{producer_key}` failed to open: {message}")]
    SourceOpen { producer_key: String, message: String },

    #[error("sampling source `{producer_key}` failed to read: {message}")]
    SourceRead { producer_key: String, message: String },

    #[error("sampling source `{producer_key}` failed to settle: {source}")]
    SourceSettle {
        producer_key: String,

        #[source]
        source: SettlementError,
    },

    #[error("sampling destination for lease `{lease_id}` rejected a batch: {}", receipt_detail(.receipt, *.expected))]
    Destination {
        lease_id: String,
        expected: u64,
        receipt: harmos::storage::ReadBatch<crate::evidence_runtime::PublishReceipt>,
        #[source]
        source: Option<Box<crate::runtime::Error>>,
    },

    #[error("sampling destination for lease `{lease_id}` refused before row commit: {source}")]
    DestinationRefused {
        lease_id: String,

        #[source]
        source: Box<crate::runtime::Error>,
    },

    #[error("sampling destination for lease `{lease_id}` could not join: {source}")]
    DestinationJoin {
        lease_id: String,

        #[source]
        source: Arc<tokio::task::JoinError>,
    },

    #[error("sampling collection for `{stream}` has no original capture debt")]
    UnownedCollection { stream: String },

    #[error("sampling capture admission failed: {source}")]
    CaptureAdmission {
        #[source]
        source: Arc<ProduceError>,
    },
}

/* === Traits === */

pub trait Sampler: Send + Sync + 'static {
    fn open(&self) -> std::result::Result<Box<dyn SamplerSession>, String>;
}

pub trait SamplerSession: Send {
    fn tick(&mut self, at: Timestamp, streams: &[String]);

    // Register Collection Completion And Timeout Wakes Without Borrowing Across A Wait.
    fn poll_ready(&mut self, context: &mut std::task::Context<'_>) -> std::task::Poll<()>;

    fn collected(&mut self) -> std::result::Result<Vec<TelemetryBatch>, String>;

    fn settle(&mut self) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<(), SettlementError>> + Send + '_>>;

    // Request Cancellation Without Releasing The Original Collection Owner.
    fn abandon(&mut self) {}
}

/* === Implementations === */

impl SamplingQuiescence {
    fn release(&self) {
        if self.released.swap(true, std::sync::atomic::Ordering::AcqRel) {
            return;
        }
        let mut state = self.sampling.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        for execution in &self.execution_ids {
            let Some(fence) = state.execution_fences.get_mut(execution) else {
                continue;
            };
            fence.scopes -= 1;
            fence.closed -= usize::from(self.closed);
            if !fence.permanent && fence.scopes == 0 {
                state.execution_fences.remove(execution);
            }
        }
        drop(state);
        self.publish.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take();
        self.sampling.fence_changed.notify_waiters();
    }
}

impl Drop for SamplingQuiescence {
    fn drop(&mut self) {
        self.release();
    }
}

impl std::fmt::Display for ReceiptDetail<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.receipt.message.as_deref() {
            Some(message) => formatter.write_str(message),
            None => write!(formatter, "capture accepted {} of {} original rows", self.receipt.accepted, self.expected),
        }
    }
}

impl CapturedSamples {
    fn new(scope: &RecordScope, mut batch: TelemetryBatch) -> Self {
        for sample in &mut batch.samples {
            sample.scope = Some(scope.clone());
        }
        Self { batch }
    }

    pub(crate) fn into_batch(self) -> TelemetryBatch {
        self.batch
    }
}

impl SamplingOwner {
    async fn drain(&mut self, runtime: &RuntimeHandle, key: &str) -> std::result::Result<(), ProduceError> {
        self.publish_returned(runtime).await?;
        if let Some(source) = self.source.as_mut() {
            if runtime.is_shutting_down() {
                source.abandon();
            }
            let batches = settle_session(source.as_mut(), key).await?;
            if !batches.is_empty() {
                self.returned = Some(CollectedSamples { batches, next: 0, accounted: 0, received_at: crate::model::time::now_utc() });
                self.publish_returned(runtime).await?;
            }
        }
        // Every Physical Collector Joined And Every Returned Row Was Accounted For.
        // A collector that returned no row leaves no materialized evidence to save.
        self.owed.clear();
        Ok(())
    }

    async fn settle(&mut self, runtime: &RuntimeHandle, key: &str) -> std::result::Result<(), ProduceError> {
        self.drain(runtime, key).await?;
        self.source = None;
        Ok(())
    }

    async fn publish_returned(&mut self, runtime: &RuntimeHandle) -> std::result::Result<(), ProduceError> {
        loop {
            if let Some(publication) = self.publication.as_mut() {
                let stopped = match self.owed.get(&publication.lease_id) {
                    Some(demand) => capture_stopped(runtime, &demand.scope).await,
                    None => false,
                };
                if stopped && publication.failure.is_some() {
                    self.owed.remove(&publication.lease_id);
                    self.returned.as_mut().expect("returned rows remain owned").accounted += 1;
                    self.publication = None;
                    continue;
                }
                if let Some(source) = &publication.failure {
                    return Err(ProduceError::CaptureAdmission { source: Arc::clone(source) });
                }
                let task = publication.task.as_mut().expect("the original write is retained");
                let outcome = match task.await {
                    Ok(outcome) => outcome,
                    Err(source) => Err(ProduceError::DestinationJoin { lease_id: publication.lease_id.clone(), source: Arc::new(source) }),
                };
                publication.task = None;
                if let Err(source) = outcome {
                    let retryable = matches!(source, ProduceError::DestinationRefused { .. });
                    let source = Arc::new(source);
                    if retryable {
                        // Preparation Refused Before Commit; The Original Rows And Debt Remain.
                        self.publication = None;
                    } else {
                        // An Uncertain Join Or Committed Receipt Must Never Be Reissued.
                        publication.failure = Some(Arc::clone(&source));
                    }
                    return Err(ProduceError::CaptureAdmission { source });
                }
                self.owed.remove(&publication.lease_id);
                self.returned.as_mut().expect("the original rows remain owned").accounted += 1;
                self.publication = None;
            }

            let Some(returned) = self.returned.as_mut() else {
                return Ok(());
            };
            let Some(batch) = returned.batches.get(returned.next) else {
                self.returned = None;
                return Ok(());
            };
            let target = self.owed.iter().find(|(_, demand)| demand.request.stream == batch.stream.as_str());
            let Some((lease_id, demand)) = target else {
                if returned.accounted == 0 && !batch.samples.is_empty() {
                    return Err(ProduceError::UnownedCollection { stream: batch.stream.to_string() });
                }
                returned.next += 1;
                returned.accounted = 0;
                continue;
            };
            let lease_id = lease_id.clone();
            let scope = demand.scope.clone();
            let batch = batch.clone();
            let received_at = returned.received_at.clone();
            let runtime = runtime.clone();
            let destination = lease_id.clone();
            let task = tokio::spawn(async move { publish_to_capture(&runtime, &destination, &scope, batch, received_at).await });
            self.publication = Some(CapturePublication { lease_id, task: Some(task), failure: None });
        }
    }
}

impl SamplingSession {
    fn new() -> Self {
        Self { owner: std::sync::Mutex::new(Some(SamplingOwner::default())), available: tokio::sync::Notify::new() }
    }

    async fn borrow(&self) -> SamplingBorrow<'_> {
        loop {
            let available = self.available.notified();
            tokio::pin!(available);
            available.as_mut().enable();
            let owner = self.owner.lock().unwrap_or_else(std::sync::PoisonError::into_inner).take();
            if let Some(owner) = owner {
                return SamplingBorrow { shared: self, owner: Some(owner) };
            }
            available.await;
        }
    }

    async fn ready(&self) {
        let mut available = Box::pin(self.available.notified());
        std::future::poll_fn(|context| {
            loop {
                available.as_mut().enable();
                let ready = {
                    let mut slot = self.owner.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                    slot.as_mut().and_then(|owner| owner.source.as_mut()).is_some_and(|source| source.poll_ready(context).is_ready())
                };
                if ready {
                    return std::task::Poll::Ready(());
                }
                if std::future::Future::poll(available.as_mut(), context).is_pending() {
                    return std::task::Poll::Pending;
                }
                available = Box::pin(self.available.notified());
            }
        })
        .await;
    }
}

impl std::ops::Deref for SamplingBorrow<'_> {
    type Target = SamplingOwner;

    fn deref(&self) -> &Self::Target {
        self.owner.as_ref().expect("the original sampling owner is borrowed")
    }
}

impl std::ops::DerefMut for SamplingBorrow<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.owner.as_mut().expect("the original sampling owner is borrowed")
    }
}

impl Drop for SamplingBorrow<'_> {
    fn drop(&mut self) {
        let Some(owner) = self.owner.take() else {
            return;
        };
        let mut slot = self.shared.owner.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(slot.is_none(), "a physical sampling owner cannot be replaced");
        *slot = Some(owner);
        drop(slot);
        self.shared.available.notify_waiters();
    }
}

impl<T> CaptureDemand<T> {
    fn pause(&mut self) {
        self.paused = true;
    }

    fn resume_at(&mut self, now: T)
    where
        T: std::ops::Add<Duration, Output = T>,
    {
        self.paused = false;
        self.next_due = now + self.interval;
    }
}

impl Sampling {
    pub(crate) fn new() -> Self {
        Self {
            state: Arc::new(std::sync::Mutex::new(SamplingState::default())),
            publish_gate: Arc::new(RwLock::new(())),
            fence_changed: Arc::new(tokio::sync::Notify::new()),
            idle_grace: Duration::from_secs(2),
        }
    }
}

impl Resident<Dokime> for Producer {
    type Error = ProduceError;

    async fn run(&mut self, context: ServiceContext<Dokime>) -> std::result::Result<(), Self::Error> {
        {
            let mut owner = self.session.borrow().await;
            owner.settle(&self.runtime, &self.producer_key).await?;
        }
        let result = self.sampling.produce(context, &self.runtime, &self.producer_key, &self.session).await;
        result
    }

    async fn cleanup(&mut self) -> std::result::Result<(), Self::Error> {
        let mut owner = self.session.borrow().await;
        owner.settle(&self.runtime, &self.producer_key).await
    }
}

impl Sampling {
    async fn request(
        &self, runtime: &RuntimeHandle, scope: SamplingScope, requests: Vec<TelemetrySamplingRequest>, sampler: Arc<dyn Sampler>,
        producer_key: String, bind: impl FnOnce(&[TelemetryCaptureLease]) -> Result<()> + Send,
    ) -> Result<Vec<TelemetryCaptureLease>> {
        if requests.is_empty() {
            return Ok(Vec::new());
        }
        let preferences = runtime.read_application_settings().await?.settings.telemetry;
        let record_scope = record_scope_from_sampling_scope(&scope);
        let admission = {
            let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            let producer = state.producers.entry(producer_key.clone()).or_insert_with(|| ProducerEntry {
                admission: Arc::new(Mutex::new(())),
                registered: false,
                sampler: Arc::clone(&sampler),
                leases: BTreeMap::new(),
                service: None,
                session: Arc::new(SamplingSession::new()),
            });
            Arc::clone(&producer.admission)
        };
        let _admission = admission.lock().await;
        let service_name = producer_service_name(&producer_key);
        let activate = {
            let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            let producer = state.producers.get_mut(&producer_key).expect("producer exists");
            if !producer.registered {
                let key = producer_key.clone();
                let sampling = self.clone();
                let owner = runtime.clone();
                let session = Arc::clone(&producer.session);
                runtime
                    .operations
                    .register_service(
                        harmos::services::ServiceDefinition { id: service_name.clone(), policy: harmos::services::POLICY },
                        move |_: Arc<()>| Producer {
                            producer_key: key.clone(),
                            sampling: sampling.clone(),
                            runtime: owner.clone(),
                            session: Arc::clone(&session),
                        },
                    )
                    .map_err(|error| Error::Operation(error.to_string()))?;
                producer.registered = true;
            }
            producer.service.is_none()
        };
        let service = if activate {
            Some(
                runtime
                    .operations
                    .submit_resident(&service_name)
                    .await
                    .map_err(|error| Error::Operation(format!("failed to activate sampling resident: {error}")))?,
            )
        } else {
            None
        };

        // Admit The Whole Group
        // Until this lock is acquired, no capture demand is visible. Cancelled
        // waits leave generation settlement with the original service manager.
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if record_scope.test_execution_id.as_ref().is_some_and(|execution| state.execution_fences.contains_key(execution)) {
            return Err(Error::Operation("execution sampling is closing".into()));
        }
        let producer = state.producers.get_mut(&producer_key).expect("producer exists");
        if let Some(service) = service {
            producer.service = Some(service);
        }
        if producer.leases.is_empty() {
            producer.sampler = sampler;
        }
        let mut leases = Vec::with_capacity(requests.len());
        for request in requests {
            let lease_id = format!("capture-{}", uuid::Uuid::now_v7());
            producer.leases.insert(
                lease_id.clone(),
                CaptureDemand {
                    scope: record_scope.clone(),
                    interval: sampling_interval(&request, &preferences),
                    request: request.clone(),
                    next_due: Instant::now(),
                    paused: false,
                    closing: false,
                },
            );
            leases.push(TelemetryCaptureLease { lease_id, job_id: service_name.clone(), request, scope: scope.clone() });
        }
        if let Err(error) = bind(&leases) {
            for lease in &leases {
                producer.leases.remove(&lease.lease_id);
            }
            return Err(error);
        }
        self.fence_changed.notify_waiters();
        Ok(leases)
    }

    async fn stop(&self, runtime: &RuntimeHandle, lease_id: &str) -> Result<bool> {
        let target = {
            let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            state.producers.iter_mut().find_map(|(key, producer)| {
                let demand = producer.leases.get_mut(lease_id)?;
                demand.closing = true;
                Some((key.clone(), Arc::clone(&producer.session)))
            })
        };
        let Some((key, session)) = target else {
            return Ok(false);
        };
        self.fence_changed.notify_waiters();
        let mut owner = session.borrow().await;
        owner.drain(runtime, &key).await.map_err(|source| Error::SamplingSettlement(Box::new(source)))?;
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let removed = state.producers.get_mut(&key).is_some_and(|producer| producer.leases.remove(lease_id).is_some());
        self.fence_changed.notify_waiters();
        Ok(removed)
    }

    pub(crate) async fn release_stream_lease(
        &self, runtime: &RuntimeHandle, leases: &crate::domains::telemetry::StreamLeases, lease_id: &str,
    ) -> Result<()> {
        let sessions = {
            let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            let Some(captures) = leases.fence_release(lease_id) else {
                return Ok(());
            };
            let mut sessions = Vec::new();
            for (key, producer) in &mut state.producers {
                let mut affected = false;
                for capture in &captures {
                    if let Some(demand) = producer.leases.get_mut(capture) {
                        demand.closing = true;
                        affected = true;
                    }
                }
                if affected {
                    sessions.push((key.clone(), Arc::clone(&producer.session)));
                }
            }
            sessions
        };
        self.fence_changed.notify_waiters();
        for (key, session) in sessions {
            let mut owner = session.borrow().await;
            owner.drain(runtime, &key).await.map_err(|source| Error::SamplingSettlement(Box::new(source)))?;
        }
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(released) = leases.release(lease_id) {
            for capture in released.captures {
                for producer in state.producers.values_mut() {
                    producer.leases.remove(&capture);
                }
            }
        }
        self.fence_changed.notify_waiters();
        Ok(())
    }

    async fn reconcile(&self, _runtime: &RuntimeHandle) {
        self.fence_changed.notify_waiters();
    }

    pub(crate) async fn contains(&self, lease_id: &str) -> bool {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .producers
            .values()
            .any(|producer| producer.leases.contains_key(lease_id))
    }

    async fn quiesce_executions(&self, runtime: &RuntimeHandle, execution_ids: &BTreeSet<TestExecutionId>) -> Result<SamplingQuiescence> {
        // Fence Tick Admission Before Borrowing Any Physical Owner.
        let mut quiescence = SamplingQuiescence {
            sampling: self.clone(),
            execution_ids: execution_ids.clone(),
            closed: false,
            released: std::sync::atomic::AtomicBool::new(false),
            publish: std::sync::Mutex::new(None),
        };
        let sessions = {
            let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            for execution in execution_ids {
                state.execution_fences.entry(execution.clone()).or_default().scopes += 1;
            }
            let mut sessions = Vec::new();
            for (key, producer) in &mut state.producers {
                let affected = producer
                    .leases
                    .values()
                    .any(|demand| demand.scope.test_execution_id.as_ref().is_some_and(|id| execution_ids.contains(id)));
                if affected {
                    sessions.push((key.clone(), Arc::clone(&producer.session)));
                }
            }
            sessions
        };
        // Drain The Same Original Collectors And Their Actual Write Receipts.
        self.fence_changed.notify_waiters();
        for (key, session) in sessions {
            let mut owner = session.borrow().await;
            owner.drain(runtime, &key).await.map_err(|source| Error::SamplingSettlement(Box::new(source)))?;
        }
        // Capture Closure Begins Only After Materialized Rows Were Accounted For.
        let publish = Arc::clone(&self.publish_gate).write_owned().await;
        {
            let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            for execution in execution_ids {
                state.execution_fences.get_mut(execution).expect("the quiescence owns its fence").closed += 1;
            }
            quiescence.closed = true;
        }
        *quiescence.publish.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(publish);
        self.fence_changed.notify_waiters();
        Ok(quiescence)
    }

    async fn release_quiescence(&self, quiescence: &SamplingQuiescence) {
        quiescence.release();
    }

    pub(crate) async fn fence_executions(&self, ids: &[TestExecutionId]) {
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        for id in ids {
            state.execution_fences.entry(id.clone()).or_default().permanent = true;
        }
        drop(state);
        self.fence_changed.notify_waiters();
    }

    pub(crate) async fn measurement_closed(&self, execution: &str) -> bool {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .execution_fences
            .iter()
            .any(|(id, fence)| id.as_str() == execution && (fence.permanent || fence.closed > 0))
    }

    pub(crate) async fn wait_measurement_closed(&self, execution: &str) {
        loop {
            let changed = self.fence_changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            if self.measurement_closed(execution).await {
                return;
            }
            changed.await;
        }
    }

    pub(crate) async fn begin_publish(&self) -> tokio::sync::RwLockReadGuard<'_, ()> {
        self.publish_gate.read().await
    }

    pub(crate) async fn closing_executions(&self) -> BTreeSet<TestExecutionId> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .execution_fences
            .iter()
            .filter(|(_, fence)| fence.permanent || fence.closed > 0)
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub(crate) async fn retain_open_samples(&self, batch: &mut crate::telemetry::TelemetryBatch) -> u64 {
        let state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let before = batch.samples.len();
        batch.samples.retain(|sample| {
            sample
                .scope
                .as_ref()
                .and_then(|scope| scope.test_execution_id.as_ref())
                .is_none_or(|id| state.execution_fences.get(id).is_none_or(|fence| !fence.permanent && fence.closed == 0))
        });
        (before - batch.samples.len()) as u64
    }

    async fn snapshot(&self, key: &str) -> Option<(Arc<dyn Sampler>, Vec<(String, CaptureDemand)>)> {
        let state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        state.producers.get(key).map(|producer| {
            let leases = producer
                .leases
                .iter()
                .map(|(id, demand)| {
                    let mut demand = demand.clone();
                    demand.closing |= demand.scope.test_execution_id.as_ref().is_some_and(|id| state.execution_fences.contains_key(id));
                    (id.clone(), demand)
                })
                .collect();
            (producer.sampler.clone(), leases)
        })
    }

    async fn remove_stopped(&self, key: &str, leases: &[String]) {
        if leases.is_empty() {
            return;
        }
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(producer) = state.producers.get_mut(key) {
            for lease in leases {
                if let Some(demand) = producer.leases.get_mut(lease) {
                    demand.closing = true;
                }
            }
        }
    }

    async fn admit_tick(
        &self, key: &str, runnable: &[(String, CaptureDemand)], source: &mut dyn SamplerSession,
        owed: &mut BTreeMap<String, CaptureDemand>, preferences: &dokime_domain::model::settings::application::TelemetryPreferences,
    ) -> Option<Instant> {
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let SamplingState { producers, execution_fences } = &mut *state;
        let producer = producers.get_mut(key)?;
        let open = |demand: &CaptureDemand| {
            !demand.closing && demand.scope.test_execution_id.as_ref().is_none_or(|id| !execution_fences.contains_key(id))
        };
        let now = Instant::now();
        let due = producer
            .leases
            .iter()
            .filter(|(lease_id, demand)| open(demand) && demand.next_due <= now && runnable.iter().any(|(id, _)| id == *lease_id))
            .map(|(lease_id, _)| lease_id.clone())
            .collect::<Vec<_>>();
        let mut streams = Vec::new();
        for lease_id in due {
            let demand = producer.leases.get_mut(&lease_id).expect("the due capture is retained");
            streams.push(demand.request.stream.clone());
            owed.entry(lease_id).or_insert_with(|| demand.clone());
            demand.interval = sampling_interval(&demand.request, preferences);
            demand.next_due += demand.interval;
        }
        streams.sort();
        streams.dedup();
        // Tick Is Nonblocking; The Short Registry Borrow Linearizes It With Fencing.
        if !streams.is_empty() {
            source.tick(crate::model::time::now_utc(), &streams);
        }
        producer
            .leases
            .iter()
            .filter(|(lease_id, demand)| open(demand) && runnable.iter().any(|(id, _)| id == *lease_id))
            .map(|(_, demand)| demand.next_due)
            .min()
    }

    async fn update_phase_transitions(&self, key: &str, paused: &[String], resumed: &[String], now: Instant) {
        if paused.is_empty() && resumed.is_empty() {
            return;
        }
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(producer) = state.producers.get_mut(key) else {
            return;
        };
        for lease_id in paused {
            if let Some(demand) = producer.leases.get_mut(lease_id) {
                demand.pause();
            }
        }
        for lease_id in resumed {
            if let Some(demand) = producer.leases.get_mut(lease_id) {
                demand.resume_at(now);
            }
        }
    }

    async fn produce(
        &self, mut context: ServiceContext<Dokime>, runtime: &RuntimeHandle, key: &str, session: &SamplingSession,
    ) -> std::result::Result<(), ProduceError> {
        let mut idle_since: Option<Instant> = None;
        let mut state_changes = runtime.state_watch.subscribe();
        while !context.is_stopping() {
            let changed = self.fence_changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            let next_due = self.advance(&mut context, runtime, key, session, &mut idle_since).await?;
            let waiting = async {
                tokio::select! {
                    () = &mut changed => true,
                    result = state_changes.changed() => result.is_ok(),
                    () = session.ready() => true,
                    () = async {
                        match next_due {
                            Some(deadline) => tokio::time::sleep_until(deadline).await,
                            None => std::future::pending().await,
                        }
                    } => true,
                }
            };
            if context.until_stopping(waiting).await != Some(true) {
                break;
            }
        }
        Ok(())
    }

    async fn advance(
        &self, context: &mut ServiceContext<Dokime>, runtime: &RuntimeHandle, key: &str, session: &SamplingSession,
        idle_since: &mut Option<Instant>,
    ) -> std::result::Result<Option<Instant>, ProduceError> {
        let mut owner = session.borrow().await;
        owner.publish_returned(runtime).await?;
        if let Some(source) = owner.source.as_mut() {
            let batches = collect_samples(key, source.as_mut())?;
            if !batches.is_empty() {
                owner.returned = Some(CollectedSamples { batches, next: 0, accounted: 0, received_at: crate::model::time::now_utc() });
                owner.publish_returned(runtime).await?;
            }
        }
        let Some((sampler, leases)) = self.snapshot(key).await else {
            owner.settle(runtime, key).await?;
            return Ok(None);
        };
        let mut runnable = Vec::new();
        let mut paused = Vec::new();
        let mut resumed = Vec::new();
        let mut stopped = Vec::new();
        let phase_now = Instant::now();
        let state = runtime.state_read();
        for (lease_id, mut demand) in leases {
            let phase = sampling_phase_state(&state, &demand.scope, demand.request.start_after.as_deref());
            if phase == SamplingPhaseState::Stopped {
                stopped.push(lease_id);
                continue;
            }
            if demand.closing {
                continue;
            }
            match phase {
                SamplingPhaseState::Running if demand.paused => {
                    resumed.push(lease_id.clone());
                    demand.paused = false;
                    demand.next_due = phase_now + demand.interval;
                    runnable.push((lease_id, demand));
                }
                SamplingPhaseState::Running => runnable.push((lease_id, demand)),
                SamplingPhaseState::Paused if !demand.paused => paused.push(lease_id),
                SamplingPhaseState::Paused => {}
                SamplingPhaseState::Stopped => unreachable!("stopped leases handled above"),
            }
        }
        self.update_phase_transitions(key, &paused, &resumed, phase_now).await;
        self.remove_stopped(key, &stopped).await;
        if !stopped.is_empty() {
            // Keep Each Closing Demand Until Its Original Tick Debt Is Drained.
            owner.drain(runtime, key).await?;
            let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(producer) = state.producers.get_mut(key) {
                for lease in &stopped {
                    producer.leases.remove(lease);
                }
            }
        }

        if runnable.is_empty() {
            let since = idle_since.get_or_insert_with(Instant::now);
            if since.elapsed() >= self.idle_grace {
                owner.settle(runtime, key).await?;
                return Ok(None);
            }
            return Ok(Some(*since + self.idle_grace));
        }
        *idle_since = None;

        let SamplingOwner { source: session, owed, .. } = &mut *owner;
        if session.is_none() {
            *session = Some(open_sampler(key, sampler.as_ref())?);
            context.ready();
        }

        let session = session.as_mut().expect("sampling session is open");
        let preferences = runtime.read_application_settings().await?.settings.telemetry;
        let next_due = self.admit_tick(key, &runnable, session.as_mut(), owed, &preferences).await;

        let batches = collect_samples(key, session.as_mut())?;
        if !batches.is_empty() {
            owner.returned = Some(CollectedSamples { batches, next: 0, accounted: 0, received_at: crate::model::time::now_utc() });
            owner.publish_returned(runtime).await?;
        }
        Ok(next_due)
    }
}

impl RuntimeHandle {
    pub async fn request_sampling<S>(
        &self, scope: SamplingScope, request: TelemetrySamplingRequest, sampler: S,
    ) -> Result<TelemetryCaptureLease>
    where
        S: Sampler,
    {
        let producer_key = producer_key(&request.sample_key())?;
        let mut leases = self.sampling().request(self, scope, vec![request], Arc::new(sampler), producer_key, |_| Ok(())).await?;
        Ok(leases.pop().expect("one capture was requested"))
    }

    pub(crate) async fn request_grouped_sampling(
        &self, scope: SamplingScope, requests: Vec<TelemetrySamplingRequest>, sampler: Arc<dyn Sampler>, producer_key: String,
        bind: impl FnOnce(&[TelemetryCaptureLease]) -> Result<()> + Send,
    ) -> Result<Vec<TelemetryCaptureLease>> {
        self.sampling().request(self, scope, requests, sampler, producer_key, bind).await
    }

    pub async fn stop_telemetry_capture(&self, lease_id: &str) -> Result<bool> {
        self.sampling().stop(self, lease_id).await
    }

    pub async fn telemetry_capture_is_active(&self, lease_id: &str) -> bool {
        self.sampling().contains(lease_id).await
    }

    pub(crate) async fn reconcile_sampling(&self) {
        self.sampling().reconcile(self).await;
    }

    pub(crate) async fn quiesce_execution_sampling(&self, execution_ids: &BTreeSet<TestExecutionId>) -> Result<SamplingQuiescence> {
        self.sampling().quiesce_executions(self, execution_ids).await
    }

    pub(crate) async fn resume_sampling(&self, quiescence: &SamplingQuiescence) {
        self.sampling().release_quiescence(quiescence).await;
        self.reconcile_sampling().await;
    }

    pub(crate) async fn complete_sampling(&self, quiescence: &SamplingQuiescence) {
        self.sampling().release_quiescence(quiescence).await;
        self.reconcile_sampling().await;
    }

    pub(crate) fn sampling(&self) -> Arc<Sampling> {
        Arc::clone(&self.sampling)
    }
}

/* === Public Functions === */

/* === Private Functions === */

async fn settle_session(session: &mut dyn SamplerSession, key: &str) -> std::result::Result<Vec<TelemetryBatch>, ProduceError> {
    session.settle().await.map_err(|source| ProduceError::SourceSettle { producer_key: key.to_owned(), source })?;
    collect_samples(key, session)
}

fn open_sampler(key: &str, sampler: &dyn Sampler) -> std::result::Result<Box<dyn SamplerSession>, ProduceError> {
    sampler.open().map_err(|error| {
        tracing::warn!(producer = key, %error, "sampling source open failed");
        ProduceError::SourceOpen { producer_key: key.to_owned(), message: error }
    })
}

fn collect_samples(key: &str, session: &mut dyn SamplerSession) -> std::result::Result<Vec<TelemetryBatch>, ProduceError> {
    session.collected().map_err(|error| {
        tracing::warn!(producer = key, %error, "sampling source read failed");
        ProduceError::SourceRead { producer_key: key.to_owned(), message: error }
    })
}

fn sampling_interval(
    request: &TelemetrySamplingRequest, settings: &dokime_domain::model::settings::application::TelemetryPreferences,
) -> Duration {
    let requested =
        request.interval.map(|interval| Duration::from_millis(interval.0)).unwrap_or(Duration::from_millis(settings.interval_ms));
    let minimum = request.min_interval.map(|interval| Duration::from_millis(interval.0)).unwrap_or(Duration::ZERO);
    requested.max(minimum).max(Duration::from_millis(1))
}

fn producer_key(sample_key: &TelemetrySampleKey) -> Result<String> {
    serde_json::to_string(sample_key).map_err(|error| Error::Operation(format!("failed to encode telemetry sample key: {error}")))
}

fn producer_service_name(producer_key: &str) -> String {
    let id = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_OID, producer_key.as_bytes());
    format!("dokime.telemetry.{id}")
}

async fn publish_to_capture(
    runtime: &RuntimeHandle, lease_id: &str, scope: &RecordScope, batch: TelemetryBatch, received_at: Timestamp,
) -> std::result::Result<(), ProduceError> {
    let expected = batch.samples.len() as u64;
    let stream = batch.stream.clone();
    if capture_stopped(runtime, scope).await {
        return Ok(());
    }
    let completion = CapturedSamples::new(scope, batch);
    let publication = match runtime.publish_captured_telemetry(completion, received_at).await {
        Ok(publication) => publication,
        Err(_) if capture_stopped(runtime, scope).await => return Ok(()),
        Err(source) if transient_capture_failure(&source) => {
            let message = format!("telemetry collection dropped during a transient storage outage: {source}");
            record_destination_failure(runtime, lease_id, scope, &message).await;
            runtime.forward_lease_gap(&stream, &message).await;
            return Ok(());
        }
        Err(source) => {
            return Err(ProduceError::DestinationRefused { lease_id: lease_id.to_owned(), source: Box::new(source) });
        }
    };
    if let Some(source) = &publication.failure
        && transient_capture_failure(source)
    {
        let message =
            format!("telemetry collection partially accepted during a transient storage outage; rejected remainder dropped: {source}");
        record_destination_failure(runtime, lease_id, scope, &message).await;
        runtime.forward_lease_gap(&stream, &message).await;
        return Ok(());
    }
    if let Some(source) = &publication.failure {
        tracing::warn!(lease = lease_id, error = ?source, "telemetry capture was partially accepted");
    }
    let receipt = publication.receipt;
    // Explicitly Discarded Stop Buffers Are Accounted For, Not Destination Failures.
    if receipt.accepted.checked_add(receipt.dropped) != Some(expected) || (receipt.dropped != 0 && !capture_stopped(runtime, scope).await) {
        match &publication.failure {
            Some(source) => record_destination_failure(runtime, lease_id, scope, source).await,
            None => record_destination_failure(runtime, lease_id, scope, &receipt_detail(&receipt, expected)).await,
        }
        return Err(ProduceError::Destination {
            lease_id: lease_id.to_owned(),
            expected,
            receipt,
            source: publication.failure.map(Box::new),
        });
    }
    Ok(())
}

fn transient_capture_failure(error: &crate::runtime::Error) -> bool {
    fn runtime(error: &harmos::Error) -> bool {
        match error {
            harmos::Error::Backend { kind: harmos::BackendFailure::Transient, .. } => true,
            harmos::Error::Persistence { source } => runtime(source),
            _ => false,
        }
    }
    match error {
        crate::runtime::Error::Harmos(source) | crate::runtime::Error::Storage(crate::storage::Error::Runtime(source)) => runtime(source),
        _ => false,
    }
}

async fn capture_stopped(runtime: &RuntimeHandle, scope: &RecordScope) -> bool {
    let Some(id) = scope.test_execution_id.as_ref() else {
        return false;
    };
    runtime.sampling().closing_executions().await.contains(id) || runtime.state_read().report_finalizations.contains_key(id)
}

fn receipt_detail(receipt: &crate::evidence_runtime::PublishReceipt, expected: u64) -> ReceiptDetail<'_> {
    ReceiptDetail { receipt, expected }
}

async fn record_destination_failure(
    runtime: &RuntimeHandle, lease_id: &str, scope: &RecordScope, message: &(impl std::fmt::Display + ?Sized),
) {
    if let Err(error) = runtime
        .add_health_item(
            scope.clone(),
            HealthSeverity::Error,
            "Telemetry capture destination failed.".to_owned(),
            Some(format!("lease {lease_id}: {message}")),
        )
        .await
    {
        tracing::error!(lease = lease_id, %error, "failed to record capture destination health");
    }
}

/* === Tests === */

#[cfg(test)]
mod destination_tests {
    use super::*;

    #[test]
    fn quiescence_preserves_owners() {
        let sampling = Sampling::new();
        let execution = TestExecutionId::from_seed("quiescent");
        let rollback = TestExecutionId::from_seed("partial");
        {
            let mut state = sampling.state.lock().unwrap();
            state.execution_fences.insert(execution.clone(), ExecutionFence { permanent: false, scopes: 2, closed: 1 });
            state.execution_fences.insert(rollback.clone(), ExecutionFence { permanent: false, scopes: 1, closed: 0 });
        }
        let partial = SamplingQuiescence {
            sampling: sampling.clone(),
            execution_ids: BTreeSet::from([execution.clone(), rollback.clone()]),
            closed: false,
            released: std::sync::atomic::AtomicBool::new(false),
            publish: std::sync::Mutex::new(None),
        };
        let closed = SamplingQuiescence {
            sampling: sampling.clone(),
            execution_ids: BTreeSet::from([execution.clone()]),
            closed: true,
            released: std::sync::atomic::AtomicBool::new(false),
            publish: std::sync::Mutex::new(None),
        };

        drop(partial);
        {
            let mut state = sampling.state.lock().unwrap();
            assert!(!state.execution_fences.contains_key(&rollback));
            let fence = state.execution_fences.get_mut(&execution).unwrap();
            assert_eq!((fence.scopes, fence.closed), (1, 1));
            fence.permanent = true;
        }
        closed.release();
        drop(closed);
        let state = sampling.state.lock().unwrap();
        let fence = &state.execution_fences[&execution];
        assert!(fence.permanent);
        assert_eq!((fence.scopes, fence.closed), (0, 0));
    }

    #[test]
    fn only_transient_storage_failures_allow_fresh_collection() {
        let transient = harmos::Error::Persistence {
            source: Arc::new(harmos::Error::backend(
                "storage",
                harmos::BackendFailure::Transient,
                std::io::Error::other("connection dropped"),
            )),
        };
        assert!(transient_capture_failure(&crate::runtime::Error::Storage(crate::storage::Error::Runtime(transient))));
        let permanent = harmos::Error::backend("storage", harmos::BackendFailure::Corrupt, std::io::Error::other("invalid stored payload"));
        assert!(!transient_capture_failure(&crate::runtime::Error::Harmos(permanent)));
        assert!(!transient_capture_failure(&crate::runtime::Error::Harmos(harmos::Error::InvalidInput {
            field: "telemetry schema",
            reason: "incompatible payload",
        })));
    }
}
