/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
#[path = "../support/rpc_artifact.rs"]
pub(crate) mod artifact_fixture;
pub(crate) mod artifact_upload;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
};
// Third Party Imports

// Local Imports
use dokime_app::{
    Runtime, RuntimeConfig,
    core::{Identifier, Metadata},
};
use dokime_gui::{
    connection::ConnectionSession,
    rpc::{Bridge, MAX_FRAME_BYTES, handler},
};
use dokime_server::grpc::{Config, Server};
use prost::Message;
use sha2::{Digest, Sha256};
use std::sync::Arc;

/* === Constants === */

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
    type Item = dokime_protocol::v1::StagePluginFrame;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let Some(entered) = self.entered.take() else {
            return std::task::Poll::Pending;
        };
        entered.send(()).unwrap();

        std::task::Poll::Ready(Some(dokime_protocol::v1::StagePluginFrame {
            frame: Some(dokime_protocol::v1::stage_plugin_frame::Frame::Header(
                dokime_protocol::v1::StagePluginHeader {
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
    pub async fn close(mut self) {
        self.session.disconnect().unwrap();
        self.server.shutdown().await.unwrap();
        self.runtime.shutdown().await.unwrap();
    }
}

/* === Public Functions === */

/* === Private Functions === */

pub(crate) async fn typescript_facade_grpc() {
    let fixture = Fixture::new().await;
    fixture.runtime.handle().set_brand(Some(
        dokime_app::capabilities::installed_plugins::load_brand(
            &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../branding"),
        )
        .unwrap(),
    ));
    let artifact_id = artifact_fixture::seed(&fixture.runtime.handle(), &fixture.artifacts).await;
    let app = tauri::test::mock_builder()
        .manage(fixture.session.clone())
        .manage(Bridge::default())
        .invoke_handler(handler())
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .unwrap();
    let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let mut child = Command::new("node")
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
    let mut input = child.stdin.take().unwrap();
    let output = BufReader::new(child.stdout.take().unwrap());
    for line in output.lines() {
        let request: serde_json::Value = serde_json::from_str(&line.unwrap()).unwrap();
        let cmd = request["cmd"].as_str().unwrap();
        let bytes: Vec<u8> = serde_json::from_value(request["bytes"].clone()).unwrap();
        let response = match cmd {
            "fixture_cancel_streams" => {
                use tauri::Manager;
                app.state::<Bridge>().cancel_owner("main");
                serde_json::json!({"value": null})
            }
            // These fixtures exercise TS-generated message semantics against concrete Prost types.
            "fixture_value" => {
                let value = dokime_protocol::v1::ValueObject::decode(bytes.as_slice()).unwrap();
                assert_eq!(
                    value.fields["minimum"].kind,
                    Some(dokime_protocol::v1::value::Kind::IntegerValue(i64::MIN))
                );
                assert_eq!(
                    value.fields["maximum"].kind,
                    Some(dokime_protocol::v1::value::Kind::IntegerValue(i64::MAX))
                );
                assert_eq!(
                    value.fields["null"].kind,
                    Some(dokime_protocol::v1::value::Kind::NullValue(0))
                );
                assert_eq!(value.fields["unset"].kind, None);
                serde_json::json!({ "bytes": value.encode_to_vec() })
            }
            "fixture_admission" => {
                let value = dokime_protocol::v1::Admission::decode(bytes.as_slice()).unwrap();
                assert_eq!(value.cursor.as_ref().unwrap().position, u64::MAX);
                assert_eq!(value.accepted_at.as_ref().unwrap().seconds, -1);
                assert_eq!(value.accepted_at.as_ref().unwrap().nanos, 999_999_999);
                serde_json::json!({ "bytes": value.encode_to_vec() })
            }
            _ => match invoke(&window, cmd, bytes) {
                Ok(tauri::ipc::InvokeResponseBody::Raw(bytes)) => {
                    serde_json::json!({ "bytes": bytes })
                }
                Ok(tauri::ipc::InvokeResponseBody::Json(json)) => {
                    serde_json::json!({ "value": serde_json::from_str::<serde_json::Value>(&json).unwrap() })
                }
                Err(error) => serde_json::json!({ "error": error }),
            },
        };
        writeln!(input, "{response}").unwrap();
        input.flush().unwrap();
    }
    assert!(
        child.wait().unwrap().success(),
        "TypeScript/native interop failed"
    );
    fixture.close().await;
}

fn large_bundle(resource_bytes: usize) -> Vec<u8> {
    bundle_for_target(resource_bytes, dokime_app::plugins::NATIVE_TARGET)
}

fn bundle_for_target(resource_bytes: usize, target: &str) -> Vec<u8> {
    let owner = "fixture";
    let declaration = serde_json::json!({"kind":"plugin","facility":"sidecar","id":owner,"version":"1.0.0","description":"A fixture.","config":"()"});
    let policy = harmos::OperationPolicy {
        timeout: std::time::Duration::from_secs(5),
        attempt_timeout: std::time::Duration::from_secs(5),
        retries: harmos::RetryPolicy::Never,
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
        prefix = std::str::from_utf8(harmos::artifact::PREFIX).unwrap()
    );
    let resource = vec![0xCDu8; resource_bytes];
    harmos::plugin::Bundle::build(
        harmos::plugin::Identity {
            name: "fixture".into(),
            version: "1.0.0".parse().unwrap(),
            description: "A fixture.".into(),
            package: Default::default(),
        },
        &[
            harmos::plugin::Source {
                path: "bin/fixture",
                bytes: executable.as_bytes(),
                executable: Some(harmos::plugin::Executable {
                    facility: harmos::plugin::Facility::Sidecar,
                    target: target.into(),
                    registrations: vec![],
                }),
            },
            harmos::plugin::Source {
                path: "resources/payload.bin",
                bytes: &resource,
                executable: None,
            },
        ],
        harmos::plugin::Limits::default(),
    )
    .unwrap()
    .bytes()
    .to_vec()
}

fn invoke(
    window: &tauri::WebviewWindow<tauri::test::MockRuntime>,
    cmd: &str,
    bytes: Vec<u8>,
) -> Result<tauri::ipc::InvokeResponseBody, serde_json::Value> {
    tauri::test::get_ipc_response(
        window,
        tauri::webview::InvokeRequest {
            cmd: cmd.into(),
            callback: tauri::ipc::CallbackFn(0),
            error: tauri::ipc::CallbackFn(1),
            url: if cfg!(windows) {
                "http://tauri.localhost"
            } else {
                "tauri://localhost"
            }
            .parse()
            .unwrap(),
            body: tauri::ipc::InvokeBody::Raw(bytes),
            headers: Default::default(),
            invoke_key: tauri::test::INVOKE_KEY.into(),
        },
    )
}

pub(crate) async fn refuses_invalid_dispatch() {
    let fixture = Fixture::new().await;
    let app = tauri::test::mock_builder()
        .manage(fixture.session.clone())
        .manage(Bridge::default())
        .invoke_handler(handler())
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .unwrap();
    let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    for (bytes, kind) in [
        (vec![], "invalid_request"),
        (vec![0; 40], "incompatible_descriptor"),
        (vec![0; MAX_FRAME_BYTES + 40 + 1], "limit_exceeded"),
    ] {
        assert_eq!(
            invoke(&window, "rpc_backend_get_health", bytes).unwrap_err()["kind"],
            kind
        );
    }
    let mut bytes = 1_u64.to_le_bytes().to_vec();
    bytes.extend(Sha256::digest(dokime_protocol::FILE_DESCRIPTOR_SET));
    bytes.push(0xff);
    assert_eq!(
        invoke(&window, "rpc_backend_get_health", bytes).unwrap_err()["kind"],
        "invalid_request"
    );
    fixture.close().await;
}

/* === Tests === */

pub(crate) fn client_stream_runtime() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let fixture = runtime.block_on(Fixture::new());
    let app = tauri::test::mock_builder()
        .manage(fixture.session.clone())
        .manage(Bridge::default())
        .invoke_handler(handler())
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .unwrap();
    let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();

    let generation: u64 = fixture.session.status().generation.parse().unwrap();
    let header = dokime_protocol::v1::StagePluginFrame {
        frame: Some(dokime_protocol::v1::stage_plugin_frame::Frame::Header(
            dokime_protocol::v1::StagePluginHeader {
                size_bytes: 1,
                sha256: "0".repeat(64),
            },
        )),
    };
    let mut bytes = generation.to_le_bytes().to_vec();
    bytes.extend(Sha256::digest(dokime_protocol::FILE_DESCRIPTOR_SET));
    bytes.extend(header.encode_to_vec());

    let response = invoke(&window, "rpc_plugins_stage_open", bytes)
        .expect("the stage opener admits the stream");
    let tauri::ipc::InvokeResponseBody::Raw(handle) = response else {
        panic!("the stage opener replies with a raw stream handle");
    };
    assert_eq!(handle.len(), 16);

    use tauri::Manager;
    app.state::<Bridge>().cancel_owner("main");
    runtime.block_on(fixture.close());
}

pub(crate) fn stages_seventy_bridge() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let fixture = runtime.block_on(Fixture::with_role(dokime_server::grpc::access::Role::Admin));
    let app = tauri::test::mock_builder()
        .manage(fixture.session.clone())
        .manage(Bridge::default())
        .invoke_handler(handler())
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .unwrap();
    let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let generation: u64 = fixture.session.status().generation.parse().unwrap();
    let descriptor_sha256 = Sha256::digest(dokime_protocol::FILE_DESCRIPTOR_SET);

    const CHUNK_BYTES: usize = 256 * 1024;
    let bytes = large_bundle(70 * 1024 * 1024);
    let sha256 = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let chunk_count = bytes.len().div_ceil(CHUNK_BYTES);
    eprintln!(
        "[bridge] bundle bytes: {}, chunks: {chunk_count}",
        bytes.len()
    );

    let header = dokime_protocol::v1::StagePluginFrame {
        frame: Some(dokime_protocol::v1::stage_plugin_frame::Frame::Header(
            dokime_protocol::v1::StagePluginHeader {
                size_bytes: bytes.len() as u64,
                sha256: sha256.clone(),
            },
        )),
    };
    let mut open_bytes = generation.to_le_bytes().to_vec();
    open_bytes.extend(descriptor_sha256);
    open_bytes.extend(header.encode_to_vec());

    let open_started = std::time::Instant::now();
    let response = invoke(&window, "rpc_plugins_stage_open", open_bytes)
        .expect("the stage opener admits the stream");
    let open_elapsed = open_started.elapsed();
    let tauri::ipc::InvokeResponseBody::Raw(handle) = response else {
        panic!("the stage opener replies with a raw stream handle");
    };
    assert_eq!(handle.len(), 16);
    let stream_id = u64::from_le_bytes(handle[8..16].try_into().unwrap());

    let writes_started = std::time::Instant::now();
    for (index, offset) in (0..bytes.len()).step_by(CHUNK_BYTES).enumerate() {
        let sequence = (index + 1) as u64;
        let end = (offset + CHUNK_BYTES).min(bytes.len());
        let chunk = dokime_protocol::v1::StagePluginFrame {
            frame: Some(dokime_protocol::v1::stage_plugin_frame::Frame::Chunk(
                dokime_protocol::v1::StagePluginChunk {
                    offset: offset as u64,
                    data: bytes[offset..end].to_vec(),
                },
            )),
        };
        let mut payload = generation.to_le_bytes().to_vec();
        payload.extend(descriptor_sha256);
        payload.extend(stream_id.to_le_bytes());
        payload.extend(sequence.to_le_bytes());
        payload.extend(chunk.encode_to_vec());
        invoke(&window, "client_stream_write", payload)
            .unwrap_or_else(|error| panic!("write {sequence} failed: {error:?}"));
    }
    let writes_elapsed = writes_started.elapsed();

    let mut finish_payload = generation.to_le_bytes().to_vec();
    finish_payload.extend(descriptor_sha256);
    finish_payload.extend(stream_id.to_le_bytes());
    let finish_started = std::time::Instant::now();
    let finish_response = invoke(&window, "client_stream_finish", finish_payload)
        .expect("finish settles the staged plugin");
    let finish_elapsed = finish_started.elapsed();

    let tauri::ipc::InvokeResponseBody::Raw(reply) = finish_response else {
        panic!("finish replies with the encoded PluginInstallation");
    };
    // `reply_bytes` prefixes the payload with the 8-byte generation, exactly
    // like the stream handle `rpc_plugins_stage_open` returned above.
    let installation = dokime_protocol::v1::PluginInstallation::decode(&reply[8..])
        .expect("finish replies with a valid PluginInstallation");
    assert_eq!(installation.sha256, sha256);

    eprintln!("[bridge] open: {open_elapsed:?}");
    eprintln!("[bridge] writes ({chunk_count} x <= {CHUNK_BYTES}B): {writes_elapsed:?}");
    eprintln!("[bridge] finish: {finish_elapsed:?}");
    eprintln!(
        "[bridge] total: {:?}",
        open_elapsed + writes_elapsed + finish_elapsed
    );

    use tauri::Manager;
    app.state::<Bridge>().cancel_owner("main");
    runtime.block_on(fixture.close());
}

pub(crate) fn stages_foreign_finish() {
    let target = if cfg!(windows) {
        "x86_64-unknown-linux-gnu"
    } else {
        "x86_64-pc-windows-msvc"
    };
    let bytes = bundle_for_target(1024, target);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let fixture = runtime.block_on(Fixture::with_role(dokime_server::grpc::access::Role::Admin));
    let app = tauri::test::mock_builder()
        .manage(fixture.session.clone())
        .manage(Bridge::default())
        .invoke_handler(handler())
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .unwrap();
    let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let generation: u64 = fixture.session.status().generation.parse().unwrap();
    let descriptor_sha256 = Sha256::digest(dokime_protocol::FILE_DESCRIPTOR_SET);

    const CHUNK_BYTES: usize = 256 * 1024;
    let sha256 = Sha256::digest(&bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let chunk_count = bytes.len().div_ceil(CHUNK_BYTES);
    eprintln!(
        "[bridge][owner-bundle] bundle bytes: {}, chunks: {chunk_count}",
        bytes.len()
    );

    let header = dokime_protocol::v1::StagePluginFrame {
        frame: Some(dokime_protocol::v1::stage_plugin_frame::Frame::Header(
            dokime_protocol::v1::StagePluginHeader {
                size_bytes: bytes.len() as u64,
                sha256: sha256.clone(),
            },
        )),
    };
    let mut open_bytes = generation.to_le_bytes().to_vec();
    open_bytes.extend(descriptor_sha256);
    open_bytes.extend(header.encode_to_vec());

    let open_started = std::time::Instant::now();
    let response = invoke(&window, "rpc_plugins_stage_open", open_bytes)
        .expect("the stage opener admits the stream");
    let open_elapsed = open_started.elapsed();
    let tauri::ipc::InvokeResponseBody::Raw(handle) = response else {
        panic!("the stage opener replies with a raw stream handle");
    };
    assert_eq!(handle.len(), 16);
    let stream_id = u64::from_le_bytes(handle[8..16].try_into().unwrap());

    let writes_started = std::time::Instant::now();
    for (index, offset) in (0..bytes.len()).step_by(CHUNK_BYTES).enumerate() {
        let sequence = (index + 1) as u64;
        let end = (offset + CHUNK_BYTES).min(bytes.len());
        let chunk = dokime_protocol::v1::StagePluginFrame {
            frame: Some(dokime_protocol::v1::stage_plugin_frame::Frame::Chunk(
                dokime_protocol::v1::StagePluginChunk {
                    offset: offset as u64,
                    data: bytes[offset..end].to_vec(),
                },
            )),
        };
        let mut payload = generation.to_le_bytes().to_vec();
        payload.extend(descriptor_sha256);
        payload.extend(stream_id.to_le_bytes());
        payload.extend(sequence.to_le_bytes());
        payload.extend(chunk.encode_to_vec());
        invoke(&window, "client_stream_write", payload)
            .unwrap_or_else(|error| panic!("write {sequence} failed: {error:?}"));
    }
    let writes_elapsed = writes_started.elapsed();

    let mut finish_payload = generation.to_le_bytes().to_vec();
    finish_payload.extend(descriptor_sha256);
    finish_payload.extend(stream_id.to_le_bytes());
    let finish_started = std::time::Instant::now();
    // A timeout here (rather than a bounded wait) is deliberate: the whole
    // point of this test is to prove `finish` no longer hangs forever the
    // way it did before the `client.ready().await` fix.
    let finish_result = std::thread::scope(|scope| {
        let handle = scope.spawn(|| invoke(&window, "client_stream_finish", finish_payload));
        for _ in 0..600 {
            if handle.is_finished() {
                return handle.join().unwrap();
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        panic!("client_stream_finish did not return within 60s; it hung");
    });
    let finish_elapsed = finish_started.elapsed();

    let error = finish_result.expect_err("a foreign-targeted bundle must be refused on this host");
    let message = error.to_string();
    assert!(
        message.contains("no executable for this host"),
        "expected a host-target refusal, got: {message}"
    );

    eprintln!("[bridge][owner-bundle] open: {open_elapsed:?}");
    eprintln!(
        "[bridge][owner-bundle] writes ({chunk_count} x <= {CHUNK_BYTES}B): {writes_elapsed:?}"
    );
    eprintln!("[bridge][owner-bundle] finish (refusal): {finish_elapsed:?}");
    eprintln!(
        "[bridge][owner-bundle] total: {:?}",
        open_elapsed + writes_elapsed + finish_elapsed
    );

    use tauri::Manager;
    app.state::<Bridge>().cancel_owner("main");
    runtime.block_on(fixture.close());
}

pub(crate) async fn owner_cancellation_health() {
    use tauri::Manager;

    let fixture = Fixture::new().await;
    let app = tauri::test::mock_builder()
        .manage(fixture.session.clone())
        .manage(Bridge::default())
        .invoke_handler(handler())
        .build(tauri::test::mock_context(tauri::test::noop_assets()))
        .unwrap();
    let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    let mut request = 1_u64.to_le_bytes().to_vec();
    request.extend(Sha256::digest(dokime_protocol::FILE_DESCRIPTOR_SET));
    request.extend(dokime_protocol::v1::WatchLogsRequest::default().encode_to_vec());
    let tauri::ipc::InvokeResponseBody::Raw(opened) =
        invoke(&window, "rpc_backend_watch_logs_open", request).unwrap()
    else {
        panic!("the log stream returns its owner handle");
    };
    assert_eq!(opened.len(), 16);
    app.state::<Bridge>().cancel_owner("main");
    let mut next = 1_u64.to_le_bytes().to_vec();
    next.extend(Sha256::digest(dokime_protocol::FILE_DESCRIPTOR_SET));
    next.extend_from_slice(&opened[8..16]);
    next.extend(0_u64.to_le_bytes());
    assert_eq!(
        invoke(&window, "stream_next", next).unwrap_err()["kind"],
        "unknown_stream"
    );
    let mut health = 1_u64.to_le_bytes().to_vec();
    health.extend(Sha256::digest(dokime_protocol::FILE_DESCRIPTOR_SET));
    health.extend(dokime_protocol::v1::GetHealthRequest {}.encode_to_vec());
    assert!(invoke(&window, "rpc_backend_get_health", health).is_ok());
    fixture.close().await;
}

pub(crate) async fn shared_client_producer() {
    use dokime::client::{Client, Error, Target};

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
        eprintln!("Scenario: typescript_facade_grpc");
        run_scenario(typescript_facade_grpc, true, Some(2), false);
    }
    {
        eprintln!("Scenario: refuses_invalid_dispatch");
        run_scenario(refuses_invalid_dispatch, false, None, false);
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

#[inline(never)]
fn run_scenario<F: std::future::Future>(
    factory: impl FnOnce() -> F,
    multithread: bool,
    threads: Option<usize>,
    paused: bool,
) -> F::Output {
    fresh_runtime(multithread, threads, paused).block_on(Box::pin(factory()))
}
