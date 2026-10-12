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
use super::detail::{Bounds, diagnostic, execution, metadata, metadata_len};
use super::*;
use dokime_app::execution::{CriterionStatus, StepRecord, StepStatus};
use dokime_app::runtime::query::RuntimeStateSnapshot;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Page {
    start: usize,
    end: usize,
    observed: v1::Cursor,
    next: Option<v1::StepPageCursor>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn list(
    service: &Service,
    request: Request<v1::ListExecutionStepsRequest>,
) -> Result<Response<v1::ListExecutionStepsReply>, Status> {
    let request = request.into_inner();
    let snapshot = service.runtime.watch_state().latest();
    let (_, _, execution) = execution(snapshot.state(), &request.execution_id)?;
    let page = page(
        &snapshot,
        &request.execution_id,
        "",
        v1::StepPageKind::Steps,
        request.limit,
        request.cursor.as_ref(),
        execution.steps.len(),
    )?;
    let mut bounds = Bounds::new(service.max_message_bytes);
    let steps = execution.steps[page.start..page.end]
        .iter()
        .enumerate()
        .map(|(index, step)| {
            bounds.text(step.step_id.as_str())?;
            bounds.bytes(metadata_len(&step.metadata)?)?;
            Ok(v1::ExecutionStep {
                step_id: step.step_id.to_string(),
                metadata: Some(metadata(&step.metadata)?),
                status: match step.status {
                    StepStatus::Pending => v1::StepStatus::Pending,
                    StepStatus::Running => v1::StepStatus::Running,
                    StepStatus::Passed => v1::StepStatus::Passed,
                    StepStatus::Failed => v1::StepStatus::Failed,
                    StepStatus::Skipped => v1::StepStatus::Skipped,
                    StepStatus::Cancelled => v1::StepStatus::Cancelled,
                } as i32,
                started_at: step.started_at.as_ref().map(value::timestamp).transpose()?,
                finished_at: step
                    .finished_at
                    .as_ref()
                    .map(value::timestamp)
                    .transpose()?,
                measurement_count: step.measurements.len() as u64,
                criterion_count: step.criteria.len() as u64,
                ordinal: (page.start + index) as u64,
                detail: step
                    .detail
                    .as_ref()
                    .map(|detail| diagnostic(detail, &mut bounds))
                    .transpose()?,
            })
        })
        .collect::<Result<Vec<_>, Status>>()?;
    reply(
        service,
        v1::ListExecutionStepsReply {
            steps,
            next_cursor: page.next,
            observed_cursor: Some(page.observed),
        },
    )
}

pub(super) fn criteria(
    service: &Service,
    request: Request<v1::ListStepEvidenceRequest>,
) -> Result<Response<v1::ListStepCriteriaReply>, Status> {
    let request = request.into_inner();
    let snapshot = service.runtime.watch_state().latest();
    let (_, _, execution) = execution(snapshot.state(), &request.execution_id)?;
    let step = step(&execution.steps, &request.step_id)?;
    let page = page(
        &snapshot,
        &request.execution_id,
        &request.step_id,
        v1::StepPageKind::Criteria,
        request.limit,
        request.cursor.as_ref(),
        step.criteria.len(),
    )?;
    let mut bounds = Bounds::new(service.max_message_bytes);
    let mut budget = value::Budget::new();
    let criteria = step.criteria[page.start..page.end]
        .iter()
        .enumerate()
        .map(|(index, criterion)| {
            bounds.text(criterion.criterion_id.as_str())?;
            bounds.bytes(metadata_len(&criterion.metadata)?)?;
            let observed = criterion
                .observed
                .as_ref()
                .map(|observed| {
                    bounds.value(observed, 0)?;
                    value::encode(observed, &mut budget, 0)
                })
                .transpose()?;
            if let Some(message) = &criterion.message {
                bounds.text(message)?;
            }
            Ok(v1::StepCriterion {
                criterion_id: criterion.criterion_id.to_string(),
                metadata: Some(metadata(&criterion.metadata)?),
                status: match criterion.status {
                    CriterionStatus::Pending => v1::CriterionStatus::Pending,
                    CriterionStatus::Passed => v1::CriterionStatus::Passed,
                    CriterionStatus::Failed => v1::CriterionStatus::Failed,
                    CriterionStatus::Indeterminate => v1::CriterionStatus::Indeterminate,
                    CriterionStatus::Skipped => v1::CriterionStatus::Skipped,
                } as i32,
                observed,
                message: criterion.message.clone(),
                ordinal: (page.start + index) as u64,
            })
        })
        .collect::<Result<Vec<_>, Status>>()?;
    reply(
        service,
        v1::ListStepCriteriaReply {
            criteria,
            next_cursor: page.next,
            observed_cursor: Some(page.observed),
        },
    )
}

/* === Private Functions === */
fn page(
    snapshot: &RuntimeStateSnapshot,
    execution_id: &str,
    step_id: &str,
    kind: v1::StepPageKind,
    limit: u32,
    given: Option<&v1::StepPageCursor>,
    len: usize,
) -> Result<Page, Status> {
    let revision = given
        .map(|cursor| {
            if cursor.execution_id != execution_id
                || cursor.step_id != step_id
                || cursor.kind != kind as i32
            {
                return Err(Status::invalid_argument(
                    "step cursor belongs to a different selection",
                ));
            }
            cursor
                .revision
                .as_ref()
                .ok_or_else(|| Status::invalid_argument("step cursor requires a revision"))
        })
        .transpose()?;
    let limit = observation::page(limit, "", revision, snapshot)?;
    let start = given
        .map(|cursor| {
            usize::try_from(cursor.next_ordinal)
                .map_err(|_| Status::out_of_range("step cursor ordinal exceeds addressable range"))
        })
        .transpose()?
        .unwrap_or(0);
    if start > len || (given.is_some() && start == len) {
        return Err(Status::out_of_range(
            "step cursor ordinal is outside the selected collection",
        ));
    }
    let end = start.saturating_add(limit).min(len);
    let observed = observation::cursor(snapshot);
    Ok(Page {
        start,
        end,
        next: (end < len).then(|| v1::StepPageCursor {
            revision: Some(observed.clone()),
            execution_id: execution_id.into(),
            step_id: step_id.into(),
            kind: kind as i32,
            next_ordinal: end as u64,
        }),
        observed,
    })
}

fn reply<T: Message>(service: &Service, value: T) -> Result<Response<T>, Status> {
    if value.encoded_len() > service.max_message_bytes {
        return Err(Status::resource_exhausted(
            "step evidence exceeds response budget",
        ));
    }
    Ok(Response::new(value))
}

fn step<'a>(steps: &'a [StepRecord], id: &str) -> Result<&'a StepRecord, Status> {
    value::text(id)?;
    steps
        .iter()
        .find(|step| step.step_id.as_str() == id)
        .ok_or_else(|| Status::not_found("step not found in requested execution"))
}

/* === Tests === */
