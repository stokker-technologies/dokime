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
use crate::support::server::{Published, attach};
use dokime_gui::connection::ConnectionSession;
use dokime_gui::native_session::backend::Backend;
use dokime_gui::native_session::storage::StorageRoot;
use dokime_gui::native_session::{BackendOwnership, BackendSelection, DesktopSession, DesktopSessionError, StorageResetReason};

/* === Constants === */
const STDERR_LOG_PREFIX: &str = "backend-stderr.";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */
enum Publish<'a> {
    Ready(&'a str),
    ReadyWithPhases(&'a str, &'a [&'a str]),
    ReadyWithNoise(&'a str),
    Failure,
    Crash,
}

/* === Traits === */

/* === Implementations === */
/* === Public Functions === */

/* === Private Functions === */
fn stub(directory: &Path, publish: Publish<'_>) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let ready_line = |endpoint: &str| {
        serde_json::json!({
            "event": "ready",
            "protocol": 3,
            "endpoint": endpoint,
            "transport": "grpc",
            "version": env!("CARGO_PKG_VERSION"),
            "dev_protocol": 4,
        })
    };
    let (name, publish) = match publish {
        Publish::Ready(endpoint) => (
            "ready",
            format!(
                "printf '%s\\n' '{}'\ncat > /dev/null\nrm -f \"$owner\"",
                ready_line(endpoint)
            ),
        ),
        Publish::ReadyWithPhases(endpoint, phases) => {
            let mut script = String::new();
            for phase in phases {
                script.push_str(&format!(
                    "mkfifo \"$root/phase-ack-{phase}\"\nprintf '%s\\n' '{}'\nread acknowledgement < \"$root/phase-ack-{phase}\"\n",
                    serde_json::json!({ "event": "phase", "name": phase })
                ));
            }
            script.push_str(&format!(
                "printf '%s\\n' '{}'\ncat > /dev/null\nrm -f \"$owner\"",
                ready_line(endpoint)
            ));
            ("ready-with-phases", script)
        }
        Publish::ReadyWithNoise(endpoint) => (
            // Far more stderr than a pipe buffers, then a ready-shaped decoy on
            // stderr: readiness succeeds only if stderr is drained and only
            // stdout is parsed.
            "ready-with-noise",
            format!(
                "i=1\nwhile [ $i -le 3000 ]; do echo \"noise line $i padded well past the pipe buffer\" >&2; i=$((i+1)); done\nprintf '%s\\n' '{}' >&2\nprintf '%s\\n' '{}'\ncat > /dev/null\nrm -f \"$owner\"",
                ready_line("http://192.0.2.1:9"),
                ready_line(endpoint)
            ),
        ),
        Publish::Failure => (
            "failure",
            "echo \"plugin admission failed\" >&2\nexit 1".to_owned(),
        ),
        Publish::Crash => (
            "crash",
            "i=1\nwhile [ $i -le 250 ]; do echo \"startup diagnostic line $i\" >&2; i=$((i+1)); done\necho \"Authorization: Bearer s3cret-credential\" >&2\necho \"plugin admission failed\" >&2\nexit 3".to_owned(),
        ),
    };
    let path = directory.join(format!("dokime-stub-{name}"));
    fs::write(
        &path,
        format!(
            r#"#!/bin/sh
set -e
if [ "$1" = "version" ]; then
  printf '{{"version":"{version}","sdk_revision":null,"dev_protocol":4}}\n'
  exit 0
fi
root=""
while [ $# -gt 0 ]; do
  if [ "$1" = "--output-root" ]; then root="$2"; fi
  shift
done
mkdir -p "$root" 2>/dev/null || {{ echo "execution store $root is unusable" >&2; exit 1; }}
owner="$root/owner.pid"
if [ -f "$owner" ] && kill -0 "$(cat "$owner")" 2>/dev/null; then
  echo "execution store $root is already in use" >&2
  exit 1
fi
echo $$ > "$owner"
{publish}
"#,
            version = env!("CARGO_PKG_VERSION"),
        ),
    )
    .unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    // A parallel test may fork while this file is still open for writing; the
    // child keeps that descriptor until it execs, and executing the stub in
    // that window fails with ETXTBSY. Retry until the kernel accepts it; the
    // forking test execs or exits promptly, and the lane deadline bounds it.
    loop {
        match std::process::Command::new(&path).arg("version").output() {
            Err(error) if error.kind() == std::io::ErrorKind::ExecutableFileBusy => {
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            result => {
                result.expect("stub executable runs");
                break;
            }
        }
    }
    path
}

fn owned(executable: std::path::PathBuf, root: &Path) -> Backend {
    let arguments = vec!["--shutdown-stdin".into(), "--dev-control".into(), "--output-root".into(), root.display().to_string()];
    Backend::new(executable, arguments)
}

fn stderr_logs(directory: &Path) -> Vec<std::path::PathBuf> {
    fs::read_dir(directory)
        .map(|entries| {
            entries
                .map(|entry| entry.unwrap().path())
                .filter(|path| path.file_name().and_then(|name| name.to_str()).is_some_and(|name| name.starts_with(STDERR_LOG_PREFIX)))
                .collect()
        })
        .unwrap_or_default()
}

fn store_owner(root: &Path) -> Option<String> {
    fs::read_to_string(root.join("owner.pid")).ok()
}

fn reason(result: Result<dokime_gui::native_session::DesktopSessionSnapshot, DesktopSessionError>) -> Option<StorageResetReason> {
    match result {
        Err(DesktopSessionError::StorageReset { reason, .. }) => Some(reason),
        _ => None,
    }
}

fn populate(store: &std::path::Path, outside: &std::path::Path) {
    fs::write(store.join("local.sqlite"), b"rows").unwrap();
    fs::write(store.join("local.sqlite-wal"), b"wal").unwrap();
    fs::create_dir_all(store.join("artifacts")).unwrap();
    fs::write(store.join("artifacts/evidence.bin"), b"bytes").unwrap();
    fs::create_dir_all(store.join("uploads/pending")).unwrap();
    fs::write(outside.join("outside.txt"), b"not ours").unwrap();
    std::os::unix::fs::symlink(outside.join("outside.txt"), store.join("linked.txt")).unwrap();
}

/* === Tests === */
#[tokio::test]
async fn shutdown_releases_store() {
    let root = tempfile::tempdir().unwrap();
    let backend = Published::start(&root.path().join("published")).await;
    let store = root.path().join("store");
    let executable = stub(root.path(), Publish::Ready(&backend.endpoint()));
    let session = ConnectionSession::default();
    let owner = DesktopSession::with_local_backend(session.clone(), owned(executable.clone(), &store));
    let started = owner.select(BackendSelection::OwnedLocal).await.unwrap();
    assert_eq!(started.ownership, BackendOwnership::OwnedLocal);
    let lease = session.lease(&started.connection.generation).unwrap();
    assert!(store_owner(&store).is_some(), "the owned backend holds its store");

    owner.shutdown().await.unwrap();
    assert_eq!(
        lease.execute(std::future::pending::<Result<(), tonic::Status>>()).await,
        Err(dokime_gui::connection::ConnectionError::Superseded)
    );
    assert!(session.lease(&started.connection.generation).is_err());
    // Shutdown returns only after the backend drained and exited, so its store
    // is already released when the desktop moves on.
    assert_eq!(store_owner(&store), None);
    assert!(matches!(owner.select(BackendSelection::OwnedLocal).await, Err(DesktopSessionError::ShuttingDown)));

    // A second owner can acquire the exact execution store only after the first
    // drained; the backend refuses a store another live backend still holds.
    let replacement = DesktopSession::with_local_backend(ConnectionSession::default(), owned(executable, &store));
    assert_eq!(replacement.select(BackendSelection::OwnedLocal).await.unwrap().ownership, BackendOwnership::OwnedLocal);
    replacement.shutdown().await.unwrap();
    backend.shutdown().await;
}

#[tokio::test]
async fn blocked_store_recovers() {
    let root = tempfile::tempdir().unwrap();
    let backend = Published::start(&root.path().join("published")).await;
    let blocker = root.path().join("blocked");
    fs::write(&blocker, b"occupied").unwrap();
    let owner = DesktopSession::with_local_backend(
        ConnectionSession::default(),
        owned(stub(root.path(), Publish::Ready(&backend.endpoint())), &blocker),
    );
    let refused = owner.select(BackendSelection::OwnedLocal).await;
    assert!(matches!(refused, Err(DesktopSessionError::Startup { .. })));
    assert_eq!(owner.status().ownership, BackendOwnership::None);

    fs::remove_file(&blocker).unwrap();
    assert_eq!(owner.select(BackendSelection::OwnedLocal).await.unwrap().ownership, BackendOwnership::OwnedLocal);
    owner.shutdown().await.unwrap();
    backend.shutdown().await;
}

#[tokio::test]
async fn start_reports_phases() {
    let root = tempfile::tempdir().unwrap();
    let backend = Published::start(&root.path().join("published")).await;
    let store = root.path().join("store");
    let executable = stub(root.path(), Publish::ReadyWithPhases(&backend.endpoint(), &["storage", "requests"]));
    let owner = DesktopSession::with_local_backend(ConnectionSession::default(), owned(executable, &store));

    let selecting = owner.clone();
    let mut task = tokio::spawn(async move { selecting.select(BackendSelection::OwnedLocal).await });

    // TODO(source-hook): a DesktopSession status watch would replace this yield poll.
    let mut seen: Vec<String> = Vec::new();
    while seen.len() < 2 {
        if let Some(phase) = owner.status().phase
            && seen.last() != Some(&phase)
        {
            seen.push(phase.clone());
            // The backend waits for acknowledgement, so observing each
            // phase never depends on a scheduler delay.
            fs::write(store.join(format!("phase-ack-{phase}")), b"continue\n").unwrap();
        }
        if seen.len() == 2 {
            break;
        }
        tokio::select! {
            result = &mut task => {
                panic!("selection completed before observing both phases: {result:?}; observed: {seen:?}");
            }
            () = tokio::task::yield_now() => {}
        }
    }
    assert_eq!(seen, vec!["storage".to_owned(), "requests".to_owned()], "observed phases: {seen:?}");

    let started = task.await.unwrap().unwrap();
    assert_eq!(started.ownership, BackendOwnership::OwnedLocal);
    assert_eq!(owner.status().phase, None);

    owner.shutdown().await.unwrap();
    backend.shutdown().await;
}

#[tokio::test]
async fn live_owner_refused() {
    let root = tempfile::tempdir().unwrap();
    let backend = Published::start(&root.path().join("published")).await;
    let store = root.path().join("store");
    let executable = stub(root.path(), Publish::Ready(&backend.endpoint()));
    let stale = DesktopSession::with_local_backend(ConnectionSession::default(), owned(executable.clone(), &store));
    stale.select(BackendSelection::OwnedLocal).await.unwrap();

    let successor = DesktopSession::with_local_backend(ConnectionSession::default(), owned(executable, &store));
    let refused = successor.select(BackendSelection::OwnedLocal).await;
    assert!(matches!(refused, Err(DesktopSessionError::Startup { .. })));
    assert_eq!(successor.status().ownership, BackendOwnership::None);

    stale.shutdown().await.unwrap();
    assert_eq!(store_owner(&store), None);
    successor.select(BackendSelection::OwnedLocal).await.unwrap();
    successor.shutdown().await.unwrap();
    backend.shutdown().await;
}

#[tokio::test]
async fn failed_start_releases() {
    let root = tempfile::tempdir().unwrap();
    let backend = Published::start(&root.path().join("published")).await;
    let store = root.path().join("store");
    let owner = DesktopSession::with_local_backend(ConnectionSession::default(), owned(stub(root.path(), Publish::Failure), &store));
    let refused = owner.select(BackendSelection::OwnedLocal).await;
    assert!(matches!(refused, Err(DesktopSessionError::Startup { .. })));
    assert_eq!(owner.status().ownership, BackendOwnership::None);

    let successor = DesktopSession::with_local_backend(
        ConnectionSession::default(),
        owned(stub(root.path(), Publish::Ready(&backend.endpoint())), &store),
    );
    successor.select(BackendSelection::OwnedLocal).await.unwrap();
    successor.shutdown().await.unwrap();
    backend.shutdown().await;
}

#[tokio::test]
async fn crash_reports_diagnostics() {
    let root = tempfile::tempdir().unwrap();
    let logs = root.path().join("logs");
    let executable = stub(root.path(), Publish::Crash);
    let owner = DesktopSession::with_local_backend(
        ConnectionSession::default(),
        owned(executable.clone(), &root.path().join("store")).with_logs(logs.clone()),
    );
    let Err(DesktopSessionError::Startup { message, diagnostics: Some(diagnostics) }) = owner.select(BackendSelection::OwnedLocal).await
    else {
        panic!("a crashed backend reports startup diagnostics");
    };

    // The Headline And Exit
    let not_ready = format!("The {} backend stopped before reporting readiness; review its diagnostics.", dokime_gui::PRODUCT);
    assert_eq!(message, not_ready);
    assert_eq!(diagnostics.exit_code, Some(3));
    assert!(diagnostics.exit_status.is_some());
    assert_eq!(diagnostics.executable, executable.display().to_string());

    // The Bounded, Redacted Tail
    let tail: Vec<&str> = diagnostics.tail.lines().collect();
    assert_eq!(tail.len(), 200, "the tail keeps the last 200 lines");
    assert_eq!(tail.first(), Some(&"startup diagnostic line 53"));
    assert_eq!(tail.last(), Some(&"plugin admission failed"));
    assert!(tail.contains(&"Authorization: <redacted>"));
    assert!(!diagnostics.tail.contains("s3cret"));

    // The Complete, Redacted Log
    let log = std::path::PathBuf::from(diagnostics.log.expect("a log directory yields a log"));
    assert_eq!(stderr_logs(&logs), vec![log.clone()]);
    let written = fs::read_to_string(&log).unwrap();
    assert!(written.lines().any(|line| line == "startup diagnostic line 1"));
    assert!(written.lines().any(|line| line == "plugin admission failed"));
    assert!(!written.contains("s3cret"));
    assert_eq!(owner.status().ownership, BackendOwnership::None);

    // The Shell-Owned Log Path
    // The shell opens only the log its session recorded, and forgets it once
    // the next transition begins.
    assert_eq!(owner.startup_log(), Some(log));
    owner.disconnect().await.unwrap();
    assert_eq!(owner.startup_log(), None);
}

#[tokio::test]
async fn stderr_beside_readiness() {
    let root = tempfile::tempdir().unwrap();
    let backend = Published::start(&root.path().join("published")).await;
    let logs = root.path().join("logs");
    let executable = stub(root.path(), Publish::ReadyWithNoise(&backend.endpoint()));
    let owner = DesktopSession::with_local_backend(
        ConnectionSession::default(),
        owned(executable, &root.path().join("store")).with_logs(logs.clone()),
    );
    // A drained stderr never blocks readiness.
    let started = owner.select(BackendSelection::OwnedLocal).await.unwrap();
    assert_eq!(started.ownership, BackendOwnership::OwnedLocal);
    assert!(!started.connection.endpoint.unwrap_or_default().contains("192.0.2.1"), "a ready line on stderr is never parsed as readiness");

    // The log streams while the backend runs; the pump may trail readiness and
    // exposes no signal, so the story polls the file the process writes.
    let written = loop {
        if let [log] = stderr_logs(&logs).as_slice()
            && let Ok(written) = fs::read_to_string(log)
            && written.contains("192.0.2.1")
        {
            break written;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    };
    assert!(written.lines().any(|line| line == "noise line 3000 padded well past the pipe buffer"));

    owner.shutdown().await.unwrap();
    backend.shutdown().await;
}

#[tokio::test]
async fn overlapping_selection() {
    let root = tempfile::tempdir().unwrap();
    let backend = Published::start(&root.path().join("published")).await;
    let store = root.path().join("store");
    let owner = DesktopSession::with_local_backend(
        ConnectionSession::default(),
        owned(stub(root.path(), Publish::Ready(&backend.endpoint())), &store),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let pending_endpoint = format!("http://{}", listener.local_addr().unwrap());
    let first_owner = owner.clone();
    let first = tokio::spawn(async move { first_owner.select(attach(pending_endpoint)).await });
    // The accepted socket proves the first transition owns pending discovery.
    let (_socket, _) = listener.accept().await.unwrap();
    first.abort();
    assert!(first.await.unwrap_err().is_cancelled());
    let second_owner = owner.clone();
    let second = tokio::spawn(async move { second_owner.select(BackendSelection::OwnedLocal).await });
    let result = second.await.unwrap().unwrap();
    assert_eq!(result.ownership, BackendOwnership::OwnedLocal);
    owner.shutdown().await.unwrap();
    assert_eq!(owner.status().ownership, BackendOwnership::None);
    assert_eq!(store_owner(&store), None);
    backend.shutdown().await;
}

#[tokio::test]
async fn reset_restarts_backend() {
    let root = tempfile::tempdir().unwrap();
    let backend = Published::start(&root.path().join("published")).await;
    let store = root.path().join("store");
    let application = root.path().join("application");
    let storage = StorageRoot::new(store.clone(), application.clone());
    let executable = stub(root.path(), Publish::Ready(&backend.endpoint()));
    let owner = DesktopSession::with_local_backend(ConnectionSession::default(), owned(executable, &store));
    let started = owner.select(BackendSelection::OwnedLocal).await.unwrap();
    populate(&store, root.path());

    // A Path Other Than The Data Root Leaves The Backend Running
    let artifacts = store.join("artifacts");
    for outside in [root.path(), application.as_path(), artifacts.as_path()] {
        assert_eq!(reason(owner.reset_local_storage(outside, storage.clone()).await), Some(StorageResetReason::OutsideRoot));
    }
    let enclosing = StorageRoot::new(root.path().to_path_buf(), application.clone());
    assert_eq!(reason(owner.reset_local_storage(root.path(), enclosing).await), Some(StorageResetReason::OutsideRoot));
    assert_eq!(owner.status().connection.generation, started.connection.generation);
    assert!(store_owner(&store).is_some());

    // A Backend Still Holding The Store Keeps Every File
    let lease = fs::OpenOptions::new().create(true).truncate(false).write(true).open(store.join("local.lock")).unwrap();
    lease.try_lock().unwrap();
    assert_eq!(reason(owner.reset_local_storage(&store, storage.clone()).await), Some(StorageResetReason::BackendRunning));
    assert_eq!(owner.status().ownership, BackendOwnership::None);
    assert!(store.join("local.sqlite").exists());
    assert!(store.join("artifacts/evidence.bin").exists());
    drop(lease);

    // The Reset Empties The Root And Starts Again On It
    let restarted = owner.select(BackendSelection::OwnedLocal).await.unwrap();
    let reset = owner.reset_local_storage(&store, storage.clone()).await.unwrap();
    assert_eq!(reset.ownership, BackendOwnership::OwnedLocal);
    assert_ne!(reset.connection.generation, restarted.connection.generation);
    assert_eq!(owner.status().phase, None);
    let mut remaining = fs::read_dir(&store).unwrap().map(|entry| entry.unwrap().file_name().into_string().unwrap()).collect::<Vec<_>>();
    remaining.sort();
    // Only the restarted backend's own ownership marker is left.
    assert_eq!(remaining, vec!["owner.pid".to_owned()]);
    // A link inside the root is removed, never followed.
    assert!(root.path().join("outside.txt").exists());

    owner.shutdown().await.unwrap();
    backend.shutdown().await;
}
