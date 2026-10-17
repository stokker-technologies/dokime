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

/* === Imports === */
// Standard Library Imports
use std::collections::BTreeMap;
// Third Party Imports
use tonic::Status;
// Local Imports
use crate::grpc::history::document::canonical::wire as document_wire;
use crate::grpc::{
    catalog, execution, notes,
    value::{self, canonical},
};
use dokime_runtime::definition::{MaterializedParameter, ParameterSource, TestCase};
use dokime_runtime::execution::result::ExecutionInvalidation;
use dokime_runtime::execution::{ExecutionSnapshot, ExecutionStatus, StepStatus, TestExecution};
use dokime_contract::operations::{
    common as common_wire, execution as execution_wire, history as history_wire,
    value as value_wire,
};
/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn execution_document(
    value: dokime_runtime::execution::storage::ExecutionDocument,
) -> Result<history_wire::HistoricalExecutionDocument, Status> {
    Ok(history_wire::HistoricalExecutionDocument {
        cycle: Some(history_wire::HistoricalExecutionCycle {
            cycle_id: value.cycle.cycle_id.as_str().to_owned(),
            metadata: Some(canonical_metadata(&value.cycle.metadata)?),
        }),
        execution: Some(test_execution(value.execution)?),
    })
}

fn test_execution(value: TestExecution) -> Result<history_wire::HistoricalTestExecution, Status> {
    let snapshot = execution_snapshot(value.snapshot, &value.scope)?;
    let output_parameters = materialized_parameters(&value.output_parameters)?;
    let documents = collect(
        value
            .documents
            .values()
            .map(|document| document_wire(document, &value.scope)),
    )?;
    let health = collect(value.records.health().iter().map(notes::health))?;
    let notes = collect(value.records.notes().iter().map(notes::note))?;
    let steps = collect(value.steps.iter().map(step))?;
    Ok(history_wire::HistoricalTestExecution {
        execution_id: value.test_execution_id.as_str().to_owned(),
        mode: match value.mode {
            dokime_runtime::execution::ExecutionMode::Automatic => {
                execution_wire::ExecutionMode::Automatic
            }
            dokime_runtime::execution::ExecutionMode::Interactive => {
                execution_wire::ExecutionMode::Interactive
            }
        },
        case_id: value.test_case_id.as_str().to_owned(),
        suite: value.suite.as_ref().map(canonical_metadata).transpose()?,
        status: execution_status(value.status),
        scope: Some(canonical::scope(&value.scope)?),
        snapshot: Some(snapshot),
        output_parameters,
        documents,
        health,
        notes,
        started_at: value
            .started_at
            .as_ref()
            .map(canonical::timestamp)
            .transpose()?,
        finished_at: value
            .finished_at
            .as_ref()
            .map(canonical::timestamp)
            .transpose()?,
        steps,
        active_step_attempt: value
            .active_step_attempt
            .as_ref()
            .map(|start| {
                Ok::<_, Status>(history_wire::HistoricalStepAttemptStart {
                    attempt_id: start.attempt_id.as_str().to_owned(),
                    step_id: start.step_id.as_str().to_owned(),
                    started_at: Some(canonical::timestamp(&start.started_at)?),
                })
            })
            .transpose()?,
        step_attempts: collect(value.step_attempts.iter().map(step_attempt))?,
        result: value.result.as_ref().map(execution::result_summary),
        invalidation: value.invalidation.as_ref().map(invalidation).transpose()?,
    })
}

fn step_attempt(
    attempt: &dokime_runtime::execution::TestStepAttempt,
) -> Result<history_wire::HistoricalStepAttempt, Status> {
    let detail = attempt
        .detail
        .as_ref()
        .map(|detail| {
            // Diagnostics Use The Preflighted Graph And Response Byte Budgets.
            // A Backtrace Is Not A Short Authored Metadata Field.
            if detail.chain.len() > value::MAX_COLLECTION
                || detail.span_trace.len() > value::MAX_COLLECTION
            {
                return Err(Status::resource_exhausted(
                    "step diagnostic exceeds history bounds",
                ));
            }
            Ok(common_wire::Diagnostic {
                message: detail.message.clone(),
                chain: detail.chain.clone(),
                location: detail.location.clone(),
                span_trace: detail.span_trace.clone(),
                backtrace: detail.backtrace.clone(),
            })
        })
        .transpose()?;
    Ok(history_wire::HistoricalStepAttempt {
        attempt_id: attempt.attempt_id.as_str().to_owned(),
        step_id: attempt.step_id.as_str().to_owned(),
        started_at: Some(canonical::timestamp(&attempt.started_at)?),
        finished_at: attempt
            .finished_at
            .as_ref()
            .map(canonical::timestamp)
            .transpose()?,
        termination: match attempt.termination {
            dokime_runtime::execution::TerminationReason::Completed => {
                history_wire::StepTermination::Completed
            }
            dokime_runtime::execution::TerminationReason::Failed => {
                history_wire::StepTermination::Failed
            }
            dokime_runtime::execution::TerminationReason::Cancelled => {
                history_wire::StepTermination::Cancelled
            }
            dokime_runtime::execution::TerminationReason::Interrupted => {
                history_wire::StepTermination::Interrupted
            }
        },
        detail,
    })
}

fn execution_snapshot(
    value: ExecutionSnapshot,
    scope: &dokime_runtime::core::RecordScope,
) -> Result<history_wire::HistoricalExecutionSnapshot, Status> {
    let mut budget = value::Budget::new();
    let parameter_definitions = catalog::parameters(value.parameter_definitions(), &mut budget)?;
    let parameters = materialized_parameters(value.parameters())?;
    let documents = collect(
        value
            .documents()
            .values()
            .map(|document| document_wire(document, scope)),
    )?;
    if let Some(version) = value.host_version() {
        value::text(version)?;
    }
    Ok(history_wire::HistoricalExecutionSnapshot {
        case_definition: Some(case_definition(value.case())?),
        host_version: value.host_version().map(str::to_owned),
        suite: value.suite().map(canonical_metadata).transpose()?,
        template_id: value.template_id().map(|id| id.as_str().to_owned()),
        template: value
            .template()
            .map(|template| canonical_metadata(&template.metadata))
            .transpose()?,
        plugin: value
            .plugin()
            .map(canonical::execution_plugin)
            .transpose()?,
        parameter_definitions,
        parameters,
        documents,
    })
}

fn case_definition(case: &TestCase) -> Result<history_wire::HistoricalCaseDefinition, Status> {
    value::text(&case.suite_id)?;
    value::collection(case.parameters.len())?;
    let mut budget = value::Budget::new();
    let parameters = collect(case.parameters.iter().map(|(name, parameter)| {
        value::text(name)?;
        Ok(execution_wire::CaseParameter {
            name: name.clone(),
            value: Some(projected_value(parameter, &mut budget, 0)?),
        })
    }))?;
    Ok(history_wire::HistoricalCaseDefinition {
        suite_id: case.suite_id.clone(),
        case_id: case.test_case_id.as_str().to_owned(),
        template_id: case.test_case_template_id.as_str().to_owned(),
        metadata: Some(canonical_metadata(&case.metadata)?),
        suite: case.suite.as_ref().map(canonical_metadata).transpose()?,
        parameters,
        generation_index: case.generation_index,
    })
}

fn invalidation(
    value: &ExecutionInvalidation,
) -> Result<execution_wire::ExecutionInvalidation, Status> {
    Ok(execution_wire::ExecutionInvalidation {
        previous: Some(canonical::execution_plugin(&value.previous)?),
        current: value
            .current
            .as_ref()
            .map(canonical::execution_plugin)
            .transpose()?,
    })
}

fn materialized_parameters(
    values: &BTreeMap<String, MaterializedParameter>,
) -> Result<Vec<execution_wire::MaterializedParameter>, Status> {
    let mut budget = value::Budget::new();
    collect(values.iter().map(|(name, parameter)| {
        value::text(name)?;
        let unit = parameter.unit.as_ref().map(catalog::unit).transpose()?;
        Ok(execution_wire::MaterializedParameter {
            name: name.clone(),
            metadata: Some(canonical_metadata(&parameter.metadata)?),
            group_id: parameter.group_id.as_ref().map(|id| id.as_str().to_owned()),
            source: parameter_source(&parameter.source),
            value: Some(projected_value(&parameter.value, &mut budget, 0)?),
            unit,
        })
    }))
}

fn parameter_source(value: &ParameterSource) -> execution_wire::ParameterSource {
    match value {
        ParameterSource::Definition => execution_wire::ParameterSource::Definition,
        ParameterSource::Traceability => execution_wire::ParameterSource::Traceability,
        ParameterSource::Environment => execution_wire::ParameterSource::Environment,
        ParameterSource::Resource => execution_wire::ParameterSource::Resource,
        ParameterSource::Operator => execution_wire::ParameterSource::Operator,
        ParameterSource::Runtime => execution_wire::ParameterSource::Runtime,
        ParameterSource::Telemetry => execution_wire::ParameterSource::Telemetry,
        ParameterSource::Record => execution_wire::ParameterSource::Record,
        ParameterSource::Output => execution_wire::ParameterSource::Output,
    }
}

fn step(value: &dokime_runtime::execution::StepRecord) -> Result<history_wire::HistoricalStep, Status> {
    let measurements = collect(value.measurements.iter().enumerate().map(
        |(ordinal, measurement)| {
            Ok(execution_wire::StepMeasurement {
                metadata: Some(canonical_metadata(&measurement.metadata)?),
                value: Some(projected_value(
                    &measurement.value,
                    &mut value::Budget::new(),
                    0,
                )?),
                unit: measurement.unit.as_ref().map(catalog::unit).transpose()?,
                ordinal: ordinal as u64,
            })
        },
    ))?;
    let criteria = collect(
        value
            .criteria
            .iter()
            .enumerate()
            .map(|(ordinal, criterion)| {
                Ok(execution_wire::StepCriterion {
                    criterion_id: criterion.criterion_id.as_str().to_owned(),
                    metadata: Some(canonical_metadata(&criterion.metadata)?),
                    status: match criterion.status {
                        dokime_runtime::execution::CriterionStatus::Pending => {
                            execution_wire::CriterionStatus::Pending
                        }
                        dokime_runtime::execution::CriterionStatus::Passed => {
                            execution_wire::CriterionStatus::Passed
                        }
                        dokime_runtime::execution::CriterionStatus::Failed => {
                            execution_wire::CriterionStatus::Failed
                        }
                        dokime_runtime::execution::CriterionStatus::Indeterminate => {
                            execution_wire::CriterionStatus::Indeterminate
                        }
                        dokime_runtime::execution::CriterionStatus::Skipped => {
                            execution_wire::CriterionStatus::Skipped
                        }
                    },
                    observed: criterion
                        .observed
                        .as_ref()
                        .map(|item| projected_value(item, &mut value::Budget::new(), 0))
                        .transpose()?,
                    message: criterion.message.clone(),
                    ordinal: ordinal as u64,
                })
            }),
    )?;
    Ok(history_wire::HistoricalStep {
        step_id: value.step_id.as_str().to_owned(),
        metadata: Some(canonical_metadata(&value.metadata)?),
        status: match value.status {
            StepStatus::Pending => execution_wire::StepStatus::Pending,
            StepStatus::Running => execution_wire::StepStatus::Running,
            StepStatus::Passed => execution_wire::StepStatus::Passed,
            StepStatus::Failed => execution_wire::StepStatus::Failed,
            StepStatus::Skipped => execution_wire::StepStatus::Skipped,
            StepStatus::Cancelled => execution_wire::StepStatus::Cancelled,
        },
        started_at: value
            .started_at
            .as_ref()
            .map(canonical::timestamp)
            .transpose()?,
        finished_at: value
            .finished_at
            .as_ref()
            .map(canonical::timestamp)
            .transpose()?,
        measurements,
        criteria,
    })
}

fn execution_status(value: ExecutionStatus) -> execution_wire::ExecutionStatus {
    match value {
        ExecutionStatus::Invalidated => execution_wire::ExecutionStatus::Invalidated,
        ExecutionStatus::Skipped => execution_wire::ExecutionStatus::Skipped,
        ExecutionStatus::Pending => execution_wire::ExecutionStatus::Pending,
        ExecutionStatus::Running => execution_wire::ExecutionStatus::Running,
        ExecutionStatus::Passed => execution_wire::ExecutionStatus::Passed,
        ExecutionStatus::Failed => execution_wire::ExecutionStatus::Failed,
        ExecutionStatus::Blocked => execution_wire::ExecutionStatus::Blocked,
        ExecutionStatus::Cancelled => execution_wire::ExecutionStatus::Cancelled,
        ExecutionStatus::Interrupted => execution_wire::ExecutionStatus::Interrupted,
    }
}

/* === Private Functions === */
fn collect<T>(source: impl ExactSizeIterator<Item = Result<T, Status>>) -> Result<Vec<T>, Status> {
    let mut target = value::projection::slots(source.len()).map_err(construction_failure)?;
    for item in source {
        target.push(item?);
    }
    Ok(target)
}
fn canonical_metadata(
    source: &dokime_runtime::core::Metadata,
) -> Result<common_wire::Metadata, Status> {
    canonical::metadata(source)
}
fn projected_value(
    source: &dokime_runtime::core::Value,
    budget: &mut value::Budget,
    depth: usize,
) -> Result<value_wire::Value, Status> {
    canonical::encode(source, budget, depth)
}
fn construction_failure(source: harmos::Error) -> Status {
    super::super::wire::checkpoint_failure(source.into())
}

/* === Tests === */
