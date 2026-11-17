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
mod root_claim;
mod state;
mod status;

/* === Re-Exports === */

pub use root_claim::detach_credentials;

pub use clock::WallClock;
pub use dokime_operations::host::ClientSettings;

/* === Imports === */

// Standard Library Imports
use std::fs::OpenOptions;
use std::fs::TryLockError;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{RwLock, Weak};
use std::time::Duration;
use std::{collections::BTreeSet, net::SocketAddr, path::Path, sync::Arc};

// Third Party Imports
use composition::Application;
use harmos_runtime::grpc::Attached;
use harmos_runtime::grpc::middleware::{Enter, Invocation, Middleware, Observation};
use harmos_runtime::operations::OperationManager;
use harmos_runtime::services::status::Registration;
use harmos_runtime::services::status::Status;
use harmos_runtime::{Refusal, Runtime};
use prost_types::Timestamp;
use tokio::sync::Mutex;
use tokio::sync::watch::{self, Sender};
use tokio::task::JoinHandle;

// Local Imports
use crate::RenderSettings;
use crate::{Error, Host, HostSettings, Result};
use dokime_operations::host::Context;
use dokime_operations::host::HostIdentity;
use dokime_operations::host::credentials::CredentialNamespace;
use dokime_protocol::host::RemoteListenerSettings;
use dokime_protocol::host::SettingsGetResponse;
use dokime_storage::Store;

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
    status: Arc<harmos_runtime::services::status::Status>,
    status_task: Option<JoinHandle<Result<()>>>,
    plugin_task: Option<JoinHandle<Result<()>>>,
    plugin_observation: tokio_util::sync::CancellationToken,
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
    /// Replaces the served host's capture folders for captures opened from now on.
    pub fn set_capture_roots(&self, roots: Vec<std::path::PathBuf>) {
        self.host.set_capture_roots(roots);
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

/// Compose Declared Defaults Without Opening Storage, Credentials or Transports.
pub fn initial_preferences() -> SettingsGetResponse {
    let mut snapshot = dokime_operations::host::settings::get::initial_preferences();
    let listener = composition::initial_listener();
    snapshot.value.as_mut().expect("declared application settings").listener = Some(RemoteListenerSettings {
        enabled: listener.enabled,
        address: listener.endpoint,
        certificate_names: listener.certificate_names,
        tls_enabled: true,
        authentication_enabled: true,
    });
    snapshot
}

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
    let host_id = std::fs::read_to_string(root.join("backend-id"))?;
    let storage = Store::open(root)?;
    // A Copied Folder Carries the Identity and Catalogs, Never the Native Owner Claim.
    root_claim::require_owner(root, &storage, namespace)?;
    dokime_operations::host::remove_root_secrets(&storage, namespace)?;
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
    let plugin_observation = tokio_util::sync::CancellationToken::new();
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
    pub backend_lease: dokime_storage::BackendLease,
    pub client_watches: Arc<ClientWatches>,
}

pub(crate) async fn assemble(host: &Host, root: &Path, settings: &ClientSettings) -> Result<Assembly> {
    root_claim::claim(root, &host.store, &host.settings.credentials)?;
    let backend_id = host.store.identity().to_owned();
    let identity = HostIdentity {
        backend_id: backend_id.clone(),
        epoch: uuid::Uuid::now_v7().to_string(),
        product_version: env!("CARGO_PKG_VERSION").into(),
        target: dokime_packaging::bundle::native_target().into(),
        started_at: Some(Timestamp {
            seconds: host.now_ns().div_euclid(1_000_000_000),
            nanos: host.now_ns().rem_euclid(1_000_000_000) as i32,
        }),
        display_name: "Dokime Execution Host".into(),
    };
    let binding = Arc::new(dokime_operations::native::Binding::default());
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
        render_settings: RenderSettings {
            page_size: NonZeroUsize::new(settings.page_items).unwrap(),
            display_points: NonZeroUsize::new(settings.page_items).unwrap(),
        },
        identity,
        temporary_report_limit: settings.max_cases,
    };
    let context = Arc::new(Context {
        administration,
        host: Arc::new(state),
        settings: settings.clone(),
        operations: runtime.operations.clone(),
        caller: None,
        captures: Arc::new(dokime_operations::host::files::captures::Captures::new(host.settings.capture.clone())),
    });
    host.executions.bind_context(&context)?;
    host.executions
        .inner
        .runtime
        .set(Arc::downgrade(&runtime))
        .map_err(|_| Error::Invalid("Bind Execution Runtime: Assemble Once".into()))?;
    host.executions.inner.library_plugins.set(plugins).map_err(|_| Error::Invalid("Bind Execution Plugins: Assemble Once".into()))?;
    let now = host.now_ns();
    dokime_operations::host::executions::commands::recover(
        &context,
        Timestamp {
            seconds: now.div_euclid(1_000_000_000),
            nanos: now.rem_euclid(1_000_000_000) as i32,
        },
    )
    .await?;
    binding.bind(&context)?;
    dokime_operations::host::content::validate_root(&context).await?;
    context.host.refresh_plugin_states().await?;
    dokime_operations::host::register_redactions(&context)?;
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
    operations: OperationManager, context: Weak<Context>, settings: ClientSettings, cancellation: tokio_util::sync::CancellationToken,
    status: Arc<Status>,
) -> Result<()> {
    use harmos_protocol::host::{PluginsWatchRequest, plugins_watch_response::Event};
    use harmos_runtime::builtins::plugins::Watch;
    use harmos_runtime::operations::StreamSettings;
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
                use harmos_protocol::host::plugin_change::Kind;
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
