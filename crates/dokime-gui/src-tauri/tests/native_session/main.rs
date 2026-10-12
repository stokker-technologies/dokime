/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(crate) mod parameters;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{path::Path, time::Duration};
// Third Party Imports

// Local Imports
use dokime_gui::connection::{
    ConnectRequest, ConnectionCredentials, ConnectionSession, ConnectionTarget,
};
use dokime_gui::native_session::DesktopSession;

#[cfg(unix)]
use dokime_gui::native_session::backend::Backend;
use dokime_gui::native_session::{BackendOwnership, BackendSelection, DesktopSessionError};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Published {
    runtime: dokime_app::Runtime,
    server: dokime_server::grpc::Server,
}

/* === Enums === */
#[cfg(unix)]
enum Publish<'a> {
    Ready(&'a str),
    ReadyWithPhases(&'a str, &'a [&'a str]),
    Failure,
}

/* === Traits === */

/* === Implementations === */
impl Published {
    async fn start(root: &Path) -> Self {
        let mut config = dokime_app::config::Config::local(root);
        config.seed_plugins = false;
        config.credential_backend =
            serde_json::from_value(serde_json::json!({"backend":"memory"})).unwrap();
        let runtime = dokime_app::app::launch(config).await.unwrap();
        let server = dokime_server::grpc::Server::start(
            runtime.handle(),
            dokime_server::grpc::Config::local(),
        )
        .await
        .unwrap();
        Self { runtime, server }
    }
    fn endpoint(&self) -> String {
        self.server.local_endpoint()
    }
    async fn shutdown(mut self) {
        self.server.shutdown().await.unwrap();
        self.runtime.shutdown().await.unwrap();
    }
}

/* === Public Functions === */

/* === Private Functions === */
#[cfg(unix)]
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
            let mut script = "mkfifo \"$root/phase-ack\"\n".to_owned();
            for phase in phases {
                script.push_str(&format!(
                    "printf '%s\\n' '{}'\nread acknowledgement < \"$root/phase-ack\"\n",
                    serde_json::json!({ "event": "phase", "name": phase })
                ));
            }
            script.push_str(&format!(
                "printf '%s\\n' '{}'\ncat > /dev/null\nrm -f \"$owner\"",
                ready_line(endpoint)
            ));
            ("ready-with-phases", script)
        }
        Publish::Failure => (
            "failure",
            "echo \"plugin admission failed\" >&2\nexit 1".to_owned(),
        ),
    };
    let path = directory.join(format!("dokime-stub-{name}"));
    std::fs::write(
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
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    // A parallel test may fork while this file is still open for writing; the
    // child keeps that descriptor until it execs, and executing the stub in
    // that window fails with ETXTBSY. Wait until the kernel accepts it.
    let started = std::time::Instant::now();
    loop {
        match std::process::Command::new(&path).arg("version").output() {
            Err(error)
                if error.kind() == std::io::ErrorKind::ExecutableFileBusy
                    && started.elapsed() < Duration::from_secs(5) =>
            {
                std::thread::sleep(Duration::from_millis(5));
            }
            result => {
                result.expect("stub executable runs");
                break;
            }
        }
    }
    path
}

#[cfg(unix)]
fn owned(executable: std::path::PathBuf, root: &Path) -> Backend {
    let arguments = vec![
        "serve".into(),
        "--shutdown-stdin".into(),
        "--dev-control".into(),
        "--output-root".into(),
        root.display().to_string(),
    ];
    Backend::new(executable, arguments)
}

#[cfg(unix)]
fn store_owner(root: &Path) -> Option<String> {
    std::fs::read_to_string(root.join("owner.pid")).ok()
}

fn attach(endpoint: String) -> BackendSelection {
    BackendSelection::Attach {
        request: ConnectRequest {
            target: ConnectionTarget::Local { endpoint },
            credentials: ConnectionCredentials::None,
            expected_backend_id: None,
        },
    }
}

#[cfg(unix)]
pub(crate) async fn owned_shutdown_order() {
    let root = tempfile::tempdir().unwrap();
    let backend = Published::start(&root.path().join("published")).await;
    let store = root.path().join("store");
    let executable = stub(root.path(), Publish::Ready(&backend.endpoint()));
    let session = ConnectionSession::default();
    let owner =
        DesktopSession::with_local_backend(session.clone(), owned(executable.clone(), &store));
    let started = owner.select(BackendSelection::OwnedLocal).await.unwrap();
    assert_eq!(started.ownership, BackendOwnership::OwnedLocal);
    let lease = session.lease(&started.connection.generation).unwrap();
    assert!(
        store_owner(&store).is_some(),
        "the owned backend holds its store"
    );

    owner.shutdown().await.unwrap();
    assert_eq!(
        lease
            .execute(std::future::pending::<Result<(), tonic::Status>>())
            .await,
        Err(dokime_gui::connection::ConnectionError::Superseded)
    );
    assert!(session.lease(&started.connection.generation).is_err());
    // Shutdown returns only after the backend drained and exited, so its store
    // is already released when the desktop moves on.
    assert_eq!(store_owner(&store), None);
    assert!(matches!(
        owner.select(BackendSelection::OwnedLocal).await,
        Err(DesktopSessionError::ShuttingDown)
    ));

    // A second owner can acquire the exact execution store only after the first
    // drained; the backend refuses a store another live backend still holds.
    let replacement =
        DesktopSession::with_local_backend(ConnectionSession::default(), owned(executable, &store));
    assert_eq!(
        replacement
            .select(BackendSelection::OwnedLocal)
            .await
            .unwrap()
            .ownership,
        BackendOwnership::OwnedLocal
    );
    replacement.shutdown().await.unwrap();
    backend.shutdown().await;
}

pub(crate) async fn external_lifetime() {
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

#[cfg(unix)]
pub(crate) async fn startup_recovery() {
    let root = tempfile::tempdir().unwrap();
    let backend = Published::start(&root.path().join("published")).await;
    let blocker = root.path().join("blocked");
    std::fs::write(&blocker, b"occupied").unwrap();
    let owner = DesktopSession::with_local_backend(
        ConnectionSession::default(),
        owned(
            stub(root.path(), Publish::Ready(&backend.endpoint())),
            &blocker,
        ),
    );
    let refused = owner.select(BackendSelection::OwnedLocal).await;
    assert!(matches!(refused, Err(DesktopSessionError::Startup { .. })));
    assert_eq!(owner.status().ownership, BackendOwnership::None);

    std::fs::remove_file(&blocker).unwrap();
    assert_eq!(
        owner
            .select(BackendSelection::OwnedLocal)
            .await
            .unwrap()
            .ownership,
        BackendOwnership::OwnedLocal
    );
    owner.shutdown().await.unwrap();
    backend.shutdown().await;
}

#[cfg(unix)]
pub(crate) async fn owned_start_status() {
    let root = tempfile::tempdir().unwrap();
    let backend = Published::start(&root.path().join("published")).await;
    let store = root.path().join("store");
    let executable = stub(
        root.path(),
        Publish::ReadyWithPhases(&backend.endpoint(), &["storage", "requests"]),
    );
    let owner =
        DesktopSession::with_local_backend(ConnectionSession::default(), owned(executable, &store));

    let selecting = owner.clone();
    let task = tokio::spawn(async move { selecting.select(BackendSelection::OwnedLocal).await });

    let mut seen: Vec<String> = Vec::new();
    tokio::time::timeout(Duration::from_secs(5), async {
        while seen.len() < 2 {
            if let Some(phase) = owner.status().phase
                && seen.last() != Some(&phase)
            {
                seen.push(phase);
                // The backend waits for acknowledgement, so observing each
                // phase never depends on a scheduler delay.
                std::fs::write(store.join("phase-ack"), b"continue\n").unwrap();
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert_eq!(
        seen,
        vec!["storage".to_owned(), "requests".to_owned()],
        "observed phases: {seen:?}"
    );

    let started = task.await.unwrap().unwrap();
    assert_eq!(started.ownership, BackendOwnership::OwnedLocal);
    assert_eq!(owner.status().phase, None);

    owner.shutdown().await.unwrap();
    backend.shutdown().await;
}

#[cfg(unix)]
pub(crate) async fn live_backend_owner() {
    let root = tempfile::tempdir().unwrap();
    let backend = Published::start(&root.path().join("published")).await;
    let store = root.path().join("store");
    let executable = stub(root.path(), Publish::Ready(&backend.endpoint()));
    let stale = DesktopSession::with_local_backend(
        ConnectionSession::default(),
        owned(executable.clone(), &store),
    );
    stale.select(BackendSelection::OwnedLocal).await.unwrap();

    let successor =
        DesktopSession::with_local_backend(ConnectionSession::default(), owned(executable, &store));
    let refused = successor.select(BackendSelection::OwnedLocal).await;
    assert!(matches!(refused, Err(DesktopSessionError::Startup { .. })));
    assert_eq!(successor.status().ownership, BackendOwnership::None);

    stale.shutdown().await.unwrap();
    assert_eq!(store_owner(&store), None);
    successor
        .select(BackendSelection::OwnedLocal)
        .await
        .unwrap();
    successor.shutdown().await.unwrap();
    backend.shutdown().await;
}

#[cfg(unix)]
pub(crate) async fn failed_start_launched() {
    let root = tempfile::tempdir().unwrap();
    let backend = Published::start(&root.path().join("published")).await;
    let store = root.path().join("store");
    let owner = DesktopSession::with_local_backend(
        ConnectionSession::default(),
        owned(stub(root.path(), Publish::Failure), &store),
    );
    let refused = owner.select(BackendSelection::OwnedLocal).await;
    assert!(matches!(refused, Err(DesktopSessionError::Startup { .. })));
    assert_eq!(owner.status().ownership, BackendOwnership::None);

    let successor = DesktopSession::with_local_backend(
        ConnectionSession::default(),
        owned(
            stub(root.path(), Publish::Ready(&backend.endpoint())),
            &store,
        ),
    );
    successor
        .select(BackendSelection::OwnedLocal)
        .await
        .unwrap();
    successor.shutdown().await.unwrap();
    backend.shutdown().await;
}

#[cfg(unix)]
pub(crate) async fn overlapping_selection() {
    let root = tempfile::tempdir().unwrap();
    let backend = Published::start(&root.path().join("published")).await;
    let store = root.path().join("store");
    let owner = DesktopSession::with_local_backend(
        ConnectionSession::default(),
        owned(
            stub(root.path(), Publish::Ready(&backend.endpoint())),
            &store,
        ),
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
    let second =
        tokio::spawn(async move { second_owner.select(BackendSelection::OwnedLocal).await });
    let result = tokio::time::timeout(Duration::from_secs(20), second)
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(result.ownership, BackendOwnership::OwnedLocal);
    owner.shutdown().await.unwrap();
    assert_eq!(owner.status().ownership, BackendOwnership::None);
    assert_eq!(store_owner(&store), None);
    backend.shutdown().await;
}

pub(crate) async fn disconnect_cancels_discovery() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let endpoint = format!("http://{}", listener.local_addr().unwrap());
    let owner = DesktopSession::attached(ConnectionSession::default());
    let connecting = owner.clone();
    let task = tokio::spawn(async move { connecting.select(attach(endpoint)).await });
    // TCP acceptance proves an actual connection attempt is pending; no timing sleeps.
    let (_socket, _) = listener.accept().await.unwrap();
    tokio::time::timeout(Duration::from_secs(2), owner.disconnect())
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(
        task.await.unwrap(),
        Err(DesktopSessionError::Superseded)
    ));
    assert_eq!(owner.status().ownership, BackendOwnership::None);
}

pub(crate) async fn invalid_local_attachment() {
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

/* === Tests === */

#[test]
fn discovery_refuses_precedence() {
    const PROBE: &str = "DOKIME_DISCOVERY_TEST_EXPECTED";
    if let Some(expected) = std::env::var_os(PROBE) {
        assert_eq!(
            dokime_gui::native_session::backend::locate().unwrap(),
            std::path::PathBuf::from(expected)
        );
        return;
    }
    // Re-exec a hard link beside fixture candidates. Child-only environment
    // avoids races with native sessions and never consults a developer keyring.
    let executable = std::env::current_exe().unwrap();
    let fixture = tempfile::tempdir_in(executable.parent().unwrap()).unwrap();
    let harness = fixture
        .path()
        .join(format!("probe{}", std::env::consts::EXE_SUFFIX));
    std::fs::hard_link(&executable, &harness).unwrap();
    let home = fixture.path().join("home");
    let path = fixture.path().join("path");
    std::fs::create_dir_all(&home).unwrap();
    std::fs::create_dir_all(&path).unwrap();
    let name = format!("dokime{}", std::env::consts::EXE_SUFFIX);
    let bundled = fixture.path().join(&name);
    let in_home = home.join(&name);
    let on_path = path.join(&name);
    for candidate in [&bundled, &in_home, &on_path] {
        std::fs::write(candidate, "backend").unwrap();
    }
    let probe = |expected: &Path| {
        let output = std::process::Command::new(&harness)
            .args(["--exact", "discovery_refuses_precedence", "--nocapture"])
            .env(PROBE, expected)
            .env("DOKIME_HOME", &home)
            .env("PATH", &path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    };
    probe(&bundled);
    std::fs::remove_file(&bundled).unwrap();
    probe(&in_home);
    std::fs::remove_file(&in_home).unwrap();
    std::fs::create_dir(&in_home).unwrap();
    probe(&on_path);
    std::fs::remove_dir(&in_home).unwrap();
    probe(&on_path);
}

fn fresh_runtime(
    multithread: bool,
    threads: Option<usize>,
    paused: bool,
) -> tokio::runtime::Runtime {
    let _ = multithread;
    let mut builder = tokio::runtime::Builder::new_current_thread();
    builder.enable_all();
    if let Some(threads) = threads {
        builder.worker_threads(threads);
    }
    let _ = paused;
    builder.build().expect("scenario runtime")
}

#[test]
fn domain_contract() {
    #[cfg(unix)]
    {
        eprintln!("Scenario: owned_shutdown_order");
        run_scenario(owned_shutdown_order, false, None, false);
    }
    {
        eprintln!("Scenario: external_lifetime");
        run_scenario(external_lifetime, false, None, false);
    }
    #[cfg(unix)]
    {
        eprintln!("Scenario: startup_recovery");
        run_scenario(startup_recovery, false, None, false);
    }
    #[cfg(unix)]
    {
        eprintln!("Scenario: owned_start_status");
        run_scenario(owned_start_status, false, None, false);
    }
    #[cfg(unix)]
    {
        eprintln!("Scenario: live_backend_owner");
        run_scenario(live_backend_owner, false, None, false);
    }
    #[cfg(unix)]
    {
        eprintln!("Scenario: failed_start_launched");
        run_scenario(failed_start_launched, false, None, false);
    }
    #[cfg(unix)]
    {
        eprintln!("Scenario: overlapping_selection");
        run_scenario(overlapping_selection, false, None, false);
    }
    {
        eprintln!("Scenario: disconnect_cancels_discovery");
        run_scenario(disconnect_cancels_discovery, false, None, false);
    }
    {
        eprintln!("Scenario: invalid_local_attachment");
        run_scenario(invalid_local_attachment, false, None, false);
    }
    {
        eprintln!("Scenario: parameters::param_replacement_failure");
        parameters::param_replacement_failure();
    }
}

#[inline(never)]
fn run_scenario<F: std::future::Future>(
    factory: impl FnOnce() -> F,
    multithread: bool,
    threads: Option<usize>,
    paused: bool,
) -> F::Output {
    fresh_runtime(multithread, threads, paused).block_on(Box::pin(factory()))
}
