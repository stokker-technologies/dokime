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
use super::detail::{Bounds, diagnostic, execution, metadata, metadata_memory};
use super::*;
use dokime_runtime::execution::{CriterionStatus, StepRecord, StepStatus};
use dokime_runtime::runtime::query::RuntimeStateSnapshot;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
struct Page {
    start: usize,
    end: usize,
    observed: Cursor,
    next: Option<wire::StepPageCursor>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub(super) fn list(
    service: &Resources,
    request: &wire::ListExecutionStepsRequest,
) -> Result<wire::ListExecutionStepsReply, Status> {
    let snapshot = service.runtime.watch_state().latest();
    let (_, _, execution) = execution(snapshot.state(), &request.execution_id)?;
    let page = page(
        &snapshot,
        &request.execution_id,
        "",
        wire::StepPageKind::Steps,
        request.limit,
        request.cursor.as_ref(),
        execution.steps.len(),
    )?;
    let mut bounds = Bounds::new(service.max_message_bytes);
    let count = page.end - page.start;
    bounds.bytes(count.saturating_mul(std::mem::size_of::<wire::ExecutionStep>()))?;
    let mut steps = canonical::slots(count)?;
    for (index, step) in execution.steps[page.start..page.end].iter().enumerate() {
        bounds.text(step.step_id.as_str())?;
        bounds.bytes(metadata_memory(&step.metadata)?)?;
        steps.push(wire::ExecutionStep {
            step_id: step.step_id.to_string(),
            metadata: Some(metadata(&step.metadata)?),
            status: match step.status {
                StepStatus::Pending => wire::StepStatus::Pending,
                StepStatus::Running => wire::StepStatus::Running,
                StepStatus::Passed => wire::StepStatus::Passed,
                StepStatus::Failed => wire::StepStatus::Failed,
                StepStatus::Skipped => wire::StepStatus::Skipped,
                StepStatus::Cancelled => wire::StepStatus::Cancelled,
            },
            started_at: step
                .started_at
                .as_ref()
                .map(canonical::timestamp)
                .transpose()?,
            finished_at: step
                .finished_at
                .as_ref()
                .map(canonical::timestamp)
                .transpose()?,
            measurement_count: step.measurements.len() as u64,
            criterion_count: step.criteria.len() as u64,
            ordinal: (page.start + index) as u64,
            detail: step
                .detail
                .as_ref()
                .map(|detail| diagnostic(detail, &mut bounds))
                .transpose()?,
        });
    }
    reply(
        service,
        wire::ListExecutionStepsReply {
            steps,
            next_cursor: page.next,
            observed_cursor: Some(page.observed),
        },
    )
}

pub(super) fn criteria(
    service: &Resources,
    request: &wire::ListStepEvidenceRequest,
) -> Result<wire::ListStepCriteriaReply, Status> {
    let snapshot = service.runtime.watch_state().latest();
    let (_, _, execution) = execution(snapshot.state(), &request.execution_id)?;
    let step = step(&execution.steps, &request.step_id)?;
    let page = page(
        &snapshot,
        &request.execution_id,
        &request.step_id,
        wire::StepPageKind::Criteria,
        request.limit,
        request.cursor.as_ref(),
        step.criteria.len(),
    )?;
    let mut bounds = Bounds::new(service.max_message_bytes);
    let mut budget = value::Budget::new();
    let count = page.end - page.start;
    bounds.bytes(count.saturating_mul(std::mem::size_of::<wire::StepCriterion>()))?;
    let mut criteria = canonical::slots(count)?;
    for (index, criterion) in step.criteria[page.start..page.end].iter().enumerate() {
        bounds.text(criterion.criterion_id.as_str())?;
        bounds.bytes(metadata_memory(&criterion.metadata)?)?;
        let observed = criterion
            .observed
            .as_ref()
            .map(|observed| {
                bounds.value(observed, 0)?;
                canonical::encode(observed, &mut budget, 0)
            })
            .transpose()?;
        if let Some(message) = &criterion.message {
            bounds.text(message)?;
        }
        criteria.push(wire::StepCriterion {
            criterion_id: criterion.criterion_id.to_string(),
            metadata: Some(metadata(&criterion.metadata)?),
            status: match criterion.status {
                CriterionStatus::Pending => wire::CriterionStatus::Pending,
                CriterionStatus::Passed => wire::CriterionStatus::Passed,
                CriterionStatus::Failed => wire::CriterionStatus::Failed,
                CriterionStatus::Indeterminate => wire::CriterionStatus::Indeterminate,
                CriterionStatus::Skipped => wire::CriterionStatus::Skipped,
            },
            observed,
            message: criterion.message.clone(),
            ordinal: (page.start + index) as u64,
        });
    }
    reply(
        service,
        wire::ListStepCriteriaReply {
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
    kind: wire::StepPageKind,
    limit: u32,
    given: Option<&wire::StepPageCursor>,
    len: usize,
) -> Result<Page, Status> {
    let revision = given
        .map(|cursor| {
            if cursor.execution_id != execution_id
                || cursor.step_id != step_id
                || cursor.kind != kind
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
    let limit = canonical::cursor(snapshot)
        .page(limit, "", revision)
        .map_err(canonical::cursor_failure)?;
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
    let observed = canonical::cursor(snapshot);
    Ok(Page {
        start,
        end,
        next: (end < len).then(|| wire::StepPageCursor {
            revision: Some(observed.clone()),
            execution_id: execution_id.into(),
            step_id: step_id.into(),
            kind,
            next_ordinal: end as u64,
        }),
        observed,
    })
}

fn reply<T: harmos::Protobuf>(service: &Resources, value: T) -> Result<T, Status> {
    if value.encoded_len() > service.max_message_bytes {
        return Err(Status::resource_exhausted(
            "step evidence exceeds response budget",
        ));
    }
    Ok(value)
}

fn step<'a>(steps: &'a [StepRecord], id: &str) -> Result<&'a StepRecord, Status> {
    value::text(id)?;
    steps
        .iter()
        .find(|step| step.step_id.as_str() == id)
        .ok_or_else(|| Status::not_found("step not found in requested execution"))
}

/* === Tests === */
