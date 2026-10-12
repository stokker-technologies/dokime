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
use tonic::{Code, transport::Channel};
// Local Imports
use super::*;
use dokime_protocol::v1::{self, backend_client::BackendClient, execution_client::ExecutionClient};

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
fn selection(suite: &dokime_app::definition::TestSuite, index: usize) -> v1::RegisteredSelection {
    v1::RegisteredSelection {
        suite_id: suite.metadata.identifier.to_string(),
        template_id: suite.test_case_templates[index]
            .test_case_template_id
            .to_string(),
        ..Default::default()
    }
}

fn plan(selection: v1::RegisteredSelection) -> v1::SelectionPlan {
    v1::SelectionPlan {
        selections: vec![selection],
        ..Default::default()
    }
}

async fn client(server: &Server) -> ExecutionClient<Channel> {
    ExecutionClient::connect(server.local_endpoint())
        .await
        .unwrap()
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
    let mut execution = client(&server).await;

    let cycle_id = execution
        .create_cycle(auth(
            v1::CreateCycleRequest {
                plan: Some(plan(selection(&suite, 0))),
                metadata: Some(v1::Metadata {
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
        .list_cases(auth(
            v1::ListCasesRequest {
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

    let request = v1::AppendPlannedCasesRequest {
        cycle_id: cycle_id.clone(),
        plan: Some(plan(selection(&suite, 1))),
        context: None,
    };
    assert_eq!(
        execution
            .append_planned_cases(auth(request.clone(), super::TOKEN))
            .await
            .unwrap_err()
            .code(),
        Code::PermissionDenied
    );
    let reply = execution
        .append_planned_cases(auth(request, CONTROL_TOKEN))
        .await
        .unwrap()
        .into_inner();
    assert!(reply.observed_cursor.is_some());

    let after = execution
        .list_cases(auth(
            v1::ListCasesRequest {
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

    let mut backend = BackendClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let read_methods = backend
        .get_capabilities(auth(v1::GetCapabilitiesRequest::default(), super::TOKEN))
        .await
        .unwrap()
        .into_inner()
        .methods;
    assert!(!read_methods.contains(&"/dokime.v1.Execution/AppendPlannedCases".into()));
    let control_methods = backend
        .get_capabilities(auth(v1::GetCapabilitiesRequest::default(), CONTROL_TOKEN))
        .await
        .unwrap()
        .into_inner()
        .methods;
    assert!(control_methods.contains(&"/dokime.v1.Execution/AppendPlannedCases".into()));

    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Tests === */
