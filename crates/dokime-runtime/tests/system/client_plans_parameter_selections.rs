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
use dokime::Value;
use dokime_client_internal::CasesCreateOptions;
use dokime_client_internal::Client;
use dokime_client_internal::RegisteredSelection;
use dokime_client_internal::RegisteredSelectionParametersEntry;
use dokime_client_internal::SelectionPlan;
use dokime_client_internal::SelectionValues;
use dokime_client_internal::host::PluginPlacement;
use dokime_client_internal::host::PluginRemote;
use dokime_client_internal::host::PluginValue;
use dokime_protocol::host::CasesCreateRequest;
use dokime_protocol::host::plugin_placement;
use dokime_protocol::model::value;
use dokime_runtime::PluginProcess;
use std::error::Error;
use tonic::transport::Endpoint;

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

#[test]
fn client_plans_parameter_selections() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Inspect the Library and Preview Repeated Selections Without Starting
            let root = super::support::tempdir()?;
            let credential_root = super::support::tempdir()?;
            let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
            let settings = super::support::settings();
            let mut plugin = PluginProcess::start(&super::support::thermal_fixture()?, true, &settings.placement).await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel.clone(), super::support::client_settings())?;
            let entry = client
                .plugins
                .create(
                    "thermal",
                    PluginPlacement {
                        kind: Some(plugin_placement::Kind::Remote(PluginRemote { url: plugin.endpoint().into() })),
                    },
                    PluginValue { enabled: true },
                )
                .await?
                .entry
                .unwrap();
            super::support::plugin_state(&client.plugins, entry, dokime_protocol::host::PluginState::Ready).await?;
            let cases = Client::from_channel(channel.clone(), super::support::client_settings())?.cases.clone();
            let selection = RegisteredSelection {
                suite_id: "thermal".into(),
                template_id: "thermal.cycle".into(),
                parameters: vec![RegisteredSelectionParametersEntry {
                    key: "limit".into(),
                    value: Some(SelectionValues {
                        values: vec![Value { kind: Some(value::Kind::Number(80.0)) }, Value { kind: Some(value::Kind::Number(85.0)) }],
                    }),
                }],
                ..Default::default()
            };
            let plan = SelectionPlan {
                selections: vec![selection.clone(), selection.clone()],
                max_cases: 4,
                ..Default::default()
            };
            let preview = cases
                .create_with(
                    plan.clone(),
                    true,
                    CasesCreateOptions {
                        request_id: Some(String::new()),
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(preview.cases.len(), 4);
            assert_eq!(preview.cases.iter().map(|case| case.selection_index).collect::<Vec<_>>(), vec![0, 0, 1, 1]);
            assert!(
                client
                    .executions
                    .list(String::new(), Default::default(), Default::default(), Default::default())
                    .await?
                    .executions
                    .is_empty()
            );
            let request = CasesCreateRequest {
                plan: Some(plan.clone()),
                validate_only: false,
                request_id: "selection".into(),
            };
            let registered = cases
                .create_with(
                    ((request.clone()).plan).unwrap_or_default(),
                    (request.clone()).validate_only,
                    CasesCreateOptions {
                        request_id: Some((request.clone()).request_id),
                        ..Default::default()
                    },
                )
                .await?;
            assert!(registered.cases.iter().all(|case| !case.case_id.is_empty()));
            assert_eq!(
                cases
                    .create_with(
                        ((request.clone()).plan).unwrap_or_default(),
                        (request.clone()).validate_only,
                        CasesCreateOptions {
                            request_id: Some((request.clone()).request_id),
                            ..Default::default()
                        }
                    )
                    .await?,
                registered
            );
            let mut conflict = request.clone();
            conflict.plan.as_mut().unwrap().selections.pop();
            assert!(
                cases
                    .create_with(
                        ((conflict.clone()).plan).unwrap_or_default(),
                        (conflict.clone()).validate_only,
                        CasesCreateOptions {
                            request_id: Some((conflict.clone()).request_id),
                            ..Default::default()
                        }
                    )
                    .await
                    .is_err()
            );
            let selected = cases
                .create(
                    SelectionPlan {
                        case_ids: vec![
                            registered.cases[2].case_id.clone(),
                            registered.cases[0].case_id.clone(),
                            registered.cases[2].case_id.clone(),
                        ],
                        ..Default::default()
                    },
                    true,
                )
                .await?;
            assert_eq!(
                selected.cases.iter().map(|case| &case.origin).collect::<Vec<_>>(),
                vec![&registered.cases[2].origin, &registered.cases[0].origin, &registered.cases[2].origin]
            );
            assert!(cases.create(SelectionPlan { max_cases: 1, ..plan.clone() }, true).await.is_err());
            let mut invalid = selection.clone();
            invalid.parameters[0].key = "unknown".into();
            assert!(
                cases
                    .create(
                        SelectionPlan {
                            selections: vec![invalid],
                            ..Default::default()
                        },
                        true
                    )
                    .await
                    .is_err()
            );
            let materialized = cases
                .create_with(
                    SelectionPlan {
                        selections: vec![selection],
                        ..Default::default()
                    },
                    false,
                    CasesCreateOptions {
                        request_id: Some("materialize".into()),
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(materialized.cases.len(), 2);
            process.close().await?;

            // The Original Registration Resolves Without a Connected Plugin After Restart.
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let cases =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .cases
                    .clone();
            assert_eq!(
                cases
                    .create_with(
                        ((request.clone()).plan).unwrap_or_default(),
                        (request.clone()).validate_only,
                        CasesCreateOptions {
                            request_id: Some((request.clone()).request_id),
                            ..Default::default()
                        }
                    )
                    .await?,
                registered
            );
            process.close().await?;
            plugin.close().await?;
            Ok::<_, Box<dyn Error>>(())
        })
        .await?
    }))
}
