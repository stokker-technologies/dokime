/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Desktop Execution Host Connection

/* === Modules === */

mod pairing;
#[rustfmt::skip]
mod operations;


/* === Re-Exports === */

pub(crate) use operations::DESCRIPTOR_SHA256;
pub use pairing::Pairing;

/* === Imports === */

// Standard Library Imports
use std::{
    collections::{BTreeSet, HashMap},
    future::Future,
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

// Third Party Imports
use harmos_protocol::model;
use prost::Message;
use prost_reflect::ReflectMessage;
use tokio::sync::{Mutex, mpsc::Receiver};
use tokio::task::JoinSet;
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;
use tonic::{Code, Request, Status, transport::Endpoint};

// Local Imports

use dokime_client_internal::BackendOverviewGetOptions;
use dokime_client_internal::OperationsListOptions;
use dokime_client_internal::StatusGetOptions;
use dokime_client_internal::host::plugin_placement;
use dokime_sdk::StorageTransferSettings;

use super::ConnectionTarget;
use dokime_client_internal::host::PluginEntry;
use dokime_client_internal::host::{PluginManaged, PluginPlacement, PluginState, PluginValue};
use dokime_client_internal::{CallOptions, PluginsCreateOptions};
use dokime_client_internal::{PluginsGetOptions, PluginsWatchOptions};
use dokime_sdk::FileReference;

use dokime_runtime::{HostSettings, services};

#[cfg(feature = "testing")]
use dokime_runtime::ControlledClock;

#[cfg(not(feature = "testing"))]
use dokime_runtime::services::WallClock;

use dokime_client_internal::StatusGetResponse;
use dokime_client_internal::{Client, ClientSettings};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/// Configuration Is Supplied By The Desktop's Settings Owner.
#[derive(Clone)]
pub struct ConnectionSettings {
    pub host: HostSettings,
    pub services: services::ClientSettings,
    pub client: ClientSettings,
    pub connect_timeout: Duration,
}

/// One SDK Connection. Leases And Watches End With Their Connection Generation.
#[derive(Clone)]
pub struct Session {
    client: Box<Client>,
    pairing: Option<Arc<Pairing>>,
    endpoint: String,
    identity: StatusGetResponse,
    product_epoch: String,
    methods: Vec<String>,
    cancellation: CancellationToken,
    watches: Arc<Mutex<JoinSet<()>>>,
    stream_buffer: usize,
    read_timeout: Duration,
    watch_timeout: Duration,
}

/// The Desktop Owns Only The Host It Embeds, Never An Attached Host.
pub struct Connection {
    session: Option<Session>,
    server: Option<services::Server>,
    settings: ConnectionSettings,
}

/* === Enums === */

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Connect to Host: {0}; select a loopback HTTP endpoint or import a pinned .pair file")]
    Target(String),

    #[error("Connect to Host: {0}; check the endpoint and import a new pairing if its certificate changed")]
    Transport(#[from] tonic::transport::Error),

    #[error("Host Request Failed: {0}")]
    Rpc(#[from] Status),

    #[error("Start Local Host: {0}")]
    Host(#[from] dokime_runtime::Error),

    #[error("Connect to Host: the host identity changed; import its current pairing")]
    IdentityChanged,

    #[error("Connect to Host: readiness omitted the host identity or history; inspect host diagnostics and retry")]
    IncompleteIdentity,

    #[error("Host Connection Changed; Retry the Action on the Current Connection")]
    Superseded,

    #[error("Connect to a Host Before Performing This Action")]
    Disconnected,
}

/* === Traits === */

/* === Implementations === */

impl ConnectionSettings {
    fn validate(&self) -> Result<(), Error> {
        if self.connect_timeout.is_zero() {
            return Err(Error::Target("configure a positive connection timeout".into()));
        }
        Ok(())
    }
}

impl Session {
    pub(crate) fn cancel(&self) {
        self.cancellation.cancel();
    }

    pub(crate) async fn join(&self) {
        self.cancel();
        let mut watches = self.watches.lock().await;
        while watches.join_next().await.is_some() {}
    }

    pub(crate) async fn connect_target(target: &ConnectionTarget, settings: &ConnectionSettings) -> Result<Self, Error> {
        match target {
            super::ConnectionTarget::Local { endpoint } => {
                validate_local(endpoint)?;
                Self::connect(endpoint, None, settings).await
            }
            super::ConnectionTarget::Remote { pairing_file } => {
                let pairing = Arc::new(Pairing::read(Path::new(pairing_file))?);
                let mut failure = None;
                for endpoint in &pairing.endpoints {
                    match Self::connect(endpoint, Some(pairing.clone()), settings).await {
                        Ok(session) => return Ok(session),
                        Err(error @ Error::Transport(_)) => failure = Some(error),
                        Err(error) => return Err(error),
                    }
                }
                Err(failure.expect("validated pairing endpoints"))
            }
        }
    }

    pub async fn unary<Q: Message, R: Message + Default>(&self, path: &str, request: Q) -> Result<R, Error> {
        let bytes = self.call_raw(path, &request.encode_to_vec()).await?;
        R::decode(bytes.as_slice()).map_err(|error| Status::data_loss(error.to_string()).into())
    }

    pub async fn install_plugin(&self, name: &str, file: FileReference) -> Result<PluginEntry, Error> {
        let sha256 = file.sha256.clone();
        let placement = PluginPlacement {
            kind: Some(plugin_placement::Kind::Managed(PluginManaged { content: Some(file), sha256 })),
        };
        let options = PluginsCreateOptions {
            origin: None,
            request_id: Some(format!("desktop-install-{name}")),
            context: self.call_options()?,
        };
        let reply = self
            .execute("Plugins.Create", self.client.plugins.create_with(name, placement, PluginValue { enabled: true }, options))
            .await?;
        let mut entry = reply.entry.ok_or_else(|| Status::data_loss("Install Plugin: Host Omitted Its Entry"))?;
        let options = PluginsWatchOptions {
            name: Some(entry.name.clone()),
            context: self.call_options()?,
            ..Default::default()
        };
        let mut watch = self.execute("Plugins.Watch", self.client.plugins.watch_with(options)).await?;
        while entry.state != PluginState::Ready as i32 {
            if matches!(PluginState::try_from(entry.state), Ok(PluginState::Unavailable | PluginState::Stopped)) {
                let mut failure = entry.failure.unwrap_or_else(|| model::Failure {
                    code: "plugin_stopped".into(),
                    message: "The plugin stopped before reporting readiness.".into(),
                    action: "Restart Plugin".into(),
                    plugin: entry.name.clone(),
                    ..Default::default()
                });
                failure.plugin = entry.name.clone();
                let status = Status::with_details(Code::FailedPrecondition, &failure.message, failure.encode_to_vec().into());
                log_failure("Plugins.Create", status.code(), &failure, &entry.name);
                return Err(status.into());
            }
            self.execute("Plugins.Watch", watch.next())
                .await?
                .ok_or_else(|| Status::unavailable("Install Plugin: Plugin Watch Closed Before Readiness"))?;
            let options = PluginsGetOptions {
                context: self.call_options()?,
                ..Default::default()
            };
            entry = self
                .execute("Plugins.Get", self.client.plugins.get_with(entry.name, options))
                .await?
                .entry
                .ok_or_else(|| Status::not_found("Install Plugin: Plugin Entry Was Removed"))?;
        }
        Ok(entry)
    }

    pub fn call_options(&self) -> Result<CallOptions, Error> {
        let mut options = CallOptions::default();
        if let Some(pairing) = &self.pairing {
            options.metadata.insert("authorization", pairing.authorization()?);
        }
        Ok(options)
    }

    pub(crate) async fn input_raw(&self, path: &str, receiver: Receiver<Vec<u8>>) -> Result<Vec<u8>, Error> {
        operations::input(self, path, receiver).await
    }

    pub(crate) fn read_timeout(&self) -> Duration {
        self.read_timeout
    }

    pub(crate) fn watch_timeout(&self) -> Duration {
        self.watch_timeout
    }

    pub(crate) fn with_timeout(&self, timeout: Duration) -> Self {
        let mut session = self.clone();
        session.read_timeout = timeout;
        session
    }

    pub(crate) fn with_watch_timeout(&self, timeout: Duration) -> Self {
        let mut session = self.clone();
        session.watch_timeout = timeout;
        session
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn identity(&self) -> &StatusGetResponse {
        &self.identity
    }

    pub(crate) fn methods(&self) -> &[String] {
        &self.methods
    }

    pub(crate) fn product_epoch(&self) -> &str {
        &self.product_epoch
    }

    pub fn cancellation(&self) -> CancellationToken {
        self.cancellation.clone()
    }

    /// Generated Library Roots Accept Metadata On Each Request.
    pub fn request<T>(&self, message: T) -> Result<Request<T>, Error> {
        let mut request = Request::new(message);
        request.set_timeout(self.read_timeout);
        if let Some(pairing) = &self.pairing {
            request.metadata_mut().insert("authorization", pairing.authorization()?);
        }
        Ok(request)
    }

    pub async fn execute<T>(&self, operation: &str, action: impl Future<Output = Result<T, Status>>) -> Result<T, Error> {
        let result = tokio::select! {
            biased;
            _ = self.cancellation.cancelled() => Err(Error::Superseded),
            result = action => result.map_err(Error::from),
        };
        if self.cancellation.is_cancelled() {
            return Err(Error::Superseded);
        }
        if let Err(Error::Rpc(status)) = &result {
            log_failure(operation, status.code(), &status_failure(status), "");
        }
        result
    }

    pub async fn call_raw(&self, path: &str, payload: &[u8]) -> Result<Vec<u8>, Error> {
        Box::pin(operations::call(self, path, payload)).await
    }

    pub async fn watch_raw(&self, path: &str, payload: &[u8]) -> Result<ReceiverStream<Result<Vec<u8>, Error>>, Error> {
        Box::pin(operations::watch(self, path, payload)).await
    }

    pub(crate) async fn observe_plugins(&self) -> Result<(), Error> {
        use dokime_client_internal::host::{plugin_change, plugins_watch_response};
        let observer = self.clone();
        let options = PluginsWatchOptions {
            context: self.call_options()?,
            ..Default::default()
        };
        let mut watch = self.execute("Plugins.Watch", self.client.plugins.watch_with(options)).await?;
        self.watches.lock().await.spawn(async move {
            let mut failures = HashMap::new();
            loop {
                let event = tokio::select! {
                    biased;
                    _ = observer.cancellation.cancelled() => break,
                    event = observer.execute("Plugins.Watch", watch.next()) => event,
                };
                let event = match event {
                    Ok(Some(event)) => event.event,
                    Ok(None) | Err(_) => break,
                };
                let entries = match event {
                    Some(plugins_watch_response::Event::Snapshot(snapshot)) => {
                        failures.retain(|name, _| snapshot.entries.iter().any(|entry| entry.name == *name));
                        snapshot.entries
                    }
                    Some(plugins_watch_response::Event::Change(change)) => match change.kind {
                        Some(plugin_change::Kind::Entry(entry)) => vec![entry],
                        Some(plugin_change::Kind::Deleted(name)) => {
                            failures.remove(&name);
                            continue;
                        }
                        None => continue,
                    },
                    Some(plugins_watch_response::Event::ResyncRequired(_)) => {
                        let options = PluginsWatchOptions {
                            context: match observer.call_options() {
                                Ok(options) => options,
                                Err(_) => break,
                            },
                            ..Default::default()
                        };
                        watch = match observer.execute("Plugins.Watch", observer.client.plugins.watch_with(options)).await {
                            Ok(watch) => watch,
                            Err(_) => break,
                        };
                        continue;
                    }
                    None => continue,
                };
                for entry in entries {
                    log_plugin_failure(&entry, &mut failures);
                }
            }
        });
        Ok(())
    }

    async fn call<Q, R>(&self, path: &'static str, payload: &[u8]) -> Result<Vec<u8>, Error>
    where
        Q: Message + Default + Send + 'static,
        R: ReflectMessage + Default + Send + 'static,
    {
        let value = Q::decode(payload).map_err(|error| Status::invalid_argument(format!("{path}: {error}")))?;
        let request = self.request(value)?;
        let transport = self.client.transport().transport_handle();
        let response = self
            .execute(path, transport.unary::<Q, R>(path.parse().expect("generated operation path"), request, R::default().descriptor()))
            .await?;
        Ok(response.into_inner().encode_to_vec())
    }

    async fn watch<Q, O>(&self, path: &'static str, payload: &[u8]) -> Result<ReceiverStream<Result<Vec<u8>, Error>>, Error>
    where
        Q: Message + Default + Send + 'static,
        O: ReflectMessage + Default + Send + 'static,
    {
        let value = Q::decode(payload).map_err(|error| Status::invalid_argument(format!("{path}: {error}")))?;
        let mut request = self.request(value)?;
        request.set_timeout(self.watch_timeout);
        let transport = self.client.transport().transport_handle();
        let mut watch = self
            .execute(
                path,
                transport.watch_known::<Q, O>(path.parse().expect("generated operation path"), request, O::default().descriptor()),
            )
            .await?;
        let (sender, receiver) = tokio::sync::mpsc::channel(self.stream_buffer);
        let cancellation = self.cancellation.clone();
        let mut tasks = self.watches.lock().await;
        if cancellation.is_cancelled() {
            return Err(Error::Superseded);
        }
        while tasks.try_join_next().is_some() {}
        tasks.spawn(async move {
            loop {
                let result = tokio::select! {
                    biased;
                    _ = cancellation.cancelled() => break,
                    _ = sender.closed() => break,
                    result = watch.next() => result,
                };
                let message = match result {
                    Ok(Some(message)) => Ok(message.encode_to_vec()),
                    Ok(None) => break,
                    Err(error) => {
                        log_failure(path, error.code(), &status_failure(&error), "");
                        Err(Error::Rpc(error))
                    }
                };
                let terminal = message.is_err();
                tokio::select! {
                    biased;
                    _ = cancellation.cancelled() => break,
                    result = sender.send(message) => if result.is_err() { break; },
                }
                if terminal {
                    break;
                }
            }
        });
        Ok(tokio_stream::wrappers::ReceiverStream::new(receiver))
    }

    async fn connect(endpoint: &str, pairing: Option<Arc<Pairing>>, settings: &ConnectionSettings) -> Result<Self, Error> {
        let mut transport = Endpoint::from_shared(endpoint.to_owned())?.connect_timeout(settings.connect_timeout);
        if let Some(pairing) = &pairing {
            transport = transport.tls_config(
                tonic::transport::ClientTlsConfig::new()
                    .ca_certificate(tonic::transport::Certificate::from_pem(pairing.certificate_pem.as_bytes())),
            )?;
        }
        let channel = transport.connect().await?;
        let client = Client::from_channel(channel.clone(), settings.client)?;
        let mut session = Self {
            client: Box::new(client),
            pairing,
            endpoint: endpoint.to_owned(),
            identity: Default::default(),
            product_epoch: String::new(),
            methods: Vec::new(),
            cancellation: CancellationToken::new(),
            watches: Arc::new(Mutex::new(JoinSet::new())),
            stream_buffer: settings.client.input_items.get(),
            read_timeout: settings.services.read_timeout,
            watch_timeout: settings.services.watch_timeout,
        };
        let options = StatusGetOptions { context: session.call_options()? };
        let reply = session.execute("Status.Get", session.client.status.get_with(options)).await?;
        let identity = reply.status.as_ref().and_then(|status| status.identity.as_ref()).ok_or(Error::IncompleteIdentity)?;
        let history = reply.resolved_at.as_ref().and_then(|position| position.history.as_ref()).ok_or(Error::IncompleteIdentity)?;
        if identity.host_id.is_empty() || history.store.is_empty() || history.epoch.is_empty() {
            return Err(Error::IncompleteIdentity);
        }
        if let Some(pairing) = &session.pairing
            && identity.host_id != pairing.backend_identifier
        {
            return Err(Error::IdentityChanged);
        }
        session.identity = reply;
        let options = BackendOverviewGetOptions {
            context: session.call_options()?,
            ..Default::default()
        };
        let overview = session.execute("Backend.Overview.Get", session.client.backend.overview.get_with(options)).await?;
        session.product_epoch = overview.overview.filter(|value| !value.session_id.is_empty()).ok_or(Error::IncompleteIdentity)?.session_id;
        let mut methods = BTreeSet::new();
        let options = OperationsListOptions { context: session.call_options()? };
        let catalog = session.execute("Operations.List", session.client.operations.list_with(options)).await?;
        let mut retained_bytes = 0usize;
        for operation in catalog.into_iter().filter(|operation| operation.grpc_available) {
            if let Some(grpc) = operation.contract.and_then(|contract| contract.grpc) {
                let path = format!("/{}.{}/{}", grpc.package, grpc.service, grpc.method);
                retained_bytes = retained_bytes.saturating_add(path.len());
                methods.insert(path);
            }
        }
        if retained_bytes > settings.client.retained_bytes.get() {
            return Err(Status::resource_exhausted("Discover Host Operations: Catalog Exceeds the Connection Retention Budget").into());
        }
        session.methods = methods.into_iter().collect();
        Ok(session)
    }
}

impl Connection {
    pub fn new(settings: ConnectionSettings) -> Result<Self, Error> {
        settings.validate()?;
        Ok(Self { session: None, server: None, settings })
    }

    pub fn session(&self) -> Result<Session, Error> {
        self.session.clone().ok_or(Error::Disconnected)
    }

    pub fn is_embedded(&self) -> bool {
        self.server.is_some()
    }

    /// Replaces the embedded host's capture folders for captures opened from now on.
    pub fn set_capture_roots(&self, roots: Vec<PathBuf>) {
        if let Some(server) = &self.server {
            server.set_capture_roots(roots);
        }
    }

    pub async fn start(&mut self, root: &Path) -> Result<Session, Error> {
        self.close().await?;
        let server =
            Box::pin(services::serve(([127, 0, 0, 1], 0).into(), root, self.settings.host.clone(), self.settings.services.clone())).await?;
        let endpoint = format!("http://{}", server.endpoint());
        let result = Session::connect(&endpoint, None, &self.settings).await;
        match result {
            Ok(session) => {
                self.server = Some(server);
                self.session = Some(session.clone());
                // Request failures are recorded by execute; observation is background work.
                let _ = session.observe_plugins().await;
                Ok(session)
            }
            Err(error) => {
                server.close().await?;
                Err(error)
            }
        }
    }

    pub async fn attach_local(&mut self, endpoint: &str) -> Result<Session, Error> {
        validate_local(endpoint)?;
        self.close().await?;
        let session = Session::connect(endpoint, None, &self.settings).await?;
        self.session = Some(session.clone());
        Ok(session)
    }

    pub async fn attach_remote(&mut self, pairing: Pairing) -> Result<Session, Error> {
        pairing.validate()?;
        self.close().await?;
        let pairing = Arc::new(pairing);
        let mut failure = None;
        for endpoint in &pairing.endpoints {
            match Session::connect(endpoint, Some(pairing.clone()), &self.settings).await {
                Ok(session) => {
                    self.session = Some(session.clone());
                    return Ok(session);
                }
                Err(error @ Error::Transport(_)) => failure = Some(error),
                Err(error) => return Err(error),
            }
        }
        Err(failure.expect("validated pairing has an endpoint"))
    }

    pub async fn close(&mut self) -> Result<(), Error> {
        if let Some(session) = self.session.take() {
            session.cancellation.cancel();
            let mut watches = session.watches.lock().await;
            while watches.join_next().await.is_some() {}
        }
        if let Some(server) = self.server.take() {
            server.close().await?;
        }
        Ok(())
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        if let Some(session) = self.session.take() {
            session.cancellation.cancel();
        }
    }
}

/* === Public Functions === */

pub(crate) fn desktop_settings() -> ConnectionSettings {
    use dokime_runtime::{HostSettings, PlacementSettings};
    use std::num::NonZeroUsize;
    let message_bytes = NonZeroUsize::new(crate::rpc::protocol::FRAME_BYTES as usize).expect("desktop frame bound");
    let stream_buffer = NonZeroUsize::new(8).expect("desktop stream buffer");

    #[cfg(feature = "testing")]
    let clock = Arc::new(ControlledClock::new(0));

    #[cfg(not(feature = "testing"))]
    let clock = Arc::new(WallClock);

    // Testing Builds Keep Their Credentials in the Story's Own Namespace.
    #[cfg(feature = "testing")]
    let credentials = crate::test_support::credentials();

    // DOKIME_CREDENTIAL_NAMESPACE Configures the Desktop's Namespace, Default `dokime`.
    #[cfg(not(feature = "testing"))]
    let credentials = match std::env::var("DOKIME_CREDENTIAL_NAMESPACE") {
        Ok(namespace) => {
            dokime_runtime::CredentialNamespace::new(namespace).expect("Start the Desktop: Correct DOKIME_CREDENTIAL_NAMESPACE")
        }
        Err(_) => dokime_runtime::CredentialNamespace::default(),
    };

    ConnectionSettings {
        host: HostSettings {
            installation: None,
            credentials,
            capture: dokime_runtime::CaptureSettings::default(),
            storage: dokime_runtime::storage_defaults(),
            clock,
            placement: PlacementSettings {
                reconnect_delay: Duration::from_millis(25),
                stream_window: stream_buffer,
                interval_window: dokime_runtime::INTERVAL_WINDOW,
                interval_lag: dokime_runtime::INTERVAL_LAG,
                message_bytes,
            },
        },
        services: services::ClientSettings {
            read_timeout: Duration::from_secs(30),
            mutation_timeout: Duration::from_secs(30),
            watch_timeout: Duration::from_secs(3600),
            max_cases: 1000,
            page_items: 1000,
            message_bytes: message_bytes.get(),
            stream_buffer: stream_buffer.get(),
        },
        client: ClientSettings {
            transfer: StorageTransferSettings { chunk_bytes: Some(64 * 1024) },
            retained_bytes: NonZeroUsize::new(64 * 1024 * 1024).expect("desktop retained bytes"),
            message_bytes,
            input_items: stream_buffer,
            input_bytes: message_bytes,
        },
        connect_timeout: Duration::from_secs(30),
    }
}

/* === Private Functions === */

/// The WebView always receives one generated `harmos.model.v1.Failure` protobuf in
/// `error.detail.details`, including plain statuses. Postcard is a host carrier
/// encoding, never a desktop wire contract. Unknown bytes remain support diagnostics.
pub(crate) fn status_failure(status: &Status) -> model::Failure {
    use base64::{Engine, engine::general_purpose::STANDARD};

    let bytes = status.details();
    let decoded = if bytes.first() == Some(&1) {
        harmos_runtime::Failure::decode(bytes).ok().map(|failure| model::Failure::from(&failure))
    } else if bytes.is_empty() {
        None
    } else {
        model::Failure::decode(bytes).ok()
    };
    decoded.unwrap_or_else(|| model::Failure {
        code: status.code().to_string(),
        message: if bytes.is_empty() { status.message().into() } else { format!("gRPC {}: {}", status.code() as i32, status.message()) },
        diagnostics: if bytes.is_empty() {
            String::new()
        } else {
            format!("Undecodable Host Details (Base64): {}", STANDARD.encode(bytes))
        },
        ..Default::default()
    })
}

fn log_plugin_failure(entry: &PluginEntry, seen: &mut HashMap<String, (u64, model::Failure)>) -> bool {
    let Some(failure) = &entry.failure else {
        seen.remove(&entry.name);
        return false;
    };
    let identity = (entry.activation, failure.clone());
    if seen.get(&entry.name) == Some(&identity) {
        return false;
    }
    seen.insert(entry.name.clone(), identity);
    let code = match failure.classification() {
        model::Classification::Refused => Code::FailedPrecondition,
        model::Classification::Cancelled => Code::Cancelled,
        model::Classification::TimedOut => Code::DeadlineExceeded,
        _ => Code::Internal,
    };
    log_failure("Plugins.Watch", code, failure, &entry.name);
    true
}

fn log_failure(operation: &str, code: Code, failure: &model::Failure, plugin: &str) {
    let mut current = Some(failure);
    let mut plugin = plugin;
    let mut action = "";
    while let Some(cause) = current {
        if plugin.is_empty() {
            plugin = &cause.plugin;
        }
        if action.is_empty() {
            action = &cause.action;
        }
        current = cause.cause.as_deref();
    }
    let expected = failure.code == "settings_not_ready"
        || matches!(
            failure.classification(),
            model::Classification::Refused | model::Classification::Cancelled | model::Classification::TimedOut
        );
    if !expected && matches!(code, Code::Internal | Code::Unknown | Code::DataLoss) {
        tracing::error!(operation, failure_operation = %failure.operation, code = code as i32, failure_code = %failure.code, message = %failure.message, action, plugin);
    } else {
        tracing::warn!(operation, failure_operation = %failure.operation, code = code as i32, failure_code = %failure.code, message = %failure.message, action, plugin);
    }
}

pub(super) fn validate_local(endpoint: &str) -> Result<(), Error> {
    let url = url::Url::parse(endpoint).map_err(|error| Error::Target(error.to_string()))?;
    let loopback = match url.host() {
        Some(url::Host::Ipv4(address)) => address.is_loopback(),
        Some(url::Host::Ipv6(address)) => address.is_loopback(),
        Some(url::Host::Domain("localhost")) => true,
        _ => false,
    };
    if url.scheme() != "http"
        || !loopback
        || !url.username().is_empty()
        || url.password().is_some()
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
        || url.port().is_none()
    {
        return Err(Error::Target("use http://localhost:PORT without credentials, a path or query".into()));
    }
    Ok(())
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{log_failure, log_plugin_failure};
    use dokime_client_internal::host::PluginEntry;
    use harmos_protocol::model::Failure;
    use std::collections::HashMap;
    use std::{
        io,
        sync::{Arc, Mutex},
    };
    use tonic::Code;
    use tracing::Level;

    #[derive(Clone)]
    struct Buffer(Arc<Mutex<Vec<u8>>>);
    impl io::Write for Buffer {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn plugin_failures_log_once_until_recovery_or_replacement() {
        let mut seen = HashMap::new();
        let failure = Failure {
            message: "bench_source".into(),
            ..Default::default()
        };
        let mut entry = PluginEntry {
            name: "bench".into(),
            activation: 1,
            failure: Some(failure.clone()),
            ..Default::default()
        };
        assert!(log_plugin_failure(&entry, &mut seen));
        assert!(!log_plugin_failure(&entry, &mut seen));
        entry.revision = vec![1];
        assert!(!log_plugin_failure(&entry, &mut seen));
        entry.failure = None;
        assert!(!log_plugin_failure(&entry, &mut seen));
        entry.failure = Some(failure);
        assert!(log_plugin_failure(&entry, &mut seen));
        entry.activation += 1;
        assert!(log_plugin_failure(&entry, &mut seen));
    }

    #[test]
    fn failures_log_structured_fields_without_payloads_at_default_levels() {
        let bytes = Arc::new(Mutex::new(Vec::new()));
        let output = Buffer(Arc::clone(&bytes));
        let subscriber = tracing_subscriber::fmt()
            .without_time()
            .with_ansi(false)
            .with_max_level(Level::WARN)
            .with_writer(move || output.clone())
            .finish();
        let failure = Failure {
            code: "bench_code".into(),
            message: "bench_message".into(),
            action: "bench_action".into(),
            plugin: "bench".into(),
            diagnostics: "private_payload".into(),
            ..Default::default()
        };
        tracing::subscriber::with_default(subscriber, || {
            log_failure("bench.operation", Code::FailedPrecondition, &failure, "");
            log_failure("bench.operation", Code::Internal, &failure, "");
        });
        let text = String::from_utf8(bytes.lock().unwrap().clone()).unwrap();
        assert_eq!(text.lines().count(), 2);
        assert!(text.contains("WARN"));
        assert!(text.contains("ERROR"));
        for value in [&failure.code, &failure.message, &failure.action, &failure.plugin] {
            assert!(text.contains(value));
        }
        for field in ["operation=", "code=", "action=", "plugin="] {
            assert!(text.contains(field));
        }
        assert!(!text.contains(&failure.diagnostics));
    }
}
