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

pub(super) use super::value::canonical::metadata;

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports

// Local Imports
use super::*;
use dokime::definition::{MaterializedParameter, ParameterSource};
use dokime::domain::model::execution::result::TestExecution;
use dokime::domain::model::execution::wire as execution_model;
use dokime::domain::model::identity::TestExecutionId;
use dokime::domain::model::identity::wire::Diagnostic;
use dokime::domain::model::values::Value;

/* === Constants === */

const SERVER_DIAGNOSTIC_TEXT_BYTES: u64 = 4_096;

const DIAGNOSTIC_TRUNCATED: &str = "\n… truncated";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn execution<'a>(
    state: &'a dokime::state::RuntimeState, id: &str,
) -> Result<(&'a dokime::domain::model::execution::result::TestCycle, usize, &'a TestExecution), Status> {
    let id = TestExecutionId::from_uuid(id.to_owned()).map_err(|_| Status::invalid_argument("invalid execution UUID"))?;
    let execution = state.executions.get(&id).ok_or_else(|| Status::not_found("execution not found in current runtime"))?;
    let cycle = state.cycles.get(&execution.test_cycle_id).ok_or_else(|| Status::failed_precondition("execution owner is unavailable"))?;
    let ordinal = cycle
        .execution_ids
        .iter()
        .position(|item| item == &id)
        .ok_or_else(|| Status::failed_precondition("execution owner membership is unavailable"))?;
    Ok((cycle, ordinal, execution))
}

pub(crate) fn result(
    value: &dokime::domain::model::execution::result::ResultSummary,
) -> dokime::domain::model::execution::wire::ResultSummary {
    use dokime::domain::model::execution::result::ResultStatus;
    dokime::domain::model::execution::wire::ResultSummary {
        status: match value.status {
            ResultStatus::Pending => dokime::domain::model::execution::wire::ResultStatus::Pending,
            ResultStatus::Passed => dokime::domain::model::execution::wire::ResultStatus::Passed,
            ResultStatus::Failed => dokime::domain::model::execution::wire::ResultStatus::Failed,
            ResultStatus::Blocked => dokime::domain::model::execution::wire::ResultStatus::Blocked,
            ResultStatus::Cancelled => dokime::domain::model::execution::wire::ResultStatus::Cancelled,
        },
        passed: value.passed,
        failed: value.failed,
        blocked: value.blocked,
        skipped: value.skipped,
    }
}

pub(super) fn get_case(service: &Resources, request: &wire::GetCase) -> Result<execution_model::CaseDetail, Status> {
    let id = cycle_id(&request.cycle_id)?;
    let snapshot = service.runtime.watch_state().latest();
    let cycle = snapshot.state().cycle_view(&id).ok_or_else(|| Status::not_found("cycle not found"))?;
    let (ordinal, case) = cycle
        .cases()
        .enumerate()
        .find(|(_, case)| case.test_case_id.as_str() == request.case_id)
        .ok_or_else(|| Status::not_found("case not found in requested cycle"))?;

    reply(
        service,
        execution_model::CaseDetail {
            summary: Some(super::case_summary(
                case,
                ordinal,
                dokime::runtime::query::case_projection(snapshot.state(), &cycle, &case.test_case_id),
            )?),
            suite: case.suite.as_ref().map(metadata).transpose()?,
            parameter_count: case.parameters.len() as u64,
            generation_index: case.generation_index,
            observed_cursor: Some(canonical::cursor(&snapshot)),
        },
    )
}

pub(super) fn get_execution(service: &Resources, request: &wire::GetExecution) -> Result<execution_model::ExecutionDetail, Status> {
    let snapshot = service.runtime.watch_state().latest();
    let (cycle, ordinal, execution) = execution(snapshot.state(), &request.execution_id)?;

    let summary = Some(execution_summary(snapshot.state(), execution, ordinal));
    reply(
        service,
        execution_model::ExecutionDetail {
            summary,
            cycle_id: cycle.test_cycle_id.to_string(),
            scope: Some(canonical::scope(&execution.scope)?),
            admitted_suite: execution.snapshot.suite().map(metadata).transpose()?,
            admitted_template_id: execution.snapshot.template_id().map(ToString::to_string),
            admitted_template: execution.snapshot.template().map(|template| metadata(&template.metadata)).transpose()?,
            admitted_plugin: execution.snapshot.plugin().map(canonical::execution_plugin).transpose()?,
            invalidation: execution
                .invalidation
                .as_ref()
                .map(|invalidation| {
                    Ok::<_, Status>(execution_model::ExecutionInvalidation {
                        previous: Some(canonical::execution_plugin(&invalidation.previous)?),
                        current: invalidation.current.as_ref().map(canonical::execution_plugin).transpose()?,
                    })
                })
                .transpose()?,
            started_at: execution.started_at.as_ref().map(canonical::timestamp).transpose()?,
            finished_at: execution.finished_at.as_ref().map(canonical::timestamp).transpose()?,
            result: execution.result.as_ref().map(result),
            input_parameter_count: execution.snapshot.parameters().len() as u64,
            controls: dokime::runtime::query::execution_controls(snapshot.state(), &execution.test_execution_id).map(|controls| {
                execution_model::ExecutionControls {
                    request_id: controls.request_id.map(|id| id.to_string()),
                    mode: match controls.mode {
                        dokime::domain::model::execution::ExecutionMode::Automatic => execution_model::ExecutionMode::Automatic,
                        dokime::domain::model::execution::ExecutionMode::Interactive => execution_model::ExecutionMode::Interactive,
                    },
                    paused: controls.paused,
                    stop_requested: controls.stop_requested,
                    can_pause: controls.can_pause,
                    can_resume: controls.can_resume,
                    can_stop: controls.can_stop,
                }
            }),
            output_parameter_count: execution.output_parameters.len() as u64,
            step_count: execution.steps.len() as u64,
            observed_cursor: Some(canonical::cursor(&snapshot)),
        },
    )
}

pub(super) fn list_execution_parameters(
    service: &Resources, request: &wire::ListExecutionParameters,
) -> Result<execution_model::ListMaterializedParametersReply, Status> {
    let snapshot = service.runtime.watch_state().latest();
    let (_, _, execution) = execution(snapshot.state(), &request.execution_id)?;
    let selected = match request.set {
        execution_model::ExecutionParameterSet::Inputs => execution.snapshot.parameters(),
        execution_model::ExecutionParameterSet::Outputs => &execution.output_parameters,
        execution_model::ExecutionParameterSet::Unspecified => {
            return Err(Status::invalid_argument("explicit input/output parameter set is required"));
        }
    };
    let selection = match request.set {
        execution_model::ExecutionParameterSet::Inputs => {
            execution_model::ParameterCursorSelection::ExecutionInputsId(request.execution_id.clone())
        }
        execution_model::ExecutionParameterSet::Outputs => {
            execution_model::ParameterCursorSelection::ExecutionOutputsId(request.execution_id.clone())
        }
        execution_model::ExecutionParameterSet::Unspecified => unreachable!(),
    };
    let limit = parameter_page(request.limit, &request.after_name, request.cursor.as_ref(), &snapshot, &selection)?;
    parameters(service, selected, &request.after_name, limit, parameter_cursor(&snapshot, selection))
}

pub(super) fn diagnostic(value: &dokime::plugin::Diagnostic) -> Result<Diagnostic, Status> {
    let message = bounded(&value.message)?;
    let mut chain = canonical::slots(value.chain.len())?;
    for cause in &value.chain {
        chain.push(bounded(cause)?);
    }
    let location = bounded(&value.location)?;
    let mut span_trace = canonical::slots(value.span_trace.len())?;
    for span in &value.span_trace {
        span_trace.push(bounded(span)?);
    }
    let backtrace = bounded(&value.backtrace)?;

    Ok(Diagnostic {
        message,
        chain,
        location,
        span_trace,
        backtrace,
    })
}

pub(super) fn list_case_parameters(
    service: &Resources, request: &wire::ListCaseParameters,
) -> Result<execution_model::ListCaseParametersReply, Status> {
    let id = cycle_id(&request.cycle_id)?;
    let snapshot = service.runtime.watch_state().latest();
    let selection = execution_model::ParameterCursorSelection::PlannedCase(execution_model::PlannedCaseParameters {
        cycle_id: request.cycle_id.clone(),
        case_id: request.case_id.clone(),
    });
    let limit = parameter_page(request.limit, &request.after_name, request.cursor.as_ref(), &snapshot, &selection)?;
    let cycle = snapshot.state().cycle_view(&id).ok_or_else(|| Status::not_found("cycle not found"))?;
    let case = cycle
        .cases()
        .find(|case| case.test_case_id.as_str() == request.case_id)
        .ok_or_else(|| Status::not_found("case not found in requested cycle"))?;
    case_parameters(service, &case.parameters, &request.after_name, limit, parameter_cursor(&snapshot, selection))
}

/* === Private Functions === */

fn reply<T: dokime::harmos::domain::Portable>(service: &Resources, value: T) -> Result<T, Status> {
    if value.wire_len()? > service.max_message_bytes {
        return Err(Status::resource_exhausted("inspection exceeds response budget"));
    }
    Ok(value)
}

fn parameters(
    service: &Resources, entries: &BTreeMap<String, MaterializedParameter>, after: &str, limit: usize,
    cursor: execution_model::ParameterCursor,
) -> Result<execution_model::ListMaterializedParametersReply, Status> {
    if !after.is_empty() && !entries.contains_key(after) {
        return Err(Status::invalid_argument("after name not found in requested parameter set"));
    }
    let mut selected = entries.iter().filter(|(name, _)| name.as_str() > after);
    let mut budget = value::Budget::new()?;
    let count = selected.clone().take(limit).count();

    let mut parameters = canonical::slots(count)?;
    for (name, item) in selected.by_ref().take(limit) {
        let metadata = metadata(&item.metadata)?;

        let unit = item.unit.as_ref().map(super::super::catalog::unit).transpose()?;
        parameters.push(execution_model::MaterializedParameter {
            name: name.clone(),
            metadata: Some(metadata),
            group_id: item.group_id.as_ref().map(ToString::to_string),
            source: match item.source {
                ParameterSource::Definition => execution_model::ParameterSource::Definition,
                ParameterSource::Traceability => execution_model::ParameterSource::Traceability,
                ParameterSource::Environment => execution_model::ParameterSource::Environment,
                ParameterSource::Resource => execution_model::ParameterSource::Resource,
                ParameterSource::Operator => execution_model::ParameterSource::Operator,
                ParameterSource::Runtime => execution_model::ParameterSource::Runtime,
                ParameterSource::Telemetry => execution_model::ParameterSource::Telemetry,
                ParameterSource::Record => execution_model::ParameterSource::Record,
                ParameterSource::Output => execution_model::ParameterSource::Output,
            },
            value: Some(canonical::encode(&item.value, &mut budget, 0)?),
            unit,
        });
    }
    let next_after_name = selected.next().is_some().then(|| parameters.last().unwrap().name.clone());
    reply(
        service,
        execution_model::ListMaterializedParametersReply {
            parameters,
            next_after_name,
            cursor: Some(cursor),
        },
    )
}

fn bounded(value: &str) -> Result<String, Status> {
    let maximum = SERVER_DIAGNOSTIC_TEXT_BYTES as usize;
    if value.len() <= maximum {
        return Ok(value.to_owned());
    }
    let marker = if maximum >= DIAGNOSTIC_TRUNCATED.len() { DIAGNOSTIC_TRUNCATED } else { "" };
    let mut cut = maximum.saturating_sub(marker.len());
    while !value.is_char_boundary(cut) {
        cut -= 1;
    }

    Ok(format!("{}{marker}", &value[..cut]))
}

fn parameter_cursor(
    snapshot: &dokime::runtime::query::RuntimeStateSnapshot, selection: execution_model::ParameterCursorSelection,
) -> execution_model::ParameterCursor {
    execution_model::ParameterCursor {
        observed: Some(canonical::cursor(snapshot)),
        selection: Some(selection),
    }
}

fn parameter_page(
    limit: u32, after: &str, cursor: Option<&execution_model::ParameterCursor>, snapshot: &dokime::runtime::query::RuntimeStateSnapshot,
    selection: &execution_model::ParameterCursorSelection,
) -> Result<usize, Status> {
    if let Some(cursor) = cursor {
        if cursor.selection.as_ref() != Some(selection) {
            return Err(Status::invalid_argument("parameter cursor selection mismatch"));
        }
        if cursor.observed.is_none() {
            return Err(Status::invalid_argument("parameter cursor requires observed revision"));
        }
    }
    canonical::cursor(snapshot).page(limit, after, cursor.and_then(|cursor| cursor.observed.as_ref())).map_err(canonical::cursor_failure)
}

fn case_parameters(
    service: &Resources, entries: &BTreeMap<String, Value>, after: &str, limit: usize, cursor: execution_model::ParameterCursor,
) -> Result<execution_model::ListCaseParametersReply, Status> {
    if !after.is_empty() && !entries.contains_key(after) {
        return Err(Status::invalid_argument("after name not found in requested case"));
    }
    let mut selected = entries.iter().filter(|(name, _)| after.is_empty() || name.as_str() > after);
    let mut budget = value::Budget::new()?;
    let count = selected.clone().take(limit).count();

    let mut parameters = canonical::slots(count)?;
    for (name, item) in selected.by_ref().take(limit) {
        parameters.push(execution_model::CaseParameter {
            name: name.clone(),
            value: Some(canonical::encode(item, &mut budget, 0)?),
        });
    }
    let next_after_name = selected.next().is_some().then(|| parameters.last().unwrap().name.clone());
    reply(
        service,
        execution_model::ListCaseParametersReply {
            parameters,
            next_after_name,
            cursor: Some(cursor),
        },
    )
}

/* === Tests === */
