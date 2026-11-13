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

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn owner(root: &Path) -> DesktopSession {
    DesktopSession::with_local_backend(ConnectionSession::default(), Backend::new(root.join("store")))
}

fn reason(result: Result<dokime_gui::native_session::DesktopSessionSnapshot, DesktopSessionError>) -> Option<StorageResetReason> {
    match result {
        Err(DesktopSessionError::StorageReset { reason, .. }) => Some(reason),
        _ => None,
    }
}

/* === Tests === */

#[test]
fn shutdown_releases_store() {
    dokime_gui::test_support::run(async {
        let root = dokime_gui::test_support::tempdir().unwrap();
        let connection = ConnectionSession::default();
        let owner = DesktopSession::with_local_backend(connection.clone(), Backend::new(root.path().join("store")));
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
    })
}

#[test]
fn blocked_store_recovers() {
    dokime_gui::test_support::run(async {
        let root = dokime_gui::test_support::tempdir().unwrap();
        let store = root.path().join("store");
        fs::write(&store, b"occupied").unwrap();
        let owner = owner(root.path());
        assert!(matches!(owner.select(BackendSelection::OwnedLocal).await, Err(DesktopSessionError::Startup { .. })));
        assert_eq!(owner.status().ownership, BackendOwnership::None);
        fs::remove_file(&store).unwrap();
        assert_eq!(owner.select(BackendSelection::OwnedLocal).await.unwrap().ownership, BackendOwnership::OwnedLocal);
        owner.shutdown().await.unwrap();
    })
}

#[test]
fn start_reports_phases() {
    dokime_gui::test_support::run(async {
        let root = dokime_gui::test_support::tempdir().unwrap();
        let mut phases = Vec::new();
        let backend = Backend::new(root.path().join("store"));
        let (mut session, endpoint) = backend::start(&backend, |phase| phases.push(phase.to_owned())).await.unwrap();
        assert_eq!(phases, ["starting"]);
        assert!(endpoint.starts_with("http://127.0.0.1:"));
        session.close().await.unwrap();
    })
}

#[test]
fn live_owner_refused() {
    dokime_gui::test_support::run(async {
        let root = dokime_gui::test_support::tempdir().unwrap();
        let first = owner(root.path());
        let initial = first.select(BackendSelection::OwnedLocal).await.unwrap();
        let successor = owner(root.path());
        let error = successor.select(BackendSelection::OwnedLocal).await.unwrap_err();
        assert!(matches!(&error, DesktopSessionError::BackendRunning { root: data, .. } if data == &root.path().join("store")));
        let serialized = serde_json::to_value(error).unwrap();
        assert_eq!(serialized["kind"], "backend_running");
        assert_eq!(serialized["root"], root.path().join("store").to_str().unwrap());
        assert_eq!(successor.status().ownership, BackendOwnership::None);
        assert_eq!(successor.status().phase, None);
        let attached = successor.select(attach(initial.connection.endpoint.clone().unwrap())).await.unwrap();
        assert_eq!(attached.ownership, BackendOwnership::Attached);
        assert_eq!(attached.connection.backend.as_ref().unwrap().backend_id, initial.connection.backend.as_ref().unwrap().backend_id);
        successor.disconnect().await.unwrap();
        assert_eq!(first.status().connection.generation, initial.connection.generation);
        first.shutdown().await.unwrap();
        successor.select(BackendSelection::OwnedLocal).await.unwrap();
        successor.shutdown().await.unwrap();
    })
}

#[test]
fn failed_start_releases() {
    dokime_gui::test_support::run(async {
        let root = dokime_gui::test_support::tempdir().unwrap();
        let store = root.path().join("store");
        fs::create_dir(&store).unwrap();
        fs::write(store.join("foreign-evidence"), b"preserved").unwrap();
        let log = root.path().join("application.log");
        let failed =
            DesktopSession::with_local_backend(ConnectionSession::default(), Backend::new(store.clone()).with_log(Some(log.clone())));
        let Err(DesktopSessionError::Startup { diagnostics: Some(diagnostics), .. }) = failed.select(BackendSelection::OwnedLocal).await
        else {
            panic!("an incompatible root refuses startup");
        };
        assert!(!diagnostics.cause.is_empty());
        assert_eq!(failed.startup_log(), Some(log));
        assert_eq!(fs::read(store.join("foreign-evidence")).unwrap(), b"preserved");
        failed.disconnect().await.unwrap();
        assert_eq!(failed.startup_log(), None);
        // Refusal Releases the Root Lease Without Modifying Evidence.
        let lease = fs::OpenOptions::new().read(true).write(true).open(store.join("local.lock")).unwrap();
        lease.try_lock().unwrap();
    })
}

#[test]
fn overlapping_selection() {
    dokime_gui::test_support::run(async {
        let root = dokime_gui::test_support::tempdir().unwrap();
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
    })
}

#[test]
fn unsupported_reset_preserves_live_host() {
    dokime_gui::test_support::run(async {
        let root = dokime_gui::test_support::tempdir().unwrap();
        let store = root.path().join("store");
        let storage = StorageRoot::new(store.clone(), root.path().join("application"));
        let owner = owner(root.path());
        let started = owner.select(BackendSelection::OwnedLocal).await.unwrap();
        fs::write(store.join("operator-evidence"), b"preserved").unwrap();
        assert_eq!(reason(owner.reset_local_storage(&store, storage.clone()).await), Some(StorageResetReason::Failed));
        assert_eq!(
            reason(owner.backup_and_reset_local_storage(storage, root.path().join("backup.zip")).await),
            Some(StorageResetReason::Failed)
        );
        assert_eq!(owner.status().connection.generation, started.connection.generation);
        assert_eq!(fs::read(store.join("operator-evidence")).unwrap(), b"preserved");
        assert!(!root.path().join("backup.zip").exists());
        let client = owner.local_client().await.unwrap();
        let _: dokime_sdk::v1::ListCyclesReply = client
            .unary("/dokime.v1.Execution/ListCycles", dokime_sdk::v1::ExecutionListCyclesRequest { limit: 1, ..Default::default() })
            .await
            .unwrap();
        owner.shutdown().await.unwrap();
    })
}
