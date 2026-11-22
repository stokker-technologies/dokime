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

mod process;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    future::Future,
    io::{self, ErrorKind},
    num::{NonZeroU32, NonZeroUsize},
    path::Path,
    pin::Pin,
    process::{Command, Stdio},
    result,
    time::Duration,
};

// Third Party Imports
use harmos_operations::plugins::Delete;
use harmos_protocol::plugin::instance_client::InstanceClient;
use harmos_protocol::plugin::{InstanceGetRequest, InstanceGetResponse, InstanceStopRequest};
use harmos_runtime::bundle::validate_launch;
use harmos_runtime::operations::ProtobufOperation;
use harmos_runtime::plugin::grpc::{GrpcPluginPolicy, GrpcPluginSettings, Launches};
use tokio::time;
use tonic::transport::{Channel, Endpoint};

// Local Imports
use crate::{Error, Result};
use dokime_packaging::project::PluginProject;
use dokime_protocol::host::{PluginEntry, PluginState};
use process::OwnedChild;

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

/// How Many Window Lengths an Interval Demand May Fall Behind Before Skipping Stale Samples.
///
/// Skipped samples leave an explicit gap and publication resumes. Three consecutive lagging wakes
/// fail the demand; a timely wake resets that count.
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

    /// Window Lengths Before Skipping Stale Samples; Three Consecutive Lagging Wakes Fail, Default [`INTERVAL_LAG`].
    pub interval_lag: NonZeroU32,

    pub message_bytes: NonZeroUsize,
}

pub struct PluginProcess {
    endpoint: String,
    child: OwnedChild,
    remote: bool,
    settings: PlacementSettings,
}

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
        let project = if executable.is_dir() {
            Some(PluginProject::read(executable).map_err(|source| Error::Invalid(source.to_string()))?)
        } else {
            None
        };
        let launch = project.as_ref().and_then(|project| project.manifest.scripts.launch.as_deref());
        let mut command = match &project {
            Some(project) => match launch {
                Some(launch) => {
                    project.validate_inputs().map_err(|source| Error::Invalid(source.to_string()))?;
                    Self::command(project, launch)?
                }
                None => {
                    let executable = project.executable().map_err(|source| Error::Invalid(source.to_string()))?;
                    let mut command = crate::process::command(executable);
                    command.current_dir(&project.root);
                    command
                }
            },
            None => crate::process::command(executable),
        };
        // The Library's Launch Credentials Make the Plugin Announce Its Bound Listener.
        let launches = Launches::new();
        let mut launch =
            launches.reserve(command.get_program().to_string_lossy()).await.map_err(|source| Error::Invalid(source.to_string()))?;
        command.envs(environment.iter().cloned());
        command.envs(launch.environment());
        command.env_remove("UV_PROJECT_ENVIRONMENT");
        let program = command.get_program().to_string_lossy().into_owned();
        command.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::inherit());
        let mut child = OwnedChild::spawn(command).map_err(|source| {
            if source.kind() == ErrorKind::NotFound {
                Error::Invalid(format!("Start Plugin: Install {program} and Put It on PATH, or Correct dokime.toml, Then Retry"))
            } else {
                Error::Io(source)
            }
        })?;
        let exited = async {
            loop {
                if let Some(status) = child.0.try_wait()? {
                    return Ok::<_, io::Error>(status);
                }
                time::sleep(PROCESS_POLL_INTERVAL).await;
            }
        };
        let announced = time::timeout(PLUGINS_TIMEOUT, async {
            tokio::select! {
                address = launch.announced() => address.map_err(|source| Error::Invalid(format!("Start Plugin: {source}"))),
                status = exited => Err(Error::Invalid(format!("Start Plugin: It Exited With {}; Check Its Startup Logs", status?))),
            }
        })
        .await;
        launches.shutdown().await;
        let address =
            announced.map_err(|_| Error::Invalid("Start Plugin: It Did Not Announce Its Listener; Check Its Startup Logs".into()))??;
        Ok(Self {
            endpoint: format!("http://{address}"),
            child,
            remote,
            settings: settings.clone(),
        })
    }
    pub fn command(project: &PluginProject, arguments: &[String]) -> Result<Command> {
        validate_launch(arguments).map_err(|source| Error::Invalid(source.to_string()))?;
        let program = &arguments[0];
        let executable = if program.contains(['/', '\\']) {
            project.path(Path::new(program)).map_err(|source| Error::Invalid(source.to_string()))?
        } else {
            program.into()
        };
        let mut command = crate::process::command(executable);
        command.args(&arguments[1..]).current_dir(&project.root).env_remove("UV_PROJECT_ENVIRONMENT");
        Ok(command)
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

/* === Public Functions === */

/// Wait for an Entry to Become Ready Using the Caller's Existing Watch and Error Boundary.
/// Callback Futures Borrow Only the Explicit State and Remain Send Across Read and Watch Awaits.
pub async fn await_plugin_ready<S, E>(
    state: &mut S,
    mut read: impl for<'a> FnMut(&'a mut S) -> Pin<Box<dyn Future<Output = result::Result<Option<PluginEntry>, E>> + Send + 'a>>,
    mut changed: impl for<'a> FnMut(&'a mut S) -> Pin<Box<dyn Future<Output = result::Result<bool, E>> + Send + 'a>>,
    unavailable: impl Fn(PluginEntry) -> Option<E>, closed: impl Fn() -> E,
) -> result::Result<PluginEntry, E> {
    loop {
        if let Some(entry) = read(state).await? {
            match PluginState::try_from(entry.state) {
                Ok(PluginState::Ready) => return Ok(entry),
                Ok(PluginState::Unavailable | PluginState::Offline | PluginState::Stopped) => {
                    if let Some(error) = unavailable(entry) {
                        return Err(error);
                    }
                }
                _ => (),
            }
        }
        if !changed(state).await? {
            return Err(closed());
        }
    }
}

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
    GrpcPluginSettings { policy, ..Default::default() }
}

/* === Private Functions === */

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
    use std::{collections::VecDeque, ffi::OsStr, fs, future};

    use dokime_packaging::project::PluginProject;
    use dokime_protocol::host::{PluginEntry, PluginState};

    use super::{PluginProcess, await_plugin_ready};
    use crate::{Error, Result};

    #[tokio::test]
    async fn entry_readiness_observes_transitions_and_returns_the_ready_entry() {
        let states = [PluginState::Unspecified, PluginState::Connecting, PluginState::Ready];
        let entries = states.map(|state| PluginEntry {
            state: state as i32,
            name: "fixture".into(),
            ..Default::default()
        });
        let expected = entries[2].clone();
        let mut readiness = (VecDeque::from(entries), 0);
        let entry = await_plugin_ready(
            &mut readiness,
            |(entries, _)| Box::pin(future::ready(Ok(entries.pop_front()))),
            |(_, changes)| {
                *changes += 1;
                Box::pin(future::ready(Ok(true)))
            },
            |_| Some("unavailable"),
            || "closed",
        )
        .await
        .unwrap();
        assert_eq!(entry, expected);
        assert_eq!(readiness.1, 2);
    }

    #[tokio::test]
    async fn entry_readiness_refuses_terminal_states_without_waiting() {
        for mut state in [PluginState::Unavailable, PluginState::Offline, PluginState::Stopped] {
            let result = await_plugin_ready(
                &mut state,
                |state| {
                    Box::pin(future::ready(Ok(Some(PluginEntry {
                        state: *state as i32,
                        ..Default::default()
                    }))))
                },
                |_| panic!("Terminal Entries Must Not Wait"),
                |entry| Some(entry.state),
                || -1,
            )
            .await;
            assert_eq!(result, Err(state as i32));
        }
    }

    #[tokio::test]
    async fn entry_readiness_preserves_host_stopped_and_missing_entry_observation() {
        let mut entries = VecDeque::from([
            None,
            Some(PluginEntry {
                state: PluginState::Stopped as i32,
                ..Default::default()
            }),
            Some(PluginEntry {
                state: PluginState::Ready as i32,
                ..Default::default()
            }),
        ]);
        let result = await_plugin_ready(
            &mut entries,
            |entries| Box::pin(future::ready(Ok(entries.pop_front().unwrap()))),
            |_| Box::pin(future::ready(Ok(true))),
            |_| None,
            || "closed",
        )
        .await;
        assert_eq!(result.unwrap().state, PluginState::Ready as i32);
    }

    #[tokio::test]
    async fn entry_readiness_propagates_read_watch_and_closed_errors() {
        let result = await_plugin_ready(
            &mut (),
            |_| Box::pin(future::ready(Err("read"))),
            |_| Box::pin(future::ready(Ok(true))),
            |_| None,
            || "closed",
        )
        .await;
        assert_eq!(result, Err("read"));
        let result = await_plugin_ready(
            &mut (),
            |_| Box::pin(future::ready(Ok(None))),
            |_| Box::pin(future::ready(Err("watch"))),
            |_| None,
            || "closed",
        )
        .await;
        assert_eq!(result, Err("watch"));
        let result = await_plugin_ready(
            &mut (),
            |_| Box::pin(future::ready(Ok(None))),
            |_| Box::pin(future::ready(Ok(false))),
            |_| None,
            || "closed",
        )
        .await;
        assert_eq!(result, Err("closed"));
    }

    #[test]
    fn manifest_command_preserves_arguments_directory_and_environment() -> Result<()> {
        let root = tempfile::tempdir()?;
        fs::write(root.path().join("dokime.toml"), "[plugin]\n[scripts]\nlaunch = ['fixture', 'entry', 'argument with spaces']\n")?;
        let project = PluginProject::read(root.path()).map_err(|source| Error::Invalid(source.to_string()))?;
        let command = PluginProcess::command(&project, project.manifest.scripts.launch.as_deref().unwrap())?;
        assert_eq!(command.get_program(), OsStr::new("fixture"));
        assert_eq!(command.get_current_dir(), Some(project.root.as_path()));
        let arguments: Vec<_> = command.get_args().collect();
        assert_eq!(arguments, [OsStr::new("entry"), OsStr::new("argument with spaces")]);
        assert!(command.get_envs().any(|(name, value)| name == "UV_PROJECT_ENVIRONMENT" && value.is_none()));

        fs::write(root.path().join("dokime.toml"), "[plugin]\n[scripts]\nlaunch = ['./bin/interpreter', 'entry']\n")?;
        let project = PluginProject::read(root.path()).map_err(|source| Error::Invalid(source.to_string()))?;
        let command = PluginProcess::command(&project, project.manifest.scripts.launch.as_deref().unwrap())?;
        assert_eq!(command.get_program(), project.root.join("bin/interpreter").as_os_str());
        Ok(())
    }
}
