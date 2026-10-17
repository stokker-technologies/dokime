/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

//! Desktop Session Attachment Stories Against An In-Process Backend.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::support::server::{Published, attach};
use dokime_gui::connection::ConnectionSession;
use dokime_gui::native_session::{
    BackendOwnership, BackendSelection, DesktopSession, DesktopSessionError,
};

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
    assert!(matches!(
        attachment.select(BackendSelection::OwnedLocal).await,
        Err(DesktopSessionError::LocalBackendUnavailable)
    ));
    assert_eq!(
        attachment
            .select(attach(endpoint.clone()))
            .await
            .unwrap()
            .ownership,
        BackendOwnership::Attached
    );
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
    assert!(matches!(
        task.await.unwrap(),
        Err(DesktopSessionError::Superseded)
    ));
    assert_eq!(owner.status().ownership, BackendOwnership::None);
}

#[tokio::test]
async fn invalid_local_attachment() {
    let root = tempfile::tempdir().unwrap();
    let backend = Published::start(root.path()).await;
    let desktop = DesktopSession::with_local_configuration(
        ConnectionSession::default(),
        Err("Settings could not be read".into()),
    );
    assert!(matches!(
        desktop.select(BackendSelection::OwnedLocal).await,
        Err(DesktopSessionError::Startup { .. })
    ));
    assert_eq!(
        desktop
            .select(attach(backend.endpoint()))
            .await
            .unwrap()
            .ownership,
        BackendOwnership::Attached
    );
    desktop.shutdown().await.unwrap();
    backend.shutdown().await;
}
