/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

mod commands;
mod control;
mod detail;
mod loan;
mod requests;
mod steps;

pub(super) use control::command_context;
pub(crate) use detail::result as result_summary;

use crate::grpc::transcode::Transcode;

use std::sync::Arc;

use dokime::harmos::{Context, RegistrationExt};
use prost::Message;
use tonic::{Code, Status};

use super::value;
use super::value::canonical;
use dokime::definition::TestCase;
use dokime::domain::model::diagnostics::ValidationIssue;
use dokime::domain::model::diagnostics::admission::AdmissionFailure;
use dokime::domain::model::execution::result::CycleStatus;
use dokime::domain::model::execution::result::CycleView;
use dokime::domain::model::execution::result::ExecutionStatus;
use dokime::domain::model::execution::wire_cursor::Cursor;
use dokime::domain::model::identity::TestCycleId;
use dokime::operations::execution as wire;
use dokime::operations::hosting::{Host, Serve, Served};
use dokime::runtime::RuntimeState;
use dokime::runtime::query::{
    CaseProjection, CaseStatus, CycleFilter, CycleOrder, CycleOrderKey, CyclePageError, case_projection, case_tally, cycle_page,
    historical_case_projection, select_cycles,
};
use dokime::{RuntimeHandle, runtime::Error};

pub(super) struct Resources {
    pub(super) runtime: RuntimeHandle,
    pub(super) max_message_bytes: usize,
}

impl Serve<wire::GetCase> for Resources {
    fn serve<'a>(
        &'a self, request: wire::GetCase, _: Context<'a, wire::GetCase>,
    ) -> Served<'a, dokime::domain::model::execution::wire::CaseDetail> {
        Box::pin(async move { detail::get_case(self, &request).map(Into::into) })
    }
}

impl Serve<wire::GetExecution> for Resources {
    fn serve<'a>(
        &'a self, request: wire::GetExecution, _: Context<'a, wire::GetExecution>,
    ) -> Served<'a, dokime::domain::model::execution::wire::ExecutionDetail> {
        Box::pin(async move { detail::get_execution(self, &request).map(Into::into) })
    }
}

impl Serve<wire::ListCaseParameters> for Resources {
    fn serve<'a>(
        &'a self, request: wire::ListCaseParameters, _: Context<'a, wire::ListCaseParameters>,
    ) -> Served<'a, dokime::domain::model::execution::wire::ListCaseParametersReply> {
        Box::pin(async move { detail::list_case_parameters(self, &request).map(Into::into) })
    }
}

impl Serve<wire::ListExecutionParameters> for Resources {
    fn serve<'a>(
        &'a self, request: wire::ListExecutionParameters, _: Context<'a, wire::ListExecutionParameters>,
    ) -> Served<'a, dokime::domain::model::execution::wire::ListMaterializedParametersReply> {
        Box::pin(async move { detail::list_execution_parameters(self, &request).map(Into::into) })
    }
}

impl Serve<wire::ListExecutionSteps> for Resources {
    fn serve<'a>(
        &'a self, request: wire::ListExecutionSteps, _: Context<'a, wire::ListExecutionSteps>,
    ) -> Served<'a, dokime::domain::model::execution::wire::ListExecutionStepsReply> {
        Box::pin(async move { steps::list(self, &request).map(Into::into) })
    }
}

impl Serve<wire::GetCycle> for Resources {
    fn serve<'a>(
        &'a self, request: wire::GetCycle, _: Context<'a, wire::GetCycle>,
    ) -> Served<'a, dokime::domain::model::execution::wire::CycleSummary> {
        Box::pin(async move { self.get_cycle(&request).map(Into::into) })
    }
}

impl Serve<wire::ListCycles> for Resources {
    fn serve<'a>(
        &'a self, request: wire::ListCycles, _: Context<'a, wire::ListCycles>,
    ) -> Served<'a, dokime::domain::model::execution::wire::ListCyclesReply> {
        Box::pin(async move { self.list_cycles(&request).map(Into::into) })
    }
}

impl Serve<wire::ListCases> for Resources {
    fn serve<'a>(
        &'a self, request: wire::ListCases, _: Context<'a, wire::ListCases>,
    ) -> Served<'a, dokime::domain::model::execution::wire::ListCasesReply> {
        Box::pin(async move { self.list_cases(&request).map(Into::into) })
    }
}

impl Serve<wire::ListExecutions> for Resources {
    fn serve<'a>(
        &'a self, request: wire::ListExecutions, _: Context<'a, wire::ListExecutions>,
    ) -> Served<'a, dokime::domain::model::execution::wire::ListExecutionsReply> {
        Box::pin(async move { self.list_executions(&request).map(Into::into) })
    }
}

impl Resources {
    fn get_cycle(&self, request: &wire::GetCycle) -> Result<dokime::domain::model::execution::wire::CycleSummary, Status> {
        let id = cycle_id(&request.cycle_id)?;
        let snapshot = self.runtime.watch_state().latest();
        let cycle = snapshot.state().cycles.get(&id).ok_or_else(|| Status::not_found("cycle not found"))?;
        let value = summary(snapshot.state(), cycle, canonical::cursor(&snapshot))?;
        inspection_reply(self, value)
    }

    fn list_cycles(&self, request: &wire::ListCycles) -> Result<dokime::domain::model::execution::wire::ListCyclesReply, Status> {
        let snapshot = self.runtime.watch_state().latest();
        let cursor = canonical::cursor(&snapshot);
        let limit = cursor.page(request.limit, &request.after_cycle_id, request.cursor.as_ref()).map_err(canonical::cursor_failure)?;
        let filter = cycle_filter(request.filter.as_ref())?;
        let order = cycle_order(request.order.as_ref());
        let selected = select_cycles(snapshot.state(), &filter, order);
        let after = if request.after_cycle_id.is_empty() {
            None
        } else {
            cycle_id(&request.after_cycle_id)?;
            Some(request.after_cycle_id.as_str())
        };
        let (page, next_after_cycle_id) = cycle_page(&selected, after, limit).map_err(|error| match error {
            CyclePageError::UnknownContinuation => Status::invalid_argument(error.to_string()),
        })?;

        let mut cycles = canonical::slots(page.len())?;
        for cycle in page {
            cycles.push(summary(snapshot.state(), cycle, cursor.clone())?);
        }
        inspection_reply(
            self,
            dokime::domain::model::execution::wire::ListCyclesReply {
                cycles,
                next_after_cycle_id,
                cursor: Some(cursor),
            },
        )
    }

    fn list_cases(&self, request: &wire::ListCases) -> Result<dokime::domain::model::execution::wire::ListCasesReply, Status> {
        let id = cycle_id(&request.cycle_id)?;
        let snapshot = self.runtime.watch_state().latest();
        let cursor = canonical::cursor(&snapshot);
        let limit = cursor.page(request.limit, &request.after_case_id, request.cursor.as_ref()).map_err(canonical::cursor_failure)?;
        let cycle = snapshot.state().cycle_view(&id).ok_or_else(|| Status::not_found("cycle not found"))?;
        let value = case_page(snapshot.state(), &cycle, &request.after_case_id, limit, cursor)?;
        inspection_reply(self, value)
    }

    fn list_executions(
        &self, request: &wire::ListExecutions,
    ) -> Result<dokime::domain::model::execution::wire::ListExecutionsReply, Status> {
        let id = cycle_id(&request.cycle_id)?;
        let snapshot = self.runtime.watch_state().latest();
        let cursor = canonical::cursor(&snapshot);
        let limit = cursor.page(request.limit, &request.after_execution_id, request.cursor.as_ref()).map_err(canonical::cursor_failure)?;
        let cycle = snapshot.state().cycles.get(&id).ok_or_else(|| Status::not_found("cycle not found"))?;
        let value =
            execution_page(snapshot.state(), snapshot.state().executions_for_cycle(cycle), &request.after_execution_id, limit, cursor)?;
        inspection_reply(self, value)
    }
}

pub(super) async fn register(runtime: &RuntimeHandle, max_message_bytes: usize) -> Result<(), super::Error> {
    control::register(runtime).await?;
    requests::register(runtime).await?;
    commands::register(runtime, max_message_bytes).await?;
    let served = Arc::new(Resources {
        runtime: runtime.clone(),
        max_message_bytes,
    });
    let mut dependencies = dokime::harmos::Dependencies::new();
    dependencies.insert(Host::<wire::GetCase>::new(served.clone()))?;
    dependencies.insert(Host::<wire::GetExecution>::new(served.clone()))?;
    dependencies.insert(Host::<wire::ListCaseParameters>::new(served.clone()))?;
    dependencies.insert(Host::<wire::ListExecutionParameters>::new(served.clone()))?;
    dependencies.insert(Host::<wire::ListExecutionSteps>::new(served.clone()))?;
    dependencies.insert(Host::<wire::GetCycle>::new(served.clone()))?;
    dependencies.insert(Host::<wire::ListCycles>::new(served.clone()))?;
    dependencies.insert(Host::<wire::ListCases>::new(served.clone()))?;
    dependencies.insert(Host::<wire::ListExecutions>::new(served))?;
    let batch = dokime::harmos::RegistrationBatch::new(dependencies)
        .operation::<wire::GetCase>()
        .operation::<wire::GetExecution>()
        .operation::<wire::ListCaseParameters>()
        .operation::<wire::ListExecutionParameters>()
        .operation::<wire::ListExecutionSteps>()
        .operation::<wire::GetCycle>()
        .operation::<wire::ListCycles>()
        .operation::<wire::ListCases>()
        .operation::<wire::ListExecutions>();
    runtime.operations().register(batch.lane(dokime::app::Lanes::Open)).await?;
    Ok(())
}

pub(super) fn status(error: Error) -> Status {
    let (grpc_code, code, message, issues) = match error {
        Error::Storage(dokime::storage::Error::SettingsConflict { expected, actual }) => {
            (Code::Aborted, "settings_conflict".into(), format!("expected settings revision {expected}, found {actual}"), vec![])
        }
        Error::Storage(dokime::storage::Error::InvalidSettings(message)) => {
            (Code::InvalidArgument, "invalid_settings".into(), message, vec![])
        }
        Error::Admission(AdmissionFailure::Validation { code, message, issues }) => (Code::InvalidArgument, code, message, issues),
        Error::Admission(AdmissionFailure::Conflict { code, message }) => (Code::FailedPrecondition, code, message, vec![]),
        Error::Admission(AdmissionFailure::Overload { code, message }) => (Code::ResourceExhausted, code, message, vec![]),
        Error::Admission(AdmissionFailure::Unavailable { code, message }) => (Code::Unavailable, code, message, vec![]),
        Error::Admission(AdmissionFailure::Forbidden { code, message }) => (Code::PermissionDenied, code, message, vec![]),
        Error::Admission(AdmissionFailure::NotFound { code, message }) => (Code::NotFound, code, message, vec![]),
        Error::Admission(AdmissionFailure::Timeout { code, message }) => (Code::DeadlineExceeded, code, message, vec![]),
        Error::Admission(AdmissionFailure::Cancelled { code, message }) => (Code::Cancelled, code, message, vec![]),
        Error::Admission(AdmissionFailure::Internal { code, message }) => (Code::Internal, code, message, vec![]),
        error => (Code::Internal, "runtime_internal".to_owned(), error.to_string(), vec![]),
    };
    let detail = rejection(code, message, issues);
    let envelope = tonic_types::Status {
        code: grpc_code as i32,
        message: detail.message.clone(),
        // A Detail The Transcoder Refuses Is Omitted; The Status Itself Still Reaches The Caller.
        details: crate::grpc::transcode::Transcode::wire_bytes(&detail)
            .ok()
            .map(|value| prost_types::Any {
                type_url: "type.googleapis.com/dokime.v1.Rejection".into(),
                value,
            })
            .into_iter()
            .collect(),
    };
    Status::with_details(grpc_code, detail.message, envelope.encode_to_vec().into())
}

pub(super) fn summary(
    state: &RuntimeState, cycle: &dokime::domain::model::execution::result::TestCycle, cursor: Cursor,
) -> Result<dokime::domain::model::execution::wire::CycleSummary, Status> {
    let tally = case_tally(state, cycle);
    Ok(dokime::domain::model::execution::wire::CycleSummary {
        cycle_id: cycle.test_cycle_id.to_string(),
        metadata: Some(canonical::metadata(&cycle.metadata)?),
        phase: cycle.phase,
        status: match cycle.lifecycle {
            CycleStatus::Ready => dokime::domain::model::execution::wire::CycleStatus::Ready,
            CycleStatus::Running => dokime::domain::model::execution::wire::CycleStatus::Running,
            CycleStatus::Paused => dokime::domain::model::execution::wire::CycleStatus::Paused,
            CycleStatus::Stopping => dokime::domain::model::execution::wire::CycleStatus::Stopping,
            CycleStatus::Completed => dokime::domain::model::execution::wire::CycleStatus::Completed,
            CycleStatus::Failed => dokime::domain::model::execution::wire::CycleStatus::Failed,
            CycleStatus::Blocked => dokime::domain::model::execution::wire::CycleStatus::Blocked,
            CycleStatus::Stopped => dokime::domain::model::execution::wire::CycleStatus::Stopped,
        },
        archived: state.archived_cycles.contains(&cycle.test_cycle_id),
        sealed: cycle.saved.is_some(),
        case_count: state.cases_for_cycle(cycle).count() as u64,
        execution_count: cycle.execution_ids.len() as u64,
        continue_automatically: cycle.continue_automatically,
        observed_cursor: Some(cursor),
        created_at: Some(canonical::timestamp(&cycle.created_at)?),
        started_at: cycle.started_at.as_ref().map(canonical::timestamp).transpose()?,
        updated_at: Some(canonical::timestamp(&cycle.updated_at)?),
        result: cycle.result.as_ref().map(detail::result),
        running_count: tally.running,
        queued_count: tally.queued,
    })
}

pub(super) fn case_summary(
    case: &TestCase, ordinal: usize, projection: CaseProjection,
) -> Result<dokime::domain::model::execution::wire::CaseSummary, Status> {
    Ok(dokime::domain::model::execution::wire::CaseSummary {
        suite_id: case.suite_id.clone(),
        case_id: case.test_case_id.as_str().to_owned(),
        template_id: case.test_case_template_id.as_str().to_owned(),
        metadata: Some(canonical::metadata(&case.metadata)?),
        ordinal: ordinal as u64,
        status: projection.status.map_or(dokime::domain::model::execution::wire::CaseStatus::Unspecified, case_status),
        latest_execution_id: projection.latest_execution_id.as_ref().map(|id| id.as_str().to_owned()),
        latest_execution_status: projection.latest_execution_status.as_ref().map(execution_status),
        attempt_count: projection.attempt_count,
        actions: Some(dokime::domain::model::execution::wire::CaseActions {
            delete: projection.actions.delete,
            unavailable_reasons: projection.actions.unavailable_reasons,
        }),
    })
}

pub(super) fn historical_case_summary(
    cycle: &CycleView, case: &TestCase, ordinal: usize,
) -> Result<dokime::domain::model::execution::wire::CaseSummary, Status> {
    case_summary(case, ordinal, historical_case_projection(cycle, &case.test_case_id))
}

pub(super) fn execution_status(status: &ExecutionStatus) -> dokime::domain::model::execution::wire::ExecutionStatus {
    match status {
        ExecutionStatus::Invalidated => dokime::domain::model::execution::wire::ExecutionStatus::Invalidated,
        ExecutionStatus::Skipped => dokime::domain::model::execution::wire::ExecutionStatus::Skipped,
        ExecutionStatus::Pending => dokime::domain::model::execution::wire::ExecutionStatus::Pending,
        ExecutionStatus::Running => dokime::domain::model::execution::wire::ExecutionStatus::Running,
        ExecutionStatus::Passed => dokime::domain::model::execution::wire::ExecutionStatus::Passed,
        ExecutionStatus::Failed => dokime::domain::model::execution::wire::ExecutionStatus::Failed,
        ExecutionStatus::Blocked => dokime::domain::model::execution::wire::ExecutionStatus::Blocked,
        ExecutionStatus::Cancelled => dokime::domain::model::execution::wire::ExecutionStatus::Cancelled,
        ExecutionStatus::Interrupted => dokime::domain::model::execution::wire::ExecutionStatus::Interrupted,
    }
}

pub(super) fn archive_scope(
    scope: Option<dokime::domain::model::execution::wire::ArchiveScope>,
) -> dokime::runtime::query::executions::ArchiveScope {
    use dokime::runtime::query::executions::ArchiveScope;
    match scope {
        None | Some(dokime::domain::model::execution::wire::ArchiveScope::Unarchived) => ArchiveScope::Unarchived,
        Some(dokime::domain::model::execution::wire::ArchiveScope::Archived) => ArchiveScope::Archived,
        Some(dokime::domain::model::execution::wire::ArchiveScope::All) => ArchiveScope::All,
    }
}

pub(super) fn execution_summary(
    state: &RuntimeState, execution: &dokime::domain::model::execution::result::TestExecution, ordinal: usize,
) -> dokime::domain::model::execution::wire::ExecutionSummary {
    dokime::domain::model::execution::wire::ExecutionSummary {
        archived: state.archived_executions.contains(&execution.test_execution_id),
        execution_id: execution.test_execution_id.to_string(),
        case_id: execution.test_case_id.to_string(),
        status: execution_status(&execution.status),
        ordinal: ordinal as u64,
        sealed: execution.saved.is_some(),
    }
}

fn inspection_reply<T: dokime::harmos::domain::Portable>(service: &Resources, value: T) -> Result<T, Status> {
    if value.wire_len()? > service.max_message_bytes {
        return Err(Status::resource_exhausted("inspection exceeds response budget"));
    }
    Ok(value)
}

fn case_status(status: CaseStatus) -> dokime::domain::model::execution::wire::CaseStatus {
    match status {
        CaseStatus::Skipped => dokime::domain::model::execution::wire::CaseStatus::Skipped,
        CaseStatus::Invalidated => dokime::domain::model::execution::wire::CaseStatus::Invalidated,
        CaseStatus::Planned => dokime::domain::model::execution::wire::CaseStatus::Planned,
        CaseStatus::Queued => dokime::domain::model::execution::wire::CaseStatus::Queued,
        CaseStatus::Running => dokime::domain::model::execution::wire::CaseStatus::Running,
        CaseStatus::Passed => dokime::domain::model::execution::wire::CaseStatus::Passed,
        CaseStatus::Failed => dokime::domain::model::execution::wire::CaseStatus::Failed,
        CaseStatus::Blocked => dokime::domain::model::execution::wire::CaseStatus::Blocked,
        CaseStatus::Cancelled => dokime::domain::model::execution::wire::CaseStatus::Cancelled,
        CaseStatus::Interrupted => dokime::domain::model::execution::wire::CaseStatus::Interrupted,
    }
}

pub(super) fn cycle_filter(filter: Option<&dokime::domain::model::execution::wire::CycleFilter>) -> Result<CycleFilter, Status> {
    let Some(filter) = filter else {
        return Ok(CycleFilter::default());
    };

    value::collection(filter.statuses.len())?;
    Ok(CycleFilter {
        text: filter.text.clone(),
        statuses: filter.statuses.iter().copied().map(cycle_status).collect::<Result<_, Status>>()?,
        created_after: filter.created_after.map(canonical::decoded_timestamp).transpose()?,
        created_before: filter.created_before.map(canonical::decoded_timestamp).transpose()?,
        archive: archive_scope(filter.archive),
    })
}

fn cycle_order(order: Option<&dokime::domain::model::execution::wire::CycleOrder>) -> CycleOrder {
    let Some(order) = order else {
        return CycleOrder::default();
    };
    let key = match order.key {
        dokime::domain::model::execution::wire::CycleOrderKey::Unspecified
        | dokime::domain::model::execution::wire::CycleOrderKey::UpdatedAt => CycleOrderKey::UpdatedAt,
        dokime::domain::model::execution::wire::CycleOrderKey::CreatedAt => CycleOrderKey::CreatedAt,
        dokime::domain::model::execution::wire::CycleOrderKey::Label => CycleOrderKey::Label,
    };
    CycleOrder { key, descending: order.descending }
}

fn cycle_status(status: dokime::domain::model::execution::wire::CycleStatus) -> Result<CycleStatus, Status> {
    match status {
        dokime::domain::model::execution::wire::CycleStatus::Unspecified => Err(Status::invalid_argument("unspecified cycle status")),
        dokime::domain::model::execution::wire::CycleStatus::Ready => Ok(CycleStatus::Ready),
        dokime::domain::model::execution::wire::CycleStatus::Running => Ok(CycleStatus::Running),
        dokime::domain::model::execution::wire::CycleStatus::Paused => Ok(CycleStatus::Paused),
        dokime::domain::model::execution::wire::CycleStatus::Stopping => Ok(CycleStatus::Stopping),
        dokime::domain::model::execution::wire::CycleStatus::Completed => Ok(CycleStatus::Completed),
        dokime::domain::model::execution::wire::CycleStatus::Failed => Ok(CycleStatus::Failed),
        dokime::domain::model::execution::wire::CycleStatus::Blocked => Ok(CycleStatus::Blocked),
        dokime::domain::model::execution::wire::CycleStatus::Stopped => Ok(CycleStatus::Stopped),
    }
}

fn cycle_id(id: &str) -> Result<TestCycleId, Status> {
    TestCycleId::from_uuid(id.to_owned()).map_err(|_| Status::invalid_argument("invalid cycle UUID"))
}

fn after_position<'a>(after: &str, mut ids: impl Iterator<Item = &'a str>) -> Result<usize, Status> {
    if after.is_empty() {
        return Ok(0);
    }
    ids.position(|id| id == after).map(|index| index + 1).ok_or_else(|| Status::invalid_argument("after ID not found in requested cycle"))
}

fn rejection(code: String, message: String, validation_issues: Vec<ValidationIssue>) -> dokime::domain::model::execution::wire::Rejection {
    dokime::domain::model::execution::wire::Rejection {
        code,
        message,
        issues: validation_issues
            .into_iter()
            .map(|issue| dokime::domain::model::execution::wire::ValidationIssue {
                path: issue.path,
                message: issue.message,
                expected: issue.expected,
            })
            .collect(),
    }
}

fn case_page(
    state: &RuntimeState, cycle: &CycleView, after: &str, limit: usize, cursor: Cursor,
) -> Result<dokime::domain::model::execution::wire::ListCasesReply, Status> {
    let start = after_position(after, cycle.cases().map(|case| case.test_case_id.as_str()))?;
    let count = cycle.cases().count();

    let mut cases = canonical::slots(count.saturating_sub(start).min(limit))?;
    for (ordinal, case) in cycle.cases().enumerate().skip(start).take(limit) {
        cases.push(case_summary(case, ordinal, case_projection(state, cycle, &case.test_case_id))?);
    }
    let next_after_case_id = if start + cases.len() < count {
        let last = cases.last().ok_or_else(|| Status::internal("empty continued case page"))?;

        Some(last.case_id.clone())
    } else {
        None
    };
    Ok(dokime::domain::model::execution::wire::ListCasesReply {
        cases,
        next_after_case_id,
        cursor: Some(cursor),
    })
}

fn execution_page<'a>(
    state: &RuntimeState, members: impl ExactSizeIterator<Item = &'a dokime::domain::model::execution::result::TestExecution> + Clone,
    after: &str, limit: usize, cursor: Cursor,
) -> Result<dokime::domain::model::execution::wire::ListExecutionsReply, Status> {
    let start = after_position(after, members.clone().map(|execution| execution.test_execution_id.as_str()))?;
    let count = members.len();

    let mut executions = canonical::slots(count.saturating_sub(start).min(limit))?;
    for (ordinal, execution) in members.enumerate().skip(start).take(limit) {
        executions.push(execution_summary(state, execution, ordinal));
    }
    let next_after_execution_id = if start + executions.len() < count {
        let last = executions.last().ok_or_else(|| Status::internal("empty continued execution page"))?;

        Some(last.execution_id.clone())
    } else {
        None
    };
    Ok(dokime::domain::model::execution::wire::ListExecutionsReply {
        executions,
        next_after_execution_id,
        cursor: Some(cursor),
    })
}
