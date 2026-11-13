/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// SPDX-License-Identifier: Apache-2.0
//! Composed Values over the Generated Dokime Client Services.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports
use tonic::{Status, transport::Channel};

// Local Imports
use crate::v1;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

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
    transport: super::Client,
    pub health: Health,
}

pub struct Plugins(Channel);

pub struct Suites(super::Client);

pub struct Templates(super::Client);

pub struct Cycles(super::Client);

pub struct Executions(super::Client);

pub struct Prompts(super::Client);

pub struct Records(super::Client);

pub struct Samples(super::Client);

pub struct Reports(super::Client);

pub struct Health(super::Client);

pub struct Cycle {
    client: super::Client,
    pub identifier: String,
    pub executions: Vec<String>,
}

pub struct Progress {
    client: super::Client,
    pending: std::collections::VecDeque<String>,
    stream: Option<harmos::client::Watching<v1::executions::ProgressWatchResponse>>,
    terminal: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl HostClient {
    pub fn new(channel: Channel, settings: super::ClientSettings) -> Result<Self, Status> {
        let transport = super::Client::from_channel(channel.clone(), settings)?;
        Ok(Self {
            plugins: Plugins(channel.clone()),
            suites: Suites(transport.clone()),
            templates: Templates(transport.clone()),
            cycles: Cycles(transport.clone()),
            executions: Executions(transport.clone()),
            prompts: Prompts(transport.clone()),
            records: Records(transport.clone()),
            samples: Samples(transport.clone()),
            reports: Reports(transport.clone()),
            transport: transport.clone(),
            health: Health(transport.clone()),
        })
    }
    /// Release This Client's Connections; Finish Owned Watches Before Closing.
    pub fn close(self) {
        drop(self);
    }
}

impl std::ops::Deref for HostClient {
    type Target = super::Client;
    fn deref(&self) -> &Self::Target {
        &self.transport
    }
}

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
    pub async fn list(&self) -> Result<v1::catalog::SuitesListResponse, Status> {
        let request = v1::catalog::SuitesListRequest::default();
        Ok(self.0.catalog.suites.list(tonic::Request::new(request)).await?.into_inner())
    }
    pub async fn get(&self, identifier: impl Into<String>) -> Result<v1::catalog::SuitesGetResponse, Status> {
        let request = v1::catalog::SuitesGetRequest { name: identifier.into(), ..Default::default() };
        Ok(self.0.catalog.suites.get(tonic::Request::new(request)).await?.into_inner())
    }
}

impl Executions {
    pub async fn get(&self, identifier: impl Into<String>) -> Result<v1::ExecutionsGetResponse, Status> {
        let request = v1::ExecutionsGetRequest { name: identifier.into(), at: None };
        Ok(self.0.executions.get(tonic::Request::new(request)).await?.into_inner())
    }
    pub async fn list(&self) -> Result<v1::ExecutionsListResponse, Status> {
        let request = v1::ExecutionsListRequest::default();
        Ok(self.0.executions.list(tonic::Request::new(request)).await?.into_inner())
    }
}

impl Prompts {
    pub async fn pending(&self, execution: impl Into<String>) -> Result<v1::PromptsListResponse, Status> {
        let request = v1::PromptsListRequest { execution_id: Some(execution.into()), pending_only: true, ..Default::default() };
        Ok(self.0.prompts.list(tonic::Request::new(request)).await?.into_inner())
    }
    pub async fn get(&self, identifier: impl Into<String>) -> Result<v1::PromptsGetResponse, Status> {
        let request = v1::PromptsGetRequest { name: identifier.into(), at: None };
        Ok(self.0.prompts.get(tonic::Request::new(request)).await?.into_inner())
    }
    pub async fn respond(&self, identifier: impl Into<String>, action: impl Into<String>, accepted: bool) -> Result<v1::prompts::AnswersCreateResponse, Status> {
        let request = v1::prompts::AnswersCreateRequest { parent: identifier.into(), action: action.into(), accepted, request_id: uuid::Uuid::now_v7().to_string(), ..Default::default() };
        Ok(self.0.prompts.answers.create(tonic::Request::new(request)).await?.into_inner())
    }
}

impl Records {
    pub async fn list(&self, execution: impl Into<String>) -> Result<v1::evidence::RecordsListResponse, Status> {
        let request = v1::evidence::RecordsListRequest {
            selection: Some(v1::CapturedRecordSelection { scope: Some(v1::EvidenceScope { execution_id: Some(execution.into()), ..Default::default() }), ..Default::default() }),
            ..Default::default()
        };
        Ok(self.0.evidence.records.list(tonic::Request::new(request)).await?.into_inner())
    }
}

impl Samples {
    pub async fn list(&self, execution: impl Into<String>) -> Result<v1::evidence::SamplesListResponse, Status> {
        let request = v1::evidence::SamplesListRequest { scope: Some(v1::EvidenceScope { execution_id: Some(execution.into()), ..Default::default() }), ..Default::default() };
        Ok(self.0.evidence.samples.list(tonic::Request::new(request)).await?.into_inner())
    }
}

impl Health {
    pub async fn list(&self) -> Result<v1::evidence::HealthEventsListResponse, Status> {
        let request = v1::evidence::HealthEventsListRequest::default();
        Ok(self.0.evidence.health_events.list(tonic::Request::new(request)).await?.into_inner())
    }
}

impl Templates {
    pub async fn materialize(
        &self, suite: impl Into<String>, template: impl Into<String>, parameters: std::collections::HashMap<String, Vec<v1::Value>>,
    ) -> Result<Vec<v1::PlannedCase>, Status> {
        let selection = v1::RegisteredSelection {
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
            };
        let request = v1::CasesCreateRequest {
            plan: Some(v1::SelectionPlan { selections: vec![selection], ..Default::default() }),
            request_id: uuid::Uuid::now_v7().to_string(),
            validate_only: false,
        };
        Ok(self.0.cases.create(tonic::Request::new(request)).await?.into_inner().cases)
    }
}

impl Cycles {
    pub async fn start(&self, cases: Vec<v1::PlannedCase>) -> Result<Cycle, Status> {
        let origins = cases
            .into_iter()
            .map(|case| case.origin.ok_or_else(|| Status::invalid_argument("Start Cycle: Case Has No Frozen Origin")))
            .collect::<Result<Vec<_>, _>>()?;
        let request = v1::CyclesCreateRequest {
            origins,
            request_id: uuid::Uuid::now_v7().to_string(),
            continue_automatically: true,
            ..Default::default()
        };
        let value = self.0.cycles.create(tonic::Request::new(request)).await?.into_inner();
        let cycle = value.cycle.ok_or_else(|| Status::data_loss("Start Cycle: Missing Cycle"))?;
        let mut executions = Vec::new();
        let mut request = v1::ExecutionsListRequest { parent: cycle.cycle_id.clone(), ..Default::default() };
        loop {
            let page = self.0.executions.list(tonic::Request::new(request.clone())).await?.into_inner();
            for execution in page.executions {
                let summary = execution.summary.ok_or_else(|| Status::data_loss("Start Cycle: Missing Execution Summary"))?;
                executions.push(summary.execution_id);
            }
            if page.next_page_token.is_empty() { break; }
            request.at = page.resolved_at;
            request.page = Some(harmos_protocol::v1::PageRequest { page_token: page.next_page_token, ..Default::default() });
        }
        Ok(Cycle { client: self.0.clone(), identifier: cycle.cycle_id, executions })
    }
}

impl Cycle {
    pub fn watch(&self) -> Progress {
        Progress {
            client: self.client.clone(),
            pending: self.executions.clone().into(),
            stream: None,
            terminal: false,
        }
    }
}

impl Progress {
    pub async fn next(&mut self) -> Result<Option<v1::executions::ProgressWatchResponse>, Status> {
        loop {
            if self.stream.is_none() {
                let Some(identifier) = self.pending.pop_front() else {
                    return Ok(None);
                };
                let request = v1::executions::ProgressWatchRequest { name: identifier, at: None };
                self.stream =
                    Some(self.client.executions.progress.watch(tonic::Request::new(request)).await?);
                self.terminal = false;
            }
            let Some(frame) = self.stream.as_mut().unwrap().next().await? else {
                if !self.terminal {
                    return Err(Status::data_loss("Watch Cycle: Missing Finished Progress"));
                }
                self.stream = None;
                continue;
            };
            if self.terminal {
                return Err(Status::data_loss("Watch Cycle: Progress After Settlement"));
            }
            self.terminal = frame.finished;
            return Ok(Some(frame));
        }
    }
}

impl Reports {
    pub async fn get(&self, request: v1::ReportsGetRequest) -> Result<v1::ReportsGetResponse, Status> {
        Ok(self.0.reports.get(tonic::Request::new(request)).await?.into_inner())
    }
    pub async fn create(&self, request: v1::ReportsCreateRequest) -> Result<v1::ReportsCreateResponse, Status> {
        Ok(self.0.reports.create(tonic::Request::new(request)).await?.into_inner())
    }
    pub async fn publish(&self, request: v1::ReportsPublishRequest) -> Result<v1::ReportsPublishResponse, Status> {
        Ok(self.0.reports.publish(tonic::Request::new(request)).await?.into_inner())
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
