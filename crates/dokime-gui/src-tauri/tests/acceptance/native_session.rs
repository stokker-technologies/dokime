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
use std::{fs, path::Path};

// Third Party Imports

// Local Imports
use crate::support::server::attach;
use dokime_gui::connection::ConnectionSession;
use dokime_gui::native_session::backend::{self, Backend};
use dokime_gui::native_session::storage::StorageRoot;
use dokime_gui::native_session::{BackendOwnership, BackendSelection, DesktopSession, DesktopSessionError, StorageResetReason};
use dokime_server::host::{Args, NetworkArgs};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn arguments(root: &Path) -> Args {
    Args {
        shutdown_stdin: false,
        dev_control: false,
        network: NetworkArgs {
            host: std::net::Ipv4Addr::LOCALHOST.into(),
            port: Some(0),
            message_limit_mib: Some(2),
        },
        title: None,
        label: None,
        output_root: Some(root.join("store")),
        application_root: Some(root.join("application")),
        env_file: None,
        plugins: Vec::new(),
        plugins_root: None,
    }
}

fn owner(root: &Path) -> DesktopSession {
    DesktopSession::with_local_backend(ConnectionSession::default(), Backend::new(arguments(root)))
}

fn reason(result: Result<dokime_gui::native_session::DesktopSessionSnapshot, DesktopSessionError>) -> Option<StorageResetReason> {
    match result {
        Err(DesktopSessionError::StorageReset { reason, .. }) => Some(reason),
        _ => None,
    }
}

/* === Tests === */

#[tokio::test]
async fn shutdown_releases_store() {
    let root = tempfile::tempdir().unwrap();
    let connection = ConnectionSession::default();
    let owner = DesktopSession::with_local_backend(connection.clone(), Backend::new(arguments(root.path())));
    let started = owner.select(BackendSelection::OwnedLocal).await.unwrap();
    let endpoint = started.connection.endpoint.clone().unwrap();
    let lease = connection.lease(&started.connection.generation).unwrap();
    assert_eq!(started.ownership, BackendOwnership::OwnedLocal);
    owner.shutdown().await.unwrap();
    assert_eq!(
        lease.execute(std::future::pending::<Result<(), tonic::Status>>()).await,
        Err(dokime_gui::connection::ConnectionError::Superseded)
    );
    assert!(connection.lease(&started.connection.generation).is_err());
    let address = endpoint.strip_prefix("http://").unwrap().trim_end_matches('/');
    assert!(tokio::net::TcpStream::connect(address).await.is_err(), "shutdown releases the listener");
    assert!(matches!(owner.select(BackendSelection::OwnedLocal).await, Err(DesktopSessionError::ShuttingDown)));
    let replacement = self::owner(root.path());
    assert_eq!(replacement.select(BackendSelection::OwnedLocal).await.unwrap().ownership, BackendOwnership::OwnedLocal);
    replacement.shutdown().await.unwrap();
}

#[tokio::test]
async fn shutdown_retry_retains() {
    let root = tempfile::tempdir().unwrap();
    let backend = Backend::new(arguments(root.path()));
    let (mut session, endpoint) = backend::start(&backend, |_| {}).await.unwrap();
    let database = dokime::storage::sqlite::rusqlite::Connection::open(root.path().join("store/local.sqlite")).unwrap();
    database.execute_batch("BEGIN IMMEDIATE").unwrap();

    tokio::time::pause();
    let mut shutdown = session.shutdown();
    std::future::poll_fn(|context| {
        assert!(shutdown.as_mut().poll(context).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    let budget = dokime::operations::STOP_SETTLEMENT.timeout + std::time::Duration::from_secs(21);
    tokio::time::advance(budget).await;
    assert!(matches!(shutdown.await, Err(dokime_server::host::Error::StopIncomplete { .. })));
    tokio::time::resume();
    let lease = std::fs::OpenOptions::new().read(true).write(true).open(root.path().join("store/local.lock")).unwrap();
    assert!(matches!(lease.try_lock(), Err(std::fs::TryLockError::WouldBlock)));

    let mut retry = session.shutdown();
    std::future::poll_fn(|context| {
        assert!(retry.as_mut().poll(context).is_pending());
        std::task::Poll::Ready(())
    })
    .await;
    database.execute_batch("ROLLBACK").unwrap();
    retry.await.unwrap();
    lease.try_lock().unwrap();
    let address = endpoint.strip_prefix("http://").unwrap().trim_end_matches('/');
    assert!(tokio::net::TcpStream::connect(address).await.is_err());
}

#[tokio::test]
async fn blocked_store_recovers() {
    let root = tempfile::tempdir().unwrap();
    let store = root.path().join("store");
    fs::write(&store, b"occupied").unwrap();
    let owner = owner(root.path());
    assert!(matches!(owner.select(BackendSelection::OwnedLocal).await, Err(DesktopSessionError::Startup { .. })));
    assert_eq!(owner.status().ownership, BackendOwnership::None);
    fs::remove_file(&store).unwrap();
    assert_eq!(owner.select(BackendSelection::OwnedLocal).await.unwrap().ownership, BackendOwnership::OwnedLocal);
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn start_reports_phases() {
    let root = tempfile::tempdir().unwrap();
    let mut phases = Vec::new();
    let backend = Backend::new(arguments(root.path()));
    let (mut session, endpoint) = backend::start(&backend, |phase| phases.push(phase.to_owned())).await.unwrap();
    assert_eq!(phases, ["storage", "plugins", "requests"]);
    assert!(endpoint.starts_with("http://127.0.0.1:"));
    session.shutdown().await.unwrap();
}

#[tokio::test]
async fn live_owner_refused() {
    let root = tempfile::tempdir().unwrap();
    let first = owner(root.path());
    first.select(BackendSelection::OwnedLocal).await.unwrap();
    let successor = owner(root.path());
    assert!(matches!(successor.select(BackendSelection::OwnedLocal).await, Err(DesktopSessionError::Startup { .. })));
    assert_eq!(successor.status().ownership, BackendOwnership::None);
    first.shutdown().await.unwrap();
    successor.select(BackendSelection::OwnedLocal).await.unwrap();
    successor.shutdown().await.unwrap();
}

#[tokio::test]
async fn failed_start_releases() {
    let root = tempfile::tempdir().unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mut args = arguments(root.path());
    args.network.port = Some(listener.local_addr().unwrap().port());
    let log = root.path().join("application.log");
    let failed = DesktopSession::with_local_backend(ConnectionSession::default(), Backend::new(args).with_log(Some(log.clone())));
    let Err(DesktopSessionError::Startup { diagnostics: Some(diagnostics), .. }) = failed.select(BackendSelection::OwnedLocal).await else {
        panic!("the occupied listener refuses startup");
    };
    assert!(!diagnostics.cause.is_empty());
    assert_eq!(failed.startup_log(), Some(log));
    failed.disconnect().await.unwrap();
    assert_eq!(failed.startup_log(), None);
    let replacement = owner(root.path());
    replacement.select(BackendSelection::OwnedLocal).await.unwrap();
    replacement.shutdown().await.unwrap();
}

#[tokio::test]
async fn overlapping_selection() {
    let root = tempfile::tempdir().unwrap();
    let owner = owner(root.path());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let pending_endpoint = format!("http://{}", listener.local_addr().unwrap());
    let first_owner = owner.clone();
    let first = tokio::spawn(async move { first_owner.select(attach(pending_endpoint)).await });
    let (_socket, _) = listener.accept().await.unwrap();
    first.abort();
    assert!(first.await.unwrap_err().is_cancelled());
    let result = owner.select(BackendSelection::OwnedLocal).await.unwrap();
    assert_eq!(result.ownership, BackendOwnership::OwnedLocal);
    owner.shutdown().await.unwrap();
    assert_eq!(owner.status().ownership, BackendOwnership::None);
}

#[tokio::test]
async fn reset_restarts_backend() {
    let root = tempfile::tempdir().unwrap();
    let store = root.path().join("store");
    let application = root.path().join("application");
    let storage = StorageRoot::new(store.clone(), application.clone());
    let owner = owner(root.path());
    let started = owner.select(BackendSelection::OwnedLocal).await.unwrap();
    fs::write(store.join("obsolete.txt"), b"retained until confirmed reset").unwrap();
    for outside in [root.path(), application.as_path()] {
        assert_eq!(reason(owner.reset_local_storage(outside, storage.clone()).await), Some(StorageResetReason::OutsideRoot));
    }
    assert_eq!(owner.status().connection.generation, started.connection.generation);
    assert!(store.join("obsolete.txt").exists());
    let reset = owner.reset_local_storage(&store, storage).await.unwrap();
    assert_eq!(reset.ownership, BackendOwnership::OwnedLocal);
    assert_ne!(reset.connection.generation, started.connection.generation);
    assert!(!store.join("obsolete.txt").exists());
    assert_eq!(owner.status().phase, None);
    owner.shutdown().await.unwrap();
}
