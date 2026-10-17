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
use dokime_domain::model::identity::TestCycleId;
use dokime_domain::model::diagnostics::ValidationIssue;
use dokime_runtime::definition::TestCase;
use dokime_runtime::execution::{CycleStatus, CycleView, ExecutionStatus};
use dokime_runtime::runtime::RuntimeState;
use dokime_runtime::runtime::query::{
    CaseProjection, CaseStatus, CycleFilter, CycleOrder, CycleOrderKey, CyclePageError,
    case_projection, case_tally, cycle_page, historical_case_projection, select_cycles,
};
use dokime_runtime::{runtime::Error, RuntimeHandle};
use dokime_domain::model::execution::wire_cursor::Cursor;
use dokime_domain::operations::execution as wire;

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
        request: &mut wire::GetCase,
        _: &mut (),
    ) -> Result<harmos::operations::Response<dokime_domain::model::execution::wire::CaseDetail>, Status> {
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
        request: &mut wire::GetExecution,
        _: &mut (),
    ) -> Result<harmos::operations::Response<dokime_domain::model::execution::wire::ExecutionDetail>, Status> {
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
        request: &mut wire::ListCaseParameters,
        _: &mut (),
    ) -> Result<harmos::operations::Response<dokime_domain::model::execution::wire::ListCaseParametersReply>, Status> {
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
        request: &mut wire::ListExecutionParameters,
        _: &mut (),
    ) -> Result<harmos::operations::Response<dokime_domain::model::execution::wire::ListMaterializedParametersReply>, Status> {
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
        request: &mut wire::ListExecutionSteps,
        _: &mut (),
    ) -> Result<harmos::operations::Response<dokime_domain::model::execution::wire::ListExecutionStepsReply>, Status> {
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
        request: &mut wire::ListStepCriteria,
        _: &mut (),
    ) -> Result<harmos::operations::Response<dokime_domain::model::execution::wire::ListStepCriteriaReply>, Status> {
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
        request: &mut wire::GetCycle,
        _: &mut (),
    ) -> Result<harmos::operations::Response<dokime_domain::model::execution::wire::CycleSummary>, Status> {
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
        request: &mut wire::ListCycles,
        _: &mut (),
    ) -> Result<harmos::operations::Response<dokime_domain::model::execution::wire::ListCyclesReply>, Status> {
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
        request: &mut wire::ListCases,
        _: &mut (),
    ) -> Result<harmos::operations::Response<dokime_domain::model::execution::wire::ListCasesReply>, Status> {
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
        request: &mut wire::ListExecutions,
        _: &mut (),
    ) -> Result<harmos::operations::Response<dokime_domain::model::execution::wire::ListExecutionsReply>, Status> {
        self.resources.list_executions(request).map(Into::into)
    }
}

impl Resources {
    fn get_cycle(&self, request: &wire::GetCycle) -> Result<dokime_domain::model::execution::wire::CycleSummary, Status> {
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
        request: &wire::ListCycles,
    ) -> Result<dokime_domain::model::execution::wire::ListCyclesReply, Status> {
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
        bounds.bytes(std::mem::size_of::<dokime_domain::model::execution::wire::ListCyclesReply>())?;
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
            dokime_domain::model::execution::wire::ListCyclesReply {
                cycles,
                next_after_cycle_id,
                cursor: Some(cursor),
            },
        )
    }

    fn list_cases(&self, request: &wire::ListCases) -> Result<dokime_domain::model::execution::wire::ListCasesReply, Status> {
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
        request: &wire::ListExecutions,
    ) -> Result<dokime_domain::model::execution::wire::ListExecutionsReply, Status> {
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
            snapshot.state(),
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
            dokime_runtime::app::Lanes::Open,
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
    cycle: &dokime_runtime::execution::TestCycle,
    cursor: Cursor,
    bounds: &mut detail::Bounds,
) -> Result<dokime_domain::model::execution::wire::CycleSummary, Status> {
    bounds.bytes(std::mem::size_of::<dokime_domain::model::execution::wire::CycleSummary>())?;
    bounds.text(cycle.test_cycle_id.as_str())?;
    bounds.text(&cursor.epoch)?;
    bounds.text(&cursor.stream)?;
    bounds.bytes(canonical::metadata_memory(&cycle.metadata)?)?;
    let tally = case_tally(state, cycle);
    Ok(dokime_domain::model::execution::wire::CycleSummary {
        cycle_id: cycle.test_cycle_id.to_string(),
        metadata: Some(canonical::metadata(&cycle.metadata)?),
        status: match cycle.lifecycle {
            CycleStatus::Ready => dokime_domain::model::execution::wire::CycleStatus::Ready,
            CycleStatus::Running => dokime_domain::model::execution::wire::CycleStatus::Running,
            CycleStatus::Paused => dokime_domain::model::execution::wire::CycleStatus::Paused,
            CycleStatus::Stopping => dokime_domain::model::execution::wire::CycleStatus::Stopping,
            CycleStatus::Completed => dokime_domain::model::execution::wire::CycleStatus::Completed,
            CycleStatus::Failed => dokime_domain::model::execution::wire::CycleStatus::Failed,
            CycleStatus::Blocked => dokime_domain::model::execution::wire::CycleStatus::Blocked,
            CycleStatus::Stopped => dokime_domain::model::execution::wire::CycleStatus::Stopped,
        },
        archived: state.archived_cycles.contains(&cycle.test_cycle_id),
        sealed: cycle.saved.is_some(),
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
        running_count: tally.running,
        queued_count: tally.queued,
    })
}

pub(super) fn case_summary(
    case: &TestCase,
    ordinal: usize,
    projection: CaseProjection,
    bounds: &mut detail::Bounds,
) -> Result<dokime_domain::model::execution::wire::CaseSummary, Status> {
    bounds.bytes(std::mem::size_of::<dokime_domain::model::execution::wire::CaseSummary>())?;
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
    Ok(dokime_domain::model::execution::wire::CaseSummary {
        suite_id: case.suite_id.clone(),
        case_id: case.test_case_id.as_str().to_owned(),
        template_id: case.test_case_template_id.as_str().to_owned(),
        metadata: Some(canonical::metadata(&case.metadata)?),
        ordinal: ordinal as u64,
        status: projection
            .status
            .map_or(dokime_domain::model::execution::wire::CaseStatus::Unspecified, case_status),
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
        actions: Some(dokime_domain::model::execution::wire::CaseActions {
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
) -> Result<dokime_domain::model::execution::wire::CaseSummary, Status> {
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

fn case_status(status: CaseStatus) -> dokime_domain::model::execution::wire::CaseStatus {
    match status {
        CaseStatus::Skipped => dokime_domain::model::execution::wire::CaseStatus::Skipped,
        CaseStatus::Invalidated => dokime_domain::model::execution::wire::CaseStatus::Invalidated,
        CaseStatus::Planned => dokime_domain::model::execution::wire::CaseStatus::Planned,
        CaseStatus::Queued => dokime_domain::model::execution::wire::CaseStatus::Queued,
        CaseStatus::Running => dokime_domain::model::execution::wire::CaseStatus::Running,
        CaseStatus::Passed => dokime_domain::model::execution::wire::CaseStatus::Passed,
        CaseStatus::Failed => dokime_domain::model::execution::wire::CaseStatus::Failed,
        CaseStatus::Blocked => dokime_domain::model::execution::wire::CaseStatus::Blocked,
        CaseStatus::Cancelled => dokime_domain::model::execution::wire::CaseStatus::Cancelled,
        CaseStatus::Interrupted => dokime_domain::model::execution::wire::CaseStatus::Interrupted,
    }
}

pub(super) fn execution_status(status: &ExecutionStatus) -> dokime_domain::model::execution::wire::ExecutionStatus {
    match status {
        ExecutionStatus::Invalidated => dokime_domain::model::execution::wire::ExecutionStatus::Invalidated,
        ExecutionStatus::Skipped => dokime_domain::model::execution::wire::ExecutionStatus::Skipped,
        ExecutionStatus::Pending => dokime_domain::model::execution::wire::ExecutionStatus::Pending,
        ExecutionStatus::Running => dokime_domain::model::execution::wire::ExecutionStatus::Running,
        ExecutionStatus::Passed => dokime_domain::model::execution::wire::ExecutionStatus::Passed,
        ExecutionStatus::Failed => dokime_domain::model::execution::wire::ExecutionStatus::Failed,
        ExecutionStatus::Blocked => dokime_domain::model::execution::wire::ExecutionStatus::Blocked,
        ExecutionStatus::Cancelled => dokime_domain::model::execution::wire::ExecutionStatus::Cancelled,
        ExecutionStatus::Interrupted => dokime_domain::model::execution::wire::ExecutionStatus::Interrupted,
    }
}

fn cycle_filter(filter: Option<&dokime_domain::model::execution::wire::CycleFilter>) -> Result<CycleFilter, Status> {
    let Some(filter) = filter else {
        return Ok(CycleFilter::default());
    };
    value::text(&filter.text)?;
    value::collection(filter.statuses.len())?;
    Ok(CycleFilter {
        text: filter.text.clone(),
        statuses: filter
            .statuses
            .iter()
            .copied()
            .map(cycle_status)
            .collect::<Result<_, Status>>()?,
        created_after: filter
            .created_after
            .map(canonical::decoded_timestamp)
            .transpose()?,
        created_before: filter
            .created_before
            .map(canonical::decoded_timestamp)
            .transpose()?,
        archive: archive_scope(filter.archive),
    })
}

fn cycle_order(order: Option<&dokime_domain::model::execution::wire::CycleOrder>) -> CycleOrder {
    let Some(order) = order else {
        return CycleOrder::default();
    };
    let key = match order.key {
        dokime_domain::model::execution::wire::CycleOrderKey::Unspecified | dokime_domain::model::execution::wire::CycleOrderKey::UpdatedAt => {
            CycleOrderKey::UpdatedAt
        }
        dokime_domain::model::execution::wire::CycleOrderKey::CreatedAt => CycleOrderKey::CreatedAt,
        dokime_domain::model::execution::wire::CycleOrderKey::Label => CycleOrderKey::Label,
    };
    CycleOrder {
        key,
        descending: order.descending,
    }
}

/// The archive selection a list request asks for; absent lists unarchived rows.
pub(super) fn archive_scope(
    scope: Option<dokime_domain::model::execution::wire::ArchiveScope>,
) -> dokime_runtime::runtime::query::executions::ArchiveScope {
    use dokime_runtime::runtime::query::executions::ArchiveScope;
    match scope {
        None | Some(dokime_domain::model::execution::wire::ArchiveScope::Unarchived) => ArchiveScope::Unarchived,
        Some(dokime_domain::model::execution::wire::ArchiveScope::Archived) => ArchiveScope::Archived,
        Some(dokime_domain::model::execution::wire::ArchiveScope::All) => ArchiveScope::All,
    }
}

fn cycle_status(status: dokime_domain::model::execution::wire::CycleStatus) -> Result<CycleStatus, Status> {
    match status {
        dokime_domain::model::execution::wire::CycleStatus::Unspecified => Err(Status::invalid_argument("unspecified cycle status")),
        dokime_domain::model::execution::wire::CycleStatus::Ready => Ok(CycleStatus::Ready),
        dokime_domain::model::execution::wire::CycleStatus::Running => Ok(CycleStatus::Running),
        dokime_domain::model::execution::wire::CycleStatus::Paused => Ok(CycleStatus::Paused),
        dokime_domain::model::execution::wire::CycleStatus::Stopping => Ok(CycleStatus::Stopping),
        dokime_domain::model::execution::wire::CycleStatus::Completed => Ok(CycleStatus::Completed),
        dokime_domain::model::execution::wire::CycleStatus::Failed => Ok(CycleStatus::Failed),
        dokime_domain::model::execution::wire::CycleStatus::Blocked => Ok(CycleStatus::Blocked),
        dokime_domain::model::execution::wire::CycleStatus::Stopped => Ok(CycleStatus::Stopped),
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
) -> dokime_domain::model::execution::wire::Rejection {
    dokime_domain::model::execution::wire::Rejection {
        code,
        message,
        issues: validation_issues
            .into_iter()
            .map(
                |issue| dokime_domain::model::execution::wire::ValidationIssue {
                    path: issue.path,
                    message: issue.message,
                    expected: issue.expected,
                },
            )
            .collect(),
    }
}

fn case_page(
    state: &RuntimeState,
    cycle: &CycleView,
    after: &str,
    limit: usize,
    cursor: Cursor,
    bounds: &mut detail::Bounds,
) -> Result<dokime_domain::model::execution::wire::ListCasesReply, Status> {
    let start = after_position(after, cycle.cases().map(|case| case.test_case_id.as_str()))?;
    let count = cycle.cases().count();
    bounds.bytes(std::mem::size_of::<dokime_domain::model::execution::wire::ListCasesReply>())?;
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
    Ok(dokime_domain::model::execution::wire::ListCasesReply {
        cases,
        next_after_case_id,
        cursor: Some(cursor),
    })
}

fn execution_page<'a>(
    state: &RuntimeState,
    members: impl ExactSizeIterator<Item = &'a dokime_runtime::execution::TestExecution> + Clone,
    after: &str,
    limit: usize,
    cursor: Cursor,
    bounds: &mut detail::Bounds,
) -> Result<dokime_domain::model::execution::wire::ListExecutionsReply, Status> {
    let start = after_position(
        after,
        members
            .clone()
            .map(|execution| execution.test_execution_id.as_str()),
    )?;
    let count = members.len();
    bounds.bytes(std::mem::size_of::<dokime_domain::model::execution::wire::ListExecutionsReply>())?;
    bounds.text(&cursor.epoch)?;
    bounds.text(&cursor.stream)?;
    let mut executions = canonical::slots(count.saturating_sub(start).min(limit))?;
    for (ordinal, execution) in members.enumerate().skip(start).take(limit) {
        bounds.bytes(std::mem::size_of::<dokime_domain::model::execution::wire::ExecutionSummary>())?;
        bounds.text(execution.test_execution_id.as_str())?;
        bounds.text(execution.test_case_id.as_str())?;
        executions.push(execution_summary(state, execution, ordinal));
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
    Ok(dokime_domain::model::execution::wire::ListExecutionsReply {
        executions,
        next_after_execution_id,
        cursor: Some(cursor),
    })
}

pub(super) fn execution_summary(
    state: &RuntimeState,
    execution: &dokime_runtime::execution::TestExecution,
    ordinal: usize,
) -> dokime_domain::model::execution::wire::ExecutionSummary {
    dokime_domain::model::execution::wire::ExecutionSummary {
        archived: state
            .archived_executions
            .contains(&execution.test_execution_id),
        execution_id: execution.test_execution_id.to_string(),
        case_id: execution.test_case_id.to_string(),
        status: execution_status(&execution.status),
        ordinal: ordinal as u64,
        sealed: execution.saved.is_some(),
    }
}

/* === Tests === */
