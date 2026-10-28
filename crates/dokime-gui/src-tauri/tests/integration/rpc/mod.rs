/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod upload;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use crate::wire::WireMessage;
use serde_json::json;
use sha2::Digest;
// Local Imports
use crate::support::bridge::{self, Harness, fenced};
use crate::support::server::Fixture;
use dokime::domain::model;
use dokime::operations::backend;
use dokime_gui::rpc::{Bridge, FRAME_END, FRAME_ERROR, FRAME_MESSAGE};

/* === Constants === */

const INSTALL: &str = "/dokime.v1.Plugins/Install";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct InstallInput {
    entered: Option<tokio::sync::oneshot::Sender<()>>,
    dropped: Option<tokio::sync::oneshot::Sender<()>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl tokio_stream::Stream for InstallInput {
    type Item = model::plugins::wire::InstallPluginFrame;

    fn poll_next(mut self: std::pin::Pin<&mut Self>, _: &mut std::task::Context<'_>) -> std::task::Poll<Option<Self::Item>> {
        let Some(entered) = self.entered.take() else {
            return std::task::Poll::Pending;
        };
        entered.send(()).unwrap();

        std::task::Poll::Ready(Some(model::plugins::wire::InstallPluginFrame {
            frame: Some(model::plugins::wire::InstallPluginFrameFrame::Header(model::plugins::wire::InstallPluginHeader {
                size_bytes: 1,
                sha256: "00".repeat(32),
            })),
        }))
    }
}

impl Drop for InstallInput {
    fn drop(&mut self) {
        if let Some(dropped) = self.dropped.take() {
            let _ = dropped.send(());
        }
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn large_bundle(resource_bytes: usize) -> Vec<u8> {
    bundle_for_target(resource_bytes, dokime::plugins::NATIVE_TARGET)
}

fn bundle_for_target(resource_bytes: usize, target: &str) -> Vec<u8> {
    let owner = "fixture";
    let declaration =
        serde_json::json!({"kind":"plugin","facility":"sidecar","id":owner,"version":"1.0.0","description":"A fixture.","config":"()"});
    let policy = dokime::harmos::operations::OperationPolicy {
        timeout: std::time::Duration::from_secs(120),
        attempt_timeout: std::time::Duration::from_secs(120),
        retries: dokime::harmos::operations::RetryPolicy::Never,
    };
    let lifecycle = [
        dokime::harmos::builtins::lifecycle::Descriptor::new(dokime::harmos::builtins::lifecycle::Action::Initialize, policy).unwrap(),
        dokime::harmos::builtins::lifecycle::Descriptor::new(dokime::harmos::builtins::lifecycle::Action::Shutdown, policy).unwrap(),
    ];
    let contracts =
        serde_json::json!({"kind":"contracts","facility":"sidecar","owner":owner,"provides":[],"requires":[],"lifecycle":lifecycle});
    let restart = serde_json::json!({"kind":"restart","facility":"sidecar","owner":owner,"policy":{"credit":3,"sustained_health_ms":30000,"backoff_base_ms":250,"backoff_cap_ms":10000}});
    let publications = serde_json::json!({"kind":"publications","facility":"sidecar","owner":owner,"streams":[]});
    let observers = serde_json::json!({"kind":"observers","facility":"sidecar","owner":owner,"observers":[]});
    let executable = format!(
        "inert{prefix}{declaration}\0{prefix}{contracts}\0{prefix}{restart}\0{prefix}{publications}\0{prefix}{observers}\0",
        prefix = std::str::from_utf8(dokime::harmos::artifacts::PREFIX).unwrap()
    );
    let resource = vec![0xCDu8; resource_bytes];
    harmos_packaging::bundle::Bundle::build(
        harmos_packaging::bundle::Identity {
            name: "fixture".into(),
            version: "1.0.0".parse().unwrap(),
            description: "A fixture.".into(),
            package: Default::default(),
        },
        &[
            harmos_packaging::bundle::Source {
                path: "bin/fixture",
                bytes: executable.as_bytes(),
                executable: Some(harmos_packaging::bundle::Executable {
                    facility: harmos_packaging::bundle::Facility::Sidecar,
                    target: target.into(),
                    registrations: vec![],
                }),
            },
            harmos_packaging::bundle::Source {
                path: "resources/payload.bin",
                bytes: &resource,
                executable: None,
            },
        ],
        harmos_packaging::bundle::Limits::default(),
    )
    .unwrap()
    .bytes()
    .to_vec()
}

fn invoke(window: &bridge::Window, cmd: &str, bytes: Vec<u8>) -> Result<tauri::ipc::InvokeResponseBody, serde_json::Value> {
    bridge::invoke(window, cmd, bytes, &serde_json::Value::Null)
}

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

fn install_header(size_bytes: u64, sha256: String) -> Vec<u8> {
    envelope(
        2,
        &model::plugins::wire::InstallPluginFrame {
            frame: Some(model::plugins::wire::InstallPluginFrameFrame::Header(model::plugins::wire::InstallPluginHeader {
                size_bytes,
                sha256,
            })),
        }
        .wire_bytes(),
    )
}

fn install(harness: &Harness, generation: u64, bytes: &[u8]) -> Vec<Vec<u8>> {
    const CHUNK_BYTES: usize = 256 * 1024;
    let sha256 = hex(&sha2::Sha256::digest(bytes));

    let (stream, channel) = harness.open(generation, INSTALL, &envelope(1, &[]), true).expect("the bridge admits the install stream");
    harness
        .control("rpc_stream_send", generation, stream, &install_header(bytes.len() as u64, sha256), json!({}))
        .expect("the header is accepted");

    for (index, offset) in (0..bytes.len()).step_by(CHUNK_BYTES).enumerate() {
        let end = (offset + CHUNK_BYTES).min(bytes.len());
        let chunk = model::plugins::wire::InstallPluginFrame {
            frame: Some(model::plugins::wire::InstallPluginFrameFrame::Chunk(model::plugins::wire::InstallPluginChunk {
                offset: offset as u64,
                data: bytes[offset..end].to_vec(),
            })),
        };
        harness
            .control("rpc_stream_send", generation, stream, &envelope(2, &chunk.wire_bytes()), json!({}))
            .unwrap_or_else(|error| panic!("write {} failed: {error:?}", index + 1));
    }

    harness.control("rpc_stream_finish", generation, stream, &[], json!({})).expect("the input closes");
    harness.settle(channel)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

/* === Tests === */

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn refuses_invalid_dispatch() {
    let fixture = Fixture::new().await;
    let harness = Harness::new(fixture.session.clone());
    let generation = fixture.generation();
    for command in ["rpc", "rpc_stream_open"] {
        for (bytes, kind) in
            [(vec![], "invalid_request"), (vec![0; 40], "incompatible_descriptor"), (vec![0; 4_194_304_usize + 40 + 1], "limit_exceeded")]
        {
            assert_eq!(invoke(&harness.window, command, bytes).unwrap_err()["kind"], kind);
        }
        for path in [None, Some("dokime.v1.Backend/GetHealth"), Some("/a/b/c")] {
            let headers = path.map_or(json!({}), |path| json!({"dokime-rpc-path": path}));
            assert_eq!(
                bridge::invoke(&harness.window, command, fenced(generation, &[]), &headers).unwrap_err()["kind"],
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
        bridge::invoke(&harness.window, "rpc", fenced(generation + 1, &[]), &json!({"dokime-rpc-path": "/dokime.v1.Backend/GetHealth"}),)
            .unwrap_err()["detail"]["kind"],
        "superseded"
    );
    for command in ["rpc_stream_send", "rpc_stream_finish", "rpc_stream_cancel"] {
        assert_eq!(harness.control(command, generation, 99, &[], json!({})).unwrap_err()["kind"], "unknown_stream");
    }
    let (stream, channel) = harness.open(generation, INSTALL, &envelope(1, &[]), true).unwrap();
    for credit in ["0", "33", "x", "1"] {
        assert_eq!(
            harness.control("rpc_stream_ack", generation, stream, &[], json!({"dokime-rpc-credit": credit}),).unwrap_err()["kind"],
            "invalid_ack",
            "credit {credit} exceeds what was delivered"
        );
    }
    harness.control("rpc_stream_cancel", generation, stream, &[], json!({})).unwrap();
    let frames = harness.settle(channel);
    assert_eq!(frames.last().unwrap()[8], FRAME_ERROR);
    fixture.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn unary_call_bridges() {
    let fixture = Fixture::new().await;
    let harness = Harness::new(fixture.session.clone());
    let generation = fixture.generation();
    let reply = bridge::invoke(
        &harness.window,
        "rpc",
        fenced(generation, &backend::GetIdentity {}.wire_bytes()),
        &json!({"dokime-rpc-path": "/dokime.v1.Backend/GetIdentity"}),
    )
    .unwrap();
    let tauri::ipc::InvokeResponseBody::Raw(reply) = reply else {
        panic!("a unary reply is raw bytes");
    };
    assert_eq!(u64::from_le_bytes(reply[..8].try_into().unwrap()), generation);
    let (identity, _admitted_memory) = model::backend::wire::BackendIdentity::from_wire(&reply[8..]).unwrap();
    assert_eq!(identity.backend_id, fixture.session.status().backend.unwrap().backend_id);
    // A typed backend status crosses unchanged.
    let missing = bridge::invoke(&harness.window, "rpc", fenced(generation, &[]), &json!({"dokime-rpc-path": "/dokime.v1.Missing/Call"}))
        .unwrap_err();
    assert_eq!(missing["kind"], "grpc");
    assert_eq!(missing["detail"]["code"], tonic::Code::PermissionDenied as i32);
    fixture.close().await;
}

#[test]
fn client_stream_opens() {
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let fixture = runtime.block_on(Fixture::new());
    let harness = Harness::new(fixture.session.clone());
    let generation = fixture.generation();
    let (stream, _channel) = harness.open(generation, INSTALL, &envelope(1, &[]), true).expect("the bridge admits the stream");
    assert!(stream > 0);
    use tauri::Manager;
    harness.app.state::<Bridge>().cancel_owner("main");
    runtime.block_on(fixture.close());
}

#[test]
fn installs_large_bundle() {
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let fixture = runtime.block_on(Fixture::new());
    let harness = Harness::new(fixture.session.clone());
    let generation = fixture.generation();
    let bytes = large_bundle(70 * 1024 * 1024);
    let frames = install(&harness, generation, &bytes);
    assert_eq!(frames.len(), 2, "one reply, then the final status");
    assert_eq!(frames[0][8], FRAME_MESSAGE);
    assert_eq!(frames[1][8], FRAME_END);
    let (installation, _admitted_memory) =
        model::plugins::wire::PluginInstallation::from_wire(&frames[0][9..]).expect("the reply is a PluginInstallation");
    assert_eq!(installation.sha256, hex(&sha2::Sha256::digest(&bytes)));
    assert!(matches!(installation.state, model::plugins::wire::PluginInstallationState::Installed));
    assert!(installation.installed_at.is_some());
    use tauri::Manager;
    harness.app.state::<Bridge>().cancel_owner("main");
    runtime.block_on(fixture.close());
}

#[test]
fn refuses_foreign_bundle() {
    let target = if cfg!(windows) { "x86_64-unknown-linux-gnu" } else { "x86_64-pc-windows-msvc" };
    let runtime = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();
    let fixture = runtime.block_on(Fixture::new());
    let harness = Harness::new(fixture.session.clone());
    let generation = fixture.generation();
    let frames = install(&harness, generation, &bundle_for_target(1024, target));
    let terminal = frames.last().unwrap();
    assert_eq!(terminal[8], FRAME_ERROR);
    let error: serde_json::Value = serde_json::from_slice(&terminal[9..]).unwrap();
    let message = error["detail"]["message"].as_str().unwrap_or_default();
    assert!(message.contains("no executable for this host"), "expected a host-target refusal, got: {error}");
    use tauri::Manager;
    harness.app.state::<Bridge>().cancel_owner("main");
    runtime.block_on(fixture.close());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn owner_cancellation_health() {
    use tauri::Manager;

    let fixture = Fixture::new().await;
    let harness = Harness::new(fixture.session.clone());
    let generation = fixture.generation();
    let (stream, channel) = harness.open(generation, INSTALL, &envelope(1, &[]), true).unwrap();
    harness.app.state::<Bridge>().cancel_owner("main");
    assert_eq!(
        harness.control("rpc_stream_send", generation, stream, &install_header(1, "0".repeat(64)), json!({}),).unwrap_err()["kind"],
        "unknown_stream"
    );
    let frames = harness.settle(channel);
    let error: serde_json::Value = serde_json::from_slice(&frames.last().unwrap()[9..]).unwrap();
    assert_eq!(error["kind"], "cancelled");
    let client =
        dokime::client::Client::connect(dokime::client::Target::local(fixture.server.local_endpoint()).unwrap(), None, None).await.unwrap();
    assert!(client.backend_get_health(backend::GetHealth {}).await.is_ok());
    fixture.close().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn generation_fences_streams() {
    let fixture = Fixture::new().await;
    let harness = Harness::new(fixture.session.clone());
    let generation = fixture.generation();
    let (_stream, channel) = harness.open(generation, "/dokime.v1.Observation/WatchRuntime", &[], false).unwrap();
    fixture.session.disconnect().unwrap();
    let frames = harness.settle(channel);
    for frame in &frames {
        assert_eq!(u64::from_le_bytes(frame[..8].try_into().unwrap()), generation);
    }
    let error: serde_json::Value = serde_json::from_slice(&frames.last().unwrap()[9..]).unwrap();
    assert_eq!(error["detail"]["kind"], "superseded");
    assert_eq!(
        bridge::invoke(&harness.window, "rpc", fenced(generation, &[]), &json!({"dokime-rpc-path": "/dokime.v1.Backend/GetHealth"}),)
            .unwrap_err()["detail"]["kind"],
        "superseded"
    );
    fixture.close().await;
}

#[tokio::test]
async fn client_close_drops() {
    use dokime::client::{Client, Error, Target};

    let fixture = Fixture::new().await;
    let client = Client::connect(Target::local(fixture.server.local_endpoint()).unwrap(), None, None).await.unwrap();
    let (entered, ready) = tokio::sync::oneshot::channel();
    let (dropped, released) = tokio::sync::oneshot::channel();
    let uploading = client.clone();
    let task = tokio::spawn(async move {
        uploading
            .plugins_install(InstallInput {
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
