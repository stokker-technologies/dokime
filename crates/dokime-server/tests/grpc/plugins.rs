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
use sha2::{Digest, Sha256};
// Local Imports
use super::*;
use dokime_client::contracts::plugins::StagePluginFrameFrame as Frame;

/* === Constants === */
const DYNAMIC_OWNER: &str = "grpc.dynamic-fixture";
const DYNAMIC_POLICY: harmos::operations::OperationPolicy = harmos::operations::OperationPolicy {
    timeout: Duration::from_secs(5),
    attempt_timeout: Duration::from_secs(5),
    retries: harmos::operations::RetryPolicy::Never,
};
const DYNAMIC_CAPABILITY_TOKEN: &str = "fixture-capability-012345678901234567890";
const DYNAMIC_DENIED_TOKEN: &str = "fixture-unlisted-admin-012345678901234567890";
const DYNAMIC_WRONG_OWNER_TOKEN: &str = "fixture-wrong-owner-012345678901234567890";
const DYNAMIC_WRONG_ID_TOKEN: &str = "fixture-wrong-operation-012345678901234567890";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[harmos::message]
#[derive(Clone)]
struct DynamicNumber {
    value: u32,
}

#[harmos::message]
struct DynamicText {
    value: String,
}

#[harmos::operation(request = DynamicText, response = DynamicText,
    id = "fixture.dynamic.unary", version = 1, policy = DYNAMIC_POLICY,
    visibility = harmos::operations::Visibility::Host,
    grpc(package = "fixture.dynamic.v1", service = "Provider", method = "Unary"))]
struct DynamicReplacement;

struct DynamicReplacementHandler {
    calls: Arc<std::sync::atomic::AtomicUsize>,
}

struct DynamicUnary;

struct DynamicUnaryHandler {
    calls: Arc<std::sync::atomic::AtomicUsize>,
}

#[harmos::operation(request = DynamicNumber, response = DynamicNumber, output = DynamicNumber,
    id = "fixture.dynamic.download", version = 1, policy = DYNAMIC_POLICY,
    visibility = harmos::operations::Visibility::Host,
    grpc(package = "fixture.dynamic.v1", service = "Provider", method = "Download"))]
struct DynamicDownload;

struct DynamicDownloadHandler {
    calls: Arc<std::sync::atomic::AtomicUsize>,
}

#[harmos::operation(request = DynamicNumber, response = DynamicNumber, input = DynamicNumber,
    id = "fixture.dynamic.upload", version = 1, policy = DYNAMIC_POLICY,
    visibility = harmos::operations::Visibility::Host,
    grpc(package = "fixture.dynamic.v1", service = "Provider", method = "Upload"))]
struct DynamicUpload;

struct DynamicUploadHandler {
    calls: Arc<std::sync::atomic::AtomicUsize>,
}

#[harmos::operation(request = DynamicNumber, response = DynamicNumber, input = DynamicNumber, output = DynamicNumber,
    id = "fixture.dynamic.duplex", version = 1, policy = DYNAMIC_POLICY,
    visibility = harmos::operations::Visibility::Host,
    grpc(package = "fixture.dynamic.v1", service = "Provider", method = "Duplex"))]
struct DynamicDuplex;

struct DynamicDuplexHandler {
    calls: Arc<std::sync::atomic::AtomicUsize>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl harmos::operations::Operation for DynamicReplacementHandler {
    type Declaration = DynamicReplacement;
    type Resources = std::sync::atomic::AtomicUsize;
    type Error = tonic::Status;
    type Context<'a> = harmos::operations::Context<'a>;
    type Endpoints = ();

    fn new(calls: Arc<Self::Resources>) -> Result<Self, Self::Error> {
        Ok(Self { calls })
    }

    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut DynamicText,
        _: &mut Self::Endpoints,
    ) -> Result<harmos::operations::Response<DynamicText>, Self::Error> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(DynamicText {
            value: request.value.clone(),
        }
        .into())
    }
}

impl harmos::operations::OperationDeclaration for DynamicUnary {
    type Request = DynamicNumber;
    type Response = DynamicNumber;
    const ID: &'static str = "fixture.dynamic.unary";
    const POLICY: harmos::operations::OperationPolicy = DYNAMIC_POLICY;
    const VISIBILITY: harmos::operations::Visibility = harmos::operations::Visibility::Host;

    fn visit_wire<V: harmos::operations::WireVisitor>(visitor: V) -> Option<V::Entry> {
        Some(visitor.unary::<Self>())
    }

    fn grant(request: &DynamicNumber) -> Option<harmos::plugin::contract::Grant> {
        match request.value {
            20 => Some(harmos::plugin::contract::Grant::Egress(
                "fixture.allowed".into(),
            )),
            21 => Some(harmos::plugin::contract::Grant::Egress(
                "fixture.refused".into(),
            )),
            _ => None,
        }
    }
}

impl harmos::operations::WireOperation for DynamicUnary {
    const VERSION: u32 = 1;
    const GRPC: Option<harmos::operations::Grpc> = Some(harmos::operations::Grpc::new(
        "fixture.dynamic.v1",
        "Provider",
        "Unary",
    ));

    fn visit_grpc<V: harmos::operations::GrpcVisitor>(visitor: V) -> Option<V::Entry> {
        Some(visitor.unary::<Self>())
    }
}

impl harmos::operations::Operation for DynamicUnaryHandler {
    type Declaration = DynamicUnary;
    type Resources = std::sync::atomic::AtomicUsize;
    type Error = tonic::Status;
    type Context<'a> = harmos::operations::Context<'a>;
    type Endpoints = ();

    fn new(calls: Arc<Self::Resources>) -> Result<Self, Self::Error> {
        Ok(Self { calls })
    }

    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut DynamicNumber,
        _: &mut Self::Endpoints,
    ) -> Result<harmos::operations::Response<DynamicNumber>, Self::Error> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(DynamicNumber {
            value: request.value + 1,
        }
        .into())
    }
}

impl harmos::operations::Operation for DynamicDownloadHandler {
    type Declaration = DynamicDownload;
    type Resources = std::sync::atomic::AtomicUsize;
    type Error = tonic::Status;
    type Context<'a> = harmos::operations::Context<'a>;
    type Endpoints = harmos::operations::StreamOutput<DynamicNumber>;

    fn new(calls: Arc<Self::Resources>) -> Result<Self, Self::Error> {
        Ok(Self { calls })
    }

    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut DynamicNumber,
        streams: &mut Self::Endpoints,
    ) -> Result<harmos::operations::Response<DynamicNumber>, Self::Error> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        for value in [request.value + 1, request.value + 2] {
            streams
                .send_with(std::mem::size_of::<DynamicNumber>(), || {
                    Ok(DynamicNumber { value })
                })
                .await
                .map_err(|error| tonic::Status::internal(error.to_string()))?;
        }
        Ok(DynamicNumber {
            value: request.value + 3,
        }
        .into())
    }
}

impl harmos::operations::Operation for DynamicUploadHandler {
    type Declaration = DynamicUpload;
    type Resources = std::sync::atomic::AtomicUsize;
    type Error = tonic::Status;
    type Context<'a> = harmos::operations::Context<'a>;
    type Endpoints = harmos::operations::StreamInput<DynamicNumber>;

    fn new(calls: Arc<Self::Resources>) -> Result<Self, Self::Error> {
        Ok(Self { calls })
    }

    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut DynamicNumber,
        streams: &mut Self::Endpoints,
    ) -> Result<harmos::operations::Response<DynamicNumber>, Self::Error> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut value = request.value;
        while let Some(item) = streams.recv().await {
            value += item.value;
        }
        Ok(DynamicNumber { value }.into())
    }
}

impl harmos::operations::Operation for DynamicDuplexHandler {
    type Declaration = DynamicDuplex;
    type Resources = std::sync::atomic::AtomicUsize;
    type Error = tonic::Status;
    type Context<'a> = harmos::operations::Context<'a>;
    type Endpoints = (
        harmos::operations::StreamInput<DynamicNumber>,
        harmos::operations::StreamOutput<DynamicNumber>,
    );

    fn new(calls: Arc<Self::Resources>) -> Result<Self, Self::Error> {
        Ok(Self { calls })
    }

    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut DynamicNumber,
        streams: &mut Self::Endpoints,
    ) -> Result<harmos::operations::Response<DynamicNumber>, Self::Error> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let mut value = request.value;
        while let Some(item) = streams.0.recv().await {
            value += item.value;
            streams
                .1
                .send_with(std::mem::size_of::<DynamicNumber>(), || {
                    Ok(DynamicNumber { value })
                })
                .await
                .map_err(|error| tonic::Status::internal(error.to_string()))?;
        }
        Ok(DynamicNumber { value }.into())
    }
}

impl contracts::Endpoint for DynamicUnary {
    const PATH: &'static str = "/fixture.dynamic.v1.Provider/Unary";
    const ACCESS: Role = Role::Read;
}

/* === Public Functions === */

/* === Private Functions === */
fn bundle() -> Vec<u8> {
    bundle_description("Fixture")
}

fn bundle_description(description: &str) -> Vec<u8> {
    use harmos::plugin::bundle::{Bundle, Executable, Facility, Identity, Limits, Source};
    let owner = "grpc-plugin";
    let version = "1.0.0";
    let declaration = serde_json::json!({"kind":"plugin","facility":"sidecar","id":owner,"version":version,"description":"A fixture.","config":"()"});
    let policy = harmos::operations::OperationPolicy {
        timeout: std::time::Duration::from_secs(5),
        attempt_timeout: std::time::Duration::from_secs(5),
        retries: harmos::operations::RetryPolicy::Never,
    };
    let lifecycle = [
        harmos::operations::lifecycle::Descriptor::new(
            harmos::operations::lifecycle::Action::Initialize,
            policy,
        )
        .unwrap(),
        harmos::operations::lifecycle::Descriptor::new(
            harmos::operations::lifecycle::Action::Shutdown,
            policy,
        )
        .unwrap(),
    ];
    let contracts = serde_json::json!({"kind":"contracts","facility":"sidecar","owner":owner,"provides":[],"requires":[],"lifecycle":lifecycle});
    let contributions = serde_json::json!({"kind":"contributions","facility":"sidecar","owner":owner,"sections":[]});
    let restart = serde_json::json!({"kind":"restart","facility":"sidecar","owner":owner,"policy":{"credit":3,"sustained_health_ms":30000,"backoff_base_ms":250,"backoff_cap_ms":10000}});
    let publications =
        serde_json::json!({"kind":"publications","facility":"sidecar","owner":owner,"streams":[]});
    let observers =
        serde_json::json!({"kind":"observers","facility":"sidecar","owner":owner,"observers":[]});
    let executable = format!(
        "inert{prefix}{declaration}\0{prefix}{contracts}\0{prefix}{contributions}\0{prefix}{restart}\0{prefix}{publications}\0{prefix}{observers}\0",
        prefix = std::str::from_utf8(harmos_contract::PREFIX).unwrap()
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
                target: dokime_app::plugins::NATIVE_TARGET.into(),
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
    let mut config = RuntimeConfig::new(
        root,
        Metadata::new(Identifier::new("plugins.fixture").unwrap(), "Plugins", None),
    );
    config.installed_plugins.push(dokime_app::InstalledPlugin {
        source: "grpc.host".into(),
        version: "1.0.0".into(),
        enabled: true,
        ..Default::default()
    });
    Runtime::start(config).await.unwrap()
}

fn frames(bytes: &[u8]) -> Vec<contracts::plugins::StagePluginFrame> {
    let mut frames = vec![contracts::plugins::StagePluginFrame {
        frame: Some(Frame::Header(contracts::plugins::StagePluginHeader {
            size_bytes: bytes.len() as u64,
            sha256: Sha256::digest(bytes)
                .iter()
                .map(|byte| format!("{byte:02x}"))
                .collect::<String>(),
        })),
    }];
    for (index, chunk) in bytes.chunks(37).enumerate() {
        frames.push(contracts::plugins::StagePluginFrame {
            frame: Some(Frame::Chunk(contracts::plugins::StagePluginChunk {
                offset: (index * 37) as u64,
                data: chunk.to_vec(),
            })),
        });
    }
    frames
}

pub(crate) async fn plugin_stage_identity() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture(root.path()).await;
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Admin)])
            .unwrap(),
    )
    .await
    .unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    assert_eq!(
        client
            .unary::<contracts::plugins::ListInstallations>(
                contracts::plugins::ListInstallationsRequest::default()
            )
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    let staged = client
        .upload::<contracts::plugins::Stage>(authenticated(tokio_stream::iter(frames(&bundle()))))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        staged.state,
        contracts::plugins::PluginInstallationState::Staged
    );
    assert!(!staged.active);
    assert!(!staged.restart_required);
    assert_eq!(staged.metadata.as_ref().unwrap().owners, ["grpc-plugin"]);
    assert_eq!(
        staged
            .metadata
            .as_ref()
            .unwrap()
            .identity
            .as_ref()
            .unwrap()
            .name,
        "grpc-fixture"
    );
    assert_eq!(
        client
            .unary::<contracts::plugins::Consent>(authenticated(
                contracts::plugins::ConsentPluginRequest {
                    install_id: staged.install_id.clone(),
                    expected_sha256: "0".repeat(64)
                }
            ))
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition
    );
    let installed = client
        .unary::<contracts::plugins::Consent>(authenticated(
            contracts::plugins::ConsentPluginRequest {
                install_id: staged.install_id.clone(),
                expected_sha256: staged.sha256.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        installed.state,
        contracts::plugins::PluginInstallationState::Installed
    );
    assert!(installed.consented_at.is_some());
    assert!(installed.restart_required);
    assert!(!installed.active);
    let disabled = client
        .unary::<contracts::plugins::SetEnabled>(authenticated(
            contracts::plugins::SetPluginEnabledRequest {
                install_id: staged.install_id.clone(),
                enabled: false,
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        disabled.state,
        contracts::plugins::PluginInstallationState::Disabled
    );
    let page = client
        .unary::<contracts::plugins::ListInstallations>(authenticated(
            contracts::plugins::ListInstallationsRequest::default(),
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(page.total_count, 1);
    assert_eq!(page.installations[0].install_id, staged.install_id);
    let removed = client
        .unary::<contracts::plugins::Remove>(authenticated(
            contracts::plugins::RemovePluginRequest {
                install_id: staged.install_id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        removed.installation.unwrap().state,
        contracts::plugins::PluginInstallationState::RemovalPending
    );
    let current = client
        .unary::<contracts::plugins::GetInstallation>(authenticated(
            contracts::plugins::GetInstallationRequest {
                install_id: staged.install_id,
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert!(current.restart_required);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
    dynamic_plugin_cardinalities().await;
}

async fn dynamic_plugin_cardinalities() {
    use dokime_client::{Client, Target, dynamic::Event};
    use dokime_server::grpc::access::OperationPermission;
    use harmos::operations::OperationDeclaration;
    use std::sync::atomic::{AtomicUsize, Ordering};
    let permission = |owner: &str, id: &str| OperationPermission {
        owner: owner.into(),
        id: id.into(),
        grants: vec![],
    };
    let operation_ids = [
        DynamicUnary::ID,
        DynamicDownload::ID,
        DynamicUpload::ID,
        DynamicDuplex::ID,
    ];
    let mut capability = permission(DYNAMIC_OWNER, DynamicUnary::ID);
    capability
        .grants
        .push(harmos::plugin::contract::Grant::Egress(
            "fixture.allowed".into(),
        ));
    let grants = vec![
        BearerGrant::new(
            "conditional-capability".into(),
            Role::Read,
            DYNAMIC_CAPABILITY_TOKEN,
        )
        .unwrap()
        .with_operations(vec![capability])
        .unwrap(),
        grant(Role::Read)
            .with_operations(
                operation_ids
                    .iter()
                    .map(|id| permission(DYNAMIC_OWNER, id))
                    .collect(),
            )
            .unwrap(),
        BearerGrant::new("unlisted-admin".into(), Role::Admin, DYNAMIC_DENIED_TOKEN).unwrap(),
        BearerGrant::new("wrong-owner".into(), Role::Admin, DYNAMIC_WRONG_OWNER_TOKEN)
            .unwrap()
            .with_operations(vec![permission("grpc.other", DynamicUnary::ID)])
            .unwrap(),
        BearerGrant::new(
            "wrong-operation".into(),
            Role::Admin,
            DYNAMIC_WRONG_ID_TOKEN,
        )
        .unwrap()
        .with_operations(vec![permission(DYNAMIC_OWNER, "fixture.dynamic.unlisted")])
        .unwrap(),
    ];
    for invalid in [
        permission("*", DynamicUnary::ID),
        permission(DYNAMIC_OWNER, "*"),
        permission("dokime", DynamicUnary::ID),
        permission("harmos", DynamicUnary::ID),
    ] {
        assert!(grant(Role::Admin).with_operations(vec![invalid]).is_err());
    }
    assert!(
        grant(Role::Admin)
            .with_operations(vec![
                permission(DYNAMIC_OWNER, DynamicUnary::ID),
                permission(DYNAMIC_OWNER, DynamicUnary::ID),
            ])
            .is_err()
    );
    let mut wildcard = permission(DYNAMIC_OWNER, DynamicUnary::ID);
    wildcard
        .grants
        .push(harmos::plugin::contract::Grant::Egress("*".into()));
    assert!(grant(Role::Admin).with_operations(vec![wildcard]).is_err());
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture(root.path()).await;
    let runtime = owner.handle();
    let calls = Arc::new(AtomicUsize::new(0));
    let server = Server::start(
        runtime.clone(),
        Config::local().with_bearer_auth(grants).unwrap(),
    )
    .await
    .unwrap();
    let allowed = Client::connect(
        Target::local(server.local_endpoint()).unwrap(),
        Some(TOKEN),
        None,
    )
    .await
    .unwrap();
    assert!(
        !allowed
            .operations()
            .await
            .unwrap()
            .iter()
            .any(|entry| entry.owner.as_str() == DYNAMIC_OWNER)
    );

    // Register After Connection To Require Live Discovery, Not A Client Dispatch Table.
    runtime
        .operations()
        .register_wire_operation::<DynamicUnaryHandler>(
            DYNAMIC_OWNER,
            dokime_app::app::Lanes::Open,
            calls.clone(),
        )
        .unwrap();
    runtime
        .operations()
        .register_wire_operation::<DynamicDownloadHandler>(
            DYNAMIC_OWNER,
            dokime_app::app::Lanes::Open,
            calls.clone(),
        )
        .unwrap();
    runtime
        .operations()
        .register_wire_operation::<DynamicUploadHandler>(
            DYNAMIC_OWNER,
            dokime_app::app::Lanes::Open,
            calls.clone(),
        )
        .unwrap();
    runtime
        .operations()
        .register_wire_operation::<DynamicDuplexHandler>(
            DYNAMIC_OWNER,
            dokime_app::app::Lanes::Open,
            calls.clone(),
        )
        .unwrap();
    let catalog = allowed.operations().await.unwrap();
    for id in [
        DynamicUnary::ID,
        DynamicDownload::ID,
        DynamicUpload::ID,
        DynamicDuplex::ID,
    ] {
        assert!(
            catalog
                .iter()
                .any(|entry| entry.owner.as_str() == DYNAMIC_OWNER
                    && entry.contract.operation().id.as_str() == id
                    && entry.grpc_available)
        );
    }

    // An Admin Role Alone Must Grant Neither Discovery Nor Direct Invocation.
    let direct = WireClient::connect(server.local_endpoint()).await.unwrap();
    for token in [
        DYNAMIC_DENIED_TOKEN,
        DYNAMIC_WRONG_OWNER_TOKEN,
        DYNAMIC_WRONG_ID_TOKEN,
    ] {
        let denied = Client::connect(
            Target::local(server.local_endpoint()).unwrap(),
            Some(token),
            None,
        )
        .await
        .unwrap();
        assert!(
            !denied
                .operations()
                .await
                .unwrap()
                .iter()
                .any(|entry| entry.owner.as_str() == DYNAMIC_OWNER)
        );
        assert!(
            denied
                .call_operation(
                    DynamicUnary::ID,
                    Some(DYNAMIC_OWNER),
                    b"{\"value\":10}",
                    vec![]
                )
                .await
                .is_err()
        );
        let mut request = Request::new(DynamicNumber { value: 10 });
        request
            .metadata_mut()
            .insert("authorization", format!("Bearer {token}").parse().unwrap());
        assert_eq!(
            direct
                .unary::<DynamicUnary>(request)
                .await
                .unwrap_err()
                .code(),
            Code::PermissionDenied
        );
        denied.close_and_wait().await;
    }
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    // A Different Request Schema Must Fail Before Handler Mutation.
    let malformed = authenticated(harmos::operations::Response::new(DynamicText {
        value: "wrong-schema".into(),
    }));
    let error = direct
        .grpc()
        .await
        .unwrap()
        .unary(
            malformed,
            <DynamicUnary as contracts::Endpoint>::PATH.parse().unwrap(),
            direct.codec::<DynamicNumber, DynamicText>().unwrap(),
        )
        .await
        .err()
        .expect("wrong request schema is refused");
    assert_eq!(error.code(), Code::InvalidArgument);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    // Explicit Plugin Permissions Do Not Elevate The Token's Product Role.
    assert_eq!(
        direct
            .unary::<contracts::execution::CreateCycle>(authenticated(
                contracts::execution::CreateCycleRequest::default(),
            ))
            .await
            .unwrap_err()
            .code(),
        Code::PermissionDenied
    );
    assert!(runtime.cycles().await.is_empty());

    for (id, input, expected, completion) in [
        (DynamicUnary::ID, false, vec![], 11),
        (DynamicDownload::ID, false, vec![11, 12], 13),
        (DynamicUpload::ID, true, vec![], 16),
        (DynamicDuplex::ID, true, vec![11, 13, 16], 16),
    ] {
        let inputs = if input {
            [1, 2, 3]
                .into_iter()
                .map(|value| serde_json::to_vec(&serde_json::json!({"value": value})).unwrap())
                .collect()
        } else {
            vec![]
        };
        let stream = allowed
            .call_operation(id, Some(DYNAMIC_OWNER), b"{\"value\":10}", inputs)
            .await
            .unwrap();
        let mut received = Vec::new();
        let mut finished = false;
        tokio::time::timeout(Duration::from_secs(5), async {
            while let Some(event) = stream.message().await.unwrap() {
                assert!(!finished, "completion must be the final event for {id}");
                match event {
                    Event::Item(value) => {
                        received.push(value["value"].as_u64().expect("numeric fixture item"))
                    }
                    Event::Completion(value) => {
                        assert_eq!(value["value"], completion, "{id}");
                        finished = true;
                    }
                }
            }
        })
        .await
        .expect("dynamic plugin call settles within its policy");
        assert_eq!(received, expected, "{id}");
        assert!(finished, "{id} needs completion and final OK status");
        stream.close().await;
    }
    assert_eq!(calls.load(Ordering::SeqCst), 4);
    // Request-Dependent Grants Must Be Checked Before The Handler Mutates Anything.
    let mut no_grant = Request::new(DynamicNumber { value: 20 });
    no_grant
        .metadata_mut()
        .insert("authorization", format!("Bearer {TOKEN}").parse().unwrap());
    assert_eq!(
        direct
            .unary::<DynamicUnary>(no_grant)
            .await
            .unwrap_err()
            .code(),
        Code::PermissionDenied
    );
    let mut wrong_grant = Request::new(DynamicNumber { value: 21 });
    wrong_grant.metadata_mut().insert(
        "authorization",
        format!("Bearer {DYNAMIC_CAPABILITY_TOKEN}")
            .parse()
            .unwrap(),
    );
    assert_eq!(
        direct
            .unary::<DynamicUnary>(wrong_grant)
            .await
            .unwrap_err()
            .code(),
        Code::PermissionDenied
    );
    assert_eq!(calls.load(Ordering::SeqCst), 4);
    let mut matched = Request::new(DynamicNumber { value: 20 });
    matched.metadata_mut().insert(
        "authorization",
        format!("Bearer {DYNAMIC_CAPABILITY_TOKEN}")
            .parse()
            .unwrap(),
    );
    assert_eq!(
        direct
            .unary::<DynamicUnary>(matched)
            .await
            .unwrap()
            .into_inner()
            .value,
        21
    );
    assert_eq!(calls.load(Ordering::SeqCst), 5);

    // Reusing An ID Must Publish The New Generation And Its Entire Schema.
    let original = catalog
        .iter()
        .find(|entry| {
            entry.owner.as_str() == DYNAMIC_OWNER
                && entry.contract.operation().id.as_str() == DynamicUnary::ID
        })
        .unwrap();
    runtime
        .operations()
        .unregister_from(DYNAMIC_OWNER, DynamicUnary::ID)
        .await
        .unwrap();
    assert!(
        !allowed
            .operations()
            .await
            .unwrap()
            .iter()
            .any(|entry| entry.owner.as_str() == DYNAMIC_OWNER
                && entry.contract.operation().id.as_str() == DynamicUnary::ID)
    );
    runtime
        .operations()
        .register_wire_operation::<DynamicReplacementHandler>(
            DYNAMIC_OWNER,
            dokime_app::app::Lanes::Open,
            calls.clone(),
        )
        .unwrap();
    let replaced = allowed.operations().await.unwrap();
    let replacement = replaced
        .iter()
        .find(|entry| {
            entry.owner.as_str() == DYNAMIC_OWNER
                && entry.contract.operation().id.as_str() == DynamicUnary::ID
        })
        .unwrap();
    assert_ne!(replacement.generation, original.generation);
    assert!(!replacement.contract.compatible(&original.contract));
    assert!(!original.contract.compatible(&replacement.contract));
    assert_eq!(
        direct
            .unary::<DynamicUnary>(authenticated(DynamicNumber { value: 10 }))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    assert!(
        allowed
            .call_operation(
                DynamicUnary::ID,
                Some(DYNAMIC_OWNER),
                b"{\"value\":10}",
                vec![]
            )
            .await
            .is_err()
    );
    assert_eq!(
        calls.load(Ordering::SeqCst),
        5,
        "stale requests must not enter the replacement"
    );
    let replacement = allowed
        .call_operation(
            DynamicUnary::ID,
            Some(DYNAMIC_OWNER),
            b"{\"value\":\"replacement\"}",
            vec![],
        )
        .await
        .unwrap();
    let Some(Event::Completion(value)) = replacement.message().await.unwrap() else {
        panic!("replacement unary call requires completion");
    };
    assert_eq!(value["value"], "replacement");
    assert!(replacement.message().await.unwrap().is_none());
    replacement.close().await;
    assert_eq!(calls.load(Ordering::SeqCst), 6);

    allowed.close_and_wait().await;
    server.shutdown().await.unwrap();
    drop(runtime);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn invalid_uploads_mutate() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture(root.path()).await;
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Admin)])
            .unwrap(),
    )
    .await
    .unwrap();
    let mut client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let mut bad = frames(&bundle());
    if let Some(Frame::Chunk(chunk)) = &mut bad[1].frame {
        chunk.offset = 1;
    }
    assert_eq!(
        client
            .upload::<contracts::plugins::Stage>(authenticated(tokio_stream::iter(bad)))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    let mut short = frames(&bundle());
    short.pop();
    assert_eq!(
        client
            .upload::<contracts::plugins::Stage>(authenticated(tokio_stream::iter(short)))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    assert_eq!(
        client
            .unary::<contracts::plugins::ListInstallations>(authenticated(
                contracts::plugins::ListInstallationsRequest::default()
            ))
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
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            match client
                .unary::<contracts::plugins::ListInstallations>(authenticated(
                    contracts::plugins::ListInstallationsRequest::default(),
                ))
                .await
            {
                Ok(page) => {
                    assert_eq!(page.into_inner().total_count, 0);
                    break;
                }
                Err(error) if error.code() == Code::ResourceExhausted => {
                    tokio::task::yield_now().await
                }
                other => panic!("unexpected cancellation result: {other:?}"),
            }
        }
    })
    .await
    .unwrap();
    server.shutdown().await.unwrap();
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Read)])
            .unwrap(),
    )
    .await
    .unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    assert_eq!(
        client
            .unary::<contracts::plugins::ListInstallations>(authenticated(
                contracts::plugins::ListInstallationsRequest::default()
            ))
            .await
            .unwrap_err()
            .code(),
        Code::PermissionDenied
    );
    let inventory = client
        .unary::<contracts::plugins::ListAdmittedPlugins>(authenticated(
            contracts::plugins::ListAdmittedPluginsRequest::default(),
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(inventory.plugins.len(), 1);
    assert_eq!(inventory.plugins[0].plugin_id, "grpc.host");
    assert_eq!(
        inventory.plugins[0].owner,
        Some(contracts::plugins::AdmittedPluginOwner::Host(true))
    );
    assert_eq!(
        client
            .upload::<contracts::plugins::Stage>(authenticated(tokio_stream::iter(frames(
                &bundle()
            ))))
            .await
            .unwrap_err()
            .code(),
        Code::PermissionDenied
    );
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

pub(crate) async fn listener_shutdown_admission() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture(root.path()).await;
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Admin)])
            .unwrap(),
    )
    .await
    .unwrap();
    let mut client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let (sender, upload) = open_upload(&mut client).await;
    tokio::time::timeout(Duration::from_secs(2), server.shutdown())
        .await
        .unwrap()
        .unwrap();
    let error = tokio::time::timeout(Duration::from_secs(2), upload)
        .await
        .unwrap()
        .unwrap()
        .unwrap_err();
    assert_eq!(error.code(), Code::Unavailable);
    drop(sender);
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Admin)])
            .unwrap(),
    )
    .await
    .unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    assert_eq!(
        client
            .unary::<contracts::plugins::ListInstallations>(authenticated(
                contracts::plugins::ListInstallationsRequest::default()
            ))
            .await
            .unwrap()
            .into_inner()
            .total_count,
        0
    );
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

// A competing query can win the sole permit before the upload arrives. Retry
// only that explicit pre-admission Busy outcome, never an uncertain mutation.
async fn open_upload(
    client: &mut WireClient,
) -> (
    tokio::sync::mpsc::Sender<contracts::plugins::StagePluginFrame>,
    tokio::task::JoinHandle<
        Result<tonic::Response<contracts::plugins::PluginInstallation>, tonic::Status>,
    >,
) {
    tokio::time::timeout(Duration::from_secs(3), async {
        loop {
            let (sender, receiver) = tokio::sync::mpsc::channel(1);
            sender.send(frames(&bundle()).remove(0)).await.unwrap();
            let uploader = client.clone();
            let upload = tokio::spawn(async move {
                uploader
                    .upload::<contracts::plugins::Stage>(authenticated(
                        tokio_stream::wrappers::ReceiverStream::new(receiver),
                    ))
                    .await
            });
            // Let the streaming RPC submit its request before the admission
            // probe; otherwise this task wins the sole permit every time.
            tokio::task::yield_now().await;
            loop {
                match client
                    .unary::<contracts::plugins::ListInstallations>(authenticated(
                        contracts::plugins::ListInstallationsRequest::default(),
                    ))
                    .await
                {
                    Err(error) if error.code() == Code::ResourceExhausted => {
                        return (sender, upload);
                    }
                    Ok(_) => {
                        if upload.is_finished() {
                            assert_eq!(
                                upload.await.unwrap().unwrap_err().code(),
                                Code::ResourceExhausted
                            );
                            break;
                        }
                        tokio::time::sleep(Duration::from_millis(1)).await;
                    }
                    other => panic!("unexpected admission result: {other:?}"),
                }
            }
        }
    })
    .await
    .unwrap()
}

pub(crate) async fn oversized_permissions_consent() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture(root.path()).await;
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_max_message_bytes(4096)
            .with_bearer_auth(vec![grant(Role::Admin)])
            .unwrap(),
    )
    .await
    .unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let bytes = bundle_description(&"d".repeat(8192));
    let staged = client
        .upload::<contracts::plugins::Stage>(authenticated(tokio_stream::iter(frames(&bytes))))
        .await
        .unwrap()
        .into_inner();
    assert!(staged.metadata.is_none());
    assert!(staged.error.as_ref().unwrap().contains("budget"));
    let page = client
        .unary::<contracts::plugins::ListInstallations>(authenticated(
            contracts::plugins::ListInstallationsRequest::default(),
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(page.installations[0].install_id, staged.install_id);
    let entry = client
        .unary::<contracts::plugins::GetInstallation>(authenticated(
            contracts::plugins::GetInstallationRequest {
                install_id: staged.install_id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(entry.sha256, staged.sha256);
    assert!(entry.metadata.is_none());
    assert_eq!(
        client
            .unary::<contracts::plugins::Consent>(authenticated(
                contracts::plugins::ConsentPluginRequest {
                    install_id: staged.install_id.clone(),
                    expected_sha256: staged.sha256
                }
            ))
            .await
            .unwrap_err()
            .code(),
        Code::ResourceExhausted
    );
    let entry = client
        .unary::<contracts::plugins::GetInstallation>(authenticated(
            contracts::plugins::GetInstallationRequest {
                install_id: staged.install_id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        entry.state,
        contracts::plugins::PluginInstallationState::Staged
    );
    assert!(entry.consented_at.is_none());
    assert!(
        client
            .unary::<contracts::plugins::Remove>(authenticated(
                contracts::plugins::RemovePluginRequest {
                    install_id: staged.install_id
                }
            ))
            .await
            .unwrap()
            .into_inner()
            .installation
            .is_none()
    );
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */
