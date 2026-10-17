/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

//! A Configurable gRPC Backend Fixture For Connection And Client Stories.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{
    collections::BTreeMap,
    convert::Infallible,
    future::Future,
    net::{Ipv4Addr, SocketAddr},
    num::NonZeroUsize,
    sync::{Arc, Mutex},
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
use dokime_client::contracts::{Endpoint, backend, plugins, runtime};
use dokime_client::credentials::{Error as CredentialError, SecretStore};
use dokime_domain::model;
use dokime_gui::connection::{ConnectRequest, ConnectionCredentials, ConnectionTarget};
use zeroize::Zeroizing;

/* === Constants === */
static IDENTITY: std::sync::LazyLock<(String, String)> = std::sync::LazyLock::new(|| {
    let generated = rcgen::generate_simple_self_signed(vec!["localhost".into(), "127.0.0.1".into()])
        .expect("a self-signed test certificate is generated");
    (generated.cert.pem(), generated.signing_key.serialize_pem())
});

static DESCRIPTOR: std::sync::LazyLock<Vec<u8>> = std::sync::LazyLock::new(|| {
    let contracts = harmos::operations::Contracts::of::<dokime_client::contracts::Operations>().unwrap();
    let artifacts = harmos_packaging::proto::Artifacts::new(&contracts).unwrap();
    Sha256::digest(artifacts.descriptors()).to_vec()
});

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Default)]
pub struct MemorySecrets(pub Arc<Mutex<BTreeMap<String, String>>>);

pub struct Gate {
    pub entered: Notify,
    pub release: Semaphore,
}

#[derive(Clone)]
pub struct Service {
    pub id: String,
    pub identity_epoch: Arc<Mutex<String>>,
    pub health_epoch: String,
    pub availability_epoch: String,
    pub api_package: String,
    pub api_version: String,
    pub readiness: model::backend::wire::Readiness,
    pub max_request_bytes: u64,
    pub max_response_bytes: u64,
    pub descriptor: Vec<u8>,
    pub methods: Vec<String>,
    pub token: Option<String>,
    pub requests: Arc<Mutex<Vec<Option<String>>>>,
    pub gate: Option<Arc<Gate>>,
    pub stream_closed: Arc<Notify>,
    pub upload_gate: Option<Arc<Gate>>,
    pub upload_drain: bool,
}

pub struct UploadInput {
    pub first: bool,
    pub dropped: Arc<Notify>,
}

pub struct Listener {
    pub endpoint: String,
    pub task: tokio::task::JoinHandle<()>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl SecretStore for MemorySecrets {
    fn read(&self, endpoint: &str) -> Result<Option<Zeroizing<String>>, CredentialError> {
        Ok(self.0.lock().unwrap().get(endpoint).cloned().map(Zeroizing::new))
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
    pub fn new() -> Self {
        Self { entered: Notify::new(), release: Semaphore::new(0) }
    }
}

impl Service {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            identity_epoch: Arc::new(Mutex::new("epoch".into())),
            health_epoch: "epoch".into(),
            availability_epoch: "epoch".into(),
            api_package: "dokime.v1".into(),
            api_version: model::backend::wire::API_VERSION.into(),
            readiness: model::backend::wire::Readiness::Ready,
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
        let token = request.metadata().get("authorization").map(|value| value.to_str().unwrap().to_owned());
        self.requests.lock().unwrap().push(token.clone());
        if self.token.as_ref().is_some_and(|expected| token.as_deref() != Some(format!("Bearer {expected}").as_str())) {
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
type LogEnvelope = harmos::grpc::OutputEnvelope<model::backend::wire::LogFrame, ()>;
type UploadEnvelope = harmos::grpc::InputEnvelope<plugins::Stage, model::plugins::wire::StagePluginFrame>;

#[tonic::async_trait]
trait FixtureUnary: harmos::operations::OperationDeclaration + Send + 'static {
    async fn reply(service: Service) -> Result<Self::Response, Status>;
}

#[tonic::async_trait]
impl FixtureUnary for backend::GetIdentity {
    async fn reply(service: Service) -> Result<model::backend::wire::BackendIdentity, Status> {
        if let Some(gate) = &service.gate {
            gate.entered.notify_one();
            gate.release.acquire().await.unwrap().forget();
        }
        Ok(model::backend::wire::BackendIdentity {
            backend_id: service.id,
            epoch: service.identity_epoch.lock().unwrap().clone(),
            display_name: "Fixture".into(),
            product_version: "0.1.0".into(),
            started_at: None,
            source_revision: None,
            sdk_revision: None,
            api: Some(model::backend::wire::ApiIdentity {
                package: service.api_package,
                version: service.api_version,
                descriptor_sha256: service.descriptor,
            }),
        })
    }
}

#[tonic::async_trait]
impl FixtureUnary for backend::GetHealth {
    async fn reply(service: Service) -> Result<model::backend::wire::BackendHealth, Status> {
        Ok(model::backend::wire::BackendHealth {
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
    async fn reply(service: Service) -> Result<model::backend::wire::BackendAvailability, Status> {
        Ok(model::backend::wire::BackendAvailability {
            backend_id: service.id,
            epoch: service.availability_epoch,
            methods: service.methods,
            limits: Some(model::backend::wire::BackendLimits {
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
    async fn reply(_: Service) -> Result<model::backend::wire::DiagnosticsResponse, Status> {
        Ok(model::backend::wire::DiagnosticsResponse {
            server: Some(model::backend::wire::BackendDiagnostics { version: "fixture".into(), process: None }),
            runtime: None,
        })
    }
}

#[tonic::async_trait]
impl FixtureUnary for runtime::GetIdentity {
    async fn reply(service: Service) -> Result<model::runtime::wire::Identity, Status> {
        Ok(model::runtime::wire::Identity {
            backend_id: service.id,
            epoch: service.identity_epoch.lock().unwrap().clone(),
            display_name: "Fixture".into(),
            product_version: "0.1.0".into(),
        })
    }
}

#[tonic::async_trait]
impl FixtureUnary for runtime::GetHealth {
    async fn reply(service: Service) -> Result<model::runtime::wire::Health, Status> {
        let readiness = match service.readiness {
            model::backend::wire::Readiness::Unspecified | model::backend::wire::Readiness::Starting => {
                model::runtime::wire::Readiness::Starting
            }
            model::backend::wire::Readiness::Ready => model::runtime::wire::Readiness::Ready,
            model::backend::wire::Readiness::Degraded => model::runtime::wire::Readiness::Degraded,
            model::backend::wire::Readiness::Stopping => model::runtime::wire::Readiness::Stopping,
        };
        Ok(model::runtime::wire::Health {
            backend_id: service.id,
            epoch: service.health_epoch,
            readiness,
            observed_at_seconds: 0,
            observed_at_nanoseconds: 0,
            message: String::new(),
        })
    }
}

impl<O> tonic::server::UnaryService<harmos::operations::StreamItem<O>> for Unary<O>
where
    O: FixtureUnary,
    O: harmos::Protobuf + harmos::MemorySize,
    O::Response: harmos::Protobuf,
{
    type Response = harmos::operations::Response<O::Response>;
    type Future = FixtureFuture<Response<Self::Response>>;

    fn call(&mut self, request: Request<harmos::operations::StreamItem<O>>) -> Self::Future {
        let service = self.service.clone();
        Box::pin(async move {
            service.authorize(&request)?;
            let response = O::reply(service).await?;
            Ok(Response::new(harmos::operations::Response::new(response)))
        })
    }
}

impl tonic::server::ServerStreamingService<harmos::operations::StreamItem<backend::WatchLogs>> for Logs {
    type Response = harmos::operations::Response<LogEnvelope>;
    type ResponseStream = tokio_stream::wrappers::ReceiverStream<Result<Self::Response, Status>>;
    type Future = FixtureFuture<Response<Self::ResponseStream>>;

    fn call(&mut self, request: Request<harmos::operations::StreamItem<backend::WatchLogs>>) -> Self::Future {
        let service = self.0.clone();
        Box::pin(async move {
            service.authorize(&request)?;
            let (sender, receiver) = tokio::sync::mpsc::channel(1);
            sender
                .send(Ok(harmos::operations::Response::new(LogEnvelope::Item(model::backend::wire::LogFrame {
                    time: None,
                    level: model::backend::wire::LogLevel::Unspecified,
                    target: String::new(),
                    message: String::new(),
                    fields: vec![],
                    dropped: 0,
                }))))
                .await
                .unwrap();
            let closed = service.stream_closed;
            tokio::spawn(async move {
                sender.closed().await;
                closed.notify_one();
            });
            Ok(Response::new(tokio_stream::wrappers::ReceiverStream::new(receiver)))
        })
    }
}

impl tonic::server::ClientStreamingService<harmos::operations::StreamItem<UploadEnvelope>> for Upload {
    type Response = harmos::operations::Response<model::plugins::wire::PluginInstallation>;
    type Future = FixtureFuture<Response<Self::Response>>;

    fn call(&mut self, request: Request<tonic::Streaming<harmos::operations::StreamItem<UploadEnvelope>>>) -> Self::Future {
        let service = self.0.clone();
        Box::pin(async move {
            service.authorize(&request)?;
            let mut input = request.into_inner();
            let opening = input.message().await?.expect("upload starts with its canonical request");
            assert!(matches!(
                opening.into_request().map_err(|error| Status::data_loss(error.to_string()))?.0,
                UploadEnvelope::Request(plugins::Stage {})
            ));
            let first = input.message().await?.expect("upload produces its first item");
            assert!(matches!(first.into_request().map_err(|error| Status::data_loss(error.to_string()))?.0, UploadEnvelope::Item(_)));
            if let Some(gate) = &service.upload_gate {
                gate.entered.notify_one();
                let _permit = gate.release.acquire().await.unwrap();
            }
            let mut count = 1;
            if service.upload_drain {
                while let Some(frame) = input.message().await? {
                    assert!(matches!(
                        frame.into_request().map_err(|error| Status::data_loss(error.to_string()))?.0,
                        UploadEnvelope::Item(_)
                    ));
                    count += 1;
                }
            }
            drop(input);
            Ok(Response::new(harmos::operations::Response::new(model::plugins::wire::PluginInstallation {
                install_id: count.to_string(),
                sha256: String::new(),
                staged_at: None,
                consented_at: None,
                state: model::plugins::wire::PluginInstallationState::Unspecified,
                last_boot: None,
                metadata: None,
                error: None,
                active: false,
                restart_required: false,
                boot_error: None,
            })))
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

    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: HttpRequest) -> Self::Future {
        let service = self.0.clone();
        Box::pin(async move {
            let response = match request.uri().path() {
                backend::GetIdentity::PATH => serve_unary::<backend::GetIdentity>(service, request).await,
                backend::GetHealth::PATH => serve_unary::<backend::GetHealth>(service, request).await,
                backend::GetAvailability::PATH => serve_unary::<backend::GetAvailability>(service, request).await,
                backend::GetDiagnostics::PATH => serve_unary::<backend::GetDiagnostics>(service, request).await,
                backend::WatchLogs::PATH => {
                    wire_codec::<backend::WatchLogs, harmos::operations::Response<LogEnvelope>>()
                        .server()
                        .server_streaming(Logs(service), request)
                        .await
                }
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

    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: HttpRequest) -> Self::Future {
        let service = self.0.clone();
        Box::pin(async move {
            let response = match request.uri().path() {
                plugins::Stage::PATH => {
                    wire_codec::<UploadEnvelope, harmos::operations::Response<model::plugins::wire::PluginInstallation>>()
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

    fn poll_ready(&mut self, _: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        std::task::Poll::Ready(Ok(()))
    }

    fn call(&mut self, request: HttpRequest) -> Self::Future {
        let service = self.0.clone();
        Box::pin(async move {
            let response = match request.uri().path() {
                runtime::GetIdentity::PATH => serve_unary::<runtime::GetIdentity>(service, request).await,
                runtime::GetHealth::PATH => serve_unary::<runtime::GetHealth>(service, request).await,
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
        harmos::grpc::DecodeSettings { message_bytes: bytes, retained_bytes: bytes },
        harmos::grpc::EncodeSettings { message_bytes: bytes, buffer_bytes: NonZeroUsize::new(8192).unwrap() },
    )
    .unwrap()
}

async fn serve_unary<O>(service: Service, request: HttpRequest) -> HttpResponse
where
    O: FixtureUnary,
    O: harmos::Protobuf + harmos::MemorySize,
    O::Response: harmos::Protobuf,
{
    wire_codec::<O, harmos::operations::Response<O::Response>>()
        .server()
        .unary(Unary::<O> { service, operation: std::marker::PhantomData }, request)
        .await
}

pub async fn server_health(endpoint: &str) -> model::backend::wire::BackendHealth {
    let channel = tonic::transport::Endpoint::from_shared(endpoint.to_owned()).unwrap().connect().await.unwrap();
    let mut client = tonic::client::Grpc::new(channel);
    client.ready().await.unwrap();
    let reply = client
        .unary(
            Request::new(harmos::operations::Response::new(backend::GetHealth {})),
            tonic::codegen::http::uri::PathAndQuery::from_static(backend::GetHealth::PATH),
            wire_codec::<model::backend::wire::BackendHealth, harmos::operations::Response<backend::GetHealth>>(),
        )
        .await
        .unwrap()
        .into_inner();
    reply.into_request().expect("canonical health reply transfers an owned payload").0
}

impl tokio_stream::Stream for UploadInput {
    type Item = model::plugins::wire::StagePluginFrame;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, _: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        if self.first {
            self.first = false;
            return std::task::Poll::Ready(Some(model::plugins::wire::StagePluginFrame::default()));
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
    pub async fn start(service: Service, tls: bool, ip: Ipv4Addr) -> Self {
        let listener = tokio::net::TcpListener::bind(SocketAddr::from((ip, 0))).await.unwrap();
        let endpoint = format!("{}://{}/", if tls { "https" } else { "http" }, listener.local_addr().unwrap());
        let mut server = Server::builder();
        if tls {
            let _ = rustls::crypto::ring::default_provider().install_default();
            server = server.tls_config(ServerTlsConfig::new().identity(Identity::from_pem(&IDENTITY.0, &IDENTITY.1))).unwrap();
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
    pub fn local(&self) -> ConnectRequest {
        ConnectRequest {
            target: ConnectionTarget::Local { endpoint: self.endpoint.clone() },
            credentials: ConnectionCredentials::None,
            expected_backend_id: None,
        }
    }
    pub fn remote(&self, trust: bool, token: Option<&str>) -> ConnectRequest {
        ConnectRequest {
            target: ConnectionTarget::Remote { endpoint: self.endpoint.clone(), ca_certificate_pem: trust.then(|| IDENTITY.0.clone()) },
            credentials: token
                .map_or(ConnectionCredentials::None, |token| ConnectionCredentials::Bearer { token: token.into(), remember: false }),
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

/* === Tests === */
