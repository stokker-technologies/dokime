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
    path::{Path, PathBuf},
    process::{ExitStatus, Stdio},
    time::Duration,
};
// Third Party Imports
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    process::{Child, ChildStdin, ChildStdout, Command},
};
// Local Imports
use super::diagnostics::Capture;
use super::{BackendDiagnostics, DesktopSessionError as Error};
// The development-stream contracts a backend must report to be trusted. The
// desktop and its backend ship from one installer, so this is the same gate
// cargo-dokime applies to a separately installed host.
use dokime_contract::{DEV_PROTOCOL, READINESS_PROTOCOL};

/* === Constants === */
const READINESS_LIMIT: u64 = 16 * 1024;

const READY_TIMEOUT: Duration = Duration::from_secs(60);
const DRAIN_TIMEOUT: Duration = Duration::from_secs(30);

const REAP_TIMEOUT: Duration = Duration::from_secs(5);

#[cfg(any(windows, target_os = "macos"))]
const PRODUCT: &str = "Dokime";
const MISSING: &str = "No Dokime backend was found beside this application. Reinstall Dokime with the desktop installer, set DOKIME_HOME to an installation directory, or attach to a backend endpoint.";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone, Debug)]
pub struct Backend {
    executable: PathBuf,
    arguments: Vec<String>,
    logs: Option<PathBuf>,
}

struct Search {
    beside: Option<PathBuf>,
    home: Option<PathBuf>,
    path: Vec<PathBuf>,
    installed: Vec<PathBuf>,
}

pub struct Process {
    child: Child,
    control: Option<ChildStdin>,
    events: BufReader<ChildStdout>,
    diagnostics: Capture,
}

#[derive(serde::Deserialize)]
struct Identity {
    version: String,
    dev_protocol: u32,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Backend {
    pub fn new(executable: PathBuf, arguments: Vec<String>) -> Self {
        Self {
            executable,
            arguments,
            logs: None,
        }
    }

    /// Streams each launch's redacted stderr into its own file under
    /// `directory`; without one, only the in-memory tail is kept.
    #[must_use]
    pub fn with_logs(mut self, directory: PathBuf) -> Self {
        self.logs = Some(directory);
        self
    }
}

impl Search {
    fn from_environment() -> Self {
        Self {
            // The installer puts `dokime-server` beside `dokime-gui` on every
            // platform, so this application's own directory is the first and
            // most specific answer. A development build finds its sibling in
            // the same Cargo target directory.
            beside: std::env::current_exe()
                .ok()
                .and_then(|executable| executable.parent().map(Path::to_path_buf)),
            home: std::env::var_os("DOKIME_HOME").map(PathBuf::from),
            path: std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
                // An empty PATH entry means the working directory, which is
                // never a trusted source for a desktop session's backend.
                .filter(|directory| !directory.as_os_str().is_empty())
                .collect(),
            installed: installed_directories(),
        }
    }

    fn find(&self) -> Option<PathBuf> {
        self.resolve(executable_file)
    }

    fn resolve(&self, mut executable: impl FnMut(&Path) -> Option<PathBuf>) -> Option<PathBuf> {
        let name = backend_name();
        self.beside
            .iter()
            .chain(self.home.iter())
            .chain(&self.path)
            .chain(&self.installed)
            .find_map(|directory| executable(&directory.join(&name)))
    }
}

impl Process {
    async fn readiness(
        &mut self,
        executable: &Path,
        on_phase: &mut (dyn FnMut(&str) + Send),
    ) -> Result<String, String> {
        tokio::time::timeout(READY_TIMEOUT, async {
            loop {
                let mut line = Vec::new();
                let read = (&mut self.events)
                    .take(READINESS_LIMIT + 1)
                    .read_until(b'\n', &mut line)
                    .await
                    .map_err(|error| {
                        format!("The Dokime backend readiness could not be read: {error}.")
                    })?;
                if read == 0 {
                    return Err("The Dokime backend stopped before reporting readiness; review its diagnostics."
                        .to_owned());
                }
                match phase_name(&line) {
                    Some(phase) => {
                        on_phase(&phase);
                        continue;
                    }
                    None => return endpoint(&line, executable),
                }
            }
        })
        .await
        .map_err(|_| "The Dokime backend did not report readiness in time.".to_owned())?
    }

    pub async fn shutdown(&mut self) -> Result<ExitStatus, String> {
        drop(self.control.take());
        match tokio::time::timeout(DRAIN_TIMEOUT, self.child.wait()).await {
            Ok(status) => status.map_err(|error| Self::failure(error, "stop")),
            Err(_) => {
                // A backend that ignores its closed control pipe still holds the
                // execution store, and the desktop cannot exit until it stops.
                self.child
                    .start_kill()
                    .map_err(|error| Self::failure(error, "terminate"))?;

                match tokio::time::timeout(REAP_TIMEOUT, self.child.wait()).await {
                    Ok(status) => status.map_err(|error| Self::failure(error, "stop")),
                    Err(_) => Err("The Dokime backend did not stop after being                                    terminated; the desktop is exiting without it."
                        .to_owned()),
                }
            }
        }
    }

    fn failure(error: std::io::Error, action: &str) -> String {
        format!("The Dokime backend could not {action}: {error}.")
    }
}

/* === Public Functions === */
pub fn locate() -> Result<PathBuf, String> {
    Search::from_environment()
        .find()
        .ok_or_else(|| MISSING.to_owned())
}

pub async fn start(
    backend: &Backend,
    mut on_phase: impl FnMut(&str) + Send,
) -> Result<(Process, String), Error> {
    // Spawn The Backend
    let executable = backend.executable.display().to_string();
    let mut command = Command::new(&backend.executable);
    command
        .args(&backend.arguments)
        .stdin(Stdio::piped())
        // Stdout is the readiness channel; stderr carries diagnostics only.
        .stdout(Stdio::piped())
        // A desktop launch has no console to inherit, so stderr is captured.
        .stderr(Stdio::piped())
        // A desktop that dies without draining must not leave an owned backend
        // behind holding the execution store.
        .kill_on_drop(true);
    no_console(&mut command);
    let mut child = command.spawn().map_err(|error| Error::Startup {
        message: format!("Dokime backend {executable} could not be started: {error}"),
        diagnostics: Some(Box::new(BackendDiagnostics {
            executable: executable.clone(),
            exit_code: None,
            exit_status: None,
            tail: String::new(),
            log: None,
        })),
    })?;
    let events = BufReader::new(child.stdout.take().expect("piped readiness output"));
    let stderr = child.stderr.take().expect("piped diagnostic output");
    let diagnostics = Capture::start(stderr, backend.logs.as_deref(), child.id()).await;
    let mut process = Process {
        control: child.stdin.take(),
        child,
        events,
        diagnostics,
    };

    // Await Readiness
    let error = match process.readiness(&backend.executable, &mut on_phase).await {
        Ok(endpoint) => return Ok((process, endpoint)),
        Err(error) => error,
    };

    // Report The Failed Launch
    // Readiness failed, but the process may still be running and still
    // holding the execution store. Drain it before reporting.
    let (message, status) = match process.shutdown().await {
        Ok(status) => (error, Some(status)),
        Err(cleanup) => (format!("{error} {cleanup}"), None),
    };
    let (tail, log) = process.diagnostics.settle().await;
    Err(Error::Startup {
        message,
        diagnostics: Some(Box::new(BackendDiagnostics {
            executable,
            exit_code: status.and_then(|status| status.code()),
            exit_status: status.map(|status| status.to_string()),
            tail,
            log: log.map(|path| path.display().to_string()),
        })),
    })
}

/* === Private Functions === */
#[cfg(windows)]
fn installed_directories() -> Vec<PathBuf> {
    ["LOCALAPPDATA", "ProgramFiles"]
        .into_iter()
        .filter_map(std::env::var_os)
        .map(|root| PathBuf::from(root).join(PRODUCT))
        .collect()
}

#[cfg(target_os = "macos")]
fn installed_directories() -> Vec<PathBuf> {
    vec![PathBuf::from(format!(
        "/Applications/{PRODUCT}.app/Contents/MacOS"
    ))]
}

#[cfg(not(any(windows, target_os = "macos")))]
fn installed_directories() -> Vec<PathBuf> {
    vec![PathBuf::from("/usr/bin"), PathBuf::from("/usr/local/bin")]
}

fn backend_name() -> String {
    format!("dokime-server{}", std::env::consts::EXE_SUFFIX)
}

fn executable_file(path: &Path) -> Option<PathBuf> {
    if !path.is_file() {
        return None;
    }
    std::path::absolute(path).ok()
}

fn phase_name(line: &[u8]) -> Option<String> {
    if line.len() as u64 > READINESS_LIMIT || !line.ends_with(b"\n") {
        return None;
    }
    let value: serde_json::Value = serde_json::from_slice(line).ok()?;
    if value["event"] != "phase" {
        return None;
    }
    value["name"].as_str().map(str::to_owned)
}

fn endpoint(line: &[u8], executable: &Path) -> Result<String, String> {
    if line.len() as u64 > READINESS_LIMIT || !line.ends_with(b"\n") {
        return Err("The Dokime backend did not report readiness.".to_owned());
    }
    let ready: serde_json::Value = serde_json::from_slice(line)
        .map_err(|error| format!("The Dokime backend readiness could not be parsed: {error}."))?;
    if ready["event"] != "ready"
        || ready["protocol"] != READINESS_PROTOCOL
        || ready["transport"] != "grpc"
    {
        return Err("The Dokime backend reported an unsupported readiness message.".to_owned());
    }
    // The bundled backend is installed together with this desktop, so the
    // identity the ready line carries must match exactly; anything else means
    // a stray Dokime was discovered. This used to be a separate `dokime
    // version --json` probe spawned before `serve`; carrying it on the ready
    // line instead means a freshly installed backend is spawned exactly once.
    identity(line).map_err(|reason| identity_failure(executable, &reason))?;
    let published = ready["endpoint"]
        .as_str()
        .ok_or_else(|| "The Dokime backend readiness omitted its gRPC endpoint.".to_owned())?;
    let published = url::Url::parse(published)
        .map_err(|error| format!("The Dokime backend reported an invalid endpoint: {error}."))?;
    // An owned backend serves this desktop alone; a published address that is
    // not loopback means something other than the bundled backend answered.
    if published.scheme() != "http"
        || !matches!(
            published.host_str(),
            Some("127.0.0.1" | "localhost" | "[::1]")
        )
    {
        return Err("The Dokime backend reported a non-local endpoint.".to_owned());
    }
    Ok(published.to_string())
}

fn identity_failure(executable: &Path, reason: &str) -> String {
    format!("Dokime backend {} {reason}.", executable.display())
}

fn identity(bytes: &[u8]) -> Result<(), String> {
    let identity: Identity = serde_json::from_slice(bytes)
        .map_err(|error| format!("returned invalid version information: {error}"))?;
    if identity.dev_protocol != DEV_PROTOCOL {
        return Err(format!(
            "uses development protocol {}, and this desktop requires {DEV_PROTOCOL}; reinstall Dokime so the application and its backend ship together",
            identity.dev_protocol
        ));
    }
    if identity.version != env!("CARGO_PKG_VERSION") {
        return Err(format!(
            "reports version {}, and this desktop is {}; reinstall Dokime so the application and its backend ship together",
            identity.version,
            env!("CARGO_PKG_VERSION")
        ));
    }
    Ok(())
}

#[cfg(windows)]
fn no_console(command: &mut Command) {
    command.creation_flags(0x0800_0000);
}

#[cfg(not(windows))]
fn no_console(_command: &mut Command) {}

/* === Tests === */
