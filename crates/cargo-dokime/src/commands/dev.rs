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
    collections::BTreeMap,
    path::{Path, PathBuf},
    process::Stdio,
    time::{Duration, Instant, SystemTime},
};
// Third Party Imports
use harmos::bundle::{Bundle, Limits};
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
};
// Local Imports
use super::{
    backend as host, build, environment::required_environment_references,
    installation::gui_override,
};
use crate::error::Error;
use dokime_packaging::process;
use dokime_terminal::{ColorMode, Terminal};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */
type Snapshot = BTreeMap<PathBuf, (SystemTime, u64)>;

/* === Structs === */
#[derive(clap::Args)]
#[group(id = "development")]
#[command(about = "Watch a plugin, build its bundle, and run it in an isolated Dokime host")]
pub struct Args {
    #[command(flatten)]
    build: build::Args,
    #[arg(
        long,
        help = "Server Override. Defaults To The Installed Dokime Server Executable."
    )]
    dokime: Option<PathBuf>,
    #[arg(
        long,
        help = "Explicit GUI executable. The default is the one beside the host."
    )]
    gui: Option<PathBuf>,
    #[arg(
        long,
        help = "Keep the terminal-only development session without launching a GUI."
    )]
    headless: bool,
    #[arg(
        long,
        help = "Keep development runtime data here. Overrides the temporary DOKIME_DEV_ROOT location."
    )]
    state_dir: Option<PathBuf>,
    #[arg(
        long,
        help = "Load environment variables for the development host from exactly this file, instead of letting it search upward from its working directory for `.env`. Refused outright when the file does not exist — never a silent skip. A variable already exported in this shell always wins over one from either source, and the host reads its environment once at spawn: changing the file requires restarting `dev`."
    )]
    env_file: Option<PathBuf>,
    #[arg(
        long,
        help = "Working directory for the development host; defaults to the directory `cargo dokime dev` was invoked from. Only the host process moves — this crate's own manifest, build and watch paths are unaffected. Without `--env-file`, this is also where the host's `.env` walk-up starts."
    )]
    cwd: Option<PathBuf>,
    #[arg(long, default_value_t = 8765)]
    port: u16,
    #[arg(
        long,
        help = "Re-run the most recently executed case from this plugin after each successful reload."
    )]
    rerun: bool,
    #[arg(
        long,
        help = "Additional source directory or file to watch; repeat for multiple paths."
    )]
    watch: Vec<PathBuf>,
    #[arg(
        long,
        help = "Include host runtime diagnostics alongside plugin output."
    )]
    verbose: bool,
    #[arg(
        long,
        help = "Paths to exclude from source watching; repeat as needed."
    )]
    ignore: Vec<PathBuf>,
    #[arg(
        long,
        hide = true,
        help = "Stop when the stdin control pipe closes (used by supervised development)."
    )]
    shutdown_stdin: bool,
}

#[derive(clap::Args)]
#[command(
    about = "Check plugin tools, SDK access, host compatibility and watched paths without building"
)]
pub struct DoctorArgs {
    #[arg(long)]
    manifest_path: Option<PathBuf>,
    #[arg(short, long)]
    package: Option<String>,
    #[arg(long)]
    dokime: Option<PathBuf>,
}

struct Host {
    process: Child,
    control: Option<ChildStdin>,
    events: BufReader<ChildStdout>,
    gui: Option<Gui>,
}

struct Gui {
    process: Child,
    control: Option<ChildStdin>,
}

/* === Enums === */
enum ProcessExit {
    Backend(std::process::ExitStatus),
    Gui(std::process::ExitStatus),
}

#[derive(Debug, PartialEq, Eq)]
enum Control {
    Retry,
    Quit,
}

enum ReadinessLine {
    Phase,
    Ready(String),
}

/* === Traits === */
trait ShutdownOwner {
    async fn drain(&mut self, terminal: &Terminal) -> Result<(), Error>;
    fn terminate(&mut self) -> Result<(), Error>;
    async fn reap(&mut self) -> Result<(), Error>;
}

/* === Implementations === */
impl ShutdownOwner for Host {
    async fn drain(&mut self, terminal: &Terminal) -> Result<(), Error> {
        drain(self, terminal).await
    }

    fn terminate(&mut self) -> Result<(), Error> {
        // Request both kills before reaping either process.
        let gui_kill = self.gui.as_mut().map(|gui| gui.process.start_kill());
        let host_kill = self.process.start_kill();
        if let Some(result) = gui_kill {
            result?;
        }
        host_kill?;
        Ok(())
    }

    async fn reap(&mut self) -> Result<(), Error> {
        let (gui, backend) = tokio::join!(
            async {
                if let Some(gui) = self.gui.as_mut() {
                    reap(&mut gui.process).await?;
                }
                Ok::<_, Error>(())
            },
            reap(&mut self.process),
        );
        gui?;
        backend
    }
}

/* === Public Functions === */
pub async fn doctor(args: DoctorArgs) -> Result<(), Error> {
    let terminal = Terminal::default();
    terminal.header("Dokime", env!("CARGO_PKG_VERSION"), "doctor");
    for tool in ["cargo", "rustc", "git"] {
        let mut command = std::process::Command::new(tool);
        command.arg("--version");
        let output = process::run(
            command,
            tool,
            Some(Duration::from_secs(10)),
            16 * 1024,
            16 * 1024,
        )
        .await?;
        terminal.field(tool, String::from_utf8_lossy(&output.stdout).trim());
    }
    let installation = host::resolve(args.dokime.as_deref())?;
    let identity = host::inspect(installation.executable()).await?;
    terminal.field(
        "Host",
        format!(
            "{} · {}",
            installation.executable().display(),
            installation.origin().label()
        ),
    );
    match installation.gui() {
        Ok(gui) => terminal.field("GUI", gui.display()),
        // Only `dev` without --headless needs the GUI, so report its absence
        // here instead of refusing the whole check.
        Err(error) => terminal.warn(error),
    }
    let build = build::Args {
        manifest_path: args.manifest_path,
        package: args.package,
        ..build::Args::default()
    };
    let manifest = build
        .manifest_path
        .clone()
        .unwrap_or_else(|| PathBuf::from("Cargo.toml"))
        .canonicalize()?;
    let (roots, target, _) = watch_roots(&build, &[], &manifest, &identity, &terminal).await?;
    terminal.field("Build output", target.display());
    for root in roots {
        terminal.field("Watch", root.display());
    }
    terminal.success("Development environment ready · SDK dependencies resolved");
    Ok(())
}

pub async fn run(mut args: Args) -> Result<(), Error> {
    let terminal = Terminal::new(ColorMode::Auto);
    if args.build.output.is_some() {
        return Err(Error::Invalid(
            "dev owns its isolated bundle output; use build --output for distribution files".into(),
        ));
    }
    let installation = host::resolve(args.dokime.as_deref())?;
    let identity = host::inspect(installation.executable()).await?;
    let gui = if args.headless {
        None
    } else {
        Some(match args.gui.as_deref() {
            Some(path) => gui_override(path)?,
            None => installation.gui()?,
        })
    };
    terminal.header("Dokime", env!("CARGO_PKG_VERSION"), "dev");
    terminal.field(
        "Host",
        format!(
            "{} · {} · {}",
            identity.version,
            installation.executable().display(),
            installation.origin().label()
        ),
    );
    let cwd = resolve_cwd(args.cwd.as_deref())?;
    args.env_file = resolve_env_file(args.env_file.as_deref())?;
    terminal.field("Working directory", cwd.display());
    print_environment_source(&terminal, args.env_file.as_deref(), &cwd);
    let root = if args.state_dir.is_none() {
        development_root(
            args.env_file.as_deref(),
            &cwd,
            std::env::var_os("DOKIME_DEV_ROOT"),
        )?
    } else {
        None
    };
    let (state, _temporary) = development_state(args.state_dir.as_deref(), root.as_deref())?;
    terminal.field("Development data", state.display());
    let manifest = args
        .build
        .manifest_path
        .clone()
        .unwrap_or_else(|| PathBuf::from("Cargo.toml"))
        .canonicalize()?;
    args.build.manifest_path = Some(manifest.clone());
    let root = manifest.parent().expect("manifest parent");
    args.build.output = Some(state.join("development.dokime"));
    let mut excluded = vec![
        state.clone(),
        std::path::absolute(
            args.build
                .target_dir
                .clone()
                .unwrap_or_else(|| root.join("target")),
        )?,
    ];
    excluded.extend(
        args.ignore
            .iter()
            .map(std::path::absolute)
            .collect::<Result<Vec<_>, _>>()?,
    );
    // A broken first build still has a source root to watch and recover from.
    let mut roots = vec![root.to_owned()];
    for path in &args.watch {
        roots.push(std::path::absolute(path)?);
    }
    roots.sort();
    roots.dedup();
    let mut observed = Snapshot::new();
    let mut running: Option<Host> = None;
    let mut first = true;
    let mut last_bundle = None;
    let mut controls = controls(args.shutdown_stdin)?;
    let mut input_open = true;
    let result = async {
        loop {
            let mut retry = false;
            if !first {
                tokio::select! {
                    _ = tokio::signal::ctrl_c() => break,
                    control = controls.recv(), if input_open => match control {
                        Some(Control::Quit) => break,
                        Some(Control::Retry) => {
                            retry = true;
                            last_bundle = None;
                            terminal.status("Retry requested · rebuilding…");
                        }
                        None => input_open = false,
                    },
                    exit = wait_for_exit(&mut running) => match exit? {
                        ProcessExit::Backend(status) => {
                            terminal.error(format!("Development host exited ({}). Waiting for changes…", status));
                            if let Some(mut host) = running.take() {
                                stop(&mut host, &terminal).await?;
                            }
                        }
                        ProcessExit::Gui(status) => {
                            terminal.warn(format!(
                                "Development GUI exited ({}); stopping the development session",
                                status
                            ));
                            break;
                        }
                    },
                    _ = tokio::time::sleep(Duration::from_millis(500)) => {}
                }
            }
            let changed = snapshot(&roots, &excluded)?;
            if !first && !retry && changed == observed { continue; }
            if !first && !retry { terminal.status("Source changed · rebuilding…"); }
            first = false;
            observed = changed;
            let iteration_started = Instant::now();
            // Coalesce editor save bursts before starting a Cargo invocation.
            tokio::time::sleep(Duration::from_millis(150)).await;
            observed = snapshot(&roots, &excluded)?;
            // Process execution owns cancellation so compiler descendants are settled before host shutdown.
            let built = async {
                let (refreshed, target, metadata) = watch_roots(&args.build, &args.watch, &manifest, &identity, &terminal).await?;
                excluded[1] = target;
                if roots != refreshed {
                    roots = refreshed;
                    observed = snapshot(&roots, &excluded)?;
                }
                let bundle = metadata.build().await?;
                Ok::<_, Error>(bundle)
            }.await;
            match built {
                Ok(bundle) => {
                    use sha2::Digest;
                    let bytes = std::fs::read(&bundle)?;
                    let digest = sha2::Sha256::digest(&bytes).to_vec();
                    if running.is_some() && last_bundle.as_ref() == Some(&digest) {
                        terminal.success("Bundle unchanged · current plugin remains running");
                        continue;
                    }
                    let started = Instant::now();
                    if running.is_none() {
                        if let Err(error) =
                            require_declared_secrets(args.env_file.as_deref(), &cwd, &bytes)
                        {
                            terminal.error(error.to_string());
                            terminal.hint("No development host is running. Watching for changes · r + Enter to retry · q + Enter to stop");
                            continue;
                        }
                        terminal.status("Starting development host…");
                        match launch(&args, installation.executable(), &state, &cwd, &bundle, gui.as_deref()).await {
                            Ok((host, url)) => {
                                running = Some(host);
                                terminal.field("Local", url);
                            }
                            Err(Error::Cancelled) => break,
                            Err(error) => {
                                terminal.error(error.to_string());
                                terminal.hint("No development host is running. Watching for changes · r + Enter to retry · q + Enter to stop");
                                continue;
                            }
                        }
                    }
                    last_bundle = None;
                    terminal.status("Reloading plugin · cancelling active executions…");
                    match reload(running.as_mut().expect("host started"), &bundle, args.rerun).await {
                        Ok(()) => {
                            last_bundle = Some(digest);
                            terminal.success(format!("Plugin ready · {:.1}s total, {:.1}s reload", iteration_started.elapsed().as_secs_f64(), started.elapsed().as_secs_f64()));
                        },
                        Err(Error::Cancelled) => break,
                        Err(error) => terminal.error(format!("Plugin reload failed: {error}")),
                    }
                    terminal.hint("Watching for changes · r + Enter to retry · q + Enter or Ctrl+C to stop");
                }
                Err(Error::Cancelled) => break,
                Err(error) => {
                    terminal.error(format!("Build failed: {error}"));
                    terminal.hint(if running.is_some() { "Previous plugin build is still running. Watching for changes · r + Enter to retry · q + Enter to stop" } else { "No development host is running. Watching for changes · r + Enter to retry · q + Enter to stop" });
                }
            }
        }
        Ok(())
    }.await;
    if let Some(mut host) = running {
        stop(&mut host, &terminal).await?;
    }
    result
}

/* === Private Functions === */
async fn wait_for_exit(running: &mut Option<Host>) -> Result<ProcessExit, Error> {
    loop {
        if let Some(host) = running.as_mut() {
            if let Some(status) = host.process.try_wait()? {
                return Ok(ProcessExit::Backend(status));
            }
            if let Some(gui) = host.gui.as_mut()
                && let Some(status) = gui.process.try_wait()?
            {
                return Ok(ProcessExit::Gui(status));
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

fn read_controls(
    mut input: impl std::io::BufRead,
    sender: tokio::sync::mpsc::Sender<Control>,
    shutdown_stdin: bool,
) {
    let mut line = String::new();
    loop {
        line.clear();
        match input.read_line(&mut line) {
            Ok(0) | Err(_) => {
                if shutdown_stdin {
                    let _ = sender.blocking_send(Control::Quit);
                }
                break;
            }
            Ok(_) => {
                let control = match line.trim() {
                    "r" => Control::Retry,
                    "q" => Control::Quit,
                    _ => continue,
                };
                if sender.blocking_send(control).is_err() {
                    break;
                }
            }
        }
    }
}

fn controls(shutdown_stdin: bool) -> Result<tokio::sync::mpsc::Receiver<Control>, Error> {
    let (sender, receiver) = tokio::sync::mpsc::channel(8);
    // Tokio stdin uses a blocking-pool read that can prevent runtime shutdown.
    // A dedicated reader can remain blocked on a terminal when the process exits.
    std::thread::Builder::new()
        .name("development-input".into())
        .spawn(move || read_controls(std::io::stdin().lock(), sender, shutdown_stdin))?;
    Ok(receiver)
}

fn arguments(args: &Args, state: &Path, bundle: &Path) -> Vec<String> {
    let mut arguments = vec![
        "--plugin".to_owned(),
        bundle.display().to_string(),
        "--application-root".to_owned(),
        state.display().to_string(),
        "--shutdown-stdin".to_owned(),
        "--dev-control".to_owned(),
        "--port".to_owned(),
        args.port.to_string(),
    ];
    if let Some(env_file) = &args.env_file {
        arguments.push("--env-file".to_owned());
        arguments.push(env_file.display().to_string());
    }
    arguments
}

async fn readiness_line(
    events: &mut (impl tokio::io::AsyncBufRead + Unpin),
) -> Result<ReadinessLine, Error> {
    let mut bytes = Vec::new();
    events
        .take(16 * 1024 + 1)
        .read_until(b'\n', &mut bytes)
        .await?;
    if bytes.len() > 16 * 1024 || !bytes.ends_with(b"\n") {
        return Err(Error::Invalid(
            "Dokime did not report readiness; review the host diagnostics above".into(),
        ));
    }
    let value: serde_json::Value = serde_json::from_slice(&bytes)?;
    if value["event"] == "phase" {
        return Ok(ReadinessLine::Phase);
    }
    if value["event"] != "ready"
        || value["protocol"] != dokime_contract::READINESS_PROTOCOL
        || value["transport"] != "grpc"
    {
        return Err(Error::Invalid(
            "Dokime returned an unsupported development readiness message".into(),
        ));
    }
    let url = value["endpoint"]
        .as_str()
        .ok_or_else(|| Error::Invalid("Dokime readiness omitted its gRPC endpoint".into()))?;
    let url = url::Url::parse(url)
        .map_err(|error| Error::Invalid(format!("invalid host URL: {error}")))?;
    if url.scheme() != "http"
        || !matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "[::1]"))
    {
        return Err(Error::Invalid(
            "development host returned a non-local URL".into(),
        ));
    }
    Ok(ReadinessLine::Ready(url.to_string()))
}

async fn read_readiness(
    events: &mut (impl tokio::io::AsyncBufRead + Unpin),
) -> Result<String, Error> {
    loop {
        match readiness_line(events).await? {
            ReadinessLine::Phase => continue,
            ReadinessLine::Ready(url) => return Ok(url),
        }
    }
}

async fn launch(
    args: &Args,
    executable: &Path,
    state: &Path,
    cwd: &Path,
    bundle: &Path,
    gui: Option<&Path>,
) -> Result<(Host, String), Error> {
    let mut command = Command::new(executable);
    let color = match dokime_terminal::color_mode() {
        ColorMode::Auto => "auto",
        ColorMode::Always => "always",
        ColorMode::Never => "never",
    };
    command
        .arg("--color")
        .arg(color)
        .args(arguments(args, state, bundle))
        .current_dir(cwd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit());
    if !args.verbose && std::env::var_os("RUST_LOG").is_none() {
        // Dev focuses on the selected plugin: both its stderr and execution
        // log are mirrored under `sidecar`. Host and dependency diagnostics
        // remain available through --verbose or an explicit RUST_LOG.
        command.env("RUST_LOG", "off,sidecar=debug");
    }
    let mut child = command.spawn()?;
    let stdout = child.stdout.take().expect("piped control output");
    let control = child.stdin.take();
    let mut host = Host {
        process: child,
        control,
        events: BufReader::new(stdout),
        gui: None,
    };
    // Keep the pipe open through cancellation drain: startup may still publish
    // readiness after Ctrl+C, before it observes the closed stdin control pipe.
    let result = tokio::select! {
        _ = tokio::signal::ctrl_c() => Err(Error::Cancelled),
        result = read_readiness(&mut host.events) => result,
    };
    match result {
        Ok(url) => {
            if let Some(gui) = gui {
                match launch_gui(gui, &url).await {
                    Ok(process) => host.gui = Some(process),
                    Err(error) => {
                        // Readiness means the backend is already serving. Do
                        // not leave it behind when the optional GUI cannot
                        // start.
                        let _ = stop(&mut host, &Terminal::new(ColorMode::Auto)).await;
                        return Err(error);
                    }
                }
            }
            Ok((host, url))
        }
        Err(error) => {
            stop(&mut host, &Terminal::new(ColorMode::Auto)).await?;
            Err(error)
        }
    }
}

async fn launch_gui(gui: &Path, endpoint: &str) -> Result<Gui, Error> {
    let mut command = Command::new(gui);
    command
        .arg("--endpoint")
        .arg(endpoint)
        .arg("--shutdown-stdin")
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    let mut process = command.spawn()?;
    Ok(Gui {
        control: process.stdin.take(),
        process,
    })
}

async fn reload(host: &mut Host, bundle: &Path, rerun: bool) -> Result<(), Error> {
    let request = serde_json::json!({"command":"reload", "path":bundle, "rerun":rerun});
    let control = host
        .control
        .as_mut()
        .ok_or_else(|| Error::Invalid("host control is closed".into()))?;
    control.write_all(format!("{request}\n").as_bytes()).await?;
    control.flush().await?;
    let mut bytes = Vec::new();
    tokio::select! {
        _ = tokio::signal::ctrl_c() => return Err(Error::Cancelled),
        result = async { (&mut host.events).take(64 * 1024 + 1).read_until(b'\n', &mut bytes).await } => { result?; }
    }
    if bytes.len() > 64 * 1024 || !bytes.ends_with(b"\n") {
        return Err(Error::Invalid(
            "host did not acknowledge plugin reload".into(),
        ));
    }
    let reply: serde_json::Value = serde_json::from_slice(&bytes)?;
    match reply["event"].as_str() {
        Some("reloaded") => {
            let terminal = Terminal::default();
            if let Some(message) = reply["rerun_message"].as_str() {
                terminal.status(message);
            }
            if let Some(error) = reply["rerun_error"].as_str() {
                terminal.warn(format!("Plugin reloaded, but rerun failed: {error}"));
            }
            Ok(())
        }
        Some("reload_failed") => Err(Error::Invalid(
            reply["message"]
                .as_str()
                .unwrap_or("plugin activation failed")
                .to_owned(),
        )),
        _ => Err(Error::Invalid("unsupported reload response".into())),
    }
}

async fn stop(host: &mut Host, terminal: &Terminal) -> Result<(), Error> {
    stop_with(host, terminal, tokio::signal::ctrl_c()).await
}

async fn stop_with(
    owner: &mut impl ShutdownOwner,
    terminal: &Terminal,
    interrupted: impl std::future::Future<Output = std::io::Result<()>>,
) -> Result<(), Error> {
    terminal.status("Stopping Dokime. Press Ctrl-C again to terminate the development session.");
    let reason = tokio::select! {
        result = tokio::time::timeout(Duration::from_secs(30), owner.drain(terminal)) => {
            match result {
                Ok(Ok(())) => return Ok(()),
                Ok(Err(error)) => format!("Orderly shutdown failed: {error}; terminating the development session"),
                Err(_) => "Dokime did not finish within 30 seconds; terminating the development session".to_owned(),
            }
        }
        _ = interrupted => "Interrupted again; terminating the development session".to_owned(),
    };

    terminal.warn(reason);
    owner.terminate()?;
    owner.reap().await
}

async fn reap(process: &mut Child) -> Result<(), Error> {
    tokio::time::timeout(Duration::from_secs(5), process.wait())
        .await
        .map_err(|_| Error::Invalid("terminated process did not exit within 5 seconds".into()))??;
    Ok(())
}

async fn drain(host: &mut Host, terminal: &Terminal) -> Result<(), Error> {
    // Close both controls immediately, so the host can drain while the GUI exits.
    drop(host.control.take());
    if let Some(gui) = host.gui.as_mut() {
        drop(gui.control.take());
        match tokio::time::timeout(Duration::from_secs(10), gui.process.wait()).await {
            Ok(result) => {
                result?;
            }
            Err(_) => {
                terminal.warn("Dokime GUI did not close after stdin drain; terminating it");
                gui.process.start_kill()?;
                reap(&mut gui.process).await?;
            }
        }
    }
    host.process.wait().await?;
    Ok(())
}

fn snapshot(roots: &[PathBuf], excluded: &[PathBuf]) -> Result<Snapshot, Error> {
    fn visit(path: &Path, excluded: &[PathBuf], result: &mut Snapshot) -> Result<(), Error> {
        if excluded.iter().any(|excluded| path.starts_with(excluded)) {
            return Ok(());
        }
        let metadata = match std::fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error.into()),
        };
        if metadata.is_symlink() {
            return Ok(());
        }
        if metadata.is_dir() {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                if matches!(
                    entry.file_name().to_str(),
                    Some("target" | ".git" | ".jj" | "node_modules" | ".venv" | "dist")
                ) {
                    continue;
                }
                visit(&entry.path(), excluded, result)?;
            }
        } else if metadata.is_file() {
            result.insert(path.to_owned(), (metadata.modified()?, metadata.len()));
        }
        Ok(())
    }
    let mut result = Snapshot::new();
    for root in roots {
        visit(root, excluded, &mut result)?;
    }
    Ok(result)
}

async fn watch_roots(
    build: &build::Args,
    watch: &[PathBuf],
    manifest: &Path,
    identity: &host::Identity,
    terminal: &Terminal,
) -> Result<(Vec<PathBuf>, PathBuf, dokime_packaging::build::Project), Error> {
    let mut options = build.clone();
    options.manifest_path = Some(manifest.to_owned());
    let project = dokime_packaging::build::Project::resolve(options).await?;
    terminal.field("Plugin", project.package_name());
    let sdk_version = semver::Version::parse(project.sdk_version())
        .map_err(|error| Error::Invalid(error.to_string()))?;
    host::compatible(identity, &sdk_version)?;
    terminal.field(
        "SDK",
        format!(
            "{} · {}",
            project.sdk_version(),
            project.sdk_source().unwrap_or("local path")
        ),
    );
    terminal.success("Compatibility checked");
    let mut roots = project.watch_paths().to_vec();
    for path in watch {
        roots.push(std::path::absolute(path)?);
    }
    roots.sort();
    roots.dedup();
    Ok((roots, project.target_directory().to_owned(), project))
}

fn resolve_cwd(explicit: Option<&Path>) -> Result<PathBuf, Error> {
    match explicit {
        Some(path) => {
            if !path.is_dir() {
                return Err(Error::Invalid(format!(
                    "--cwd {} does not exist or is not a directory",
                    path.display()
                )));
            }
            Ok(path.canonicalize()?)
        }
        None => Ok(std::env::current_dir()?),
    }
}

fn resolve_env_file(explicit: Option<&Path>) -> Result<Option<PathBuf>, Error> {
    match explicit {
        Some(path) if !path.is_file() => Err(Error::Invalid(format!(
            "--env-file {} does not exist",
            path.display()
        ))),
        Some(path) => Ok(Some(std::path::absolute(path)?)),
        None => Ok(None),
    }
}

fn locate_dotenv(directory: &Path) -> Option<PathBuf> {
    let mut current = directory.to_owned();
    loop {
        let candidate = current.join(".env");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn development_root(
    env_file: Option<&Path>,
    cwd: &Path,
    exported: Option<std::ffi::OsString>,
) -> Result<Option<PathBuf>, Error> {
    let root = match exported {
        Some(value) => Some(value),
        None => match env_file.map(Path::to_owned).or_else(|| locate_dotenv(cwd)) {
            Some(path) => environment_file_keys(&path)?
                .remove("DOKIME_DEV_ROOT")
                .map(Into::into),
            None => None,
        },
    };
    root.map(|root| {
        if root.is_empty() {
            return Err(Error::Invalid("DOKIME_DEV_ROOT must not be empty".into()));
        }
        Ok(cwd.join(root))
    })
    .transpose()
}

fn development_state(
    persistent: Option<&Path>,
    root: Option<&Path>,
) -> Result<(PathBuf, Option<tempfile::TempDir>), Error> {
    if let Some(path) = persistent {
        std::fs::create_dir_all(path)?;
        return Ok((path.canonicalize()?, None));
    }

    let temporary = match root {
        Some(root) => {
            std::fs::create_dir_all(root)?;
            tempfile::Builder::new()
                .prefix("dokime-dev-")
                .tempdir_in(root)?
        }
        None => tempfile::tempdir()?,
    };
    Ok((temporary.path().canonicalize()?, Some(temporary)))
}

fn print_environment_source(terminal: &Terminal, env_file: Option<&Path>, cwd: &Path) {
    match env_file {
        Some(path) => terminal.success(format!("Environment · loaded from {}", path.display())),
        None => match locate_dotenv(cwd) {
            Some(path) => terminal.success(format!(
                "Environment · loaded from {} (found searching up from {})",
                path.display(),
                cwd.display()
            )),
            None => terminal.success(format!(
                "Environment · no .env found searching up from {}; only already-exported variables reach the host",
                cwd.display()
            )),
        },
    }
}

fn environment_file_keys(path: &Path) -> Result<BTreeMap<String, String>, Error> {
    let file = std::fs::File::open(path).map_err(|error| {
        Error::Invalid(format!(
            "cannot read environment file {}: {error}",
            path.display()
        ))
    })?;
    read_environment_keys(path, file)
}

fn read_environment_keys(
    path: &Path,
    reader: impl std::io::Read,
) -> Result<BTreeMap<String, String>, Error> {
    let mut values = BTreeMap::new();
    for pair in dotenv::from_read_iter(reader) {
        let (key, value) = pair.map_err(|error| {
            Error::Invalid(format!(
                "cannot read environment file {}: {error}",
                path.display()
            ))
        })?;
        values.entry(key).or_insert(value);
    }
    Ok(values)
}

/// Refuses a launch whose plugin requires a secret that neither the process
/// environment nor the environment file provides. Only the variable names are
/// ever reported, never a value.
fn require_declared_secrets(
    env_file: Option<&Path>,
    cwd: &Path,
    bundle_bytes: &[u8],
) -> Result<(), Error> {
    let bundle = Bundle::inspect(bundle_bytes, Limits::default())?;
    let required = required_environment_references(&bundle)?;
    if required.is_empty() {
        return Ok(());
    }
    let declared_in_file = match env_file.map(Path::to_owned).or_else(|| locate_dotenv(cwd)) {
        Some(path) => environment_file_keys(&path)?,
        None => BTreeMap::new(),
    };
    let missing = required
        .iter()
        .filter(|reference| {
            let exported = std::env::var(&reference.variable).is_ok_and(|value| !value.is_empty());
            let filed = declared_in_file
                .get(&reference.variable)
                .is_some_and(|value| !value.is_empty());
            !exported && !filed
        })
        .map(|reference| {
            format!(
                "{} (secret {:?} of {})",
                reference.variable, reference.setting, reference.owner
            )
        })
        .collect::<Vec<_>>();
    if missing.is_empty() {
        return Ok(());
    }
    Err(Error::Invalid(format!(
        "required environment variable not set: {}; export it or add it to the environment file, then press r + Enter to retry",
        missing.join(", ")
    )))
}

/* === Tests === */
