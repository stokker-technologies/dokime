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
    num::{NonZeroU32, NonZeroUsize},
    path::Path,
    process::Stdio,
    time::Duration,
};

// Third Party Imports
use harmos_operations::plugins::Delete;
use harmos_protocol::plugin::instance_client::InstanceClient;
use harmos_runtime::operations::ProtobufOperation;
use harmos_runtime::plugin::grpc::{GrpcPluginPolicy, GrpcPluginSettings};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::ChildStdout,
    task::JoinHandle,
};
use tonic::transport::Channel;

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

struct OwnedChild(std::process::Child);

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
        let mut command = crate::process::command(executable);
        command.envs(environment.iter().cloned());
        if remote {
            command.arg("--remote");
        }
        let mut child = OwnedChild(
            command.env("HARMOS_ADDRESS", "127.0.0.1:0").stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::inherit()).spawn()?,
        );
        let output = child.0.stdout.take().ok_or_else(|| Error::Invalid("Start the Plugin with a Readiness Output Pipe".into()))?;
        let mut lines = BufReader::new(ChildStdout::from_std(output)?).lines();
        let line = tokio::time::timeout(PLUGINS_TIMEOUT, lines.next_line())
            .await
            .map_err(|_| Error::Invalid("Plugin Did Not Announce Readiness; Check Its Startup Logs".into()))??
            .ok_or_else(|| Error::Invalid("Plugin Exited Before Readiness; Check Its Startup Logs".into()))?;
        let endpoint = format!(
            "http://{}",
            line.strip_prefix("harmos listening ")
                .ok_or_else(|| Error::Invalid("Serve the Plugin Instance and Announce the Bound Endpoint".into()))?
        );
        let drain = tokio::spawn(async move { while lines.next_line().await.ok().flatten().is_some() {} });
        Ok(Self {
            endpoint,
            child,
            drain,
            remote,
            settings: settings.clone(),
        })
    }
    pub async fn describe(&self) -> Result<harmos_protocol::plugin::InstanceGetResponse> {
        describe_plugin(&self.endpoint, &self.settings).await
    }
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
    pub fn id(&self) -> u32 {
        self.child.0.id()
    }
    pub async fn close(&mut self) -> Result<()> {
        let stopped = tokio::time::timeout(STOP_GRACE, async {
            let mut client = connect(&self.endpoint, &self.settings).await?;
            client.stop(harmos_protocol::plugin::InstanceStopRequest { reason: "Host Removed the Entry".into() }).await?;
            Ok::<(), Error>(())
        })
        .await;
        let force = self.remote || !matches!(stopped, Ok(Ok(_)));
        if force && self.child.0.try_wait()?.is_none() {
            self.child.0.kill()?;
        }
        let exited = async {
            loop {
                if self.child.0.try_wait()?.is_some() {
                    return Ok::<(), std::io::Error>(());
                }
                tokio::time::sleep(PROCESS_POLL_INTERVAL).await;
            }
        };
        match tokio::time::timeout(STOP_GRACE, exited).await {
            Ok(status) => {
                status?;
            }
            Err(_) => {
                self.child.0.kill()?;
                self.child.0.wait()?;
            }
        }
        Ok(())
    }
}
impl Drop for OwnedChild {
    fn drop(&mut self) {
        // Reap Before Installation Rollback Removes the Executable, Including on Cancellation
        if matches!(self.0.try_wait(), Ok(Some(_))) {
            return;
        }
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
impl Drop for PluginProcess {
    fn drop(&mut self) {
        self.drain.abort();
    }
}
/* === Public Functions === */

pub async fn describe_plugin(endpoint: &str, settings: &PlacementSettings) -> Result<harmos_protocol::plugin::InstanceGetResponse> {
    let mut client = connect(endpoint, settings).await?;
    let response = client
        .get(harmos_protocol::plugin::InstanceGetRequest {
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

async fn connect(endpoint: &str, settings: &PlacementSettings) -> Result<InstanceClient<Channel>> {
    let endpoint = tonic::transport::Endpoint::from_shared(endpoint.to_owned())
        .map_err(|error| Error::Invalid(format!("Supply a Valid Plugin URL: {error}")))?
        .connect_timeout(PLUGINS_TIMEOUT);
    let channel = endpoint.connect().await?;
    Ok(InstanceClient::new(channel)
        .max_decoding_message_size(settings.message_bytes.get())
        .max_encoding_message_size(settings.message_bytes.get()))
}

/* === Tests === */
