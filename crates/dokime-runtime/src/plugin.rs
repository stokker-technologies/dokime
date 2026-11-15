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
use std::{num::NonZeroUsize, path::Path, process::Stdio, time::Duration};

// Third Party Imports
use harmos_protocol::plugin::instance_client::InstanceClient;
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

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone)]
pub struct PlacementSettings {
    pub connect_timeout: Duration,
    pub reconnect_delay: Duration,
    pub stop_grace: Duration,
    pub stream_window: NonZeroUsize,
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
        let line = tokio::time::timeout(settings.connect_timeout, lines.next_line())
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
    pub async fn close(&mut self) -> Result<()> {
        let stopped = tokio::time::timeout(self.settings.stop_grace, async {
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
        match tokio::time::timeout(self.settings.stop_grace, exited).await {
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

/* === Private Functions === */

async fn connect(endpoint: &str, settings: &PlacementSettings) -> Result<InstanceClient<Channel>> {
    let endpoint = tonic::transport::Endpoint::from_shared(endpoint.to_owned())
        .map_err(|error| Error::Invalid(format!("Supply a Valid Plugin URL: {error}")))?
        .connect_timeout(settings.connect_timeout);
    let channel = endpoint.connect().await?;
    Ok(InstanceClient::new(channel)
        .max_decoding_message_size(settings.message_bytes.get())
        .max_encoding_message_size(settings.message_bytes.get()))
}

/* === Tests === */
