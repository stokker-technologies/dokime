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
use std::sync::Arc;

// Third Party Imports
use dokime::harmos::plugin::{
    PluginArtifact, PluginBackend, PluginContributions, PluginDefinition, PluginDefinitions, PluginLifecycle, PluginLifecycleAuthority,
    PluginManager, PluginTransition, Router,
};
use sha2::{Digest, Sha256};
// Local Imports
use super::*;
use dokime::domain::model;
use model::plugins::wire::InstallPluginFrameFrame as Frame;

/* === Constants === */

const DYNAMIC_OWNER: &str = "grpc.dynamic-fixture";
const DYNAMIC_POLICY: dokime::harmos::operations::OperationPolicy = dokime::harmos::operations::OperationPolicy {
    timeout: Duration::from_secs(120),
    attempt_timeout: Duration::from_secs(120),
    retries: dokime::harmos::operations::RetryPolicy::Never,
    restart: dokime::harmos::operations::RestartPolicy::Never,
};

/* === Macros === */

/* === Type Aliases === */

type DynamicBackend = PluginBackend<DynamicResources, DynamicResources, DynamicResources>;

/* === Structs === */

#[dokime::harmos::message(harmos_path = dokime::harmos)]
#[derive(Clone)]
struct DynamicNumber {
    value: u32,
}

#[dokime::harmos::message(harmos_path = dokime::harmos)]
struct DynamicText {
    value: String,
}

#[dokime::harmos::message(harmos_path = dokime::harmos)]
#[dokime::harmos::operation(harmos_path = dokime::harmos, response = DynamicText,
    id = "fixture.dynamic.unary", version = 1, policy = DYNAMIC_POLICY,
    grpc(package = "fixture.dynamic.v1", service = "Provider", method = "Unary"))]
struct DynamicReplacement {
    value: String,
}

struct DynamicReplacementHandler {
    calls: Arc<std::sync::atomic::AtomicUsize>,
}

#[dokime::harmos::message(harmos_path = dokime::harmos)]
#[dokime::harmos::operation(harmos_path = dokime::harmos, response = DynamicNumber,
    id = "fixture.dynamic.unary", version = 1, policy = DYNAMIC_POLICY,
    grant = dynamic_grant,
    grpc(package = "fixture.dynamic.v1", service = "Provider", method = "Unary"))]
struct DynamicUnary {
    value: u32,
}

struct DynamicUnaryHandler {
    calls: Arc<std::sync::atomic::AtomicUsize>,
}

#[dokime::harmos::message(harmos_path = dokime::harmos)]
#[dokime::harmos::operation(harmos_path = dokime::harmos, response = DynamicNumber, output = DynamicNumber,
    id = "fixture.dynamic.download", version = 1, policy = DYNAMIC_POLICY,
    grpc(package = "fixture.dynamic.v1", service = "Provider", method = "Download"))]
struct DynamicDownload {
    value: u32,
}

struct DynamicDownloadHandler {
    calls: Arc<std::sync::atomic::AtomicUsize>,
}

#[dokime::harmos::message(harmos_path = dokime::harmos)]
#[dokime::harmos::operation(harmos_path = dokime::harmos, response = DynamicNumber, input = DynamicNumber,
    id = "fixture.dynamic.upload", version = 1, policy = DYNAMIC_POLICY,
    grpc(package = "fixture.dynamic.v1", service = "Provider", method = "Upload"))]
struct DynamicUpload {
    value: u32,
}

struct DynamicUploadHandler {
    calls: Arc<std::sync::atomic::AtomicUsize>,
}

#[dokime::harmos::message(harmos_path = dokime::harmos)]
#[dokime::harmos::operation(harmos_path = dokime::harmos, response = DynamicNumber, input = DynamicNumber, output = DynamicNumber,
    id = "fixture.dynamic.duplex", version = 1, policy = DYNAMIC_POLICY,
    grpc(package = "fixture.dynamic.v1", service = "Provider", method = "Duplex"))]
struct DynamicDuplex {
    value: u32,
}

struct DynamicDuplexHandler {
    calls: Arc<std::sync::atomic::AtomicUsize>,
}

#[derive(Default)]
struct DynamicResources {
    endpoint: Option<dokime::harmos::plugin::PluginEndpoint>,
    registration: Option<dokime::harmos::operations::PluginRegistration>,
}

struct DynamicHost {
    operations: dokime::harmos::operations::OperationManager,
    calls: Arc<std::sync::atomic::AtomicUsize>,
}

/* === Enums === */

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum DynamicOperations {
    Unary(DynamicUnaryHandler),
    Download(DynamicDownloadHandler),
    Upload(DynamicUploadHandler),
    Duplex(DynamicDuplexHandler),
}

#[dokime::harmos::catalog(harmos_path = dokime::harmos)]
enum DynamicReplacements {
    Unary(DynamicReplacementHandler),
}

/* === Traits === */

/* === Implementations === */

impl dokime::harmos::operations::Operation for DynamicReplacementHandler {
    type Declaration = DynamicReplacement;
    type Resources = std::sync::atomic::AtomicUsize;
    type Error = tonic::Status;
    type Context<'a> = dokime::harmos::operations::Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(calls: Arc<Self::Resources>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
        Ok(Self { calls })
    }

    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut DynamicReplacement, _: &mut Self::Endpoints,
    ) -> Result<dokime::harmos::operations::Response<DynamicText>, Self::Error> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(DynamicText { value: request.value.clone() }.into())
    }
}

impl dokime::harmos::operations::Operation for DynamicUnaryHandler {
    type Declaration = DynamicUnary;
    type Resources = std::sync::atomic::AtomicUsize;
    type Error = tonic::Status;
    type Context<'a> = dokime::harmos::operations::Context<'a>;
    type Endpoints = ();

    async fn initialize<'ctx>(calls: Arc<Self::Resources>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
        Ok(Self { calls })
    }

    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut DynamicUnary, _: &mut Self::Endpoints,
    ) -> Result<dokime::harmos::operations::Response<DynamicNumber>, Self::Error> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(DynamicNumber { value: request.value + 1 }.into())
    }
}

impl dokime::harmos::operations::Operation for DynamicDownloadHandler {
    type Declaration = DynamicDownload;
    type Resources = std::sync::atomic::AtomicUsize;
    type Error = tonic::Status;
    type Context<'a> = dokime::harmos::operations::Context<'a>;
    type Endpoints = dokime::harmos::operations::StreamOutput<DynamicNumber>;

    async fn initialize<'ctx>(calls: Arc<Self::Resources>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
        Ok(Self { calls })
    }

    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut DynamicDownload, streams: &mut Self::Endpoints,
    ) -> Result<dokime::harmos::operations::Response<DynamicNumber>, Self::Error> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        for value in [request.value + 1, request.value + 2] {
            streams
                .send_with(std::mem::size_of::<DynamicNumber>(), || Ok(DynamicNumber { value }))
                .await
                .map_err(|error| tonic::Status::internal(error.to_string()))?;
        }
        Ok(DynamicNumber { value: request.value + 3 }.into())
    }
}

impl dokime::harmos::operations::Operation for DynamicUploadHandler {
    type Declaration = DynamicUpload;
    type Resources = std::sync::atomic::AtomicUsize;
    type Error = tonic::Status;
    type Context<'a> = dokime::harmos::operations::Context<'a>;
    type Endpoints = dokime::harmos::operations::StreamInput<DynamicNumber>;

    async fn initialize<'ctx>(calls: Arc<Self::Resources>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
        Ok(Self { calls })
    }

    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut DynamicUpload, streams: &mut Self::Endpoints,
    ) -> Result<dokime::harmos::operations::Response<DynamicNumber>, Self::Error> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut value = request.value;
        while let Some(item) = streams.recv().await {
            value += item.value;
        }
        Ok(DynamicNumber { value }.into())
    }
}

impl dokime::harmos::operations::Operation for DynamicDuplexHandler {
    type Declaration = DynamicDuplex;
    type Resources = std::sync::atomic::AtomicUsize;
    type Error = tonic::Status;
    type Context<'a> = dokime::harmos::operations::Context<'a>;
    type Endpoints = (dokime::harmos::operations::StreamInput<DynamicNumber>, dokime::harmos::operations::StreamOutput<DynamicNumber>);

    async fn initialize<'ctx>(calls: Arc<Self::Resources>, _: Self::Context<'ctx>) -> Result<Self, Self::Error> {
        Ok(Self { calls })
    }

    async fn run<'a>(
        &mut self, _: Self::Context<'a>, request: &mut DynamicDuplex, streams: &mut Self::Endpoints,
    ) -> Result<dokime::harmos::operations::Response<DynamicNumber>, Self::Error> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut value = request.value;
        while let Some(item) = streams.0.recv().await {
            value += item.value;
            streams
                .1
                .send_with(std::mem::size_of::<DynamicNumber>(), || Ok(DynamicNumber { value }))
                .await
                .map_err(|error| tonic::Status::internal(error.to_string()))?;
        }
        Ok(DynamicNumber { value }.into())
    }
}

impl contracts::Endpoint for DynamicUnary {
    const PATH: &'static str = "/fixture.dynamic.v1.Provider/Unary";
}

impl PluginLifecycle for DynamicHost {
    type Guest = DynamicResources;
    type Sidecar = DynamicResources;
    type Embedded = DynamicResources;

    fn preflight(&mut self, candidate: &PluginDefinitions) -> dokime::harmos::Result<()> {
        self.operations.validate_requirements(candidate)
    }

    fn prepare(&mut self, _: &PluginDefinition) -> dokime::harmos::Result<DynamicBackend> {
        Ok(PluginBackend::Embedded(DynamicResources::default()))
    }

    async fn fence(&mut self, _: PluginTransition, _: &[PluginDefinition]) -> dokime::harmos::Result<()> {
        Ok(())
    }

    async fn start(
        &mut self, definition: &PluginDefinition, router: &Router, _: &PluginLifecycleAuthority<'_>, backend: &mut DynamicBackend,
    ) -> dokime::harmos::Result<()> {
        let PluginBackend::Embedded(resources) = backend else { unreachable!("embedded fixture") };
        let endpoint = router.enter(&definition.id).expect("the admitted plugin can enter its original router");
        let scope = endpoint.scope().expect("the manager issued an original plugin scope");
        resources.registration = Some(self.operations.stage_plugin(scope, |bindings| {
            if matches!(&definition.artifact, PluginArtifact::Embedded { revision } if revision == "replacement") {
                bindings.register_wire_operations::<DynamicReplacements, _>(dokime::app::Lanes::Open, self.calls.clone())
            } else {
                bindings.register_wire_operations::<DynamicOperations, _>(dokime::app::Lanes::Open, self.calls.clone())
            }
        })?);
        resources.endpoint = Some(endpoint);
        Ok(())
    }

    async fn stop(_: &PluginDefinition, _: &PluginLifecycleAuthority<'_>, backend: &mut DynamicBackend) -> dokime::harmos::Result<()> {
        let PluginBackend::Embedded(resources) = backend else { unreachable!("embedded fixture") };
        if let Some(registration) = resources.registration.as_mut() {
            registration.shutdown().await?;
        }
        resources.registration.take();
        resources.endpoint.take();
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn dynamic_grant(request: &DynamicUnary) -> Option<dokime::harmos::plugin::Grant> {
    match request.value {
        20 => Some(dokime::harmos::plugin::Grant::of(
            <DynamicUnary as dokime::harmos::operations::OperationDeclaration>::ID,
            "fixture.allowed",
        )),
        21 => Some(dokime::harmos::plugin::Grant::of(
            <DynamicUnary as dokime::harmos::operations::OperationDeclaration>::ID,
            "fixture.refused",
        )),
        22 => Some(dokime::harmos::plugin::Grant::of(
            <DynamicDownload as dokime::harmos::operations::OperationDeclaration>::ID,
            "fixture.foreign",
        )),
        _ => None,
    }
}

fn dynamic_definition(replacement: bool) -> PluginDefinitions {
    use dokime::harmos::operations::{AuthoredContracts, AuthoredDeclarations};
    let (revision, operations, wire_contracts) = if replacement {
        ("replacement", AuthoredDeclarations::of::<DynamicReplacements>().unwrap(), AuthoredContracts::of::<DynamicReplacements>().unwrap())
    } else {
        ("original", AuthoredDeclarations::of::<DynamicOperations>().unwrap(), AuthoredContracts::of::<DynamicOperations>().unwrap())
    };
    let definition = PluginDefinition {
        id: DYNAMIC_OWNER.into(),
        version: "1.0.0".parse().unwrap(),
        artifact: PluginArtifact::Embedded { revision: revision.into() },
        requires: vec![],
        configuration: b"null".to_vec(),
        contributions: PluginContributions { operations, wire_contracts, ..Default::default() },
    };
    PluginDefinitions::new([definition], []).unwrap()
}

fn bundle() -> Vec<u8> {
    bundle_description("Fixture")
}

fn bundle_description(description: &str) -> Vec<u8> {
    use harmos_packaging::bundle::{Bundle, Executable, Facility, Identity, Limits, Source};
    let owner = "grpc-plugin";
    let version = "1.0.0";
    let declaration =
        serde_json::json!({"kind":"plugin","facility":"sidecar","id":owner,"version":version,"description":"A fixture.","config":"()"});
    let policy = dokime::harmos::operations::OperationPolicy {
        timeout: std::time::Duration::from_secs(120),
        attempt_timeout: std::time::Duration::from_secs(120),
        retries: dokime::harmos::operations::RetryPolicy::Never,
        restart: dokime::harmos::operations::RestartPolicy::Never,
    };
    let lifecycle = [
        dokime::harmos::operations::lifecycle::Descriptor::new(dokime::harmos::operations::lifecycle::Action::Initialize, policy).unwrap(),
        dokime::harmos::operations::lifecycle::Descriptor::new(dokime::harmos::operations::lifecycle::Action::Shutdown, policy).unwrap(),
    ];
    let contracts =
        serde_json::json!({"kind":"contracts","facility":"sidecar","owner":owner,"provides":[],"requires":[],"lifecycle":lifecycle});
    let contributions = serde_json::json!({"kind":"contributions","facility":"sidecar","owner":owner,"sections":[]});
    let restart = serde_json::json!({"kind":"restart","facility":"sidecar","owner":owner,"policy":{"credit":3,"sustained_health_ms":30000,"backoff_base_ms":250,"backoff_cap_ms":10000}});
    let publications = serde_json::json!({"kind":"publications","facility":"sidecar","owner":owner,"streams":[]});
    let observers = serde_json::json!({"kind":"observers","facility":"sidecar","owner":owner,"observers":[]});
    let executable = format!(
        "inert{prefix}{declaration}\0{prefix}{contracts}\0{prefix}{contributions}\0{prefix}{restart}\0{prefix}{publications}\0{prefix}{observers}\0",
        prefix = std::str::from_utf8(dokime::harmos::artifacts::PREFIX).unwrap()
    );
    Bundle::build(
        Identity {
            name: "grpc-fixture".into(),
            version: "1.0.0".parse().unwrap(),
            description: description.into(),
            package: Default::default(),
        },
        &[Source {
            path: "bin/fixture",
            bytes: executable.as_bytes(),
            executable: Some(Executable {
                facility: Facility::Sidecar,
                target: dokime::plugins::NATIVE_TARGET.into(),
                registrations: vec![],
            }),
        }],
        Limits::default(),
    )
    .unwrap()
    .bytes()
    .to_vec()
}

async fn fixture(root: &std::path::Path) -> Runtime {
    let mut config =
        crate::support::budgets::runtime_config(root, Metadata::new(Identifier::new("plugins.fixture").unwrap(), "Plugins", None));
    config.installed_plugins.push(dokime::InstalledPlugin {
        source: "grpc.host".into(),
        version: "1.0.0".into(),
        enabled: true,
        ..Default::default()
    });
    Runtime::start(config).await.unwrap()
}

fn frames(bytes: &[u8]) -> Vec<model::plugins::wire::InstallPluginFrame> {
    let mut frames = vec![model::plugins::wire::InstallPluginFrame {
        frame: Some(Frame::Header(model::plugins::wire::InstallPluginHeader {
            size_bytes: bytes.len() as u64,
            sha256: Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        })),
    }];
    for (index, chunk) in bytes.chunks(37).enumerate() {
        frames.push(model::plugins::wire::InstallPluginFrame {
            frame: Some(Frame::Chunk(model::plugins::wire::InstallPluginChunk { offset: (index * 37) as u64, data: chunk.to_vec() })),
        });
    }
    frames
}

#[tokio::test]
async fn plugin_install_identity() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture(root.path()).await;
    let server = Server::start(owner.handle(), keyed()).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    assert_eq!(
        client.unary::<contracts::plugins::ListInstallations>(contracts::plugins::ListInstallations::default()).await.unwrap_err().code(),
        Code::Unauthenticated
    );
    let staged =
        client.upload::<contracts::plugins::Install>(authenticated(tokio_stream::iter(frames(&bundle())))).await.unwrap().into_inner();
    assert_eq!(staged.state, model::plugins::wire::PluginInstallationState::Installed);
    assert!(!staged.active);
    assert!(staged.restart_required);
    assert_eq!(staged.metadata.as_ref().unwrap().owners, ["grpc-plugin"]);
    assert_eq!(staged.metadata.as_ref().unwrap().identity.as_ref().unwrap().name, "grpc-fixture");
    assert!(staged.installed_at.is_some());
    let disabled = client
        .unary::<contracts::plugins::SetEnabled>(authenticated(contracts::plugins::SetEnabled {
            install_id: staged.install_id.clone(),
            enabled: false,
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(disabled.state, model::plugins::wire::PluginInstallationState::Disabled);
    let page = client
        .unary::<contracts::plugins::ListInstallations>(authenticated(contracts::plugins::ListInstallations::default()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(page.total_count, 1);
    assert_eq!(page.installations[0].install_id, staged.install_id);
    let removed = client
        .unary::<contracts::plugins::Remove>(authenticated(contracts::plugins::Remove { install_id: staged.install_id.clone() }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(removed.installation.unwrap().state, model::plugins::wire::PluginInstallationState::RemovalPending);
    let current = client
        .unary::<contracts::plugins::GetInstallation>(authenticated(contracts::plugins::GetInstallation { install_id: staged.install_id }))
        .await
        .unwrap()
        .into_inner();
    assert!(current.restart_required);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn full_control_plugin_operations() {
    use dokime::client::{Client, Target, dynamic::Event};
    use dokime::harmos::operations::OperationDeclaration;
    use std::sync::atomic::{AtomicUsize, Ordering};

    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture(root.path()).await;
    let runtime = owner.handle();
    let calls = Arc::new(AtomicUsize::new(0));
    let server = Server::start(runtime.clone(), keyed()).await.unwrap();
    let allowed = Client::connect(Target::local(server.local_endpoint()).unwrap(), Some(TOKEN), None).await.unwrap();
    assert!(!allowed.operations().await.unwrap().iter().any(|entry| entry.owner.as_str() == DYNAMIC_OWNER));

    // A Host Provider Cannot Become A Plugin By Choosing Its Owner Name.
    runtime.operations().register_wire_operation::<DynamicUnaryHandler>(DYNAMIC_OWNER, dokime::app::Lanes::Open, calls.clone()).unwrap();
    assert!(!allowed.operations().await.unwrap().iter().any(|entry| entry.owner.as_str() == DYNAMIC_OWNER));
    let direct = WireClient::connect(server.local_endpoint()).await.unwrap();
    let refused = direct.unary::<DynamicUnary>(authenticated(DynamicUnary { value: 10 })).await.unwrap_err();
    assert_eq!(refused.code(), Code::PermissionDenied);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    runtime.operations().unregister_from(DYNAMIC_OWNER, DynamicUnary::ID).await.unwrap();

    // Activate A Real Plugin After Connecting; Discovery Uses Its Live Scope.
    let mut host = DynamicHost { operations: runtime.operations().clone(), calls: calls.clone() };
    let mut plugins = PluginManager::new(dynamic_definition(false)).unwrap();
    plugins.initialize(&mut host).await.unwrap();
    let catalog = allowed.operations().await.unwrap();
    for id in [DynamicUnary::ID, DynamicDownload::ID, DynamicUpload::ID, DynamicDuplex::ID] {
        assert!(
            catalog
                .iter()
                .any(|entry| entry.owner.as_str() == DYNAMIC_OWNER && entry.contract.operation().id.as_str() == id && entry.grpc_available)
        );
    }

    // A Different Request Schema Must Fail Before Handler Mutation.
    let malformed = authenticated(dokime::harmos::operations::Response::new(DynamicReplacement { value: "wrong-schema".into() }));
    let error = direct
        .grpc()
        .await
        .unwrap()
        .unary(
            malformed,
            <DynamicUnary as contracts::Endpoint>::PATH.parse().unwrap(),
            direct.codec::<DynamicNumber, DynamicReplacement>().unwrap(),
        )
        .await
        .err()
        .expect("wrong request schema is refused");
    assert_eq!(error.code(), Code::InvalidArgument);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    for (id, input, expected, completion) in [
        (DynamicUnary::ID, false, vec![], 11),
        (DynamicDownload::ID, false, vec![11, 12], 13),
        (DynamicUpload::ID, true, vec![], 16),
        (DynamicDuplex::ID, true, vec![11, 13, 16], 16),
    ] {
        let inputs = if input {
            [1, 2, 3].into_iter().map(|value| serde_json::to_vec(&serde_json::json!({"value": value})).unwrap()).collect()
        } else {
            vec![]
        };
        let stream = allowed.call_operation(id, Some(DYNAMIC_OWNER), b"{\"value\":10}", inputs).await.unwrap();
        let mut received = Vec::new();
        let mut finished = false;
        {
            while let Some(event) = stream.message().await.unwrap() {
                assert!(!finished, "completion must be the final event for {id}");
                match event {
                    Event::Item(value) => received.push(value["value"].as_u64().expect("numeric fixture item")),
                    Event::Completion(value) => {
                        assert_eq!(value["value"], completion, "{id}");
                        finished = true;
                    }
                }
            }
        }
        assert_eq!(received, expected, "{id}");
        assert!(finished, "{id} needs completion and final OK status");
        stream.close().await;
    }
    assert_eq!(calls.load(Ordering::SeqCst), 4);
    // Full Control Covers New Scopes But Still Refuses A Foreign Operation Grant.
    let foreign = authenticated(DynamicUnary { value: 22 });
    assert_eq!(direct.unary::<DynamicUnary>(foreign).await.unwrap_err().code(), Code::PermissionDenied);
    assert_eq!(calls.load(Ordering::SeqCst), 4);
    for value in [20, 21] {
        let reply = direct.unary::<DynamicUnary>(authenticated(DynamicUnary { value })).await.unwrap();
        assert_eq!(reply.into_inner().value, value + 1);
    }
    assert_eq!(calls.load(Ordering::SeqCst), 6);

    // Reusing An ID Must Publish The New Generation And Its Entire Schema.
    let original = catalog
        .iter()
        .find(|entry| entry.owner.as_str() == DYNAMIC_OWNER && entry.contract.operation().id.as_str() == DynamicUnary::ID)
        .unwrap();
    plugins.reload(dynamic_definition(true), &mut host).await.unwrap();
    let replaced = allowed.operations().await.unwrap();
    let replacement = replaced
        .iter()
        .find(|entry| entry.owner.as_str() == DYNAMIC_OWNER && entry.contract.operation().id.as_str() == DynamicUnary::ID)
        .unwrap();
    assert_ne!(replacement.generation, original.generation);
    assert!(!replacement.contract.compatible(&original.contract));
    assert!(!original.contract.compatible(&replacement.contract));
    assert_eq!(direct.unary::<DynamicUnary>(authenticated(DynamicUnary { value: 10 })).await.unwrap_err().code(), Code::InvalidArgument);
    assert!(allowed.call_operation(DynamicUnary::ID, Some(DYNAMIC_OWNER), b"{\"value\":10}", vec![]).await.is_err());
    assert_eq!(calls.load(Ordering::SeqCst), 6, "stale requests must not enter the replacement");
    let replacement = allowed.call_operation(DynamicUnary::ID, Some(DYNAMIC_OWNER), b"{\"value\":\"replacement\"}", vec![]).await.unwrap();
    let Some(Event::Completion(value)) = replacement.message().await.unwrap() else {
        panic!("replacement unary call requires completion");
    };
    assert_eq!(value["value"], "replacement");
    assert!(replacement.message().await.unwrap().is_none());
    replacement.close().await;
    assert_eq!(calls.load(Ordering::SeqCst), 7);

    allowed.close_and_wait().await;
    server.shutdown().await.unwrap();
    plugins.shutdown(&mut host).await.unwrap();
    drop(runtime);
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn invalid_uploads_mutate() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture(root.path()).await;
    let server = Server::start(owner.handle(), keyed()).await.unwrap();
    let mut client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let mut bad = frames(&bundle());
    if let Some(Frame::Chunk(chunk)) = &mut bad[1].frame {
        chunk.offset = 1;
    }
    assert_eq!(
        client.upload::<contracts::plugins::Install>(authenticated(tokio_stream::iter(bad))).await.unwrap_err().code(),
        Code::InvalidArgument
    );
    let mut short = frames(&bundle());
    short.pop();
    assert_eq!(
        client.upload::<contracts::plugins::Install>(authenticated(tokio_stream::iter(short))).await.unwrap_err().code(),
        Code::InvalidArgument
    );
    assert_eq!(
        client
            .unary::<contracts::plugins::ListInstallations>(authenticated(contracts::plugins::ListInstallations::default()))
            .await
            .unwrap()
            .into_inner()
            .total_count,
        0
    );
    let (sender, upload) = open_upload(&mut client).await;
    upload.abort();
    let _ = upload.await;
    drop(sender);
    {
        loop {
            match client
                .unary::<contracts::plugins::ListInstallations>(authenticated(contracts::plugins::ListInstallations::default()))
                .await
            {
                Ok(page) => {
                    assert_eq!(page.into_inner().total_count, 0);
                    break;
                }
                Err(error) if error.code() == Code::ResourceExhausted => tokio::task::yield_now().await,
                other => panic!("unexpected cancellation result: {other:?}"),
            }
        }
    }
    let inventory = client
        .unary::<contracts::plugins::ListAdmittedPlugins>(authenticated(contracts::plugins::ListAdmittedPlugins::default()))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(inventory.plugins.len(), 1);
    assert_eq!(inventory.plugins[0].plugin_id, "grpc.host");
    assert_eq!(inventory.plugins[0].owner, Some(model::plugins::wire::AdmittedPluginOwner::Host(true)));
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn listener_shutdown_admission() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture(root.path()).await;
    let service = dokime_server::grpc::Service::new(owner.handle(), keyed()).await.unwrap();
    let server = service.listen().await.unwrap();
    let mut client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let (sender, upload) = open_upload(&mut client).await;
    server.shutdown().await.unwrap();
    let error = upload.await.unwrap().unwrap_err();
    assert_eq!(error.code(), Code::Unavailable);
    drop(sender);
    let server = service.listen().await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    assert_eq!(
        client
            .unary::<contracts::plugins::ListInstallations>(authenticated(contracts::plugins::ListInstallations::default()))
            .await
            .unwrap()
            .into_inner()
            .total_count,
        0
    );
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

// A competing mutation can win admission before the upload arrives. Retry
// only that explicit pre-admission Busy outcome, never an uncertain mutation.
async fn open_upload(
    client: &mut WireClient,
) -> (
    tokio::sync::mpsc::Sender<model::plugins::wire::InstallPluginFrame>,
    tokio::task::JoinHandle<Result<tonic::Response<model::plugins::wire::PluginInstallation>, tonic::Status>>,
) {
    loop {
        let (sender, receiver) = tokio::sync::mpsc::channel(1);
        sender.send(frames(&bundle()).remove(0)).await.unwrap();
        let uploader = client.clone();
        let upload = tokio::spawn(async move {
            uploader.upload::<contracts::plugins::Install>(authenticated(tokio_stream::wrappers::ReceiverStream::new(receiver))).await
        });
        // Let the streaming RPC submit its request before the admission
        // probe; otherwise this task wins the sole permit every time.
        tokio::task::yield_now().await;
        loop {
            match client
                .unary::<contracts::plugins::SetEnabled>(authenticated(contracts::plugins::SetEnabled {
                    install_id: "01900000-0000-7000-8000-000000000001".into(),
                    enabled: true,
                }))
                .await
            {
                Err(error) if error.code() == Code::ResourceExhausted => {
                    return (sender, upload);
                }
                Err(error) if error.code() == Code::NotFound => {
                    if upload.is_finished() {
                        assert_eq!(upload.await.unwrap().unwrap_err().code(), Code::ResourceExhausted);
                        break;
                    }
                    tokio::task::yield_now().await;
                }
                other => panic!("unexpected admission result: {other:?}"),
            }
        }
    }
}

#[tokio::test]
async fn result_budget_atomicity() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture(root.path()).await;
    let store = owner.handle().plugin_store();
    let first = store.install(&bundle(), &[], &std::collections::BTreeSet::new()).unwrap().installation.install_id;
    let original = std::fs::read(store.path(first)).unwrap();
    let settings = owner.handle().settings().paths().settings();
    let before = std::fs::read(&settings).unwrap();
    let server = Server::start(owner.handle(), keyed().with_max_message_bytes(512)).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let error = client.upload::<contracts::plugins::Install>(authenticated(tokio_stream::iter(frames(&bundle())))).await.unwrap_err();
    assert_eq!(error.code(), Code::ResourceExhausted);
    assert_eq!(std::fs::read(&settings).unwrap(), before);
    assert_eq!(std::fs::read(store.path(first)).unwrap(), original);
    assert_eq!(store.list().unwrap().len(), 1);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn bounded_installed_metadata() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture(root.path()).await;
    let server = Server::start(owner.handle(), keyed().with_max_message_bytes(4096)).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let bytes = bundle_description(&"d".repeat(8192));
    let installed =
        client.upload::<contracts::plugins::Install>(authenticated(tokio_stream::iter(frames(&bytes)))).await.unwrap().into_inner();
    assert_eq!(installed.state, model::plugins::wire::PluginInstallationState::Installed);
    assert!(installed.restart_required);
    assert!(installed.metadata.is_none());
    assert!(installed.error.is_some());
    let entry = client
        .unary::<contracts::plugins::GetInstallation>(authenticated(contracts::plugins::GetInstallation {
            install_id: installed.install_id.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(entry.sha256, installed.sha256);
    assert_eq!(entry.state, model::plugins::wire::PluginInstallationState::Installed);
    assert!(entry.metadata.is_none());
    let removed = client
        .unary::<contracts::plugins::Remove>(authenticated(contracts::plugins::Remove { install_id: installed.install_id }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(removed.installation.unwrap().state, model::plugins::wire::PluginInstallationState::RemovalPending);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */
