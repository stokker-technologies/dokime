/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Plans Parameter Selections

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime::{client::HostClient, v1};
use dokime_runtime_next::PluginProcess;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[tokio::test]
async fn client_plans_parameter_selections() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Inspect the Library and Preview Repeated Selections Without Starting
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let settings = super::support::settings();
        let mut plugin = PluginProcess::start(&super::support::thermal_fixture()?, true, &settings.placement).await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = HostClient::new(channel.clone(), super::support::client_settings())?;
        client.plugins.add(plugin.endpoint()).await?;
        let mut cases = dokime_protocol::client::cases_client::CasesClient::new(channel.clone());
        let selection = dokime_protocol::client::RegisteredSelection {
            suite_id: "thermal".into(),
            template_id: "thermal.cycle".into(),
            parameters: vec![dokime_protocol::client::RegisteredSelectionParametersEntry {
                key: "limit".into(),
                value: Some(dokime_protocol::client::SelectionValues {
                    values: vec![
                        v1::Value {
                            kind: Some(v1::value::Kind::Number(80.0)),
                        },
                        v1::Value {
                            kind: Some(v1::value::Kind::Number(85.0)),
                        },
                    ],
                }),
            }],
            ..Default::default()
        };
        let plan = dokime_protocol::client::SelectionPlan {
            selections: vec![selection.clone(), selection.clone()],
            max_cases: 4,
            ..Default::default()
        };
        let preview = cases
            .create(dokime_protocol::client::CasesCreateRequest {
                plan: Some(plan.clone()),
                validate_only: true,
                request_id: String::new(),
            })
            .await?
            .into_inner();
        assert_eq!(preview.cases.len(), 4);
        assert_eq!(preview.cases.iter().map(|case| case.selection_index).collect::<Vec<_>>(), vec![0, 0, 1, 1]);
        assert!(client.executions.list().await?.executions.is_empty());
        let request = dokime_protocol::client::CasesCreateRequest {
            plan: Some(plan.clone()),
            validate_only: false,
            request_id: "selection".into(),
        };
        let registered = cases.create(request.clone()).await?.into_inner();
        assert!(registered.cases.iter().all(|case| !case.case_id.is_empty()));
        assert_eq!(cases.create(request.clone()).await?.into_inner(), registered);
        let mut conflict = request.clone();
        conflict.plan.as_mut().unwrap().selections.pop();
        assert!(cases.create(conflict).await.is_err());
        let selected = cases
            .create(dokime_protocol::client::CasesCreateRequest {
                plan: Some(dokime_protocol::client::SelectionPlan {
                    case_ids: vec![
                        registered.cases[2].case_id.clone(),
                        registered.cases[0].case_id.clone(),
                        registered.cases[2].case_id.clone(),
                    ],
                    ..Default::default()
                }),
                validate_only: true,
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(
            selected.cases.iter().map(|case| &case.origin).collect::<Vec<_>>(),
            vec![&registered.cases[2].origin, &registered.cases[0].origin, &registered.cases[2].origin]
        );
        assert!(
            cases
                .create(dokime_protocol::client::CasesCreateRequest {
                    plan: Some(dokime_protocol::client::SelectionPlan { max_cases: 1, ..plan.clone() }),
                    validate_only: true,
                    ..Default::default()
                })
                .await
                .is_err()
        );
        let mut invalid = selection.clone();
        invalid.parameters[0].key = "unknown".into();
        assert!(
            cases
                .create(dokime_protocol::client::CasesCreateRequest {
                    plan: Some(dokime_protocol::client::SelectionPlan {
                        selections: vec![invalid],
                        ..Default::default()
                    }),
                    validate_only: true,
                    ..Default::default()
                })
                .await
                .is_err()
        );
        let materialized = cases
            .create(dokime_protocol::client::CasesCreateRequest {
                plan: Some(dokime_protocol::client::SelectionPlan {
                    selections: vec![selection],
                    ..Default::default()
                }),
                validate_only: false,
                request_id: "materialize".into(),
            })
            .await?
            .into_inner();
        assert_eq!(materialized.cases.len(), 2);
        process.close().await?;

        // The Original Registration Resolves Without a Connected Plugin After Restart.
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let mut cases = dokime_protocol::client::cases_client::CasesClient::connect(process.url.clone()).await?;
        assert_eq!(cases.create(request).await?.into_inner(), registered);
        process.close().await?;
        plugin.close().await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    })
    .await?
}
