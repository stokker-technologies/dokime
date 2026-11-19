/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Interrupted Execution Story

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime::EvidenceScope;
use dokime::Termination;
use dokime_client_internal::Client;
use dokime_client_internal::EvidenceSamplesListOptions;
use dokime_client_internal::ExecutionState;
use dokime_client_internal::ExecutionsProgressWatchOptions;
use dokime_client_internal::PluginPlacement;
use dokime_client_internal::PluginRemote;
use dokime_client_internal::PluginState;
use dokime_client_internal::PluginValue;
use dokime_protocol::host::evidence::SamplesListRequest;
use dokime_protocol::host::execution_outcome;
use dokime_runtime::PluginProcess;
use harmos_protocol::host::plugin_placement;
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
fn client_reads_interrupted_execution() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Start an Execution in the Real Host and Wait for Its Prompt
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let settings = super::support::settings();
        let mut plugin = PluginProcess::start(&super::support::execution_fixture()?, true, &settings.placement).await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
        let plugins = Client::from_channel(channel.clone(), super::support::client_settings())?.plugins.clone();
        let entry = plugins
            .create(
                "execution",
                PluginPlacement {
                    kind: Some(plugin_placement::Kind::Remote(PluginRemote { url: plugin.endpoint().into() })),
                },
                PluginValue { enabled: true },
            )
            .await?
            .entry
            .unwrap();
        let entry = super::support::plugin_state(&plugins, entry, harmos_protocol::host::PluginState::Ready).await?;
        let client = Client::from_channel(channel.clone(), super::support::client_settings())?;
        let parameters = super::support::parameters(19.0, 0, true).into_iter().map(|(key, value)| (key, vec![value])).collect();
        let planned = super::support::materialize(&client, "execution", "execution.capture", parameters).await?;
        let cycle = super::support::admit(&client, planned).await?;
        let identifier = cycle[0].clone();
        let mut progress =
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .executions
                .progress
                .clone()
                .watch_with(identifier.clone(), ExecutionsProgressWatchOptions { at: None, ..Default::default() })
                .await?;
        let mut prompted = false;
        while let Some(frame) = progress.next().await? {
            if frame.phase == "prompt" {
                prompted = true;
                break;
            }
        }
        assert!(prompted);
        let before = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .executions
            .clone()
            .get(identifier.clone())
            .await?
            .execution
            .unwrap();
        assert!(before.controls.unwrap().can_cancel);
        let request = SamplesListRequest {
            scope: Some(EvidenceScope {
                execution_id: Some(identifier.clone()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let samples = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .evidence
            .samples
            .clone()
            .list_with(((request.clone()).scope).unwrap_or_default(), (request.clone()).streams, EvidenceSamplesListOptions { versions: Some((request.clone()).versions), fields: Some((request.clone()).fields), sources: Some((request.clone()).sources), signals: Some((request.clone()).signals), between: Some(((request.clone()).between).unwrap_or_default()), bucket: Some(((request.clone()).bucket).unwrap_or_default()), latest: Some((request.clone()).latest),
                    at: (request.clone()).at,
                    page: (request.clone()).page,
                    ..Default::default()
                })
            .await?;
        assert!(!samples.samples.is_empty());
        drop(progress);

        // Kill the Host and Inspect the Interrupted Evidence Through the Same Reads
        process.kill().await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;

        let plugins = Client::from_channel(channel.clone(), super::support::client_settings())?.plugins.clone();
        let recovered = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .executions
            .clone()
            .get(identifier.clone())
            .await?
            .execution
            .unwrap();
        assert_eq!(recovered.summary.as_ref().unwrap().state, ExecutionState::Sealed as i32);
        assert!(matches!(recovered.outcome.as_ref().unwrap().attribution.as_ref(),
                        Some(execution_outcome::Attribution::NotEvaluated(outcome)) if outcome.termination == Termination::Interrupted as i32));
        let controls = recovered.controls.as_ref().unwrap();
        assert!(controls.can_rerun && controls.can_inspect && controls.can_export);
        assert!(!controls.can_start && !controls.can_cancel);
        assert_eq!(
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .evidence
                .samples
                .clone()
                .list_with(((request.clone()).scope).unwrap_or_default(), (request.clone()).streams, EvidenceSamplesListOptions { versions: Some((request.clone()).versions), fields: Some((request.clone()).fields), sources: Some((request.clone()).sources), signals: Some((request.clone()).signals), between: Some(((request.clone()).between).unwrap_or_default()), bucket: Some(((request.clone()).bucket).unwrap_or_default()), latest: Some((request.clone()).latest),
                        at: (request.clone()).at,
                        page: (request.clone()).page,
                        ..Default::default()
                    })
                .await?
                .samples,
            samples.samples
        );
        let page = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .executions
            .clone()
            .list(String::new(), Default::default(), Default::default(), Default::default())
            .await?;
        assert_eq!(page.executions.iter().filter_map(|row| row.summary.clone()).collect::<Vec<_>>(), vec![recovered.summary.clone().unwrap()]);
        let counts = page.outcome_counts.unwrap();
        assert_eq!((counts.evaluated_count, counts.not_evaluated_count, counts.declared_count), (0, 1, 0));
        assert_eq!(counts.result_status, None);
        assert_eq!(recovered.cycle_counts.as_ref(), Some(&counts));
        let saved = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .plugins
            .clone()
            .list()
            .await?;
        assert_eq!(saved.entries.len(), 1);
        assert_eq!(saved.entries[0].name, entry.name);
        super::support::plugin_state(&plugins, saved.entries[0].clone(), PluginState::Ready).await?;
        assert_eq!(saved.entries[0].description, entry.description);
        let mut progress =
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .executions
                .progress
                .clone()
                .watch_with(identifier.clone(), ExecutionsProgressWatchOptions { at: None, ..Default::default() })
                .await?;
        let mut terminal = false;
        while let Some(frame) = progress.next().await? {
            if frame.sealed {
                terminal = true;
            }
        }
        assert!(terminal);

        // Reattach the Plugin Without Replaying or Changing Recovered Evidence
        let restarted = plugins.restart("execution", plugins.get("execution").await?.entry.unwrap().revision).await?;
        super::support::plugin_state(&plugins, restarted, PluginState::Ready).await?;
        assert_eq!(
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .plugins
                .clone()
                .list()
                .await?
                .entries[0]
                .state,
            PluginState::Ready as i32
        );
        assert_eq!(
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .executions
                .clone()
                .get(identifier)
                .await?
                .execution
                .unwrap(),
            recovered
        );
        process.close().await?;
        plugin.close().await?;
        Ok::<(), Box<dyn Error>>(())
    })
    .await?
    }))
}
