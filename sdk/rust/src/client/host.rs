// SPDX-License-Identifier: Apache-2.0
//! Composed Values over the Generated Dokime Client Services.
use crate::v1;
use tonic::{Status, transport::Channel};

pub struct HostClient {
    pub plugins: Plugins,
    pub suites: Suites,
    pub templates: Templates,
    pub cycles: Cycles,
    pub executions: Executions,
    pub prompts: Prompts,
    pub records: Records,
    pub samples: Samples,
    pub reports: Reports,
    transport: harmos::Client,
    pub health: Health,
}

impl HostClient {
    pub fn new(channel: Channel, settings: super::ClientSettings) -> Result<Self, Status> {
        let transport = harmos::Client::new(channel.clone(), settings)?;
        Ok(Self {
            plugins: Plugins(channel.clone()),
            suites: Suites(channel.clone()),
            templates: Templates(channel.clone()),
            cycles: Cycles(channel.clone()),
            executions: Executions(channel.clone()),
            prompts: Prompts(channel.clone()),
            records: Records(channel.clone()),
            samples: Samples(channel.clone()),
            reports: Reports(channel.clone()),
            transport,
            health: Health(channel.clone()),
        })
    }
    /// Release This Client's Connections; Finish Owned Watches Before Closing.
    pub fn close(self) {
        drop(self);
    }
}

impl std::ops::Deref for HostClient {
    type Target = harmos::Client;
    fn deref(&self) -> &Self::Target {
        &self.transport
    }
}

pub struct Plugins(Channel);

pub struct Suites(Channel);

pub struct Templates(Channel);

pub struct Cycles(Channel);

pub struct Executions(Channel);

pub struct Prompts(Channel);

pub struct Records(Channel);

pub struct Samples(Channel);

pub struct Reports(Channel);

pub struct Health(Channel);

impl Plugins {
    pub async fn add(&self, url: impl Into<String>) -> Result<v1::Plugin, Status> {
        let request = v1::AddRemotePlugin { url: url.into() };
        Ok(v1::plugins_client::PluginsClient::new(self.0.clone()).add(request).await?.into_inner())
    }
    pub async fn list(&self) -> Result<v1::PluginEntries, Status> {
        let request = v1::ListPlugins {};
        Ok(v1::plugins_client::PluginsClient::new(self.0.clone()).list(request).await?.into_inner())
    }
    pub async fn reload(&self, request: v1::PluginsReloadRequest) -> Result<v1::ClientEmpty, Status> {
        Ok(v1::plugins_client::PluginsClient::new(self.0.clone()).reload(request).await?.into_inner())
    }
}

impl Suites {
    pub async fn list(&self) -> Result<v1::ListSuitesReply, Status> {
        let request = v1::CatalogListSuitesRequest::default();
        Ok(v1::catalog_client::CatalogClient::new(self.0.clone()).list_suites(request).await?.into_inner())
    }
    pub async fn get(&self, identifier: impl Into<String>) -> Result<v1::Suite, Status> {
        let request = v1::CatalogGetSuiteRequest { suite_id: identifier.into() };
        Ok(v1::catalog_client::CatalogClient::new(self.0.clone()).get_suite(request).await?.into_inner())
    }
}

impl Executions {
    pub async fn get(&self, identifier: impl Into<String>) -> Result<v1::ExecutionDetail, Status> {
        let request = v1::ExecutionGetExecutionRequest { execution_id: identifier.into() };
        Ok(v1::execution_client::ExecutionClient::new(self.0.clone()).get_execution(request).await?.into_inner())
    }
    pub async fn list(&self) -> Result<v1::ExecutionPage, Status> {
        let request = v1::QueryListExecutionsRequest::default();
        Ok(v1::query_client::QueryClient::new(self.0.clone()).list_executions(request).await?.into_inner())
    }
}

impl Prompts {
    pub async fn pending(&self, execution: impl Into<String>) -> Result<v1::ListPendingPromptsReply, Status> {
        let request = v1::PromptsListPendingRequest {
            execution_id: Some(execution.into()),
            ..Default::default()
        };
        Ok(v1::prompts_client::PromptsClient::new(self.0.clone()).list_pending(request).await?.into_inner())
    }
    pub async fn get(&self, identifier: impl Into<String>) -> Result<v1::PromptView, Status> {
        let request = v1::PromptsGetPromptRequest { prompt_id: identifier.into() };
        Ok(v1::prompts_client::PromptsClient::new(self.0.clone()).get_prompt(request).await?.into_inner())
    }
    pub async fn respond(
        &self, identifier: impl Into<String>, action: impl Into<String>, accepted: bool,
    ) -> Result<v1::PromptResponseReceipt, Status> {
        let request = v1::PromptsRespondRequest {
            prompt_id: identifier.into(),
            action: action.into(),
            accepted,
            ..Default::default()
        };
        Ok(v1::prompts_client::PromptsClient::new(self.0.clone()).respond(request).await?.into_inner())
    }
}

impl Records {
    pub async fn list(&self, execution: impl Into<String>) -> Result<v1::RecordPage, Status> {
        let request = v1::EvidenceListRecordsRequest {
            selection: Some(v1::CapturedRecordSelection {
                scope: Some(v1::EvidenceScope {
                    execution_id: Some(execution.into()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            ..Default::default()
        };
        Ok(v1::evidence_client::EvidenceClient::new(self.0.clone()).list_records(request).await?.into_inner())
    }
}

impl Samples {
    pub async fn list(&self, execution: impl Into<String>) -> Result<v1::TelemetrySamplesReply, Status> {
        let request = v1::TelemetryListSamplesRequest {
            scope: Some(v1::EvidenceScope {
                execution_id: Some(execution.into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        Ok(v1::telemetry_client::TelemetryClient::new(self.0.clone()).list_samples(request).await?.into_inner())
    }
}

impl Health {
    pub async fn list(&self) -> Result<v1::ListHealthResponse, Status> {
        let request = v1::HealthListHealthRequest::default();
        Ok(v1::health_client::HealthClient::new(self.0.clone()).list_health(request).await?.into_inner())
    }
}

impl Templates {
    pub async fn materialize(
        &self, suite: impl Into<String>, template: impl Into<String>, parameters: std::collections::HashMap<String, Vec<v1::Value>>,
    ) -> Result<Vec<v1::PlannedCase>, Status> {
        let request = v1::PlanningMaterializeTemplateRequest {
            selection: Some(v1::RegisteredSelection {
                suite_id: suite.into(),
                template_id: template.into(),
                parameters: parameters
                    .into_iter()
                    .map(|(key, values)| v1::RegisteredSelectionParametersEntry {
                        key,
                        value: Some(v1::SelectionValues { values }),
                    })
                    .collect(),
                ..Default::default()
            }),
        };
        Ok(v1::planning_client::PlanningClient::new(self.0.clone()).materialize_template(request).await?.into_inner().cases)
    }
}

pub struct Cycle {
    channel: Channel,
    pub identifier: String,
    pub executions: Vec<String>,
}
impl Cycles {
    pub async fn start(&self, cases: Vec<v1::PlannedCase>) -> Result<Cycle, Status> {
        let origins = cases
            .into_iter()
            .map(|case| case.origin.ok_or_else(|| Status::invalid_argument("Start Cycle: Case Has No Frozen Origin")))
            .collect::<Result<Vec<_>, _>>()?;
        let request = v1::ExecutionCreateCycleRequest {
            origins,
            continue_automatically: true,
            ..Default::default()
        };
        let value = v1::execution_client::ExecutionClient::new(self.0.clone()).create_cycle(request).await?.into_inner();
        Ok(Cycle {
            channel: self.0.clone(),
            identifier: value.cycle_id,
            executions: value.execution_ids,
        })
    }
}
pub struct Progress {
    channel: Channel,
    pending: std::collections::VecDeque<String>,
    stream: Option<tonic::Streaming<v1::ObservationWatchExecutionProgressOutput>>,
    terminal: bool,
}
impl Cycle {
    pub fn watch(&self) -> Progress {
        Progress {
            channel: self.channel.clone(),
            pending: self.executions.clone().into(),
            stream: None,
            terminal: false,
        }
    }
}
impl Progress {
    pub async fn next(&mut self) -> Result<Option<v1::ExecutionProgress>, Status> {
        loop {
            if self.stream.is_none() {
                let Some(identifier) = self.pending.pop_front() else {
                    return Ok(None);
                };
                let request = v1::ObservationWatchExecutionProgressRequest { test_execution_id: identifier };
                self.stream = Some(
                    v1::observation_client::ObservationClient::new(self.channel.clone())
                        .watch_execution_progress(request)
                        .await?
                        .into_inner(),
                );
                self.terminal = false;
            }
            let Some(frame) = self.stream.as_mut().unwrap().message().await? else {
                if !self.terminal {
                    return Err(Status::data_loss("Watch Cycle: Missing Terminal Response"));
                }
                self.stream = None;
                continue;
            };
            match frame.value {
                Some(v1::observation_watch_execution_progress_output::Value::Item(frame)) if !self.terminal => {
                    return frame.progress.map(Some).ok_or_else(|| Status::data_loss("Watch Cycle: Missing Progress"));
                }
                Some(v1::observation_watch_execution_progress_output::Value::Response(_)) if !self.terminal => self.terminal = true,
                _ => return Err(Status::data_loss("Watch Cycle: Invalid Terminal Response")),
            }
        }
    }
}
impl Reports {
    pub async fn download(
        &self, execution: impl Into<String>,
    ) -> Result<tonic::Streaming<v1::ReportsDownloadExecutionReportOutput>, Status> {
        let request = v1::ReportsDownloadExecutionReportRequest { execution_id: execution.into() };
        Ok(v1::reports_client::ReportsClient::new(self.0.clone()).download_execution_report(request).await?.into_inner())
    }
}
