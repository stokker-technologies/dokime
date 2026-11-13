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

// Third Party Imports

// Local Imports
#[cfg(feature = "updater")]
use dokime_gui::testing::update_client;
use dokime_gui::testing::update_policy::Source;
use dokime_gui::testing::update_policy::{Mode, UpdatePolicy};
use dokime_gui::testing::{UpdateProbe, UpdateSelection as Selection};
#[cfg(feature = "updater")]
use std::{sync::Arc, time::Duration};
#[cfg(feature = "updater")]
use tauri_plugin_updater::UpdaterExt;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[cfg(feature = "updater")]
#[test]
fn downloaded_update_rejects_signature_with_compiled_trust() {
    dokime_gui::test_support::run(async {
        use rustls::pki_types::PrivatePkcs8KeyDer;
        use std::io::{Read, Write};
        use tauri::test::{mock_builder, mock_context, noop_assets};

        let certificate = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let trust = reqwest::Certificate::from_der(certificate.cert.der()).unwrap();
        let key = PrivatePkcs8KeyDer::from(certificate.signing_key.serialize_der());
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let config = rustls::ServerConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .unwrap()
            .with_no_client_auth()
            .with_single_cert(vec![certificate.cert.der().clone()], key.into())
            .unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let origin = format!("https://localhost:{}", listener.local_addr().unwrap().port());
        let manifest = serde_json::json!({
            "version": "999.0.0", "url": format!("{origin}/update"), "signature": "bm90IGEgc2lnbmF0dXJl"
        })
        .to_string();
        let runtime = tokio::runtime::Handle::current();
        let server = tokio::task::spawn_blocking(move || {
            let config = Arc::new(config);
            for body in [manifest.as_bytes(), b"untrusted update bytes".as_slice()] {
                let (socket, _) = runtime.block_on(tokio::time::timeout(Duration::from_secs(10), listener.accept())).unwrap().unwrap();
                let socket = socket.into_std().unwrap();
                socket.set_nonblocking(false).unwrap();
                socket.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
                socket.set_write_timeout(Some(Duration::from_secs(10))).unwrap();
                let connection = rustls::ServerConnection::new(Arc::clone(&config)).unwrap();
                let mut stream = rustls::StreamOwned::new(connection, socket);
                let mut request = Vec::new();
                while !request.ends_with(b"\r\n\r\n") {
                    let mut byte = [0];
                    stream.read_exact(&mut byte).unwrap();
                    request.push(byte[0]);
                    assert!(request.len() < 16384);
                }
                write!(stream, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len()).unwrap();
                stream.write_all(body).unwrap();
                stream.flush().unwrap();
            }
        });
        let compiled: serde_json::Value = serde_json::from_str(include_str!("../../../tauri.conf.json")).unwrap();
        let key = compiled["plugins"]["updater"]["pubkey"].as_str().unwrap();
        let mut context = mock_context(noop_assets());
        context.config_mut().plugins.0.insert("updater".into(), compiled["plugins"]["updater"].clone());
        let app = mock_builder().plugin(tauri_plugin_updater::Builder::new().pubkey(key).build()).build(context).unwrap();
        let update = app
            .updater_builder()
            .endpoints(vec![format!("{origin}/manifest").parse().unwrap()])
            .unwrap()
            .configure_client(move |builder| {
                update_client(builder.add_root_certificate(trust.clone())).timeout(Duration::from_secs(10)).no_proxy()
            })
            .build()
            .unwrap()
            .check()
            .await
            .unwrap()
            .unwrap();
        let mut downloaded = 0;
        let result = update.download(|bytes, _| downloaded += bytes, || {}).await;
        assert_eq!(downloaded, b"untrusted update bytes".len());
        assert!(matches!(result, Err(tauri_plugin_updater::Error::Minisign(_))));
        server.await.unwrap();
    })
}

#[test]
fn invalid_selection_preserves_saved_policy_and_switching_invalidates_review() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("selection.json");
    let mut policy = UpdatePolicy::default();
    policy.sources.insert("public".into(), Source::Github { repository: "example/product".into() });
    let state = UpdateProbe::new(path.clone(), policy);
    state
        .configure(Selection {
            mode: Mode::Manual,
            source: Some("public".into()),
        })
        .unwrap();
    let saved = std::fs::read(&path).unwrap();
    let generation = state.generation().unwrap();
    assert!(
        state
            .configure(Selection {
                mode: Mode::AutomaticCheck,
                source: Some("missing".into())
            })
            .is_err()
    );
    assert_eq!(std::fs::read(&path).unwrap(), saved);
    assert_eq!(state.generation().unwrap(), generation);

    state.configure(Selection { mode: Mode::Disabled, source: None }).unwrap();
    assert_eq!(state.status().unwrap().mode, Mode::Disabled);
    assert_ne!(state.generation().unwrap(), generation);
    assert!(state.status().unwrap().last_check.is_none());
}
