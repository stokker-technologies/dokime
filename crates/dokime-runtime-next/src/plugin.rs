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
use std::{collections::HashSet, num::NonZeroUsize, path::Path, process::Stdio, time::Duration};

// Third Party Imports
use harmos_protocol::v1::{self as h, plugin_client::PluginClient};
use prost::Message;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    task::JoinHandle,
};
use tonic::transport::Channel;

// Local Imports
use crate::{Error, Result};
use dokime_protocol::v1;

/* === Constants === */

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
    child: Child,
    drain: JoinHandle<()>,
    remote: bool,
    settings: PlacementSettings,
}

pub(crate) struct Entry {
    pub client: PluginClient<Channel>,
    pub description: h::Description,
    pub suites: v1::Suites,
    pub state: EntryState,
    pub process: tokio::sync::Mutex<Option<PluginProcess>>,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryState {
    Starting,
    Connecting,
    Ready,
    Failed,
    Unavailable,
}

/* === Traits === */

/* === Implementations === */

impl PluginProcess {
    pub async fn start(executable: &Path, remote: bool, settings: &PlacementSettings) -> Result<Self> {
        let mut command = Command::from(crate::process::command(executable));
        if remote {
            command.arg("--remote");
        }
        let mut child = command
            .env("HARMOS_ADDRESS", "127.0.0.1:0")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()?;
        let output = child.stdout.take().ok_or_else(|| Error::Invalid("Start the Plugin with a Readiness Output Pipe".into()))?;
        let mut lines = BufReader::new(output).lines();
        let line = tokio::time::timeout(settings.connect_timeout, lines.next_line())
            .await
            .map_err(|_| Error::Invalid("Plugin Did Not Announce Readiness; Check Its Startup Logs".into()))??
            .ok_or_else(|| Error::Invalid("Plugin Exited Before Readiness; Check Its Startup Logs".into()))?;
        let endpoint = format!(
            "http://{}",
            line.strip_prefix("harmos listening ")
                .ok_or_else(|| Error::Invalid("Serve harmos.v1 and Announce the Bound Endpoint".into()))?
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
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
    pub async fn close(&mut self) -> Result<()> {
        let mut client = connect(&self.endpoint, &self.settings).await?;
        let stopped =
            tokio::time::timeout(self.settings.stop_grace, client.stop(h::StopRequest { reason: "Host Removed the Entry".into() })).await;
        if self.remote || !matches!(stopped, Ok(Ok(_))) {
            self.child.kill().await?;
        }
        match tokio::time::timeout(self.settings.stop_grace, self.child.wait()).await {
            Ok(status) => {
                status?;
            }
            Err(_) => {
                self.child.kill().await?;
                self.child.wait().await?;
            }
        }
        Ok(())
    }
}
impl Drop for PluginProcess {
    fn drop(&mut self) {
        self.drain.abort();
    }
}
impl Entry {
    pub async fn admit(endpoint: &str, process: Option<PluginProcess>, settings: &PlacementSettings) -> Result<Self> {
        let mut client = connect(endpoint, settings).await?;
        let description = client
            .describe(h::DescribeRequest {
                revision: 1,
                host: "dokime".into(),
                host_version: env!("CARGO_PKG_VERSION").into(),
            })
            .await?
            .into_inner();
        if description.id.is_empty() || description.started_at.is_none() || description.revision < 1 {
            return Err(Error::Invalid("Serve a Plugin Identity, Process Start Time and Protocol Revision".into()));
        }
        let catalog = description.catalog.as_ref().ok_or_else(|| Error::Invalid("Reconnect the Plugin with Its Catalog".into()))?;
        if catalog.digest != harmos_protocol::schema::catalog_digest(catalog) {
            return Err(Error::Invalid("Rebuild the Plugin Catalog with a Valid Digest".into()));
        }
        let types = catalog.types.as_ref().ok_or_else(|| Error::Invalid("Reconnect the Plugin with Its Message Descriptors".into()))?;
        let declarations = dokime_operations_next::declarations()?;
        for requirement in &catalog.requires {
            let available =
                declarations.iter().any(|operation| operation.id == requirement.id && operation.fingerprint == requirement.fingerprint);
            if !available {
                return Err(Error::Invalid(format!("Connect a Matching Provider for Required Operation {}", requirement.id)));
            }
        }
        let mut identifiers = HashSet::new();
        for operation in &catalog.operations {
            if !identifiers.insert(&operation.id) {
                return Err(Error::Invalid("Give Every Plugin Operation a Unique Identifier".into()));
            }
            let fingerprint = harmos_protocol::schema::fingerprint(&operation.id, types, &operation.request, &operation.response)
                .map_err(|error| Error::Invalid(error.to_string()))?;
            if fingerprint != operation.fingerprint {
                return Err(Error::Invalid(format!("Rebuild the Contract Fingerprint for {}", operation.id)));
            }
        }
        let section = catalog
            .sections
            .iter()
            .find(|section| section.r#type == "dokime.v1.Suites")
            .ok_or_else(|| Error::Invalid("Publish a dokime.v1.Suites Catalog Section".into()))?;
        let suites = v1::Suites::decode(section.payload.as_slice())?;
        Ok(Self {
            client,
            description,
            suites,
            state: EntryState::Ready,
            process: tokio::sync::Mutex::new(process),
        })
    }
    pub fn template(&self, identifier: &str) -> Result<v1::TestCaseTemplate> {
        self.suites
            .suites
            .iter()
            .flat_map(|suite| &suite.templates)
            .find(|template| template.metadata.as_ref().is_some_and(|metadata| metadata.identifier == identifier))
            .cloned()
            .ok_or_else(|| Error::Invalid(format!("Select a Catalog Template: {identifier}")))
    }
    pub async fn close(&self, settings: &PlacementSettings) -> Result<()> {
        if let Some(mut process) = self.process.lock().await.take() {
            return process.close().await;
        }
        let mut client = self.client.clone();
        tokio::time::timeout(
            settings.stop_grace,
            client.stop(h::StopRequest {
                reason: "Host Removed the Remote Entry".into(),
            }),
        )
        .await
        .map_err(|_| Error::Invalid("Remote Plugin Did Not Stop; Check Its Logs".into()))??;
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

async fn connect(endpoint: &str, settings: &PlacementSettings) -> Result<PluginClient<Channel>> {
    let endpoint = tonic::transport::Endpoint::from_shared(endpoint.to_owned())
        .map_err(|error| Error::Invalid(format!("Supply a Valid Plugin URL: {error}")))?
        .connect_timeout(settings.connect_timeout);
    let channel = endpoint.connect().await?;
    Ok(PluginClient::new(channel)
        .max_decoding_message_size(settings.message_bytes.get())
        .max_encoding_message_size(settings.message_bytes.get()))
}

/* === Tests === */
