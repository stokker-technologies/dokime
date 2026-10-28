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
use dokime::domain::model;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn labelled(identifier: &str, label: &str, description: Option<&str>) -> contracts::execution::CreateCycle {
    contracts::execution::CreateCycle {
        metadata: Some(model::identity::wire::Metadata {
            identifier: identifier.into(),
            label: label.into(),
            description: description.map(Into::into),
            ..Default::default()
        }),
        ..create()
    }
}

async fn walk(
    client: &mut WireClient, filter: model::execution::wire::CycleFilter, order: model::execution::wire::CycleOrder, limit: u32,
) -> Vec<String> {
    let mut request = contracts::execution::ListCycles {
        limit,
        filter: Some(filter),
        order: Some(order),
        ..Default::default()
    };
    let mut walked = Vec::new();
    loop {
        let page = client.unary::<contracts::execution::ListCycles>(request.clone()).await.unwrap().into_inner();
        assert!(page.cycles.len() as u32 <= limit);
        walked.extend(page.cycles.iter().map(|cycle| cycle.metadata.as_ref().unwrap().label.clone()));
        let Some(next) = page.next_after_cycle_id else {
            break;
        };
        request.after_cycle_id = next;
        request.cursor = page.cursor;
    }
    walked
}

/* === Tests === */

#[tokio::test]
async fn cycle_search_once() {
    let root = tempfile::tempdir().unwrap();
    let prepared_owner = dokime_server::grpc::Service::prepare(Config::default()).unwrap();
    let mut owner = fixture(root.path(), Arc::new(Gate::default()), &prepared_owner).await;
    support::register_fixture_execution(&owner.handle(), support::fixture_suite(&["passed"])).await;
    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
    let mut client = WireClient::connect(server.local_endpoint()).await.unwrap();
    for (identifier, label, description) in [
        ("grpc.burn-c", "Burn In Gamma", None),
        ("grpc.calibration", "Calibration Sweep", Some("Thermal soak")),
        ("grpc.burn-a", "Burn In Alpha", Some("Thermal soak")),
        ("grpc.burn-b", "Burn In Beta", None),
    ] {
        client.unary::<contracts::execution::CreateCycle>(labelled(identifier, label, description)).await.unwrap();
    }
    let by_label = model::execution::wire::CycleOrder {
        key: model::execution::wire::CycleOrderKey::Label,
        descending: false,
    };

    // Text narrows on the server, and one cycle per page still yields the
    // whole match set once, in the declared order.
    let walked = walk(
        &mut client,
        model::execution::wire::CycleFilter {
            text: "BURN in".into(),
            ..Default::default()
        },
        by_label.clone(),
        1,
    )
    .await;
    assert_eq!(walked, ["Burn In Alpha", "Burn In Beta", "Burn In Gamma"]);

    // The same matches, in one page, are the same sequence.
    let single = walk(
        &mut client,
        model::execution::wire::CycleFilter {
            text: "burn in".into(),
            ..Default::default()
        },
        by_label.clone(),
        50,
    )
    .await;
    assert_eq!(single, walked);

    // Text also matches the description, and the identifier.
    let by_description = walk(
        &mut client,
        model::execution::wire::CycleFilter {
            text: "thermal SOAK".into(),
            ..Default::default()
        },
        by_label.clone(),
        1,
    )
    .await;
    assert_eq!(by_description, ["Burn In Alpha", "Calibration Sweep"]);
    let by_identifier = walk(
        &mut client,
        model::execution::wire::CycleFilter {
            text: "grpc.burn-b".into(),
            ..Default::default()
        },
        by_label.clone(),
        2,
    )
    .await;
    assert_eq!(by_identifier, ["Burn In Beta"]);

    // A status the runtime has not reached selects nothing; the one it has
    // selects everything.
    assert!(
        walk(
            &mut client,
            model::execution::wire::CycleFilter {
                statuses: vec![model::execution::wire::CycleStatus::Completed],
                ..Default::default()
            },
            by_label.clone(),
            10,
        )
        .await
        .is_empty()
    );
    assert_eq!(
        walk(
            &mut client,
            model::execution::wire::CycleFilter {
                statuses: vec![model::execution::wire::CycleStatus::Ready],
                ..Default::default()
            },
            by_label.clone(),
            10,
        )
        .await
        .len(),
        4
    );

    // A continuation the filter excludes is rejected, not silently restarted.
    let excluded =
        client.unary::<contracts::execution::ListCycles>(contracts::execution::ListCycles::default()).await.unwrap().into_inner();
    let outsider =
        excluded.cycles.iter().find(|cycle| cycle.metadata.as_ref().unwrap().label == "Calibration Sweep").unwrap().cycle_id.clone();
    assert_eq!(
        client
            .unary::<contracts::execution::ListCycles>(contracts::execution::ListCycles {
                after_cycle_id: outsider,
                cursor: excluded.cursor.clone(),
                filter: Some(model::execution::wire::CycleFilter {
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
            .unary::<contracts::execution::ListCycles>(contracts::execution::ListCycles {
                filter: Some(model::execution::wire::CycleFilter {
                    statuses: vec![model::execution::wire::CycleStatus::Unspecified],
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

#[tokio::test]
async fn listed_cases_page() {
    let root = tempfile::tempdir().unwrap();
    let prepared_owner = dokime_server::grpc::Service::prepare(Config::default()).unwrap();
    let mut owner = fixture(root.path(), Arc::new(Gate::default()), &prepared_owner).await;
    support::register_fixture_execution(&owner.handle(), support::fixture_suite(&["passed"])).await;
    let server = prepared_owner.ready().await.unwrap().listen().await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let cycle_id = support::planned_cycle(&client, Request::new(create()), selection_plan(&owner)).await.unwrap();
    let page = client
        .unary::<contracts::execution::ListCases>(contracts::execution::ListCases { cycle_id, ..Default::default() })
        .await
        .unwrap()
        .into_inner();
    assert!(!page.cases.is_empty());
    for case in &page.cases {
        // Creation materializes one undispatched attempt per case, which is
        // queued work rather than a skip and is the attempt a run dispatches.
        assert_eq!(case.status, model::execution::wire::CaseStatus::Queued);
        assert_eq!(case.attempt_count, 1);
        assert_eq!(case.latest_execution_status, Some(model::execution::wire::ExecutionStatus::Skipped));
        assert!(case.latest_execution_id.is_some());
        let actions = case.actions.as_ref().unwrap();
        assert!(actions.delete);
        assert!(actions.unavailable_reasons.is_empty());
    }

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}
