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
const TERMINAL_CASE_BUDGET: Duration = Duration::from_millis(300);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

pub(crate) async fn large_cycles_order() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let mut owner = fixture(root.path(), gate.clone()).await;
    let mut suite = common::fixture_suite(&["passed"]);
    let parameter = &mut suite.test_case_templates[0].parameters[0];
    parameter.expansion = dokime_app::definition::ParameterExpansion::Cartesian;
    parameter.value_type = dokime_app::core::ValueType::Integer;
    parameter.expression = dokime_app::definition::ParameterExpr::Range {
        start: 0,
        stop: 256,
        step: 1,
    };
    common::register_fixture_execution(&owner.handle(), suite).await;
    let server = Server::start(owner.handle(), Config::default())
        .await
        .unwrap();
    let mut client = ExecutionClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let id = client
        .create_cycle(create(&owner))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let summary = client
        .get_cycle(v1::GetCycleRequest {
            cycle_id: id.clone(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(summary.case_count, 257);
    assert_eq!(summary.execution_count, 257);
    let mut request = v1::ListCasesRequest {
        cycle_id: id.clone(),
        limit: 73,
        ..Default::default()
    };
    let mut cases = Vec::new();
    loop {
        let page = client
            .list_cases(request.clone())
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
            .list_cases(v1::ListCasesRequest {
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
            .list_cases(v1::ListCasesRequest {
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
    common::register_fixture_execution(&owner.handle(), common::fixture_suite(&["passed"])).await;
    client.create_cycle(create(&owner)).await.unwrap();
    assert_eq!(
        client.list_cases(request).await.unwrap_err().code(),
        Code::Aborted
    );
    let list = client
        .list_cycles(v1::ListCyclesRequest::default())
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
    let mut request = v1::ListExecutionsRequest {
        cycle_id: id.clone(),
        limit: 73,
        ..Default::default()
    };
    let mut attempts = Vec::new();
    loop {
        let page = client
            .list_executions(request.clone())
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
        assert_eq!(execution.status(), v1::ExecutionStatus::Skipped);
    }
    assert_eq!(
        client
            .list_executions(v1::ListExecutionsRequest {
                cycle_id: id,
                after_execution_id: "missing".into(),
                cursor: list.cursor.clone(),
                ..Default::default()
            })
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    assert_eq!(
        client
            .list_cycles(v1::ListCyclesRequest {
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

pub(crate) async fn large_run_preparation() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(Gate::default());
    let mut owner = fixture(root.path(), gate.clone()).await;
    let mut suite = common::fixture_suite(&["passed"]);
    let parameter = &mut suite.test_case_templates[0].parameters[0];
    parameter.expansion = dokime_app::definition::ParameterExpansion::Cartesian;
    parameter.value_type = dokime_app::core::ValueType::Integer;
    parameter.expression = dokime_app::definition::ParameterExpr::Range {
        start: 0,
        stop: PREPARATION_CASES as i64 - 1,
        step: 1,
    };
    common::register_fixture_execution(&owner.handle(), suite).await;
    let server = Server::start(owner.handle(), Config::default())
        .await
        .unwrap();
    let mut client = ExecutionClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let id = client
        .create_cycle(create(&owner))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let outcome = client
        .run_cycle(run(&id))
        .await
        .unwrap()
        .into_inner()
        .outcome
        .unwrap();
    assert!(matches!(outcome, v1::run_reply::Outcome::Accepted(_)));
    tokio::time::timeout(Duration::from_secs(10), async {
        client.stop_cycle(control(&id)).await.unwrap();
        let summary = client
            .get_cycle(v1::GetCycleRequest {
                cycle_id: id.clone(),
            })
            .await
            .unwrap()
            .into_inner();
        assert!(matches!(
            summary.status(),
            v1::CycleStatus::Stopping | v1::CycleStatus::Stopped
        ));
    })
    .await
    .expect("stop did not interrupt preparation promptly");
    // Terminal Acknowledgement Also Publishes One Durable Report Per Case.
    // Keep The Stop Deadline Separate From That Bounded Storage Work.
    let after = tokio::time::timeout(TERMINAL_CASE_BUDGET * PREPARATION_CASES as u32, async {
        loop {
            let summary = client
                .get_cycle(v1::GetCycleRequest {
                    cycle_id: id.clone(),
                })
                .await
                .unwrap()
                .into_inner();
            if summary.status() == v1::CycleStatus::Stopped {
                break summary;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(after.case_count, PREPARATION_CASES as u64);
    assert_eq!(after.execution_count, PREPARATION_CASES as u64);
    assert!(
        tokio::time::timeout(Duration::from_millis(1), gate.started.notified())
            .await
            .is_err(),
        "stopping preparation must prevent provider dispatch"
    );
    let mut request = v1::ListExecutionsRequest {
        cycle_id: id.clone(),
        limit: 73,
        ..Default::default()
    };
    let mut count = 0;
    loop {
        let page = client
            .list_executions(request.clone())
            .await
            .unwrap()
            .into_inner();
        for execution in &page.executions {
            assert!(matches!(
                execution.status(),
                v1::ExecutionStatus::Cancelled | v1::ExecutionStatus::Skipped
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
