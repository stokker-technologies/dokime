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

// Third Party Imports
use harmos_runtime::bundle::Manifest;
use prost::Message;
use serde_json::json;
use sha2::{Digest, Sha256};

// Local Imports

use crate::support::bridge::{self, Harness, fenced};
use crate::support::server::Fixture;
use dokime_client_internal::PluginsListOptions;
use dokime_client_internal::StatusGetOptions;
use dokime_client_internal::host::PluginEntry;
use dokime_client_internal::host::plugin_placement;
use dokime_gui::rpc::{Bridge, FRAME_ERROR};
use harmos_protocol::host;

/* === Constants === */

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

async fn install(fixture: &Fixture, bytes: Vec<u8>) -> Result<PluginEntry, String> {
    let lease = fixture.session.lease(&fixture.generation().to_string()).unwrap();
    let progress = tauri::ipc::Channel::new(|_| Ok(()));
    dokime_gui::testing::install(&lease, bytes, progress, None).await
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
            {"path":filename, "size":executable.len(), "sha256":hex(&Sha256::digest(&executable)), "executable":{"facility":"sidecar", "target":target, "registrations":[]}},
            {"path":"dokime.json", "size":compatibility.len(), "sha256":hex(&Sha256::digest(compatibility))},
            {"path":"resources/payload.bin", "size":resource.len(), "sha256":hex(&Sha256::digest(&resource))}
        ]})).unwrap();
    let manifest: Manifest = serde_json::from_slice(&manifest).unwrap();
    let manifest = serde_json::to_vec(&manifest).unwrap();
    let mut archive = tar::Builder::new(Vec::new());
    for (path, bytes) in [
        ("manifest.json", manifest.as_slice()),
        (filename.as_str(), executable.as_slice()),
        ("dokime.json", compatibility.as_slice()),
        ("resources/payload.bin", resource.as_slice()),
    ] {
        let mut header = tar::Header::new_ustar();
        header.set_entry_type(tar::EntryType::Regular);
        header.set_uid(0);
        header.set_gid(0);
        header.set_mtime(0);
        header.set_size(bytes.len() as u64);
        header.set_mode(if path == filename { 0o755 } else { 0o644 });
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
            for path in [None, Some("dokime.host.v1.Status/Get"), Some("/a/b/c")] {
                let headers = path.map_or(json!({}), |path| json!({"dokime-rpc-path":path}));
                assert_eq!(
                    bridge::invoke(&harness.window, command, fenced(generation, &[]), &headers).unwrap_err()["kind"],
                    "invalid_request"
                );
            }
        }
        assert_eq!(
            bridge::invoke(&harness.window, "rpc", fenced(generation + 1, &[]), &json!({"dokime-rpc-path":"/dokime.host.v1.Status/Get"}))
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
            bridge::invoke(&harness.window, "rpc", fenced(generation, &[]), &json!({"dokime-rpc-path":"/dokime.host.v1.Status/Get"}))
                .unwrap();
        let tauri::ipc::InvokeResponseBody::Raw(reply) = reply else {
            panic!("Status Replies with Canonical Bytes");
        };
        assert_eq!(u64::from_le_bytes(reply[..8].try_into().unwrap()), generation);
        let identity = host::StatusGetResponse::decode(&reply[8..]).unwrap().status.unwrap().identity.unwrap();
        assert_eq!(identity.host_id, fixture.session.status().backend.unwrap().backend_id);
        let error =
            bridge::invoke(&harness.window, "rpc", fenced(generation, &[]), &json!({"dokime-rpc-path":"/dokime.host.v1.Missing/Call"}))
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
        let (stream, channel) = harness.open(generation, "/dokime.host.v1.backend.Overview/Watch", &[], false).unwrap();
        harness.app.state::<Bridge>().cancel_owner("main");
        assert_eq!(
            harness.control("rpc_stream_ack", generation, stream, &[], json!({"dokime-rpc-credit":"1"})).unwrap_err()["kind"],
            "unknown_stream"
        );
        assert_eq!(harness.settle(channel).last().unwrap()[8], FRAME_ERROR);
        let client = fixture.client().await;
        let response =
            client.execute(client.client().status.get_with(StatusGetOptions { context: client.call_options().unwrap() })).await.unwrap();
        assert!(response.status.unwrap().available);
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
        let (_, channel) = harness.open(generation, "/dokime.host.v1.backend.Overview/Watch", &[], false).unwrap();
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
        let bytes = bundle(70 * 1024 * 1024, native_target());
        let digest = Sha256::digest(&bytes).to_vec();
        let installed = install(&fixture, bytes).await.unwrap();
        let placement = installed.placement.unwrap().kind.unwrap();
        let plugin_placement::Kind::Managed(managed) = placement else {
            panic!("Expected Managed Placement");
        };
        assert_eq!(managed.sha256, digest);
        assert!(installed.value.unwrap().enabled);
        fixture.close().await;
    })
}

#[test]
fn refuses_foreign_bundle() {
    dokime_gui::test_support::run(async {
        // Target Selection Belongs to the Host that Receives the Bundle
        let target = if cfg!(target_os = "windows") { "x86_64-unknown-linux-gnu" } else { "x86_64-pc-windows-msvc" };
        let fixture = Fixture::new().await;
        assert!(install(&fixture, bundle(1024, target)).await.is_err());
        let client = fixture.client().await;
        let page = client
            .execute(client.client().plugins.list_with(PluginsListOptions {
                context: client.call_options().unwrap(),
                ..Default::default()
            }))
            .await
            .unwrap();
        assert!(page.entries.is_empty());
        fixture.close().await;
    })
}
