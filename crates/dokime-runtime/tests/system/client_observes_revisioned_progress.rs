/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Plugin Progress Has Revisioned Values and Retained Client Reads

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::HashMap;
use std::error::Error;

// Third Party Imports
use harmos_protocol::host::plugin_placement;
use harmos_protocol::model::ReadCut;
use tonic::transport::Endpoint;

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use super::support;
use dokime::Client;
use dokime::MapValue;
use dokime::client::CyclesCreateOptions;
use dokime::client::ExecutionsGetOptions;
use dokime::client::ExecutionsListOptions;
use dokime::client::ExecutionsProgressGetOptions;
use dokime::client::ExecutionsProgressWatchOptions;
use dokime::client::PluginPlacement;
use dokime::client::PluginRemote;
use dokime::client::PluginValue;
use dokime_protocol::host::executions::ProgressGetRequest;
use dokime_runtime::PluginProcess;

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
fn client_observes_revisioned_progress() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(support::SYSTEM_HANG_GUARD, async {
            // Start the Product and Admit the Plugin Through Its Client API
            let root = support::tempdir()?;
            let credential_root = support::tempdir()?;
            let credentials = support::CredentialStore::start(credential_root.path()).await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let settings = support::settings();
            let mut plugin = PluginProcess::start(&support::content_fixture()?, true, &settings.placement).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let host = Client::from_channel(channel.clone(), support::client_settings())?;
            let client = Client::from_channel(channel.clone(), support::client_settings())?;
            let entry = host
                .plugins
                .create(
                    "content",
                    PluginPlacement {
                        kind: Some(plugin_placement::Kind::Remote(PluginRemote { url: plugin.endpoint().into() })),
                    },
                    PluginValue { enabled: true },
                )
                .await?
                .entry
                .unwrap();
            super::support::plugin_state(&host.plugins, entry, harmos_protocol::host::PluginState::Ready).await?;
            let cases = super::support::materialize(&host, "content", "content.progress", HashMap::new()).await?;
            let cycle = Client::from_channel(channel.clone(), super::support::client_settings())?
                .cycles
                .clone()
                .create_with(
                    cases.into_iter().map(|case| case.origin.unwrap()).collect(),
                    dokime::Metadata {
                        label: "Story Cycle".into(),
                        ..Default::default()
                    },
                    true,
                    String::new(),
                    CyclesCreateOptions {
                        request_id: Some(uuid::Uuid::now_v7().to_string()),
                        ..Default::default()
                    },
                )
                .await?;
            let admitted = Client::from_channel(channel.clone(), super::support::client_settings())?
                .executions
                .clone()
                .list_with(
                    cycle.cycle.unwrap().cycle_id,
                    Default::default(),
                    Default::default(),
                    Default::default(),
                    ExecutionsListOptions {
                        at: Some(ReadCut {
                            positions: vec![cycle.receipt.unwrap().durable_at.unwrap()],
                            revisions: Vec::new(),
                        }),
                        ..Default::default()
                    },
                )
                .await?;
            let name = admitted.executions[0].summary.as_ref().unwrap().execution_id.clone();

            // Open the Descriptor-Derived Hierarchy and Retain a Durable Read
            let mut watch = client
                .executions
                .progress
                .watch_with(name.clone(), ExecutionsProgressWatchOptions { at: None, ..Default::default() })
                .await?;
            loop {
                let response = watch.next().await?.ok_or("Watch Progress: Missing Prompt Event")?;
                if response.phase == "prompt" {
                    break;
                }
            }
            let first = client
                .executions
                .progress
                .get_with(ExecutionsProgressGetOptions {
                    name: Some(name.clone()),
                    at: None,
                    ..Default::default()
                })
                .await?;
            assert_eq!(first.value.as_ref().unwrap().completed, 1);
            let prompt = host.prompts.list(name.clone(), true).await?.prompts[0].prompt.as_ref().unwrap().prompt_id.clone();
            host.prompts.answers.create(prompt, "continue", MapValue::default(), true).await?;
            let mut finished = false;
            while let Some(response) = watch.next().await? {
                finished |= response.finished;
            }
            assert!(finished);

            // Whole-Value Replacement Clears Fields and Execution Detail Agrees
            let current = client
                .executions
                .progress
                .get_with(ExecutionsProgressGetOptions {
                    name: Some(name.clone()),
                    at: None,
                    ..Default::default()
                })
                .await?;
            assert_eq!(current.value.as_ref().unwrap().completed, 2);
            assert!(current.value.as_ref().unwrap().label.is_empty());
            assert_ne!(current.revision, first.revision);
            let detail = Client::from_channel(channel.clone(), super::support::client_settings())?
                .executions
                .clone()
                .get_with(name.clone(), ExecutionsGetOptions { at: None, ..Default::default() })
                .await?
                .execution
                .unwrap();
            assert_eq!(detail.progress, current.value);
            assert_eq!(detail.resolved_at, current.resolved_at);
            let pinned = ProgressGetRequest {
                name: Some(name.clone()),
                at: first.resolved_at.clone(),
            };
            assert_eq!(
                client
                    .executions
                    .progress
                    .get_with(ExecutionsProgressGetOptions {
                        name: Some(((pinned.clone()).name).unwrap_or_default()),
                        at: (pinned.clone()).at,
                        ..Default::default()
                    })
                    .await?,
                first
            );
            let mut unavailable = pinned.clone();
            unavailable.at.as_mut().unwrap().history.as_mut().unwrap().epoch = "another-execution".into();
            assert!(
                client
                    .executions
                    .progress
                    .get_with(ExecutionsProgressGetOptions {
                        name: Some(((unavailable.clone()).name).unwrap_or_default()),
                        at: (unavailable.clone()).at,
                        ..Default::default()
                    })
                    .await
                    .is_err()
            );
            process.close().await?;

            // Reopen Without Losing a Returned Position or Its Value
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, support::client_settings())?;
            assert_eq!(
                client
                    .executions
                    .progress
                    .get_with(ExecutionsProgressGetOptions {
                        name: Some(((pinned.clone()).name).unwrap_or_default()),
                        at: (pinned.clone()).at,
                        ..Default::default()
                    })
                    .await?,
                first
            );
            process.close().await?;
            plugin.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}
