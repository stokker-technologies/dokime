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
use super::*;
use dokime_app::{capabilities::execution::authoring::SidecarStepContext, core::Value};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

pub(crate) async fn inspection_separates_outputs() {
    let Gated {
        root: _root,
        gate,
        mut owner,
        server,
        execution: mut client,
        cycle: id,
    } = Gated::open().await;
    let cycle = client
        .get_cycle(authenticated(v1::GetCycleRequest {
            cycle_id: id.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert!(cycle.created_at.is_some());
    assert!(cycle.updated_at.is_some());
    assert!(cycle.started_at.is_none());
    assert!(cycle.result.is_none());
    let case = client
        .list_cases(authenticated(v1::ListCasesRequest {
            cycle_id: id.clone(),
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner()
        .cases
        .remove(0);
    let detail = client
        .get_case(authenticated(v1::GetCaseRequest {
            cycle_id: id.clone(),
            case_id: case.case_id.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(detail.summary.as_ref(), Some(&case));
    assert!(detail.parameter_count > 0);
    let planned = client
        .list_case_parameters(authenticated(v1::ListCaseParametersRequest {
            cycle_id: id.clone(),
            case_id: case.case_id.clone(),
            ..Default::default()
        }))
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
            .kind,
        Some(v1::value::Kind::TextValue("passed".into()))
    );
    assert_eq!(
        client
            .get_case(authenticated(v1::GetCaseRequest {
                cycle_id: id.clone(),
                case_id: "missing".into()
            }))
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
    let refused = client
        .run_execution(authenticated(v1::RunExecutionRequest {
            mode: 99,
            execution_id: selected.clone(),
            idempotency_key: Some(uuid::Uuid::now_v7().to_string()),
            context: None,
        }))
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
    client
        .run_execution(authenticated(v1::RunExecutionRequest {
            mode: Default::default(),
            execution_id: selected,
            idempotency_key: Some(uuid::Uuid::now_v7().to_string()),
            context: None,
        }))
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(5), gate.started.notified())
        .await
        .unwrap();
    let handle = owner.handle();
    let native = handle.cycles().await.remove(0).test_executions.remove(0);
    let execution_id = native.test_execution_id.to_string();
    assert_eq!(
        client
            .get_execution(v1::GetExecutionRequest {
                execution_id: execution_id.clone()
            })
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    let request = |set| v1::ListExecutionParametersRequest {
        execution_id: execution_id.clone(),
        set,
        ..Default::default()
    };
    let before = client
        .list_execution_parameters(authenticated(request(
            v1::ExecutionParameterSet::Inputs as i32,
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
        .list_execution_parameters(authenticated(request(
            v1::ExecutionParameterSet::Inputs as i32,
        )))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(after.parameters, before.parameters);
    let mut output_request = request(v1::ExecutionParameterSet::Outputs as i32);
    output_request.limit = 1;
    let first = client
        .list_execution_parameters(authenticated(output_request.clone()))
        .await
        .unwrap()
        .into_inner();
    let current_outputs = client
        .list_execution_parameters(authenticated(request(
            v1::ExecutionParameterSet::Outputs as i32,
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
        counter.value.as_ref().unwrap().kind,
        Some(v1::value::Kind::IntegerValue(i64::MAX))
    );
    assert_eq!(counter.source(), v1::ParameterSource::Runtime);
    // The shared name exists in both inputs and outputs: scope validation must
    // reject switching sets, not rely on an absent continuation marker.
    let mismatch = v1::ListExecutionParametersRequest {
        set: v1::ExecutionParameterSet::Inputs as i32,
        after_name: "fixture.result".into(),
        cursor: first.cursor.clone(),
        ..request(v1::ExecutionParameterSet::Inputs as i32)
    };
    assert_eq!(
        client
            .list_execution_parameters(authenticated(mismatch))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    let malformed = v1::ListExecutionParametersRequest {
        cursor: Some(v1::ParameterCursor {
            observed: None,
            selection: first.cursor.as_ref().unwrap().selection.clone(),
        }),
        ..request(v1::ExecutionParameterSet::Outputs as i32)
    };
    assert_eq!(
        client
            .list_execution_parameters(authenticated(malformed))
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
            .list_execution_parameters(authenticated(output_request))
            .await
            .unwrap_err()
            .code(),
        Code::Aborted
    );
    let outputs = client
        .list_execution_parameters(authenticated(request(
            v1::ExecutionParameterSet::Outputs as i32,
        )))
        .await
        .unwrap()
        .into_inner();
    assert!(
        outputs
            .parameters
            .iter()
            .any(|entry| entry.name == "nullable"
                && entry.value.as_ref().unwrap().kind == Some(v1::value::Kind::NullValue(0)))
    );
    assert_eq!(
        client
            .list_execution_parameters(authenticated(request(0)))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    assert_eq!(
        client
            .list_execution_parameters(authenticated(request(999)))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    let admitted = client
        .get_execution(authenticated(v1::GetExecutionRequest {
            execution_id: execution_id.clone(),
        }))
        .await
        .unwrap()
        .into_inner();
    let controls = admitted.controls.as_ref().unwrap();
    assert!(controls.can_pause && controls.can_stop && !controls.can_resume);
    client
        .pause_execution(authenticated(v1::ExecutionControlRequest {
            execution_id: execution_id.clone(),
            context: None,
        }))
        .await
        .unwrap();
    let paused = client
        .get_execution(authenticated(v1::GetExecutionRequest {
            execution_id: execution_id.clone(),
        }))
        .await
        .unwrap()
        .into_inner()
        .controls
        .unwrap();
    assert_eq!(paused.request_id, controls.request_id);
    assert!(paused.paused && paused.can_resume && !paused.can_pause);
    client
        .resume_execution(authenticated(v1::ExecutionControlRequest {
            execution_id: execution_id.clone(),
            context: None,
        }))
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
    tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let final_detail = client
                .get_execution(authenticated(v1::GetExecutionRequest {
                    execution_id: execution_id.clone(),
                }))
                .await
                .unwrap()
                .into_inner();
            if final_detail.finished_at.is_some() {
                assert!(final_detail.controls.is_none());
                assert_eq!(
                    final_detail.summary.unwrap().status(),
                    v1::ExecutionStatus::Passed
                );
                assert_eq!(
                    final_detail.result.unwrap().status(),
                    v1::ResultStatus::Passed
                );
                break;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    drop(context);
    drop(handle);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

pub(crate) async fn planned_params_limit() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture(root.path(), Arc::new(Gate::default())).await;
    let mut suite = common::fixture_suite(&["passed"]);
    let prototype = suite.test_case_templates[0].parameters[0].clone();
    suite.test_case_templates[0].parameters = (0..257)
        .map(|index| {
            let mut parameter = prototype.clone();
            parameter.parameter_id = dokime_app::core::ParameterId::new();
            parameter.metadata = common::metadata(&format!("p{index:03}"), "Parameter");
            parameter.expression = dokime_app::definition::ParameterExpr::Literal {
                value: Value::Integer(index),
            };
            parameter.value_type = dokime_app::core::ValueType::Integer;
            parameter
        })
        .collect();
    common::register_fixture_execution(&owner.handle(), suite).await;
    let server = Server::start(owner.handle(), Config::local())
        .await
        .unwrap();
    let mut client = ExecutionClient::connect(server.local_endpoint())
        .await
        .unwrap();
    // Native registered suites can exceed the transport's creation schema
    // bound. Inspection must still expose every existing planned parameter.
    let cycle_id = owner
        .handle()
        .start_registered_cycle(
            "fixture.suite",
            common::metadata("large.case", "Large case"),
            false,
            dokime_app::runtime::command::CommandContext::default(),
        )
        .await
        .unwrap()
        .to_string();
    let other_cycle = owner
        .handle()
        .start_registered_cycle(
            "fixture.suite",
            common::metadata("other.case", "Other case"),
            false,
            dokime_app::runtime::command::CommandContext::default(),
        )
        .await
        .unwrap()
        .to_string();
    let case_id = client
        .list_cases(v1::ListCasesRequest {
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
        .get_case(v1::GetCaseRequest {
            cycle_id: cycle_id.clone(),
            case_id: case_id.clone(),
        })
        .await
        .unwrap()
        .into_inner();
    let execution_id = client
        .list_executions(v1::ListExecutionsRequest {
            cycle_id: cycle_id.clone(),
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner()
        .executions
        .remove(0)
        .execution_id;
    assert_eq!(detail.parameter_count, 257);
    let mut request = v1::ListCaseParametersRequest {
        cycle_id: cycle_id.clone(),
        case_id: case_id.clone(),
        limit: 73,
        ..Default::default()
    };
    let mut all = Vec::new();
    loop {
        let page = client
            .list_case_parameters(request.clone())
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
                client.list_case_parameters(wrong).await.unwrap_err().code(),
                Code::InvalidArgument
            );
            let mut wrong = request.clone();
            wrong.case_id = "another-case".into();
            wrong.cursor = cursor.clone();
            assert_eq!(
                client.list_case_parameters(wrong).await.unwrap_err().code(),
                Code::InvalidArgument
            );
            assert_eq!(
                client
                    .list_execution_parameters(v1::ListExecutionParametersRequest {
                        execution_id: execution_id.clone(),
                        set: v1::ExecutionParameterSet::Inputs as i32,
                        cursor: cursor.clone(),
                        ..Default::default()
                    })
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
                    .list_case_parameters(unknown)
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
            item.value.as_ref().unwrap().kind,
            Some(v1::value::Kind::IntegerValue(index as i64))
        );
    }
    assert_eq!(
        client
            .get_cycle(v1::GetCycleRequest { cycle_id })
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
