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
use dokime_domain::model;
use super::*;
use {
    dokime_runtime::Value,
    dokime_runtime::domains::execution::authoring::SidecarStepContext,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

#[tokio::test]
async fn inspection_separates_outputs() {
    let Gated {
        root: _root,
        gate,
        mut owner,
        service: _service,
        server,
        execution: client,
        cycle: id,
    } = Gated::open().await;
    let cycle = client
        .unary::<contracts::execution::GetCycle>(authenticated(
            contracts::execution::GetCycle {
                cycle_id: id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert!(cycle.created_at.is_some());
    assert!(cycle.updated_at.is_some());
    assert!(cycle.started_at.is_none());
    assert!(cycle.result.is_none());
    let case = client
        .unary::<contracts::execution::ListCases>(authenticated(
            contracts::execution::ListCases {
                cycle_id: id.clone(),
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .cases
        .remove(0);
    let detail = client
        .unary::<contracts::execution::GetCase>(authenticated(
            contracts::execution::GetCase {
                cycle_id: id.clone(),
                case_id: case.case_id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(detail.summary.as_ref(), Some(&case));
    assert!(detail.parameter_count > 0);
    let planned = client
        .unary::<contracts::execution::ListCaseParameters>(authenticated(
            contracts::execution::ListCaseParameters {
                cycle_id: id.clone(),
                case_id: case.case_id.clone(),
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(
        planned
            .parameters
            .iter()
            .find(|item| item.name == "fixture.result")
            .unwrap()
            .value
            .as_ref()
            .unwrap()
            .to_core()
            .unwrap(),
        Value::Text("passed".into())
    );
    assert_eq!(
        client
            .unary::<contracts::execution::GetCase>(authenticated(
                contracts::execution::GetCase {
                    cycle_id: id.clone(),
                    case_id: "missing".into()
                }
            ))
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    let selected = owner
        .handle()
        .cycles()
        .await
        .remove(0)
        .test_executions
        .remove(0)
        .test_execution_id
        .to_string();
    for unknown_variant in [false, true] {
        let refused = client
            .invalid_enum::<contracts::execution::RunExecution, model::execution::wire::ExecutionMode>(authenticated(contracts::execution::RunExecution {
                mode: Default::default(),
                execution_id: selected.clone(),
                idempotency_key: Some(uuid::Uuid::now_v7().to_string()),
                context: None,
            }), "mode", unknown_variant)
            .await
            .unwrap_err();
        assert_eq!(refused.code(), Code::InvalidArgument);
        assert_eq!(
            owner
                .handle()
                .cycles()
                .await
                .remove(0)
                .test_executions
                .remove(0)
                .started_at,
            None
        );
    }
    client
        .unary::<contracts::execution::RunExecution>(authenticated(
            contracts::execution::RunExecution {
                mode: Default::default(),
                execution_id: selected,
                idempotency_key: Some(uuid::Uuid::now_v7().to_string()),
                context: None,
            },
        ))
        .await
        .unwrap();
    gate.started.notified().await;
    let handle = owner.handle();
    let native = handle.cycles().await.remove(0).test_executions.remove(0);
    let execution_id = native.test_execution_id.to_string();
    assert_eq!(
        client
            .unary::<contracts::execution::GetExecution>(
                contracts::execution::GetExecution {
                    execution_id: execution_id.clone()
                }
            )
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    let request = |set| contracts::execution::ListExecutionParameters {
        execution_id: execution_id.clone(),
        set,
        limit: 0,
        after_name: String::new(),
        cursor: None,
    };
    let before = client
        .unary::<contracts::execution::ListExecutionParameters>(authenticated(request(
            model::execution::wire::ExecutionParameterSet::Inputs,
        )))
        .await
        .unwrap()
        .into_inner();
    let context = SidecarStepContext::new(handle.clone(), native.scope.clone());
    assert!(
        context
            .set_runtime_parameter_async("fixture.result", "changed output")
            .await
            .unwrap()
    );
    assert!(
        context
            .set_runtime_parameter_async("counter", i64::MAX)
            .await
            .unwrap()
    );
    assert!(
        context
            .set_runtime_parameter_async("nullable", Value::Null)
            .await
            .unwrap()
    );
    let after = client
        .unary::<contracts::execution::ListExecutionParameters>(authenticated(request(
            model::execution::wire::ExecutionParameterSet::Inputs,
        )))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(after.parameters, before.parameters);
    let mut output_request = request(model::execution::wire::ExecutionParameterSet::Outputs);
    output_request.limit = 1;
    let first = client
        .unary::<contracts::execution::ListExecutionParameters>(authenticated(
            output_request.clone(),
        ))
        .await
        .unwrap()
        .into_inner();
    let current_outputs = client
        .unary::<contracts::execution::ListExecutionParameters>(authenticated(request(
            model::execution::wire::ExecutionParameterSet::Outputs,
        )))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(first.parameters[0], current_outputs.parameters[0]);
    let counter = current_outputs
        .parameters
        .iter()
        .find(|item| item.name == "counter")
        .unwrap();
    assert_eq!(
        counter.value.as_ref().unwrap().to_core().unwrap(),
        Value::Integer(i64::MAX)
    );
    assert_eq!(
        counter.source,
        model::execution::wire::ParameterSource::Runtime
    );
    // The shared name exists in both inputs and outputs: scope validation must
    // reject switching sets, not rely on an absent continuation marker.
    let mismatch = contracts::execution::ListExecutionParameters {
        set: model::execution::wire::ExecutionParameterSet::Inputs,
        after_name: "fixture.result".into(),
        cursor: first.cursor.clone(),
        ..request(model::execution::wire::ExecutionParameterSet::Inputs)
    };
    assert_eq!(
        client
            .unary::<contracts::execution::ListExecutionParameters>(authenticated(mismatch))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    let malformed = contracts::execution::ListExecutionParameters {
        cursor: Some(model::execution::wire::ParameterCursor {
            observed: None,
            selection: first.cursor.as_ref().unwrap().selection.clone(),
        }),
        ..request(model::execution::wire::ExecutionParameterSet::Outputs)
    };
    assert_eq!(
        client
            .unary::<contracts::execution::ListExecutionParameters>(authenticated(malformed))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    output_request.after_name = first.next_after_name.unwrap();
    output_request.cursor = first.cursor;
    assert!(
        context
            .set_runtime_parameter_async("counter", i64::MIN)
            .await
            .unwrap()
    );
    assert_eq!(
        client
            .unary::<contracts::execution::ListExecutionParameters>(authenticated(output_request))
            .await
            .unwrap_err()
            .code(),
        Code::Aborted
    );
    let outputs = client
        .unary::<contracts::execution::ListExecutionParameters>(authenticated(request(
            model::execution::wire::ExecutionParameterSet::Outputs,
        )))
        .await
        .unwrap()
        .into_inner();
    assert!(
        outputs
            .parameters
            .iter()
            .any(|entry| entry.name == "nullable"
                && entry.value.as_ref().unwrap().to_core().unwrap() == Value::Null)
    );
    assert_eq!(
        client
            .unary::<contracts::execution::ListExecutionParameters>(authenticated(request(
                model::execution::wire::ExecutionParameterSet::Unspecified
            )))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    for unknown_variant in [false, true] {
        assert_eq!(
            client.invalid_enum::<contracts::execution::ListExecutionParameters, model::execution::wire::ExecutionParameterSet>(
                authenticated(request(model::execution::wire::ExecutionParameterSet::Unspecified)),
                "set", unknown_variant,
            ).await.unwrap_err().code(),
            Code::InvalidArgument,
        );
    }
    let admitted = client
        .unary::<contracts::execution::GetExecution>(authenticated(
            contracts::execution::GetExecution {
                execution_id: execution_id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let controls = admitted.controls.as_ref().unwrap();
    assert!(controls.can_pause && controls.can_stop && !controls.can_resume);
    client
        .unary::<contracts::execution::PauseExecution>(authenticated(
            contracts::execution::PauseExecution {
                execution_id: execution_id.clone(),
                context: None,
            },
        ))
        .await
        .unwrap();
    let paused = client
        .unary::<contracts::execution::GetExecution>(authenticated(
            contracts::execution::GetExecution {
                execution_id: execution_id.clone(),
            },
        ))
        .await
        .unwrap()
        .into_inner()
        .controls
        .unwrap();
    assert_eq!(paused.request_id, controls.request_id);
    assert!(paused.paused && paused.can_resume && !paused.can_pause);
    client
        .unary::<contracts::execution::ResumeExecution>(authenticated(
            contracts::execution::ResumeExecution {
                execution_id: execution_id.clone(),
                context: None,
            },
        ))
        .await
        .unwrap();
    assert_eq!(
        admitted.admitted_template_id.as_deref(),
        Some(case.template_id.as_str())
    );
    assert_eq!(
        admitted.scope.as_ref().unwrap().execution_id.as_deref(),
        Some(execution_id.as_str())
    );
    // Entering the cycle provider does not imply this particular attempt has
    // emitted an execution-start fact. Preserve the native timestamp presence.
    assert_eq!(admitted.started_at.is_some(), native.started_at.is_some());
    assert!(admitted.finished_at.is_none());
    assert_eq!(
        admitted.input_parameter_count,
        before.parameters.len() as u64
    );
    assert_eq!(
        admitted.output_parameter_count,
        outputs.parameters.len() as u64
    );
    gate.release.notify_one();
    support::on_state_change(&handle, async || {
        let final_detail = client
            .unary::<contracts::execution::GetExecution>(authenticated(
                contracts::execution::GetExecution {
                    execution_id: execution_id.clone(),
                },
            ))
            .await
            .unwrap()
            .into_inner();
        if final_detail.finished_at.is_some() {
            assert!(final_detail.controls.is_none());
            assert_eq!(
                final_detail.summary.unwrap().status,
                model::execution::wire::ExecutionStatus::Passed
            );
            assert_eq!(
                final_detail.result.unwrap().status,
                model::execution::wire::ResultStatus::Passed
            );
            return Some(());
        }
        None
    })
    .await;
    drop(context);
    drop(handle);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn planned_params_limit() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture(root.path(), Arc::new(Gate::default())).await;
    let mut suite = support::fixture_suite(&["passed"]);
    let prototype = suite.test_case_templates[0].parameters[0].clone();
    suite.test_case_templates[0].parameters = (0..257)
        .map(|index| {
            let mut parameter = prototype.clone();
            parameter.parameter_id = dokime_runtime::model::identity::new_id::<dokime_runtime::ParameterId>();
            parameter.metadata = support::metadata(&format!("p{index:03}"), "Parameter");
            parameter.expression = dokime_runtime::definition::ParameterExpr::Literal {
                value: Value::Integer(index),
            };
            parameter.value_type = dokime_runtime::ValueType::Integer;
            parameter
        })
        .collect();
    support::register_fixture_execution(&owner.handle(), suite.clone()).await;
    let server = Server::start(owner.handle(), Config::local())
        .await
        .unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    // Native registered suites can exceed the transport's creation schema
    // bound. Inspection must still expose every existing planned parameter.
    let cycle_id = support::cycle_with_cases(
        &owner.handle(),
        support::metadata("large.case", "Large case"),
        support::materialize_cases("fixture.suite", &suite),
    )
    .await
    .to_string();
    let other_cycle = support::cycle_with_cases(
        &owner.handle(),
        support::metadata("other.case", "Other case"),
        support::materialize_cases("fixture.suite", &suite),
    )
    .await
    .to_string();
    let case_id = client
        .unary::<contracts::execution::ListCases>(contracts::execution::ListCases {
            cycle_id: cycle_id.clone(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner()
        .cases
        .remove(0)
        .case_id;
    let detail = client
        .unary::<contracts::execution::GetCase>(contracts::execution::GetCase {
            cycle_id: cycle_id.clone(),
            case_id: case_id.clone(),
        })
        .await
        .unwrap()
        .into_inner();
    let execution_id = client
        .unary::<contracts::execution::ListExecutions>(
            contracts::execution::ListExecutions {
                cycle_id: cycle_id.clone(),
                ..Default::default()
            },
        )
        .await
        .unwrap()
        .into_inner()
        .executions
        .remove(0)
        .execution_id;
    assert_eq!(detail.parameter_count, 257);
    let mut request = contracts::execution::ListCaseParameters {
        cycle_id: cycle_id.clone(),
        case_id: case_id.clone(),
        limit: 73,
        ..Default::default()
    };
    let mut all = Vec::new();
    loop {
        let page = client
            .unary::<contracts::execution::ListCaseParameters>(request.clone())
            .await
            .unwrap()
            .into_inner();
        assert!(page.parameters.len() <= 73);
        if all.is_empty() {
            let cursor = page.cursor.clone();
            let mut wrong = request.clone();
            wrong.cycle_id = other_cycle.clone();
            wrong.cursor = cursor.clone();
            wrong.after_name = "p000".into();
            assert_eq!(
                client
                    .unary::<contracts::execution::ListCaseParameters>(wrong)
                    .await
                    .unwrap_err()
                    .code(),
                Code::InvalidArgument
            );
            let mut wrong = request.clone();
            wrong.case_id = "another-case".into();
            wrong.cursor = cursor.clone();
            assert_eq!(
                client
                    .unary::<contracts::execution::ListCaseParameters>(wrong)
                    .await
                    .unwrap_err()
                    .code(),
                Code::InvalidArgument
            );
            assert_eq!(
                client
                    .unary::<contracts::execution::ListExecutionParameters>(
                        contracts::execution::ListExecutionParameters {
                            execution_id: execution_id.clone(),
                            set: model::execution::wire::ExecutionParameterSet::Inputs,
                            limit: 0,
                            after_name: String::new(),
                            cursor: cursor.clone(),
                        }
                    )
                    .await
                    .unwrap_err()
                    .code(),
                Code::InvalidArgument
            );
            let mut unknown = request.clone();
            unknown.cursor = cursor;
            unknown.after_name = "unknown".into();
            assert_eq!(
                client
                    .unary::<contracts::execution::ListCaseParameters>(unknown)
                    .await
                    .unwrap_err()
                    .code(),
                Code::InvalidArgument
            );
        }
        all.extend(page.parameters);
        let Some(after) = page.next_after_name else {
            break;
        };
        request.after_name = after;
        request.cursor = page.cursor;
    }
    assert_eq!(all.len(), 257);
    for (index, item) in all.iter().enumerate() {
        assert_eq!(item.name, format!("p{index:03}"));
        assert_eq!(
            item.value.as_ref().unwrap().to_core().unwrap(),
            Value::Integer(index as i64)
        );
    }
    assert_eq!(
        client
            .unary::<contracts::execution::GetCycle>(contracts::execution::GetCycle {
                cycle_id
            })
            .await
            .unwrap()
            .into_inner()
            .case_count,
        1
    );
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */
