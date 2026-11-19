/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bound Client Handles

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

// Third Party Imports
use sha2::{Digest, Sha256};

// Local Imports
use super::mutations::{
    self, ArtifactCreateOptions, CycleCreateOptions, CycleSealOptions, ExecutionCancelOptions, ExecutionCreateOptions,
    ExecutionSealOptions, ExecutionStartOptions, ExecutionStopOptions, NoteCreateOptions,
};
use super::{
    AnnotationOwner, AnnotationOwnerOwner, CaseSummary, CycleFilter, CycleOrder, CyclePhase, CycleSelection, CycleSummary, EvidenceSubject,
    ExecutionBinding, ExecutionDetail, ExecutionMode, ExecutionNoteOwner, ExecutionOutcome, NoteWritable, OperatorNote, PluginEntry,
    PluginState, ProgressWatchResponse, RegisteredSelection, ResultStatus, SelectionPlan, annotation_owner_owner, evidence_subject,
    execution_outcome,
};
use super::{
    CasesCreateOptions, CasesGetOptions, CasesParametersOptions, ContentArtifactsCreateOptions, CyclesCreateOptions, CyclesGetOptions,
    CyclesSealOptions, CyclesWatchOptions, EvidenceArchiveOptions, EvidenceNotesCreateOptions, ExecutionsCancelOptions,
    ExecutionsCreateOptions, ExecutionsGetOptions, ExecutionsProgressWatchOptions, ExecutionsSealOptions, ExecutionsStartOptions,
    ExecutionsStopOptions, FilesGetOptions, FilesReadOptions, SuitesListOptions,
};
use super::{
    CasesListOptions, CyclesListOptions, ExecutionFilter, ExecutionOrder, ExecutionSelection, ExecutionState, ExecutionsListOptions,
};
use super::{PluginsDeleteOptions, PluginsGetOptions, PluginsListOptions};
use super::{Watching, queries::Paging};
use crate::{
    ArtifactMetadata, ArtifactScope, Client, Error, FileReference, Message, Metadata, PageRequest, Result, Termination, Value, authoring,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Default)]
pub(super) struct Lifetime {
    closed: AtomicBool,
}

pub struct Plugins {
    client: Client,
}

#[derive(Clone)]
pub struct RegisteredPlugin {
    client: Client,
    entry: PluginEntry,
    detached: Arc<AtomicBool>,
}

pub struct Cases {
    client: Client,
}
pub struct Cycles {
    client: Client,
}
pub struct Executions {
    client: Client,
}

#[derive(Default)]
pub struct CyclesQuery {
    pub label: String,
    pub phases: Vec<CyclePhase>,
    pub page_size: u32,
}

#[derive(Default)]
pub struct ExecutionsQuery {
    pub cycle: String,
    pub templates: Vec<String>,
    pub states: Vec<ExecutionState>,
    pub page_size: u32,
}

#[derive(Clone)]
pub struct Case {
    client: Client,
    identifier: String,
    label: String,
    suite: String,
    template: String,
    cycle: String,
}

#[derive(Clone)]
pub struct Cycle {
    client: Client,
    identifier: String,
    label: String,
    phase: CyclePhase,
}

#[derive(Clone)]
pub struct Execution {
    pub(super) client: Client,
    identifier: String,
    pub cycle: Cycle,
    pub case: Case,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Outcome {
    pub status: ResultStatus,
    pub attribution: Attribution,
    pub reason: String,
    pub termination: Option<Termination>,
    pub importer: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Progress {
    pub phase: String,
    pub completed: u64,
    pub total: u64,
    pub label: String,
    pub sealed: bool,
}

// Dropping This Value Closes Observation, Never the Execution.
pub struct ProgressStream {
    execution: Execution,
    start: bool,
    watch: Option<Watching<ProgressWatchResponse>>,
    sealed: bool,
}

/* === Enums === */

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attribution {
    NotEvaluated,
    Evaluated,
    Declared,
}

/* === Traits === */

/* === Implementations === */

impl Client {
    pub(super) fn ensure_open(&self) -> Result<()> {
        if self.lifetime.closed.load(Ordering::Acquire) {
            return Err(Error::Invalid("Use Client: Connect a New Client After Close".into()));
        }
        Ok(())
    }
    pub fn plugins(&self) -> Plugins {
        Plugins { client: self.clone() }
    }
    pub fn cases(&self) -> Cases {
        Cases { client: self.clone() }
    }
    pub fn cycles(&self) -> Cycles {
        Cycles { client: self.clone() }
    }
    pub fn executions(&self) -> Executions {
        Executions { client: self.clone() }
    }
    pub async fn cycle(&self, label: impl Into<String>) -> Result<Cycle> {
        self.cycle_with(label, CycleCreateOptions::default()).await
    }
    pub async fn cycle_with(&self, label: impl Into<String>, options: CycleCreateOptions) -> Result<Cycle> {
        self.ensure_open()?;
        let reply = self
            .resources
            .cycles
            .create_with(
                Vec::new(),
                Metadata {
                    label: label.into(),
                    ..Default::default()
                },
                false,
                "",
                CyclesCreateOptions {
                    request_id: Some(mutations::request_id(options.request_id)?),
                    context: self.context.clone(),
                },
            )
            .await?;
        Cycle::bind(self.clone(), reply.cycle.ok_or_else(|| invalid("Create Cycle", "Return a Bound Cycle"))?)
    }
    pub async fn close(&self) -> Result<()> {
        self.lifetime.closed.store(true, Ordering::Release);
        Ok(())
    }
}

impl Plugins {
    pub async fn list(&self) -> Result<Vec<RegisteredPlugin>> {
        self.client.ensure_open()?;
        let values = self
            .client
            .resources
            .plugins
            .list_with(PluginsListOptions {
                context: self.client.context.clone(),
                ..Default::default()
            })
            .await?;
        Ok(values
            .entries
            .into_iter()
            .map(|entry| RegisteredPlugin {
                client: self.client.clone(),
                entry,
                detached: Arc::new(AtomicBool::new(false)),
            })
            .collect())
    }
    pub async fn get(&self, identifier: &str) -> Result<RegisteredPlugin> {
        self.client.ensure_open()?;
        let reply = self
            .client
            .resources
            .plugins
            .get_with(
                identifier,
                PluginsGetOptions {
                    context: self.client.context.clone(),
                    ..Default::default()
                },
            )
            .await?;
        let entry = reply.entry.ok_or_else(|| invalid("Read Plugin", "Return Its Registry Entry"))?;
        if entry.name.is_empty() {
            return Err(invalid("Read Plugin", "Return Its Registry Identity"));
        }
        Ok(RegisteredPlugin {
            client: self.client.clone(),
            entry,
            detached: Arc::new(AtomicBool::new(false)),
        })
    }
}

impl RegisteredPlugin {
    pub fn identifier(&self) -> &str {
        &self.entry.name
    }
    pub fn label(&self) -> &str {
        self.entry
            .description
            .as_ref()
            .map(|description| description.id.as_str())
            .filter(|value| !value.is_empty())
            .unwrap_or(&self.entry.name)
    }
    pub fn version(&self) -> &str {
        self.entry.description.as_ref().map(|description| description.version.as_str()).unwrap_or_default()
    }
    pub fn state(&self) -> Result<PluginState> {
        PluginState::try_from(self.entry.state).map_err(|_| invalid("Read Plugin", "Return a Known Activation State"))
    }
    pub async fn refresh(&mut self) -> Result<()> {
        let value = self.client.plugins().get(self.identifier()).await?;
        self.entry = value.entry;
        Ok(())
    }
    pub async fn unregister(&self) -> Result<()> {
        if self.detached.load(Ordering::Acquire) {
            return Ok(());
        }
        self.client.ensure_open()?;
        let value = match self.client.plugins().get(self.identifier()).await {
            Ok(value) => value,
            Err(Error::Harmos(harmos::Error::Transport(status))) if status.code() == tonic::Code::NotFound => {
                self.detached.store(true, Ordering::Release);
                return Ok(());
            }
            Err(error) => return Err(error),
        };
        self.client
            .resources
            .plugins
            .delete_with(self.identifier(), value.entry.revision, PluginsDeleteOptions { context: self.client.context.clone() })
            .await?;
        self.detached.store(true, Ordering::Release);
        Ok(())
    }
}

impl Cases {
    pub async fn list(&self) -> Result<Vec<Case>> {
        self.client.ensure_open()?;
        let page = self
            .client
            .resources
            .cases
            .list_with(
                "",
                CasesListOptions {
                    context: self.client.context.clone(),
                    ..Default::default()
                },
            )
            .await?;
        page.cases.into_iter().map(|summary| Case::bind(self.client.clone(), summary, String::new())).collect()
    }
    pub async fn materialize<T: 'static>(&self) -> Result<Vec<Case>> {
        self.client.ensure_open()?;
        let identifier = authoring::code_name::<T>(true);
        let suites = self
            .client
            .resources
            .suites
            .list_with(
                "",
                SuitesListOptions {
                    context: self.client.context.clone(),
                    ..Default::default()
                },
            )
            .await?;
        let mut matches = Vec::new();
        for suite in suites.values {
            let suite_id = suite.metadata.ok_or_else(|| invalid("Materialize Cases", "Read Suite Metadata"))?.identifier;
            for template in suite.templates {
                let template_id = template.metadata.ok_or_else(|| invalid("Materialize Cases", "Read Template Metadata"))?.identifier;
                if template_id.rsplit('.').next() == Some(identifier.as_str()) {
                    matches.push((suite_id.clone(), template_id));
                }
            }
        }
        if matches.len() != 1 {
            return Err(invalid("Materialize Cases", "Register Exactly One Plugin Containing the Selected Template Type"));
        }
        let (suite_id, template_id) = matches.remove(0);
        self.materialize_selection(suite_id, template_id).await
    }
    async fn materialize_selection(&self, suite_id: String, template_id: String) -> Result<Vec<Case>> {
        let selection = RegisteredSelection {
            suite_id,
            template_id,
            ..Default::default()
        };
        let reply = self
            .client
            .resources
            .cases
            .create_with(
                SelectionPlan {
                    selections: vec![selection],
                    ..Default::default()
                },
                false,
                CasesCreateOptions {
                    context: self.client.context.clone(),
                    ..Default::default()
                },
            )
            .await?;
        reply
            .cases
            .into_iter()
            .map(|case| {
                if case.case_id.is_empty() {
                    return Err(invalid("Materialize Cases", "Return a Case Identity"));
                }
                Ok(Case {
                    client: self.client.clone(),
                    identifier: case.case_id,
                    label: case.metadata.map(|value| value.label).unwrap_or_default(),
                    suite: case.suite_id,
                    template: case.template_id,
                    cycle: String::new(),
                })
            })
            .collect()
    }
    pub async fn get(&self, identifier: &str) -> Result<Case> {
        self.client.ensure_open()?;
        let reply = self
            .client
            .resources
            .cases
            .get_with(
                identifier,
                "",
                CasesGetOptions {
                    context: self.client.context.clone(),
                    ..Default::default()
                },
            )
            .await?;
        let summary = reply.value.and_then(|value| value.summary).ok_or_else(|| invalid("Read Case", "Return Its Summary"))?;
        Case::bind(self.client.clone(), summary, String::new())
    }
}

impl Case {
    fn bind(client: Client, summary: CaseSummary, cycle: String) -> Result<Self> {
        if summary.case_id.is_empty() {
            return Err(invalid("Read Case", "Return a Case Identity"));
        }
        Ok(Self {
            client,
            identifier: summary.case_id,
            label: summary.metadata.map(|metadata| metadata.label).unwrap_or_default(),
            suite: summary.suite_id,
            template: summary.template_id,
            cycle,
        })
    }
    pub fn identifier(&self) -> &str {
        &self.identifier
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn suite(&self) -> &str {
        &self.suite
    }
    pub fn template(&self) -> &str {
        &self.template
    }
    pub async fn refresh(&mut self) -> Result<()> {
        self.client.ensure_open()?;
        let page = self
            .client
            .resources
            .cases
            .get_with(
                self.identifier(),
                &self.cycle,
                CasesGetOptions {
                    context: self.client.context.clone(),
                    ..Default::default()
                },
            )
            .await?;
        let summary = page.value.and_then(|value| value.summary).ok_or_else(|| invalid("Refresh Case", "Return Its Summary"))?;
        let value = Self::bind(self.client.clone(), summary, self.cycle.clone())?;
        self.label = value.label;
        self.suite = value.suite;
        self.template = value.template;
        Ok(())
    }
    pub async fn parameters<T: Message>(&self) -> Result<T> {
        self.client.ensure_open()?;
        let detail = self
            .client
            .resources
            .cases
            .get_with(
                &self.identifier,
                &self.cycle,
                CasesGetOptions {
                    context: self.client.context.clone(),
                    ..Default::default()
                },
            )
            .await?;
        let reply = self
            .client
            .resources
            .cases
            .parameters_with(
                &self.identifier,
                &self.cycle,
                CasesParametersOptions {
                    at: detail.resolved_at,
                    context: self.client.context.clone(),
                },
            )
            .await?;
        let values = reply
            .parameters
            .into_iter()
            .map(|parameter| {
                let value = parameter.value.ok_or_else(|| invalid("Read Parameters", "Return Each Parameter Value"))?;
                Ok((parameter.name, value))
            })
            .collect::<Result<HashMap<String, Value>>>()?;
        crate::data::decode(&values)
    }
}

impl Cycles {
    pub async fn list(&self) -> Result<Vec<Cycle>> {
        self.list_with(CyclesQuery::default()).await
    }
    pub async fn list_with(&self, query: CyclesQuery) -> Result<Vec<Cycle>> {
        self.client.ensure_open()?;
        let mut paging = Paging::default();
        let mut values = Vec::new();
        loop {
            let page = self
                .client
                .resources
                .cycles
                .list_with(
                    CycleSelection {
                        phases: query.phases.iter().map(|phase| *phase as i32).collect(),
                        ..Default::default()
                    },
                    CycleFilter {
                        text: query.label.clone(),
                        ..Default::default()
                    },
                    CycleOrder::default(),
                    CyclesListOptions {
                        at: paging.at.clone(),
                        page: Some(PageRequest {
                            page_size: query.page_size,
                            page_token: paging.token.clone(),
                            ..Default::default()
                        }),
                        context: self.client.context.clone(),
                    },
                )
                .await?;
            for value in page.cycles {
                values.push(Cycle::bind(self.client.clone(), value)?);
            }
            if !paging.advance(page.resolved_at, page.next_page_token)? {
                break;
            }
        }
        Ok(values)
    }
    pub async fn get(&self, identifier: &str) -> Result<Cycle> {
        self.client.ensure_open()?;
        let reply = self
            .client
            .resources
            .cycles
            .get_with(
                identifier,
                CyclesGetOptions {
                    context: self.client.context.clone(),
                    ..Default::default()
                },
            )
            .await?;
        Cycle::bind(self.client.clone(), reply.cycle.ok_or_else(|| invalid("Read Cycle", "Return Its Summary"))?)
    }
}

impl Cycle {
    fn bind(client: Client, summary: CycleSummary) -> Result<Self> {
        if summary.cycle_id.is_empty() {
            return Err(invalid("Read Cycle", "Return a Cycle Identity"));
        }
        Ok(Self {
            client,
            identifier: summary.cycle_id,
            label: summary.metadata.map(|metadata| metadata.label).unwrap_or_default(),
            phase: CyclePhase::try_from(summary.phase).map_err(|_| invalid("Read Cycle", "Return a Known Phase"))?,
        })
    }
    pub fn identifier(&self) -> &str {
        &self.identifier
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    pub fn phase(&self) -> CyclePhase {
        self.phase
    }
    pub async fn refresh(&mut self) -> Result<()> {
        let value = self.client.cycles().get(self.identifier()).await?;
        self.label = value.label;
        self.phase = value.phase;
        Ok(())
    }
    pub async fn execution(&self, case: &Case) -> Result<Execution> {
        self.execution_with(case, ExecutionCreateOptions::default()).await
    }
    pub async fn execution_with(&self, case: &Case, options: ExecutionCreateOptions) -> Result<Execution> {
        self.client.ensure_open()?;
        if !Arc::ptr_eq(&self.client.lifetime, &case.client.lifetime) {
            return Err(invalid("Create Execution", "Select a Case From This Client"));
        }
        let request_id = mutations::request_id(options.request_id)?;
        let execution_id = mutations::execution_id(&request_id);
        let reply = self
            .client
            .resources
            .executions
            .create_with(
                &self.identifier,
                vec![ExecutionBinding {
                    case_id: case.identifier.clone(),
                    execution_id,
                }],
                ExecutionsCreateOptions {
                    request_id: Some(request_id),
                    context: self.client.context.clone(),
                },
            )
            .await?;
        if reply.executions.len() != 1 {
            return Err(invalid("Create Execution", "Return Exactly One Bound Execution"));
        }
        let execution = Execution::bind(self.client.clone(), reply.executions.into_iter().next().expect("One Execution"))?;
        if execution.cycle.identifier != self.identifier || execution.case.identifier != case.identifier {
            return Err(invalid("Create Execution", "Preserve the Selected Cycle and Case"));
        }
        Ok(Execution {
            cycle: self.clone(),
            case: case.clone(),
            ..execution
        })
    }
    pub async fn seal(&self) -> Result<()> {
        self.seal_with(CycleSealOptions::default()).await
    }
    pub async fn seal_with(&self, options: CycleSealOptions) -> Result<()> {
        self.client.ensure_open()?;
        let reply = self
            .client
            .resources
            .cycles
            .seal_with(
                &self.identifier,
                CyclesSealOptions {
                    request_id: Some(mutations::request_id(options.request_id)?),
                    context: self.client.context.clone(),
                },
            )
            .await?;
        if !reply.seal.is_empty() {
            return Ok(());
        }
        let mut watch = self
            .client
            .resources
            .cycles
            .watch_with(
                CycleSelection {
                    cycle_ids: vec![self.identifier.clone()],
                    ..Default::default()
                },
                CycleFilter::default(),
                CycleOrder::default(),
                CyclesWatchOptions {
                    context: self.client.context.clone(),
                    ..Default::default()
                },
            )
            .await?;
        while let Some(frame) = watch.next().await? {
            if let Some(failure) = frame.resync_required {
                return Err(failure.into());
            }
            for cycle in frame.cycles {
                if cycle.cycle_id != self.identifier {
                    continue;
                }
                match CyclePhase::try_from(cycle.phase) {
                    Ok(CyclePhase::Sealed) => return Ok(()),
                    Ok(CyclePhase::SealFailed) => {
                        return Err(invalid(
                            "Seal Cycle",
                            &format!("Resolve Its Seal Failure and Retry: {}", cycle.seal_failure.unwrap_or_default()),
                        ));
                    }
                    _ => {}
                }
            }
        }
        Err(invalid("Seal Cycle", "Reconnect After Observation Ended Before Seal Settlement"))
    }
    pub async fn artifact(&self, name: &str, data: Vec<u8>, mime: &str) -> Result<ArtifactMetadata> {
        self.artifact_with(name, data, mime, ArtifactCreateOptions::default()).await
    }
    pub async fn artifact_with(&self, name: &str, data: Vec<u8>, mime: &str, options: ArtifactCreateOptions) -> Result<ArtifactMetadata> {
        artifact(
            &self.client,
            ArtifactScope {
                cycle_id: Some(self.identifier.clone()),
                ..Default::default()
            },
            name,
            data,
            mime,
            options,
        )
        .await
    }
    pub async fn evidence(&self) -> Result<Vec<u8>> {
        archive(&self.client, evidence_subject::Kind::Cycle(self.identifier.clone())).await
    }
    pub async fn note(&self, text: impl Into<String>) -> Result<OperatorNote> {
        self.note_with(text, NoteCreateOptions::default()).await
    }
    pub async fn note_with(&self, text: impl Into<String>, options: NoteCreateOptions) -> Result<OperatorNote> {
        note(&self.client, &self.identifier, None, text.into(), options).await
    }
}

impl Executions {
    pub async fn list(&self) -> Result<Vec<Execution>> {
        self.list_with(ExecutionsQuery::default()).await
    }
    pub async fn list_with(&self, query: ExecutionsQuery) -> Result<Vec<Execution>> {
        self.client.ensure_open()?;
        let mut paging = Paging::default();
        let mut values = Vec::new();
        loop {
            let page = self
                .client
                .resources
                .executions
                .list_with(
                    &query.cycle,
                    ExecutionSelection {
                        template_ids: query.templates.clone(),
                        statuses: query.states.iter().map(|state| *state as i32).collect(),
                        ..Default::default()
                    },
                    ExecutionFilter::default(),
                    ExecutionOrder::default(),
                    ExecutionsListOptions {
                        at: paging.at.clone(),
                        page: Some(PageRequest {
                            page_size: query.page_size,
                            page_token: paging.token.clone(),
                            ..Default::default()
                        }),
                        context: self.client.context.clone(),
                    },
                )
                .await?;
            for value in page.executions {
                values.push(Execution::bind(self.client.clone(), value)?);
            }
            if !paging.advance(page.resolved_at, page.next_page_token)? {
                break;
            }
        }
        Ok(values)
    }
    pub async fn get(&self, identifier: &str) -> Result<Execution> {
        self.client.ensure_open()?;
        let reply = self
            .client
            .resources
            .executions
            .get_with(
                identifier,
                ExecutionsGetOptions {
                    context: self.client.context.clone(),
                    ..Default::default()
                },
            )
            .await?;
        Execution::bind(self.client.clone(), reply.execution.ok_or_else(|| invalid("Read Execution", "Return Its Detail"))?)
    }
}

impl Execution {
    fn bind(client: Client, detail: ExecutionDetail) -> Result<Self> {
        let summary = detail.summary.ok_or_else(|| invalid("Read Execution", "Return Its Summary"))?;
        if summary.execution_id.is_empty() || detail.cycle_id.is_empty() || summary.case_id.is_empty() {
            return Err(invalid("Read Execution", "Return Its Complete Execution, Cycle and Case Binding"));
        }
        let cycle = Cycle {
            client: client.clone(),
            identifier: detail.cycle_id.clone(),
            label: String::new(),
            phase: CyclePhase::Unspecified,
        };
        let case = Case {
            client: client.clone(),
            identifier: summary.case_id,
            label: String::new(),
            suite: detail.admitted_suite.map(|metadata| metadata.identifier).unwrap_or_default(),
            template: detail.admitted_template.map(|metadata| metadata.identifier).unwrap_or_default(),
            cycle: detail.cycle_id,
        };
        Ok(Self {
            client,
            identifier: summary.execution_id,
            cycle,
            case,
        })
    }
    pub fn identifier(&self) -> &str {
        &self.identifier
    }
    pub async fn refresh(&self) -> Result<()> {
        let value = self.client.executions().get(self.identifier()).await?;
        if value.cycle.identifier != self.cycle.identifier || value.case.identifier != self.case.identifier {
            return Err(invalid("Refresh Execution", "Preserve Its Cycle and Case"));
        }
        Ok(())
    }
    fn observe(&self, detail: Option<ExecutionDetail>) -> Result<()> {
        let observed = Self::bind(self.client.clone(), detail.ok_or_else(|| invalid("Observe Execution", "Return Its Detail"))?)?;
        if self.identifier != observed.identifier
            || self.cycle.identifier != observed.cycle.identifier
            || self.case.identifier != observed.case.identifier
        {
            return Err(invalid("Observe Execution", "Preserve Its Identity, Cycle and Case"));
        }
        Ok(())
    }
    pub async fn start(&self) -> Result<()> {
        self.start_with(ExecutionStartOptions::default()).await
    }
    pub async fn start_with(&self, options: ExecutionStartOptions) -> Result<()> {
        self.client.ensure_open()?;
        let reply = self
            .client
            .resources
            .executions
            .start_with(
                &self.identifier,
                ExecutionMode::Automatic,
                ExecutionsStartOptions {
                    request_id: Some(mutations::request_id(options.request_id)?),
                    context: self.client.context.clone(),
                },
            )
            .await?;
        self.observe(reply.execution)
    }
    pub fn watch(&self) -> ProgressStream {
        ProgressStream {
            execution: self.clone(),
            start: false,
            watch: None,
            sealed: false,
        }
    }
    pub fn run(&self) -> ProgressStream {
        ProgressStream {
            execution: self.clone(),
            start: true,
            watch: None,
            sealed: false,
        }
    }
    pub async fn stop(&self) -> Result<()> {
        self.stop_with(ExecutionStopOptions::default()).await
    }
    pub async fn stop_with(&self, options: ExecutionStopOptions) -> Result<()> {
        self.client.ensure_open()?;
        let reply = self
            .client
            .resources
            .executions
            .stop_with(
                &self.identifier,
                ExecutionsStopOptions {
                    request_id: Some(mutations::request_id(options.request_id)?),
                    context: self.client.context.clone(),
                },
            )
            .await?;
        self.observe(reply.execution)
    }
    pub async fn cancel(&self) -> Result<()> {
        self.cancel_with(ExecutionCancelOptions::default()).await
    }
    pub async fn cancel_with(&self, options: ExecutionCancelOptions) -> Result<()> {
        self.client.ensure_open()?;
        let reply = self
            .client
            .resources
            .executions
            .cancel_with(
                &self.identifier,
                ExecutionsCancelOptions {
                    request_id: Some(mutations::request_id(options.request_id)?),
                    context: self.client.context.clone(),
                },
            )
            .await?;
        self.observe(reply.execution)
    }
    pub async fn seal(&self) -> Result<Outcome> {
        self.seal_with(ExecutionSealOptions::default()).await
    }
    pub async fn seal_with(&self, options: ExecutionSealOptions) -> Result<Outcome> {
        self.client.ensure_open()?;
        let reply = self
            .client
            .resources
            .executions
            .seal_with(
                &self.identifier,
                ExecutionsSealOptions {
                    request_id: Some(mutations::request_id(options.request_id)?),
                    context: self.client.context.clone(),
                },
            )
            .await?;
        Outcome::decode(reply.outcome.ok_or_else(|| invalid("Seal Execution", "Return a Stable Outcome"))?)
    }
    pub async fn artifact(&self, name: &str, data: Vec<u8>, mime: &str) -> Result<ArtifactMetadata> {
        self.artifact_with(name, data, mime, ArtifactCreateOptions::default()).await
    }
    pub async fn artifact_with(&self, name: &str, data: Vec<u8>, mime: &str, options: ArtifactCreateOptions) -> Result<ArtifactMetadata> {
        artifact(
            &self.client,
            ArtifactScope {
                execution_id: Some(self.identifier.clone()),
                ..Default::default()
            },
            name,
            data,
            mime,
            options,
        )
        .await
    }
    pub async fn evidence(&self) -> Result<Vec<u8>> {
        archive(&self.client, evidence_subject::Kind::Execution(self.identifier.clone())).await
    }
    pub async fn note(&self, text: impl Into<String>) -> Result<OperatorNote> {
        self.note_with(text, NoteCreateOptions::default()).await
    }
    pub async fn note_with(&self, text: impl Into<String>, options: NoteCreateOptions) -> Result<OperatorNote> {
        note(&self.client, self.cycle.identifier(), Some(self.identifier()), text.into(), options).await
    }
}

impl Outcome {
    fn decode(value: ExecutionOutcome) -> Result<Self> {
        let (status, attribution, reason, termination, importer) = match value.attribution {
            Some(execution_outcome::Attribution::NotEvaluated(value)) => {
                (ResultStatus::Pending, Attribution::NotEvaluated, value.reason, Some(termination(value.termination)?), None)
            }
            Some(execution_outcome::Attribution::Evaluated(value)) => {
                (result_status(value.status)?, Attribution::Evaluated, value.reason, None, None)
            }
            Some(execution_outcome::Attribution::Declared(value)) => {
                (result_status(value.status)?, Attribution::Declared, value.reason, None, Some(value.importer))
            }
            None => return Err(invalid("Seal Execution", "Return a Stable Attributed Outcome")),
        };
        Ok(Self {
            status,
            attribution,
            reason,
            termination,
            importer,
        })
    }
}

impl ProgressStream {
    pub async fn next(&mut self) -> Result<Option<Progress>> {
        if self.sealed {
            return Ok(None);
        }
        self.execution.client.ensure_open()?;
        if self.start {
            self.execution.start().await?;
            self.start = false;
        }
        if self.watch.is_none() {
            let watch = self
                .execution
                .client
                .resources
                .executions
                .progress
                .watch_with(
                    self.execution.identifier(),
                    ExecutionsProgressWatchOptions {
                        context: self.execution.client.context.clone(),
                        ..Default::default()
                    },
                )
                .await?;
            self.watch = Some(watch);
        }
        let frame = self
            .watch
            .as_mut()
            .expect("Open Watch")
            .next()
            .await?
            .ok_or_else(|| invalid("Watch Execution", "Reconnect After Observation Ended Without Sealed State"))?;
        let value = frame.value.unwrap_or_default();
        self.sealed = frame.sealed;
        if self.sealed {
            self.watch = None;
        }
        Ok(Some(Progress {
            phase: frame.phase,
            completed: value.completed,
            total: value.total,
            label: value.label,
            sealed: frame.sealed,
        }))
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn result_status(value: i32) -> Result<ResultStatus> {
    match ResultStatus::try_from(value) {
        Ok(ResultStatus::Unspecified) | Err(_) => Err(invalid("Seal Execution", "Return a Known Result Status")),
        Ok(value) => Ok(value),
    }
}

fn termination(value: i32) -> Result<Termination> {
    match Termination::try_from(value) {
        Ok(Termination::Unspecified) | Err(_) => Err(invalid("Seal Execution", "Return a Known Termination")),
        Ok(value) => Ok(value),
    }
}

fn invalid(action: &str, correction: &str) -> Error {
    Error::Invalid(format!("{action}: {correction}"))
}

async fn note(client: &Client, cycle: &str, execution: Option<&str>, text: String, options: NoteCreateOptions) -> Result<OperatorNote> {
    client.ensure_open()?;
    let owner = match execution {
        Some(execution) => annotation_owner_owner::Kind::Execution(ExecutionNoteOwner {
            cycle_id: cycle.into(),
            execution_id: execution.into(),
        }),
        None => annotation_owner_owner::Kind::CycleId(cycle.into()),
    };
    let reply = client
        .resources
        .evidence
        .notes
        .create_with(
            AnnotationOwner {
                owner: Some(AnnotationOwnerOwner { kind: Some(owner) }),
            },
            NoteWritable { message: text },
            EvidenceNotesCreateOptions {
                request_id: Some(mutations::request_id(options.request_id)?),
                context: client.context.clone(),
            },
        )
        .await?;
    reply.note.ok_or_else(|| invalid("Create Note", "Return Its Acknowledged Value"))
}

async fn artifact(
    client: &Client, scope: ArtifactScope, name: &str, data: Vec<u8>, mime: &str, options: ArtifactCreateOptions,
) -> Result<ArtifactMetadata> {
    client.ensure_open()?;
    let request_id = mutations::request_id(options.request_id)?;
    let file = client.resources.files.upload_bytes(data, client.context.clone(), &request_id).await?;
    let reply = client
        .resources
        .content
        .artifacts
        .create_with(
            file,
            name,
            mime,
            ContentArtifactsCreateOptions {
                request_id: Some(format!("{request_id}/attach")),
                scope: Some(scope),
                context: client.context.clone(),
                ..Default::default()
            },
        )
        .await?;
    reply.artifact.ok_or_else(|| invalid("Publish Artifact", "Return Its Acknowledged Value"))
}

async fn archive(client: &Client, subject: evidence_subject::Kind) -> Result<Vec<u8>> {
    client.ensure_open()?;
    let reply = client
        .resources
        .evidence
        .archive_with(
            EvidenceSubject { kind: Some(subject) },
            EvidenceArchiveOptions {
                context: client.context.clone(),
                ..Default::default()
            },
        )
        .await?;
    download(client, reply.file.ok_or_else(|| invalid("Read Evidence", "Return a Retained Archive"))?).await
}

async fn download(client: &Client, reference: FileReference) -> Result<Vec<u8>> {
    let meta = client
        .resources
        .files
        .get_with(
            reference.clone(),
            FilesGetOptions {
                context: client.context.clone(),
                ..Default::default()
            },
        )
        .await?;
    if meta.file.as_ref() != Some(&reference) || meta.chunk_bytes == 0 || meta.chunk_bytes >= meta.message_bytes {
        return Err(invalid("Read File", "Return Stable Content and Valid Transfer Limits"));
    }
    let mut data = Vec::new();
    let mut digest = Sha256::new();
    while (data.len() as u64) < reference.length {
        let reply = client
            .resources
            .files
            .read_with(
                reference.clone(),
                data.len() as u64,
                meta.chunk_bytes.min(reference.length - data.len() as u64),
                FilesReadOptions {
                    at: meta.resolved_at.clone(),
                    context: client.context.clone(),
                },
            )
            .await?;
        if reply.resolved_at != meta.resolved_at
            || reply.total_length != reference.length
            || reply.sha256 != reference.sha256
            || reply.data.is_empty()
            || reply.data.len() as u64 > meta.chunk_bytes
            || reply.data.len() as u64 > reference.length - data.len() as u64
        {
            return Err(invalid("Read File", "Preserve Its Retained Cut, Length, Digest and Bounded Ranges"));
        }
        digest.update(&reply.data);
        data.extend(reply.data);
    }
    if digest.finalize().as_slice() != reference.sha256 {
        return Err(invalid("Read File", "Return Bytes Matching Their SHA-256"));
    }
    Ok(data)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::{Attribution, Outcome};
    use crate::Termination;
    use crate::client::{DeclaredOutcome, EvaluatedOutcome, ExecutionOutcome, ResultStatus, UnevaluatedOutcome, execution_outcome};

    #[test]
    fn stable_outcomes_preserve_attribution_and_refuse_unknown_values() {
        let evaluated = Outcome::decode(ExecutionOutcome {
            attribution: Some(execution_outcome::Attribution::Evaluated(EvaluatedOutcome {
                status: ResultStatus::Passed as i32,
                reason: "Accepted Evidence".into(),
            })),
        })
        .unwrap();
        assert_eq!(evaluated.status, ResultStatus::Passed);
        assert_eq!(evaluated.attribution, Attribution::Evaluated);
        assert_eq!(evaluated.termination, None);
        let declared = Outcome::decode(ExecutionOutcome {
            attribution: Some(execution_outcome::Attribution::Declared(DeclaredOutcome {
                status: ResultStatus::Failed as i32,
                reason: "External Result".into(),
                importer: "fixture".into(),
            })),
        })
        .unwrap();
        assert_eq!(declared.attribution, Attribution::Declared);
        assert_eq!(declared.importer.as_deref(), Some("fixture"));
        let unfinished = Outcome::decode(ExecutionOutcome {
            attribution: Some(execution_outcome::Attribution::NotEvaluated(UnevaluatedOutcome {
                termination: Termination::Cancelled as i32,
                reason: "Operator Cancelled".into(),
            })),
        })
        .unwrap();
        assert_eq!(unfinished.status, ResultStatus::Pending);
        assert_eq!(unfinished.termination, Some(Termination::Cancelled));
        assert!(Outcome::decode(ExecutionOutcome::default()).is_err());
        for status in [0, -1, 999] {
            assert!(
                Outcome::decode(ExecutionOutcome {
                    attribution: Some(execution_outcome::Attribution::Evaluated(EvaluatedOutcome { status, reason: String::new() }))
                })
                .is_err()
            );
            assert!(
                Outcome::decode(ExecutionOutcome {
                    attribution: Some(execution_outcome::Attribution::NotEvaluated(UnevaluatedOutcome {
                        termination: status,
                        reason: String::new()
                    }))
                })
                .is_err()
            );
        }
    }
}
