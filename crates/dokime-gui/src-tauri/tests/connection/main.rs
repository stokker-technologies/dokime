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
    net::{Ipv4Addr, SocketAddr},
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
use dokime_gui::connection::*;
use dokime_protocol::v1::{
    self,
    backend_client::BackendClient,
    backend_server::{Backend, BackendServer},
};
use keyring::credential::{Credential, CredentialApi, CredentialBuilderApi, CredentialPersistence};
use std::{
    future::Future,
    sync::{Arc, Mutex},
};

/* === Constants === */
static IDENTITY: std::sync::LazyLock<(String, String)> = std::sync::LazyLock::new(|| {
    let generated =
        rcgen::generate_simple_self_signed(vec!["localhost".into(), "127.0.0.1".into()])
            .expect("a self-signed test certificate is generated");
    (generated.cert.pem(), generated.signing_key.serialize_pem())
});

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Default)]
struct MemorySecrets(Arc<Mutex<BTreeMap<String, Vec<u8>>>>);

struct CredentialEntry {
    store: Arc<Mutex<BTreeMap<String, Vec<u8>>>>,
    account: String,
}

struct Gate {
    entered: Notify,
    release: Semaphore,
}

#[derive(Clone)]
struct Service {
    id: String,
    identity_epoch: Arc<Mutex<String>>,
    health_epoch: String,
    capabilities_epoch: String,
    api_package: String,
    api_version: String,
    readiness: i32,
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
impl CredentialBuilderApi for MemorySecrets {
    fn build(
        &self,
        _: Option<&str>,
        service: &str,
        user: &str,
    ) -> keyring::Result<Box<Credential>> {
        assert_eq!(service, "com.stokker.dokime.backend");
        Ok(Box::new(CredentialEntry {
            store: self.0.clone(),
            account: user.into(),
        }))
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn persistence(&self) -> CredentialPersistence {
        CredentialPersistence::ProcessOnly
    }
}

impl CredentialApi for CredentialEntry {
    fn set_secret(&self, secret: &[u8]) -> keyring::Result<()> {
        self.store
            .lock()
            .unwrap()
            .insert(self.account.clone(), secret.to_vec());
        Ok(())
    }
    fn get_secret(&self) -> keyring::Result<Vec<u8>> {
        self.store
            .lock()
            .unwrap()
            .get(&self.account)
            .cloned()
            .ok_or(keyring::Error::NoEntry)
    }
    fn delete_credential(&self) -> keyring::Result<()> {
        self.store
            .lock()
            .unwrap()
            .remove(&self.account)
            .map(|_| ())
            .ok_or(keyring::Error::NoEntry)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
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
            capabilities_epoch: "epoch".into(),
            api_package: "dokime.v1".into(),
            api_version: dokime_protocol::API_VERSION.into(),
            readiness: v1::Readiness::Ready as i32,
            max_request_bytes: u64::MAX,
            max_response_bytes: u64::MAX,
            descriptor: Sha256::digest(dokime_protocol::FILE_DESCRIPTOR_SET).to_vec(),
            methods: vec![
                "/dokime.v1.Backend/GetIdentity".into(),
                "/dokime.v1.Backend/GetHealth".into(),
                "/dokime.v1.Backend/GetCapabilities".into(),
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

#[tonic::async_trait]
impl Backend for Service {
    async fn get_identity(
        &self,
        request: Request<v1::GetIdentityRequest>,
    ) -> Result<Response<v1::BackendIdentity>, Status> {
        self.authorize(&request)?;
        if let Some(gate) = &self.gate {
            gate.entered.notify_one();
            gate.release.acquire().await.unwrap().forget();
        }
        Ok(Response::new(v1::BackendIdentity {
            backend_id: self.id.clone(),
            epoch: self.identity_epoch.lock().unwrap().clone(),
            display_name: "Fixture".into(),
            product_version: "0.1.0".into(),
            started_at: None,
            source_revision: None,
            sdk_revision: None,
            api: Some(v1::ApiIdentity {
                package: self.api_package.clone(),
                version: self.api_version.clone(),
                descriptor_sha256: self.descriptor.clone(),
            }),
        }))
    }
    async fn get_health(
        &self,
        request: Request<v1::GetHealthRequest>,
    ) -> Result<Response<v1::BackendHealth>, Status> {
        self.authorize(&request)?;
        Ok(Response::new(v1::BackendHealth {
            backend_id: self.id.clone(),
            epoch: self.health_epoch.clone(),
            readiness: self.readiness,
            observed_at: None,
            message: String::new(),
        }))
    }
    async fn get_capabilities(
        &self,
        request: Request<v1::GetCapabilitiesRequest>,
    ) -> Result<Response<v1::BackendCapabilities>, Status> {
        self.authorize(&request)?;
        Ok(Response::new(v1::BackendCapabilities {
            backend_id: self.id.clone(),
            epoch: self.capabilities_epoch.clone(),
            methods: self.methods.clone(),
            limits: Some(v1::BackendLimits {
                max_request_bytes: self.max_request_bytes,
                max_response_bytes: self.max_response_bytes,
                max_content_bytes: Some(u64::MAX),
                max_chunk_bytes: Some(u32::MAX),
                max_page_items: None,
                max_stream_buffer_items: None,
            }),
        }))
    }

    async fn get_diagnostics(
        &self,
        request: Request<v1::GetDiagnosticsRequest>,
    ) -> Result<Response<v1::DiagnosticsResponse>, Status> {
        self.authorize(&request)?;
        Ok(Response::new(v1::DiagnosticsResponse {
            server: Some(v1::BackendDiagnostics {
                version: "fixture".into(),
                process: None,
            }),
            runtime: None,
        }))
    }

    type CollectDiagnosticsStream = std::pin::Pin<
        Box<
            dyn tokio_stream::Stream<Item = Result<v1::DiagnosticsBundleFrame, Status>>
                + Send
                + 'static,
        >,
    >;
    async fn collect_diagnostics(
        &self,
        _request: Request<v1::CollectDiagnosticsRequest>,
    ) -> Result<Response<Self::CollectDiagnosticsStream>, Status> {
        Err(Status::unimplemented("not exercised by this fixture"))
    }

    type WatchLogsStream = std::pin::Pin<
        Box<dyn tokio_stream::Stream<Item = Result<v1::LogFrame, Status>> + Send + 'static>,
    >;
    async fn watch_logs(
        &self,
        request: Request<v1::WatchLogsRequest>,
    ) -> Result<Response<Self::WatchLogsStream>, Status> {
        self.authorize(&request)?;
        let (sender, receiver) = tokio::sync::mpsc::channel(1);
        sender.send(Ok(v1::LogFrame::default())).await.unwrap();
        let closed = self.stream_closed.clone();
        tokio::spawn(async move {
            sender.closed().await;
            closed.notify_one();
        });

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(receiver),
        )))
    }
}

impl tokio_stream::Stream for UploadInput {
    type Item = v1::StagePluginFrame;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        if self.first {
            self.first = false;
            return std::task::Poll::Ready(Some(v1::StagePluginFrame::default()));
        }

        std::task::Poll::Pending
    }
}

impl Drop for UploadInput {
    fn drop(&mut self) {
        self.dropped.notify_one();
    }
}

#[tonic::async_trait]
impl v1::plugins_server::Plugins for Service {
    async fn stage(
        &self,
        request: Request<tonic::Streaming<v1::StagePluginFrame>>,
    ) -> Result<Response<v1::PluginInstallation>, Status> {
        let mut input = request.into_inner();
        assert!(input.message().await?.is_some());
        if let Some(gate) = &self.upload_gate {
            gate.entered.notify_one();
            let _permit = gate.release.acquire().await.unwrap();
        }
        let mut count = 1;
        if self.upload_drain {
            while input.message().await?.is_some() {
                count += 1;
            }
        }
        drop(input);

        Ok(Response::new(v1::PluginInstallation {
            install_id: count.to_string(),
            ..Default::default()
        }))
    }
    async fn list_installations(
        &self,
        _: Request<v1::ListInstallationsRequest>,
    ) -> Result<Response<v1::InstallationPage>, Status> {
        Err(Status::unimplemented("unused test route"))
    }
    async fn get_installation(
        &self,
        _: Request<v1::GetInstallationRequest>,
    ) -> Result<Response<v1::PluginInstallation>, Status> {
        Err(Status::unimplemented("unused test route"))
    }
    async fn consent(
        &self,
        _: Request<v1::ConsentPluginRequest>,
    ) -> Result<Response<v1::PluginInstallation>, Status> {
        Err(Status::unimplemented("unused test route"))
    }
    async fn set_enabled(
        &self,
        _: Request<v1::SetPluginEnabledRequest>,
    ) -> Result<Response<v1::PluginInstallation>, Status> {
        Err(Status::unimplemented("unused test route"))
    }
    async fn remove(
        &self,
        _: Request<v1::RemovePluginRequest>,
    ) -> Result<Response<v1::RemovePluginReply>, Status> {
        Err(Status::unimplemented("unused test route"))
    }
    async fn list_admitted_plugins(
        &self,
        _: Request<v1::ListAdmittedPluginsRequest>,
    ) -> Result<Response<v1::AdmittedPluginPage>, Status> {
        Err(Status::unimplemented("unused test route"))
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
                .add_service(v1::plugins_server::PluginsServer::new(service.clone()))
                .add_service(BackendServer::new(service))
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
    let mut client = BackendClient::connect(second.endpoint.clone())
        .await
        .unwrap();
    bounded(client.get_health(v1::GetHealthRequest {}))
        .await
        .unwrap();
}

pub(crate) async fn identity_is_consistent() {
    for fault in ["pin", "health", "capabilities", "api"] {
        let mut service = Service::new("backend");
        match fault {
            "health" => service.health_epoch = "restarted".into(),
            "capabilities" => service.capabilities_epoch = "restarted".into(),
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

pub(crate) async fn api_negotiation_capabilities() {
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
        "/dokime.v1.Backend/GetCapabilities",
    ] {
        let mut service = Service::new("backend");
        service.methods.retain(|method| method != missing);
        let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
        assert_eq!(
            bounded(ConnectionSession::default().connect(server.local()))
                .await
                .unwrap_err(),
            ConnectionError::IncompatibleApi,
            "missing required capability {missing}"
        );
    }
}

pub(crate) async fn credentials_endpoint_bound() {
    let secrets = MemorySecrets::default();
    let entries = secrets.0.clone();
    keyring::set_default_credential_builder(Box::new(secrets));
    let session = ConnectionSession::default();
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
    assert_eq!(
        entries.lock().unwrap()[&Sha256::digest(first.endpoint.as_bytes())
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()],
        b"private-token"
    );
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
        "../../../../dokime-protocol/proto/fixtures/negotiation.json"
    ))
    .unwrap();

    for case in scenarios {
        let mut service = Service::new("backend");
        service.api_version = case["version"].as_str().unwrap().into();
        service.readiness = case["readiness"]
            .as_i64()
            .unwrap_or(v1::Readiness::Ready as i64) as i32;
        service.max_request_bytes = case["max_request_bytes"].as_u64().unwrap_or(u64::MAX);
        service.max_response_bytes = case["max_response_bytes"].as_u64().unwrap_or(u64::MAX);
        service.health_epoch = case["health_epoch"].as_str().unwrap_or("epoch").into();
        service.capabilities_epoch = case["capabilities_epoch"]
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
    use dokime::client::{Client, Error, Target};

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

    let mut backend = BackendClient::connect(server.endpoint.clone())
        .await
        .unwrap();
    bounded(backend.get_health(v1::GetHealthRequest {}))
        .await
        .unwrap();
}

pub(crate) async fn shared_client_identity() {
    use dokime::client::{Client, Error, Target};

    let service = Service::new("shared");
    let epoch = service.identity_epoch.clone();
    let requests = service.requests.clone();
    let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
    let client = Client::connect(Target::local(&server.endpoint).unwrap(), None, None)
        .await
        .unwrap();

    client.verify(true).await.unwrap();
    let health = client
        .backend_get_health(v1::GetHealthRequest {})
        .await
        .unwrap();
    assert_eq!(health.backend_id, "shared");
    let calls = requests.lock().unwrap().len();
    assert!(matches!(
        client.catalog_get_suite(v1::GetSuiteRequest { suite_id: "absent".into() }).await,
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
    use dokime::client::{Client, Error, Target};

    let mut service = Service::new("shared");
    service.max_response_bytes = 1;
    let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
    let client = Client::connect(Target::local(&server.endpoint).unwrap(), None, None)
        .await
        .unwrap();

    assert!(matches!(
        client.backend_get_health(v1::GetHealthRequest {}).await,
        Err(Error::Rpc(status)) if status.code() == tonic::Code::OutOfRange
    ));
}

pub(crate) async fn shared_stream_path() {
    use dokime::client::{Client, Error, Target};

    for action in ["client", "stream", "drop"] {
        let mut service = Service::new("shared");
        service.methods.push("/dokime.v1.Backend/WatchLogs".into());
        let closed = service.stream_closed.clone();
        let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
        let client = Client::connect(Target::local(&server.endpoint).unwrap(), None, None)
            .await
            .unwrap();
        let stream = client
            .backend_watch_logs(v1::WatchLogsRequest::default())
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
                    .backend_get_health(v1::GetHealthRequest {})
                    .await
                    .unwrap();
            }
            "drop" => drop(stream),
            _ => unreachable!(),
        }

        bounded(closed.notified()).await;
        let mut backend = BackendClient::connect(server.endpoint.clone())
            .await
            .unwrap();
        bounded(backend.get_health(v1::GetHealthRequest {}))
            .await
            .unwrap();
    }
}

pub(crate) async fn shared_stream_together() {
    use dokime::client::{Client, Error, Target};

    let mut service = Service::new("shared");
    service.methods.push("/dokime.v1.Backend/WatchLogs".into());
    let closed = service.stream_closed.clone();
    let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
    let client = Client::connect(Target::local(&server.endpoint).unwrap(), None, None)
        .await
        .unwrap();
    let stream = client
        .backend_watch_logs(v1::WatchLogsRequest::default())
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
        .backend_get_health(v1::GetHealthRequest {})
        .await
        .unwrap();
}

pub(crate) async fn shared_upload_stops() {
    use dokime::client::{Client, Error, Target};

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
        let mut backend = BackendClient::connect(server.endpoint.clone())
            .await
            .unwrap();
        bounded(backend.get_health(v1::GetHealthRequest {}))
            .await
            .unwrap();
    }
}

pub(crate) async fn shared_upload_operation() {
    use dokime::client::{Client, Error, Target};

    for action in ["finish", "cancel", "client", "waiter"] {
        let mut service = Service::new("shared");
        service.methods.push("/dokime.v1.Plugins/Stage".into());
        service.upload_drain = true;
        service.max_request_bytes = 1;
        let gate = Arc::new(Gate::new());
        if action != "finish" {
            service.upload_gate = Some(gate.clone());
        }
        let server = Listener::start(service, false, Ipv4Addr::LOCALHOST).await;
        let client = Client::connect(Target::local(&server.endpoint).unwrap(), None, None)
            .await
            .unwrap();
        let upload = client.plugins_stage_open().unwrap();
        let oversized = v1::StagePluginFrame {
            frame: Some(v1::stage_plugin_frame::Frame::Header(
                v1::StagePluginHeader {
                    size_bytes: 1,
                    sha256: String::new(),
                },
            )),
        };
        assert!(matches!(bounded(upload.send(oversized)).await,
            Err(Error::Rpc(status)) if status.code() == tonic::Code::OutOfRange));
        let retained = upload.clone();
        bounded(upload.send(v1::StagePluginFrame::default()))
            .await
            .unwrap();
        drop(upload);

        if action == "finish" {
            bounded(retained.send(v1::StagePluginFrame::default()))
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
            bounded(retained.send(v1::StagePluginFrame::default())).await,
            Err(Error::Closed)
        ));
        bounded(retained.close()).await;
        bounded(retained.close()).await;
        let mut backend = BackendClient::connect(server.endpoint.clone())
            .await
            .unwrap();
        bounded(backend.get_health(v1::GetHealthRequest {}))
            .await
            .unwrap();
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
        eprintln!("Scenario: api_negotiation_capabilities");
        run_scenario(api_negotiation_capabilities, false, None, false);
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
