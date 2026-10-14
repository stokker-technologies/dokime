/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod commands;
mod control;
mod detail;
mod loan;
mod requests;
mod steps;

/* === Re-Exports === */
pub(super) use control::command_context;
pub(super) use detail::Bounds;
pub(crate) use detail::result as result_summary;

/* === Imports === */
// Standard Library Imports
use std::sync::Arc;

// Third Party Imports
use harmos::operations::{Context, Operation};
use prost::Message;
use tonic::{Code, Status};
// Local Imports
use super::value;
use super::value::canonical;
use dokime_app::core::{TestCycleId, ValidationIssue};
use dokime_app::definition::TestCase;
use dokime_app::execution::{CycleStatus, CycleView, ExecutionStatus};
use dokime_app::runtime::RuntimeState;
use dokime_app::runtime::query::{
    CaseProjection, CaseStatus, CycleFilter, CycleOrder, CycleOrderKey, CyclePageError,
    case_projection, cycle_page, historical_case_projection, select_cycles,
};
use dokime_app::{Error, RuntimeHandle};
use dokime_contract::operations::{common, cursor::Cursor, execution as wire};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Resources {
    pub(super) runtime: RuntimeHandle,
    pub(super) max_message_bytes: usize,
}

struct GetCycle {
    resources: Arc<Resources>,
}
struct ListCycles {
    resources: Arc<Resources>,
}
struct ListCases {
    resources: Arc<Resources>,
}
struct ListExecutions {
    resources: Arc<Resources>,
}

struct GetCase {
    resources: Arc<Resources>,
}
struct GetExecution {
    resources: Arc<Resources>,
}
struct ListCaseParameters {
    resources: Arc<Resources>,
}
struct ListExecutionParameters {
    resources: Arc<Resources>,
}
struct ListExecutionSteps {
    resources: Arc<Resources>,
}
struct ListStepCriteria {
    resources: Arc<Resources>,
}

/* === Enums === */
#[harmos::catalog]
enum Operations {
    GetCase(GetCase),
    GetExecution(GetExecution),
    ListCaseParameters(ListCaseParameters),
    ListExecutionParameters(ListExecutionParameters),
    ListExecutionSteps(ListExecutionSteps),
    ListStepCriteria(ListStepCriteria),

    GetCycle(GetCycle),
    ListCycles(ListCycles),
    ListCases(ListCases),
    ListExecutions(ListExecutions),
}

/* === Traits === */

/* === Implementations === */
impl Operation for GetCase {
    type Declaration = wire::GetCase;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::GetCaseRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::CaseDetail>, Status> {
        detail::get_case(&self.resources, request).map(Into::into)
    }
}

impl Operation for GetExecution {
    type Declaration = wire::GetExecution;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::GetExecutionRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ExecutionDetail>, Status> {
        detail::get_execution(&self.resources, request).map(Into::into)
    }
}

impl Operation for ListCaseParameters {
    type Declaration = wire::ListCaseParameters;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::ListCaseParametersRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ListCaseParametersReply>, Status> {
        detail::list_case_parameters(&self.resources, request).map(Into::into)
    }
}

impl Operation for ListExecutionParameters {
    type Declaration = wire::ListExecutionParameters;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::ListExecutionParametersRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ListMaterializedParametersReply>, Status> {
        detail::list_execution_parameters(&self.resources, request).map(Into::into)
    }
}

impl Operation for ListExecutionSteps {
    type Declaration = wire::ListExecutionSteps;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::ListExecutionStepsRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ListExecutionStepsReply>, Status> {
        steps::list(&self.resources, request).map(Into::into)
    }
}

impl Operation for ListStepCriteria {
    type Declaration = wire::ListStepCriteria;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::ListStepEvidenceRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ListStepCriteriaReply>, Status> {
        steps::criteria(&self.resources, request).map(Into::into)
    }
}

impl Operation for GetCycle {
    type Declaration = wire::GetCycle;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::GetCycleRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::CycleSummary>, Status> {
        self.resources.get_cycle(request).map(Into::into)
    }
}

impl Operation for ListCycles {
    type Declaration = wire::ListCycles;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::ListCyclesRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ListCyclesReply>, Status> {
        self.resources.list_cycles(request).map(Into::into)
    }
}

impl Operation for ListCases {
    type Declaration = wire::ListCases;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::ListCasesRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ListCasesReply>, Status> {
        self.resources.list_cases(request).map(Into::into)
    }
}

impl Operation for ListExecutions {
    type Declaration = wire::ListExecutions;
    type Resources = Resources;
    type Error = Status;
    type Context<'a> = Context<'a>;
    type Endpoints = ();
    async fn initialize<'ctx>(
        resources: Arc<Resources>,
        _: Self::Context<'ctx>,
    ) -> Result<Self, Status> {
        Ok(Self { resources })
    }
    async fn run<'a>(
        &mut self,
        _: Self::Context<'a>,
        request: &mut wire::ListExecutionsRequest,
        _: &mut (),
    ) -> Result<harmos::operations::Response<wire::ListExecutionsReply>, Status> {
        self.resources.list_executions(request).map(Into::into)
    }
}

impl Resources {
    fn get_cycle(&self, request: &wire::GetCycleRequest) -> Result<wire::CycleSummary, Status> {
        let id = cycle_id(&request.cycle_id)?;
        let snapshot = self.runtime.watch_state().latest();
        let cycle = snapshot
            .state()
            .cycles
            .get(&id)
            .ok_or_else(|| Status::not_found("cycle not found"))?;
        let mut bounds = detail::Bounds::new(self.max_message_bytes);
        let value = summary(
            snapshot.state(),
            cycle,
            canonical::cursor(&snapshot),
            &mut bounds,
        )?;
        inspection_reply(self, value)
    }

    fn list_cycles(
        &self,
        request: &wire::ListCyclesRequest,
    ) -> Result<wire::ListCyclesReply, Status> {
        let snapshot = self.runtime.watch_state().latest();
        let cursor = canonical::cursor(&snapshot);
        let limit = cursor
            .page(
                request.limit,
                &request.after_cycle_id,
                request.cursor.as_ref(),
            )
            .map_err(canonical::cursor_failure)?;
        let filter = cycle_filter(request.filter.as_ref())?;
        let order = cycle_order(request.order.as_ref());
        let selected = select_cycles(snapshot.state(), &filter, order);
        let after = if request.after_cycle_id.is_empty() {
            None
        } else {
            cycle_id(&request.after_cycle_id)?;
            Some(request.after_cycle_id.as_str())
        };
        let (page, next_after_cycle_id) =
            cycle_page(&selected, after, limit).map_err(|error| match error {
                CyclePageError::UnknownContinuation => Status::invalid_argument(error.to_string()),
            })?;
        let mut bounds = detail::Bounds::new(self.max_message_bytes);
        bounds.bytes(std::mem::size_of::<wire::ListCyclesReply>())?;
        if let Some(after) = &next_after_cycle_id {
            bounds.text(after)?;
        }
        let mut cycles = canonical::slots(page.len())?;
        for cycle in page {
            cycles.push(summary(
                snapshot.state(),
                cycle,
                cursor.clone(),
                &mut bounds,
            )?);
        }
        inspection_reply(
            self,
            wire::ListCyclesReply {
                cycles,
                next_after_cycle_id,
                cursor: Some(cursor),
            },
        )
    }

    fn list_cases(&self, request: &wire::ListCasesRequest) -> Result<wire::ListCasesReply, Status> {
        let id = cycle_id(&request.cycle_id)?;
        let snapshot = self.runtime.watch_state().latest();
        let cursor = canonical::cursor(&snapshot);
        let limit = cursor
            .page(
                request.limit,
                &request.after_case_id,
                request.cursor.as_ref(),
            )
            .map_err(canonical::cursor_failure)?;
        let cycle = snapshot
            .state()
            .cycle_view(&id)
            .ok_or_else(|| Status::not_found("cycle not found"))?;
        let mut bounds = detail::Bounds::new(self.max_message_bytes);
        let value = case_page(
            snapshot.state(),
            &cycle,
            &request.after_case_id,
            limit,
            cursor,
            &mut bounds,
        )?;
        inspection_reply(self, value)
    }

    fn list_executions(
        &self,
        request: &wire::ListExecutionsRequest,
    ) -> Result<wire::ListExecutionsReply, Status> {
        let id = cycle_id(&request.cycle_id)?;
        let snapshot = self.runtime.watch_state().latest();
        let cursor = canonical::cursor(&snapshot);
        let limit = cursor
            .page(
                request.limit,
                &request.after_execution_id,
                request.cursor.as_ref(),
            )
            .map_err(canonical::cursor_failure)?;
        let cycle = snapshot
            .state()
            .cycles
            .get(&id)
            .ok_or_else(|| Status::not_found("cycle not found"))?;
        let mut bounds = detail::Bounds::new(self.max_message_bytes);
        let value = execution_page(
            snapshot.state().executions_for_cycle(cycle),
            &request.after_execution_id,
            limit,
            cursor,
            &mut bounds,
        )?;
        inspection_reply(self, value)
    }
}

/* === Public Functions === */
pub(super) fn register(
    runtime: &RuntimeHandle,
    max_message_bytes: usize,
) -> Result<(), super::Error> {
    control::register(runtime)?;
    requests::register(runtime, max_message_bytes)?;
    commands::register(runtime, max_message_bytes)?;
    runtime
        .operations()
        .register_wire_operations::<Operations, Resources>(
            "dokime",
            dokime_app::app::Lanes::Open,
            Arc::new(Resources {
                runtime: runtime.clone(),
                max_message_bytes,
            }),
        )?;
    Ok(())
}

pub(super) fn status(error: Error) -> Status {
    let (grpc_code, code, message, issues) = match error {
        Error::Validation {
            code,
            message,
            issues,
        } => (Code::InvalidArgument, code, message, issues),
        Error::Conflict { code, message } => (Code::FailedPrecondition, code, message, vec![]),
        Error::Overload { code, message } => (Code::ResourceExhausted, code, message, vec![]),
        Error::Unavailable { code, message } => (Code::Unavailable, code, message, vec![]),
        Error::Forbidden { code, message } => (Code::PermissionDenied, code, message, vec![]),
        Error::NotFound { code, message } => (Code::NotFound, code, message, vec![]),
        Error::Timeout { code, message } => (Code::DeadlineExceeded, code, message, vec![]),
        Error::Cancelled { code, message } => (Code::Cancelled, code, message, vec![]),
        Error::Internal { code, message } => (Code::Internal, code, message, vec![]),
        error => (
            Code::Internal,
            "runtime_internal".to_owned(),
            error.to_string(),
            vec![],
        ),
    };
    let detail = rejection(code, message, issues);
    let envelope = tonic_types::Status {
        code: grpc_code as i32,
        message: detail.message.clone(),
        details: vec![prost_types::Any {
            type_url: "type.googleapis.com/dokime.v1.Rejection".into(),
            value: harmos::Protobuf::encode_to_vec(&detail),
        }],
    };
    Status::with_details(grpc_code, detail.message, envelope.encode_to_vec().into())
}

pub(super) fn summary(
    state: &RuntimeState,
    cycle: &dokime_app::execution::TestCycle,
    cursor: Cursor,
    bounds: &mut detail::Bounds,
) -> Result<wire::CycleSummary, Status> {
    bounds.bytes(std::mem::size_of::<wire::CycleSummary>())?;
    bounds.text(cycle.test_cycle_id.as_str())?;
    bounds.text(&cursor.epoch)?;
    bounds.text(&cursor.stream)?;
    bounds.bytes(canonical::metadata_memory(&cycle.metadata)?)?;
    let suite = cycle_suite(state, cycle, bounds)?;
    Ok(wire::CycleSummary {
        cycle_id: cycle.test_cycle_id.to_string(),
        metadata: Some(canonical::metadata(&cycle.metadata)?),
        status: match cycle.lifecycle {
            CycleStatus::Previewed => wire::CycleStatus::Previewed,
            CycleStatus::Ready => wire::CycleStatus::Ready,
            CycleStatus::Running => wire::CycleStatus::Running,
            CycleStatus::Paused => wire::CycleStatus::Paused,
            CycleStatus::Stopping => wire::CycleStatus::Stopping,
            CycleStatus::Completed => wire::CycleStatus::Completed,
            CycleStatus::Failed => wire::CycleStatus::Failed,
            CycleStatus::Blocked => wire::CycleStatus::Blocked,
            CycleStatus::Stopped => wire::CycleStatus::Stopped,
            CycleStatus::Cleared => wire::CycleStatus::Cleared,
        },
        case_count: state.cases_for_cycle(cycle).count() as u64,
        execution_count: cycle.execution_ids.len() as u64,
        continue_on_failure: cycle.continue_on_failure,
        observed_cursor: Some(cursor),
        created_at: Some(canonical::timestamp(&cycle.created_at)?),
        started_at: cycle
            .started_at
            .as_ref()
            .map(canonical::timestamp)
            .transpose()?,
        updated_at: Some(canonical::timestamp(&cycle.updated_at)?),
        result: cycle.result.as_ref().map(detail::result),
        suite,
    })
}

pub(super) fn case_summary(
    case: &TestCase,
    ordinal: usize,
    projection: CaseProjection,
    bounds: &mut detail::Bounds,
) -> Result<wire::CaseSummary, Status> {
    bounds.bytes(std::mem::size_of::<wire::CaseSummary>())?;
    bounds.text(&case.suite_id)?;
    bounds.text(case.test_case_id.as_str())?;
    bounds.text(case.test_case_template_id.as_str())?;
    bounds.bytes(canonical::metadata_memory(&case.metadata)?)?;
    if let Some(id) = &projection.latest_execution_id {
        bounds.text(id.as_str())?;
    }
    value::collection(projection.actions.unavailable_reasons.len())?;
    bounds.bytes(
        projection
            .actions
            .unavailable_reasons
            .capacity()
            .saturating_mul(std::mem::size_of::<String>()),
    )?;
    for reason in &projection.actions.unavailable_reasons {
        bounds.text(reason)?;
    }
    Ok(wire::CaseSummary {
        suite_id: case.suite_id.clone(),
        case_id: case.test_case_id.as_str().to_owned(),
        template_id: case.test_case_template_id.as_str().to_owned(),
        metadata: Some(canonical::metadata(&case.metadata)?),
        ordinal: ordinal as u64,
        status: projection
            .status
            .map_or(wire::CaseStatus::Unspecified, case_status),
        latest_execution_id: projection
            .latest_execution_id
            .as_ref()
            .map(|id| id.as_str().to_owned()),
        latest_execution_status: projection
            .latest_execution_status
            .as_ref()
            .map(execution_status),
        attempt_count: projection.attempt_count,
        rerun_scheduled: projection.rerun_scheduled,
        actions: Some(wire::CaseActions {
            run: projection.actions.run,
            schedule_rerun: projection.actions.schedule_rerun,
            delete: projection.actions.delete,
            unavailable_reasons: projection.actions.unavailable_reasons,
        }),
    })
}

pub(super) fn historical_case_summary(
    cycle: &CycleView,
    case: &TestCase,
    ordinal: usize,
    bounds: &mut detail::Bounds,
) -> Result<wire::CaseSummary, Status> {
    case_summary(
        case,
        ordinal,
        historical_case_projection(cycle, &case.test_case_id),
        bounds,
    )
}

/* === Private Functions === */
fn inspection_reply<T: harmos::Protobuf>(service: &Resources, value: T) -> Result<T, Status> {
    if value.encoded_len() > service.max_message_bytes {
        return Err(Status::resource_exhausted(
            "inspection exceeds response budget",
        ));
    }
    Ok(value)
}

fn case_status(status: CaseStatus) -> wire::CaseStatus {
    match status {
        CaseStatus::Skipped => wire::CaseStatus::Skipped,
        CaseStatus::Invalidated => wire::CaseStatus::Invalidated,
        CaseStatus::Planned => wire::CaseStatus::Planned,
        CaseStatus::Queued => wire::CaseStatus::Queued,
        CaseStatus::Running => wire::CaseStatus::Running,
        CaseStatus::Passed => wire::CaseStatus::Passed,
        CaseStatus::Failed => wire::CaseStatus::Failed,
        CaseStatus::Blocked => wire::CaseStatus::Blocked,
        CaseStatus::Cancelled => wire::CaseStatus::Cancelled,
        CaseStatus::Interrupted => wire::CaseStatus::Interrupted,
    }
}

pub(super) fn execution_status(status: &ExecutionStatus) -> wire::ExecutionStatus {
    match status {
        ExecutionStatus::Invalidated => wire::ExecutionStatus::Invalidated,
        ExecutionStatus::Skipped => wire::ExecutionStatus::Skipped,
        ExecutionStatus::Pending => wire::ExecutionStatus::Pending,
        ExecutionStatus::Running => wire::ExecutionStatus::Running,
        ExecutionStatus::Passed => wire::ExecutionStatus::Passed,
        ExecutionStatus::Failed => wire::ExecutionStatus::Failed,
        ExecutionStatus::Blocked => wire::ExecutionStatus::Blocked,
        ExecutionStatus::Cancelled => wire::ExecutionStatus::Cancelled,
        ExecutionStatus::Interrupted => wire::ExecutionStatus::Interrupted,
    }
}

fn cycle_filter(filter: Option<&wire::CycleFilter>) -> Result<CycleFilter, Status> {
    let Some(filter) = filter else {
        return Ok(CycleFilter::default());
    };
    value::text(&filter.text)?;
    value::collection(filter.statuses.len())?;
    value::collection(filter.suite_ids.len())?;
    for suite_id in &filter.suite_ids {
        value::text(suite_id)?;
    }
    Ok(CycleFilter {
        text: filter.text.clone(),
        statuses: filter
            .statuses
            .iter()
            .copied()
            .map(cycle_status)
            .collect::<Result<_, Status>>()?,
        suite_ids: filter.suite_ids.clone(),
        created_after: filter
            .created_after
            .map(canonical::decoded_timestamp)
            .transpose()?,
        created_before: filter
            .created_before
            .map(canonical::decoded_timestamp)
            .transpose()?,
    })
}

fn cycle_order(order: Option<&wire::CycleOrder>) -> CycleOrder {
    let Some(order) = order else {
        return CycleOrder::default();
    };
    let key = match order.key {
        wire::CycleOrderKey::Unspecified | wire::CycleOrderKey::UpdatedAt => {
            CycleOrderKey::UpdatedAt
        }
        wire::CycleOrderKey::CreatedAt => CycleOrderKey::CreatedAt,
        wire::CycleOrderKey::Label => CycleOrderKey::Label,
    };
    CycleOrder {
        key,
        descending: order.descending,
    }
}

fn cycle_status(status: wire::CycleStatus) -> Result<CycleStatus, Status> {
    match status {
        wire::CycleStatus::Unspecified => Err(Status::invalid_argument("unspecified cycle status")),
        wire::CycleStatus::Previewed => Ok(CycleStatus::Previewed),
        wire::CycleStatus::Ready => Ok(CycleStatus::Ready),
        wire::CycleStatus::Running => Ok(CycleStatus::Running),
        wire::CycleStatus::Paused => Ok(CycleStatus::Paused),
        wire::CycleStatus::Stopping => Ok(CycleStatus::Stopping),
        wire::CycleStatus::Completed => Ok(CycleStatus::Completed),
        wire::CycleStatus::Failed => Ok(CycleStatus::Failed),
        wire::CycleStatus::Blocked => Ok(CycleStatus::Blocked),
        wire::CycleStatus::Stopped => Ok(CycleStatus::Stopped),
        wire::CycleStatus::Cleared => Ok(CycleStatus::Cleared),
    }
}

fn cycle_id(id: &str) -> Result<TestCycleId, Status> {
    TestCycleId::from_uuid(id.to_owned())
        .map_err(|_| Status::invalid_argument("invalid cycle UUID"))
}

fn after_position<'a>(
    after: &str,
    mut ids: impl Iterator<Item = &'a str>,
) -> Result<usize, Status> {
    if after.is_empty() {
        return Ok(0);
    }
    ids.position(|id| id == after)
        .map(|index| index + 1)
        .ok_or_else(|| Status::invalid_argument("after ID not found in requested cycle"))
}

fn rejection(
    code: String,
    message: String,
    validation_issues: Vec<ValidationIssue>,
) -> dokime_contract::operations::execution::Rejection {
    dokime_contract::operations::execution::Rejection {
        code,
        message,
        issues: validation_issues
            .into_iter()
            .map(
                |issue| dokime_contract::operations::execution::ValidationIssue {
                    path: issue.path,
                    message: issue.message,
                    expected: issue.expected,
                },
            )
            .collect(),
    }
}

fn cycle_suite(
    state: &RuntimeState,
    cycle: &dokime_app::execution::TestCycle,
    bounds: &mut detail::Bounds,
) -> Result<Option<common::Metadata>, Status> {
    let mut suites = state
        .cases_for_cycle(cycle)
        .filter_map(|case| case.suite.as_ref())
        .chain(
            state
                .executions_for_cycle(cycle)
                .filter_map(|execution| execution.suite.as_ref()),
        );
    let Some(first) = suites.next() else {
        return Ok(None);
    };
    if !suites.all(|suite| suite.identifier == first.identifier) {
        return Ok(None);
    }
    bounds.bytes(canonical::metadata_memory(first)?)?;
    Ok(Some(canonical::metadata(first)?))
}

fn case_page(
    state: &RuntimeState,
    cycle: &CycleView,
    after: &str,
    limit: usize,
    cursor: Cursor,
    bounds: &mut detail::Bounds,
) -> Result<wire::ListCasesReply, Status> {
    let start = after_position(after, cycle.cases().map(|case| case.test_case_id.as_str()))?;
    let count = cycle.cases().count();
    bounds.bytes(std::mem::size_of::<wire::ListCasesReply>())?;
    bounds.text(&cursor.epoch)?;
    bounds.text(&cursor.stream)?;
    let mut cases = canonical::slots(count.saturating_sub(start).min(limit))?;
    for (ordinal, case) in cycle.cases().enumerate().skip(start).take(limit) {
        cases.push(case_summary(
            case,
            ordinal,
            case_projection(state, cycle, &case.test_case_id),
            bounds,
        )?);
    }
    let next_after_case_id = if start + cases.len() < count {
        let last = cases
            .last()
            .ok_or_else(|| Status::internal("empty continued case page"))?;
        bounds.text(&last.case_id)?;
        Some(last.case_id.clone())
    } else {
        None
    };
    Ok(wire::ListCasesReply {
        cases,
        next_after_case_id,
        cursor: Some(cursor),
    })
}

fn execution_page<'a>(
    members: impl ExactSizeIterator<Item = &'a dokime_app::execution::TestExecution> + Clone,
    after: &str,
    limit: usize,
    cursor: Cursor,
    bounds: &mut detail::Bounds,
) -> Result<wire::ListExecutionsReply, Status> {
    let start = after_position(
        after,
        members
            .clone()
            .map(|execution| execution.test_execution_id.as_str()),
    )?;
    let count = members.len();
    bounds.bytes(std::mem::size_of::<wire::ListExecutionsReply>())?;
    bounds.text(&cursor.epoch)?;
    bounds.text(&cursor.stream)?;
    let mut executions = canonical::slots(count.saturating_sub(start).min(limit))?;
    for (ordinal, execution) in members.enumerate().skip(start).take(limit) {
        bounds.bytes(std::mem::size_of::<wire::ExecutionSummary>())?;
        bounds.text(execution.test_execution_id.as_str())?;
        bounds.text(execution.test_case_id.as_str())?;
        executions.push(execution_summary(execution, ordinal));
    }
    let next_after_execution_id = if start + executions.len() < count {
        let last = executions
            .last()
            .ok_or_else(|| Status::internal("empty continued execution page"))?;
        bounds.text(&last.execution_id)?;
        Some(last.execution_id.clone())
    } else {
        None
    };
    Ok(wire::ListExecutionsReply {
        executions,
        next_after_execution_id,
        cursor: Some(cursor),
    })
}

fn execution_summary(
    execution: &dokime_app::execution::TestExecution,
    ordinal: usize,
) -> wire::ExecutionSummary {
    wire::ExecutionSummary {
        execution_id: execution.test_execution_id.to_string(),
        case_id: execution.test_case_id.to_string(),
        status: execution_status(&execution.status),
        ordinal: ordinal as u64,
    }
}

/* === Tests === */
