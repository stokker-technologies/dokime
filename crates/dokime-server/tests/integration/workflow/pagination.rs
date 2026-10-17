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

/* === Constants === */
const PREPARATION_CASES: usize = 48;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

#[tokio::test]
async fn large_cycles_order() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let mut owner = fixture(root.path(), gate.clone()).await;
    let mut suite = support::fixture_suite(&["passed"]);
    let parameter = &mut suite.test_case_templates[0].parameters[0];
    parameter.expansion = dokime_runtime::definition::ParameterExpansion::Cartesian;
    parameter.value_type = dokime_runtime::ValueType::Integer;
    parameter.expression = dokime_runtime::definition::ParameterExpr::Range {
        start: 0,
        stop: 256,
        step: 1,
    };
    support::register_fixture_execution(&owner.handle(), suite).await;
    let server = Server::start(owner.handle(), Config::default())
        .await
        .unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let id = support::planned_cycle(&client, Request::new(create()), selection_plan(&owner))
        .await
        .unwrap();
    let summary = client
        .unary::<contracts::execution::GetCycle>(contracts::execution::GetCycle {
            cycle_id: id.clone(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(summary.case_count, 257);
    assert_eq!(summary.execution_count, 257);
    let mut request = contracts::execution::ListCases {
        cycle_id: id.clone(),
        limit: 73,
        ..Default::default()
    };
    let mut cases = Vec::new();
    loop {
        let page = client
            .unary::<contracts::execution::ListCases>(request.clone())
            .await
            .unwrap()
            .into_inner();
        assert!(page.cases.len() <= 73);
        cases.extend(page.cases);
        let Some(next) = page.next_after_case_id else {
            break;
        };
        request.after_case_id = next;
        request.cursor = page.cursor;
    }
    assert_eq!(cases.len(), 257);
    let native = owner.handle().watch_state().latest();
    let native_cycle = native
        .state()
        .cycle_views()
        .find(|cycle| cycle.test_cycle_id.as_str() == id)
        .unwrap();
    for (ordinal, (case, native)) in cases.iter().zip(native_cycle.cases()).enumerate() {
        assert_eq!(case.ordinal, ordinal as u64);
        assert_eq!(case.case_id, native.test_case_id.as_str());
    }
    assert_eq!(
        client
            .unary::<contracts::execution::ListCases>(contracts::execution::ListCases {
                cycle_id: id.clone(),
                after_case_id: "missing".into(),
                cursor: summary.observed_cursor.clone(),
                ..Default::default()
            })
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    assert_eq!(
        client
            .unary::<contracts::execution::ListCases>(contracts::execution::ListCases {
                cycle_id: id.clone(),
                after_case_id: cases[0].case_id.clone(),
                ..Default::default()
            })
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    // A single-case cycle is enough to advance the observed cursor. The first
    // cycle still crosses the 256-case boundary through every RPC projection.
    support::register_fixture_execution(&owner.handle(), support::fixture_suite(&["passed"])).await;
    client
        .unary::<contracts::execution::CreateCycle>(create())
        .await
        .unwrap();
    assert_eq!(
        client
            .unary::<contracts::execution::ListCases>(request)
            .await
            .unwrap_err()
            .code(),
        Code::Aborted
    );
    let list = client
        .unary::<contracts::execution::ListCycles>(
            contracts::execution::ListCycles::default(),
        )
        .await
        .unwrap()
        .into_inner();
    let large = list
        .cycles
        .iter()
        .find(|cycle| cycle.cycle_id == id)
        .unwrap();
    assert_eq!(large.case_count, 257);
    assert_eq!(large.execution_count, 257);
    let mut request = contracts::execution::ListExecutions {
        cycle_id: id.clone(),
        limit: 73,
        ..Default::default()
    };
    let mut attempts = Vec::new();
    loop {
        let page = client
            .unary::<contracts::execution::ListExecutions>(request.clone())
            .await
            .unwrap()
            .into_inner();
        assert!(page.executions.len() <= 73);
        attempts.extend(page.executions);
        let Some(next) = page.next_after_execution_id else {
            break;
        };
        request.after_execution_id = next;
        request.cursor = page.cursor;
    }
    assert_eq!(attempts.len(), 257);
    for (ordinal, (execution, native)) in attempts
        .iter()
        .zip(&native_cycle.test_executions)
        .enumerate()
    {
        assert_eq!(execution.ordinal, ordinal as u64);
        assert_eq!(execution.execution_id, native.test_execution_id.as_str());
        assert_eq!(execution.case_id, native.test_case_id.as_str());
        assert_eq!(
            execution.status,
            contracts::execution::ExecutionStatus::Skipped
        );
    }
    assert_eq!(
        client
            .unary::<contracts::execution::ListExecutions>(
                contracts::execution::ListExecutions {
                    cycle_id: id,
                    after_execution_id: "missing".into(),
                    cursor: list.cursor.clone(),
                    ..Default::default()
                }
            )
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    assert_eq!(
        client
            .unary::<contracts::execution::ListCycles>(contracts::execution::ListCycles {
                after_cycle_id: "00000000-0000-0000-0000-000000000001".into(),
                cursor: list.cursor,
                ..Default::default()
            })
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn large_run_preparation() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let mut owner = fixture(root.path(), gate.clone()).await;
    let mut suite = support::fixture_suite(&["passed"]);
    let parameter = &mut suite.test_case_templates[0].parameters[0];
    parameter.expansion = dokime_runtime::definition::ParameterExpansion::Cartesian;
    parameter.value_type = dokime_runtime::ValueType::Integer;
    parameter.expression = dokime_runtime::definition::ParameterExpr::Range {
        start: 0,
        stop: PREPARATION_CASES as i64 - 1,
        step: 1,
    };
    support::register_fixture_execution(&owner.handle(), suite).await;
    let server = Server::start(owner.handle(), Config::default())
        .await
        .unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let id = support::planned_cycle(&client, Request::new(create()), selection_plan(&owner))
        .await
        .unwrap();
    let outcome = client
        .unary::<contracts::execution::RunCycle>(run(&id))
        .await
        .unwrap()
        .into_inner()
        .outcome
        .unwrap();
    assert!(matches!(
        outcome,
        contracts::execution::RunReplyOutcome::Accepted(_)
    ));
    {
        client
            .unary::<contracts::execution::StopCycle>(contracts::execution::StopCycle { cycle_id: id.clone(), context: None })
            .await
            .unwrap();
        let summary = client
            .unary::<contracts::execution::GetCycle>(contracts::execution::GetCycle {
                cycle_id: id.clone(),
            })
            .await
            .unwrap()
            .into_inner();
        assert!(matches!(
            summary.status,
            contracts::execution::CycleStatus::Stopping
                | contracts::execution::CycleStatus::Stopped
        ));
    }
    // Terminal Acknowledgement Also Publishes One Durable Report Per Case.
    let after = support::on_state_change(&owner.handle(), async || {
        let summary = client
            .unary::<contracts::execution::GetCycle>(contracts::execution::GetCycle {
                cycle_id: id.clone(),
            })
            .await
            .unwrap()
            .into_inner();
        if summary.status == contracts::execution::CycleStatus::Stopped {
            return Some(summary);
        }
        None
    })
    .await;
    assert_eq!(after.case_count, PREPARATION_CASES as u64);
    assert_eq!(after.execution_count, PREPARATION_CASES as u64);
    // A Dispatch Would Have Stored The Gate's Permit, So One Poll Decides
    assert!(
        futures_util::FutureExt::now_or_never(gate.started.notified()).is_none(),
        "stopping preparation must prevent provider dispatch"
    );
    let mut request = contracts::execution::ListExecutions {
        cycle_id: id.clone(),
        limit: 73,
        ..Default::default()
    };
    let mut count = 0;
    loop {
        let page = client
            .unary::<contracts::execution::ListExecutions>(request.clone())
            .await
            .unwrap()
            .into_inner();
        for execution in &page.executions {
            assert!(matches!(
                execution.status,
                contracts::execution::ExecutionStatus::Cancelled
                    | contracts::execution::ExecutionStatus::Skipped
            ));
        }
        count += page.executions.len();
        let Some(next) = page.next_after_execution_id else {
            break;
        };
        request.after_execution_id = next;
        request.cursor = page.cursor;
    }
    assert_eq!(count, PREPARATION_CASES);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */
