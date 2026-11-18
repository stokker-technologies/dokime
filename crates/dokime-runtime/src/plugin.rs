/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    env::{self, consts::EXE_SUFFIX},
    io::{self, ErrorKind},
    num::{NonZeroU32, NonZeroUsize},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::Duration,
};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

// Third Party Imports
use harmos_operations::plugins::Delete;
use harmos_protocol::plugin::instance_client::InstanceClient;
use harmos_protocol::plugin::{InstanceGetRequest, InstanceGetResponse, InstanceStopRequest};
use harmos_runtime::operations::ProtobufOperation;
use harmos_runtime::plugin::grpc::{GrpcPluginPolicy, GrpcPluginSettings};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::ChildStdout,
    task::JoinHandle,
    time,
};
use tonic::transport::{Channel, Endpoint};

// Local Imports
use crate::{Error, Result};

/* === Constants === */

const PROCESS_POLL_INTERVAL: Duration = Duration::from_millis(25);

/// The Plugins Operation Timeout: It Bounds a Plugin's Load and a Whole Managed Plugin Teardown.
pub const PLUGINS_TIMEOUT: Duration = <Delete as ProtobufOperation>::POLICY.timeout;
/// The Bound of Each Teardown Phase: Settle, Stop and Exit, Then Kill and Reap.
pub const STOP_GRACE: Duration = PLUGINS_TIMEOUT.checked_div(3).unwrap();
/// The Default Interval Window: How Much Sampling Time One Host Grant Covers.
///
/// Longer windows batch more samples per message and stored row; shorter ones deliver live
/// samples sooner. A stream whose observer names a window setting takes that plugin setting instead.
pub const INTERVAL_WINDOW: Duration = Duration::from_millis(100);

/// How Many Window Lengths an Interval Demand May Fall Behind a Window's End Before It Fails.
///
/// Clock jitter passes; a host that cannot keep up with a stream's rate fails within a few windows
/// instead of granting ever longer ones.
pub const INTERVAL_LAG: NonZeroU32 = NonZeroU32::new(10).unwrap();

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct PlacementSettings {
    pub reconnect_delay: Duration,
    pub stream_window: NonZeroUsize,
    /// Sampling Time One Interval Window Covers, Unless a Stream's Window Setting Overrides It.
    pub interval_window: Duration,
    /// Window Lengths an Interval Demand May Fall Behind Before It Fails, Default [`INTERVAL_LAG`].
    pub interval_lag: NonZeroU32,
    pub message_bytes: NonZeroUsize,
}

pub struct PluginProcess {
    endpoint: String,
    child: OwnedChild,
    drain: JoinHandle<()>,
    remote: bool,
    settings: PlacementSettings,
}

struct OwnedChild(Child);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl PluginProcess {
    pub async fn start(executable: &Path, remote: bool, settings: &PlacementSettings) -> Result<Self> {
        Self::start_with_environment(executable, remote, settings, &[]).await
    }
    pub async fn start_with_environment(
        executable: &Path, remote: bool, settings: &PlacementSettings, environment: &[(String, String)],
    ) -> Result<Self> {
        let python = executable.is_dir();
        let mut command = if python {
            let uv = uv_executable()?;
            python_command(executable, &uv)?
        } else {
            crate::process::command(executable)
        };
        command.envs(environment.iter().cloned());
        if python {
            command.env_remove("UV_PROJECT_ENVIRONMENT");
        }
        #[cfg(unix)]
        command.process_group(0);
        if remote && !python {
            command.arg("--remote");
        }
        let mut child = OwnedChild(
            command
                .env("HARMOS_ADDRESS", "127.0.0.1:0")
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .map_err(|source| {
                    if python && source.kind() == ErrorKind::NotFound {
                        Error::Invalid("Start Python Plugin: Install uv and Put It on PATH, Then Retry".into())
                    } else {
                        Error::Io(source)
                    }
                })?,
        );
        let output = child.0.stdout.take().ok_or_else(|| Error::Invalid("Start the Plugin with a Readiness Output Pipe".into()))?;
        let mut lines = BufReader::new(ChildStdout::from_std(output)?).lines();
        let line = time::timeout(PLUGINS_TIMEOUT, lines.next_line())
            .await
            .map_err(|_| Error::Invalid("Plugin Did Not Announce Readiness; Check Its Startup Logs".into()))??
            .ok_or_else(|| Error::Invalid("Plugin Exited Before Readiness; Check Its Startup Logs".into()))?;
        let endpoint = announced_endpoint(&line)?;
        let drain = tokio::spawn(async move { while lines.next_line().await.ok().flatten().is_some() {} });
        Ok(Self {
            endpoint,
            child,
            drain,
            remote,
            settings: settings.clone(),
        })
    }
    pub async fn describe(&self) -> Result<InstanceGetResponse> {
        describe_plugin(&self.endpoint, &self.settings).await
    }
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
    pub fn id(&self) -> u32 {
        self.child.0.id()
    }
    pub async fn close(&mut self) -> Result<()> {
        if self.child.0.try_wait()?.is_some() {
            return Ok(());
        }
        let stopped = time::timeout(STOP_GRACE, async {
            let mut client = connect(&self.endpoint, &self.settings).await?;
            client.stop(InstanceStopRequest { reason: "Host Removed the Entry".into() }).await?;
            Ok::<(), Error>(())
        })
        .await;
        let force = self.remote || !matches!(stopped, Ok(Ok(_)));
        if force && self.child.0.try_wait()?.is_none() {
            self.child.kill()?;
        }
        let exited = async {
            loop {
                if self.child.0.try_wait()?.is_some() {
                    return Ok::<(), io::Error>(());
                }
                time::sleep(PROCESS_POLL_INTERVAL).await;
            }
        };
        match time::timeout(STOP_GRACE, exited).await {
            Ok(status) => {
                status?;
            }
            Err(_) => {
                self.child.kill()?;
                self.child.0.wait()?;
            }
        }
        Ok(())
    }
}
impl OwnedChild {
    fn kill(&mut self) -> io::Result<()> {
        #[cfg(unix)]
        {
            // The Child Creates This Group Before Starting uv or Any Sidecar Descendant.
            let group = i32::try_from(self.0.id()).map_err(io::Error::other)?;
            let killed = unsafe { libc::kill(-group, libc::SIGKILL) };
            if killed != 0 {
                let error = io::Error::last_os_error();
                if error.raw_os_error() != Some(libc::ESRCH) {
                    return Err(error);
                }
            }
            Ok(())
        }
        #[cfg(not(unix))]
        self.0.kill()
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        // Reap Before Installation Rollback Removes the Executable, Including on Cancellation
        let _ = self.kill();
        let _ = self.0.wait();
    }
}
impl Drop for PluginProcess {
    fn drop(&mut self) {
        self.drain.abort();
    }
}
/* === Public Functions === */

pub async fn describe_plugin(endpoint: &str, settings: &PlacementSettings) -> Result<InstanceGetResponse> {
    let mut client = connect(endpoint, settings).await?;
    let response = client
        .get(InstanceGetRequest {
            revision: 1,
            host: "dokime".into(),
            host_version: env!("CARGO_PKG_VERSION").into(),
        })
        .await?;
    Ok(response.into_inner())
}

/// The gRPC Plugin Policies: Each Plugin Load Is Bounded by the Plugins Operation Timeout.
pub fn plugin_settings() -> GrpcPluginSettings {
    let mut policy = GrpcPluginPolicy::default();
    policy.load.timeout = PLUGINS_TIMEOUT;
    GrpcPluginSettings { policy }
}

/* === Private Functions === */

fn uv_executable() -> Result<PathBuf> {
    let path = env::var_os("PATH").unwrap_or_default();
    let executable = format!("uv{EXE_SUFFIX}");
    let uv = env::split_paths(&path)
        .map(|directory| directory.join(&executable))
        .find(|path| path.is_file())
        .ok_or_else(|| Error::Invalid("Start Python Plugin: Install uv and Put It on PATH, Then Retry".into()))?;
    Ok(uv.canonicalize()?)
}

fn python_command(project: &Path, uv: &Path) -> Result<Command> {
    let project = project.canonicalize()?;
    if !project.join("pyproject.toml").is_file() || !project.join("plugin.py").is_file() {
        return Err(Error::Invalid("Start Python Plugin: Supply a Folder with pyproject.toml and plugin.py".into()));
    }
    let mut command = crate::process::command(uv);
    command.arg("run").arg("--project").arg(&project).args(["python", "plugin.py"]).current_dir(project);
    Ok(command)
}

fn announced_endpoint(line: &str) -> Result<String> {
    let endpoint = match line.strip_prefix("harmos listening ") {
        Some(address) => format!("http://{address}"),
        None => line.to_owned(),
    };
    let parsed = Endpoint::from_shared(endpoint.clone())
        .map_err(|_| Error::Invalid("Serve the Plugin Instance and Print Its Endpoint URL on Standard Output".into()))?;
    if !matches!(parsed.uri().scheme_str(), Some("http" | "https")) || parsed.uri().authority().is_none() {
        return Err(Error::Invalid("Serve the Plugin Instance and Print Its Endpoint URL on Standard Output".into()));
    }
    Ok(endpoint)
}

async fn connect(endpoint: &str, settings: &PlacementSettings) -> Result<InstanceClient<Channel>> {
    let endpoint = Endpoint::from_shared(endpoint.to_owned())
        .map_err(|error| Error::Invalid(format!("Supply a Valid Plugin URL: {error}")))?
        .connect_timeout(PLUGINS_TIMEOUT);
    let channel = endpoint.connect().await?;
    Ok(InstanceClient::new(channel)
        .max_decoding_message_size(settings.message_bytes.get())
        .max_encoding_message_size(settings.message_bytes.get()))
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use std::{ffi::OsStr, fs, path::Path};

    use super::{announced_endpoint, python_command};

    #[test]
    fn python_project_command_uses_its_own_folder() -> crate::Result<()> {
        let root = tempfile::tempdir()?;
        fs::write(root.path().join("pyproject.toml"), "[project]\nname = 'fixture'\nversion = '0.1.0'\n")?;
        fs::write(root.path().join("plugin.py"), "print('http://127.0.0.1:1234')\n")?;
        let project = root.path().canonicalize()?;
        let uv = Path::new("fixture-uv");
        let command = python_command(root.path(), uv)?;
        assert_eq!(command.get_program(), uv.as_os_str());
        assert_eq!(command.get_current_dir(), Some(project.as_path()));
        let arguments: Vec<_> = command.get_args().collect();
        assert_eq!(
            arguments,
            [OsStr::new("run"), OsStr::new("--project"), project.as_os_str(), OsStr::new("python"), OsStr::new("plugin.py")]
        );
        fs::remove_file(root.path().join("plugin.py"))?;
        assert!(python_command(root.path(), uv).is_err());
        Ok(())
    }

    #[test]
    fn readiness_accepts_native_and_python_announcements() -> crate::Result<()> {
        assert_eq!(announced_endpoint("http://127.0.0.1:1234")?, "http://127.0.0.1:1234");
        assert_eq!(announced_endpoint("harmos listening 127.0.0.1:1234")?, "http://127.0.0.1:1234");
        assert!(announced_endpoint("startup log").is_err());
        assert!(announced_endpoint("file:///tmp/plugin").is_err());
        Ok(())
    }
}
