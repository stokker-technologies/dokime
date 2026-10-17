/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod args;
mod error;
mod hub_log;
mod logging;
mod network;
pub mod version;

#[cfg(debug_assertions)]
mod profiles;

/* === Re-Exports === */
pub use args::{Args, NetworkArgs};
pub use error::{Error, Result};
// The subscriber install is part of the shell's surface, not just `run`'s
// internals: it is a process-global side effect, so anything hosting this
// backend installs it exactly once and holds the guards for the process.
pub use logging::{LoggingGuards, init as install_logging};

/* === Imports === */
// Standard Library Imports
use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};
// Third Party Imports

// Local Imports
use dokime_runtime::config::ApplicationOptions;
use dokime_contract::{DEV_PROTOCOL, READINESS_PROTOCOL};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct ExpectedPlugin {
    package: String,
    digest: String,
    owners: Vec<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub async fn run(
    args: Args,
    server_config: crate::grpc::Config,
    terminal: dokime::terminal::Terminal,
) -> Result<()> {
    let mut config = ApplicationOptions {
        application_root: args.application_root.clone(),
        storage_root: args.output_root.clone(),
        plugins_root: args.plugins_root.clone(),
    }
    .resolve()?;
    let application_root = config
        .application_root
        .clone()
        .expect("resolved options name an application root");
    let storage_root = config.output_root.clone();
    let _log_guards = logging::init(&application_root, terminal.color_enabled())?;
    if let Some(title) = &args.title {
        config.title = title.clone();
    }
    config.label = args.label.clone();
    config.seed_plugins = !args.no_seed_validation_artifact;
    config.explicit_plugins = if args.dev_control {
        Vec::new()
    } else {
        args.plugins.clone()
    };
    let expected = expected_plugins(&args.plugins)?;
    emit_phase(args.dev_control, "storage");
    let mut runtime = dokime_runtime::app::launch(config)
        .await
        .map_err(|error| Error::launch(error, storage_root.clone()))?;
    emit_phase(args.dev_control, "plugins");
    if let Err(error) = verify_plugins(
        &runtime.handle(),
        if args.dev_control { &[] } else { &expected },
    )
    .await
    {
        if let Err(cleanup) = runtime.shutdown().await {
            return Err(crate::host::Error::Input(format!(
                "{error}; runtime shutdown also failed: {cleanup}"
            )));
        }
        return Err(error);
    }
    emit_phase(args.dev_control, "requests");
    let server = match crate::grpc::Server::start(runtime.handle(), server_config).await {
        Ok(server) => server,
        Err(error) => {
            let cleanup = runtime.shutdown().await.map(|_| ());
            return Err(crate::host::Error::startup(error, cleanup));
        }
    };
    let local_addr = server.address();
    let local_url = server.local_endpoint();
    if args.dev_control {
        use std::io::Write;
        let ready = serde_json::json!({
            "event": "ready",
            "protocol": READINESS_PROTOCOL,
            "endpoint": local_url,
            "transport": "grpc",
            "version": env!("CARGO_PKG_VERSION"),
            "dev_protocol": DEV_PROTOCOL,
        });
        let result = writeln!(std::io::stdout().lock(), "{ready}");
        if let Err(error) = result {
            let server_result = server.shutdown().await;
            let runtime_result = runtime.shutdown().await.map(|_| ());
            crate::host::Error::shutdown(server_result, runtime_result)?;
            return Err(error.into());
        }
    } else {
        print_startup(
            &terminal,
            &application_root,
            &storage_root,
            local_addr,
            &local_url,
        );
    }

    if args.dev_control {
        development_control(&mut runtime, &args.plugins, &expected).await;
    } else {
        shutdown_signal(args.shutdown_stdin).await;
    }
    arm_force_quit();
    let server_result = server.shutdown().await;
    let runtime_result = runtime.shutdown().await.map(|_| ());
    crate::host::Error::shutdown(server_result, runtime_result)
}

/* === Private Functions === */
async fn development_control(
    runtime: &mut dokime_runtime::Runtime,
    paths: &[PathBuf],
    expected: &[ExpectedPlugin],
) {
    use std::io::{BufRead, Write};
    let allowed: Vec<_> = paths.iter().map(|path| path.canonicalize().ok()).collect();
    let (send, mut commands) = tokio::sync::mpsc::channel(8);
    std::thread::spawn(move || {
        let mut input = std::io::stdin().lock();
        loop {
            let mut line = String::new();
            // Bound each request independently; EOF still owns host shutdown.
            match std::io::Read::take(&mut input, 16 * 1024 + 1).read_line(&mut line) {
                Ok(0) | Err(_) => break,
                Ok(_) if line.len() > 16 * 1024 || !line.ends_with('\n') => break,
                Ok(_) => {
                    if send.blocking_send(line).is_err() {
                        break;
                    }
                }
            }
        }
    });
    loop {
        let line = tokio::select! {
            _ = stop_signal() => break,
            line = commands.recv() => match line { Some(line) => line, None => break },
        };
        let result = async {
            let request: serde_json::Value = serde_json::from_str(&line)?;
            if request["command"] != "reload" {
                return Err(crate::host::Error::Input(
                    "unsupported development command".into(),
                ));
            }
            let path = request["path"]
                .as_str()
                .ok_or_else(|| crate::host::Error::Input("reload path is required".into()))?;
            let path = PathBuf::from(path).canonicalize()?;
            let index = allowed
                .iter()
                .position(|allowed| allowed.as_ref() == Some(&path))
                .ok_or_else(|| {
                    crate::host::Error::Input(
                        "bundle path was not selected for this development session".into(),
                    )
                })?;
            let candidate = expected_plugins(std::slice::from_ref(&path))?;
            if candidate[0].package != expected[index].package {
                return Err(crate::host::Error::Input(
                    "package name changed; restart the development command".into(),
                ));
            }
            runtime
                .reload_plugin(dokime_runtime::plugins::PluginBundle::new(path))
                .await?;
            verify_plugins(&runtime.handle(), &candidate).await?;
            let rerun = if request["rerun"].as_bool() == Some(true) {
                Some(rerun_latest_case(&runtime.handle(), &candidate[0].owners).await)
            } else {
                None
            };
            Ok::<_, crate::host::Error>(rerun)
        }
        .await;
        let reply = match result {
            Ok(rerun) => {
                serde_json::json!({"event":"reloaded", "rerun_message":rerun})
            }
            Err(error) => serde_json::json!({"event":"reload_failed", "message":error.to_string()}),
        };
        if writeln!(std::io::stdout().lock(), "{reply}").is_err() {
            break;
        }
    }
}

async fn rerun_latest_case(handle: &dokime_runtime::RuntimeHandle, owners: &[String]) -> String {
    let cycles = handle.cycles().await;
    let latest = cycles
        .iter()
        .flat_map(|cycle| {
            cycle
                .test_executions
                .iter()
                .filter(|execution| {
                    execution
                        .snapshot
                        .plugin()
                        .is_some_and(|plugin| owners.contains(&plugin.source))
                        && cycle
                            .cases()
                            .any(|case| case.test_case_id == execution.test_case_id)
                })
                .map(move |execution| (cycle, execution))
        })
        .max_by_key(|(_, execution)| execution.started_at.as_ref());
    let Some((cycle, execution)) = latest else {
        return "Run a case in the desktop GUI to select it for automatic re-run.".into();
    };
    if let Err(error) = handle
        .rerun_cycle_case(
            cycle.test_cycle_id.clone(),
            execution.test_case_id.clone(),
            Default::default(),
        )
        .await
    {
        return format!("Automatic re-run unavailable: {error}");
    }
    let receipt = handle
        .run_cycle_case(
            cycle.test_cycle_id.clone(),
            execution.test_case_id.clone(),
            Default::default(),
        )
        .await;
    match receipt.rejection {
        Some(error) => format!("Automatic re-run refused: {}", error.message),
        None => "Re-running the latest case with its retained inputs.".into(),
    }
}

fn emit_phase(dev_control: bool, name: &str) {
    if !dev_control {
        return;
    }
    use std::io::Write;
    let phase = serde_json::json!({ "event": "phase", "name": name });
    let _ = writeln!(std::io::stdout().lock(), "{phase}");
}

fn print_startup(
    terminal: &dokime::terminal::Terminal,
    application_root: &Path,
    storage_root: &Path,
    bind_addr: SocketAddr,
    local_url: &str,
) {
    terminal.header("dokime-server", env!("CARGO_PKG_VERSION"), "Server");
    terminal.field("Bind", bind_addr);
    terminal.field("Application", application_root.display());
    terminal.field("Data", storage_root.display());
    terminal.field("Transport", "gRPC");
    terminal.success(local_url);
    terminal.hint("Press Ctrl-C to stop");
}

fn expected_plugins(paths: &[PathBuf]) -> crate::host::Result<Vec<ExpectedPlugin>> {
    use sha2::{Digest, Sha256};
    paths
        .iter()
        .map(|path| {
            // Match the runtime's bundle input bound before allocation.
            use std::io::Read;
            const MAX_BYTES: u64 = 272 * 1024 * 1024;
            if !std::fs::symlink_metadata(path)?.file_type().is_file() {
                return Err(crate::host::Error::Input(
                    "plugin bundle must be a regular file".into(),
                ));
            }
            let mut bytes = Vec::new();
            std::fs::File::open(path)?
                .take(MAX_BYTES + 1)
                .read_to_end(&mut bytes)?;
            if bytes.len() as u64 > MAX_BYTES {
                return Err(crate::host::Error::Input(format!(
                    "plugin bundle too large: {}",
                    path.display()
                )));
            }
            let (_, metadata) = dokime_runtime::plugins::inspect(&bytes)
                .map_err(|error| crate::host::Error::Input(error.to_string()))?;
            Ok(ExpectedPlugin {
                package: metadata.identity.name,
                digest: Sha256::digest(&bytes)
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>(),
                owners: metadata.owners,
            })
        })
        .collect()
}

async fn verify_plugins(
    handle: &dokime_runtime::RuntimeHandle,
    expected: &[ExpectedPlugin],
) -> crate::host::Result<()> {
    use dokime_runtime::runtime::query::{
        RuntimeResolution, RuntimeViewPayload, RuntimeViewQuery, RuntimeViewTopic, RuntimeWindow,
    };
    if expected.is_empty() {
        return Ok(());
    }
    let frame = handle
        .query_runtime_view(RuntimeViewQuery {
            scope: None,
            topic: RuntimeViewTopic::InstalledPlugins,
            window: RuntimeWindow::Full,
            resolution: RuntimeResolution::Raw,
            cursor: None,
        })
        .await?;
    let RuntimeViewPayload::InstalledPlugins { frame } = frame.payload else {
        return Err(crate::host::Error::Input(
            "runtime returned an unexpected admission view".into(),
        ));
    };
    for plugin in expected {
        for owner in &plugin.owners {
            let admitted = frame.installed_plugins.iter().any(|artifact| {
                artifact.enabled && artifact.plugin_id.as_str() == owner
                    && matches!(&artifact.owner,
                        Some(dokime_runtime::state::PluginOwner::Bundle { package, content_digest, installation: None, .. })
                        if package == &plugin.package && content_digest == &plugin.digest)
            });
            if !admitted {
                return Err(crate::host::Error::Input(format!(
                    "plugin {} owner {owner} was not admitted; see runtime diagnostics",
                    plugin.package
                )));
            }
        }
    }
    Ok(())
}

async fn shutdown_signal(stdin: bool) {
    if !stdin {
        stop_signal().await;
        return;
    }
    let (closed, eof) = tokio::sync::oneshot::channel();
    // A dedicated thread cannot hold Tokio's blocking pool open on Ctrl-C.
    std::thread::spawn(move || {
        let _ = std::io::copy(&mut std::io::stdin().lock(), &mut std::io::sink());
        let _ = closed.send(());
    });
    tokio::select! {
        _ = stop_signal() => {}
        _ = eof => {}
    }
}

/// Both Service Termination And Interactive Stop Finalize Persisted Evidence.
async fn stop_signal() {
    #[cfg(unix)]
    {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut terminate) => {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => {},
                    _ = terminate.recv() => {},
                }
            }
            Err(error) => {
                tracing::error!(%error, "could not install SIGTERM handler");
                let _ = tokio::signal::ctrl_c().await;
            }
        }
    }
    #[cfg(not(unix))]
    let _ = tokio::signal::ctrl_c().await;
}

fn arm_force_quit() {
    eprintln!("Stopping Dokime. Press Ctrl-C again to exit without finishing.");

    tokio::spawn(force_quit(tokio::signal::ctrl_c(), |code| {
        std::process::exit(code)
    }));
}

async fn force_quit(
    signal: impl std::future::Future<Output = std::io::Result<()>>,
    exit: impl FnOnce(i32),
) {
    let _ = signal.await;
    eprintln!("Interrupted again: exiting without finishing the orderly stop.");
    exit(130);
}

/* === Tests === */
