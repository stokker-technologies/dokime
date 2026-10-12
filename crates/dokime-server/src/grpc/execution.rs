/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
pub(super) mod codec;
mod commands;
mod control;
mod detail;
mod requests;
mod steps;

mod generated {
    include!(concat!(env!("OUT_DIR"), "/dokime.v1.Execution.rs"));
}

/* === Re-Exports === */
pub(crate) use detail::result as result_summary;
pub(super) use generated::execution_server::ExecutionServer;

/* === Imports === */
// Standard Library Imports

// Third Party Imports
use prost::Message;
use tonic::{Code, Request, Response, Status};
// Local Imports
use super::{
    access::{Principal, Role},
    observation, value,
};
use dokime_app::core::{CaseId, CorrelationId, TestCycleId, ValidationIssue};
use dokime_app::definition::TestCase;
use dokime_app::execution::{CycleStatus, CycleView, ExecutionStatus};
use dokime_app::runtime::RuntimeState;
use dokime_app::runtime::command::{CommandContext, CommandReceipt, CommandStatus};
use dokime_app::runtime::query::{
    CaseProjection, CaseStatus, CycleFilter, CycleOrder, CycleOrderKey, CyclePageError,
    case_projection, cycle_page, historical_case_projection, select_cycles,
};
use dokime_app::{Error, RuntimeHandle};
use dokime_protocol::v1;
use generated::execution_server::Execution;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Service {
    pub(super) runtime: RuntimeHandle,
    pub(super) max_message_bytes: usize,
}

/* === Enums === */
#[derive(Clone, Copy)]
enum CycleOp {
    Pause,
    Resume,
    Stop,
}

/* === Traits === */

/* === Implementations === */
#[tonic::async_trait]
impl Execution for Service {
    async fn copy_cycle(
        &self,
        request: Request<v1::CopyCycleRequest>,
    ) -> Result<Response<v1::CreateCycleReply>, Status> {
        let context = context(&request, request.get_ref().context.as_ref())?;
        let id = self
            .runtime
            .copy_cycle(cycle_id(&request.get_ref().source_cycle_id)?, context)
            .await
            .map_err(status)?;
        Ok(Response::new(v1::CreateCycleReply {
            cycle_id: id.to_string(),
            observed_cursor: Some(observation::cursor(&self.runtime.watch_state().latest())),
        }))
    }

    async fn list_execution_steps(
        &self,
        request: Request<v1::ListExecutionStepsRequest>,
    ) -> Result<Response<v1::ListExecutionStepsReply>, Status> {
        steps::list(self, request)
    }
    async fn list_step_criteria(
        &self,
        request: Request<v1::ListStepEvidenceRequest>,
    ) -> Result<Response<v1::ListStepCriteriaReply>, Status> {
        steps::criteria(self, request)
    }

    async fn delete_case(
        &self,
        request: Request<v1::CaseControlRequest>,
    ) -> Result<Response<v1::ControlReply>, Status> {
        control::case_control(&self.runtime, request, control::CaseOp::Delete).await
    }
    async fn schedule_case_rerun(
        &self,
        request: Request<v1::CaseControlRequest>,
    ) -> Result<Response<v1::ControlReply>, Status> {
        control::case_control(&self.runtime, request, control::CaseOp::Rerun).await
    }
    async fn clear_cycle(
        &self,
        request: Request<v1::CycleControlRequest>,
    ) -> Result<Response<v1::ControlReply>, Status> {
        control::clear_cycle(&self.runtime, request).await
    }
    async fn update_cycle_policy(
        &self,
        request: Request<v1::UpdateCyclePolicyRequest>,
    ) -> Result<Response<v1::ControlReply>, Status> {
        control::update_cycle_policy(&self.runtime, request).await
    }
    async fn get_case(
        &self,
        request: Request<v1::GetCaseRequest>,
    ) -> Result<Response<v1::CaseDetail>, Status> {
        detail::get_case(self, request)
    }
    async fn get_execution(
        &self,
        request: Request<v1::GetExecutionRequest>,
    ) -> Result<Response<v1::ExecutionDetail>, Status> {
        detail::get_execution(self, request)
    }
    async fn list_case_parameters(
        &self,
        request: Request<v1::ListCaseParametersRequest>,
    ) -> Result<Response<v1::ListCaseParametersReply>, Status> {
        detail::list_case_parameters(self, request)
    }
    async fn list_execution_parameters(
        &self,
        request: Request<v1::ListExecutionParametersRequest>,
    ) -> Result<Response<v1::ListMaterializedParametersReply>, Status> {
        detail::list_execution_parameters(self, request)
    }
    async fn create_cycle(
        &self,
        request: Request<v1::CreateCycleRequest>,
    ) -> Result<Response<v1::CreateCycleReply>, Status> {
        let context = context(&request, request.get_ref().context.as_ref())?;
        let request = request.into_inner();
        let snapshot = self.runtime.watch_state().latest();
        let registration_id = context
            .idempotency_key
            .as_deref()
            .filter(|key| !key.is_empty())
            .map_or_else(dokime_app::core::CommandId::new, |key| {
                dokime_app::core::CommandId::from_seed(format!(
                    "selection:{}:{key}",
                    snapshot.state().identity.node.node_id
                ))
            });
        let plan =
            super::planning::resolve(&snapshot.state().suites, request.plan, &registration_id)?;
        let metadata = request
            .metadata
            .ok_or_else(|| Status::invalid_argument("cycle metadata is required"))?;
        value::text(&metadata.label)?;
        if metadata.label.trim().is_empty() {
            return Err(Status::invalid_argument("cycle label is required"));
        }
        if let Some(description) = &metadata.description {
            value::text(description)?;
        }
        value::text(&metadata.identifier)?;
        let metadata = value::decode_metadata(metadata)?;
        let id = self
            .runtime
            .start_cycle(
                dokime_app::transactions::CycleDefinition {
                    suite: plan.suite,
                    metadata,
                    cases: plan.cases,
                    continue_on_failure: request.continue_on_failure,
                },
                context,
            )
            .await
            .map_err(status)?;
        Ok(Response::new(v1::CreateCycleReply {
            cycle_id: id.to_string(),
            observed_cursor: Some(observation::cursor(&self.runtime.watch_state().latest())),
        }))
    }
    async fn append_planned_cases(
        &self,
        request: Request<v1::AppendPlannedCasesRequest>,
    ) -> Result<Response<v1::ControlReply>, Status> {
        let context = context(&request, request.get_ref().context.as_ref())?;
        let request = request.into_inner();
        let id = cycle_id(&request.cycle_id)?;
        let snapshot = self.runtime.watch_state().latest();
        let registration_id = context
            .idempotency_key
            .as_deref()
            .filter(|key| !key.is_empty())
            .map_or_else(dokime_app::core::CommandId::new, |key| {
                dokime_app::core::CommandId::from_seed(format!(
                    "selection:{}:{key}",
                    snapshot.state().identity.node.node_id
                ))
            });
        let plan =
            super::planning::resolve(&snapshot.state().suites, request.plan, &registration_id)?;
        self.runtime
            .append_cycle_templates(id, plan.suite, plan.cases, context)
            .await
            .map_err(status)?;
        Ok(Response::new(v1::ControlReply {
            observed_cursor: Some(observation::cursor(&self.runtime.watch_state().latest())),
        }))
    }
    async fn get_cycle(
        &self,
        request: Request<v1::GetCycleRequest>,
    ) -> Result<Response<v1::CycleSummary>, Status> {
        let id = cycle_id(&request.get_ref().cycle_id)?;
        let snapshot = self.runtime.watch_state().latest();
        let cycle = snapshot
            .state()
            .cycles
            .get(&id)
            .ok_or_else(|| Status::not_found("cycle not found"))?;
        Ok(Response::new(summary(
            snapshot.state(),
            cycle,
            observation::cursor(&snapshot),
        )?))
    }
    async fn list_cycles(
        &self,
        request: Request<v1::ListCyclesRequest>,
    ) -> Result<Response<v1::ListCyclesReply>, Status> {
        let request = request.into_inner();
        let snapshot = self.runtime.watch_state().latest();
        let limit = observation::page(
            request.limit,
            &request.after_cycle_id,
            request.cursor.as_ref(),
            &snapshot,
        )?;
        let filter = cycle_filter(request.filter)?;
        let order = cycle_order(request.order)?;
        let selected = select_cycles(snapshot.state(), &filter, order);
        let after = (!request.after_cycle_id.is_empty()).then(|| {
            // Reject the spelling before the membership check, so an invalid
            // UUID never reads as "filtered out".
            cycle_id(&request.after_cycle_id).map(|_| request.after_cycle_id.clone())
        });
        let after = after.transpose()?;
        let (page, next_after_cycle_id) =
            cycle_page(&selected, after.as_deref(), limit).map_err(|error| match error {
                CyclePageError::UnknownContinuation => Status::invalid_argument(error.to_string()),
            })?;
        let cursor = observation::cursor(&snapshot);
        let cycles = page
            .into_iter()
            .map(|cycle| summary(snapshot.state(), cycle, cursor.clone()))
            .collect::<Result<_, Status>>()?;
        Ok(Response::new(v1::ListCyclesReply {
            cycles,
            next_after_cycle_id,
            cursor: Some(cursor),
        }))
    }
    async fn list_cases(
        &self,
        request: Request<v1::ListCasesRequest>,
    ) -> Result<Response<v1::ListCasesReply>, Status> {
        let request = request.into_inner();
        let id = cycle_id(&request.cycle_id)?;
        let snapshot = self.runtime.watch_state().latest();
        let limit = observation::page(
            request.limit,
            &request.after_case_id,
            request.cursor.as_ref(),
            &snapshot,
        )?;
        let cycle = snapshot
            .state()
            .cycle_view(&id)
            .ok_or_else(|| Status::not_found("cycle not found"))?;
        Ok(Response::new(case_page(
            snapshot.state(),
            &cycle,
            &request.after_case_id,
            limit,
            observation::cursor(&snapshot),
        )?))
    }
    async fn list_executions(
        &self,
        request: Request<v1::ListExecutionsRequest>,
    ) -> Result<Response<v1::ListExecutionsReply>, Status> {
        let request = request.into_inner();
        let id = cycle_id(&request.cycle_id)?;
        let snapshot = self.runtime.watch_state().latest();
        let limit = observation::page(
            request.limit,
            &request.after_execution_id,
            request.cursor.as_ref(),
            &snapshot,
        )?;
        let cycle = snapshot
            .state()
            .cycles
            .get(&id)
            .ok_or_else(|| Status::not_found("cycle not found"))?;
        Ok(Response::new(execution_page(
            snapshot.state().executions_for_cycle(cycle),
            &request.after_execution_id,
            limit,
            observation::cursor(&snapshot),
        )?))
    }
    async fn run_cycle(
        &self,
        request: Request<v1::RunCycleRequest>,
    ) -> Result<Response<v1::RunReply>, Status> {
        let context = requests::context(
            &request,
            request.get_ref().context.as_ref(),
            request.get_ref().idempotency_key.as_deref(),
        )?;
        let id = cycle_id(&request.get_ref().cycle_id)?;
        let result = self.runtime.run_cycle_cases(id, context).await;
        Ok(Response::new(receipt(
            result,
            self.runtime.process_epoch().to_string(),
        )?))
    }
    async fn run_case(
        &self,
        request: Request<v1::RunCaseRequest>,
    ) -> Result<Response<v1::RunReply>, Status> {
        let context = requests::context(
            &request,
            request.get_ref().context.as_ref(),
            request.get_ref().idempotency_key.as_deref(),
        )?;
        let id = cycle_id(&request.get_ref().cycle_id)?;
        value::text(&request.get_ref().case_id)?;
        let case = CaseId::from_wire(request.get_ref().case_id.clone())
            .map_err(|_| Status::invalid_argument("invalid case ID"))?;
        let result = self.runtime.run_cycle_case(id, case, context).await;
        Ok(Response::new(receipt(
            result,
            self.runtime.process_epoch().to_string(),
        )?))
    }

    async fn run_execution(
        &self,
        request: Request<v1::RunExecutionRequest>,
    ) -> Result<Response<v1::RunReply>, Status> {
        let context = requests::context(
            &request,
            request.get_ref().context.as_ref(),
            request.get_ref().idempotency_key.as_deref(),
        )?;
        value::text(&request.get_ref().execution_id)?;
        let id =
            dokime_app::core::TestExecutionId::from_uuid(request.get_ref().execution_id.clone())
                .map_err(|_| Status::invalid_argument("invalid execution ID"))?;
        let mode = v1::ExecutionMode::try_from(request.get_ref().mode)
            .map_err(|_| Status::invalid_argument("unknown execution mode"))?;
        let mode = match mode {
            v1::ExecutionMode::Automatic => dokime_app::execution::ExecutionMode::Automatic,
            v1::ExecutionMode::Interactive => dokime_app::execution::ExecutionMode::Interactive,
        };
        let result = self.runtime.run_execution(id, mode, context).await;
        Ok(Response::new(receipt(
            result,
            self.runtime.process_epoch().to_string(),
        )?))
    }

    async fn pause_execution(
        &self,
        request: Request<v1::ExecutionControlRequest>,
    ) -> Result<Response<v1::ControlReply>, Status> {
        self.execution_control(request, CycleOp::Pause).await
    }

    async fn resume_execution(
        &self,
        request: Request<v1::ExecutionControlRequest>,
    ) -> Result<Response<v1::ControlReply>, Status> {
        self.execution_control(request, CycleOp::Resume).await
    }

    async fn stop_execution(
        &self,
        request: Request<v1::ExecutionControlRequest>,
    ) -> Result<Response<v1::ControlReply>, Status> {
        self.execution_control(request, CycleOp::Stop).await
    }

    async fn submit_execution_command(
        &self,
        request: Request<v1::SubmitExecutionCommandRequest>,
    ) -> Result<Response<codec::CommandReceipt>, Status> {
        commands::submit(&self.runtime, request)
            .await
            .map(Response::new)
    }

    async fn get_execution_command(
        &self,
        request: Request<v1::GetExecutionCommandRequest>,
    ) -> Result<Response<codec::CommandReceipt>, Status> {
        commands::get(&self.runtime, request)
            .await
            .map(Response::new)
    }

    async fn get_run_request(
        &self,
        request: Request<v1::GetRunRequestRequest>,
    ) -> Result<Response<codec::RunReceipt>, Status> {
        requests::get(&self.runtime, request)
            .await
            .map(Response::new)
    }

    async fn pause_cycle(
        &self,
        request: Request<v1::CycleControlRequest>,
    ) -> Result<Response<v1::ControlReply>, Status> {
        self.cycle_control(request, CycleOp::Pause).await
    }

    async fn resume_cycle(
        &self,
        request: Request<v1::CycleControlRequest>,
    ) -> Result<Response<v1::ControlReply>, Status> {
        self.cycle_control(request, CycleOp::Resume).await
    }

    async fn stop_cycle(
        &self,
        request: Request<v1::CycleControlRequest>,
    ) -> Result<Response<v1::ControlReply>, Status> {
        self.cycle_control(request, CycleOp::Stop).await
    }
}

impl Service {
    async fn execution_control(
        &self,
        request: Request<v1::ExecutionControlRequest>,
        op: CycleOp,
    ) -> Result<Response<v1::ControlReply>, Status> {
        let context = context(&request, request.get_ref().context.as_ref())?;
        value::text(&request.get_ref().execution_id)?;
        let id =
            dokime_app::core::TestExecutionId::from_uuid(request.get_ref().execution_id.clone())
                .map_err(|_| Status::invalid_argument("invalid execution ID"))?;
        match op {
            CycleOp::Pause => self.runtime.pause_execution(id, context).await,
            CycleOp::Resume => self.runtime.resume_execution(id, context).await,
            CycleOp::Stop => self.runtime.stop_execution(id, context).await,
        }
        .map_err(status)?;
        Ok(control::observed(&self.runtime))
    }

    async fn cycle_control(
        &self,
        request: Request<v1::CycleControlRequest>,
        op: CycleOp,
    ) -> Result<Response<v1::ControlReply>, Status> {
        let context = context(&request, request.get_ref().context.as_ref())?;
        let id = cycle_id(&request.get_ref().cycle_id)?;
        match op {
            CycleOp::Pause => self.runtime.pause_cycle(id, context).await,
            CycleOp::Resume => self.runtime.resume_cycle(id, context).await,
            CycleOp::Stop => self.runtime.stop_cycle(id, context).await,
        }
        .map_err(status)?;
        Ok(control::observed(&self.runtime))
    }
}

/* === Public Functions === */
pub(super) fn context<T>(
    request: &Request<T>,
    context: Option<&v1::MutationContext>,
) -> Result<CommandContext, Status> {
    let principal = Principal::of(request)?;
    let correlation_id = context
        .and_then(|context| context.correlation_id.as_ref())
        .map(|id| {
            CorrelationId::from_uuid(id.clone())
                .map_err(|_| Status::invalid_argument("invalid correlation UUID"))
        })
        .transpose()?;
    Ok(CommandContext {
        correlation_id,
        idempotency_key: None,
        submitted_by: Some(principal.subject().to_owned()),
    })
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
            value: detail.encode_to_vec(),
        }],
    };
    Status::with_details(grpc_code, detail.message, envelope.encode_to_vec().into())
}

pub(super) fn summary(
    state: &RuntimeState,
    cycle: &dokime_app::execution::TestCycle,
    cursor: v1::Cursor,
) -> Result<v1::CycleSummary, Status> {
    Ok(v1::CycleSummary {
        cycle_id: cycle.test_cycle_id.to_string(),
        metadata: Some(value::metadata(&cycle.metadata)?),
        status: match cycle.lifecycle {
            CycleStatus::Previewed => v1::CycleStatus::Previewed,
            CycleStatus::Ready => v1::CycleStatus::Ready,
            CycleStatus::Running => v1::CycleStatus::Running,
            CycleStatus::Paused => v1::CycleStatus::Paused,
            CycleStatus::Stopping => v1::CycleStatus::Stopping,
            CycleStatus::Completed => v1::CycleStatus::Completed,
            CycleStatus::Failed => v1::CycleStatus::Failed,
            CycleStatus::Blocked => v1::CycleStatus::Blocked,
            CycleStatus::Stopped => v1::CycleStatus::Stopped,
            CycleStatus::Cleared => v1::CycleStatus::Cleared,
        } as i32,
        case_count: state.cases_for_cycle(cycle).count() as u64,
        execution_count: cycle.execution_ids.len() as u64,
        continue_on_failure: cycle.continue_on_failure,
        observed_cursor: Some(cursor),
        created_at: Some(value::timestamp(&cycle.created_at)?),
        started_at: cycle
            .started_at
            .as_ref()
            .map(value::timestamp)
            .transpose()?,
        updated_at: Some(value::timestamp(&cycle.updated_at)?),
        result: cycle.result.as_ref().map(detail::result),
        suite: cycle_suite(state, cycle)?,
    })
}

pub(crate) fn receipt(receipt: CommandReceipt, epoch: String) -> Result<v1::RunReply, Status> {
    use v1::run_reply::Outcome;
    let outcome = match receipt.status {
        CommandStatus::Rejected => {
            let detail = receipt
                .rejection
                .ok_or_else(|| Status::internal("rejected receipt omitted reason"))?;
            Outcome::Rejected(rejection(
                detail.code,
                detail.message,
                detail.validation_issues,
            ))
        }
        CommandStatus::Accepted => {
            let at = time::OffsetDateTime::parse(
                receipt.accepted_at.as_str(),
                &time::format_description::well_known::Rfc3339,
            )
            .map_err(|_| Status::internal("invalid admission timestamp"))?;
            Outcome::Accepted(v1::Admission {
                command_id: receipt.command_id.to_string(),
                job_id: receipt.job_id.map(|id| id.to_string()),
                cursor: Some(v1::Cursor {
                    epoch,
                    stream: observation::STREAM.into(),
                    position: receipt.runtime_revision,
                }),
                accepted_at: Some(prost_types::Timestamp {
                    seconds: at.unix_timestamp(),
                    nanos: at.nanosecond() as i32,
                }),
            })
        }
    };
    Ok(v1::RunReply {
        outcome: Some(outcome),
    })
}

pub(super) fn case_summary(
    case: &TestCase,
    ordinal: usize,
    projection: CaseProjection,
) -> Result<v1::CaseSummary, Status> {
    value::text(&case.suite_id)?;
    value::projection::metadata_fields_count(&case.metadata)
        .map_err(|_| Status::resource_exhausted("case metadata construction exceeds its bound"))?;
    Ok(v1::CaseSummary {
        suite_id: case.suite_id.clone(),
        case_id: case.test_case_id.as_str().to_owned(),
        template_id: case.test_case_template_id.as_str().to_owned(),
        metadata: Some(value::projection::metadata(&case.metadata).map_err(|_| {
            Status::resource_exhausted("case metadata construction exceeds its bound")
        })?),
        ordinal: ordinal as u64,
        status: projection
            .status
            .map_or(v1::CaseStatus::Unspecified, case_status) as i32,
        latest_execution_id: projection
            .latest_execution_id
            .as_ref()
            .map(|id| id.as_str().to_owned()),
        latest_execution_status: projection
            .latest_execution_status
            .as_ref()
            .map(|status| execution_status(status) as i32),
        attempt_count: projection.attempt_count,
        rerun_scheduled: projection.rerun_scheduled,
        actions: Some(v1::CaseActions {
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
) -> Result<v1::CaseSummary, Status> {
    case_summary(
        case,
        ordinal,
        historical_case_projection(cycle, &case.test_case_id),
    )
}

/* === Private Functions === */
fn case_status(status: CaseStatus) -> v1::CaseStatus {
    match status {
        CaseStatus::Skipped => v1::CaseStatus::Skipped,
        CaseStatus::Invalidated => v1::CaseStatus::Invalidated,
        CaseStatus::Planned => v1::CaseStatus::Planned,
        CaseStatus::Queued => v1::CaseStatus::Queued,
        CaseStatus::Running => v1::CaseStatus::Running,
        CaseStatus::Passed => v1::CaseStatus::Passed,
        CaseStatus::Failed => v1::CaseStatus::Failed,
        CaseStatus::Blocked => v1::CaseStatus::Blocked,
        CaseStatus::Cancelled => v1::CaseStatus::Cancelled,
        CaseStatus::Interrupted => v1::CaseStatus::Interrupted,
    }
}

fn execution_status(status: &ExecutionStatus) -> v1::ExecutionStatus {
    match status {
        ExecutionStatus::Invalidated => v1::ExecutionStatus::Invalidated,
        ExecutionStatus::Skipped => v1::ExecutionStatus::Skipped,
        ExecutionStatus::Pending => v1::ExecutionStatus::Pending,
        ExecutionStatus::Running => v1::ExecutionStatus::Running,
        ExecutionStatus::Passed => v1::ExecutionStatus::Passed,
        ExecutionStatus::Failed => v1::ExecutionStatus::Failed,
        ExecutionStatus::Blocked => v1::ExecutionStatus::Blocked,
        ExecutionStatus::Cancelled => v1::ExecutionStatus::Cancelled,
        ExecutionStatus::Interrupted => v1::ExecutionStatus::Interrupted,
    }
}

fn cycle_filter(filter: Option<v1::CycleFilter>) -> Result<CycleFilter, Status> {
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
        text: filter.text,
        statuses: filter
            .statuses
            .into_iter()
            .map(cycle_status)
            .collect::<Result<_, Status>>()?,
        suite_ids: filter.suite_ids,
        created_after: filter
            .created_after
            .map(value::decoded_timestamp)
            .transpose()?,
        created_before: filter
            .created_before
            .map(value::decoded_timestamp)
            .transpose()?,
    })
}

fn cycle_order(order: Option<v1::CycleOrder>) -> Result<CycleOrder, Status> {
    let Some(order) = order else {
        return Ok(CycleOrder::default());
    };
    let key = match v1::CycleOrderKey::try_from(order.key)
        .map_err(|_| Status::invalid_argument("unknown cycle order key"))?
    {
        v1::CycleOrderKey::Unspecified | v1::CycleOrderKey::UpdatedAt => CycleOrderKey::UpdatedAt,
        v1::CycleOrderKey::CreatedAt => CycleOrderKey::CreatedAt,
        v1::CycleOrderKey::Label => CycleOrderKey::Label,
    };
    Ok(CycleOrder {
        key,
        descending: order.descending,
    })
}

fn cycle_status(status: i32) -> Result<CycleStatus, Status> {
    match v1::CycleStatus::try_from(status)
        .map_err(|_| Status::invalid_argument("unknown cycle status"))?
    {
        v1::CycleStatus::Unspecified => Err(Status::invalid_argument("unspecified cycle status")),
        v1::CycleStatus::Previewed => Ok(CycleStatus::Previewed),
        v1::CycleStatus::Ready => Ok(CycleStatus::Ready),
        v1::CycleStatus::Running => Ok(CycleStatus::Running),
        v1::CycleStatus::Paused => Ok(CycleStatus::Paused),
        v1::CycleStatus::Stopping => Ok(CycleStatus::Stopping),
        v1::CycleStatus::Completed => Ok(CycleStatus::Completed),
        v1::CycleStatus::Failed => Ok(CycleStatus::Failed),
        v1::CycleStatus::Blocked => Ok(CycleStatus::Blocked),
        v1::CycleStatus::Stopped => Ok(CycleStatus::Stopped),
        v1::CycleStatus::Cleared => Ok(CycleStatus::Cleared),
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
) -> v1::Rejection {
    v1::Rejection {
        code,
        message,
        issues: validation_issues
            .into_iter()
            .map(|issue| v1::ValidationIssue {
                path: issue.path,
                message: issue.message,
                expected: issue.expected,
            })
            .collect(),
    }
}

fn cycle_suite(
    state: &RuntimeState,
    cycle: &dokime_app::execution::TestCycle,
) -> Result<Option<v1::Metadata>, Status> {
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
    suites
        .all(|suite| suite.identifier == first.identifier)
        .then(|| value::metadata(first))
        .transpose()
}

fn case_page(
    state: &RuntimeState,
    cycle: &CycleView,
    after: &str,
    limit: usize,
    cursor: v1::Cursor,
) -> Result<v1::ListCasesReply, Status> {
    let start = after_position(after, cycle.cases().map(|case| case.test_case_id.as_str()))?;
    let cases: Vec<_> = cycle
        .cases()
        .enumerate()
        .skip(start)
        .take(limit)
        .map(|(ordinal, case)| {
            case_summary(
                case,
                ordinal,
                case_projection(state, cycle, &case.test_case_id),
            )
        })
        .collect::<Result<_, Status>>()?;
    let next_after_case_id = (start + cases.len() < cycle.cases().count())
        .then(|| cases.last().unwrap().case_id.clone());
    Ok(v1::ListCasesReply {
        cases,
        next_after_case_id,
        cursor: Some(cursor),
    })
}

fn execution_page<'a>(
    members: impl ExactSizeIterator<Item = &'a dokime_app::execution::TestExecution> + Clone,
    after: &str,
    limit: usize,
    cursor: v1::Cursor,
) -> Result<v1::ListExecutionsReply, Status> {
    let start = after_position(
        after,
        members
            .clone()
            .map(|execution| execution.test_execution_id.as_str()),
    )?;
    let execution_count = members.len();
    let executions: Vec<_> = members
        .enumerate()
        .skip(start)
        .take(limit)
        .map(|(ordinal, execution)| execution_summary(execution, ordinal))
        .collect();
    let next_after_execution_id = (start + executions.len() < execution_count)
        .then(|| executions.last().unwrap().execution_id.clone());
    Ok(v1::ListExecutionsReply {
        executions,
        next_after_execution_id,
        cursor: Some(cursor),
    })
}

fn execution_summary(
    execution: &dokime_app::execution::TestExecution,
    ordinal: usize,
) -> v1::ExecutionSummary {
    v1::ExecutionSummary {
        execution_id: execution.test_execution_id.to_string(),
        case_id: execution.test_case_id.to_string(),
        status: execution_status(&execution.status) as i32,
        ordinal: ordinal as u64,
    }
}

/* === Tests === */
