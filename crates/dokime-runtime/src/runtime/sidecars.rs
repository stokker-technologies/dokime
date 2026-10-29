/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

pub(crate) mod core;

use std::collections::{BTreeMap, BTreeSet};

use harmos::Message;

use super::BackgroundTask;
use crate::RuntimeHandle;
use crate::records::RecordPublication;
use crate::transport::{DeliverRecordRequest, ExecutionSignal};
use dokime_domain::model::identity::Identifier;
use dokime_domain::model::records::publication::RecordPublishRequest;
use dokime_domain::model::time::Timestamp;

#[derive(Clone)]
pub(crate) struct PublicationIngest {
    streams: crate::app::Streams,
    progress: tokio::sync::watch::Receiver<harmos::storage::Cursor>,
}

#[derive(Default)]
pub(crate) struct RecordDemands {
    sources: BTreeMap<String, RecordDemandSource>,
    listeners: BTreeMap<(String, String, String), harmos::sidecar::DemandLease>,
}

struct RecordDemandSource {
    demands: harmos::sidecar::DemandSource,
    ready: tokio::sync::watch::Receiver<bool>,
    _watch: BackgroundTask,
}

impl PublicationIngest {
    fn frontier(&self) -> harmos::storage::Cursor {
        self.streams.frontier()
    }

    // Wait until ingest has admitted every publication through `target`.
    async fn reached(&self, target: harmos::storage::Cursor) -> crate::runtime::Result<()> {
        let mut progress = self.progress.clone();
        progress
            .wait_for(|cursor| *cursor >= target)
            .await
            .map(|_| ())
            .map_err(|_| crate::runtime::Error::Operation("publication ingest stopped before draining".into()))
    }
}

impl RuntimeHandle {
    pub(crate) fn register_record_demand_source(
        &self, source: String, demands: harmos::sidecar::DemandSource, ready: tokio::sync::watch::Receiver<bool>,
    ) {
        let owned = std::sync::Arc::downgrade(&self.record_demands);
        let state = std::sync::Arc::clone(&self.state);
        let unavailable = std::sync::Arc::clone(&self.unavailable_plugins);
        let routing = std::sync::Arc::clone(&self.routing);
        let mut changed = ready.clone();
        let task = BackgroundTask(tokio::spawn(async move {
            while changed.changed().await.is_ok() {
                let Some(owned) = owned.upgrade() else {
                    break;
                };
                let state = state.read().expect("runtime state lock poisoned");
                let unavailable = unavailable.read().expect("plugin availability lock poisoned");
                let mut demands = owned.lock().expect("record demand lock poisoned");
                if let Err(error) = demands.reconcile(&state.installed_plugins, &unavailable, routing.get()) {
                    tracing::warn!(%error, "could not reconcile listener demand after readiness changed");
                }
            }
        }));
        self.record_demands
            .lock()
            .expect("record demand lock poisoned")
            .sources
            .insert(source, RecordDemandSource { demands, ready, _watch: task });
    }

    pub(crate) fn remove_record_demand_source(&self, source: &str) {
        let mut demands = self.record_demands.lock().expect("record demand lock poisoned");
        demands.sources.remove(source);
        demands.listeners.retain(|(listener, producer, _), _| listener != source && producer != source);
    }

    pub(crate) fn hold_record_demand(&self, records: &[Identifier]) -> crate::runtime::Result<Vec<harmos::sidecar::DemandLease>> {
        let state = self.state_read();
        let artifacts = &state.installed_plugins;
        let unavailable = self.unavailable_plugins.read().expect("plugin availability lock poisoned");
        let demands = self.record_demands.lock().expect("record demand lock poisoned");
        let records = records.iter().collect::<BTreeSet<_>>();
        let mut held = Vec::new();
        for artifact in artifacts.values().filter(|artifact| artifact.enabled && !unavailable.contains(&artifact.source)) {
            let Some(source) = demands.sources.get(&artifact.source) else {
                continue;
            };
            for &record in &records {
                if artifact.record_contracts.iter().any(|contract| &contract.record_type == record) {
                    held.push(
                        source
                            .demands
                            .hold(&record_demand_key(record.as_str()))
                            .map_err(|error| crate::runtime::Error::Operation(error.to_string()))?,
                    );
                }
            }
        }
        Ok(held)
    }

    pub(crate) fn reconcile_listener_record_demand(&self) -> crate::runtime::Result<()> {
        let state = self.state_read();
        let artifacts = &state.installed_plugins;
        let unavailable = self.unavailable_plugins.read().expect("plugin availability lock poisoned");
        let mut demands = self.record_demands.lock().expect("record demand lock poisoned");
        demands.reconcile(artifacts, &unavailable, self.routing.get())
    }

    pub(crate) async fn flush_plugin_publications(&self) -> crate::runtime::Result<()> {
        let ingest = self
            .publication_ingest
            .get()
            .ok_or_else(|| crate::runtime::Error::Operation("publication ingest is not initialized".into()))?;
        flush_publications(ingest).await
    }
}

impl RecordDemands {
    fn reconcile(
        &mut self, artifacts: &super::handle::InstalledPlugins, unavailable: &BTreeSet<String>, router: Option<&harmos::plugin::Router>,
    ) -> crate::runtime::Result<()> {
        let wanted = listener_demands(artifacts, unavailable, router, |owner| self.sources.get(owner).map(|source| *source.ready.borrow()));

        self.listeners.retain(|key, _| wanted.contains(key));
        for key in wanted {
            if !self.listeners.contains_key(&key) {
                let held =
                    self.sources[&key.1].demands.hold(&key.2).map_err(|error| crate::runtime::Error::Operation(error.to_string()))?;
                self.listeners.insert(key, held);
            }
        }
        Ok(())
    }
}

pub(crate) fn spawn_lease_gap_watch(
    handle: RuntimeHandle,
    mut residents: tokio::sync::watch::Receiver<
        std::collections::BTreeMap<harmos::operations::resident::Key, harmos::operations::resident::Snapshot>,
    >,
) -> BackgroundTask {
    let root_scope = harmos::operations::resident::Key::root(String::new()).scope;
    BackgroundTask(tokio::spawn(async move {
        let mut down = std::collections::BTreeSet::<String>::new();
        while residents.changed().await.is_ok() {
            let inactive = residents
                .borrow()
                .iter()
                .filter(|(key, instance)| {
                    key.scope == root_scope && matches!(instance.health, Some(harmos::services::ServiceStatus::Inactive { .. }))
                })
                .map(|(key, _)| key.definition.clone())
                .collect::<std::collections::BTreeSet<_>>();
            for source in inactive.difference(&down) {
                for stream in handle.streams_owned_by(source) {
                    handle.stream_leases().mark_gap(&stream);
                    // Told to the runs as well as written down: a step
                    // listening on the stream and a criterion counting its
                    // rows both need to know the silence was not a
                    // measurement, and neither reads the host's ledger.
                    handle.forward_lease_gap(&stream, "the stream owner stopped while this run held it").await;
                    tracing::warn!(source, stream = stream.as_str(), "a leased stream's owner stopped; every run holding it records a gap");
                }
            }
            down = inactive;
        }
    }))
}

pub(crate) fn spawn_publication_ingest(handle: RuntimeHandle) -> (PublicationIngest, BackgroundTask) {
    let streams = handle.streams.clone();
    let limits = handle.storage().settings().response;
    let mut feed = streams.follow(harmos::storage::Cursor::ORIGIN);
    let (admitted, progress) = tokio::sync::watch::channel(harmos::storage::Cursor::ORIGIN);
    let task = tokio::spawn(async move {
        while let Some(batch) = feed.next().await {
            for definition in batch.incomplete() {
                let Some(owner) = definition.owner() else {
                    continue;
                };
                tracing::warn!(owner, stream = definition.id(), "plugin capture is incomplete");
                if let Ok(stream) = Identifier::new(definition.id()) {
                    super::native_telemetry::incomplete(&handle, owner, &stream, "the native stream or capture window lost rows").await;
                }
            }

            let mut publications = batch.publications().peekable();
            while let Some((definition, cursor, payload)) = publications.next() {
                let mut through = cursor;
                // Preserve global order; only consecutive rows with the same
                // declaration and execution context share an admission.
                let mut rows = vec![(cursor, payload.value)];
                let mut bytes = publication_bytes(payload.value);
                while rows.len() < limits.batch_rows.get()
                    && publications.peek().is_some_and(|(next, _, row)| {
                        let same_stream =
                            next.id() == definition.id() && next.version() == definition.version() && next.owner() == definition.owner();
                        let same_context = row.value.context == payload.value.context;
                        let next_bytes = bytes.checked_add(publication_bytes(row.value));
                        let fits = next_bytes.is_some_and(|bytes| bytes <= limits.batch_bytes.get());
                        same_stream && same_context && fits
                    })
                {
                    let (_, cursor, row) = publications.next().unwrap();
                    through = cursor;
                    bytes += publication_bytes(row.value);
                    rows.push((cursor, row.value));
                }
                super::native_telemetry::ingest_batch(&handle, definition, &rows).await;
                // A fixed flush frontier must not wait for later rows in this
                // capture batch, particularly an unrelated continuous observer.
                admitted.send_replace(through);
            }
            admitted.send_replace(batch.reaches());
        }
    });
    (PublicationIngest { streams, progress }, BackgroundTask(task))
}

pub(super) async fn flush_publications(ingest: &PublicationIngest) -> crate::runtime::Result<()> {
    ingest.reached(ingest.frontier()).await
}

pub(crate) fn spawn_record_forwarding(handle: RuntimeHandle) -> BackgroundTask {
    BackgroundTask(tokio::spawn(async move {
        let mut records = handle.subscribe_records();
        while let Some(entry) = records.next().await {
            // A read that failed is the required capture having failed, which
            // is terminal for the whole process and does not advance the
            // reader's position. Ending here rather than carrying on is the
            // difference between a pump that stops and one that spins: there is
            // nothing left to forward, and the same error would answer forever.
            let Ok(entry) = entry else {
                tracing::warn!("record capture failed; nothing further is forwarded to sidecars");
                return;
            };
            forward(&handle, &entry).await;
        }
    }))
}

pub(crate) fn record_demand_key(record: &str) -> String {
    format!("dokime.record:{record}")
}

pub(super) fn record_publication(artifact_id: &str, publication: RecordPublication) -> Option<(RecordPublishRequest, Timestamp)> {
    let mut request: RecordPublishRequest = match serde_json::from_str(&publication.request_json) {
        Ok(request) => request,
        Err(error) => {
            tracing::warn!(
                artifact_id,
                %error,
                "dropping a record publication whose payload is not a publish request"
            );
            return None;
        }
    };
    request.scope = None;

    // An absent production time means the sidecar did not stamp one; anything
    // else is passed through and validated with the rest of the record.
    let produced_at =
        if publication.produced_at.is_empty() { crate::model::time::now_utc() } else { Timestamp::new(publication.produced_at) };
    Some((request, produced_at))
}

fn publication_bytes(row: &harmos::streams::PublicationRow) -> usize {
    let context_bytes = row.context.as_ref().map_or(0, String::len);
    let payload_bytes = std::mem::size_of_val(row).saturating_add(row.payload.len());
    payload_bytes.saturating_add(context_bytes)
}

fn listener_demands(
    artifacts: &super::handle::InstalledPlugins, unavailable: &BTreeSet<String>, router: Option<&harmos::plugin::Router>,
    source: impl Fn(&str) -> Option<bool>,
) -> BTreeSet<(String, String, String)> {
    let ready = |artifact: &&crate::domains::InstalledPlugin| artifact.enabled && !unavailable.contains(&artifact.source);
    let mut wanted = BTreeSet::new();
    for listener in artifacts.values().filter(ready) {
        if source(&listener.source) != Some(true) {
            continue;
        }
        // A ready plugin must also have admitted every listener operation.
        if !router.is_some_and(|router| {
            router.graph().ordered().iter().any(|definition| {
                definition.id == listener.source
                    && listener
                        .listeners
                        .iter()
                        .all(|declared| definition.contributions.operations.iter().any(|entry| entry.id.as_str() == declared.operation))
            })
        }) {
            continue;
        }
        for producer in artifacts.values().filter(ready) {
            if source(&producer.source).is_none() {
                continue;
            }
            for record in listener.listened_records() {
                if producer.record_contracts.iter().any(|contract| contract.record_type.as_str() == record) {
                    wanted.insert((listener.source.clone(), producer.source.clone(), record_demand_key(record)));
                }
            }
        }
    }
    wanted
}

async fn forward(handle: &RuntimeHandle, record: &dokime_domain::model::records::evidence::RecordEntry) {
    let Ok(record_json) = serde_json::to_string(&record.record) else {
        tracing::warn!(
            record_type = %record.record_type,
            "an admitted record could not be encoded for delivery"
        );
        return;
    };
    let delivery = crate::plugin::Delivery { record_json: record_json.clone() };
    for operation in listeners(handle, record) {
        if let Err(error) = handle.operations.call_encoded(&operation, delivery.encode()).await {
            tracing::debug!(
                %operation,
                %error,
                record_type = %record.record_type,
                "a listener did not take a delivered record"
            );
        }
    }
    let scopes = record.captured_scopes.clone();
    let captured = match handle
        .state_manager
        .state()
        .read(move |state| {
            state
                .executions
                .values()
                .filter_map(|execution| {
                    let scope = scopes.iter().find(|scope| scope.test_execution_id.as_ref() == Some(&execution.test_execution_id))?;
                    Some((
                        execution.snapshot.case().metadata.identifier.to_string(),
                        execution.test_execution_id.to_string(),
                        serde_json::to_string(scope).expect("captured scope serializes"),
                    ))
                })
                .collect::<Vec<_>>()
        })
        .await
    {
        Ok(read) => read.value,
        Err(error) => {
            tracing::warn!(%error, "could not read record subscription recipients");
            return;
        }
    };
    for (template, execution, scope) in captured {
        if handle.execution_sessions.deliver_records(&execution).await {
            continue;
        }
        let target = crate::domains::execution::run::Target::of(handle, &template);
        let signal = ExecutionSignal {
            execution_token: execution,
            record: Some(DeliverRecordRequest {
                ordinal: record.ordinal,
                record_json: record_json.clone(),
                captured_scopes: vec![scope],
            }),
            ..ExecutionSignal::default()
        };
        if let Err(error) = target.signal(signal).await {
            tracing::debug!(
                %template,
                detail = %error.message,
                record_type = %record.record_type,
                "a running case did not take a delivered record"
            );
        }
    }
}

fn listeners(handle: &RuntimeHandle, record: &dokime_domain::model::records::evidence::Record) -> Vec<String> {
    // Forwarding starts before plugin admission; boot records have no targets yet.
    let Some(router) = handle.routing.get() else {
        return Vec::new();
    };
    let graph = router.graph();
    handle
        .state_read()
        .installed_plugins
        .values()
        .filter(|artifact| artifact.enabled && handle.plugin_available(&artifact.source))
        .filter(|artifact| graph.ordered().iter().any(|definition| definition.id == artifact.source))
        .flat_map(|artifact| artifact.listeners.iter())
        .filter(|listener| listener.record == record.record_type.as_str())
        .map(|listener| listener.operation.clone())
        .collect()
}
