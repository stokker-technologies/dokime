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
use std::collections::HashSet;

// Third Party Imports

// Local Imports
use super::{Client, Error, ResponseStream, Result, TestExecutionHandle, contracts, validate_observed_cursor, validate_observed_epoch};
use dokime_domain::model::identity::CaseId;
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::operations::Endpoint;
use dokime_domain::operations::execution as contract;
use dokime_domain::operations::planning;

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
    pub fn cycle(&self, id: TestCycleId) -> TestCycleHandle {
        TestCycleHandle { client: self.clone(), id }
    }

    pub async fn preview_selection(
        &self, request: planning::PreviewSelection,
    ) -> Result<dokime_domain::model::planning::wire::PreviewSelectionReply> {
        self.verify(true).await?;
        let reply = self.unary::<planning::PreviewSelection>(planning::PreviewSelection::PATH, request).await?;
        validate_observed_epoch(&self.backend().epoch, reply.observed_cursor.as_ref().map(|cursor| cursor.epoch.as_str()))?;
        super::transport::value(reply)
    }

    pub async fn register_selection(
        &self, request: planning::RegisterSelection,
    ) -> Result<dokime_domain::model::planning::wire::PreviewSelectionReply> {
        dokime_domain::model::execution::command::validate_idempotency_key(&request.idempotency_key)
            .map_err(|error| Error::Rpc(tonic::Status::invalid_argument(error.to_string())))?;
        self.verify(true).await?;
        let reply = self.unary::<planning::RegisterSelection>(planning::RegisterSelection::PATH, request).await?;
        validate_observed_epoch(&self.backend().epoch, reply.observed_cursor.as_ref().map(|cursor| cursor.epoch.as_str()))?;
        for case in &reply.cases {
            CaseId::from_wire(&case.case_id).map_err(|_| invalid("registered case has an invalid identity"))?;
        }
        super::transport::value(reply)
    }

    pub async fn materialize_template(
        &self, request: planning::MaterializeTemplate,
    ) -> Result<dokime_domain::model::planning::wire::PreviewSelectionReply> {
        self.verify(true).await?;
        let reply = self.unary::<planning::MaterializeTemplate>(planning::MaterializeTemplate::PATH, request).await?;
        validate_observed_cursor(&self.backend().epoch, reply.observed_cursor.as_ref())?;
        for case in &reply.cases {
            CaseId::from_wire(&case.case_id).map_err(|_| invalid("materialized case has an invalid identity"))?;
            if case.definition_revision.is_empty() || case.provider.is_empty() {
                return Err(invalid("materialized case is missing its definition or provider revision"));
            }
        }
        super::transport::value(reply)
    }

    pub async fn create_cycle(&self, request: contract::CreateCycle) -> Result<TestCycleHandle> {
        self.verify(true).await?;
        let reply = self.unary::<contract::CreateCycle>(contract::CreateCycle::PATH, request).await?;
        validate_observed_epoch(&self.backend().epoch, reply.observed_cursor.as_ref().map(|cursor| cursor.epoch.as_str()))?;
        let id = TestCycleId::from_uuid(&reply.cycle_id).map_err(|_| invalid("created cycle has an invalid identity"))?;
        Ok(self.cycle(id))
    }
}

impl TestCycleHandle {
    pub async fn export(
        &self, destination: std::path::PathBuf, overwrite: bool,
    ) -> std::result::Result<super::Download<dokime_domain::model::exports::EvidenceExportMetadata>, super::DownloadError> {
        let subject = dokime_domain::model::exports::EvidenceSubject::Cycle(self.id.to_string());
        self.client.download_evidence(contracts::content::DownloadEvidence { subject: Some(subject) }, destination, overwrite).await
    }

    pub fn samples(
        &self, streams: Vec<String>, execution_id: Option<TestExecutionId>, page_size: u32,
    ) -> Result<ResponseStream<dokime_domain::model::telemetry::wire::TelemetrySamplesReply>> {
        super::execution::validate_filters(&streams)?;
        super::execution::validate_sample_page(page_size)?;
        let handle = self.clone();
        ResponseStream::spawn(&self.client, move |sender| async move {
            let result = async {
                handle.snapshot().await?;
                let scope = dokime_domain::model::identity::wire::EvidenceScope {
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
                super::execution::read_samples(&handle.client, scope, streams, page_size, false, &sender).await
            }
            .await;
            if let Err(error) = result {
                let _ = sender.send(Err(error)).await;
            }
        })
    }

    pub fn records(
        &self, record_types: Vec<String>, execution_id: Option<TestExecutionId>,
    ) -> Result<ResponseStream<dokime_domain::model::evidence::wire::RecordPage>> {
        super::execution::validate_filters(&record_types)?;
        let handle = self.clone();
        ResponseStream::spawn(&self.client, move |sender| async move {
            let result = async {
                handle.snapshot().await?;
                let completion = None::<std::future::Ready<Result<()>>>;
                let scope = dokime_domain::model::identity::wire::EvidenceScope {
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
                super::execution::read_records(&handle.client, scope, record_types, completion, &sender).await
            }
            .await;
            if let Err(error) = result {
                let _ = sender.send(Err(error)).await;
            }
        })
    }

    pub fn id(&self) -> &TestCycleId {
        &self.id
    }

    pub async fn seal(&self) -> Result<dokime_domain::model::execution::wire::ControlReply> {
        use dokime_domain::operations::execution::SealCycle;
        self.client.verify(true).await?;
        let reply = self
            .client
            .unary::<SealCycle>(
                SealCycle::PATH,
                SealCycle {
                    cycle_id: self.id.to_string(),
                    context: None,
                },
            )
            .await?;
        validate_observed_epoch(&self.client.backend().epoch, reply.observed_cursor.as_ref().map(|cursor| cursor.epoch.as_str()))?;
        super::transport::value(reply)
    }

    pub async fn snapshot(&self) -> Result<dokime_domain::model::execution::wire::CycleSummary> {
        self.client.verify(true).await?;
        let snapshot = self.client.execution_get_cycle(contracts::execution::GetCycle { cycle_id: self.id.to_string() }).await?;
        validate_snapshot(&self.id, &snapshot)?;
        validate_observed_cursor(&self.client.backend().epoch, snapshot.observed_cursor.as_ref())?;
        Ok(snapshot)
    }

    pub async fn executions(&self) -> Result<Vec<TestExecutionHandle>> {
        self.client
            .guard(async {
                self.snapshot().await?;
                let mut request = contracts::execution::ListExecutions {
                    cycle_id: self.id.to_string(),
                    limit: 100,
                    after_execution_id: String::new(),
                    cursor: None,
                };
                let mut handles = Vec::new();
                let mut seen = HashSet::new();
                loop {
                    let page = self.client.execution_list_executions(request.clone()).await?;
                    let ids = execution_members(&self.client.backend().epoch, &request, &page, handles.len() as u64, &mut seen)?;
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

    pub async fn append(
        &self, executions: Vec<contract::ExecutionBinding>,
    ) -> Result<dokime_domain::model::execution::wire::AppendCasesReply> {
        let expected = executions.iter().map(|binding| binding.execution_id.clone()).collect::<Vec<_>>();
        let mut seen = HashSet::new();
        for binding in &executions {
            TestExecutionId::from_uuid(&binding.execution_id)
                .map_err(|error| Error::Rpc(tonic::Status::invalid_argument(error.to_string())))?;
            CaseId::from_wire(&binding.case_id).map_err(|error| Error::Rpc(tonic::Status::invalid_argument(error.to_string())))?;
            if !seen.insert(&binding.execution_id) {
                return Err(Error::Rpc(tonic::Status::invalid_argument("append repeats an execution identity")));
            }
        }
        self.client.verify(true).await?;
        let request = contract::AppendExecutions {
            cycle_id: self.id.to_string(),
            executions,
        };
        let reply = self.client.unary::<contract::AppendExecutions>(contract::AppendExecutions::PATH, request).await?;
        validate_observed_cursor(&self.client.backend().epoch, reply.observed_cursor.as_ref())?;
        if reply.execution_ids != expected {
            return Err(invalid("append reply differs from the registered execution identities or order"));
        }
        super::transport::value(reply)
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn execution_members(
    epoch: &str, request: &contracts::execution::ListExecutions, page: &dokime_domain::model::execution::wire::ListExecutionsReply,
    first_ordinal: u64, seen: &mut HashSet<TestExecutionId>,
) -> Result<Vec<TestExecutionId>> {
    validate_observed_cursor(epoch, page.cursor.as_ref())?;
    if request.cursor.is_some() && request.cursor != page.cursor {
        return Err(invalid("execution membership changed its runtime cursor"));
    }
    if page.executions.len() > request.limit as usize {
        return Err(invalid("execution membership exceeds the requested page limit"));
    }
    if let Some(after) = &page.next_after_execution_id
        && (after.is_empty() || page.executions.last().map(|row| &row.execution_id) != Some(after))
    {
        return Err(invalid("execution membership has an invalid continuation"));
    }
    let mut ids = Vec::with_capacity(page.executions.len());
    for (offset, execution) in page.executions.iter().enumerate() {
        let id =
            TestExecutionId::from_uuid(&execution.execution_id).map_err(|_| invalid("execution membership has an invalid identity"))?;
        CaseId::from_wire(&execution.case_id).map_err(|_| invalid("execution membership has an invalid case identity"))?;
        if first_ordinal.checked_add(offset as u64) != Some(execution.ordinal) || !seen.insert(id.clone()) {
            return Err(invalid("execution membership has repeated identity or invalid order"));
        }
        ids.push(id);
    }
    Ok(ids)
}

fn validate_snapshot(id: &TestCycleId, snapshot: &dokime_domain::model::execution::wire::CycleSummary) -> Result<()> {
    if snapshot.cycle_id != id.as_str() {
        return Err(invalid("cycle snapshot identity differs"));
    }
    Ok(())
}

fn invalid(message: &'static str) -> Error {
    Error::Rpc(tonic::Status::data_loss(message))
}

/* === Tests === */
