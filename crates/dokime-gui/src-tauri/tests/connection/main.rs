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
    collections::BTreeMap,
    convert::Infallible,
    net::{Ipv4Addr, SocketAddr},
    num::NonZeroUsize,
    time::Duration,
};
// Third Party Imports
use sha2::{Digest, Sha256};
use tokio::sync::{Notify, Semaphore};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::{
    Request, Response, Status,
    transport::{Identity, Server, ServerTlsConfig},
};
// Local Imports
use dokime_client::contracts::{Endpoint, backend, catalog, plugins, runtime};
use dokime_client::credentials::{Error as CredentialError, SecretStore};
use dokime_gui::connection::*;
use std::{
    future::Future,
    sync::{Arc, Mutex},
};
use zeroize::Zeroizing;

/* === Constants === */
static IDENTITY: std::sync::LazyLock<(String, String)> = std::sync::LazyLock::new(|| {
    let generated =
        rcgen::generate_simple_self_signed(vec!["localhost".into(), "127.0.0.1".into()])
            .expect("a self-signed test certificate is generated");
    (generated.cert.pem(), generated.signing_key.serialize_pem())
});

static DESCRIPTOR: std::sync::LazyLock<Vec<u8>> = std::sync::LazyLock::new(|| {
    let contracts =
        harmos::operations::Contracts::of::<dokime_client::contracts::Operations>().unwrap();
    let artifacts = harmos::grpc::Artifacts::new(&contracts).unwrap();
    Sha256::digest(artifacts.descriptors()).to_vec()
});

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Default)]
struct MemorySecrets(Arc<Mutex<BTreeMap<String, String>>>);

struct Gate {
    entered: Notify,
    release: Semaphore,
}

#[derive(Clone)]
struct Service {
    id: String,
    identity_epoch: Arc<Mutex<String>>,
    health_epoch: String,
    availability_epoch: String,
    api_package: String,
    api_version: String,
    readiness: backend::Readiness,
    max_request_bytes: u64,
    max_response_bytes: u64,
    descriptor: Vec<u8>,
    methods: Vec<String>,
    token: Option<String>,
    requests: Arc<Mutex<Vec<Option<String>>>>,
    gate: Option<Arc<Gate>>,
    stream_closed: Arc<Notify>,
    upload_gate: Option<Arc<Gate>>,
    upload_drain: bool,
}

struct UploadInput {
    first: bool,
    dropped: Arc<Notify>,
}

struct Listener {
    endpoint: String,
    task: tokio::task::JoinHandle<()>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl SecretStore for MemorySecrets {
    fn read(&self, endpoint: &str) -> Result<Option<Zeroizing<String>>, CredentialError> {
        Ok(self
            .0
            .lock()
            .unwrap()
            .get(endpoint)
            .cloned()
            .map(Zeroizing::new))
    }

    fn write(&self, endpoint: &str, token: &str) -> Result<(), CredentialError> {
        self.0.lock().unwrap().insert(endpoint.into(), token.into());
        Ok(())
    }

    fn remove(&self, endpoint: &str) -> Result<(), CredentialError> {
        self.0.lock().unwrap().remove(endpoint);
        Ok(())
    }
}

impl Gate {
    fn new() -> Self {
        Self {
            entered: Notify::new(),
            release: Semaphore::new(0),
        }
    }
}

impl Service {
    fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            identity_epoch: Arc::new(Mutex::new("epoch".into())),
            health_epoch: "epoch".into(),
            availability_epoch: "epoch".into(),
            api_package: "dokime.v1".into(),
            api_version: backend::API_VERSION.into(),
            readiness: backend::Readiness::Ready,
            max_request_bytes: u64::MAX,
            max_response_bytes: u64::MAX,
            descriptor: DESCRIPTOR.clone(),
            methods: vec![
                "/dokime.v1.Backend/GetIdentity".into(),
                "/dokime.v1.Backend/GetHealth".into(),
                "/dokime.v1.Backend/GetAvailability".into(),
                runtime::GetIdentity::PATH.into(),
                runtime::GetHealth::PATH.into(),
            ],
            token: None,
            requests: Arc::default(),
            gate: None,
            upload_gate: None,
            upload_drain: false,
            stream_closed: Arc::new(Notify::new()),
        }
    }
    fn authorize<T>(&self, request: &Request<T>) -> Result<(), Status> {
        let token = request
            .metadata()
            .get("authorization")
            .map(|value| value.to_str().unwrap().to_owned());
        self.requests.lock().unwrap().push(token.clone());
        if self
            .token
            .as_ref()
            .is_some_and(|expected| token.as_deref() != Some(format!("Bearer {expected}").as_str()))
        {
            return Err(Status::unauthenticated("credentials rejected"));
        }
        Ok(())
    }
}

#[derive(Clone)]
struct BackendService(Service);

#[derive(Clone)]
struct PluginsService(Service);

#[derive(Clone)]
struct RuntimeService(Service);

struct Unary<O> {
    service: Service,
    operation: std::marker::PhantomData<O>,
}

struct Logs(Service);
struct Upload(Service);

type FixtureFuture<T> = std::pin::Pin<Box<dyn Future<Output = Result<T, Status>> + Send>>;
type HttpRequest = tonic::codegen::http::Request<tonic::body::Body>;
type HttpResponse = tonic::codegen::http::Response<tonic::body::Body>;
type HttpFuture = std::pin::Pin<Box<dyn Future<Output = Result<HttpResponse, Infallible>> + Send>>;
type LogEnvelope = harmos::grpc::OutputEnvelope<backend::LogFrame, ()>;
type UploadEnvelope = harmos::grpc::InputEnvelope<(), plugins::StagePluginFrame>;

#[tonic::async_trait]
trait FixtureUnary: harmos::operations::OperationDeclaration + Send + 'static {
    async fn reply(service: Service) -> Result<Self::Response, Status>;
}

#[tonic::async_trait]
impl FixtureUnary for backend::GetIdentity {
    async fn reply(service: Service) -> Result<backend::BackendIdentity, Status> {
        if let Some(gate) = &service.gate {
            gate.entered.notify_one();
            gate.release.acquire().await.unwrap().forget();
        }
        Ok(backend::BackendIdentity {
            backend_id: service.id,
            epoch: service.identity_epoch.lock().unwrap().clone(),
            display_name: "Fixture".into(),
            product_version: "0.1.0".into(),
            started_at: None,
            source_revision: None,
            sdk_revision: None,
            api: Some(backend::ApiIdentity {
                package: service.api_package,
                version: service.api_version,
                descriptor_sha256: service.descriptor,
            }),
        })
    }
}

#[tonic::async_trait]
impl FixtureUnary for backend::GetHealth {
    async fn reply(service: Service) -> Result<backend::BackendHealth, Status> {
        Ok(backend::BackendHealth {
            backend_id: service.id,
            epoch: service.health_epoch,
            readiness: service.readiness,
            observed_at: None,
            message: String::new(),
        })
    }
}

#[tonic::async_trait]
impl FixtureUnary for backend::GetAvailability {
    async fn reply(service: Service) -> Result<backend::BackendAvailability, Status> {
        Ok(backend::BackendAvailability {
            backend_id: service.id,
            epoch: service.availability_epoch,
            methods: service.methods,
            limits: Some(backend::BackendLimits {
                max_request_bytes: service.max_request_bytes,
                max_response_bytes: service.max_response_bytes,
                max_content_bytes: Some(u64::MAX),
                max_chunk_bytes: Some(u32::MAX),
                max_page_items: None,
                max_stream_buffer_items: None,
            }),
        })
    }
}

#[tonic::async_trait]
impl FixtureUnary for backend::GetDiagnostics {
    async fn reply(_: Service) -> Result<backend::DiagnosticsResponse, Status> {
        Ok(backend::DiagnosticsResponse {
            server: Some(backend::BackendDiagnostics {
                version: "fixture".into(),
                process: None,
            }),
            runtime: None,
        })
    }
}

#[tonic::async_trait]
impl FixtureUnary for runtime::GetIdentity {
    async fn reply(service: Service) -> Result<runtime::Identity, Status> {
        Ok(runtime::Identity {
            backend_id: service.id,
            epoch: service.identity_epoch.lock().unwrap().clone(),
            display_name: "Fixture".into(),
            product_version: "0.1.0".into(),
        })
    }
}

#[tonic::async_trait]
impl FixtureUnary for runtime::GetHealth {
    async fn reply(service: Service) -> Result<runtime::Health, Status> {
        let readiness = match service.readiness {
            backend::Readiness::Unspecified | backend::Readiness::Starting => {
                runtime::Readiness::Starting
            }
            backend::Readiness::Ready => runtime::Readiness::Ready,
            backend::Readiness::Degraded => runtime::Readiness::Degraded,
            backend::Readiness::Stopping => runtime::Readiness::Stopping,
        };
        Ok(runtime::Health {
            backend_id: service.id,
            epoch: service.health_epoch,
            readiness,
            observed_at_seconds: 0,
            observed_at_nanoseconds: 0,
            message: String::new(),
        })
    }
}

impl<O> tonic::server::UnaryService<harmos::operations::StreamItem<O::Request>> for Unary<O>
where
    O: FixtureUnary,
    O::Request: harmos::Protobuf + harmos::MemorySize,
    O::Response: harmos::Protobuf,
{
    type Response = harmos::operations::Response<O::Response>;
    type Future = FixtureFuture<Response<Self::Response>>;

    fn call(
        &mut self,
        request: Request<harmos::operations::StreamItem<O::Request>>,
    ) -> Self::Future {
        let service = self.service.clone();
        Box::pin(async move {
            service.authorize(&request)?;
            let response = O::reply(service).await?;
            Ok(Response::new(harmos::operations::Response::new(response)))
        })
    }
}

impl
    tonic::server::ServerStreamingService<harmos::operations::StreamItem<backend::WatchLogsRequest>>
    for Logs
{
    type Response = harmos::operations::Response<LogEnvelope>;
    type ResponseStream = tokio_stream::wrappers::ReceiverStream<Result<Self::Response, Status>>;
    type Future = FixtureFuture<Response<Self::ResponseStream>>;

    fn call(
        &mut self,
        request: Request<harmos::operations::StreamItem<backend::WatchLogsRequest>>,
    ) -> Self::Future {
        let service = self.0.clone();
        Box::pin(async move {
            service.authorize(&request)?;
            let (sender, receiver) = tokio::sync::mpsc::channel(1);
            sender
                .send(Ok(harmos::operations::Response::new(LogEnvelope::Item(
                    backend::LogFrame {
                        time: None,
                        level: backend::LogLevel::Unspecified,
                        target: String::new(),
                        message: String::new(),
                        fields: vec![],
                        dropped: 0,
                    },
                ))))
                .await
                .unwrap();
            let closed = service.stream_closed;
            tokio::spawn(async move {
                sender.closed().await;
                closed.notify_one();
            });
            Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(
                receiver,
            )))
        })
    }
}

impl tonic::server::ClientStreamingService<harmos::operations::StreamItem<UploadEnvelope>>
    for Upload
{
    type Response = harmos::operations::Response<plugins::PluginInstallation>;
    type Future = FixtureFuture<Response<Self::Response>>;

    fn call(
        &mut self,
        request: Request<tonic::Streaming<harmos::operations::StreamItem<UploadEnvelope>>>,
    ) -> Self::Future {
        let service = self.0.clone();
        Box::pin(async move {
            service.authorize(&request)?;
            let mut input = request.into_inner();
            let opening = input
                .message()
                .await?
                .expect("upload starts with its canonical request");
            assert!(matches!(
                opening
                    .into_request()
                    .map_err(|error| Status::data_loss(error.to_string()))?
                    .0,
                UploadEnvelope::Request(())
            ));
            let first = input
                .message()
                .await?
                .expect("upload produces its first item");
            assert!(matches!(
                first
                    .into_request()
                    .map_err(|error| Status::data_loss(error.to_string()))?
                    .0,
                UploadEnvelope::Item(_)
            ));
            if let Some(gate) = &service.upload_gate {
                gate.entered.notify_one();
                let _permit = gate.release.acquire().await.unwrap();
            }
            let mut count = 1;
            if service.upload_drain {
                while let Some(frame) = input.message().await? {
                    assert!(matches!(
                        frame
                            .into_request()
                            .map_err(|error| Status::data_loss(error.to_string()))?
                            .0,
                        UploadEnvelope::Item(_)
                    ));
                    count += 1;
                }
            }
            drop(input);
            Ok(Response::new(harmos::operations::Response::new(
                plugins::PluginInstallation {
                    install_id: count.to_string(),
                    sha256: String::new(),
                    staged_at: None,
                    consented_at: None,
                    state: plugins::PluginInstallationState::Unspecified,
                    last_boot: None,
                    metadata: None,
                    error: None,
                    active: false,
                    restart_required: false,
                    boot_error: None,
                },
            )))
        })
    }
}

impl tonic::server::NamedService for BackendService {
    const NAME: &'static str = "dokime.v1.Backend";
}
impl tonic::server::NamedService for PluginsService {
    const NAME: &'static str = "dokime.v1.Plugins";
}

impl tonic::codegen::Service<HttpRequest> for BackendService {
    type Response = HttpResponse;
    type Error = Infallible;
    type Future = HttpFuture;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: HttpRequest) -> Self::Future {
        let service = self.0.clone();
        Box::pin(async move {
            let response = match request.uri().path() {
                backend::GetIdentity::PATH => {
                    serve_unary::<backend::GetIdentity>(service, request).await
                }
                backend::GetHealth::PATH => {
                    serve_unary::<backend::GetHealth>(service, request).await
                }
                backend::GetAvailability::PATH => {
                    serve_unary::<backend::GetAvailability>(service, request).await
                }
                backend::GetDiagnostics::PATH => {
                    serve_unary::<backend::GetDiagnostics>(service, request).await
                }
                backend::WatchLogs::PATH => wire_codec::<
                    backend::WatchLogsRequest,
                    harmos::operations::Response<LogEnvelope>,
                >()
                .server()
                .server_streaming(Logs(service), request)
                .await,
                _ => Status::unimplemented("unused test route").into_http(),
            };
            Ok(response)
        })
    }
}

impl tonic::codegen::Service<HttpRequest> for PluginsService {
    type Response = HttpResponse;
    type Error = Infallible;
    type Future = HttpFuture;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: HttpRequest) -> Self::Future {
        let service = self.0.clone();
        Box::pin(async move {
            let response = match request.uri().path() {
                plugins::Stage::PATH => {
                    wire_codec::<
                        UploadEnvelope,
                        harmos::operations::Response<plugins::PluginInstallation>,
                    >()
                    .server()
                    .client_streaming(Upload(service), request)
                    .await
                }
                _ => Status::unimplemented("unused test route").into_http(),
            };
            Ok(response)
        })
    }
}

impl tonic::server::NamedService for RuntimeService {
    const NAME: &'static str = "dokime.v1.Runtime";
}

impl tonic::codegen::Service<HttpRequest> for RuntimeService {
    type Response = HttpResponse;
    type Error = Infallible;
    type Future = HttpFuture;

    fn poll_ready(
        &mut self,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: HttpRequest) -> Self::Future {
        let service = self.0.clone();
        Box::pin(async move {
            let response = match request.uri().path() {
                runtime::GetIdentity::PATH => {
                    serve_unary::<runtime::GetIdentity>(service, request).await
                }
                runtime::GetHealth::PATH => {
                    serve_unary::<runtime::GetHealth>(service, request).await
                }
                _ => Status::unimplemented("unused test route").into_http(),
            };
            Ok(response)
        })
    }
}

fn wire_codec<D, E>() -> harmos::grpc::Codec<D, E>
where
    D: harmos::Protobuf + harmos::MemorySize + 'static,
    E: harmos::grpc::Encode,
{
    let bytes = NonZeroUsize::new(1024 * 1024).unwrap();
    harmos::grpc::Codec::new(
        harmos::grpc::DecodeSettings {
            message_bytes: bytes,
            retained_bytes: bytes,
        },
        harmos::grpc::EncodeSettings {
            message_bytes: bytes,
            buffer_bytes: NonZeroUsize::new(8192).unwrap(),
        },
    )
    .unwrap()
}

async fn serve_unary<O>(service: Service, request: HttpRequest) -> HttpResponse
where
    O: FixtureUnary,
    O::Request: harmos::Protobuf + harmos::MemorySize,
    O::Response: harmos::Protobuf,
{
    wire_codec::<O::Request, harmos::operations::Response<O::Response>>()
        .server()
        .unary(
            Unary::<O> {
                service,
                operation: std::marker::PhantomData,
            },
            request,
        )
        .await
}

async fn server_health(endpoint: &str) -> backend::BackendHealth {
    let channel = tonic::transport::Endpoint::from_shared(endpoint.to_owned())
        .unwrap()
        .connect()
        .await
        .unwrap();
    let mut client = tonic::client::Grpc::new(channel);
    client.ready().await.unwrap();
    let reply =
        bounded(client.unary(
            Request::new(harmos::operations::Response::new(
                backend::GetHealthRequest {},
            )),
            tonic::codegen::http::uri::PathAndQuery::from_static(backend::GetHealth::PATH),
            wire_codec::<
                backend::BackendHealth,
                harmos::operations::Response<backend::GetHealthRequest>,
            >(),
        ))
        .await
        .unwrap()
        .into_inner();
    reply
        .into_request()
        .expect("canonical health reply transfers an owned payload")
        .0
}

impl tokio_stream::Stream for UploadInput {
    type Item = plugins::StagePluginFrame;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        if self.first {
            self.first = false;
            return std::task::Poll::Ready(Some(plugins::StagePluginFrame::default()));
        }

        std::task::Poll::Pending
    }
}

impl Drop for UploadInput {
    fn drop(&mut self) {
        self.dropped.notify_one();
    }
}

impl Listener {
    async fn start(service: Service, tls: bool, ip: Ipv4Addr) -> Self {
        let listener = tokio::net::TcpListener::bind(SocketAddr::from((ip, 0)))
            .await
            .unwrap();
        let endpoint = format!(
            "{}://{}/",
            if tls { "https" } else { "http" },
            listener.local_addr().unwrap()
        );
        let mut server = Server::builder();
        if tls {
            let _ = rustls::crypto::ring::default_provider().install_default();
            server = server
                .tls_config(
                    ServerTlsConfig::new().identity(Identity::from_pem(&IDENTITY.0, &IDENTITY.1)),
                )
                .unwrap();
        }
        let task = tokio::spawn(async move {
            server
                .add_service(RuntimeService(service.clone()))
                .add_service(PluginsService(service.clone()))
                .add_service(BackendService(service))
                .serve_with_incoming(TcpListenerStream::new(listener))
                .await
                .unwrap();
        });
        Self { endpoint, task }
    }
    fn local(&self) -> ConnectRequest {
        ConnectRequest {
            target: ConnectionTarget::Local {
                endpoint: self.endpoint.clone(),
            },
            credentials: ConnectionCredentials::None,
            expected_backend_id: None,
        }
    }
    fn remote(&self, trust: bool, token: Option<&str>) -> ConnectRequest {
        ConnectRequest {
            target: ConnectionTarget::Remote {
                endpoint: self.endpoint.clone(),
                ca_certificate_pem: trust.then(|| IDENTITY.0.clone()),
            },
            credentials: token.map_or(ConnectionCredentials::None, |token| {
                ConnectionCredentials::Bearer {
                    token: token.into(),
                    remember: false,
                }
            }),
            expected_backend_id: None,
        }
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        self.task.abort();
    }
}

/* === Public Functions === */

/* === Private Functions === */
async fn bounded<T>(future: impl Future<Output = T>) -> T {
    tokio::time::timeout(Duration::from_secs(5), future)
        .await
        .expect("native session operation timed out")
}

pub(crate) async fn switching_cancels_work() {
    let session = ConnectionSession::default();
    let gate = Arc::new(Gate::new());
    let mut service = Service::new("first");
    service.gate = Some(gate.clone());
    let first = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
    let request = first.local();
    let pending = tokio::spawn({
        let session = session.clone();
        async move { session.connect(request).await }
    });
    bounded(gate.entered.notified()).await;
    assert_eq!(session.status().phase, ConnectionPhase::Connecting);
    let second = Listener::start(Service::new("second"), false, Ipv4Addr::LOCALHOST).await;
    let connected = bounded(session.connect(second.local())).await.unwrap();
    assert_eq!(connected.backend.as_ref().unwrap().backend_id, "second");
    assert_eq!(
        bounded(pending).await.unwrap().unwrap_err(),
        ConnectionError::Superseded
    );
    gate.release.add_permits(1);
    assert_eq!(session.status().generation, connected.generation);
    assert!(matches!(
        session.lease("1"),
        Err(ConnectionError::Superseded)
    ));
    let json = serde_json::to_value(&connected).unwrap();
    assert_eq!(
        json["backend"]["limits"]["maxContentBytes"],
        u64::MAX.to_string()
    );
    assert_eq!(
        json["backend"]["limits"]["maxRequestBytes"],
        u64::MAX.to_string()
    );
    let lease = session.lease(&connected.generation).unwrap();
    let pending_read = tokio::spawn({
        let lease = lease.clone();
        async move {
            lease
                .execute(std::future::pending::<Result<(), Status>>())
                .await
        }
    });
    let disconnected = session.disconnect().unwrap();
    assert_eq!(disconnected.phase, ConnectionPhase::Disconnected);
    assert_eq!(
        bounded(pending_read).await.unwrap(),
        Err(ConnectionError::Superseded)
    );
    assert_eq!(
        lease.execute(async { Ok(()) }).await,
        Err(ConnectionError::Superseded)
    );
    // The session never owns the server: it remains callable after disconnect.
    bounded(server_health(&second.endpoint)).await;
}

pub(crate) async fn identity_is_consistent() {
    for fault in ["pin", "health", "availability", "api"] {
        let mut service = Service::new("backend");
        match fault {
            "health" => service.health_epoch = "restarted".into(),
            "availability" => service.availability_epoch = "restarted".into(),
            "api" => service.api_version = "2".into(),
            _ => {}
        }
        let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
        let mut request = server.local();
        if fault == "pin" {
            request.expected_backend_id = Some("other".into());
        }
        let session = ConnectionSession::default();
        let error = bounded(session.connect(request)).await.unwrap_err();
        assert_eq!(
            error,
            if fault == "api" {
                ConnectionError::IncompatibleApi
            } else {
                ConnectionError::IdentityChanged
            }
        );
        assert_eq!(session.status().phase, ConnectionPhase::Failed);
        assert!(session.status().backend.is_none());
    }
}

pub(crate) async fn compatible_api_descriptor() {
    let mut service = Service::new("backend");
    service.descriptor = vec![7; 32];
    let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
    let connected = bounded(ConnectionSession::default().connect(server.local()))
        .await
        .unwrap();
    assert_eq!(
        connected.backend.unwrap().descriptor_sha256,
        "07".repeat(32)
    );
}

pub(crate) async fn api_negotiation_availability() {
    let mut wrong_package = Service::new("backend");
    wrong_package.api_package = "other.v1".into();
    let server = Listener::start(wrong_package, false, Ipv4Addr::LOCALHOST).await;
    assert_eq!(
        bounded(ConnectionSession::default().connect(server.local()))
            .await
            .unwrap_err(),
        ConnectionError::IncompatibleApi
    );

    for missing in [
        "/dokime.v1.Backend/GetIdentity",
        "/dokime.v1.Backend/GetHealth",
        "/dokime.v1.Backend/GetAvailability",
        runtime::GetIdentity::PATH,
        runtime::GetHealth::PATH,
    ] {
        let mut service = Service::new("backend");
        service.methods.retain(|method| method != missing);
        let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
        assert_eq!(
            bounded(ConnectionSession::default().connect(server.local()))
                .await
                .unwrap_err(),
            ConnectionError::IncompatibleApi,
            "missing required method {missing}"
        );
    }
}

pub(crate) async fn credentials_endpoint_bound() {
    let secrets = MemorySecrets::default();
    let entries = secrets.0.clone();
    let session = ConnectionSession::with_credentials(Arc::new(secrets));
    let mut service = Service::new("backend");
    service.token = Some("private-token".into());
    let first = Listener::start(service.clone(), false, Ipv4Addr::LOCALHOST).await;
    let second = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
    let mut request = first.local();
    request.credentials = ConnectionCredentials::Bearer {
        token: "private-token".into(),
        remember: true,
    };
    let connected = bounded(session.connect(request)).await.unwrap();
    assert!(
        !serde_json::to_string(&connected)
            .unwrap()
            .contains("private-token")
    );
    let mut request = first.local();
    request.target = ConnectionTarget::Local {
        endpoint: first.endpoint.trim_end_matches('/').into(),
    };
    request.credentials = ConnectionCredentials::Saved;
    bounded(session.connect(request)).await.unwrap();
    let mut request = second.local();
    request.credentials = ConnectionCredentials::Saved;
    assert_eq!(
        bounded(session.connect(request)).await.unwrap_err(),
        ConnectionError::CredentialsUnavailable
    );
    let mut request = first.local();
    request.credentials = ConnectionCredentials::Bearer {
        token: "wrong".into(),
        remember: true,
    };
    assert_eq!(
        bounded(session.connect(request)).await.unwrap_err(),
        ConnectionError::Unauthenticated
    );
    assert_eq!(entries.lock().unwrap().len(), 1);
    assert_eq!(entries.lock().unwrap()[&first.endpoint], "private-token");
}

pub(crate) async fn remote_requires_auth() {
    let mut service = Service::new("secure");
    service.token = Some("private-token".into());
    let server = Listener::start(service.clone(), true, Ipv4Addr::LOCALHOST).await;
    let session = ConnectionSession::default();
    assert_eq!(
        bounded(session.connect(server.remote(false, Some("private-token"))))
            .await
            .unwrap_err(),
        ConnectionError::Transport
    );
    assert_eq!(
        bounded(session.connect(server.remote(true, None)))
            .await
            .unwrap_err(),
        ConnectionError::AuthenticationRequired
    );
    assert_eq!(
        bounded(session.connect(server.remote(true, Some("wrong"))))
            .await
            .unwrap_err(),
        ConnectionError::Unauthenticated
    );
    let connected = bounded(session.connect(server.remote(true, Some("private-token"))))
        .await
        .unwrap();
    assert!(connected.remote);
    let wrong_name = Listener::start(service, true, Ipv4Addr::new(127, 0, 0, 2)).await;
    assert_eq!(
        bounded(session.connect(wrong_name.remote(true, Some("private-token"))))
            .await
            .unwrap_err(),
        ConnectionError::Transport
    );
    assert!(session.status().backend.is_none());
}

pub(crate) async fn shared_connection_scenarios() {
    let scenarios: Vec<serde_json::Value> = serde_json::from_str(include_str!(
        "../../../../../packages/dokime/tests/integration/client/fixtures/negotiation.json"
    ))
    .unwrap();

    for case in scenarios {
        let mut service = Service::new("backend");
        service.api_version = case["version"].as_str().unwrap().into();
        service.readiness = match case["readiness"].as_i64().unwrap_or(2) {
            0 => backend::Readiness::Unspecified,
            1 => backend::Readiness::Starting,
            2 => backend::Readiness::Ready,
            3 => backend::Readiness::Degraded,
            4 => backend::Readiness::Stopping,
            value => panic!("fixture has an unknown readiness: {value}"),
        };
        service.max_request_bytes = case["max_request_bytes"].as_u64().unwrap_or(u64::MAX);
        service.max_response_bytes = case["max_response_bytes"].as_u64().unwrap_or(u64::MAX);
        service.health_epoch = case["health_epoch"].as_str().unwrap_or("epoch").into();
        service.availability_epoch = case["availability_epoch"]
            .as_str()
            .unwrap_or("epoch")
            .into();

        service.api_package = case["package"].as_str().unwrap_or("dokime.v1").into();
        service.descriptor = vec![case["descriptor"].as_u64().unwrap_or(0) as u8; 32];
        if let Some(missing) = case["missing"].as_str() {
            service.methods.retain(|method| method != missing);
        }

        let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
        let session = ConnectionSession::default();
        let result = bounded(session.connect(server.local())).await;
        if case["accepted"].as_bool().unwrap() {
            assert!(result.is_ok(), "{}: {result:?}", case["name"]);
        } else {
            let expected = match case["error"].as_str() {
                Some("identity") => ConnectionError::IdentityChanged,
                Some("unavailable") => ConnectionError::BackendUnavailable,
                _ => ConnectionError::IncompatibleApi,
            };
            assert_eq!(result.unwrap_err(), expected, "{}", case["name"]);
        }
    }
}

/* === Tests === */

pub(crate) async fn shared_client_server() {
    use dokime_client::{Client, Error, Target};

    let server = Listener::start(Service::new("shared"), false, Ipv4Addr::LOCALHOST).await;
    let client = Client::connect(
        Target::local(&server.endpoint).unwrap(),
        None,
        Some("shared"),
    )
    .await
    .unwrap();
    assert_eq!(client.backend().backend_id, "shared");
    assert_eq!(client.backend().limits.max_request_bytes, u64::MAX);
    let retained = client.clone();
    let (started, running) = tokio::sync::oneshot::channel();
    let pending = tokio::spawn(async move {
        retained
            .guard(async move {
                started.send(()).unwrap();
                std::future::pending::<()>().await
            })
            .await
    });
    bounded(running).await.unwrap();

    client.close();
    client.close();
    assert!(matches!(
        bounded(pending).await.unwrap(),
        Err(Error::Closed)
    ));
    assert!(matches!(client.channel(), Err(Error::Closed)));
    assert!(matches!(
        client.execute(async { Ok(()) }).await,
        Err(Error::Closed)
    ));

    bounded(server_health(&server.endpoint)).await;
}

pub(crate) async fn shared_client_identity() {
    use dokime_client::{Client, Error, Target};

    let service = Service::new("shared");
    let epoch = service.identity_epoch.clone();
    let requests = service.requests.clone();
    let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
    let client = Client::connect(Target::local(&server.endpoint).unwrap(), None, None)
        .await
        .unwrap();

    client.verify(true).await.unwrap();
    let health = client
        .backend_get_health(backend::GetHealthRequest {})
        .await
        .unwrap();
    assert_eq!(health.backend_id, "shared");
    let calls = requests.lock().unwrap().len();
    assert!(matches!(
        client.catalog_get_suite(catalog::GetSuiteRequest { suite_id: "absent".into() }).await,
        Err(Error::Rpc(status)) if status.code() == tonic::Code::PermissionDenied
    ));

    assert_eq!(requests.lock().unwrap().len(), calls);

    *epoch.lock().unwrap() = "restarted".into();
    assert!(matches!(
        client.verify(true).await,
        Err(Error::IdentityChanged)
    ));
    client.verify(false).await.unwrap();

    client.close();
    assert!(matches!(client.verify(false).await, Err(Error::Closed)));
}

pub(crate) async fn shared_client_limit() {
    use dokime_client::{Client, Error, Target};

    let mut service = Service::new("shared");
    service.max_response_bytes = 1;
    let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
    let client = Client::connect(Target::local(&server.endpoint).unwrap(), None, None)
        .await
        .unwrap();

    assert!(matches!(
        client.backend_get_health(backend::GetHealthRequest {}).await,
        Err(Error::Rpc(status)) if status.code() == tonic::Code::OutOfRange
    ));
}

pub(crate) async fn shared_stream_path() {
    use dokime_client::{Client, Error, Target};

    for action in ["client", "stream", "drop"] {
        let mut service = Service::new("shared");
        service.methods.push("/dokime.v1.Backend/WatchLogs".into());
        let closed = service.stream_closed.clone();
        let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
        let client = Client::connect(Target::local(&server.endpoint).unwrap(), None, None)
            .await
            .unwrap();
        let stream = client
            .backend_watch_logs(backend::WatchLogsRequest {
                min_level: backend::LogLevel::Unspecified,
                replay_lines: 0,
            })
            .await
            .unwrap();
        assert!(bounded(stream.message()).await.unwrap().is_some());

        match action {
            "client" => {
                bounded(client.close_and_wait()).await;
                assert!(matches!(
                    bounded(stream.message()).await,
                    Err(Error::Closed)
                ));
            }
            "stream" => {
                bounded(stream.close()).await;
                client
                    .backend_get_health(backend::GetHealthRequest {})
                    .await
                    .unwrap();
            }
            "drop" => drop(stream),
            _ => unreachable!(),
        }

        bounded(closed.notified()).await;
        bounded(server_health(&server.endpoint)).await;
    }
}

pub(crate) async fn shared_stream_together() {
    use dokime_client::{Client, Error, Target};

    let mut service = Service::new("shared");
    service.methods.push("/dokime.v1.Backend/WatchLogs".into());
    let closed = service.stream_closed.clone();
    let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
    let client = Client::connect(Target::local(&server.endpoint).unwrap(), None, None)
        .await
        .unwrap();
    let stream = client
        .backend_watch_logs(backend::WatchLogsRequest {
            min_level: backend::LogLevel::Unspecified,
            replay_lines: 0,
        })
        .await
        .unwrap();
    let other = stream.clone();
    assert!(bounded(stream.message()).await.unwrap().is_some());
    drop(stream);

    let first = other.message();
    let second = other.message();
    tokio::pin!(first, second);
    assert!(
        tokio::time::timeout(Duration::from_millis(10), &mut first)
            .await
            .is_err()
    );
    assert!(
        tokio::time::timeout(Duration::from_millis(10), &mut second)
            .await
            .is_err()
    );
    bounded(other.close()).await;
    assert!(matches!(bounded(first).await, Err(Error::Closed)));
    assert!(matches!(bounded(second).await, Err(Error::Closed)));
    bounded(other.close()).await;
    bounded(closed.notified()).await;
    client
        .backend_get_health(backend::GetHealthRequest {})
        .await
        .unwrap();
}

pub(crate) async fn shared_upload_stops() {
    use dokime_client::{Client, Error, Target};

    for action in ["reply", "caller", "client"] {
        let mut service = Service::new("shared");
        service.methods.push("/dokime.v1.Plugins/Stage".into());
        let gate = Arc::new(Gate::new());
        if action != "reply" {
            service.upload_gate = Some(gate.clone());
        }
        let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
        let client = Client::connect(Target::local(&server.endpoint).unwrap(), None, None)
            .await
            .unwrap();
        let dropped = Arc::new(Notify::new());
        let input = UploadInput {
            first: true,
            dropped: dropped.clone(),
        };
        let retained = client.clone();
        let task = tokio::spawn(async move { retained.plugins_stage(input).await });

        match action {
            "reply" => {
                bounded(task).await.unwrap().unwrap();
            }
            "caller" => {
                bounded(gate.entered.notified()).await;
                task.abort();
                assert!(bounded(task).await.unwrap_err().is_cancelled());
            }
            "client" => {
                bounded(gate.entered.notified()).await;
                bounded(client.close_and_wait()).await;
                assert!(matches!(bounded(task).await.unwrap(), Err(Error::Closed)));
            }
            _ => unreachable!(),
        }

        bounded(dropped.notified()).await;
        gate.release.add_permits(1);
        bounded(server_health(&server.endpoint)).await;
    }
}

pub(crate) async fn shared_upload_operation() {
    use dokime_client::{Client, Error, Target};

    for action in ["finish", "cancel", "client", "waiter"] {
        let mut service = Service::new("shared");
        service.methods.push("/dokime.v1.Plugins/Stage".into());
        service.upload_drain = true;
        // An Empty Canonical Input Envelope Requires Two Framing Bytes.
        service.max_request_bytes = 2;
        let gate = Arc::new(Gate::new());
        if action != "finish" {
            service.upload_gate = Some(gate.clone());
        }
        let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
        let client = Client::connect(Target::local(&server.endpoint).unwrap(), None, None)
            .await
            .unwrap();
        let upload = client.plugins_stage_open().unwrap();
        let oversized = plugins::StagePluginFrame {
            frame: Some(plugins::StagePluginFrameFrame::Header(
                plugins::StagePluginHeader {
                    size_bytes: 1,
                    sha256: String::new(),
                },
            )),
        };
        assert!(matches!(bounded(upload.send(oversized)).await,
            Err(Error::Rpc(status)) if status.code() == tonic::Code::OutOfRange));
        let retained = upload.clone();
        bounded(upload.send(plugins::StagePluginFrame::default()))
            .await
            .unwrap();
        drop(upload);

        if action == "finish" {
            bounded(retained.send(plugins::StagePluginFrame::default()))
                .await
                .unwrap();
            assert_eq!(bounded(retained.finish()).await.unwrap().install_id, "2");
            assert!(matches!(
                bounded(retained.finish()).await,
                Err(Error::Closed)
            ));
        } else {
            bounded(gate.entered.notified()).await;
            match action {
                "cancel" => bounded(retained.close()).await,
                "client" => bounded(client.close_and_wait()).await,
                "waiter" => {
                    let waiting = retained.clone();
                    let (started, running) = tokio::sync::oneshot::channel();
                    let task = tokio::spawn(async move {
                        let result = waiting.finish();
                        tokio::pin!(result);
                        assert!(
                            tokio::time::timeout(Duration::from_millis(10), &mut result)
                                .await
                                .is_err()
                        );
                        started.send(()).unwrap();
                        result.await
                    });
                    bounded(running).await.unwrap();
                    task.abort();
                    assert!(bounded(task).await.unwrap_err().is_cancelled());
                    bounded(retained.close()).await;
                }
                _ => unreachable!(),
            }
            gate.release.add_permits(1);
        }

        assert!(matches!(
            bounded(retained.send(plugins::StagePluginFrame::default())).await,
            Err(Error::Closed)
        ));
        bounded(retained.close()).await;
        bounded(retained.close()).await;
        bounded(server_health(&server.endpoint)).await;
    }
}

fn fresh_runtime(
    multithread: bool,
    threads: Option<usize>,
    paused: bool,
) -> tokio::runtime::Runtime {
    let _ = multithread;
    let mut builder = tokio::runtime::Builder::new_current_thread();
    builder.enable_all();
    if let Some(threads) = threads {
        builder.worker_threads(threads);
    }
    let _ = paused;
    builder.build().expect("scenario runtime")
}

#[test]
fn domain_contract() {
    {
        eprintln!("Scenario: switching_cancels_work");
        run_scenario(switching_cancels_work, false, None, false);
    }
    {
        eprintln!("Scenario: identity_is_consistent");
        run_scenario(identity_is_consistent, false, None, false);
    }
    {
        eprintln!("Scenario: compatible_api_descriptor");
        run_scenario(compatible_api_descriptor, false, None, false);
    }
    {
        eprintln!("Scenario: api_negotiation_availability");
        run_scenario(api_negotiation_availability, false, None, false);
    }
    {
        eprintln!("Scenario: credentials_endpoint_bound");
        run_scenario(credentials_endpoint_bound, false, None, false);
    }
    {
        eprintln!("Scenario: remote_requires_auth");
        run_scenario(remote_requires_auth, false, None, false);
    }
    {
        eprintln!("Scenario: shared_connection_scenarios");
        run_scenario(shared_connection_scenarios, false, None, false);
    }
    {
        eprintln!("Scenario: shared_client_server");
        run_scenario(shared_client_server, false, None, false);
    }
    {
        eprintln!("Scenario: shared_client_identity");
        run_scenario(shared_client_identity, false, None, false);
    }
    {
        eprintln!("Scenario: shared_client_limit");
        run_scenario(shared_client_limit, false, None, false);
    }
    {
        eprintln!("Scenario: shared_stream_path");
        run_scenario(shared_stream_path, false, None, false);
    }
    {
        eprintln!("Scenario: shared_stream_together");
        run_scenario(shared_stream_together, false, None, false);
    }
    {
        eprintln!("Scenario: shared_upload_stops");
        run_scenario(shared_upload_stops, false, None, false);
    }
    {
        eprintln!("Scenario: shared_upload_operation");
        run_scenario(shared_upload_operation, false, None, false);
    }
}

#[inline(never)]
fn run_scenario<F: std::future::Future>(
    factory: impl FnOnce() -> F,
    multithread: bool,
    threads: Option<usize>,
    paused: bool,
) -> F::Output {
    fresh_runtime(multithread, threads, paused).block_on(Box::pin(factory()))
}
