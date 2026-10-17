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
use std::{
    collections::{BTreeMap, HashSet},
    time::Duration,
};

// Third Party Imports

// Local Imports
use super::{
    Client, Error, ExecutionRequestHandle, ResponseStream, Result, TestExecutionHandle, contracts,
    validate_observed_cursor, validate_observed_epoch,
};
use dokime_domain::model::identity::CaseId;
use dokime_domain::model::identity::CommandId;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::operations::Endpoint;
use dokime_domain::operations::common::Metadata;
use dokime_domain::operations::execution as contract;
use dokime_domain::operations::planning;
use dokime_domain::operations::value::Value;

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
    /// Creates an empty cycle, then appends one case planned from a suite
    /// template named by its readable ID. Not idempotent: a retry after an
    /// ambiguous failure may leave an extra cycle.
    pub async fn plan_cycle(
        &self,
        suite: &str,
        template: &str,
        parameters: BTreeMap<String, dokime_domain::model::values::Value>,
        label: Option<String>,
    ) -> Result<TestCycleHandle> {
        let definition = self.suite(suite).await?;
        let (plan, default_label) = single_plan(&definition, template, parameters)?;
        let cycle = self
            .create_cycle(contract::CreateCycleRequest {
                metadata: Some(Metadata {
                    identifier: String::new(),
                    label: label.unwrap_or(default_label),
                    description: None,
                    tags: Vec::new(),
                    attributes: Vec::new(),
                    created_at: None,
                    updated_at: None,
                }),
                continue_on_failure: false,
                context: None,
            })
            .await?;
        cycle.append_cases(plan).await?;
        Ok(cycle)
    }

    pub fn cycle(&self, id: TestCycleId) -> TestCycleHandle {
        TestCycleHandle {
            client: self.clone(),
            id,
        }
    }

    pub async fn preview_selection(
        &self,
        request: planning::PreviewSelectionRequest,
    ) -> Result<planning::PreviewSelectionReply> {
        self.verify(true).await?;
        let reply = self
            .unary::<planning::PreviewSelection>(planning::PreviewSelection::PATH, request)
            .await?;
        validate_observed_epoch(
            &self.backend().epoch,
            reply
                .observed_cursor
                .as_ref()
                .map(|cursor| cursor.epoch.as_str()),
        )?;
        super::transport::value(reply)
    }

    pub async fn register_selection(
        &self,
        request: planning::RegisterSelectionRequest,
    ) -> Result<planning::PreviewSelectionReply> {
        dokime_domain::model::execution::command::validate_idempotency_key(&request.idempotency_key)
            .map_err(|error| Error::Rpc(tonic::Status::invalid_argument(error.to_string())))?;
        self.verify(true).await?;
        let reply = self
            .unary::<planning::RegisterSelection>(planning::RegisterSelection::PATH, request)
            .await?;
        validate_observed_epoch(
            &self.backend().epoch,
            reply
                .observed_cursor
                .as_ref()
                .map(|cursor| cursor.epoch.as_str()),
        )?;
        for case in &reply.cases {
            CaseId::from_wire(&case.case_id)
                .map_err(|_| invalid("registered case has an invalid identity"))?;
        }
        super::transport::value(reply)
    }

    pub async fn create_cycle(
        &self,
        request: contract::CreateCycleRequest,
    ) -> Result<TestCycleHandle> {
        self.verify(true).await?;
        let reply = self
            .unary::<contract::CreateCycle>(contract::CreateCycle::PATH, request)
            .await?;
        validate_observed_epoch(
            &self.backend().epoch,
            reply
                .observed_cursor
                .as_ref()
                .map(|cursor| cursor.epoch.as_str()),
        )?;
        let id = TestCycleId::from_uuid(&reply.cycle_id)
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
    ) -> Result<ResponseStream<contracts::telemetry::TelemetrySamplesReply>> {
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
                let scope = contracts::common::EvidenceScope {
                    cycle_id: Some(handle.id.to_string()),
                    execution_id: execution_id.map(|id| id.to_string()),
                    session_id: None,
                    case_id: None,
                    step_id: None,
                    criterion_id: None,
                    template_id: None,
                    suite_id: None,
                    origin: None,
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
    ) -> Result<ResponseStream<contracts::evidence::RecordPage>> {
        let until = until
            .map(|request| request.bind_cycle(&self.client, &self.id))
            .transpose()?;
        let handle = self.clone();
        ResponseStream::spawn(&self.client, move |sender| async move {
            let result = async {
                handle.snapshot().await?;
                let completion = until.map(|request| async move { request.wait_evidence().await });
                let scope = contracts::common::EvidenceScope {
                    cycle_id: Some(handle.id.to_string()),
                    execution_id: execution_id.map(|id| id.to_string()),
                    session_id: None,
                    case_id: None,
                    step_id: None,
                    criterion_id: None,
                    template_id: None,
                    suite_id: None,
                    origin: None,
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

    pub async fn stop(&self) -> Result<dokime_domain::operations::execution::ControlReply> {
        use dokime_domain::operations::Endpoint;
        use dokime_domain::operations::execution::CycleControlRequest;
        use dokime_domain::operations::execution::StopCycle;
        self.client.verify(true).await?;
        let reply = self
            .client
            .unary::<StopCycle>(
                StopCycle::PATH,
                CycleControlRequest {
                    cycle_id: self.id.to_string(),
                    context: None,
                },
            )
            .await?;
        validate_observed_epoch(
            &self.client.backend().epoch,
            reply
                .observed_cursor
                .as_ref()
                .map(|cursor| cursor.epoch.as_str()),
        )?;
        super::transport::value(reply)
    }

    pub async fn run(&self, key: &str) -> Result<contracts::execution::Admission> {
        dokime_domain::model::execution::command::validate_idempotency_key(key)
            .map_err(|error| Error::Rpc(tonic::Status::invalid_argument(error.to_string())))?;
        self.client.verify(true).await?;
        let reply = self
            .client
            .execution_run_cycle(contracts::execution::RunCycleRequest {
                cycle_id: self.id.to_string(),
                idempotency_key: Some(key.to_owned()),
                context: None,
            })
            .await?;
        super::execution::admission(&self.client.backend().epoch, reply)
    }

    /// Append the case's next attempt as a new execution without running it.
    ///
    /// Creation is not idempotent: every accepted call appends another attempt.
    /// Earlier attempts and their evidence are never reused or changed; admit
    /// the new one with [`Self::run_case`].
    pub async fn schedule_rerun(
        &self,
        case_id: &str,
    ) -> Result<contracts::execution::ControlReply> {
        self.client.verify(true).await?;
        let reply = self
            .client
            .execution_schedule_case_rerun(contracts::execution::CaseControlRequest {
                cycle_id: self.id.to_string(),
                case_id: case_id.to_owned(),
                context: None,
            })
            .await?;
        validate_observed_epoch(
            &self.client.backend().epoch,
            reply
                .observed_cursor
                .as_ref()
                .map(|cursor| cursor.epoch.as_str()),
        )?;
        Ok(reply)
    }

    /// Admit the case's queued attempt once under a caller-retained key.
    pub async fn run_case(
        &self,
        case_id: &str,
        key: &str,
    ) -> Result<contracts::execution::Admission> {
        dokime_domain::model::execution::command::validate_idempotency_key(key)
            .map_err(|error| Error::Rpc(tonic::Status::invalid_argument(error.to_string())))?;
        self.client.verify(true).await?;
        let reply = self
            .client
            .execution_run_case(contracts::execution::RunCaseRequest {
                cycle_id: self.id.to_string(),
                case_id: case_id.to_owned(),
                context: None,
                idempotency_key: Some(key.to_owned()),
            })
            .await?;
        super::execution::admission(&self.client.backend().epoch, reply)
    }

    pub async fn snapshot(&self) -> Result<contracts::execution::CycleSummary> {
        self.client.verify(true).await?;
        let snapshot = self
            .client
            .execution_get_cycle(contracts::execution::GetCycleRequest {
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
                let mut request = contracts::execution::ListExecutionsRequest {
                    cycle_id: self.id.to_string(),
                    limit: 100,
                    after_execution_id: String::new(),
                    cursor: None,
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

    /// Appends the planned cases to this cycle; template selections continue
    /// the cycle's case numbering.
    pub async fn append_cases(
        &self,
        plan: planning::SelectionPlan,
    ) -> Result<contract::ControlReply> {
        self.client.verify(true).await?;
        let reply = self
            .client
            .unary::<contract::AppendPlannedCases>(
                contract::AppendPlannedCases::PATH,
                contract::AppendPlannedCasesRequest {
                    cycle_id: self.id.to_string(),
                    plan: Some(plan),
                    context: None,
                },
            )
            .await?;
        validate_observed_epoch(
            &self.client.backend().epoch,
            reply
                .observed_cursor
                .as_ref()
                .map(|cursor| cursor.epoch.as_str()),
        )?;
        super::transport::value(reply)
    }
}

/* === Public Functions === */

/* === Private Functions === */
fn single_plan(
    definition: &dokime_domain::operations::catalog::SuiteDefinition,
    template: &str,
    parameters: BTreeMap<String, dokime_domain::model::values::Value>,
) -> Result<(planning::SelectionPlan, String)> {
    let identity = definition
        .identity
        .as_ref()
        .ok_or_else(|| invalid("suite definition has no identity"))?;
    // Templates are referenced by their readable id; the UUID stays internal.
    let selected = super::catalog::registered_template(definition, template)?;
    let plan = planning::SelectionPlan {
        max_cases: 1,
        case_ids: Vec::new(),
        selections: vec![planning::RegisteredSelection {
            suite_id: identity.suite_id.clone(),
            template_id: selected.template_id.clone(),
            parameters: parameters
                .into_iter()
                .map(
                    |(key, value)| planning::RegisteredSelectionParametersEntry {
                        key,
                        value: Some(planning::SelectionValues {
                            values: vec![Value::from(value)],
                        }),
                    },
                )
                .collect(),
            files: Vec::new(),
        }],
        cycle_id: String::new(),
    };
    let label = selected
        .metadata
        .as_ref()
        .map(|metadata| metadata.label.clone())
        .unwrap_or_default();
    Ok((plan, label))
}

fn execution_members(
    epoch: &str,
    request: &contracts::execution::ListExecutionsRequest,
    page: &contracts::execution::ListExecutionsReply,
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

fn validate_snapshot(
    id: &TestCycleId,
    snapshot: &contracts::execution::CycleSummary,
) -> Result<()> {
    if snapshot.cycle_id != id.as_str() {
        return Err(invalid("cycle snapshot identity differs"));
    }
    Ok(())
}

fn invalid(message: &'static str) -> Error {
    Error::Rpc(tonic::Status::data_loss(message))
}

/* === Tests === */
