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
use std::{future::Future, io::Write, net::SocketAddr, path::Path, pin::Pin};

// Third Party Imports

// Local Imports
use dokime::domain::{DEV_PROTOCOL, READINESS_PROTOCOL};
use dokime_server::host;

/* === Constants === */

const DEVELOPMENT_COMMAND_ITEMS: u64 = 8;
const DEVELOPMENT_COMMAND_BYTES: u64 = 16_384;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
#[command(about = "Run The Dokime Server.")]
pub struct Args {
    #[command(flatten)]
    runtime: host::Args,

    #[command(subcommand)]
    command: Option<Command>,
}

/* === Enums === */

#[derive(Debug, clap::Subcommand)]
enum Command {
    Version(host::version::Args),
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn configuration(args: &Args) -> crate::Result<dokime::config::Config> {
    let loaded = load_environment(args.runtime.env_file.as_deref());
    loaded.and_then(|()| args.runtime.configuration()).map_err(|error| crate::Error::Process {
        exit_code: error.exit_code(),
        source: Box::new(error),
    })
}

pub async fn run(args: Args, configuration: dokime::config::Config) -> crate::Result<()> {
    if let Some(Command::Version(args)) = args.command {
        host::version::run(args);
        return Ok(());
    }
    run_server(args.runtime, configuration).await.map_err(|error| crate::Error::Process {
        exit_code: error.exit_code(),
        source: Box::new(error),
    })
}

/* === Private Functions === */

async fn run_server(args: host::Args, config: dokime::config::Config) -> host::Result<()> {
    let network = args.network.config()?;
    let application_root = config.application_root.clone().expect("resolved application root");
    let storage_root = config.output_root.clone();
    let terminal = dokime::terminal::Terminal::new(crate::terminal::color_mode());
    let _guards = host::install_logging(&application_root, terminal.color_enabled())?;
    let network = network.with_trace_journal(_guards.journal());

    let start = host::Session::start(&args, config, network, |name| emit_phase(args.dev_control, name));
    let starting: Pin<Box<dyn Future<Output = host::Result<host::Session>> + '_>> = Box::pin(start);
    let mut session = starting.await?;
    let endpoint = session.endpoint();
    if args.dev_control {
        let ready = serde_json::json!({
            "event": "ready", "protocol": READINESS_PROTOCOL, "endpoint": endpoint,
            "transport": "grpc", "version": env!("CARGO_PKG_VERSION"), "dev_protocol": DEV_PROTOCOL,
        });
        if let Err(error) = writeln!(std::io::stdout().lock(), "{ready}") {
            session.shutdown().await?;
            return Err(error.into());
        }
        development_control(&mut session).await;
    } else {
        print_startup(&terminal, &application_root, &storage_root, session.address(), &endpoint);
        shutdown_signal(args.shutdown_stdin).await;
    }
    arm_force_quit();
    session.shutdown().await
}

fn load_environment(env_file: Option<&Path>) -> host::Result<()> {
    if let Some(path) = env_file {
        return dotenv::from_path(path).map_err(|_| host::Error::Input(format!("cannot load environment file {}", path.display())));
    }
    match dotenv::dotenv() {
        Ok(_) => Ok(()),
        Err(dotenv::Error::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(_) => Err(host::Error::Input("cannot load project .env file".into())),
    }
}

async fn development_control(session: &mut host::Session) {
    use std::io::{BufRead, Write};
    let (send, mut commands) = tokio::sync::mpsc::channel(DEVELOPMENT_COMMAND_ITEMS as usize);
    let maximum = DEVELOPMENT_COMMAND_BYTES;
    std::thread::spawn(move || {
        let mut input = std::io::stdin().lock();
        loop {
            let mut line = String::new();
            // Bound each request independently; EOF still owns host shutdown.
            match std::io::Read::take(&mut input, maximum.saturating_add(1)).read_line(&mut line) {
                Ok(0) | Err(_) => break,
                Ok(_) if line.len() as u64 > maximum || !line.ends_with('\n') => break,
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
            if request["command"] == "storage_counters" {
                let mut reply = serde_json::to_value(dokime::storage::sqlite::counters())?;
                reply["event"] = "storage_counters".into();
                return Ok(reply);
            }
            if request["command"] != "reload" {
                return Err(host::Error::Input("unsupported development command".into()));
            }
            let path = request["path"].as_str().ok_or_else(|| host::Error::Input("reload path is required".into()))?;
            let reloading = session.reload(Path::new(path), request["rerun"].as_bool() == Some(true));
            let reload: Pin<Box<dyn Future<Output = host::Result<Option<String>>> + '_>> = Box::pin(reloading);
            let rerun = reload.await?;
            Ok::<_, host::Error>(serde_json::json!({"event":"reloaded", "rerun_message":rerun}))
        }
        .await;
        let reply = match result {
            Ok(reply) => reply,
            Err(error) => serde_json::json!({"event":"reload_failed", "message":error.to_string()}),
        };
        if writeln!(std::io::stdout().lock(), "{reply}").is_err() {
            break;
        }
    }
}

fn emit_phase(dev_control: bool, name: &str) {
    if !dev_control {
        return;
    }
    let phase = serde_json::json!({ "event": "phase", "name": name });
    let _ = writeln!(std::io::stdout().lock(), "{phase}");
}

fn print_startup(
    terminal: &dokime::terminal::Terminal, application_root: &Path, storage_root: &Path, bind_addr: SocketAddr, local_url: &str,
) {
    terminal.header("dokime server", env!("CARGO_PKG_VERSION"), "Server");
    terminal.field("Bind", bind_addr);
    terminal.field("Application", application_root.display());
    terminal.field("Data", storage_root.display());
    terminal.field("Transport", "gRPC");
    terminal.success(local_url);
    terminal.hint("Press Ctrl-C to stop");
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

    let interrupt = tokio::signal::ctrl_c();
    let exit = |code| std::process::exit(code);
    let quit = force_quit(interrupt, exit);
    tokio::spawn(quit);
}

async fn force_quit(signal: impl std::future::Future<Output = std::io::Result<()>>, exit: impl FnOnce(i32)) {
    if let Err(error) = signal.await {
        tracing::debug!(%error, "force-quit listener unavailable");
        return;
    }
    eprintln!("Interrupted again: exiting without finishing the orderly stop.");
    exit(130);
}

#[cfg(test)]
#[tokio::test]
async fn force_quit_error() {
    let exit = std::cell::Cell::new(None);
    let record_exit = |code| {
        let status = Some(code);
        exit.set(status);
    };
    let error = std::io::Error::other("signal listener unavailable");
    let unavailable = std::future::ready(Err(error));
    force_quit(unavailable, record_exit).await;
    assert_eq!(exit.get(), None, "a listener error must preserve orderly shutdown");

    let interrupt = std::future::ready(Ok(()));
    force_quit(interrupt, record_exit).await;
    assert_eq!(exit.get(), Some(130), "a second interrupt must still force exit");
}

/* === Tests === */

#[cfg(all(test, unix))]
#[test]
fn force_quit_shutdown() {
    let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let handle = runtime.handle().clone();
    drop(runtime);

    let _entered = handle.enter();
    let exit = std::cell::Cell::new(None);
    let record_exit = |code| {
        let status = Some(code);
        exit.set(status);
    };
    let interrupt = tokio::signal::ctrl_c();
    let quit = force_quit(interrupt, record_exit);
    let outcome = futures_util::FutureExt::now_or_never(quit);
    assert_eq!(outcome, Some(()));
    assert_eq!(exit.get(), None, "runtime teardown is not another Ctrl-C");
}
