/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod prompts;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::HashSet, time::Duration};

// Third Party Imports

// Local Imports
use super::{
    Client, Error, ExecutionRequestHandle, ResponseStream, Result, TestExecutionHandle,
    validate_observed_cursor, wire,
};
use crate::core::{CaseId, CommandId, TemplateId, TestCycleId, TestExecutionId};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone)]
pub struct TestCycleHandle {
    client: Client,
    id: TestCycleId,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Client {
    pub async fn plan_cycle(
        &self,
        suite: &str,
        template: &str,
        parameters: wire::ValueObject,
        label: Option<String>,
    ) -> Result<TestCycleHandle> {
        let definition = self.suite(suite).await?;
        let (plan, default_label) = single_plan(&definition, template, parameters)?;
        let suite_id = plan.selections[0].suite_id.clone();
        let template_id = plan.selections[0].template_id.clone();
        let key = CommandId::new().to_string();
        let registered = self.register_selection(plan, &key).await?;
        if registered.cases.len() != 1 {
            return Err(invalid(
                "single-case registration did not return exactly one case",
            ));
        }
        let registered_case = &registered.cases[0];
        if registered_case.suite_id != suite_id
            || registered_case.template_id != template_id
            || registered_case.ordinal != 0
            || registered_case.selection_index != 0
        {
            return Err(invalid(
                "registered case differs from the selected template",
            ));
        }
        let case = CaseId::from_wire(&registered.cases[0].case_id)
            .map_err(|_| invalid("registered case has an invalid identity"))?;
        self.create_cycle(
            &[case],
            wire::Metadata {
                identifier: format!("client.{key}"),
                label: label.unwrap_or(default_label),
                ..Default::default()
            },
            false,
        )
        .await
    }

    pub fn cycle(&self, id: TestCycleId) -> TestCycleHandle {
        TestCycleHandle {
            client: self.clone(),
            id,
        }
    }

    pub async fn register_selection(
        &self,
        plan: wire::SelectionPlan,
        key: &str,
    ) -> Result<wire::PreviewSelectionReply> {
        if key.trim().is_empty() {
            return Err(Error::Rpc(tonic::Status::invalid_argument(
                "registration key must not be empty",
            )));
        }
        self.verify(true).await?;
        let reply = self
            .planning_register_selection(wire::RegisterSelectionRequest {
                plan: Some(plan),
                idempotency_key: key.to_owned(),
            })
            .await?;
        validate_observed_cursor(&self.backend().epoch, reply.observed_cursor.as_ref())?;
        for case in &reply.cases {
            CaseId::from_wire(&case.case_id)
                .map_err(|_| invalid("registered case has an invalid identity"))?;
        }
        Ok(reply)
    }

    pub async fn create_cycle(
        &self,
        cases: &[CaseId],
        metadata: wire::Metadata,
        continue_on_failure: bool,
    ) -> Result<TestCycleHandle> {
        self.verify(true).await?;
        let reply = self
            .execution_create_cycle(wire::CreateCycleRequest {
                plan: Some(case_plan(cases)),
                metadata: Some(metadata),
                continue_on_failure,
                context: None,
            })
            .await?;
        validate_observed_cursor(&self.backend().epoch, reply.observed_cursor.as_ref())?;
        let id = TestCycleId::from_uuid(reply.cycle_id)
            .map_err(|_| invalid("created cycle has an invalid identity"))?;
        Ok(self.cycle(id))
    }
}

impl TestCycleHandle {
    pub fn samples(
        &self,
        streams: Vec<String>,
        execution_id: Option<TestExecutionId>,
        until: Option<&ExecutionRequestHandle>,
        page_size: u32,
        interval: Option<Duration>,
    ) -> Result<ResponseStream<wire::TelemetrySamplesReply>> {
        let interval = super::execution::sample_interval(page_size, interval)?;
        let until = until
            .map(|request| request.bind_cycle(&self.client, &self.id))
            .transpose()?;
        let handle = self.clone();
        ResponseStream::spawn(&self.client, move |sender| async move {
            let result = async {
                handle.snapshot().await?;
                let completion = || async {
                    match &until {
                        Some(request) => request.evidence_ready().await,
                        None => Ok(true),
                    }
                };
                let scope = wire::EvidenceScope {
                    cycle_id: Some(handle.id.to_string()),
                    execution_id: execution_id.map(|id| id.to_string()),
                    ..Default::default()
                };
                super::execution::read_samples(
                    &handle.client,
                    scope,
                    streams,
                    page_size,
                    interval,
                    completion,
                    &sender,
                )
                .await
            }
            .await;
            if let Err(error) = result {
                let _ = sender.send(Err(error)).await;
            }
        })
    }

    pub fn records(
        &self,
        record_types: Vec<String>,
        execution_id: Option<TestExecutionId>,
        until: Option<&ExecutionRequestHandle>,
    ) -> Result<ResponseStream<wire::RecordPage>> {
        let until = until
            .map(|request| request.bind_cycle(&self.client, &self.id))
            .transpose()?;
        let handle = self.clone();
        ResponseStream::spawn(&self.client, move |sender| async move {
            let result = async {
                handle.snapshot().await?;
                let completion = until.map(|request| async move { request.wait_evidence().await });
                let scope = wire::EvidenceScope {
                    cycle_id: Some(handle.id.to_string()),
                    execution_id: execution_id.map(|id| id.to_string()),
                    ..Default::default()
                };
                super::execution::read_records(
                    &handle.client,
                    scope,
                    record_types,
                    completion,
                    &sender,
                )
                .await
            }
            .await;
            if let Err(error) = result {
                let _ = sender.send(Err(error)).await;
            }
        })
    }

    pub fn request(&self, id: CommandId) -> ExecutionRequestHandle {
        ExecutionRequestHandle::new(self.client.clone(), id, Some(self.id.clone()))
    }

    pub fn id(&self) -> &TestCycleId {
        &self.id
    }

    pub async fn run(&self, key: &str) -> Result<wire::Admission> {
        crate::core::command::validate_idempotency_key(key)
            .map_err(|error| Error::Rpc(tonic::Status::invalid_argument(error.to_string())))?;
        self.client.verify(true).await?;
        let reply = self
            .client
            .execution_run_cycle(wire::RunCycleRequest {
                cycle_id: self.id.to_string(),
                idempotency_key: Some(key.to_owned()),
                context: None,
            })
            .await?;
        super::execution::admission(&self.client.backend().epoch, reply)
    }

    pub async fn snapshot(&self) -> Result<wire::CycleSummary> {
        self.client.verify(true).await?;
        let snapshot = self
            .client
            .execution_get_cycle(wire::GetCycleRequest {
                cycle_id: self.id.to_string(),
            })
            .await?;
        validate_snapshot(&self.id, &snapshot)?;
        validate_observed_cursor(
            &self.client.backend().epoch,
            snapshot.observed_cursor.as_ref(),
        )?;
        Ok(snapshot)
    }

    pub async fn executions(&self) -> Result<Vec<TestExecutionHandle>> {
        self.client
            .guard(async {
                self.snapshot().await?;
                let mut request = wire::ListExecutionsRequest {
                    cycle_id: self.id.to_string(),
                    limit: 100,
                    ..Default::default()
                };
                let mut handles = Vec::new();
                let mut seen = HashSet::new();
                loop {
                    let page = self
                        .client
                        .execution_list_executions(request.clone())
                        .await?;
                    let ids = execution_members(
                        &self.client.backend().epoch,
                        &request,
                        &page,
                        handles.len() as u64,
                        &mut seen,
                    )?;
                    handles.extend(ids.into_iter().map(|id| self.client.execution(id)));
                    match page.next_after_execution_id {
                        None => return Ok(handles),
                        Some(after) => {
                            request.after_execution_id = after;
                            request.cursor = page.cursor;
                        }
                    }
                }
            })
            .await?
    }

    pub async fn append(&self, cases: &[CaseId]) -> Result<wire::ControlReply> {
        self.client.verify(true).await?;
        let reply = self
            .client
            .execution_append_planned_cases(wire::AppendPlannedCasesRequest {
                cycle_id: self.id.to_string(),
                plan: Some(case_plan(cases)),
                context: None,
            })
            .await?;
        validate_observed_cursor(&self.client.backend().epoch, reply.observed_cursor.as_ref())?;
        Ok(reply)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn single_plan(
    definition: &wire::SuiteDefinition,
    template: &str,
    parameters: wire::ValueObject,
) -> Result<(wire::SelectionPlan, String)> {
    let identity = definition
        .identity
        .as_ref()
        .ok_or_else(|| invalid("suite definition has no identity"))?;
    let mut matches = definition.templates.iter().filter(|candidate| {
        candidate.template_id == template
            || candidate
                .metadata
                .as_ref()
                .is_some_and(|metadata| metadata.identifier == template)
    });
    let selected = matches.next().ok_or_else(|| {
        Error::Rpc(tonic::Status::invalid_argument(
            "template must identify exactly one registered template in the suite",
        ))
    })?;
    if matches.next().is_some() {
        return Err(Error::Rpc(tonic::Status::invalid_argument(
            "template must identify exactly one registered template in the suite",
        )));
    }
    TemplateId::from_uuid(&selected.template_id)
        .map_err(|_| invalid("template has an invalid identity"))?;
    let plan = wire::SelectionPlan {
        max_cases: 1,
        selections: vec![wire::RegisteredSelection {
            suite_id: identity.suite_id.clone(),
            template_id: selected.template_id.clone(),
            parameters: parameters
                .fields
                .into_iter()
                .map(|(key, value)| {
                    (
                        key,
                        wire::SelectionValues {
                            values: vec![value],
                        },
                    )
                })
                .collect(),
            ..Default::default()
        }],
        ..Default::default()
    };
    let label = selected
        .metadata
        .as_ref()
        .map(|metadata| metadata.label.clone())
        .unwrap_or_default();
    Ok((plan, label))
}

fn case_plan(cases: &[CaseId]) -> wire::SelectionPlan {
    wire::SelectionPlan {
        case_ids: cases.iter().map(ToString::to_string).collect(),
        ..Default::default()
    }
}

fn execution_members(
    epoch: &str,
    request: &wire::ListExecutionsRequest,
    page: &wire::ListExecutionsReply,
    first_ordinal: u64,
    seen: &mut HashSet<TestExecutionId>,
) -> Result<Vec<TestExecutionId>> {
    validate_observed_cursor(epoch, page.cursor.as_ref())?;
    if request.cursor.is_some() && request.cursor != page.cursor {
        return Err(invalid("execution membership changed its runtime cursor"));
    }
    if page.executions.len() > request.limit as usize {
        return Err(invalid(
            "execution membership exceeds the requested page limit",
        ));
    }
    if let Some(after) = &page.next_after_execution_id
        && (after.is_empty() || page.executions.last().map(|row| &row.execution_id) != Some(after))
    {
        return Err(invalid("execution membership has an invalid continuation"));
    }
    let mut ids = Vec::with_capacity(page.executions.len());
    for (offset, execution) in page.executions.iter().enumerate() {
        let id = TestExecutionId::from_uuid(&execution.execution_id)
            .map_err(|_| invalid("execution membership has an invalid identity"))?;
        CaseId::from_wire(&execution.case_id)
            .map_err(|_| invalid("execution membership has an invalid case identity"))?;
        if first_ordinal.checked_add(offset as u64) != Some(execution.ordinal)
            || !seen.insert(id.clone())
        {
            return Err(invalid(
                "execution membership has repeated identity or invalid order",
            ));
        }
        ids.push(id);
    }
    Ok(ids)
}

fn validate_snapshot(id: &TestCycleId, snapshot: &wire::CycleSummary) -> Result<()> {
    if snapshot.cycle_id != id.as_str() {
        return Err(invalid("cycle snapshot identity differs"));
    }
    Ok(())
}

fn invalid(message: &'static str) -> Error {
    Error::Rpc(tonic::Status::data_loss(message))
}

/* === Tests === */
