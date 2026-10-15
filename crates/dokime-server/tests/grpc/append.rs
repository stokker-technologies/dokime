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
use tonic::Code;
// Local Imports
use super::*;

/* === Constants === */
const CONTROL_TOKEN: &str = "append-control-token-012345678901234567890";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn selection(
    suite: &dokime_app::definition::TestSuite,
    index: usize,
) -> contracts::planning::RegisteredSelection {
    contracts::planning::RegisteredSelection {
        suite_id: suite.metadata.identifier.to_string(),
        template_id: suite.test_case_templates[index]
            .test_case_template_id
            .to_string(),
        ..Default::default()
    }
}

fn plan(selection: contracts::planning::RegisteredSelection) -> contracts::planning::SelectionPlan {
    contracts::planning::SelectionPlan {
        selections: vec![selection],
        ..Default::default()
    }
}

async fn client(server: &Server) -> WireClient {
    WireClient::connect(server.local_endpoint()).await.unwrap()
}

pub(crate) async fn append_cases_control() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let suite = common::fixture_suite(&["passed", "passed"]);
    common::register_fixture_execution(&owner.handle(), suite.clone()).await;
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![
                grant(Role::Read),
                BearerGrant::new("append-control".into(), Role::Control, CONTROL_TOKEN).unwrap(),
            ])
            .unwrap(),
    )
    .await
    .unwrap();
    let execution = client(&server).await;

    let cycle_id = execution
        .unary::<contracts::execution::CreateCycle>(auth(
            contracts::execution::CreateCycleRequest {
                plan: Some(plan(selection(&suite, 0))),
                metadata: Some(contracts::common::Metadata {
                    identifier: "append.cycle".into(),
                    label: "Append cycle".into(),
                    description: None,
                    ..Default::default()
                }),
                ..Default::default()
            },
            CONTROL_TOKEN,
        ))
        .await
        .unwrap()
        .into_inner()
        .cycle_id;
    let before = execution
        .unary::<contracts::execution::ListCases>(auth(
            contracts::execution::ListCasesRequest {
                cycle_id: cycle_id.clone(),
                ..Default::default()
            },
            CONTROL_TOKEN,
        ))
        .await
        .unwrap()
        .into_inner()
        .cases;
    assert_eq!(before.len(), 1);

    let request = contracts::execution::AppendPlannedCasesRequest {
        cycle_id: cycle_id.clone(),
        plan: Some(plan(selection(&suite, 1))),
        context: None,
    };
    assert_eq!(
        execution
            .unary::<contracts::execution::AppendPlannedCases>(auth(request.clone(), super::TOKEN))
            .await
            .unwrap_err()
            .code(),
        Code::PermissionDenied
    );
    let reply = execution
        .unary::<contracts::execution::AppendPlannedCases>(auth(request, CONTROL_TOKEN))
        .await
        .unwrap()
        .into_inner();
    assert!(reply.observed_cursor.is_some());

    let after = execution
        .unary::<contracts::execution::ListCases>(auth(
            contracts::execution::ListCasesRequest {
                cycle_id: cycle_id.clone(),
                ..Default::default()
            },
            CONTROL_TOKEN,
        ))
        .await
        .unwrap()
        .into_inner()
        .cases;
    assert_eq!(after.len(), 2);
    assert_eq!(after[0].case_id, before[0].case_id);
    assert_eq!(after[0].template_id, before[0].template_id);
    assert_ne!(after[1].template_id, before[0].template_id);

    // Appending a template again continues its numbering within the cycle.
    execution
        .unary::<contracts::execution::AppendPlannedCases>(auth(
            contracts::execution::AppendPlannedCasesRequest {
                cycle_id: cycle_id.clone(),
                plan: Some(plan(selection(&suite, 0))),
                context: None,
            },
            CONTROL_TOKEN,
        ))
        .await
        .unwrap();
    let labels = execution
        .unary::<contracts::execution::ListCases>(auth(
            contracts::execution::ListCasesRequest {
                cycle_id: cycle_id.clone(),
                ..Default::default()
            },
            CONTROL_TOKEN,
        ))
        .await
        .unwrap()
        .into_inner()
        .cases
        .into_iter()
        .map(|case| case.metadata.unwrap().label)
        .collect::<Vec<_>>();
    let first = &suite.test_case_templates[0].metadata.label;
    let second = &suite.test_case_templates[1].metadata.label;
    assert_eq!(
        labels,
        [
            format!("{first} #1"),
            format!("{second} #1"),
            format!("{first} #2")
        ]
    );

    let backend = WireClient::connect(server.local_endpoint()).await.unwrap();
    let read_methods = backend
        .unary::<contracts::backend::GetAvailability>(auth(
            contracts::backend::GetAvailabilityRequest::default(),
            super::TOKEN,
        ))
        .await
        .unwrap()
        .into_inner()
        .methods;
    assert!(!read_methods.contains(&"/dokime.v1.Execution/AppendPlannedCases".into()));
    let control_methods = backend
        .unary::<contracts::backend::GetAvailability>(auth(
            contracts::backend::GetAvailabilityRequest::default(),
            CONTROL_TOKEN,
        ))
        .await
        .unwrap()
        .into_inner()
        .methods;
    assert!(control_methods.contains(&"/dokime.v1.Execution/AppendPlannedCases".into()));

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */
