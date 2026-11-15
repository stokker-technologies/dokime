/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Observes Runtime State

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime::Client;
use dokime::EvidenceScope;
use dokime::MapValue;
use dokime::client::BackendOverviewGetOptions;
use dokime::client::BackendOverviewWatchOptions;
use dokime::client::ExecutionsProgressWatchOptions;
use dokime::client::PluginPlacement;
use dokime::client::PluginRemote;
use dokime::client::PluginValue;
use dokime_runtime::PluginProcess;
use harmos_protocol::host::plugin_placement::Kind as PluginPlacementKind;
use std::collections::HashMap;
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
fn client_observes_runtime_state() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Observe the Initial State Before Adding a Plugin
            let root = super::support::tempdir()?;
            let credential_root = super::support::tempdir()?;
            let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
            let settings = super::support::settings();
            let mut plugin = PluginProcess::start(&super::support::thermal_fixture()?, true, &settings.placement).await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel.clone(), super::support::client_settings())?;
            let observation = Client::from_channel(channel.clone(), super::support::client_settings())?.backend.overview.clone();
            let initial = observation.get().await?;
            assert!(initial.overview.as_ref().unwrap().cycle_summaries.is_empty());
            let mut watch = observation
                .watch_with(BackendOverviewWatchOptions {
                    at: initial.resolved_at.clone(),
                    ..Default::default()
                })
                .await?;
            let opening = watch.next().await?.unwrap();
            assert!(opening.opening);
            assert_eq!(opening.overview, initial.overview);
            let mut invalid = initial.resolved_at.unwrap();
            invalid.revisions[0].revision.push(0);
            assert!(observation.get_with(BackendOverviewGetOptions { at: Some(invalid), ..Default::default() }).await.is_err());
            let entry = client
                .plugins
                .create(
                    "thermal",
                    PluginPlacement {
                        kind: Some(PluginPlacementKind::Remote(PluginRemote { url: plugin.endpoint().into() })),
                    },
                    PluginValue { enabled: true },
                )
                .await?
                .entry
                .unwrap();
            super::support::plugin_state(&client.plugins, entry, harmos_protocol::host::PluginState::Ready).await?;
            loop {
                let changed = watch.next().await?.ok_or("Observe Ready Plugin: Overview Watch Closed Before Admission")?;
                assert!(!changed.opening);
                if let Some(failure) = changed.resync_required {
                    return Err(format!("Observe Ready Plugin: {failure:?}").into());
                }
                let count = changed.overview.ok_or("Observe Ready Plugin: Overview Watch Returned No Snapshot")?.test_suite_count;
                if count == 0 {
                    continue;
                }
                assert_eq!(count, 1);
                break;
            }
            let cases = super::support::materialize(&client, "thermal", "thermal.cycle", HashMap::new()).await?;
            let cycle = super::support::admit(&client, cases).await?;
            let identifier = cycle[0].clone();
            let mut progress = Client::from_channel(channel.clone(), super::support::client_settings())?
                .executions
                .progress
                .clone()
                .watch_with(identifier.clone(), ExecutionsProgressWatchOptions { at: None, ..Default::default() })
                .await?;
            while let Some(frame) = progress.next().await? {
                if frame.phase == "prompt" {
                    break;
                }
            }
            let shell = observation.get().await?.overview.unwrap();
            assert_eq!(shell.test_suite_count, 1);
            assert_eq!(shell.cycle_summaries[0].cycle_id, client.executions.get(identifier.clone()).await?.execution.unwrap().cycle_id);
            assert_eq!(shell.evidence_counts.unwrap().prompts, 1);
            let traceability = Client::from_channel(channel, super::support::client_settings())?
                .evidence
                .traceability
                .clone()
                .get(EvidenceScope {
                    execution_id: Some(identifier.clone()),
                    ..Default::default()
                })
                .await?;
            assert!(traceability.snapshots.is_empty());
            let mut cancelled = observation.watch().await?;
            cancelled.next().await?.unwrap();
            drop(cancelled);
            assert!(client.executions.get(identifier.clone()).await?.execution.unwrap().controls.unwrap().can_cancel);
            let prompt = client.prompts.list(identifier.clone(), true).await?.prompts[0].prompt.as_ref().unwrap().prompt_id.clone();
            client.prompts.answers.create(prompt, "yes", MapValue::default(), true).await?;
            let mut terminal = false;
            while let Some(frame) = progress.next().await? {
                terminal |= frame.finished;
            }
            assert!(terminal);
            process.close().await?;
            let mut stopped = false;
            while let Some(frame) = watch.next().await? {
                stopped |= frame.overview.as_ref().is_some_and(|overview| overview.shutting_down);
            }
            assert!(stopped);
            plugin.close().await?;
            Ok::<_, Box<dyn Error>>(())
        })
        .await?
    }))
}
