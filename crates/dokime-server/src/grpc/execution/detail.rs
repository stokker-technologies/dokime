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
use dokime_app::core::{Metadata, TestExecutionId, Value};
use dokime_app::definition::{MaterializedParameter, ParameterSource};
use dokime_app::execution::TestExecution;
use dokime_contract::operations::common::Diagnostic;

/* === Constants === */
const MAX_DIAGNOSTIC_TEXT: usize = 4096;
const DIAGNOSTIC_TRUNCATED: &str = "\n… truncated";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub(in crate::grpc) struct Bounds {
    remaining: usize,
    nodes: value::Budget,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Bounds {
    pub(in crate::grpc) fn new(limit: usize) -> Self {
        Self {
            remaining: limit.min(4 * 1024 * 1024),
            nodes: value::Budget::new(),
        }
    }
    pub(in crate::grpc) fn bytes(&mut self, count: usize) -> Result<(), Status> {
        self.remaining = self
            .remaining
            .checked_sub(count)
            .ok_or_else(|| Status::resource_exhausted("inspection allocation budget exceeded"))?;
        Ok(())
    }
    pub(in crate::grpc) fn text(&mut self, text: &str) -> Result<(), Status> {
        value::text(text)?;
        self.bytes(text.len().saturating_add(32))
    }
    pub(in crate::grpc) fn value(&mut self, item: &Value, depth: usize) -> Result<(), Status> {
        let bytes = canonical::memory(item, &mut self.nodes, depth)?;
        self.bytes(bytes)
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

pub(crate) fn result(
    value: &dokime_app::execution::ResultSummary,
) -> dokime_contract::operations::execution::ResultSummary {
    use dokime_app::execution::ResultStatus;
    dokime_contract::operations::execution::ResultSummary {
        status: match value.status {
            ResultStatus::Pending => dokime_contract::operations::execution::ResultStatus::Pending,
            ResultStatus::Passed => dokime_contract::operations::execution::ResultStatus::Passed,
            ResultStatus::Failed => dokime_contract::operations::execution::ResultStatus::Failed,
            ResultStatus::Blocked => dokime_contract::operations::execution::ResultStatus::Blocked,
            ResultStatus::Cancelled => {
                dokime_contract::operations::execution::ResultStatus::Cancelled
            }
        },
        passed: value.passed,
        failed: value.failed,
        blocked: value.blocked,
        skipped: value.skipped,
    }
}

pub(super) fn get_case(
    service: &Resources,
    request: &wire::GetCaseRequest,
) -> Result<wire::CaseDetail, Status> {
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
    let mut bounds = Bounds::new(service.max_message_bytes);
    bounds.bytes(std::mem::size_of::<wire::CaseDetail>())?;
    if let Some(suite) = &case.suite {
        bounds.bytes(metadata_memory(suite)?)?;
    }
    reply(
        service,
        wire::CaseDetail {
            summary: Some(super::case_summary(
                case,
                ordinal,
                dokime_app::runtime::query::case_projection(
                    snapshot.state(),
                    &cycle,
                    &case.test_case_id,
                ),
                &mut bounds,
            )?),
            suite: case.suite.as_ref().map(metadata).transpose()?,
            parameter_count: case.parameters.len() as u64,
            generation_index: case.generation_index,
            observed_cursor: Some(canonical::cursor(&snapshot)),
        },
    )
}

pub(super) fn get_execution(
    service: &Resources,
    request: &wire::GetExecutionRequest,
) -> Result<wire::ExecutionDetail, Status> {
    let snapshot = service.runtime.watch_state().latest();
    let (cycle, ordinal, execution) = execution(snapshot.state(), &request.execution_id)?;
    let mut bounds = Bounds::new(service.max_message_bytes);
    bounds.bytes(std::mem::size_of::<wire::ExecutionDetail>())?;
    bounds.text(cycle.test_cycle_id.as_str())?;
    bounds.text(execution.test_execution_id.as_str())?;
    bounds.text(execution.test_case_id.as_str())?;
    bounds.bytes(canonical::scope_memory(&execution.scope)?)?;
    if let Some(suite) = execution.snapshot.suite() {
        bounds.bytes(metadata_memory(suite)?)?;
    }
    if let Some(id) = execution.snapshot.template_id() {
        bounds.text(id.as_str())?;
    }
    if let Some(template) = execution.snapshot.template() {
        bounds.bytes(metadata_memory(&template.metadata)?)?;
    }
    if let Some(plugin) = execution.snapshot.plugin() {
        bounds.bytes(canonical::execution_plugin_memory(plugin)?)?;
    }
    if let Some(invalidation) = &execution.invalidation {
        bounds.bytes(canonical::execution_plugin_memory(&invalidation.previous)?)?;
        if let Some(current) = &invalidation.current {
            bounds.bytes(canonical::execution_plugin_memory(current)?)?;
        }
    }
    let summary = Some(execution_summary(execution, ordinal));
    reply(
        service,
        wire::ExecutionDetail {
            summary,
            cycle_id: cycle.test_cycle_id.to_string(),
            scope: Some(canonical::scope(&execution.scope)?),
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
                .map(canonical::execution_plugin)
                .transpose()?,
            invalidation: execution
                .invalidation
                .as_ref()
                .map(|invalidation| {
                    Ok::<_, Status>(wire::ExecutionInvalidation {
                        previous: Some(canonical::execution_plugin(&invalidation.previous)?),
                        current: invalidation
                            .current
                            .as_ref()
                            .map(canonical::execution_plugin)
                            .transpose()?,
                    })
                })
                .transpose()?,
            started_at: execution
                .started_at
                .as_ref()
                .map(canonical::timestamp)
                .transpose()?,
            finished_at: execution
                .finished_at
                .as_ref()
                .map(canonical::timestamp)
                .transpose()?,
            result: execution.result.as_ref().map(result),
            input_parameter_count: execution.snapshot.parameters().len() as u64,
            controls: dokime_app::runtime::query::execution_controls(
                snapshot.state(),
                &execution.test_execution_id,
            )
            .map(|controls| wire::ExecutionControls {
                request_id: controls.request_id.to_string(),
                mode: match controls.mode {
                    dokime_app::execution::ExecutionMode::Automatic => {
                        wire::ExecutionMode::Automatic
                    }
                    dokime_app::execution::ExecutionMode::Interactive => {
                        wire::ExecutionMode::Interactive
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
            observed_cursor: Some(canonical::cursor(&snapshot)),
        },
    )
}

pub(super) fn list_execution_parameters(
    service: &Resources,
    request: &wire::ListExecutionParametersRequest,
) -> Result<wire::ListMaterializedParametersReply, Status> {
    let snapshot = service.runtime.watch_state().latest();
    let (_, _, execution) = execution(snapshot.state(), &request.execution_id)?;
    let selected = match request.set {
        wire::ExecutionParameterSet::Inputs => execution.snapshot.parameters(),
        wire::ExecutionParameterSet::Outputs => &execution.output_parameters,
        wire::ExecutionParameterSet::Unspecified => {
            return Err(Status::invalid_argument(
                "explicit input/output parameter set is required",
            ));
        }
    };
    let selection = match request.set {
        wire::ExecutionParameterSet::Inputs => {
            wire::ParameterCursorSelection::ExecutionInputsId(request.execution_id.clone())
        }
        wire::ExecutionParameterSet::Outputs => {
            wire::ParameterCursorSelection::ExecutionOutputsId(request.execution_id.clone())
        }
        wire::ExecutionParameterSet::Unspecified => unreachable!(),
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

pub(super) fn metadata_memory(value: &Metadata) -> Result<usize, Status> {
    canonical::metadata_memory(value)
}

pub(super) fn diagnostic(
    value: &dokime_app::plugin::Diagnostic,
    bounds: &mut Bounds,
) -> Result<Diagnostic, Status> {
    value::collection(value.chain.len())?;
    value::collection(value.span_trace.len())?;
    bounds.bytes(std::mem::size_of::<Diagnostic>())?;
    bounds.bytes(
        value
            .chain
            .len()
            .saturating_add(value.span_trace.len())
            .saturating_mul(std::mem::size_of::<String>()),
    )?;
    let message = bounded(&value.message, bounds)?;
    let mut chain = canonical::slots(value.chain.len())?;
    for cause in &value.chain {
        chain.push(bounded(cause, bounds)?);
    }
    let location = bounded(&value.location, bounds)?;
    let mut span_trace = canonical::slots(value.span_trace.len())?;
    for span in &value.span_trace {
        span_trace.push(bounded(span, bounds)?);
    }
    let backtrace = bounded(&value.backtrace, bounds)?;

    Ok(Diagnostic {
        message,
        chain,
        location,
        span_trace,
        backtrace,
    })
}

pub(super) fn list_case_parameters(
    service: &Resources,
    request: &wire::ListCaseParametersRequest,
) -> Result<wire::ListCaseParametersReply, Status> {
    let id = cycle_id(&request.cycle_id)?;
    value::text(&request.case_id)?;
    let snapshot = service.runtime.watch_state().latest();
    let selection = wire::ParameterCursorSelection::PlannedCase(wire::PlannedCaseParameters {
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
fn reply<T: harmos::Protobuf>(service: &Resources, value: T) -> Result<T, Status> {
    if value.encoded_len() > service.max_message_bytes {
        return Err(Status::resource_exhausted(
            "inspection exceeds response budget",
        ));
    }
    Ok(value)
}

fn parameters(
    service: &Resources,
    entries: &BTreeMap<String, MaterializedParameter>,
    after: &str,
    limit: usize,
    cursor: wire::ParameterCursor,
) -> Result<wire::ListMaterializedParametersReply, Status> {
    if !after.is_empty() && !entries.contains_key(after) {
        return Err(Status::invalid_argument(
            "after name not found in requested parameter set",
        ));
    }
    let mut selected = entries.iter().filter(|(name, _)| name.as_str() > after);
    let mut bounds = Bounds::new(service.max_message_bytes);
    let mut budget = value::Budget::new();
    let count = selected.clone().take(limit).count();
    bounds.bytes(count.saturating_mul(std::mem::size_of::<wire::MaterializedParameter>()))?;
    let mut parameters = canonical::slots(count)?;
    for (name, item) in selected.by_ref().take(limit) {
        bounds.text(name)?;
        bounds.value(&item.value, 0)?;
        bounds.bytes(metadata_memory(&item.metadata)?)?;
        let metadata = metadata(&item.metadata)?;
        if let Some(group) = &item.group_id {
            bounds.text(group.as_str())?;
        }
        let unit = item
            .unit
            .as_ref()
            .map(|unit| {
                let bytes = super::super::catalog::unit_memory(unit).map_err(|_| {
                    Status::resource_exhausted("inspection unit exceeds response budget")
                })?;
                bounds.bytes(bytes)?;
                super::super::catalog::unit(unit)
            })
            .transpose()?;
        parameters.push(wire::MaterializedParameter {
            name: name.clone(),
            metadata: Some(metadata),
            group_id: item.group_id.as_ref().map(ToString::to_string),
            source: match item.source {
                ParameterSource::Definition => wire::ParameterSource::Definition,
                ParameterSource::Traceability => wire::ParameterSource::Traceability,
                ParameterSource::Environment => wire::ParameterSource::Environment,
                ParameterSource::Resource => wire::ParameterSource::Resource,
                ParameterSource::Operator => wire::ParameterSource::Operator,
                ParameterSource::Runtime => wire::ParameterSource::Runtime,
                ParameterSource::Telemetry => wire::ParameterSource::Telemetry,
                ParameterSource::Record => wire::ParameterSource::Record,
                ParameterSource::Output => wire::ParameterSource::Output,
            },
            value: Some(canonical::encode(&item.value, &mut budget, 0)?),
            unit,
        });
    }
    let next_after_name = selected
        .next()
        .is_some()
        .then(|| parameters.last().unwrap().name.clone());
    reply(
        service,
        wire::ListMaterializedParametersReply {
            parameters,
            next_after_name,
            cursor: Some(cursor),
        },
    )
}

fn bounded(value: &str, bounds: &mut Bounds) -> Result<String, Status> {
    if value.len() <= MAX_DIAGNOSTIC_TEXT {
        bounds.text(value)?;
        return Ok(value.to_owned());
    }
    let mut cut = MAX_DIAGNOSTIC_TEXT.saturating_sub(DIAGNOSTIC_TRUNCATED.len());
    while !value.is_char_boundary(cut) {
        cut -= 1;
    }
    bounds.bytes(
        cut.saturating_add(DIAGNOSTIC_TRUNCATED.len())
            .saturating_add(32),
    )?;
    Ok(format!("{}{DIAGNOSTIC_TRUNCATED}", &value[..cut]))
}

fn parameter_cursor(
    snapshot: &dokime_app::runtime::query::RuntimeStateSnapshot,
    selection: wire::ParameterCursorSelection,
) -> wire::ParameterCursor {
    wire::ParameterCursor {
        observed: Some(canonical::cursor(snapshot)),
        selection: Some(selection),
    }
}

fn parameter_page(
    limit: u32,
    after: &str,
    cursor: Option<&wire::ParameterCursor>,
    snapshot: &dokime_app::runtime::query::RuntimeStateSnapshot,
    selection: &wire::ParameterCursorSelection,
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
    canonical::cursor(snapshot)
        .page(
            limit,
            after,
            cursor.and_then(|cursor| cursor.observed.as_ref()),
        )
        .map_err(canonical::cursor_failure)
}

fn case_parameters(
    service: &Resources,
    entries: &BTreeMap<String, Value>,
    after: &str,
    limit: usize,
    cursor: wire::ParameterCursor,
) -> Result<wire::ListCaseParametersReply, Status> {
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
    let count = selected.clone().take(limit).count();
    bounds.bytes(count.saturating_mul(std::mem::size_of::<wire::CaseParameter>()))?;
    let mut parameters = canonical::slots(count)?;
    for (name, item) in selected.by_ref().take(limit) {
        bounds.text(name)?;
        bounds.value(item, 0)?;
        parameters.push(wire::CaseParameter {
            name: name.clone(),
            value: Some(canonical::encode(item, &mut budget, 0)?),
        });
    }
    let next_after_name = selected
        .next()
        .is_some()
        .then(|| parameters.last().unwrap().name.clone());
    reply(
        service,
        wire::ListCaseParametersReply {
            parameters,
            next_after_name,
            cursor: Some(cursor),
        },
    )
}

/* === Tests === */
