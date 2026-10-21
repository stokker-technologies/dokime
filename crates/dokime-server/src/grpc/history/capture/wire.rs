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
use dokime::definition::{MaterializedParameter, ParameterSource, TestCase};
use dokime::domain::model::execution::result::ExecutionInvalidation;
use dokime::domain::model::execution::result::ExecutionStatus;
use dokime::domain::model::execution::result::StepStatus;
use dokime::domain::model::execution::result::TestExecution;
use dokime::domain::model::execution::snapshot::ExecutionSnapshot;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn execution_document(
    value: dokime::execution::storage::ExecutionDocument,
) -> Result<dokime::domain::model::history::wire::HistoricalExecutionDocument, Status> {
    Ok(dokime::domain::model::history::wire::HistoricalExecutionDocument {
        cycle: Some(dokime::domain::model::history::wire::HistoricalExecutionCycle {
            cycle_id: value.cycle.cycle_id.as_str().to_owned(),
            metadata: Some(canonical_metadata(&value.cycle.metadata)?),
        }),
        execution: Some(test_execution(value.execution)?),
    })
}

/* === Private Functions === */

fn test_execution(value: TestExecution) -> Result<dokime::domain::model::history::wire::HistoricalTestExecution, Status> {
    let snapshot = execution_snapshot(value.snapshot, &value.scope)?;
    let output_parameters = materialized_parameters(&value.output_parameters)?;
    let documents = collect(value.documents.values().map(|document| document_wire(document, &value.scope)))?;
    let health = collect(value.records.health().iter().map(notes::health))?;
    let notes = collect(value.records.notes().iter().map(notes::note))?;
    let steps = collect(value.steps.iter().map(step))?;
    Ok(dokime::domain::model::history::wire::HistoricalTestExecution {
        execution_id: value.test_execution_id.as_str().to_owned(),
        mode: match value.mode {
            dokime::domain::model::execution::ExecutionMode::Automatic => dokime::domain::model::execution::wire::ExecutionMode::Automatic,
            dokime::domain::model::execution::ExecutionMode::Interactive => {
                dokime::domain::model::execution::wire::ExecutionMode::Interactive
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
        started_at: value.started_at.as_ref().map(canonical::timestamp).transpose()?,
        finished_at: value.finished_at.as_ref().map(canonical::timestamp).transpose()?,
        steps,
        active_step_attempt: value
            .active_step_attempt
            .as_ref()
            .map(|start| {
                Ok::<_, Status>(dokime::domain::model::history::wire::HistoricalStepAttemptStart {
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
    attempt: &dokime::domain::model::execution::result::TestStepAttempt,
) -> Result<dokime::domain::model::history::wire::HistoricalStepAttempt, Status> {
    let detail = attempt
        .detail
        .as_ref()
        .map(|detail| {
            // Diagnostics Use The Preflighted Graph And Response Byte Budgets.
            // A Backtrace Is Not A Short Authored Metadata Field.
            if detail.chain.len() > value::MAX_COLLECTION || detail.span_trace.len() > value::MAX_COLLECTION {
                return Err(Status::resource_exhausted("step diagnostic exceeds history bounds"));
            }
            Ok(dokime::domain::model::identity::wire::Diagnostic {
                message: detail.message.clone(),
                chain: detail.chain.clone(),
                location: detail.location.clone(),
                span_trace: detail.span_trace.clone(),
                backtrace: detail.backtrace.clone(),
            })
        })
        .transpose()?;
    Ok(dokime::domain::model::history::wire::HistoricalStepAttempt {
        attempt_id: attempt.attempt_id.as_str().to_owned(),
        step_id: attempt.step_id.as_str().to_owned(),
        started_at: Some(canonical::timestamp(&attempt.started_at)?),
        finished_at: attempt.finished_at.as_ref().map(canonical::timestamp).transpose()?,
        termination: match attempt.termination {
            dokime::domain::model::execution::termination::TerminationReason::Completed => {
                dokime::domain::model::history::wire::StepTermination::Completed
            }
            dokime::domain::model::execution::termination::TerminationReason::Failed => {
                dokime::domain::model::history::wire::StepTermination::Failed
            }
            dokime::domain::model::execution::termination::TerminationReason::Cancelled => {
                dokime::domain::model::history::wire::StepTermination::Cancelled
            }
            dokime::domain::model::execution::termination::TerminationReason::Interrupted => {
                dokime::domain::model::history::wire::StepTermination::Interrupted
            }
        },
        detail,
    })
}

fn execution_snapshot(
    value: ExecutionSnapshot, scope: &dokime::domain::model::execution::RecordScope,
) -> Result<dokime::domain::model::history::wire::HistoricalExecutionSnapshot, Status> {
    let mut budget = value::Budget::new();
    let parameter_definitions = catalog::parameters(value.parameter_definitions(), &mut budget)?;
    let parameters = materialized_parameters(value.parameters())?;
    let documents = collect(value.documents().values().map(|document| document_wire(document, scope)))?;
    if let Some(version) = value.host_version() {
        value::text(version)?;
    }
    Ok(dokime::domain::model::history::wire::HistoricalExecutionSnapshot {
        case_definition: Some(case_definition(value.case())?),
        host_version: value.host_version().map(str::to_owned),
        suite: value.suite().map(canonical_metadata).transpose()?,
        template_id: value.template_id().map(|id| id.as_str().to_owned()),
        template: value.template().map(|template| canonical_metadata(&template.metadata)).transpose()?,
        plugin: value.plugin().map(canonical::execution_plugin).transpose()?,
        parameter_definitions,
        parameters,
        documents,
    })
}

fn case_definition(case: &TestCase) -> Result<dokime::domain::model::history::wire::HistoricalCaseDefinition, Status> {
    value::text(&case.suite_id)?;
    value::collection(case.parameters.len())?;
    let mut budget = value::Budget::new();
    let parameters = collect(case.parameters.iter().map(|(name, parameter)| {
        value::text(name)?;
        Ok(dokime::domain::model::execution::wire::CaseParameter {
            name: name.clone(),
            value: Some(projected_value(parameter, &mut budget, 0)?),
        })
    }))?;
    Ok(dokime::domain::model::history::wire::HistoricalCaseDefinition {
        suite_id: case.suite_id.clone(),
        case_id: case.test_case_id.as_str().to_owned(),
        template_id: case.test_case_template_id.as_str().to_owned(),
        metadata: Some(canonical_metadata(&case.metadata)?),
        suite: case.suite.as_ref().map(canonical_metadata).transpose()?,
        parameters,
        generation_index: case.generation_index,
    })
}

fn invalidation(value: &ExecutionInvalidation) -> Result<dokime::domain::model::execution::wire::ExecutionInvalidation, Status> {
    Ok(dokime::domain::model::execution::wire::ExecutionInvalidation {
        previous: Some(canonical::execution_plugin(&value.previous)?),
        current: value.current.as_ref().map(canonical::execution_plugin).transpose()?,
    })
}

fn materialized_parameters(
    values: &BTreeMap<String, MaterializedParameter>,
) -> Result<Vec<dokime::domain::model::execution::wire::MaterializedParameter>, Status> {
    let mut budget = value::Budget::new();
    collect(values.iter().map(|(name, parameter)| {
        value::text(name)?;
        let unit = parameter.unit.as_ref().map(catalog::unit).transpose()?;
        Ok(dokime::domain::model::execution::wire::MaterializedParameter {
            name: name.clone(),
            metadata: Some(canonical_metadata(&parameter.metadata)?),
            group_id: parameter.group_id.as_ref().map(|id| id.as_str().to_owned()),
            source: parameter_source(&parameter.source),
            value: Some(projected_value(&parameter.value, &mut budget, 0)?),
            unit,
        })
    }))
}

fn parameter_source(value: &ParameterSource) -> dokime::domain::model::execution::wire::ParameterSource {
    match value {
        ParameterSource::Definition => dokime::domain::model::execution::wire::ParameterSource::Definition,
        ParameterSource::Traceability => dokime::domain::model::execution::wire::ParameterSource::Traceability,
        ParameterSource::Environment => dokime::domain::model::execution::wire::ParameterSource::Environment,
        ParameterSource::Resource => dokime::domain::model::execution::wire::ParameterSource::Resource,
        ParameterSource::Operator => dokime::domain::model::execution::wire::ParameterSource::Operator,
        ParameterSource::Runtime => dokime::domain::model::execution::wire::ParameterSource::Runtime,
        ParameterSource::Telemetry => dokime::domain::model::execution::wire::ParameterSource::Telemetry,
        ParameterSource::Record => dokime::domain::model::execution::wire::ParameterSource::Record,
        ParameterSource::Output => dokime::domain::model::execution::wire::ParameterSource::Output,
    }
}

fn step(
    value: &dokime::domain::model::execution::result::StepRecord,
) -> Result<dokime::domain::model::history::wire::HistoricalStep, Status> {
    let measurements = collect(value.measurements.iter().enumerate().map(|(ordinal, measurement)| {
        Ok(dokime::domain::model::execution::wire::StepMeasurement {
            metadata: Some(canonical_metadata(&measurement.metadata)?),
            value: Some(projected_value(&measurement.value, &mut value::Budget::new(), 0)?),
            unit: measurement.unit.as_ref().map(catalog::unit).transpose()?,
            ordinal: ordinal as u64,
        })
    }))?;
    let criteria = collect(value.criteria.iter().enumerate().map(|(ordinal, criterion)| {
        Ok(dokime::domain::model::execution::wire::StepCriterion {
            criterion_id: criterion.criterion_id.as_str().to_owned(),
            metadata: Some(canonical_metadata(&criterion.metadata)?),
            status: match criterion.status {
                dokime::domain::model::execution::result::CriterionStatus::Pending => {
                    dokime::domain::model::execution::wire::CriterionStatus::Pending
                }
                dokime::domain::model::execution::result::CriterionStatus::Passed => {
                    dokime::domain::model::execution::wire::CriterionStatus::Passed
                }
                dokime::domain::model::execution::result::CriterionStatus::Failed => {
                    dokime::domain::model::execution::wire::CriterionStatus::Failed
                }
                dokime::domain::model::execution::result::CriterionStatus::Indeterminate => {
                    dokime::domain::model::execution::wire::CriterionStatus::Indeterminate
                }
                dokime::domain::model::execution::result::CriterionStatus::Skipped => {
                    dokime::domain::model::execution::wire::CriterionStatus::Skipped
                }
            },
            observed: criterion.observed.as_ref().map(|item| projected_value(item, &mut value::Budget::new(), 0)).transpose()?,
            message: criterion.message.clone(),
            ordinal: ordinal as u64,
        })
    }))?;
    Ok(dokime::domain::model::history::wire::HistoricalStep {
        step_id: value.step_id.as_str().to_owned(),
        metadata: Some(canonical_metadata(&value.metadata)?),
        status: match value.status {
            StepStatus::Pending => dokime::domain::model::execution::wire::StepStatus::Pending,
            StepStatus::Running => dokime::domain::model::execution::wire::StepStatus::Running,
            StepStatus::Passed => dokime::domain::model::execution::wire::StepStatus::Passed,
            StepStatus::Failed => dokime::domain::model::execution::wire::StepStatus::Failed,
            StepStatus::Skipped => dokime::domain::model::execution::wire::StepStatus::Skipped,
            StepStatus::Cancelled => dokime::domain::model::execution::wire::StepStatus::Cancelled,
        },
        started_at: value.started_at.as_ref().map(canonical::timestamp).transpose()?,
        finished_at: value.finished_at.as_ref().map(canonical::timestamp).transpose()?,
        measurements,
        criteria,
    })
}

fn execution_status(value: ExecutionStatus) -> dokime::domain::model::execution::wire::ExecutionStatus {
    match value {
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

fn collect<T>(source: impl ExactSizeIterator<Item = Result<T, Status>>) -> Result<Vec<T>, Status> {
    let mut target = value::projection::slots(source.len()).map_err(construction_failure)?;
    for item in source {
        target.push(item?);
    }
    Ok(target)
}

fn canonical_metadata(
    source: &dokime::domain::model::identity::Metadata,
) -> Result<dokime::domain::model::identity::wire::Metadata, Status> {
    canonical::metadata(source)
}

fn projected_value(
    source: &dokime::domain::model::values::Value, budget: &mut value::Budget, depth: usize,
) -> Result<dokime::domain::model::values::wire::Value, Status> {
    canonical::encode(source, budget, depth)
}

fn construction_failure(source: harmos::Error) -> Status {
    super::super::wire::checkpoint_failure(source.into())
}

/* === Tests === */
