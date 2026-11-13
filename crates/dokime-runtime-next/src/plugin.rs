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
use harmos_protocol::{plugin::instance_client::InstanceClient, v1 as h};
use prost::Message;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::ChildStdout,
    task::JoinHandle,
};
use tonic::transport::Channel;

// Local Imports
use crate::{Error, Result};
use dokime_protocol::v1;

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

pub(crate) struct Entry {
    pub client: InstanceClient<Channel>,
    pub description: harmos_protocol::plugin::InstanceGetResponse,
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
impl Entry {
    pub async fn admit(endpoint: &str, process: Option<PluginProcess>, settings: &PlacementSettings) -> Result<Self> {
        let mut client = connect(endpoint, settings).await?;
        let description = client
            .get(harmos_protocol::plugin::InstanceGetRequest {
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
            if !matches!(h::Visibility::try_from(operation.visibility), Ok(h::Visibility::Protected | h::Visibility::Public)) {
                return Err(Error::Invalid(format!(
                    "Admit {}: Declare Protected or Public Visibility; Keep Private Operations Local",
                    operation.id
                )));
            }
            let policy = harmos_protocol::policy::OperationPolicy::try_from(operation.policy.as_ref())
                .map_err(|error| Error::Invalid(format!("Admit {}: {error}", operation.id)))?;
            if policy.retries != harmos_protocol::policy::RetryPolicy::Never && !operation.replay_safe {
                return Err(Error::Invalid(format!("Admit {}: Declare Replay Safety Before Enabling Retries", operation.id)));
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
            client.stop(harmos_protocol::plugin::InstanceStopRequest {
                reason: "Host Removed the Remote Entry".into(),
            }),
        )
        .await
        .map_err(|_| Error::Invalid("Remote Plugin Did Not Stop; Check Its Logs".into()))??;
        Ok(())
    }
}

/* === Public Functions === */

// Materialize a Case Against Its Admitted Catalog
pub(crate) async fn materialize(
    entry: std::sync::Arc<Entry>, template: &str, parameters: std::collections::HashMap<String, v1::Value>, settings: &crate::HostSettings,
) -> Result<v1::Origin> {
    let definition = entry.template(template)?;
    let dispatch = crate::dispatch::Dispatch {
        entry: entry.clone(),
        resources: None,
        cancellation: tokio_util::sync::CancellationToken::new(),
        settings: settings.placement.clone(),
        clock: settings.clock.clone(),
    };
    let test_case: v1::TestCase = dispatch
        .invoke(
            &definition.materialize,
            v1::MaterializeRequest {
                template: template.into(),
                parameters: parameters.clone(),
            },
        )
        .await?;
    if test_case.parameters != parameters {
        return Err(Error::Invalid("Materialize Without Changing the Resolved Parameter Set".into()));
    }
    let catalog = entry.description.catalog.as_ref().ok_or_else(|| Error::Invalid("Reconnect the Plugin Catalog".into()))?;
    for operation in
        test_case.steps.iter().map(|step| &step.operation).chain(test_case.criteria.iter().map(|criterion| &criterion.operation))
    {
        if !catalog.operations.iter().any(|contract| &contract.id == operation && contract.visibility == h::Visibility::Protected as i32) {
            return Err(Error::Invalid(format!("Declare the Materialized Operation {operation} as Private")));
        }
    }
    let run = v1::PluginRun {
        test_case: Some(test_case),
        plugin: entry.description.id.clone(),
        template: template.into(),
        digest: catalog.digest.clone(),
        started_at: entry.description.started_at,
    };
    run.validate().map_err(Error::Invalid)?;
    Ok(v1::Origin {
        kind: Some(v1::origin::Kind::PluginRun(run)),
    })
}

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
