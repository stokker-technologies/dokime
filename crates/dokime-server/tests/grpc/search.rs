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

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn labelled(owner: &Runtime, identifier: &str, label: &str) -> v1::CreateCycleRequest {
    v1::CreateCycleRequest {
        metadata: Some(v1::Metadata {
            identifier: identifier.into(),
            label: label.into(),
            description: None,
            ..Default::default()
        }),
        ..create(owner)
    }
}

async fn walk(
    client: &mut ExecutionClient<tonic::transport::Channel>,
    filter: v1::CycleFilter,
    order: v1::CycleOrder,
    limit: u32,
) -> Vec<String> {
    let mut request = v1::ListCyclesRequest {
        limit,
        filter: Some(filter),
        order: Some(order),
        ..Default::default()
    };
    let mut walked = Vec::new();
    loop {
        let page = client
            .list_cycles(request.clone())
            .await
            .unwrap()
            .into_inner();
        assert!(page.cycles.len() as u32 <= limit);
        walked.extend(
            page.cycles
                .iter()
                .map(|cycle| cycle.metadata.as_ref().unwrap().label.clone()),
        );
        let Some(next) = page.next_after_cycle_id else {
            break;
        };
        request.after_cycle_id = next;
        request.cursor = page.cursor;
    }
    walked
}

/* === Tests === */

pub(crate) async fn cycle_search_once() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture(root.path(), Arc::new(Gate::default())).await;
    common::register_fixture_execution(&owner.handle(), common::fixture_suite(&["passed"])).await;
    let server = Server::start(owner.handle(), Config::default())
        .await
        .unwrap();
    let mut client = ExecutionClient::connect(server.local_endpoint())
        .await
        .unwrap();
    for (identifier, label) in [
        ("grpc.burn-c", "Burn In Gamma"),
        ("grpc.calibration", "Calibration Sweep"),
        ("grpc.burn-a", "Burn In Alpha"),
        ("grpc.burn-b", "Burn In Beta"),
    ] {
        client
            .create_cycle(labelled(&owner, identifier, label))
            .await
            .unwrap();
    }
    let by_label = v1::CycleOrder {
        key: v1::CycleOrderKey::Label as i32,
        descending: false,
    };

    // Text narrows on the server, and one cycle per page still yields the
    // whole match set once, in the declared order.
    let walked = walk(
        &mut client,
        v1::CycleFilter {
            text: "BURN in".into(),
            ..Default::default()
        },
        by_label,
        1,
    )
    .await;
    assert_eq!(walked, ["Burn In Alpha", "Burn In Beta", "Burn In Gamma"]);

    // The same matches, in one page, are the same sequence.
    let single = walk(
        &mut client,
        v1::CycleFilter {
            text: "burn in".into(),
            ..Default::default()
        },
        by_label,
        50,
    )
    .await;
    assert_eq!(single, walked);

    // Text also matches the suite label every fixture cycle admits.
    let by_suite = walk(
        &mut client,
        v1::CycleFilter {
            text: "fixture".into(),
            ..Default::default()
        },
        by_label,
        2,
    )
    .await;
    assert_eq!(by_suite.len(), 4);

    // A status the runtime has not reached selects nothing; the one it has
    // selects everything.
    assert!(
        walk(
            &mut client,
            v1::CycleFilter {
                statuses: vec![v1::CycleStatus::Completed as i32],
                ..Default::default()
            },
            by_label,
            10,
        )
        .await
        .is_empty()
    );
    assert_eq!(
        walk(
            &mut client,
            v1::CycleFilter {
                statuses: vec![v1::CycleStatus::Ready as i32],
                ..Default::default()
            },
            by_label,
            10,
        )
        .await
        .len(),
        4
    );

    // A continuation the filter excludes is rejected, not silently restarted.
    let excluded = client
        .list_cycles(v1::ListCyclesRequest::default())
        .await
        .unwrap()
        .into_inner();
    let outsider = excluded
        .cycles
        .iter()
        .find(|cycle| cycle.metadata.as_ref().unwrap().label == "Calibration Sweep")
        .unwrap()
        .cycle_id
        .clone();
    assert_eq!(
        client
            .list_cycles(v1::ListCyclesRequest {
                after_cycle_id: outsider,
                cursor: excluded.cursor.clone(),
                filter: Some(v1::CycleFilter {
                    text: "burn in".into(),
                    ..Default::default()
                }),
                order: Some(by_label),
                ..Default::default()
            })
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );

    // An undeclared status number is refused rather than widened away.
    assert_eq!(
        client
            .list_cycles(v1::ListCyclesRequest {
                filter: Some(v1::CycleFilter {
                    statuses: vec![v1::CycleStatus::Unspecified as i32],
                    ..Default::default()
                }),
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

pub(crate) async fn listed_cases_page() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = fixture(root.path(), Arc::new(Gate::default())).await;
    common::register_fixture_execution(&owner.handle(), common::fixture_suite(&["passed"])).await;
    let server = Server::start(owner.handle(), Config::default())
        .await
        .unwrap();
    let mut client = ExecutionClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let cycle_id = client
        .create_cycle(create(&owner))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let page = client
        .list_cases(v1::ListCasesRequest {
            cycle_id,
            ..Default::default()
        })
        .await
        .unwrap()
        .into_inner();
    assert!(!page.cases.is_empty());
    for case in &page.cases {
        // Creation materializes one undispatched attempt per case, which is
        // queued work rather than a skip and is the attempt a run dispatches.
        assert_eq!(case.status(), v1::CaseStatus::Queued);
        assert_eq!(case.attempt_count, 1);
        assert!(!case.rerun_scheduled);
        assert_eq!(case.latest_execution_status(), v1::ExecutionStatus::Skipped);
        assert!(case.latest_execution_id.is_some());
        let actions = case.actions.as_ref().unwrap();
        assert!(actions.run);
        assert!(actions.schedule_rerun);
        assert!(actions.delete);
        assert!(actions.unavailable_reasons.is_empty());
    }

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}
