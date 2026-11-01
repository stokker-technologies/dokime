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

// Third Party Imports

// Local Imports
use super::detail::{diagnostic, execution, metadata};
use super::*;
use dokime::domain::model::execution::result::CriterionResult;
use dokime::domain::model::execution::result::CriterionStatus;
use dokime::domain::model::execution::result::StepRecord;
use dokime::domain::model::execution::result::StepStatus;
use dokime::domain::model::execution::wire as execution_model;
use dokime::runtime::query::RuntimeStateSnapshot;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct Page {
    start: usize,
    end: usize,
    observed: Cursor,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) async fn list(
    service: &Resources, request: &wire::ListExecutionSteps,
) -> Result<execution_model::ListExecutionStepsReply, Status> {
    let id = dokime::domain::model::identity::TestExecutionId::from_uuid(request.execution_id.clone())
        .map_err(|_| Status::invalid_argument("invalid execution UUID"))?;
    let snapshot = dokime::app::live_runtime(&service.runtime).await?.inspect_execution(&id).await.map_err(status)?;
    let (_, _, execution) = execution(snapshot.state(), &request.execution_id)?;
    let page = page(&snapshot, &request.execution_id, request.limit, request.cursor.as_ref(), execution.steps.len())?;
    let mut budget = value::Budget::new()?;
    let count = page.end - page.start;

    // Each step carries its criteria; a step that no longer fits ends the page.
    let mut steps = canonical::slots(count)?;
    for (index, step) in execution.steps[page.start..page.end].iter().enumerate() {
        let row = row(step, (page.start + index) as u64, &mut budget);
        let Some(row) = fit(row, steps.is_empty())? else { break };
        steps.push(row);
    }

    let end = page.start + steps.len();
    let next_cursor = (end < execution.steps.len()).then(|| execution_model::StepPageCursor {
        revision: Some(page.observed.clone()),
        execution_id: request.execution_id.clone(),
        step_id: String::new(),
        kind: execution_model::StepPageKind::Steps,
        next_ordinal: end as u64,
    });
    reply(
        service,
        execution_model::ListExecutionStepsReply {
            steps,
            next_cursor,
            observed_cursor: Some(page.observed),
        },
    )
}

/* === Private Functions === */

fn row(step: &StepRecord, ordinal: u64, budget: &mut value::Budget) -> Result<execution_model::ExecutionStep, Status> {
    let mut criteria = canonical::slots(step.criteria.len())?;
    for (index, record) in step.criteria.iter().enumerate() {
        criteria.push(criterion(record, index as u64, budget)?);
    }
    Ok(execution_model::ExecutionStep {
        step_id: step.step_id.to_string(),
        metadata: Some(metadata(&step.metadata)?),
        status: match step.status {
            StepStatus::Pending => execution_model::StepStatus::Pending,
            StepStatus::Running => execution_model::StepStatus::Running,
            StepStatus::Passed => execution_model::StepStatus::Passed,
            StepStatus::Failed => execution_model::StepStatus::Failed,
            StepStatus::Skipped => execution_model::StepStatus::Skipped,
            StepStatus::Cancelled => execution_model::StepStatus::Cancelled,
        },
        started_at: step.started_at.as_ref().map(canonical::timestamp).transpose()?,
        finished_at: step.finished_at.as_ref().map(canonical::timestamp).transpose()?,
        measurement_count: step.measurements.len() as u64,
        criterion_count: step.criteria.len() as u64,
        ordinal,
        detail: step.detail.as_ref().map(diagnostic).transpose()?,
        criteria,
    })
}

fn criterion(criterion: &CriterionResult, ordinal: u64, budget: &mut value::Budget) -> Result<execution_model::StepCriterion, Status> {
    let observed = criterion.observed.as_ref().map(|observed| canonical::encode(observed, budget, 0)).transpose()?;

    Ok(execution_model::StepCriterion {
        criterion_id: criterion.criterion_id.to_string(),
        metadata: Some(metadata(&criterion.metadata)?),
        status: match criterion.status {
            CriterionStatus::Pending => execution_model::CriterionStatus::Pending,
            CriterionStatus::Passed => execution_model::CriterionStatus::Passed,
            CriterionStatus::Failed => execution_model::CriterionStatus::Failed,
            CriterionStatus::Indeterminate => execution_model::CriterionStatus::Indeterminate,
            CriterionStatus::Skipped => execution_model::CriterionStatus::Skipped,
        },
        observed,
        message: criterion.message.clone(),
        ordinal,
    })
}

// A step past the budget ends a page that already holds one; alone it fails.
fn fit<T>(row: Result<T, Status>, first: bool) -> Result<Option<T>, Status> {
    match row {
        Err(error) if !first && error.code() == tonic::Code::ResourceExhausted => Ok(None),
        row => row.map(Some),
    }
}

fn page(
    snapshot: &RuntimeStateSnapshot, execution_id: &str, limit: u32, given: Option<&execution_model::StepPageCursor>, len: usize,
) -> Result<Page, Status> {
    let revision = given
        .map(|cursor| {
            if cursor.execution_id != execution_id || !cursor.step_id.is_empty() || cursor.kind != execution_model::StepPageKind::Steps {
                return Err(Status::invalid_argument("step cursor belongs to a different selection"));
            }
            cursor.revision.as_ref().ok_or_else(|| Status::invalid_argument("step cursor requires a revision"))
        })
        .transpose()?;
    let limit = canonical::cursor(snapshot).page(limit, "", revision).map_err(canonical::cursor_failure)?;
    let start = given
        .map(|cursor| {
            usize::try_from(cursor.next_ordinal).map_err(|_| Status::out_of_range("step cursor ordinal exceeds addressable range"))
        })
        .transpose()?
        .unwrap_or(0);
    if start > len || (given.is_some() && start == len) {
        return Err(Status::out_of_range("step cursor ordinal is outside the selected collection"));
    }
    Ok(Page {
        start,
        end: start.saturating_add(limit).min(len),
        observed: canonical::cursor(snapshot),
    })
}

fn reply<T: dokime::harmos::domain::Portable>(service: &Resources, value: T) -> Result<T, Status> {
    if value.wire_len()? > service.max_message_bytes {
        return Err(Status::resource_exhausted("step evidence exceeds response budget"));
    }
    Ok(value)
}

/* === Tests === */
