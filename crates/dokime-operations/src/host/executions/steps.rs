/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Executions Steps

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use harmos_protocol::model::{History, Position};

// Local Imports
use crate::host::{self, Context, execution_state};
use crate::{Error, Result};
use dokime_protocol::host::ExecutionStep;
use dokime_protocol::host::ExecutionsStepsRequest;
use dokime_protocol::host::ExecutionsStepsResponse;
use dokime_protocol::host::StepStatus;
use dokime_storage::{ExecutionCut, ExecutionState, Store};

/* === Constants === */

pub const ID: &str = "dokime.host.v1.Executions/Steps";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn handle(context: &Context, request: ExecutionsStepsRequest) -> Result<ExecutionsStepsResponse> {
    host::complete(ID, context.settings.read_timeout, async {
        let store = context.host.store();
        let position = store.resource_value(&request.name, "progress", request.at.as_ref())?.resolved_at;
        let mut fixed = store.cut(&request.name)?;
        fixed.frontier = position.sequence;
        response_at(&store, &fixed, NonZeroUsize::new(context.settings.page_items).unwrap())
    })
    .await
}

pub fn response_at(store: &Store, fixed: &ExecutionCut, page_items: NonZeroUsize) -> Result<ExecutionsStepsResponse> {
    let snapshot = execution_state::evidence_at(store, fixed, page_items)?;
    let origin =
        snapshot.execution.origin.as_ref().ok_or_else(|| Error::Invalid("List Execution Steps: Restore Its Frozen Origin".into()))?;
    let case = execution_state::case(origin)?;
    let mut steps = Vec::new();
    for (index, step) in case.steps.iter().enumerate() {
        let metadata =
            step.metadata.as_ref().ok_or_else(|| Error::Invalid("List Execution Steps: Restore Its Frozen Step Metadata".into()))?;
        let identifier = metadata.identifier.clone();
        let evidence: Vec<_> = snapshot.evidence.iter().filter(|entry| entry.step == identifier).collect();
        let started_at =
            evidence.iter().find(|entry| entry.kind == "step:start").map(|entry| crate::host::projection::timestamp(entry.receipt_ns));
        let finished_at =
            evidence.iter().rev().find(|entry| entry.kind == "step:end").map(|entry| crate::host::projection::timestamp(entry.receipt_ns));
        let status = if finished_at.is_some() {
            if evidence.iter().any(|entry| entry.kind == "step:failure") { StepStatus::Failed } else { StepStatus::Passed }
        } else if started_at.is_some() {
            if snapshot.state == ExecutionState::Sealed { StepStatus::Cancelled } else { StepStatus::Running }
        } else {
            StepStatus::Pending
        };
        steps.push(ExecutionStep {
            step_id: identifier,
            metadata: step.metadata.clone(),
            ordinal: index as u64,
            status: status as i32,
            started_at,
            finished_at,
            ..Default::default()
        });
    }
    let position = Position {
        history: Some(History {
            store: store.identity().into(),
            epoch: fixed.execution.identifier.clone(),
        }),
        sequence: fixed.frontier,
    };
    Ok(ExecutionsStepsResponse { steps, resolved_at: Some(position) })
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use std::error::Error;

    use super::{NonZeroUsize, StepStatus, Store, response_at};
    use crate::host::executions::parameters;
    use dokime_protocol::host::ExecutionParameterSet;
    use dokime_protocol::model::{Metadata, Origin, PluginRun, Step, TestCase, Value, origin, value};
    use dokime_storage::{Evidence, Execution};

    #[test]
    fn fixed_step_responses_exclude_later_step_evidence() -> Result<(), Box<dyn Error>> {
        // Create an Execution with Frozen Steps and Parameters
        let root = tempfile::tempdir()?;
        let store = Store::open(root.path())?;
        let execution = Execution {
            identifier: "execution".into(),
            cycle: "cycle".into(),
            origin: Some(Origin {
                kind: Some(origin::Kind::PluginRun(PluginRun {
                    plugin: "plugin".into(),
                    template: "template".into(),
                    digest: vec![1],
                    started_at: Some(Default::default()),
                    test_case: Some(TestCase {
                        steps: vec![Step {
                            metadata: Some(Metadata {
                                identifier: "measure".into(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }],
                        parameters: [("voltage".into(), Value { kind: Some(value::Kind::Integer(24)) })].into(),
                        ..Default::default()
                    }),
                })),
            }),
            ..Default::default()
        };
        store.create(&execution)?;
        let pending = store.cut(&execution.identifier)?;

        // Capture a Running Step Before Its Failure and Completion
        store.append(
            &execution.identifier,
            Evidence {
                kind: "step:start".into(),
                step: "measure".into(),
                receipt_ns: 1_000_000_000,
                ..Default::default()
            },
        )?;
        let running = store.cut(&execution.identifier)?;
        store.append(
            &execution.identifier,
            Evidence {
                kind: "step:failure".into(),
                step: "measure".into(),
                receipt_ns: 2_000_000_000,
                ..Default::default()
            },
        )?;
        store.append(
            &execution.identifier,
            Evidence {
                kind: "step:end".into(),
                step: "measure".into(),
                receipt_ns: 3_000_000_000,
                ..Default::default()
            },
        )?;
        let failed = store.cut(&execution.identifier)?;

        // Read Every Response Through Its Selected Cut, Across Bounded Pages
        let page_items = NonZeroUsize::new(1).unwrap();
        let pending_response = response_at(&store, &pending, page_items)?;
        let running_response = response_at(&store, &running, page_items)?;
        let failed_response = response_at(&store, &failed, page_items)?;
        assert_eq!(pending_response.steps[0].status, StepStatus::Pending as i32);
        assert!(pending_response.steps[0].started_at.is_none());
        assert_eq!(running_response.steps[0].status, StepStatus::Running as i32);
        assert!(running_response.steps[0].started_at.is_some());
        assert!(running_response.steps[0].finished_at.is_none());
        assert_eq!(running_response.resolved_at.as_ref().unwrap().sequence, running.frontier);
        assert_eq!(failed_response.steps[0].status, StepStatus::Failed as i32);
        assert!(failed_response.steps[0].finished_at.is_some());

        // Preserve the Frozen Parameter Contract at the Same Evidence Cut
        let inputs = parameters::response_at(&store, &running, ExecutionParameterSet::Inputs as i32)?;
        let outputs = parameters::response_at(&store, &running, ExecutionParameterSet::Outputs as i32)?;
        assert_eq!(
            inputs.parameters[0].value,
            execution.origin.as_ref().and_then(|origin| match origin.kind.as_ref() {
                Some(origin::Kind::PluginRun(run)) => run.test_case.as_ref()?.parameters.get("voltage").cloned(),
                _ => None,
            })
        );
        assert_eq!(inputs.resolved_at, running_response.resolved_at);
        assert!(outputs.parameters.is_empty());
        assert!(parameters::response_at(&store, &running, ExecutionParameterSet::Unspecified as i32).is_err());
        Ok(())
    }
}

crate::native::unary!(ExecutionsStepsRequest, ExecutionsStepsResponse, context, "dokime.host.v1", "Executions", "Steps");
