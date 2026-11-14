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
use std::{future::Future, path::Path, sync::Arc, time::Duration};

// Third Party Imports
use prost::Message;
use prost_reflect::ReflectMessage;
use tokio_util::sync::CancellationToken;
use tonic::{Request, Status, transport::Endpoint};

// Local Imports
use dokime_protocol::client;
use dokime_runtime_next::{HostSettings, services};
use dokime_sdk::{Client, client::ClientSettings};

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
    channel: tonic::transport::Channel,
    pairing: Option<Arc<Pairing>>,
    endpoint: String,
    identity: dokime_sdk::client::v1::StatusGetResponse,
    product_epoch: String,
    methods: Vec<String>,
    cancellation: CancellationToken,
    watches: Arc<tokio::sync::Mutex<tokio::task::JoinSet<()>>>,
    stream_buffer: usize,
    read_timeout: Duration,
    watch_timeout: Duration,
    mutation_timeout: Duration,
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
    Host(#[from] dokime_runtime_next::Error),

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

    pub(crate) async fn connect_target(target: &super::ConnectionTarget, settings: &ConnectionSettings) -> Result<Self, Error> {
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

    pub async fn install_plugin(
        &self, items: impl tokio_stream::Stream<Item = dokime_protocol::client::InstallPluginFrame> + Send + 'static,
    ) -> Result<dokime_protocol::client::PluginInstallation, Error> {
        use tokio_stream::StreamExt;
        let opening = client::PluginsInstallInput {
            value: Some(client::plugins_install_input::Value::Request(client::PluginsInstallRequest {})),
        };
        let items = tokio_stream::once(opening).chain(items.map(|item| client::PluginsInstallInput {
            value: Some(client::plugins_install_input::Value::Item(item)),
        }));
        let mut request = self.request(items)?;
        request.set_timeout(self.mutation_timeout);
        let mut client = client::plugins_client::PluginsClient::new(self.channel.clone());
        Ok(self.execute(client.install(request)).await?.into_inner())
    }

    pub(crate) async fn input_raw(&self, path: &str, receiver: tokio::sync::mpsc::Receiver<Vec<u8>>) -> Result<Vec<u8>, Error> {
        operations::input(self, path, receiver).await
    }

    async fn input<Q, R>(&self, path: &'static str, mut receiver: tokio::sync::mpsc::Receiver<Vec<u8>>) -> Result<Vec<u8>, Error>
    where
        Q: Message + Default + Send + Sync + 'static,
        R: Message + Default + Send + Sync + 'static,
    {
        let (sender, input) = tokio::sync::mpsc::channel(self.stream_buffer);
        let feed = async move {
            while let Some(bytes) = receiver.recv().await {
                let value = Q::decode(bytes.as_slice()).map_err(|error| Status::invalid_argument(format!("{path}: {error}")))?;
                sender.send(value).await.map_err(|_| Status::cancelled(format!("{path}: Host Closed Input")))?;
            }
            Ok::<_, Error>(())
        };
        let consume = async {
            let mut client = tonic::client::Grpc::new(self.channel.clone())
                .max_encoding_message_size(crate::rpc::protocol::FRAME_BYTES as usize)
                .max_decoding_message_size(crate::rpc::protocol::FRAME_BYTES as usize);
            client.ready().await.map_err(|error| Status::unavailable(format!("{path}: {error}")))?;
            let mut request = self.request(tokio_stream::wrappers::ReceiverStream::new(input))?;
            request.set_timeout(self.mutation_timeout);
            let reply: tonic::Response<R> = self
                .execute(client.client_streaming(
                    request,
                    path.parse().expect("Generated Input Path"),
                    tonic_prost::ProstCodec::<Q, R>::default(),
                ))
                .await?;
            Ok::<_, Error>(reply.into_inner().encode_to_vec())
        };
        let (_, response) = tokio::try_join!(feed, consume)?;
        Ok(response)
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

    pub fn identity(&self) -> &dokime_sdk::client::v1::StatusGetResponse {
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

    pub async fn execute<T>(&self, action: impl Future<Output = Result<T, Status>>) -> Result<T, Error> {
        let result = tokio::select! {
            biased;
            _ = self.cancellation.cancelled() => Err(Error::Superseded),
            result = action => result.map_err(Error::from),
        };
        if self.cancellation.is_cancelled() {
            return Err(Error::Superseded);
        }
        result
    }

    pub async fn call_raw(&self, path: &str, payload: &[u8]) -> Result<Vec<u8>, Error> {
        Box::pin(operations::call(self, path, payload)).await
    }

    pub async fn watch_raw(
        &self, path: &str, payload: &[u8],
    ) -> Result<tokio_stream::wrappers::ReceiverStream<Result<Vec<u8>, Error>>, Error> {
        Box::pin(operations::watch(self, path, payload)).await
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
            .execute(transport.unary::<Q, R>(path.parse().expect("generated operation path"), request, R::default().descriptor()))
            .await?;
        Ok(response.into_inner().encode_to_vec())
    }

    async fn watch<Q, O>(
        &self, path: &'static str, payload: &[u8],
    ) -> Result<tokio_stream::wrappers::ReceiverStream<Result<Vec<u8>, Error>>, Error>
    where
        Q: Message + Default + Send + 'static,
        O: ReflectMessage + Default + Send + 'static,
    {
        let value = Q::decode(payload).map_err(|error| Status::invalid_argument(format!("{path}: {error}")))?;
        let mut request = self.request(value)?;
        request.set_timeout(self.watch_timeout);
        let transport = self.client.transport().transport_handle();
        let mut watch = self
            .execute(transport.watch_known::<Q, O>(path.parse().expect("generated operation path"), request, O::default().descriptor()))
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
                    Err(error) => Err(Error::Rpc(error)),
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
        let mut client = Client::from_channel(channel.clone(), settings.client)?;
        if let Some(pairing) = &pairing {
            client = client.authorization(pairing.authorization()?);
        }
        let mut session = Self {
            client: Box::new(client),
            channel,
            pairing,
            endpoint: endpoint.to_owned(),
            identity: Default::default(),
            product_epoch: String::new(),
            methods: Vec::new(),
            cancellation: CancellationToken::new(),
            watches: Arc::new(tokio::sync::Mutex::new(tokio::task::JoinSet::new())),
            stream_buffer: settings.client.input_items.get(),
            read_timeout: settings.services.read_timeout,
            watch_timeout: settings.services.watch_timeout,
            mutation_timeout: settings.services.mutation_timeout,
        };
        let request = session.request(dokime_sdk::client::v1::StatusGetRequest {})?;
        let reply = session.client.status.get(request).await?.into_inner();
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
        let overview: client::backend::OverviewGetResponse =
            session.unary("/dokime.client.v1.backend.Overview/Get", client::backend::OverviewGetRequest::default()).await?;
        session.product_epoch = overview.overview.filter(|value| !value.session_id.is_empty()).ok_or(Error::IncompleteIdentity)?.session_id;
        let mut methods = std::collections::BTreeSet::new();
        let catalog: dokime_sdk::client::v1::CatalogEntriesListResponse = session
            .unary(
                "/harmos.v1.CatalogEntries/List",
                dokime_sdk::client::v1::CatalogEntriesListRequest {
                    kind: dokime_sdk::client::v1::CatalogKind::Operations as i32,
                    ..Default::default()
                },
            )
            .await?;
        let mut retained_bytes = 0usize;
        for operation in catalog.operations.into_iter().filter(|operation| operation.grpc_available) {
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
    use dokime_runtime_next::{HostSettings, PlacementSettings};
    use std::num::NonZeroUsize;
    let message_bytes = NonZeroUsize::new(crate::rpc::protocol::FRAME_BYTES as usize).expect("desktop frame bound");
    let stream_buffer = NonZeroUsize::new(8).expect("desktop stream buffer");
    ConnectionSettings {
        host: HostSettings {
            clock: Arc::new(services::WallClock),
            placement: PlacementSettings {
                connect_timeout: Duration::from_secs(30),
                reconnect_delay: Duration::from_millis(25),
                stop_grace: Duration::from_secs(30),
                stream_window: stream_buffer,
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
            transfer: dokime_sdk::client::StorageTransferSettings { chunk_bytes: Some(64 * 1024) },
            retained_bytes: NonZeroUsize::new(64 * 1024 * 1024).expect("desktop retained bytes"),
            message_bytes,
            input_items: stream_buffer,
            input_bytes: message_bytes,
        },
        connect_timeout: Duration::from_secs(30),
    }
}

/* === Private Functions === */

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
