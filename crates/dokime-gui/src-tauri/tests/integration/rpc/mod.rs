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

#[cfg(feature = "testing")]
mod native {
    /* === Modules === */

    /* === Re-Exports === */

    /* === Imports === */

    use super::*;
    use dokime_gui::testing::{DroppedFiles, InstallProgress, desktop_plugin_install_file};
    use tauri::Manager;

    /* === Constants === */

    /* === Macros === */

    /* === Type Aliases === */

    /* === Structs === */

    /* === Enums === */

    /* === Traits === */

    /* === Implementations === */

    /* === Public Functions === */

    /* === Private Functions === */

    fn app_and_window(session: &dokime_gui::connection::ConnectionSession) -> (tauri::App<tauri::test::MockRuntime>, bridge::Window) {
        let app = tauri::test::mock_builder()
            .manage(session.clone())
            .manage(DroppedFiles::default())
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default()).build().unwrap();
        (app, window)
    }

    async fn install_file(
        app: &tauri::App<tauri::test::MockRuntime>, window: &bridge::Window, drop: bool,
    ) -> Result<Option<String>, String> {
        let session = app.state::<dokime_gui::connection::ConnectionSession>();
        let generation = session.status().generation;
        desktop_plugin_install_file(
            window.clone(),
            session,
            app.state::<DroppedFiles>(),
            app.handle().clone(),
            generation,
            drop,
            tauri::ipc::Channel::<InstallProgress>::new(|_| Ok(())),
        )
        .await
    }

    /* === Tests === */

    #[test]
    fn invalid_bundle_returns_error_then_valid_file_installs() {
        dokime_gui::test_support::run(async {
            // A Failed File Installation Leaves the Native Command Usable
            let fixture = Fixture::new().await;
            let (app, window) = app_and_window(&fixture.session);
            let root = dokime_gui::test_support::tempdir().unwrap();
            let path = root.path().join("plugin.dokime");
            std::fs::write(&path, b"not a plugin bundle").unwrap();
            app.state::<DroppedFiles>().record("main", std::slice::from_ref(&path));
            assert!(!install_file(&app, &window, true).await.unwrap_err().is_empty());
            std::fs::write(&path, bundle(1024, native_target())).unwrap();
            app.state::<DroppedFiles>().record("main", &[path]);
            assert!(install_file(&app, &window, true).await.unwrap().is_some());
            fixture.close().await;
        })
    }

    #[test]
    fn unreadable_file_returns_error() {
        dokime_gui::test_support::run(async {
            // A Missing Dropped File Returns an Actionable Native Error
            let fixture = Fixture::new().await;
            let (app, window) = app_and_window(&fixture.session);
            let root = dokime_gui::test_support::tempdir().unwrap();
            app.state::<DroppedFiles>().record("main", &[root.path().join("missing.dokime")]);
            assert!(install_file(&app, &window, true).await.is_err());
            fixture.close().await;
        })
    }

    #[test]
    fn missing_picker_returns_error() {
        dokime_gui::test_support::run(async {
            // Missing Dialog State Must Return to the Frontend Without Panicking
            let fixture = Fixture::new().await;
            let (app, window) = app_and_window(&fixture.session);
            assert!(install_file(&app, &window, false).await.is_err());
            fixture.close().await;
        })
    }
}

/* === Re-Exports === */

/* === Imports === */

use dokime_protocol::client;

// Third Party Imports
use prost::Message;
use serde_json::json;
use sha2::{Digest, Sha256};

// Local Imports
use crate::support::bridge::{self, Harness, fenced};
use crate::support::server::Fixture;
use dokime_gui::rpc::{Bridge, FRAME_END, FRAME_ERROR, FRAME_MESSAGE};
use dokime_sdk::client::v1 as library;

/* === Constants === */

const INSTALL: &str = "/dokime.client.v1.Plugins/Install";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn invoke(window: &bridge::Window, command: &str, bytes: Vec<u8>) -> Result<tauri::ipc::InvokeResponseBody, serde_json::Value> {
    bridge::invoke(window, command, bytes, &serde_json::Value::Null)
}

fn install_item(kind: client::install_plugin_frame_frame::Kind) -> client::InstallPluginFrame {
    client::InstallPluginFrame {
        frame: Some(client::InstallPluginFrameFrame { kind: Some(kind) }),
    }
}

fn install_header(size_bytes: u64, sha256: String) -> Vec<u8> {
    client::PluginsInstallInput {
        value: Some(client::plugins_install_input::Value::Item(install_item(client::install_plugin_frame_frame::Kind::Header(
            client::InstallPluginHeader { size_bytes, sha256 },
        )))),
    }
    .encode_to_vec()
}

fn install(harness: &Harness, generation: u64, bytes: &[u8]) -> Vec<Vec<u8>> {
    // Open the Canonical Client Input and Stream Bounded Chunks
    let opening = client::PluginsInstallInput {
        value: Some(client::plugins_install_input::Value::Request(client::PluginsInstallRequest {})),
    };
    let (stream, channel) = harness.open(generation, INSTALL, &opening.encode_to_vec(), true).unwrap();
    harness
        .control("rpc_stream_send", generation, stream, &install_header(bytes.len() as u64, hex(&Sha256::digest(bytes))), json!({}))
        .unwrap();
    for (index, data) in bytes.chunks(256 * 1024).enumerate() {
        let frame = client::PluginsInstallInput {
            value: Some(client::plugins_install_input::Value::Item(install_item(client::install_plugin_frame_frame::Kind::Chunk(
                client::InstallPluginChunk {
                    offset: (index * 256 * 1024) as u64,
                    data: data.to_vec(),
                },
            )))),
        };
        harness.control("rpc_stream_send", generation, stream, &frame.encode_to_vec(), json!({})).unwrap();
    }
    harness.control("rpc_stream_finish", generation, stream, &[], json!({})).unwrap();
    harness.settle(channel)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn bundle(resource_bytes: usize, target: &str) -> Vec<u8> {
    // Package the Prepared SDK Executable, Never Compile from a Story
    let executable = std::env::var_os("DOKIME_THERMAL_PLUGIN")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| dokime_gui::test_support::fixture("dokime-rust-thermal"));
    let executable = std::fs::read(executable).expect("Build Host Fixtures Before the Desktop Stories");
    let compatibility = br#"{"schema":1,"requires_dokime":">=0.1.0"}"#;
    let resource = vec![0xCD; resource_bytes];
    let filename = format!("bin/thermal{}", std::env::consts::EXE_SUFFIX);
    let manifest = serde_json::to_vec(&json!({"schema":1, "identity":{"name":"desktop-fixture", "version":"1.0.0", "description":"Desktop Fixture"},
        "files":[
            {"path":filename, "size":executable.len(), "sha256":hex(&Sha256::digest(&executable)), "executable":{"facility":"sidecar", "target":target}},
            {"path":"dokime.json", "size":compatibility.len(), "sha256":hex(&Sha256::digest(compatibility))},
            {"path":"resources/payload.bin", "size":resource.len(), "sha256":hex(&Sha256::digest(&resource))}
        ]})).unwrap();
    let mut archive = tar::Builder::new(Vec::new());
    for (path, bytes) in [
        ("manifest.json", manifest.as_slice()),
        (filename.as_str(), executable.as_slice()),
        ("dokime.json", compatibility.as_slice()),
        ("resources/payload.bin", resource.as_slice()),
    ] {
        let mut header = tar::Header::new_ustar();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o700);
        header.set_cksum();
        archive.append_data(&mut header, path, bytes).unwrap();
    }
    archive.into_inner().unwrap()
}

fn native_target() -> &'static str {
    if cfg!(target_os = "windows") { "x86_64-pc-windows-msvc" } else { "x86_64-unknown-linux-gnu" }
}

/* === Tests === */

#[test]
fn refuses_invalid_dispatch() {
    dokime_gui::test_support::run(async {
        // Invalid Envelopes Are Refused Before Host Dispatch
        let fixture = Fixture::new().await;
        let harness = Harness::new(fixture.session.clone());
        let generation = fixture.generation();
        for command in ["rpc", "rpc_stream_open"] {
            for (bytes, kind) in
                [(vec![], "invalid_request"), (vec![0; 40], "incompatible_descriptor"), (vec![0; 4_194_304 + 41], "limit_exceeded")]
            {
                assert_eq!(invoke(&harness.window, command, bytes).unwrap_err()["kind"], kind);
            }
            for path in [None, Some("harmos.v1.Status/Get"), Some("/a/b/c")] {
                let headers = path.map_or(json!({}), |path| json!({"dokime-rpc-path":path}));
                assert_eq!(
                    bridge::invoke(&harness.window, command, fenced(generation, &[]), &headers).unwrap_err()["kind"],
                    "invalid_request"
                );
            }
        }
        assert_eq!(
            bridge::invoke(&harness.window, "rpc", fenced(generation + 1, &[]), &json!({"dokime-rpc-path":"/harmos.v1.Status/Get"}))
                .unwrap_err()["detail"]["kind"],
            "superseded"
        );
        fixture.close().await;
    })
}

#[test]
fn unary_call_bridges() {
    dokime_gui::test_support::run(async {
        // Canonical Library Status Crosses Tauri with Its Generation Fence
        let fixture = Fixture::new().await;
        let harness = Harness::new(fixture.session.clone());
        let generation = fixture.generation();
        let reply =
            bridge::invoke(&harness.window, "rpc", fenced(generation, &[]), &json!({"dokime-rpc-path":"/harmos.v1.Status/Get"})).unwrap();
        let tauri::ipc::InvokeResponseBody::Raw(reply) = reply else {
            panic!("Status Replies with Canonical Bytes");
        };
        assert_eq!(u64::from_le_bytes(reply[..8].try_into().unwrap()), generation);
        let identity = library::StatusGetResponse::decode(&reply[8..]).unwrap().status.unwrap().identity.unwrap();
        assert_eq!(identity.host_id, fixture.session.status().backend.unwrap().backend_id);
        let error =
            bridge::invoke(&harness.window, "rpc", fenced(generation, &[]), &json!({"dokime-rpc-path":"/dokime.client.v1.Missing/Call"}))
                .unwrap_err();
        assert_eq!(error["kind"], "grpc");
        fixture.close().await;
    })
}

#[test]
fn owner_cancellation_health() {
    dokime_gui::test_support::run(async {
        use tauri::Manager;
        // Releasing a WebView Cancels Its Input and Leaves the Host Available
        let fixture = Fixture::new().await;
        let harness = Harness::new(fixture.session.clone());
        let generation = fixture.generation();
        let opening = client::PluginsInstallInput {
            value: Some(client::plugins_install_input::Value::Request(client::PluginsInstallRequest {})),
        };
        let (stream, channel) = harness.open(generation, INSTALL, &opening.encode_to_vec(), true).unwrap();
        harness.app.state::<Bridge>().cancel_owner("main");
        assert_eq!(
            harness.control("rpc_stream_send", generation, stream, &install_header(1, "00".repeat(32)), json!({})).unwrap_err()["kind"],
            "unknown_stream"
        );
        assert_eq!(harness.settle(channel).last().unwrap()[8], FRAME_ERROR);
        let client = fixture.client().await;
        let _: library::StatusGetResponse = client.unary("/harmos.v1.Status/Get", library::StatusGetRequest {}).await.unwrap();
        fixture.close().await;
    })
}

#[test]
fn generation_fences_streams() {
    dokime_gui::test_support::run(async {
        // Open the Host Watch Before Invalidating Its Native Generation
        let fixture = Fixture::new().await;
        let harness = Harness::new(fixture.session.clone());
        let generation = fixture.generation();
        let (_, channel) = harness.open(generation, "/dokime.client.v1.backend.Overview/Watch", &[], false).unwrap();
        fixture.session.disconnect().unwrap();
        let frames = harness.settle(channel);
        assert!(frames.iter().all(|frame| u64::from_le_bytes(frame[..8].try_into().unwrap()) == generation));
        assert_eq!(frames.last().unwrap()[8], FRAME_ERROR);
        let error: serde_json::Value = serde_json::from_slice(&frames.last().unwrap()[9..]).unwrap();
        assert_eq!(error["detail"]["kind"], "superseded");
        fixture.close().await;
    })
}

#[test]
fn installs_large_bundle() {
    dokime_gui::test_support::run(async {
        // A Large Verified SDK Bundle Crosses Bounded Native Input Frames
        let fixture = Fixture::new().await;
        let harness = Harness::new(fixture.session.clone());
        let bytes = bundle(70 * 1024 * 1024, native_target());
        let frames = install(&harness, fixture.generation(), &bytes);
        assert_eq!(frames.len(), 2);
        assert_eq!(frames[0][8], FRAME_MESSAGE);
        assert_eq!(frames[1][8], FRAME_END);
        let installation = client::PluginInstallation::decode(&frames[0][9..]).unwrap();
        assert_eq!(installation.sha256, hex(&Sha256::digest(&bytes)));
        assert_eq!(installation.state, client::PluginInstallationState::Installed as i32);
        fixture.close().await;
    })
}

#[test]
fn refuses_foreign_bundle() {
    dokime_gui::test_support::run(async {
        // Target Selection Belongs to the Host that Receives the Bundle
        let target = if cfg!(target_os = "windows") { "x86_64-unknown-linux-gnu" } else { "x86_64-pc-windows-msvc" };
        let fixture = Fixture::new().await;
        let harness = Harness::new(fixture.session.clone());
        let frames = install(&harness, fixture.generation(), &bundle(1024, target));
        assert_eq!(frames.last().unwrap()[8], FRAME_ERROR);
        let error: serde_json::Value = serde_json::from_slice(&frames.last().unwrap()[9..]).unwrap();
        assert_eq!(error["kind"], "grpc");
        assert_eq!(error["detail"]["code"], tonic::Code::InvalidArgument as i32);
        let page: client::InstallationPage = fixture
            .client()
            .await
            .unary("/dokime.client.v1.Plugins/ListInstallations", client::PluginsListInstallationsRequest::default())
            .await
            .unwrap();
        assert!(page.installations.is_empty());
        fixture.close().await;
    })
}
