/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// The desktop RPC bridge exercised against a live server: unary, streaming and artifact upload frames.

/* === Modules === */
#[path = "../support/rpc_artifact.rs"]
pub(crate) mod artifact_fixture;
pub(crate) mod artifact_upload;

#[path = "../support/bridge.rs"]
pub(crate) mod bridge;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::sync::Arc;
use std::{
    process::{Command, Stdio},
    time::Duration,
};
// Third Party Imports
use harmos::Protobuf;
use serde_json::json;
use sha2::Digest;
// Local Imports
use bridge::{Harness, Local, fenced};
use dokime_app::{
    Runtime, RuntimeConfig,
    core::{Identifier, Metadata},
};
use dokime_contract::operations::{backend, execution, plugins, value};
use dokime_gui::{
    connection::ConnectionSession,
    rpc::{Bridge, FRAME_END, FRAME_ERROR, FRAME_MESSAGE, MAX_FRAME_BYTES},
};
use dokime_server::grpc::{Config, Server};

/* === Constants === */
const STAGE: &str = "/dokime.v1.Plugins/Stage";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Fixture {
    pub session: ConnectionSession,
    server: Server,
    runtime: Runtime,
    _root: tempfile::TempDir,
    artifacts: Arc<artifact_fixture::Provider>,
}

struct StageInput {
    entered: Option<tokio::sync::oneshot::Sender<()>>,
    dropped: Option<tokio::sync::oneshot::Sender<()>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl tokio_stream::Stream for StageInput {
    type Item = plugins::StagePluginFrame;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let Some(entered) = self.entered.take() else {
            return std::task::Poll::Pending;
        };
        entered.send(()).unwrap();

        std::task::Poll::Ready(Some(plugins::StagePluginFrame {
            frame: Some(plugins::StagePluginFrameFrame::Header(
                plugins::StagePluginHeader {
                    size_bytes: 1,
                    sha256: "00".repeat(32),
                },
            )),
        }))
    }
}

impl Drop for StageInput {
    fn drop(&mut self) {
        if let Some(dropped) = self.dropped.take() {
            let _ = dropped.send(());
        }
    }
}

impl Fixture {
    pub async fn new() -> Self {
        Self::with_role(dokime_server::grpc::access::Role::Control).await
    }
    pub async fn with_role(role: dokime_server::grpc::access::Role) -> Self {
        let root = tempfile::tempdir().unwrap();
        let artifacts = Arc::new(artifact_fixture::Provider::default());
        let runtime = Runtime::with_execution_fixture(
            RuntimeConfig::new(
                root.path(),
                Metadata::new(Identifier::new("rpc.fixture").unwrap(), "RPC fixture", None),
            ),
            "fixture",
            artifacts.clone(),
        )
        .await
        .unwrap();
        runtime
            .handle()
            .publish_product_record(
                dokime_app::records::ProductCalibrationDetails::new(
                    "rpc-device",
                    dokime_app::records::ProductCalibrationKind::Field,
                    dokime_app::records::ProductCalibrationPhase::Completed,
                )
                .into_draft("rpc.fixture"),
            )
            .await
            .unwrap();
        let server = Server::start(
            runtime.handle(),
            Config::local()
                .with_bearer_auth(vec![
                    dokime_server::grpc::access::BearerGrant::new(
                        "native-gui".into(),
                        role,
                        "rpc-test-token-012345678901234567890123456789",
                    )
                    .unwrap(),
                ])
                .unwrap(),
        )
        .await
        .unwrap();
        let session = ConnectionSession::default();
        session
            .connect(dokime_gui::connection::ConnectRequest {
                target: dokime_gui::connection::ConnectionTarget::Local {
                    endpoint: server.local_endpoint(),
                },
                credentials: dokime_gui::connection::ConnectionCredentials::Bearer {
                    token: "rpc-test-token-012345678901234567890123456789".into(),
                    remember: false,
                },
                expected_backend_id: None,
            })
            .await
            .unwrap();
        Self {
            session,
            server,
            runtime,
            _root: root,
            artifacts,
        }
    }
    pub fn generation(&self) -> u64 {
        self.session.status().generation.parse().unwrap()
    }

    pub async fn close(mut self) {
        self.session.disconnect().unwrap();
        self.server.shutdown().await.unwrap();
        self.runtime.shutdown().await.unwrap();
    }
}

/* === Public Functions === */
pub(crate) async fn typescript_facade_bridge() {
    let fixture = Fixture::new().await;
    fixture.runtime.handle().set_brand(Some(
        dokime_app::domains::installed_plugins::load_brand(
            &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../branding"),
        )
        .unwrap(),
    ));
    let artifact_id = artifact_fixture::seed(&fixture.runtime.handle(), &fixture.artifacts).await;
    let harness = Harness::new(fixture.session.clone());
    let snapshot = fixture.session.status();
    let mut child = Command::new("node")
        .env("DOKIME_TEST_GENERATION", &snapshot.generation)
        .env(
            "DOKIME_TEST_BACKEND_ID",
            &snapshot.backend.as_ref().unwrap().backend_id,
        )
        .env("DOKIME_TEST_ARTIFACT_ID", artifact_id)
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/rpc/fixtures/test-rpc-interop.mjs"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap();
    let status = harness.drive(
        &mut child,
        |command, bytes| match command {
            "fixture_cancel_streams" => {
                use tauri::Manager;
                harness.app.state::<Bridge>().cancel_owner("main");
                Local::Reply(json!({"value": null}))
            }
            // These fixtures exercise TS-generated message semantics against canonical Harmos types.
            "fixture_value" => {
                let value = value::Value::decode(bytes).unwrap();
                assert_eq!(value.nodes.len(), 5);
                let Some(value::ValueKind::Object(fields)) = &value.nodes[0].kind else {
                    panic!("the fixture value has an object root");
                };
                assert_eq!(
                    fields
                        .fields
                        .iter()
                        .map(|field| (field.key.as_str(), field.value))
                        .collect::<Vec<_>>(),
                    vec![("minimum", 1), ("maximum", 2), ("null", 3), ("unset", 4)]
                );
                assert!(matches!(
                    value.nodes[1].kind,
                    Some(value::ValueKind::Integer(i64::MIN))
                ));
                assert!(matches!(
                    value.nodes[2].kind,
                    Some(value::ValueKind::Integer(i64::MAX))
                ));
                assert!(matches!(value.nodes[3].kind, Some(value::ValueKind::Null)));
                assert!(value.nodes[4].kind.is_none());
                Local::Reply(json!({ "bytes": base64(&value.encode_to_vec()) }))
            }
            "fixture_admission" => {
                let value = execution::Admission::decode(bytes).unwrap();
                assert_eq!(value.cursor.as_ref().unwrap().position, u64::MAX);
                assert_eq!(value.accepted_at.as_ref().unwrap().seconds, -1);
                assert_eq!(value.accepted_at.as_ref().unwrap().nanos, 999_999_999);
                Local::Reply(json!({ "bytes": base64(&value.encode_to_vec()) }))
            }
            _ => Local::Dispatch,
        },
        |line| panic!("unexpected observation {line}"),
    );
    assert!(status.success(), "TypeScript/native interop failed");
    fixture.close().await;
}

pub(crate) async fn refuses_invalid_dispatch() {
    let fixture = Fixture::with_role(dokime_server::grpc::access::Role::Admin).await;
    let harness = Harness::new(fixture.session.clone());
    let generation = fixture.generation();
    for command in ["rpc", "rpc_stream_open"] {
        for (bytes, kind) in [
            (vec![], "invalid_request"),
            (vec![0; 40], "incompatible_descriptor"),
            (vec![0; MAX_FRAME_BYTES + 40 + 1], "limit_exceeded"),
        ] {
            assert_eq!(
                invoke(&harness.window, command, bytes).unwrap_err()["kind"],
                kind
            );
        }
        for path in [None, Some("dokime.v1.Backend/GetHealth"), Some("/a/b/c")] {
            let headers = path.map_or(json!({}), |path| json!({"dokime-rpc-path": path}));
            assert_eq!(
                bridge::invoke(&harness.window, command, fenced(generation, &[]), &headers)
                    .unwrap_err()["kind"],
                "invalid_request",
                "{command} {path:?}"
            );
        }
    }
    // A stream needs a channel to deliver to.
    assert_eq!(
        bridge::invoke(
            &harness.window,
            "rpc_stream_open",
            fenced(generation, &[]),
            &json!({"dokime-rpc-path": "/dokime.v1.Backend/GetHealth"}),
        )
        .unwrap_err()["kind"],
        "invalid_request"
    );
    // A superseded generation is refused before anything reaches the backend.
    assert_eq!(
        bridge::invoke(
            &harness.window,
            "rpc",
            fenced(generation + 1, &[]),
            &json!({"dokime-rpc-path": "/dokime.v1.Backend/GetHealth"}),
        )
        .unwrap_err()["detail"]["kind"],
        "superseded"
    );
    for command in ["rpc_stream_send", "rpc_stream_finish", "rpc_stream_cancel"] {
        assert_eq!(
            harness
                .control(command, generation, 99, &[], json!({}))
                .unwrap_err()["kind"],
            "unknown_stream"
        );
    }
    let (stream, channel) = harness
        .open(generation, STAGE, &envelope(1, &[]), true)
        .unwrap();
    for credit in ["0", "33", "x", "1"] {
        assert_eq!(
            harness
                .control(
                    "rpc_stream_ack",
                    generation,
                    stream,
                    &[],
                    json!({"dokime-rpc-credit": credit}),
                )
                .unwrap_err()["kind"],
            "invalid_ack",
            "credit {credit} exceeds what was delivered"
        );
    }
    harness
        .control("rpc_stream_cancel", generation, stream, &[], json!({}))
        .unwrap();
    let frames = harness.settle(channel, Duration::from_secs(10));
    assert_eq!(frames.last().unwrap()[8], FRAME_ERROR);
    fixture.close().await;
}

pub(crate) async fn unary_bridge() {
    let fixture = Fixture::new().await;
    let harness = Harness::new(fixture.session.clone());
    let generation = fixture.generation();
    let reply = bridge::invoke(
        &harness.window,
        "rpc",
        fenced(generation, &backend::GetIdentityRequest {}.encode_to_vec()),
        &json!({"dokime-rpc-path": "/dokime.v1.Backend/GetIdentity"}),
    )
    .unwrap();
    let tauri::ipc::InvokeResponseBody::Raw(reply) = reply else {
        panic!("a unary reply is raw bytes");
    };
    assert_eq!(
        u64::from_le_bytes(reply[..8].try_into().unwrap()),
        generation
    );
    let identity = backend::BackendIdentity::decode(&reply[8..]).unwrap();
    assert_eq!(
        identity.backend_id,
        fixture.session.status().backend.unwrap().backend_id
    );
    // A typed backend status crosses unchanged.
    let missing = bridge::invoke(
        &harness.window,
        "rpc",
        fenced(generation, &[]),
        &json!({"dokime-rpc-path": "/dokime.v1.Missing/Call"}),
    )
    .unwrap_err();
    assert_eq!(missing["kind"], "grpc");
    assert_eq!(
        missing["detail"]["code"],
        tonic::Code::PermissionDenied as i32
    );
    fixture.close().await;
}

pub(crate) fn client_stream_runtime() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let fixture = runtime.block_on(Fixture::new());
    let harness = Harness::new(fixture.session.clone());
    let generation = fixture.generation();
    let (stream, _channel) = harness
        .open(generation, STAGE, &envelope(1, &[]), true)
        .expect("the bridge admits the stream");
    assert!(stream > 0);
    use tauri::Manager;
    harness.app.state::<Bridge>().cancel_owner("main");
    runtime.block_on(fixture.close());
}

pub(crate) fn stages_seventy_bridge() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let fixture = runtime.block_on(Fixture::with_role(dokime_server::grpc::access::Role::Admin));
    let harness = Harness::new(fixture.session.clone());
    let generation = fixture.generation();
    let bytes = large_bundle(70 * 1024 * 1024);
    let frames = stage(&harness, generation, &bytes, "");
    assert_eq!(frames.len(), 2, "one reply, then the final status");
    assert_eq!(frames[0][8], FRAME_MESSAGE);
    assert_eq!(frames[1][8], FRAME_END);
    let installation = plugins::PluginInstallation::decode(&frames[0][9..])
        .expect("the reply is a PluginInstallation");
    assert_eq!(installation.sha256, hex(&sha2::Sha256::digest(&bytes)));
    use tauri::Manager;
    harness.app.state::<Bridge>().cancel_owner("main");
    runtime.block_on(fixture.close());
}

pub(crate) fn stages_foreign_finish() {
    let target = if cfg!(windows) {
        "x86_64-unknown-linux-gnu"
    } else {
        "x86_64-pc-windows-msvc"
    };
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let fixture = runtime.block_on(Fixture::with_role(dokime_server::grpc::access::Role::Admin));
    let harness = Harness::new(fixture.session.clone());
    let generation = fixture.generation();
    let frames = stage(
        &harness,
        generation,
        &bundle_for_target(1024, target),
        "[owner-bundle]",
    );
    let terminal = frames.last().unwrap();
    assert_eq!(terminal[8], FRAME_ERROR);
    let error: serde_json::Value = serde_json::from_slice(&terminal[9..]).unwrap();
    let message = error["detail"]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains("no executable for this host"),
        "expected a host-target refusal, got: {error}"
    );
    use tauri::Manager;
    harness.app.state::<Bridge>().cancel_owner("main");
    runtime.block_on(fixture.close());
}

/// Cancelling An Owner Ends Its Live Stream Cancelled And Leaves The Backend Healthy.
pub(crate) async fn owner_cancellation_health() {
    use tauri::Manager;

    // Staging is an administrator operation: under any lesser role the call
    // fails on its own before the cancellation reaches it.
    let fixture = Fixture::with_role(dokime_server::grpc::access::Role::Admin).await;
    let harness = Harness::new(fixture.session.clone());
    let generation = fixture.generation();
    let (stream, channel) = harness
        .open(generation, STAGE, &envelope(1, &[]), true)
        .unwrap();
    harness.app.state::<Bridge>().cancel_owner("main");
    assert_eq!(
        harness
            .control(
                "rpc_stream_send",
                generation,
                stream,
                &stage_header(1, "0".repeat(64)),
                json!({}),
            )
            .unwrap_err()["kind"],
        "unknown_stream"
    );
    let frames = harness.settle(channel, Duration::from_secs(10));
    let error: serde_json::Value = serde_json::from_slice(&frames.last().unwrap()[9..]).unwrap();
    assert_eq!(error["kind"], "cancelled");
    let client = dokime_client::Client::connect(
        dokime_client::Target::local(fixture.server.local_endpoint()).unwrap(),
        Some("rpc-test-token-012345678901234567890123456789"),
        None,
    )
    .await
    .unwrap();
    assert!(
        client
            .backend_get_health(backend::GetHealthRequest {})
            .await
            .is_ok()
    );
    fixture.close().await;
}

/// A Session Advance Ends Every Stream Of The Old Generation.
pub(crate) async fn generation_fences_streams() {
    let fixture = Fixture::new().await;
    let harness = Harness::new(fixture.session.clone());
    let generation = fixture.generation();
    let (_stream, channel) = harness
        .open(
            generation,
            "/dokime.v1.Observation/WatchRuntime",
            &[],
            false,
        )
        .unwrap();
    fixture.session.disconnect().unwrap();
    let frames = harness.settle(channel, Duration::from_secs(10));
    for frame in &frames {
        assert_eq!(
            u64::from_le_bytes(frame[..8].try_into().unwrap()),
            generation
        );
    }
    let error: serde_json::Value = serde_json::from_slice(&frames.last().unwrap()[9..]).unwrap();
    assert_eq!(error["detail"]["kind"], "superseded");
    assert_eq!(
        bridge::invoke(
            &harness.window,
            "rpc",
            fenced(generation, &[]),
            &json!({"dokime-rpc-path": "/dokime.v1.Backend/GetHealth"}),
        )
        .unwrap_err()["detail"]["kind"],
        "superseded"
    );
    fixture.close().await;
}

pub(crate) async fn shared_client_producer() {
    use dokime_client::{Client, Error, Target};

    let fixture = Fixture::with_role(dokime_server::grpc::access::Role::Admin).await;
    let client = Client::connect(
        Target::local(fixture.server.local_endpoint()).unwrap(),
        Some("rpc-test-token-012345678901234567890123456789"),
        None,
    )
    .await
    .unwrap();
    let (entered, ready) = tokio::sync::oneshot::channel();
    let (dropped, released) = tokio::sync::oneshot::channel();
    let uploading = client.clone();
    let task = tokio::spawn(async move {
        uploading
            .plugins_stage(StageInput {
                entered: Some(entered),
                dropped: Some(dropped),
            })
            .await
    });
    ready.await.unwrap();

    client.close();
    assert!(matches!(task.await.unwrap(), Err(Error::Closed)));
    released.await.unwrap();

    fixture.close().await;
}

/* === Private Functions === */
fn large_bundle(resource_bytes: usize) -> Vec<u8> {
    bundle_for_target(resource_bytes, dokime_app::plugins::NATIVE_TARGET)
}

fn bundle_for_target(resource_bytes: usize, target: &str) -> Vec<u8> {
    let owner = "fixture";
    let declaration = serde_json::json!({"kind":"plugin","facility":"sidecar","id":owner,"version":"1.0.0","description":"A fixture.","config":"()"});
    let policy = harmos::operations::OperationPolicy {
        timeout: std::time::Duration::from_secs(5),
        attempt_timeout: std::time::Duration::from_secs(5),
        retries: harmos::operations::RetryPolicy::Never,
        restart: harmos::operations::RestartPolicy::Never,
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
    let restart = serde_json::json!({"kind":"restart","facility":"sidecar","owner":owner,"policy":{"credit":3,"sustained_health_ms":30000,"backoff_base_ms":250,"backoff_cap_ms":10000}});
    let publications =
        serde_json::json!({"kind":"publications","facility":"sidecar","owner":owner,"streams":[]});
    let observers =
        serde_json::json!({"kind":"observers","facility":"sidecar","owner":owner,"observers":[]});
    let executable = format!(
        "inert{prefix}{declaration}\0{prefix}{contracts}\0{prefix}{restart}\0{prefix}{publications}\0{prefix}{observers}\0",
        prefix = std::str::from_utf8(harmos::artifacts::PREFIX).unwrap()
    );
    let resource = vec![0xCDu8; resource_bytes];
    harmos::bundle::Bundle::build(
        harmos::bundle::Identity {
            name: "fixture".into(),
            version: "1.0.0".parse().unwrap(),
            description: "A fixture.".into(),
            package: Default::default(),
        },
        &[
            harmos::bundle::Source {
                path: "bin/fixture",
                bytes: executable.as_bytes(),
                executable: Some(harmos::bundle::Executable {
                    facility: harmos::bundle::Facility::Sidecar,
                    target: target.into(),
                    registrations: vec![],
                }),
            },
            harmos::bundle::Source {
                path: "resources/payload.bin",
                bytes: &resource,
                executable: None,
            },
        ],
        harmos::bundle::Limits::default(),
    )
    .unwrap()
    .bytes()
    .to_vec()
}

fn invoke(
    window: &bridge::Window,
    cmd: &str,
    bytes: Vec<u8>,
) -> Result<tauri::ipc::InvokeResponseBody, serde_json::Value> {
    bridge::invoke(window, cmd, bytes, &serde_json::Value::Null)
}

fn base64(bytes: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

/// Encodes One `oneof value` Arm Of A Client-Stream Input Envelope.
fn envelope(field: u8, message: &[u8]) -> Vec<u8> {
    let mut bytes = vec![(field << 3) | 2];
    let mut length = message.len();
    while length >= 0x80 {
        bytes.push((length as u8) | 0x80);
        length >>= 7;
    }
    bytes.push(length as u8);
    bytes.extend_from_slice(message);
    bytes
}

fn stage_header(size_bytes: u64, sha256: String) -> Vec<u8> {
    envelope(
        2,
        &plugins::StagePluginFrame {
            frame: Some(plugins::StagePluginFrameFrame::Header(
                plugins::StagePluginHeader { size_bytes, sha256 },
            )),
        }
        .encode_to_vec(),
    )
}

/// Streams A Bundle Through The Generic Bridge And Returns Its Channel Frames.
fn stage(harness: &Harness, generation: u64, bytes: &[u8], label: &str) -> Vec<Vec<u8>> {
    const CHUNK_BYTES: usize = 256 * 1024;
    let sha256 = hex(&sha2::Sha256::digest(bytes));
    let chunk_count = bytes.len().div_ceil(CHUNK_BYTES);
    eprintln!(
        "[bridge]{label} bundle bytes: {}, chunks: {chunk_count}",
        bytes.len()
    );

    let open_started = std::time::Instant::now();
    let (stream, channel) = harness
        .open(generation, STAGE, &envelope(1, &[]), true)
        .expect("the bridge admits the stage stream");
    harness
        .control(
            "rpc_stream_send",
            generation,
            stream,
            &stage_header(bytes.len() as u64, sha256),
            json!({}),
        )
        .expect("the header is accepted");
    let open_elapsed = open_started.elapsed();

    let writes_started = std::time::Instant::now();
    for (index, offset) in (0..bytes.len()).step_by(CHUNK_BYTES).enumerate() {
        let end = (offset + CHUNK_BYTES).min(bytes.len());
        let chunk = plugins::StagePluginFrame {
            frame: Some(plugins::StagePluginFrameFrame::Chunk(
                plugins::StagePluginChunk {
                    offset: offset as u64,
                    data: bytes[offset..end].to_vec(),
                },
            )),
        };
        harness
            .control(
                "rpc_stream_send",
                generation,
                stream,
                &envelope(2, &chunk.encode_to_vec()),
                json!({}),
            )
            .unwrap_or_else(|error| panic!("write {} failed: {error:?}", index + 1));
    }
    let writes_elapsed = writes_started.elapsed();

    let finish_started = std::time::Instant::now();
    harness
        .control("rpc_stream_finish", generation, stream, &[], json!({}))
        .expect("the input closes");
    let frames = harness.settle(channel, Duration::from_secs(60));
    let finish_elapsed = finish_started.elapsed();
    eprintln!("[bridge]{label} open: {open_elapsed:?}");
    eprintln!("[bridge]{label} writes ({chunk_count} x <= {CHUNK_BYTES}B): {writes_elapsed:?}");
    eprintln!("[bridge]{label} finish: {finish_elapsed:?}");
    frames
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn fresh_runtime(
    multithread: bool,
    threads: Option<usize>,
    paused: bool,
) -> tokio::runtime::Runtime {
    let mut builder = if multithread {
        tokio::runtime::Builder::new_multi_thread()
    } else {
        tokio::runtime::Builder::new_current_thread()
    };
    builder.enable_all();
    if let Some(threads) = threads {
        builder.worker_threads(threads);
    }
    let _ = paused;
    builder.build().expect("scenario runtime")
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

/* === Tests === */
#[test]
fn domain_contract() {
    {
        eprintln!("Scenario: artifact_upload::upload_sink_completion");
        run_scenario(
            artifact_upload::upload_sink_completion,
            true,
            Some(2),
            false,
        );
    }
    {
        eprintln!("Scenario: artifact_upload::upload_sink_sequence");
        run_scenario(artifact_upload::upload_sink_sequence, true, Some(2), false);
    }
    {
        eprintln!("Scenario: artifact_upload::upload_sink_owner");
        run_scenario(artifact_upload::upload_sink_owner, true, Some(2), false);
    }
    {
        eprintln!("Scenario: artifact_upload::recording_cancel_file");
        run_scenario(artifact_upload::recording_cancel_file, true, Some(2), false);
    }
    {
        eprintln!("Scenario: artifact_upload::upload_sink_runtime");
        artifact_upload::upload_sink_runtime();
    }
    {
        eprintln!("Scenario: typescript_facade_bridge");
        run_scenario(typescript_facade_bridge, true, Some(2), false);
    }
    {
        eprintln!("Scenario: refuses_invalid_dispatch");
        run_scenario(refuses_invalid_dispatch, true, Some(2), false);
    }
    {
        eprintln!("Scenario: unary_bridge");
        run_scenario(unary_bridge, true, Some(2), false);
    }
    {
        eprintln!("Scenario: generation_fences_streams");
        run_scenario(generation_fences_streams, true, Some(2), false);
    }
    {
        eprintln!("Scenario: client_stream_runtime");
        client_stream_runtime();
    }
    {
        eprintln!("Scenario: stages_seventy_bridge");
        stages_seventy_bridge();
    }
    {
        eprintln!("Scenario: stages_foreign_finish");
        stages_foreign_finish();
    }
    {
        eprintln!("Scenario: owner_cancellation_health");
        run_scenario(owner_cancellation_health, true, Some(2), false);
    }
    {
        eprintln!("Scenario: shared_client_producer");
        run_scenario(shared_client_producer, false, None, false);
    }
}
