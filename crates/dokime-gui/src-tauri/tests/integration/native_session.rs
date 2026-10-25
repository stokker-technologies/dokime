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
use crate::support::server::{Published, attach};
use dokime_gui::connection::ConnectionSession;
use dokime_gui::native_session::{BackendChoice, BackendOwnership, BackendSelection, DesktopSession, DesktopSessionError, preferences};

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
#[tokio::test]
async fn attached_backend_survives() {
    let root = tempfile::tempdir().unwrap();
    let backend = Published::start(root.path()).await;
    let endpoint = backend.endpoint();
    let attachment = DesktopSession::attached(ConnectionSession::default());
    assert!(matches!(attachment.select(BackendSelection::OwnedLocal).await, Err(DesktopSessionError::LocalBackendUnavailable)));
    assert_eq!(attachment.select(attach(endpoint.clone())).await.unwrap().ownership, BackendOwnership::Attached);
    attachment.shutdown().await.unwrap();
    let probe = ConnectionSession::default();
    if let BackendSelection::Attach { request } = attach(endpoint) {
        probe.connect(request).await.unwrap();
    }
    probe.disconnect().unwrap();
    backend.shutdown().await;
}

#[tokio::test]
async fn disconnect_cancels_discovery() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let owner = DesktopSession::attached(ConnectionSession::default());
    let connecting = owner.clone();
    let task = tokio::spawn(async move { connecting.select(attach(endpoint)).await });
    // TCP acceptance proves an actual connection attempt is pending; no timing sleeps.
    let (_socket, _) = listener.accept().await.unwrap();
    owner.disconnect().await.unwrap();
    assert!(matches!(task.await.unwrap(), Err(DesktopSessionError::Superseded)));
    assert_eq!(owner.status().ownership, BackendOwnership::None);
}

#[tokio::test]
async fn invalid_local_attachment() {
    let root = tempfile::tempdir().unwrap();
    let backend = Published::start(root.path()).await;
    let desktop = DesktopSession::with_local_configuration(ConnectionSession::default(), Err("Settings could not be read".into()));
    assert!(matches!(desktop.select(BackendSelection::OwnedLocal).await, Err(DesktopSessionError::Startup { .. })));
    assert_eq!(desktop.select(attach(backend.endpoint())).await.unwrap().ownership, BackendOwnership::Attached);
    desktop.shutdown().await.unwrap();
    backend.shutdown().await;
}

#[tokio::test]
async fn profile_choice_survives() {
    let root = tempfile::tempdir().unwrap();
    let backend = Published::start(root.path()).await;
    let catalog = dokime::client::profiles::Store::new(root.path().join("backends.json"));
    catalog
        .update(|document| {
            for name in ["cli", "lab"] {
                document.profiles.push(dokime::client::profiles::Profile {
                    name: name.to_owned(),
                    endpoint: backend.endpoint(),
                    expected_backend_id: None,
                    ca_certificate_pem: None,
                    secret: dokime::client::profiles::Secret::None,
                });
            }
            document.default = Some("cli".to_owned());
            Ok(())
        })
        .unwrap();
    let before = std::fs::read(catalog.path()).unwrap();
    let path = root.path().join("desktop.json");
    assert_eq!(preferences::load(&path).unwrap(), BackendChoice::OwnedLocal);
    let desktop = DesktopSession::attached(ConnectionSession::default())
        .with_preferences(path.clone(), BackendChoice::OwnedLocal)
        .with_profile_store(catalog.path().to_owned());

    let selected = desktop.select(BackendSelection::Profile { name: "lab".to_owned() }).await.unwrap();
    let remembered = BackendChoice::Profile { name: "lab".to_owned() };
    assert_eq!(selected.selection, Some(remembered.clone()));
    assert_eq!(selected.ownership, BackendOwnership::Attached);
    assert_eq!(preferences::load(&path).unwrap(), remembered);
    assert!(desktop.select(BackendSelection::Profile { name: "missing".to_owned() }).await.is_err());
    assert_eq!(desktop.status().selection, Some(remembered.clone()));
    assert_eq!(desktop.status().connection.generation, selected.connection.generation);
    assert_eq!(desktop.status().ownership, BackendOwnership::Attached);
    assert_eq!(preferences::load(&path).unwrap(), remembered);
    desktop.disconnect().await.unwrap();
    desktop.select(attach(backend.endpoint())).await.unwrap();
    assert_eq!(desktop.status().selection, None);
    assert_eq!(preferences::load(&path).unwrap(), remembered);
    assert!(desktop.profile_selected("lab").unwrap());
    assert_eq!(std::fs::read(catalog.path()).unwrap(), before);
    desktop.disconnect().await.unwrap();

    assert!(desktop.select(BackendSelection::Profile { name: "missing".to_owned() }).await.is_err());
    assert_eq!(desktop.status().selection, None);
    assert_eq!(desktop.status().ownership, BackendOwnership::None);
    assert_eq!(preferences::load(&path).unwrap(), remembered);
    assert_eq!(std::fs::read(catalog.path()).unwrap(), before);
    desktop.shutdown().await.unwrap();
    let saved = std::fs::read(&path).unwrap();
    let closed = desktop.status();
    let rejected = desktop.select(BackendSelection::Profile { name: "lab".to_owned() }).await;
    assert!(matches!(rejected, Err(DesktopSessionError::ShuttingDown)));
    assert_eq!(desktop.status().selection, closed.selection);
    assert_eq!(desktop.status().connection.generation, closed.connection.generation);
    assert_eq!(std::fs::read(&path).unwrap(), saved);
    backend.shutdown().await;
}

#[test]
fn invalid_choice_preserved() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("desktop.json");
    let bytes = br#"{"kind":"attach","endpoint":"http://127.0.0.1:1234"}"#;
    std::fs::write(&path, bytes).unwrap();
    assert!(preferences::load(&path).is_err());
    assert_eq!(std::fs::read(path).unwrap(), bytes);
}
