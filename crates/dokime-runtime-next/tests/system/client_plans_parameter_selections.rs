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
        let root = tempfile::tempdir()?;
        let credential_root = tempfile::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let settings = super::support::settings();
        let mut plugin = PluginProcess::start(&super::support::thermal_fixture()?, true, &settings.placement).await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = HostClient::new(channel.clone(), super::support::client_settings())?;
        client.plugins.add(plugin.endpoint()).await?;
        let mut planning = v1::planning_client::PlanningClient::new(channel.clone());
        let selection = v1::RegisteredSelection {
            suite_id: "thermal".into(),
            template_id: "thermal.cycle".into(),
            parameters: vec![v1::RegisteredSelectionParametersEntry {
                key: "limit".into(),
                value: Some(v1::SelectionValues {
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
        let plan = v1::SelectionPlan {
            selections: vec![selection.clone(), selection.clone()],
            max_cases: 4,
            ..Default::default()
        };
        let preview = planning.preview_selection(v1::PlanningPreviewSelectionRequest { plan: Some(plan.clone()) }).await?.into_inner();
        assert_eq!(preview.cases.len(), 4);
        assert_eq!(preview.cases.iter().map(|case| case.selection_index).collect::<Vec<_>>(), vec![0, 0, 1, 1]);
        assert!(client.executions.list().await?.rows.is_empty());

        // Register the Entire Plan Atomically and Resolve a Lost Reply
        let request = v1::PlanningRegisterSelectionRequest {
            plan: Some(plan.clone()),
            idempotency_key: "operator-selection".into(),
        };
        let registered = planning.register_selection(request.clone()).await?.into_inner();
        assert!(registered.cases.iter().all(|case| !case.case_id.is_empty()));
        assert_eq!(planning.register_selection(request).await?.into_inner(), registered);
        let changed = planning
            .register_selection(v1::PlanningRegisterSelectionRequest {
                plan: Some(v1::SelectionPlan {
                    selections: vec![selection.clone()],
                    ..Default::default()
                }),
                idempotency_key: "operator-selection".into(),
            })
            .await
            .unwrap_err();
        assert_eq!(changed.code(), tonic::Code::FailedPrecondition);
        let selected = planning
            .preview_selection(v1::PlanningPreviewSelectionRequest {
                plan: Some(v1::SelectionPlan {
                    case_ids: vec![
                        registered.cases[2].case_id.clone(),
                        registered.cases[0].case_id.clone(),
                        registered.cases[2].case_id.clone(),
                    ],
                    ..Default::default()
                }),
            })
            .await?
            .into_inner();
        assert_eq!(
            selected.cases.iter().map(|case| &case.origin).collect::<Vec<_>>(),
            vec![&registered.cases[2].origin, &registered.cases[0].origin, &registered.cases[2].origin]
        );

        // Reject Invalid Expansion Before Registering Any Cases
        assert!(
            planning
                .preview_selection(v1::PlanningPreviewSelectionRequest {
                    plan: Some(v1::SelectionPlan { max_cases: 1, ..plan })
                })
                .await
                .is_err()
        );
        let mut invalid = selection.clone();
        invalid.parameters[0].key = "unknown".into();
        assert!(planning.materialize_template(v1::PlanningMaterializeTemplateRequest { selection: Some(invalid) }).await.is_err());

        // Materialize Through the Bidirectional Call and Await Final Status
        let mut run = planning
            .run_materialize_template(tokio_stream::iter([
                v1::PlanningMaterializeTemplateRunInput {
                    value: Some(v1::planning_materialize_template_run_input::Value::Request(v1::PlanningMaterializeTemplateRequest {
                        selection: Some(selection),
                    })),
                },
                v1::PlanningMaterializeTemplateRunInput {
                    value: Some(v1::planning_materialize_template_run_input::Value::Finish(v1::ClientEmpty {})),
                },
            ]))
            .await?
            .into_inner();
        let Some(v1::planning_materialize_template_run_output::Value::Response(materialized)) = run.message().await?.unwrap().value else {
            panic!("Missing Materialization Response");
        };
        assert_eq!(materialized.cases.len(), 2);
        assert!(run.message().await?.is_none());
        process.close().await?;

        // Resolve the Same Registration After Restart Without the Plugin
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let recovered = v1::planning_client::PlanningClient::new(channel)
            .register_selection(v1::PlanningRegisterSelectionRequest {
                plan: Some(v1::SelectionPlan {
                    selections: vec![
                        v1::RegisteredSelection {
                            suite_id: "thermal".into(),
                            template_id: "thermal.cycle".into(),
                            parameters: vec![v1::RegisteredSelectionParametersEntry {
                                key: "limit".into(),
                                value: Some(v1::SelectionValues {
                                    values: vec![
                                        v1::Value {
                                            kind: Some(v1::value::Kind::Number(80.0))
                                        },
                                        v1::Value {
                                            kind: Some(v1::value::Kind::Number(85.0))
                                        }
                                    ]
                                })
                            }],
                            ..Default::default()
                        };
                        2
                    ],
                    max_cases: 4,
                    ..Default::default()
                }),
                idempotency_key: "operator-selection".into(),
            })
            .await?
            .into_inner();
        assert_eq!(recovered, registered);
        process.close().await?;
        plugin.close().await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    })
    .await?
}
