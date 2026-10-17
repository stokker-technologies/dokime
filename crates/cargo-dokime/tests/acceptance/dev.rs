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
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};
// Third Party Imports
use tokio::time::{Instant, sleep};
// Local Imports
use dokime_client::{Client, Target, contracts};

/* === Constants === */
const PLUGIN: &str = r#"
const POLICY: dokime::harmos::operations::OperationPolicy = dokime::harmos::operations::OperationPolicy {
    timeout: std::time::Duration::from_secs(5),
    attempt_timeout: std::time::Duration::from_secs(5),
    retries: dokime::harmos::operations::RetryPolicy::Never,
    restart: dokime::harmos::operations::RestartPolicy::Never,
};
struct Plugin;
struct Ping;
#[dokime::catalog]
enum Operations { Ping(Ping) }
#[dokime::operation(id = "obc-plugin.ping", version = 1, description = "Answers.", policy = POLICY)]
impl Ping {
    async fn initialize(_: std::sync::Arc<()>, _: &dokime::Supervisor) -> Self { Self }
    async fn run(&mut self, _: &dokime::Supervisor, _: &()) -> Result<(), dokime::Refusal> { Ok(()) }
}
impl dokime::harmos::operations::lifecycle::Policy for Plugin {
    const INITIALIZATION_POLICY: dokime::harmos::operations::OperationPolicy = POLICY;
    const SHUTDOWN_POLICY: dokime::harmos::operations::OperationPolicy = POLICY;
}
#[dokime::plugin(id = "obc-plugin", description = "A plugin whose whole purpose is to say when it opened and when it closed.", provides = Operations, requires(), restart = "never")]
impl dokime::Plugin for Plugin {
    type Config = dokime::plugin::Configuration;
    type Error = dokime::Refusal;
    type Context = dokime::Supervisor;
    type Resources = ();
    fn prepare(_: dokime::plugin::Configuration) -> Result<Self, dokime::Refusal> { Ok(Self) }
    async fn initialize(&mut self, _: &dokime::Supervisor) -> Result<(), dokime::Refusal> {
        let _ = development_support::value();
        record("started");
        Ok(())
    }
    fn resources(&self) -> std::sync::Arc<()> { std::sync::Arc::new(()) }
    fn bind_operations(
        &self,
        table: &mut dokime::harmos::operations::WireTable<dokime::Supervisor>,
    ) -> Result<(), dokime::harmos::operations::ServingError> {
        table.bind_operations::<Operations, _>(self.resources())
    }
    async fn stop(&mut self, _: &dokime::Supervisor) -> Result<(), dokime::Refusal> { Ok(()) }
    async fn cleanup(&mut self) -> Result<(), dokime::Refusal> { record("finalized"); Ok(()) }
}
fn record(kind: &str) {
    use std::io::Write;
    let path = LIFECYCLE_PATH;
    let mut file = std::fs::OpenOptions::new().create(true).append(true).open(path).unwrap();
    writeln!(file, "{} {}", kind, std::process::id()).unwrap();
}
fn main() -> std::io::Result<()> { dokime::plugin::run::<Plugin>() }
"#;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Process(std::process::Child);

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl std::ops::Deref for Process {
    type Target = std::process::Child;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::ops::DerefMut for Process {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        if self.0.try_wait().ok().flatten().is_some() {
            return;
        }
        drop(self.0.stdin.take());
        // Failed assertions must still give the owned host its ordinary drain.
        for _ in 0..300 {
            if self.0.try_wait().ok().flatten().is_some() {
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        eprintln!("test cleanup: process {} did not drain", self.0.id());
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/* === Public Functions === */

/* === Private Functions === */
// The exact prepared server mise selects; the `dokime` CLI is not a host.
fn prepared_host() -> std::path::PathBuf {
    let backend = std::path::PathBuf::from(
        std::env::var_os("DOKIME_TEST_HOST").expect("mise selects the exact prepared server"),
    );
    assert!(
        backend.is_file(),
        "test:plugin-dev requires mise's prepared server artifact ({})",
        backend.display()
    );
    backend
}

async fn wait_for(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(180);
    while !condition() {
        assert!(
            Instant::now() < deadline,
            "development process did not reach expected state"
        );
        sleep(Duration::from_millis(50)).await;
    }
}

fn text(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_default()
}

async fn retry_and_rebuild_preserve_host_session() {
    let root = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(root.path().join("plugin/src")).unwrap();
    std::fs::write(
        root.path().join("Cargo.toml"),
        "[workspace]\nmembers = [\"plugin\"]\nresolver = \"3\"\n",
    )
    .unwrap();
    let sdk = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../dokime")
        .canonicalize()
        .unwrap();
    let dependency = tempfile::tempdir().unwrap();
    std::fs::create_dir(dependency.path().join("src")).unwrap();
    std::fs::write(dependency.path().join("Cargo.toml"), "[package]\nname = \"development-support\"\nversion = \"1.0.0\"\nedition = \"2024\"\n[workspace]\n").unwrap();
    std::fs::write(
        dependency.path().join("src/lib.rs"),
        "pub fn value() -> u32 { 1 }",
    )
    .unwrap();
    let dependency_path = dependency.path();
    std::fs::write(
        root.path().join("plugin/Cargo.toml"),
        format!(
            r#"[package]
name = "development-fixture"
version = "1.0.0"
edition = "2024"
description = "Development lifecycle proof"
[dependencies]
dokime = {{ path = {sdk:?} }}
development-support = {{ path = {dependency_path:?} }}
"#
        ),
    )
    .unwrap();
    let source = root.path().join("plugin/src/main.rs");
    let target = super::support::target_directory().canonicalize().unwrap();
    let backend = prepared_host();
    let installation = tempfile::tempdir().unwrap();
    let host = installation
        .path()
        .join(format!("cargo-dokime{}", std::env::consts::EXE_SUFFIX));
    std::fs::copy(super::support::tool(), &host).unwrap();
    let path = std::env::join_paths(
        std::iter::once(installation.path().to_owned())
            .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap())),
    )
    .unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let lifecycle = root.path().join("lifecycle.log");
    let diagnostics = root.path().join("diagnostics.log");
    // Logs live outside the watched source tree to avoid self-triggered rebuilds.
    let state = tempfile::tempdir().unwrap();
    // The development state is the host's application root; keep any ambient
    // profile outside both watched source trees as well.
    let application_environment = [
        ("LOCALAPPDATA", state.path().join("profile/local")),
        ("XDG_CONFIG_HOME", state.path().join("profile/config")),
        ("XDG_STATE_HOME", state.path().join("profile/state")),
        ("XDG_DATA_HOME", state.path().join("profile/data")),
    ];
    let lifecycle = state.path().join(lifecycle.file_name().unwrap());
    let diagnostics = state.path().join(diagnostics.file_name().unwrap());
    let plugin = PLUGIN.replace("LIFECYCLE_PATH", &format!("{lifecycle:?}"));
    std::fs::write(&source, "invalid Rust source").unwrap();
    let dependency_manifest = text(&dependency_path.join("Cargo.toml"));
    std::fs::write(dependency_path.join("Cargo.toml"), "invalid manifest").unwrap();
    let mut dev = Process(
        Command::new("cargo")
            .envs(
                application_environment
                    .iter()
                    .map(|(name, path)| (*name, path)),
            )
            .current_dir(root.path())
            .env("PATH", path)
            .args([
                "dokime",
                "dev",
                "-p",
                "development-fixture",
                "--shutdown-stdin",
            ])
            .arg("--dokime")
            .arg(&backend)
            .arg("--headless")
            .arg("--watch")
            .arg(dependency_path)
            .arg("--target-dir")
            .arg(target.join("plugin-cli-fixture"))
            .arg("--state-dir")
            .arg(state.path())
            .arg("--port")
            .arg(port.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(std::fs::File::create(&diagnostics).unwrap())
            .spawn()
            .unwrap(),
    );
    wait_for(|| text(&diagnostics).contains("No development host is running")).await;
    assert!(dev.try_wait().unwrap().is_none(), "{}", text(&diagnostics));
    assert!(text(&lifecycle).is_empty());
    // An explicit retry runs even though neither watched source nor metadata changed.
    let failures = text(&diagnostics)
        .matches("No development host is running")
        .count();
    dev.stdin.as_mut().unwrap().write_all(b"r\n").unwrap();
    wait_for(|| {
        text(&diagnostics)
            .matches("No development host is running")
            .count()
            > failures
    })
    .await;
    assert!(text(&diagnostics).contains("Retry requested"));
    // Explicit watches must recover even before Cargo metadata can discover dependencies.
    std::fs::write(dependency_path.join("Cargo.toml"), dependency_manifest).unwrap();
    wait_for(|| text(&diagnostics).contains("expected one of")).await;
    assert!(dev.try_wait().unwrap().is_none(), "{}", text(&diagnostics));
    assert!(text(&lifecycle).is_empty());
    std::fs::write(&source, &plugin).unwrap();
    wait_for(|| {
        assert!(dev.try_wait().unwrap().is_none(), "{}", text(&diagnostics));
        text(&lifecycle)
            .lines()
            .any(|line| line.starts_with("started "))
    })
    .await;
    assert!(
        state.path().join("settings.json").is_file(),
        "the development host keeps its settings and identity in the development state"
    );
    let original = text(&lifecycle)
        .lines()
        .next()
        .unwrap()
        .strip_prefix("started ")
        .unwrap()
        .to_owned();
    let endpoint = format!("http://127.0.0.1:{port}");
    let backend_client = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            if let Ok(client) =
                Client::connect(Target::local(endpoint.clone()).unwrap(), None, None).await
                && client
                    .backend_get_health(contracts::backend::GetHealthRequest {})
                    .await
                    .is_ok()
            {
                break client;
            }
            sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("native host unavailable: {}", text(&diagnostics)));
    let session = backend_client
        .backend_get_identity(contracts::backend::GetIdentityRequest {})
        .await
        .unwrap()
        .epoch;
    let observation = Client::connect(Target::local(endpoint.clone()).unwrap(), None, None)
        .await
        .unwrap();
    let updates = observation
        .observation_watch_runtime(contracts::observation::WatchRuntimeRequest { cursor: None })
        .await
        .unwrap();
    assert!(updates.message().await.unwrap().unwrap().resynchronized);
    let plugins = Client::connect(Target::local(endpoint).unwrap(), None, None)
        .await
        .unwrap();
    // The development host initializes the replacement before committing its
    // catalog. Wait for the observable admission commit instead of using the
    // plugin's initialization marker as a proxy for inventory visibility.
    let inventory = tokio::time::timeout(Duration::from_secs(30), async {
        loop {
            let inventory = plugins
                .plugins_list_admitted_plugins(contracts::plugins::ListAdmittedPluginsRequest {
                    page_size: 100,
                    after: None,
                })
                .await
                .unwrap();
            if inventory
                .plugins
                .iter()
                .any(|entry| entry.plugin_id == "obc-plugin")
            {
                break inventory;
            }
            sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("development plugin admission did not become visible");
    assert!(inventory.next.is_none());
    let owners = inventory
        .plugins
        .iter()
        .map(|entry| entry.plugin_id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(owners, ["host", "obc-plugin"].into_iter().collect());
    std::fs::write(&source, "invalid Rust source").unwrap();
    wait_for(|| text(&diagnostics).contains("Previous plugin build is still running")).await;
    assert_eq!(
        text(&lifecycle).lines().count(),
        1,
        "failed compilation must not stop the active native process"
    );
    assert!(dev.try_wait().unwrap().is_none());
    // Fail activation once using state outside the watched roots. Retrying the
    // unchanged bundle must reach the existing host after that state is repaired.
    let refusal = state.path().join("refuse-initialization");
    std::fs::write(&refusal, "refuse").unwrap();
    let retry_plugin = plugin.replace(
        "record(\"started\");",
        &format!("if std::path::Path::new({refusal:?}).exists() {{ return Err(dokime::Refusal::of(\"retry activation fixture\")); }} record(\"started\");"),
    );
    std::fs::write(&source, retry_plugin).unwrap();
    wait_for(|| text(&diagnostics).contains("Plugin reload failed")).await;
    assert!(dev.try_wait().unwrap().is_none());
    std::fs::remove_file(&refusal).unwrap();
    dev.stdin.as_mut().unwrap().write_all(b"r\n").unwrap();
    wait_for(|| {
        text(&lifecycle)
            .lines()
            .filter(|line| line.starts_with("started "))
            .count()
            == 2
    })
    .await;
    let records = text(&lifecycle);
    let lines = records.lines().collect::<Vec<_>>();
    assert_eq!(
        lines[1],
        format!("finalized {original}"),
        "old native resources must finalize before replacement initialization"
    );
    // The refused activation is cleaned up like any partial initialization
    // (a Harmos plugin's `cleanup` runs after a failed `initialize`), so its
    // own `finalized` line may precede the replacement's start.
    assert!(lines.last().unwrap().starts_with("started "), "{records}");
    assert!(
        lines[2..lines.len() - 1]
            .iter()
            .all(|line| line.starts_with("finalized ") && *line != lines[1]),
        "{records}"
    );
    std::fs::write(
        dependency.path().join("src/lib.rs"),
        "pub fn value() -> u32 { 2 }",
    )
    .unwrap();
    wait_for(|| {
        text(&lifecycle)
            .lines()
            .filter(|line| line.starts_with("started "))
            .count()
            == 3
    })
    .await;
    let replacement = tempfile::tempdir().unwrap();
    std::fs::create_dir(replacement.path().join("src")).unwrap();
    std::fs::copy(
        dependency.path().join("Cargo.toml"),
        replacement.path().join("Cargo.toml"),
    )
    .unwrap();
    std::fs::write(
        replacement.path().join("src/lib.rs"),
        "pub fn value() -> u32 { 3 }",
    )
    .unwrap();
    let manifest_path = root.path().join("plugin/Cargo.toml");
    let manifest = text(&manifest_path).replace(
        &format!("{dependency_path:?}"),
        &format!("{:?}", replacement.path()),
    );
    std::fs::write(manifest_path, manifest).unwrap();
    // Newly discovered dependency roots are watched before compilation starts.
    wait_for(|| {
        text(&lifecycle)
            .lines()
            .filter(|line| line.starts_with("started "))
            .count()
            == 4
    })
    .await;
    std::fs::write(
        replacement.path().join("src/lib.rs"),
        "pub fn value() -> u32 { 4 }",
    )
    .unwrap();
    wait_for(|| {
        text(&lifecycle)
            .lines()
            .filter(|line| line.starts_with("started "))
            .count()
            == 5
    })
    .await;
    let identity = backend_client
        .backend_get_identity(contracts::backend::GetIdentityRequest {})
        .await
        .unwrap();
    assert_eq!(
        identity.epoch, session,
        "plugin replacement must retain the host session"
    );
    // The original native subscription stays held throughout replacement and shutdown.
    drop(dev.stdin.take());
    wait_for(|| dev.try_wait().unwrap().is_some()).await;
    assert!(dev.wait().unwrap().success(), "{}", text(&diagnostics));
    drop(updates);
    assert!(
        !text(&diagnostics).contains("Broken pipe"),
        "{}",
        text(&diagnostics)
    );
    // Five activations that started, plus the refused one cleaned up after
    // its failed initialization.
    assert_eq!(
        text(&lifecycle)
            .lines()
            .filter(|line| line.starts_with("finalized "))
            .count(),
        6
    );
    let mut host = Process(
        Command::new(&backend)
            .envs(
                application_environment
                    .iter()
                    .map(|(name, path)| (*name, path)),
            )
            .args([
                "--shutdown-stdin",
                "--no-seed-validation-artifact",
                "--port",
            ])
            .arg(port.to_string())
            .arg("--application-root")
            .arg(state.path().join("installed"))
            .arg("--output-root")
            .arg(state.path().join("installation-runtime"))
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap(),
    );
    let url = format!("http://127.0.0.1:{port}");
    let client = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            if let Ok(client) =
                Client::connect(Target::local(url.clone()).unwrap(), None, None).await
                && client
                    .plugins_list_installations(contracts::plugins::ListInstallationsRequest {
                        page_size: 1,
                        after: None,
                    })
                    .await
                    .is_ok()
            {
                break client;
            }
            sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .unwrap();
    let installed = tokio::process::Command::new(super::support::tool())
        .args(["dokime", "install"])
        .arg(state.path().join("development.dokime"))
        .arg("--endpoint")
        .arg(&url)
        .arg("--yes")
        .output()
        .await
        .unwrap();
    assert!(
        installed.status.success(),
        "{}",
        String::from_utf8_lossy(&installed.stderr)
    );
    assert!(String::from_utf8_lossy(&installed.stderr).contains("restart the host"));
    let entries = client
        .plugins_list_installations(contracts::plugins::ListInstallationsRequest {
            page_size: 100,
            after: None,
        })
        .await
        .unwrap();
    assert!(entries.next.is_none());
    assert_eq!(entries.installations.len(), 1);
    assert_eq!(
        entries.installations[0].state,
        contracts::plugins::PluginInstallationState::Installed
    );
    assert_eq!(
        entries.installations[0].metadata.as_ref().unwrap().owners,
        ["obc-plugin"]
    );
    drop(host.stdin.take());
    wait_for(|| host.try_wait().unwrap().is_some()).await;
    assert!(host.wait().unwrap().success());
}

async fn stdin_quit_and_unrequested_eof() {
    let root = tempfile::tempdir().unwrap();
    std::fs::write(root.path().join("Cargo.toml"), "invalid manifest").unwrap();
    let state = tempfile::tempdir().unwrap();
    let diagnostics = state.path().join("diagnostics.log");
    let backend = prepared_host();
    for quit in [true, false] {
        let mut dev = Process(
            Command::new(super::support::tool())
                .current_dir(root.path())
                .args(["dev", "--dokime"])
                .arg(&backend)
                .arg("--headless")
                .arg("--state-dir")
                .arg(state.path())
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(std::fs::File::create(&diagnostics).unwrap())
                .spawn()
                .unwrap(),
        );
        wait_for(|| text(&diagnostics).contains("No development host is running")).await;
        if quit {
            dev.stdin.as_mut().unwrap().write_all(b"q\n").unwrap();
            wait_for(|| dev.try_wait().unwrap().is_some()).await;
            assert!(dev.wait().unwrap().success(), "{}", text(&diagnostics));
        } else {
            drop(dev.stdin.take());
            let failures = text(&diagnostics).matches("Build failed").count();
            sleep(Duration::from_millis(1200)).await;
            assert!(
                dev.try_wait().unwrap().is_none(),
                "EOF without --shutdown-stdin must keep watching"
            );
            assert_eq!(text(&diagnostics).matches("Build failed").count(), failures);
            // No host or compiler is active after the metadata refusal.
            dev.kill().unwrap();
            dev.wait().unwrap();
        }
    }
}

/* === Tests === */
fn development_and_install_help_need_no_workspace_or_host() {
    let root = tempfile::tempdir().unwrap();
    for command in ["dev", "install", "serve", "version"] {
        let output = Command::new(super::support::tool())
            .current_dir(root.path())
            .args(["dokime", command, "--help"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        if command == "dev" {
            let help = String::from_utf8_lossy(&output.stdout);
            assert!(help.contains("--rerun"));
            assert!(!help.contains("--shutdown-stdin"));
        }
    }
}

pub(super) async fn development_contract() {
    retry_and_rebuild_preserve_host_session().await;
    stdin_quit_and_unrequested_eof().await;
    development_and_install_help_need_no_workspace_or_host();
}
