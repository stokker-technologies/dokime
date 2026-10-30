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
use dokime_gui::native_session::{BackendOwnership, BackendSelection, DesktopSession, DesktopSessionError};

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
    let mut status_updates = attachment.subscribe_status();
    assert!(matches!(attachment.select(BackendSelection::OwnedLocal).await, Err(DesktopSessionError::LocalBackendUnavailable)));
    let attached = attachment.select(attach(endpoint.clone())).await.unwrap();
    assert_eq!(attached.ownership, BackendOwnership::Attached);
    status_updates.changed().await.unwrap();
    let announced = status_updates.borrow_and_update().clone();
    assert_eq!(announced.ownership, BackendOwnership::Attached);
    assert!(announced.status_revision >= attached.status_revision);
    attachment.shutdown().await.unwrap();
    status_updates.changed().await.unwrap();
    let stopped = status_updates.borrow_and_update().clone();
    assert_eq!(stopped.ownership, BackendOwnership::None);
    assert!(stopped.status_revision > announced.status_revision);
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
async fn owned_trace_tail() {
    use dokime::domain::model::backend::wire::{LogLevel, TraceScope};
    use dokime::operations::backend::WatchTrace;
    use dokime_gui::native_session::backend::Backend;
    use dokime_server::host::{Args, NetworkArgs};
    use tracing_subscriber::prelude::*;

    let root = tempfile::tempdir().unwrap();
    let journal = dokime_server::trace::Journal::new(dokime_server::trace::Limits::default());
    let capture = tracing::Dispatch::new(tracing_subscriber::registry().with(dokime_server::trace::layer(journal.clone())));
    let execution_id = "00000000-0000-0000-0000-000000000001";
    tracing::dispatcher::with_default(&capture, || tracing::info!(execution_id, "history"));
    let args = Args {
        shutdown_stdin: false,
        dev_control: false,
        network: NetworkArgs {
            host: std::net::Ipv4Addr::LOCALHOST.into(),
            port: Some(0),
            message_limit_mib: None,
        },
        title: None,
        label: None,
        output_root: Some(root.path().join("store")),
        application_root: Some(root.path().join("application")),
        env_file: None,
        plugins: Vec::new(),
        plugins_root: None,
    };
    let backend = Backend::new(args).with_trace_journal(journal);
    let owner = DesktopSession::with_local_backend(ConnectionSession::default(), backend);
    owner.select(BackendSelection::OwnedLocal).await.unwrap();
    let client = owner.local_client().await.unwrap();
    let stream = client
        .stream(WatchTrace {
            epoch: None,
            scope: Some(TraceScope::Execution(execution_id.to_owned())),
            min_level: LogLevel::Info,
            after_sequence: 0,
            max_events: 256,
            max_bytes: 262_144,
            follow: true,
        })
        .await
        .unwrap();
    let history = stream.message().await.unwrap().unwrap();
    assert_eq!(history.events.len(), 1);
    assert_eq!(history.events[0].execution_id.as_deref(), Some(execution_id));
    tracing::dispatcher::with_default(&capture, || tracing::warn!(execution_id, "live"));
    let live = stream.message().await.unwrap().unwrap();
    assert_eq!(live.events.len(), 1);
    assert_eq!(live.events[0].level, LogLevel::Warn);
    assert!(live.next_sequence > history.next_sequence);
    stream.close().await;
    owner.shutdown().await.unwrap();
}
