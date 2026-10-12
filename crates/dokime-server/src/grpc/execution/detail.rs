/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */
pub(super) use super::value::checked_metadata as metadata;

/* === Imports === */
// Standard Library Imports
use std::collections::BTreeMap;
// Third Party Imports

// Local Imports
use super::*;
use dokime_app::core::{Metadata, TestExecutionId, Value};
use dokime_app::definition::{MaterializedParameter, ParameterSource};
use dokime_app::execution::TestExecution;

/* === Constants === */
const MAX_DIAGNOSTIC_TEXT: usize = 4096;
const DIAGNOSTIC_TRUNCATED: &str = "\n… truncated";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(super) struct Bounds {
    remaining: usize,
    nodes: value::Budget,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Bounds {
    pub(super) fn new(limit: usize) -> Self {
        Self {
            remaining: limit.min(4 * 1024 * 1024),
            nodes: value::Budget::new(),
        }
    }
    pub(super) fn bytes(&mut self, count: usize) -> Result<(), Status> {
        self.remaining = self
            .remaining
            .checked_sub(count)
            .ok_or_else(|| Status::resource_exhausted("inspection allocation budget exceeded"))?;
        Ok(())
    }
    pub(super) fn text(&mut self, text: &str) -> Result<(), Status> {
        value::text(text)?;
        self.bytes(text.len().saturating_add(32))
    }
    pub(super) fn value(&mut self, item: &Value, depth: usize) -> Result<(), Status> {
        self.nodes.node(depth)?;
        self.bytes(32)?;
        match item {
            Value::Text(text) => self.text(text)?,
            Value::Array(values) => {
                value::collection(values.len())?;
                for item in values {
                    self.value(item, depth + 1)?;
                }
            }
            Value::Object(values) => {
                value::collection(values.len())?;
                for (key, item) in values {
                    self.text(key)?;
                    self.value(item, depth + 1)?;
                }
            }
            _ => (),
        }
        Ok(())
    }
}

/* === Public Functions === */
pub(super) fn execution<'a>(
    state: &'a dokime_app::state::RuntimeState,
    id: &str,
) -> Result<
    (
        &'a dokime_app::execution::TestCycle,
        usize,
        &'a TestExecution,
    ),
    Status,
> {
    let id = TestExecutionId::from_uuid(id.to_owned())
        .map_err(|_| Status::invalid_argument("invalid execution UUID"))?;
    let execution = state
        .executions
        .get(&id)
        .ok_or_else(|| Status::not_found("execution not found in current runtime"))?;
    let cycle = state
        .cycles
        .get(&execution.test_cycle_id)
        .ok_or_else(|| Status::failed_precondition("execution owner is unavailable"))?;
    let ordinal = cycle
        .execution_ids
        .iter()
        .position(|item| item == &id)
        .ok_or_else(|| Status::failed_precondition("execution owner membership is unavailable"))?;
    Ok((cycle, ordinal, execution))
}

pub(crate) fn result(value: &dokime_app::execution::ResultSummary) -> v1::ResultSummary {
    use dokime_app::execution::ResultStatus;
    v1::ResultSummary {
        status: match value.status {
            ResultStatus::Pending => v1::ResultStatus::Pending,
            ResultStatus::Passed => v1::ResultStatus::Passed,
            ResultStatus::Failed => v1::ResultStatus::Failed,
            ResultStatus::Blocked => v1::ResultStatus::Blocked,
            ResultStatus::Cancelled => v1::ResultStatus::Cancelled,
        } as i32,
        passed: value.passed,
        failed: value.failed,
        blocked: value.blocked,
        skipped: value.skipped,
    }
}

pub(super) fn get_case(
    service: &Service,
    request: Request<v1::GetCaseRequest>,
) -> Result<Response<v1::CaseDetail>, Status> {
    let request = request.into_inner();
    let id = cycle_id(&request.cycle_id)?;
    super::value::text(&request.case_id)?;
    let snapshot = service.runtime.watch_state().latest();
    let cycle = snapshot
        .state()
        .cycle_view(&id)
        .ok_or_else(|| Status::not_found("cycle not found"))?;
    let (ordinal, case) = cycle
        .cases()
        .enumerate()
        .find(|(_, case)| case.test_case_id.as_str() == request.case_id)
        .ok_or_else(|| Status::not_found("case not found in requested cycle"))?;
    reply(
        service,
        v1::CaseDetail {
            summary: Some(super::case_summary(
                case,
                ordinal,
                dokime_app::runtime::query::case_projection(
                    snapshot.state(),
                    &cycle,
                    &case.test_case_id,
                ),
            )?),
            suite: case.suite.as_ref().map(metadata).transpose()?,
            parameter_count: case.parameters.len() as u64,
            generation_index: case.generation_index,
            observed_cursor: Some(observation::cursor(&snapshot)),
        },
    )
}

pub(super) fn get_execution(
    service: &Service,
    request: Request<v1::GetExecutionRequest>,
) -> Result<Response<v1::ExecutionDetail>, Status> {
    let snapshot = service.runtime.watch_state().latest();
    let (cycle, ordinal, execution) = execution(snapshot.state(), &request.get_ref().execution_id)?;
    let summary = Some(execution_summary(execution, ordinal));
    reply(
        service,
        v1::ExecutionDetail {
            summary,
            cycle_id: cycle.test_cycle_id.to_string(),
            scope: Some(value::scope(&execution.scope)),
            admitted_suite: execution.snapshot.suite().map(metadata).transpose()?,
            admitted_template_id: execution.snapshot.template_id().map(ToString::to_string),
            admitted_template: execution
                .snapshot
                .template()
                .map(|template| metadata(&template.metadata))
                .transpose()?,
            admitted_plugin: execution
                .snapshot
                .plugin()
                .map(value::execution_plugin)
                .transpose()?,
            invalidation: execution
                .invalidation
                .as_ref()
                .map(|invalidation| {
                    Ok::<_, Status>(v1::ExecutionInvalidation {
                        previous: Some(value::execution_plugin(&invalidation.previous)?),
                        current: invalidation
                            .current
                            .as_ref()
                            .map(value::execution_plugin)
                            .transpose()?,
                    })
                })
                .transpose()?,
            started_at: execution
                .started_at
                .as_ref()
                .map(value::timestamp)
                .transpose()?,
            finished_at: execution
                .finished_at
                .as_ref()
                .map(value::timestamp)
                .transpose()?,
            result: execution.result.as_ref().map(result),
            input_parameter_count: execution.snapshot.parameters().len() as u64,
            controls: dokime_app::runtime::query::execution_controls(
                snapshot.state(),
                &execution.test_execution_id,
            )
            .map(|controls| v1::ExecutionControls {
                request_id: controls.request_id.to_string(),
                mode: match controls.mode {
                    dokime_app::execution::ExecutionMode::Automatic => {
                        v1::ExecutionMode::Automatic as i32
                    }
                    dokime_app::execution::ExecutionMode::Interactive => {
                        v1::ExecutionMode::Interactive as i32
                    }
                },
                paused: controls.paused,
                stop_requested: controls.stop_requested,
                can_pause: controls.can_pause,
                can_resume: controls.can_resume,
                can_stop: controls.can_stop,
            }),
            output_parameter_count: execution.output_parameters.len() as u64,
            step_count: execution.steps.len() as u64,
            observed_cursor: Some(observation::cursor(&snapshot)),
        },
    )
}

pub(super) fn list_execution_parameters(
    service: &Service,
    request: Request<v1::ListExecutionParametersRequest>,
) -> Result<Response<v1::ListMaterializedParametersReply>, Status> {
    let request = request.into_inner();
    let snapshot = service.runtime.watch_state().latest();
    let (_, _, execution) = execution(snapshot.state(), &request.execution_id)?;
    let selected = match request.set() {
        v1::ExecutionParameterSet::Inputs => execution.snapshot.parameters(),
        v1::ExecutionParameterSet::Outputs => &execution.output_parameters,
        v1::ExecutionParameterSet::Unspecified => {
            return Err(Status::invalid_argument(
                "explicit input/output parameter set is required",
            ));
        }
    };
    if v1::ExecutionParameterSet::try_from(request.set).is_err() {
        return Err(Status::invalid_argument("unknown parameter set"));
    }
    let selection = match request.set() {
        v1::ExecutionParameterSet::Inputs => {
            v1::parameter_cursor::Selection::ExecutionInputsId(request.execution_id.clone())
        }
        v1::ExecutionParameterSet::Outputs => {
            v1::parameter_cursor::Selection::ExecutionOutputsId(request.execution_id.clone())
        }
        v1::ExecutionParameterSet::Unspecified => unreachable!(),
    };
    let limit = parameter_page(
        request.limit,
        &request.after_name,
        request.cursor.as_ref(),
        &snapshot,
        &selection,
    )?;
    parameters(
        service,
        selected,
        &request.after_name,
        limit,
        parameter_cursor(&snapshot, selection),
    )
}

pub(super) fn metadata_len(value: &Metadata) -> Result<usize, Status> {
    Ok(metadata(value)?.encoded_len())
}

pub(super) fn diagnostic(
    value: &dokime_app::plugin::Diagnostic,
    bounds: &mut Bounds,
) -> Result<v1::Diagnostic, Status> {
    let message = bounded(&value.message, bounds)?;
    let chain = value
        .chain
        .iter()
        .map(|cause| bounded(cause, bounds))
        .collect::<Result<Vec<_>, Status>>()?;
    let location = bounded(&value.location, bounds)?;
    let span_trace = value
        .span_trace
        .iter()
        .map(|span| bounded(span, bounds))
        .collect::<Result<Vec<_>, Status>>()?;
    let backtrace = bounded(&value.backtrace, bounds)?;

    Ok(v1::Diagnostic {
        message,
        chain,
        location,
        span_trace,
        backtrace,
    })
}

pub(super) fn list_case_parameters(
    service: &Service,
    request: Request<v1::ListCaseParametersRequest>,
) -> Result<Response<v1::ListCaseParametersReply>, Status> {
    let request = request.into_inner();
    let id = cycle_id(&request.cycle_id)?;
    value::text(&request.case_id)?;
    let snapshot = service.runtime.watch_state().latest();
    let selection = v1::parameter_cursor::Selection::PlannedCase(v1::PlannedCaseParameters {
        cycle_id: request.cycle_id.clone(),
        case_id: request.case_id.clone(),
    });
    let limit = parameter_page(
        request.limit,
        &request.after_name,
        request.cursor.as_ref(),
        &snapshot,
        &selection,
    )?;
    let cycle = snapshot
        .state()
        .cycle_view(&id)
        .ok_or_else(|| Status::not_found("cycle not found"))?;
    let case = cycle
        .cases()
        .find(|case| case.test_case_id.as_str() == request.case_id)
        .ok_or_else(|| Status::not_found("case not found in requested cycle"))?;
    case_parameters(
        service,
        &case.parameters,
        &request.after_name,
        limit,
        parameter_cursor(&snapshot, selection),
    )
}

/* === Private Functions === */
fn reply<T: Message>(service: &Service, value: T) -> Result<Response<T>, Status> {
    if value.encoded_len() > service.max_message_bytes {
        return Err(Status::resource_exhausted(
            "inspection exceeds response budget",
        ));
    }
    Ok(Response::new(value))
}

fn parameters(
    service: &Service,
    entries: &BTreeMap<String, MaterializedParameter>,
    after: &str,
    limit: usize,
    cursor: v1::ParameterCursor,
) -> Result<Response<v1::ListMaterializedParametersReply>, Status> {
    if !after.is_empty() && !entries.contains_key(after) {
        return Err(Status::invalid_argument(
            "after name not found in requested parameter set",
        ));
    }
    let mut selected = entries.iter().filter(|(name, _)| name.as_str() > after);
    let mut bounds = Bounds::new(service.max_message_bytes);
    let mut budget = value::Budget::new();
    let parameters = selected
        .by_ref()
        .take(limit)
        .map(|(name, item)| {
            bounds.text(name)?;
            bounds.value(&item.value, 0)?;
            let metadata = metadata(&item.metadata)?;
            bounds.bytes(metadata.encoded_len())?;
            if let Some(group) = &item.group_id {
                bounds.text(group.as_str())?;
            }
            let unit = item
                .unit
                .as_ref()
                .map(|unit| {
                    bounds.text(&unit.symbol)?;
                    bounds.bytes(metadata_len(&unit.metadata)?)?;
                    super::super::catalog::unit(unit)
                })
                .transpose()?;
            Ok(v1::MaterializedParameter {
                name: name.clone(),
                metadata: Some(metadata),
                group_id: item.group_id.as_ref().map(ToString::to_string),
                source: match item.source {
                    ParameterSource::Definition => v1::ParameterSource::Definition,
                    ParameterSource::Traceability => v1::ParameterSource::Traceability,
                    ParameterSource::Environment => v1::ParameterSource::Environment,
                    ParameterSource::Resource => v1::ParameterSource::Resource,
                    ParameterSource::Operator => v1::ParameterSource::Operator,
                    ParameterSource::Runtime => v1::ParameterSource::Runtime,
                    ParameterSource::Telemetry => v1::ParameterSource::Telemetry,
                    ParameterSource::Record => v1::ParameterSource::Record,
                    ParameterSource::Output => v1::ParameterSource::Output,
                } as i32,
                value: Some(value::encode(&item.value, &mut budget, 0)?),
                unit,
            })
        })
        .collect::<Result<Vec<_>, Status>>()?;
    let next_after_name = selected
        .next()
        .is_some()
        .then(|| parameters.last().unwrap().name.clone());
    reply(
        service,
        v1::ListMaterializedParametersReply {
            parameters,
            next_after_name,
            cursor: Some(cursor),
        },
    )
}

fn bounded(value: &str, bounds: &mut Bounds) -> Result<String, Status> {
    let mut cut = MAX_DIAGNOSTIC_TEXT.saturating_sub(DIAGNOSTIC_TRUNCATED.len());
    let value = if value.len() <= MAX_DIAGNOSTIC_TEXT {
        value.to_owned()
    } else {
        while !value.is_char_boundary(cut) {
            cut -= 1;
        }
        format!("{}{DIAGNOSTIC_TRUNCATED}", &value[..cut])
    };
    bounds.text(&value)?;
    Ok(value)
}

fn parameter_cursor(
    snapshot: &dokime_app::runtime::query::RuntimeStateSnapshot,
    selection: v1::parameter_cursor::Selection,
) -> v1::ParameterCursor {
    v1::ParameterCursor {
        observed: Some(observation::cursor(snapshot)),
        selection: Some(selection),
    }
}

fn parameter_page(
    limit: u32,
    after: &str,
    cursor: Option<&v1::ParameterCursor>,
    snapshot: &dokime_app::runtime::query::RuntimeStateSnapshot,
    selection: &v1::parameter_cursor::Selection,
) -> Result<usize, Status> {
    if let Some(cursor) = cursor {
        if cursor.selection.as_ref() != Some(selection) {
            return Err(Status::invalid_argument(
                "parameter cursor selection mismatch",
            ));
        }
        if cursor.observed.is_none() {
            return Err(Status::invalid_argument(
                "parameter cursor requires observed revision",
            ));
        }
    }
    observation::page(
        limit,
        after,
        cursor.and_then(|cursor| cursor.observed.as_ref()),
        snapshot,
    )
}

fn case_parameters(
    service: &Service,
    entries: &BTreeMap<String, Value>,
    after: &str,
    limit: usize,
    cursor: v1::ParameterCursor,
) -> Result<Response<v1::ListCaseParametersReply>, Status> {
    if !after.is_empty() && !entries.contains_key(after) {
        return Err(Status::invalid_argument(
            "after name not found in requested case",
        ));
    }
    let mut selected = entries
        .iter()
        .filter(|(name, _)| after.is_empty() || name.as_str() > after);
    let mut bounds = Bounds::new(service.max_message_bytes);
    let mut budget = value::Budget::new();
    let parameters = selected
        .by_ref()
        .take(limit)
        .map(|(name, item)| {
            bounds.text(name)?;
            bounds.value(item, 0)?;
            Ok(v1::CaseParameter {
                name: name.clone(),
                value: Some(value::encode(item, &mut budget, 0)?),
            })
        })
        .collect::<Result<Vec<_>, Status>>()?;
    let next_after_name = selected
        .next()
        .is_some()
        .then(|| parameters.last().unwrap().name.clone());
    reply(
        service,
        v1::ListCaseParametersReply {
            parameters,
            next_after_name,
            cursor: Some(cursor),
        },
    )
}

/* === Tests === */
