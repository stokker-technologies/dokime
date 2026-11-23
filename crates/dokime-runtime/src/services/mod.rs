/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

mod access_credentials;
mod clock;
pub(crate) mod composition;
#[cfg(test)]
mod record_fanout;
mod root_claim;
mod state;
mod status;
#[cfg(test)]
mod stop_waits;

/* === Re-Exports === */

pub use root_claim::detach_credentials;

pub use clock::WallClock;
pub use dokime_operations::host::ClientSettings;

/* === Imports === */

// Standard Library Imports
use std::fs;
use std::fs::OpenOptions;
use std::fs::TryLockError;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{RwLock, Weak};
use std::time::Duration;
use std::{collections::BTreeSet, net::SocketAddr, path::Path, sync::Arc};

// Third Party Imports
use harmos_operations::plugins::Watch;
use harmos_protocol::host::plugin_change::Kind;
use harmos_protocol::host::{PluginsWatchRequest, plugins_watch_response::Event};
use harmos_runtime::grpc::Attached;
use harmos_runtime::grpc::middleware::{Enter, Invocation, Middleware, Observation};
use harmos_runtime::operations::OperationManager;
use harmos_runtime::operations::StreamSettings;
use harmos_runtime::services::status::Registration;
use harmos_runtime::services::status::Status;
use harmos_runtime::{Refusal, Runtime};
use prost_types::Timestamp;
use tokio::sync::Mutex;
use tokio::sync::watch::{self, Sender};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

// Local Imports
use crate::{Error, Host, HostSettings, Result};
use composition::Application;
use dokime_operations::host;
use dokime_operations::host::Context;
use dokime_operations::host::HostIdentity;
use dokime_operations::host::content;
use dokime_operations::host::credentials::CredentialNamespace;
use dokime_operations::host::cycles::seal;
use dokime_operations::host::executions::commands;
use dokime_operations::host::files::captures::Captures;
use dokime_operations::native::Binding;
use dokime_protocol::host::HostBuild;
use dokime_storage::{BackendLease, Store};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(crate) struct ClientWatches {
    active: Sender<BTreeSet<u64>>,
    next: AtomicU64,
}

/// The Transport Call Identity of One Open Client Watch.
struct WatchCall(u64);

impl Default for ClientWatches {
    fn default() -> Self {
        Self {
            active: watch::channel(BTreeSet::new()).0,
            next: AtomicU64::new(0),
        }
    }
}
impl ClientWatches {
    pub async fn settle(&self) -> Result<()> {
        let mut active = self.active.subscribe();
        while !active.borrow_and_update().is_empty() {
            active.changed().await.map_err(|source| Error::Invalid(format!("Settle Client Watches: {source}")))?;
        }
        Ok(())
    }
}
// A Watch Settles When Its Transport Sends the Final Status, Not When Its Handler Returns, So
// Detaching the Listener Never Cuts a Finished Watch's Last Frames.
impl Middleware for ClientWatches {
    fn enter<'a>(&'a self, call: &'a mut Invocation) -> Enter<'a> {
        Box::pin(async move {
            let path = call.path();
            if path.starts_with("/dokime.host.v1.") && path.ends_with("/Watch") {
                let watch = self.next.fetch_add(1, Ordering::Relaxed);
                call.context_mut().insert(WatchCall(watch))?;
                self.active.send_modify(|active| {
                    active.insert(watch);
                });
            }
            Ok(())
        })
    }
    fn observe(&self, call: &Invocation, _: Observation) {
        if let Some(watch) = call.context().get::<WatchCall>() {
            self.active.send_modify(|active| {
                active.remove(&watch.0);
            });
        }
    }
}

pub struct Server {
    endpoint: SocketAddr,
    host: Host,
    listener: Option<Attached>,
    status: Arc<Status>,
    status_task: Option<JoinHandle<Result<()>>>,
    plugin_task: Option<JoinHandle<Result<()>>>,
    plugin_observation: CancellationToken,
    _status_registration: Arc<Registration>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Server {
    pub fn endpoint(&self) -> SocketAddr {
        self.endpoint
    }
    pub fn status(&self) -> &Arc<Status> {
        &self.status
    }
    pub async fn close(mut self) -> Result<()> {
        self.plugin_observation.cancel();
        if let Some(task) = self.plugin_task.take() {
            task.await??;
        }
        self.host.executions.shutdown().await?;
        // Every Product Watch Ends on Shutdown; a Client That Stops Reading Is Cut When the Listener Detaches.
        let assembly = self.host.assembly.as_ref().expect("assembled host");
        match tokio::time::timeout(assembly.close_timeout, assembly.client_watches.settle()).await {
            Ok(settled) => settled?,
            Err(_) => tracing::warn!(
                timeout = ?assembly.close_timeout,
                "Settle Client Watches: A Client Stopped Reading; Detaching the Listener Ends Its Watch"
            ),
        }
        if let Some(listener) = self.listener.take() {
            listener.detach().await?;
        }
        self.host.shutdown().await?;
        if let Some(task) = self.status_task.take() {
            task.await??;
        }
        Ok(())
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.plugin_observation.cancel();
        if let Some(task) = self.plugin_task.take() {
            task.abort();
        }
        if let Some(task) = self.status_task.take() {
            task.abort();
        }
    }
}

/* === Public Functions === */

/// Remove Protected Values Before Explicitly Discarding a Storage Root.
/// The Host Must Be Stopped; Evidence and the Root Itself Are Retained. Only Entries Inside `namespace`, the
/// Namespace the Root's Host Ran With, Are Removed.
pub fn remove_credentials(root: &Path, namespace: &CredentialNamespace) -> Result<()> {
    if !root.join("backend-id").is_file() {
        return Ok(());
    }
    let lease = OpenOptions::new().create(true).truncate(false).read(true).write(true).open(root.join("local.lock"))?;
    match lease.try_lock() {
        Ok(()) => {}
        Err(TryLockError::WouldBlock) => return Err(Error::OccupiedRoot(root.to_owned())),
        Err(TryLockError::Error(error)) => return Err(error.into()),
    }
    let host_id = fs::read_to_string(root.join("backend-id"))?;
    let storage = Store::open(root)?;
    // A Copied Folder Carries the Identity and Catalogs, Never the Native Owner Claim.
    root_claim::require_owner(root, &storage, namespace)?;
    host::remove_root_secrets(&storage, namespace)?;
    access_credentials::Store::new(storage, host_id.clone(), namespace).remove()?;
    root_claim::release(root, &host_id, namespace)?;
    // Unlock Before Closing, So the Root Is Free When This Call Returns.
    lease.unlock()?;
    Ok(())
}

pub async fn serve(endpoint: SocketAddr, root: &Path, host_settings: HostSettings, settings: ClientSettings) -> Result<Server> {
    if !endpoint.ip().is_loopback() {
        return Err(Error::Invalid("Use the Authenticated Remote Listener for Non-Loopback Access".into()));
    }
    if settings.read_timeout.is_zero()
        || settings.mutation_timeout.is_zero()
        || settings.watch_timeout.is_zero()
        || settings.max_cases == 0
        || settings.page_items == 0
        || settings.stream_buffer == 0
        || settings.message_bytes < 1024
    {
        return Err(Error::Invalid("Serve with Positive Deadlines and Bounds, and at Least 1024 Message Bytes".into()));
    }
    let host = Host::open(root, host_settings, settings.clone()).await?;
    let assembly = host.assembly.as_ref().expect("assembled host");
    let runtime = assembly.runtime.clone();
    let context = assembly.context.clone();
    let status = assembly.status.clone();
    let registration = assembly.registration.clone();
    let backend_id = host.store.identity().to_owned();
    let mut changes = context.host.changes();
    let weak_host = Arc::downgrade(&context.host);
    let plugin_context = Arc::downgrade(&context);
    runtime.grpc().register_middleware(assembly.client_watches.clone())?;
    runtime
        .grpc()
        .configure_access(
            composition::access_settings(&settings),
            Arc::new(access_credentials::Store::new(host.store.clone(), backend_id, &host.settings.credentials)),
        )
        .await?;
    let mut transport = composition::transport(&settings);
    transport.address = endpoint;
    let listener = runtime.grpc().attach(transport).await?;
    let endpoint = listener.local_addr();
    let plugin_observation = CancellationToken::new();
    let plugin_task = tokio::spawn(observe_plugins(
        runtime.operations.clone(),
        plugin_context,
        settings.clone(),
        plugin_observation.clone(),
        status.clone(),
    ));
    let changed_registration = registration.clone();
    let status_task = tokio::spawn(async move {
        while changes.changed().await.is_ok() {
            changed_registration.changed().map_err(|source| status::failure("Refresh Host Status", source))?;
            if weak_host.upgrade().is_none_or(|host| host.shutting_down()) {
                break;
            }
        }
        Ok(())
    });
    Ok(Server {
        endpoint,
        host,
        listener: Some(listener),
        status,
        status_task: Some(status_task),
        plugin_task: Some(plugin_task),
        plugin_observation,
        _status_registration: registration,
    })
}

/* === Private Functions === */

pub(crate) struct Assembly {
    pub runtime: Arc<Runtime<Application>>,
    pub context: Arc<Context>,
    pub status: Arc<Status>,
    pub registration: Arc<Registration>,
    pub close_timeout: Duration,
    pub backend_lease: BackendLease,
    pub client_watches: Arc<ClientWatches>,
}

pub(crate) async fn assemble(host: &Host, root: &Path, settings: &ClientSettings) -> Result<Assembly> {
    root_claim::claim(root, &host.store, &host.settings.credentials)?;
    let backend_id = host.store.identity().to_owned();
    let identity = HostIdentity {
        backend_id: backend_id.clone(),
        epoch: uuid::Uuid::now_v7().to_string(),
        build: HostBuild {
            source_revision: env!("DOKIME_SOURCE_REVISION").into(),
            channel: env!("DOKIME_BUILD_CHANNEL").into(),
        },
        target: dokime_packaging::bundle::native_target().into(),
        started_at: Some(Timestamp {
            seconds: host.now_ns().div_euclid(1_000_000_000),
            nanos: host.now_ns().rem_euclid(1_000_000_000) as i32,
        }),
        display_name: "Dokime Execution Host".into(),
    };
    let binding = Arc::new(Binding::default());
    let client_watches = Arc::new(ClientWatches::default());
    let (runtime, backend_lease, plugins) = Box::pin(composition::launch(host, root, settings, binding.clone())).await?;
    let status = runtime.services().configure_status(status::settings(settings, backend_id.clone()))?;
    let administration = Arc::new(Mutex::new(()));
    let catalogs = Arc::new(RwLock::new(Vec::new()));
    let state = state::State {
        status: status.clone(),
        root: root.to_owned(),
        host: host.clone(),
        operations: runtime.operations.clone(),
        catalogs: catalogs.clone(),
        identity,
    };
    let context = Arc::new(Context {
        administration,
        reports: Arc::default(),
        archives: Arc::default(),
        host: Arc::new(state),
        settings: settings.clone(),
        operations: runtime.operations.clone(),
        caller: None,
        captures: Arc::new(Captures::new(host.settings.capture.clone())),
    });
    host.executions.bind_context(&context)?;
    host.executions
        .inner
        .runtime
        .set(Arc::downgrade(&runtime))
        .map_err(|_| Error::Invalid("Bind Execution Runtime: Assemble Once".into()))?;
    host.executions.inner.library_plugins.set(plugins).map_err(|_| Error::Invalid("Bind Execution Plugins: Assemble Once".into()))?;
    // Retire Dead Attempts Before Recovery Reads Can Retain Their Native Bytes.
    seal::recover(&context).await?;
    let now = host.now_ns();
    commands::recover(
        &context,
        Timestamp {
            seconds: now.div_euclid(1_000_000_000),
            nanos: now.rem_euclid(1_000_000_000) as i32,
        },
    )
    .await?;
    binding.bind(&context)?;
    content::validate_root(&context).await?;
    context.host.refresh_plugin_states().await?;
    host::register_redactions(&context)?;
    let registration = Arc::new(
        status
            .contribute(Arc::new(status::Facts {
                host: Arc::downgrade(&context.host),
                plugins: catalogs,
            }))
            .map_err(|source| status::failure("Contribute Host Status", source))?,
    );
    Ok(Assembly {
        runtime,
        context,
        status,
        registration,
        close_timeout: settings.mutation_timeout,
        backend_lease,
        client_watches,
    })
}

async fn observe_plugins(
    operations: OperationManager, context: Weak<Context>, settings: ClientSettings, cancellation: CancellationToken, status: Arc<Status>,
) -> Result<()> {
    loop {
        let mut watch = operations.download_protobuf::<Watch>(
            PluginsWatchRequest { name: None, at: None },
            StreamSettings {
                items: NonZeroUsize::new(settings.stream_buffer).unwrap(),
                bytes: NonZeroUsize::new(settings.message_bytes).unwrap(),
            },
        )?;
        loop {
            let event = tokio::select! {
                biased;
                _ = cancellation.cancelled() => {
                    watch.request_cancel();
                    match watch.join().await {
                        Ok(_) | Err(harmos_runtime::Error::OperationRefused(Refusal::Cancelled { .. })) => {}
                        Err(source) => return Err(source.into()),
                    }
                    return Ok(());
                }
                event = watch.next() => event?,
            };
            let Some(event) = event else {
                return Err(Error::Invalid("Observe Plugins: Reopen the Closed Plugin Watch".into()));
            };
            let Some(context) = context.upgrade() else {
                watch.request_cancel();
                match watch.join().await {
                    Ok(_) | Err(harmos_runtime::Error::OperationRefused(Refusal::Cancelled { .. })) => {}
                    Err(source) => return Err(source.into()),
                }
                return Ok(());
            };
            if let Some(Event::Change(change)) = &event.event {
                let fields = match &change.kind {
                    Some(Kind::Entry(entry)) => [("plugin".into(), entry.name.clone()), ("state".into(), entry.state.to_string())].into(),
                    Some(Kind::Deleted(name)) => [("plugin".into(), name.clone()), ("deleted".into(), true.to_string())].into(),
                    None => return Err(Error::Invalid("Observe Plugins: Supply Its Change".into())),
                };
                let observation = harmos_protocol::host::status::Event {
                    target: "dokime.plugins".into(),
                    level: "INFO".into(),
                    fields,
                    ..Default::default()
                };
                status.record_log(observation.clone()).map_err(|source| status::failure("Record Plugin Log", source))?;
                status.record_trace(observation).map_err(|source| status::failure("Record Plugin Trace", source))?;
            }
            context.host.refresh_plugin_states().await?;
            context.host.signal_change();
            if matches!(event.event, Some(Event::ResyncRequired(_))) {
                watch.request_cancel();
                match watch.join().await {
                    Ok(_) | Err(harmos_runtime::Error::OperationRefused(Refusal::Cancelled { .. })) => {}
                    Err(source) => return Err(source.into()),
                }
                break;
            }
        }
    }
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use std::{env, env::consts::EXE_SUFFIX, error::Error, num::NonZeroUsize, sync::Arc, thread, time::Duration};

    use harmos_operations::storage::{
        AppendFileBytes, AppendFileBytesRequest, CompoundOutcome, CreateFileProduction, CreateFileProductionRequest, CreateFileWriter,
        CreateFileWriterRequest, DeleteScope, DeleteScopeRequest, Info, InfoRequest, ListFileProductions, ListFileProductionsRequest,
    };
    use harmos_protocol::host::PluginState;
    use harmos_protocol::model::{Declarations, PageRequest, StorageTransferSettings, StreamDeclaration};
    use harmos_protocol::policy::UNBOUNDED;
    use harmos_runtime::Identifier;
    use harmos_runtime::storage::{FileMetadata, MutationId};
    use harmos_runtime::streams::Item;
    use prost::Message;
    use prost_types::Timestamp;
    use tokio_stream::StreamExt;
    use tonic::transport::Endpoint;

    use super::{ClientSettings, WallClock, serve};
    use crate::{CaptureSettings, ControlledClock, Host, HostSettings, PlacementSettings, PluginProcess};
    use dokime_client_internal::Client;
    use dokime_operations::host::executions::commands;
    use dokime_operations::host::plots::snapshots;
    use dokime_operations::host::reports;
    use dokime_operations::host::storage::administration;
    use dokime_protocol::host::executions::{ExecutionCommandAction, ExecutionCommandPhase, ExecutionCommandReceipt};
    use dokime_protocol::host::plots::{PlotRequest, SnapshotsWatchRequest};
    use dokime_protocol::host::{CycleSummary, ExecutionOutcome, ReportSubject, ReportsCreateRequest, report_subject};
    use dokime_protocol::model::{EvidenceScope, Import, Origin, PluginRun, Termination, origin};
    use dokime_storage::persisted::Scope;
    use dokime_storage::storage_defaults;
    use dokime_storage::{Execution, Sample};
    use dokime_test_support::credentials;

    #[derive(dokime::Row)]
    #[dokime(package = "fixture.live")]
    struct Reading {
        value: u64,
    }

    #[test]
    fn live_plot_admission() -> Result<(), Box<dyn Error>> {
        credentials::run(Box::pin(async {
            // Start a Host With a Controlled Execution Clock and No Sampling Plugin.
            let root = credentials::tempdir()?;
            let settings = HostSettings {
                installation: None,
                placement: PlacementSettings {
                    reconnect_delay: Duration::from_secs(1),
                    stream_window: NonZeroUsize::new(8).unwrap(),
                    interval_window: crate::INTERVAL_WINDOW,
                    interval_lag: crate::INTERVAL_LAG,
                    message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
                },
                storage: storage_defaults(),
                clock: Arc::new(ControlledClock::new(1_000_000_000)),
                capture: CaptureSettings::default(),
                credentials: credentials::namespace(),
            };
            let operations = ClientSettings {
                read_timeout: crate::PLUGINS_TIMEOUT,
                mutation_timeout: crate::PLUGINS_TIMEOUT,
                watch_timeout: UNBOUNDED,
                max_cases: 1000,
                page_items: 50,
                message_bytes: settings.placement.message_bytes.get(),
                stream_buffer: settings.placement.stream_window.get(),
            };
            let host = Host::open(root.path(), settings, operations).await?;
            host.store.create(&Execution {
                identifier: "execution".into(),
                cycle: "cycle".into(),
                origin: Some(Origin {
                    kind: Some(origin::Kind::Import(Import::default())),
                }),
                catalog: Declarations {
                    types: Some(<Reading as dokime::encoding::contract::Message>::types()),
                    streams: vec![StreamDeclaration {
                        id: "readings".into(),
                        row: <Reading as dokime::encoding::contract::Message>::name(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }
                .encode_to_vec(),
                ..Default::default()
            })?;
            host.store.open_stream("execution", "readings")?;
            let context = &host.assembly.as_ref().unwrap().context;
            let request = PlotRequest {
                plot_id: Some("readings".into()),
                stream: "readings".into(),
                y_keys: vec!["value".into()],
                scope: Some(EvidenceScope {
                    cycle_id: Some("cycle".into()),
                    execution_id: Some("execution".into()),
                    ..Default::default()
                }),
                window_s: Some(60.0),
                ..Default::default()
            };
            let mut updates = snapshots::watch::handle(
                context,
                SnapshotsWatchRequest {
                    requests: vec![request],
                    ..Default::default()
                },
            )
            .await?;
            assert_eq!(updates.next().await.unwrap()?.updates[0].envelope.as_ref().unwrap().revision, 0);

            // Admit Outside a Tokio Runtime: No Flush Worker or Timer Can Make These Rows Durable.
            let store = host.store.clone();
            thread::spawn(move || {
                store.sample_batch(
                    "execution",
                    &(0..128)
                        .map(|index| Sample {
                            stream: "readings".into(),
                            source_ns: index * 1_000_000,
                            row: Reading { value: index as u64 }.encode_to_vec(),
                            ..Default::default()
                        })
                        .collect::<Vec<_>>(),
                )
            })
            .join()
            .unwrap()?;
            let update = updates.next().await.unwrap()?;
            let envelope = update.updates[0].envelope.as_ref().unwrap();
            assert_eq!(envelope.revision, 128);
            assert_eq!(envelope.payload.as_ref().unwrap().series[0].raw_points, 128);
            assert_eq!(host.store.metadata("execution")?.frontier, 0);
            drop(updates);
            host.shutdown().await?;
            Ok(())
        }))
    }

    #[test]
    fn shutdown_cancels_an_in_flight_freeze_and_the_next_download_creates_the_report() -> Result<(), Box<dyn Error>> {
        credentials::run(Box::pin(async {
            // Seal a Run Before Its Report Is Frozen
            let root = credentials::tempdir()?;
            let settings = HostSettings {
                installation: None,
                placement: PlacementSettings {
                    reconnect_delay: Duration::from_secs(1),
                    stream_window: NonZeroUsize::new(8).unwrap(),
                    interval_window: crate::INTERVAL_WINDOW,
                    interval_lag: crate::INTERVAL_LAG,
                    message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
                },
                storage: storage_defaults(),
                clock: Arc::new(ControlledClock::new(1_000_000_000)),
                capture: CaptureSettings::default(),
                credentials: credentials::namespace(),
            };
            let operations = ClientSettings {
                read_timeout: crate::PLUGINS_TIMEOUT,
                mutation_timeout: crate::PLUGINS_TIMEOUT,
                watch_timeout: UNBOUNDED,
                max_cases: 1000,
                page_items: 50,
                message_bytes: settings.placement.message_bytes.get(),
                stream_buffer: settings.placement.stream_window.get(),
            };
            let host = Host::open(root.path(), settings.clone(), operations.clone()).await?;
            host.store.create(&Execution {
                identifier: "execution".into(),
                cycle: "cycle".into(),
                origin: Some(Origin {
                    kind: Some(origin::Kind::PluginRun(PluginRun {
                        plugin: "plugin".into(),
                        template: "template".into(),
                        digest: vec![1],
                        started_at: Some(Timestamp::default()),
                        test_case: Some(Default::default()),
                    })),
                }),
                ..Default::default()
            })?;
            host.store.close("execution", Termination::Completed, ExecutionOutcome::default())?;
            host.store.seal("execution", Default::default())?;

            // Shut Down While Report Admission Holds the Seal's Freeze in Flight
            let context = host.assembly.as_ref().unwrap().context.clone();
            let admitted = context.reports.lock().await;
            let settled = host.executions.admitted_work()?;
            let freezing = tokio::spawn({
                let context = context.clone();
                async move { reports::create::freeze(&context, "execution").await.map(|_| ()) }
            });
            while host.executions.admitted_work()? == settled {
                tokio::task::yield_now().await;
            }
            host.shutdown().await?;
            assert!(matches!(freezing.await?, Err(dokime_operations::Error::Cancelled(_))));
            assert!(host.store.report_identifiers("execution")?.is_empty());
            drop(admitted);
            drop((context, host));

            // The Next Download Creates the Report Once
            let host = Host::open(root.path(), settings, operations).await?;
            let download = ReportsCreateRequest {
                subject: Some(ReportSubject {
                    kind: Some(report_subject::Kind::Execution("execution".into())),
                }),
                request_id: "download".into(),
                ..Default::default()
            };
            let created = reports::create::handle(&host.assembly.as_ref().unwrap().context, download).await?;
            assert_eq!(host.store.report_identifiers("execution")?, vec![created.report.unwrap().identifier]);
            host.shutdown().await?;
            Ok(())
        }))
    }

    #[test]
    fn seal_upload_recovery_discovers_lost_responses_and_settles_exact_scope_pages() -> Result<(), Box<dyn Error>> {
        credentials::run(Box::pin(async {
            // Open a Host With One-Production Recovery Pages.
            let root = credentials::tempdir()?;
            let installation = credentials::tempdir()?;
            let settings = HostSettings {
                installation: Some(installation.path().to_path_buf()),
                placement: PlacementSettings {
                    reconnect_delay: Duration::from_secs(1),
                    stream_window: NonZeroUsize::new(8).unwrap(),
                    interval_window: crate::INTERVAL_WINDOW,
                    interval_lag: crate::INTERVAL_LAG,
                    message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
                },
                storage: storage_defaults(),
                clock: Arc::new(ControlledClock::new(1_000_000_000)),
                capture: CaptureSettings::default(),
                credentials: credentials::namespace(),
            };
            let operations = ClientSettings {
                read_timeout: crate::PLUGINS_TIMEOUT,
                mutation_timeout: crate::PLUGINS_TIMEOUT,
                watch_timeout: UNBOUNDED,
                max_cases: 1000,
                page_items: 1,
                message_bytes: settings.placement.message_bytes.get(),
                stream_buffer: settings.placement.stream_window.get(),
            };
            let host = Host::open(root.path(), settings.clone(), operations.clone()).await?;
            let context = &host.assembly.as_ref().unwrap().context;
            let window = context.storage::<Info>(InfoRequest {}).await?.mutations.unwrap().id;
            let mutation = |key: &str| -> Result<MutationId, Box<dyn Error>> {
                Ok(MutationId {
                    window: window.clone(),
                    key: Identifier::new(key)?,
                })
            };
            let scope = Scope::CycleSeal("cycle").identifier()?;

            // Lose Every Create Response, Leaving Only Native Production Ownership.
            for id in ["upload-a", "upload-b", "upload-c"] {
                context
                    .storage::<CreateFileProduction>(CreateFileProductionRequest {
                        metadata: Some(FileMetadata {
                            id: Identifier::new(id)?,
                            scope: Some(scope.clone()),
                            media_type: "application/octet-stream".into(),
                            expected_length: None,
                            expected_sha256: None,
                        }),
                        mutation: Some(mutation(id)?),
                    })
                    .await?;
            }

            // Rediscover One Writer, Fence Its Generation and Save Accepted Bytes.
            let first = context
                .storage::<ListFileProductions>(ListFileProductionsRequest {
                    scope: scope.clone(),
                    page: Some(PageRequest { page_size: 1, ..Default::default() }),
                    at: None,
                })
                .await?;
            assert_eq!(first.productions.len(), 1);
            assert!(!first.next_page_token.is_empty());
            let original = first.productions[0].session.clone();
            let resumed = context
                .storage::<CreateFileWriter>(CreateFileWriterRequest {
                    session: Some(original.clone()),
                    mutation: Some(mutation("resume")?),
                })
                .await?
                .production
                .unwrap();
            assert!(resumed.session.generation > original.generation);
            context
                .storage::<AppendFileBytes>(AppendFileBytesRequest {
                    session: Some(resumed.session),
                    offset: 0,
                    data: b"accepted".to_vec(),
                    mutation: Some(mutation("append")?),
                })
                .await?;

            // Keep a Prefix-Related Scope and a Completed Cycle Independent.
            let retained_bytes = b"retained";
            let mut retained = None;
            for (id, owner) in [("foreign", "cycle/other"), ("completed", "completed")] {
                let created = context
                    .storage::<CreateFileProduction>(CreateFileProductionRequest {
                        metadata: Some(FileMetadata {
                            id: Identifier::new(id)?,
                            scope: Some(Scope::CycleSeal(owner).identifier()?),
                            media_type: "application/octet-stream".into(),
                            expected_length: None,
                            expected_sha256: None,
                        }),
                        mutation: Some(mutation(id)?),
                    })
                    .await?
                    .production
                    .unwrap();
                if id == "foreign" {
                    context
                        .storage::<AppendFileBytes>(AppendFileBytesRequest {
                            session: Some(created.session.clone()),
                            offset: 0,
                            data: retained_bytes.to_vec(),
                            mutation: Some(mutation("retain-bytes")?),
                        })
                        .await?;
                    retained = context
                        .storage::<CreateFileWriter>(CreateFileWriterRequest {
                            session: Some(created.session),
                            mutation: Some(mutation("retain-writer")?),
                        })
                        .await?
                        .production
                        .map(|production| production.session);
                }
            }
            let retained = retained.unwrap();
            host.store.save_cycle(&CycleSummary {
                cycle_id: "cycle".into(),
                ..Default::default()
            })?;
            host.store.save_cycle(&CycleSummary {
                cycle_id: "completed".into(),
                sealed: true,
                ..Default::default()
            })?;
            host.shutdown().await?;
            drop(host);

            // Startup Settles Every Discovered Page Before Exact-Scope Deletion.
            let host = Host::open(root.path(), settings, operations).await?;
            let context = &host.assembly.as_ref().unwrap().context;
            let page = context
                .storage::<ListFileProductions>(ListFileProductionsRequest {
                    scope: scope.clone(),
                    page: None,
                    at: None,
                })
                .await?;
            assert!(page.productions.is_empty());
            assert!(page.next_page_token.is_empty());
            let window = context.storage::<Info>(InfoRequest {}).await?.mutations.unwrap().id;
            let deleted = context
                .storage::<DeleteScope>(DeleteScopeRequest {
                    scope,
                    mutation: Some(MutationId {
                        window,
                        key: Identifier::new("delete-settled")?,
                    }),
                })
                .await?;
            assert!(matches!(deleted.outcome, CompoundOutcome::Applied { .. }));
            for owner in ["cycle/other", "completed"] {
                let page = context
                    .storage::<ListFileProductions>(ListFileProductionsRequest {
                        scope: Scope::CycleSeal(owner).identifier()?,
                        page: None,
                        at: None,
                    })
                    .await?;
                assert_eq!(page.productions.len(), 1);
                if owner == "cycle/other" {
                    let production = &page.productions[0];
                    assert_eq!(production.session, retained);
                    assert_eq!(production.accepted_length, retained_bytes.len() as u64);
                    assert_eq!(production.durable_length, retained_bytes.len() as u64);
                }
            }
            host.shutdown().await?;
            Ok(())
        }))
    }

    #[test]
    fn scoped_commands_recover_in_bounded_pages_and_replay_admission() -> Result<(), Box<dyn Error>> {
        credentials::run(Box::pin(async {
            // Open a Native Command Owner With One-Row Pages.
            let root = credentials::tempdir()?;
            let installation = credentials::tempdir()?;
            let settings = HostSettings {
                installation: Some(installation.path().to_path_buf()),
                placement: PlacementSettings {
                    reconnect_delay: Duration::from_secs(1),
                    stream_window: NonZeroUsize::new(8).unwrap(),
                    interval_window: crate::INTERVAL_WINDOW,
                    interval_lag: crate::INTERVAL_LAG,
                    message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
                },
                storage: storage_defaults(),
                clock: Arc::new(ControlledClock::new(1_000_000_000)),
                capture: CaptureSettings::default(),
                credentials: credentials::namespace(),
            };
            let operations = ClientSettings {
                read_timeout: crate::PLUGINS_TIMEOUT,
                mutation_timeout: crate::PLUGINS_TIMEOUT,
                watch_timeout: UNBOUNDED,
                max_cases: 2,
                page_items: 1,
                message_bytes: settings.placement.message_bytes.get(),
                stream_buffer: settings.placement.stream_window.get(),
            };
            let host = Host::open(root.path(), settings.clone(), operations.clone()).await?;
            let context = &host.assembly.as_ref().unwrap().context;

            // Preserve Admission Bytes and Cuts While Advancing Every Command.
            let phases = [
                ExecutionCommandPhase::Accepted,
                ExecutionCommandPhase::Dispatching,
                ExecutionCommandPhase::Running,
                ExecutionCommandPhase::Completed,
                ExecutionCommandPhase::Interrupted,
            ];
            let mut admissions = Vec::new();
            for (index, phase) in phases.into_iter().enumerate() {
                let command = ExecutionCommandReceipt {
                    request_id: format!("request-{index}"),
                    execution_id: format!("execution-{index}"),
                    command_id: "command".into(),
                    action: ExecutionCommandAction::Next as i32,
                    phase: ExecutionCommandPhase::Accepted as i32,
                    accepted_at: Some(Timestamp { seconds: 1, nanos: 0 }),
                    ..Default::default()
                };
                let receipt = commands::save(context, &command).await?;
                let (held, at) = commands::read(context, &command.execution_id, &command.command_id, None).await?;
                assert_eq!(held, Some(command.clone()));
                let mut advanced = command.clone();
                advanced.phase = phase as i32;
                commands::save(context, &advanced).await?;
                let (held, _) = commands::read(context, &command.execution_id, &command.command_id, Some(at)).await?;
                assert_eq!(held, Some(command.clone()));
                assert_eq!(commands::receipt(context, &command.execution_id, &command.command_id).await?.durable_at, receipt.durable_at);
                admissions.push((command, phase, receipt));
            }

            // Restart and Recover All Pages Without Replacing Completed Commands.
            host.shutdown().await?;
            drop(host);
            let host = Host::open(root.path(), settings.clone(), operations.clone()).await?;
            let context = &host.assembly.as_ref().unwrap().context;
            for (command, phase, admission) in &admissions {
                let (recovered, _) = commands::read(context, &command.execution_id, &command.command_id, None).await?;
                let recovered = recovered.unwrap();
                let expected = match phase {
                    ExecutionCommandPhase::Accepted | ExecutionCommandPhase::Dispatching | ExecutionCommandPhase::Running => {
                        ExecutionCommandPhase::Interrupted
                    }
                    phase => *phase,
                };
                assert_eq!(recovered.phase, expected as i32);
                assert_eq!(recovered.accepted_at, command.accepted_at);
                let replay = commands::receipt(context, &command.execution_id, &command.command_id).await?;
                assert_eq!(replay.durable_at, admission.durable_at);
                assert_eq!(replay.accepted_at, admission.accepted_at);
                assert!(replay.completed);
                assert_eq!(replay.request_id, command.request_id);
            }

            // Refuse the Superseded Envelope Without Removing Its Root or Bytes.
            let legacy = vec![8, 1, 18, 2, 255, 0];
            let key = "execution-command/legacy";
            administration::replace(context, key, Scope::Execution("legacy").identifier()?, Vec::new(), "legacy", legacy.clone()).await?;
            let refused = commands::recover(context, Timestamp { seconds: 2, nanos: 0 }).await;
            assert!(matches!(refused, Err(dokime_operations::Error::Precondition(_))));
            assert_eq!(administration::read(context, key, &[], None).await?.payload, legacy);
            host.shutdown().await?;
            drop(host);
            assert!(Host::open(root.path(), settings, operations).await.is_err());
            assert!(root.path().join("storage.sqlite3").is_file());
            Ok(())
        }))
    }

    #[test]
    fn host_plugin_launcher_stays_ready_while_collections_publish() -> Result<(), Box<dyn Error>> {
        credentials::run(Box::pin(async {
            tokio::time::timeout(crate::PLUGINS_TIMEOUT, async {
                // Start the Shipped Plugin With the Product Launcher on Fresh Storage.
                let root = credentials::tempdir()?;
                let installation = credentials::tempdir()?;
                let executable = env::current_exe()?.parent().unwrap().parent().unwrap().join(format!("plugin-host{EXE_SUFFIX}"));
                if !executable.is_file() {
                    return Err("Build the Shipped Plugin With mise run build -- --package plugin-host".into());
                }
                let message_bytes = NonZeroUsize::new(4 * 1024 * 1024).unwrap();
                let stream_window = NonZeroUsize::new(8).unwrap();
                let settings = HostSettings {
                    installation: Some(installation.path().to_path_buf()),
                    placement: PlacementSettings {
                        reconnect_delay: Duration::from_secs(1),
                        stream_window,
                        interval_window: crate::INTERVAL_WINDOW,
                        interval_lag: crate::INTERVAL_LAG,
                        message_bytes,
                    },
                    storage: storage_defaults(),
                    clock: Arc::new(WallClock),
                    capture: CaptureSettings::default(),
                    credentials: credentials::namespace(),
                };
                let mut plugin = PluginProcess::start(&executable, true, &settings.placement).await?;
                let host = Host::open(
                    root.path(),
                    settings,
                    ClientSettings {
                        read_timeout: crate::PLUGINS_TIMEOUT,
                        mutation_timeout: crate::PLUGINS_TIMEOUT,
                        watch_timeout: UNBOUNDED,
                        max_cases: 1000,
                        page_items: 500,
                        message_bytes: message_bytes.get(),
                        stream_buffer: stream_window.get(),
                    },
                )
                .await?;
                let admitted = host.add_remote("host", plugin.endpoint()).await?;
                host.await_plugin_ready(&admitted).await?;

                // Observe Two Consecutive Collection Ticks, Without Running a Control.
                let runtime = &host.assembly.as_ref().unwrap().runtime;
                let stream = runtime
                    .streams
                    .current_publication("host.network_probe")
                    .ok_or("Observe Host Network Probe: Admit the Collection Stream")?;
                let mut rows = stream.watch();
                for _ in 0..2 {
                    let row = rows.next().await.ok_or("Observe Host Network Probe: Keep the Collection Publication Open")?;
                    let Item::Row(row) = row else {
                        return Err("Observe Host Network Probe: Resolve the Stream Gap".into());
                    };
                    assert!(!row.value.payload.is_empty());
                    assert!(row.value.published_unix_micros.is_some());
                    assert_eq!(host.entry_state(&admitted)?, PluginState::Ready);
                }

                // Release Demand and Join Only the Host and Child This Story Started.
                drop(rows);
                host.shutdown().await?;
                plugin.close().await?;
                Ok::<(), Box<dyn Error>>(())
            })
            .await?
        }))
    }

    #[test]
    fn client_wrapper_watches_settle_on_disconnect_and_host_stop() -> Result<(), Box<dyn Error>> {
        credentials::run(Box::pin(async {
            // Start an Isolated Host and Observe Every Relayed Watch Opening.
            let root = credentials::tempdir()?;
            let message_bytes = NonZeroUsize::new(4 * 1024 * 1024).unwrap();
            let stream_window = NonZeroUsize::new(8).unwrap();
            let server = serve(
                "127.0.0.1:0".parse()?,
                root.path(),
                HostSettings {
                    installation: None,
                    placement: PlacementSettings {
                        reconnect_delay: Duration::from_secs(1),
                        stream_window,
                        interval_window: crate::INTERVAL_WINDOW,
                        interval_lag: crate::INTERVAL_LAG,
                        message_bytes,
                    },
                    storage: storage_defaults(),
                    clock: Arc::new(WallClock),
                    capture: CaptureSettings::default(),
                    credentials: credentials::namespace(),
                },
                ClientSettings {
                    read_timeout: crate::PLUGINS_TIMEOUT,
                    mutation_timeout: crate::PLUGINS_TIMEOUT,
                    watch_timeout: UNBOUNDED,
                    max_cases: 1000,
                    page_items: 500,
                    message_bytes: message_bytes.get(),
                    stream_buffer: stream_window.get(),
                },
            )
            .await?;
            let watches = server.host.assembly.as_ref().unwrap().client_watches.clone();
            let channel = Endpoint::from_shared(format!("http://{}", server.endpoint()))?.connect().await?;
            let client = Client::from_channel(
                channel,
                dokime_client_internal::ClientSettings {
                    transfer: StorageTransferSettings { chunk_bytes: Some(1024) },
                    retained_bytes: message_bytes,
                    message_bytes,
                    input_items: stream_window,
                    input_bytes: message_bytes,
                },
            )?;
            let mut status = client.status.watch().await?;
            let mut logs = client.status.logs.watch(4096, 16).await?;
            let mut traces = client.status.trace_events.watch(4096, 16).await?;
            let mut plugins = client.plugins.watch().await?;
            assert!(status.next().await?.is_some());
            assert!(logs.next().await?.is_some());
            assert!(traces.next().await?.is_some());
            assert!(plugins.next().await?.is_some());
            assert_eq!(watches.active.borrow().len(), 4);

            // Dropping Every Client Observation Releases Its Transport Subscription.
            drop((status, logs, traces, plugins));
            watches.settle().await?;
            assert!(watches.active.borrow().is_empty());

            // Host Stop Ends All Four Watches Cleanly and Joins Their Native Children.
            let mut status = client.status.watch().await?;
            let mut logs = client.status.logs.watch(4096, 16).await?;
            let mut traces = client.status.trace_events.watch(4096, 16).await?;
            let mut plugins = client.plugins.watch().await?;
            assert!(status.next().await?.is_some());
            assert!(logs.next().await?.is_some());
            assert!(traces.next().await?.is_some());
            assert!(plugins.next().await?.is_some());
            let drain = async {
                while status.next().await?.is_some() {}
                while logs.next().await?.is_some() {}
                while traces.next().await?.is_some() {}
                while plugins.next().await?.is_some() {}
                Ok::<(), Box<dyn Error>>(())
            };
            let (closed, drained) = tokio::join!(server.close(), drain);
            closed?;
            drained?;
            assert!(watches.active.borrow().is_empty());
            Ok(())
        }))
    }
}
