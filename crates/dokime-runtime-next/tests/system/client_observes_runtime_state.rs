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
async fn client_observes_runtime_state() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Observe the Initial State Before Adding a Plugin
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let settings = super::support::settings();
        let mut plugin = PluginProcess::start(&super::support::thermal_fixture()?, true, &settings.placement).await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = HostClient::new(channel.clone(), super::support::client_settings())?;
        let mut observation = v1::backend::overview_client::OverviewClient::new(channel.clone());
        let initial = observation.get(v1::backend::OverviewGetRequest::default()).await?.into_inner();
        assert!(initial.overview.as_ref().unwrap().cycle_summaries.is_empty());
        let mut watch = observation.watch(v1::backend::OverviewWatchRequest { at: initial.resolved_at.clone() }).await?.into_inner();
        let opening = watch.message().await?.unwrap();
        assert!(opening.opening);
        assert_eq!(opening.overview, initial.overview);
        let mut invalid = initial.resolved_at.unwrap();
        invalid.revisions[0].revision.push(0);
        assert!(observation.get(v1::backend::OverviewGetRequest { at: Some(invalid) }).await.is_err());
        client.plugins.add(plugin.endpoint()).await?;
        let changed = watch.message().await?.unwrap();
        assert!(!changed.opening);
        assert_eq!(changed.overview.unwrap().test_suite_count, 1);
        let cases = client.templates.materialize("thermal", "thermal.cycle", std::collections::HashMap::new()).await?;
        let cycle = client.cycles.start(cases).await?;
        let identifier = cycle.executions[0].clone();
        let mut progress = v1::executions::progress_client::ProgressClient::new(channel.clone())
            .watch(v1::executions::ProgressWatchRequest { name: identifier.clone(), at: None })
            .await?
            .into_inner();
        while let Some(frame) = progress.message().await? {
            if frame.phase == "prompt" {
                break;
            }
        }
        let shell = observation.get(v1::backend::OverviewGetRequest::default()).await?.into_inner().overview.unwrap();
        assert_eq!(shell.test_suite_count, 1);
        assert_eq!(shell.cycle_summaries[0].cycle_id, cycle.identifier);
        assert_eq!(shell.evidence_counts.unwrap().prompts, 1);
        let traceability = v1::evidence::traceability_client::TraceabilityClient::new(channel)
            .get(v1::evidence::TraceabilityGetRequest {
                scope: Some(v1::EvidenceScope {
                    execution_id: Some(identifier.clone()),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert!(traceability.snapshots.is_empty());
        let mut cancelled = observation.watch(v1::backend::OverviewWatchRequest::default()).await?.into_inner();
        cancelled.message().await?.unwrap();
        drop(cancelled);
        assert!(client.executions.get(identifier.clone()).await?.execution.unwrap().controls.unwrap().can_cancel);
        let prompt = client.prompts.pending(identifier.clone()).await?.prompts[0].prompt.as_ref().unwrap().prompt_id.clone();
        client.prompts.respond(prompt, "yes", true).await?;
        let mut terminal = false;
        while let Some(frame) = progress.message().await? {
            terminal |= frame.finished;
        }
        assert!(terminal);
        process.close().await?;
        let mut stopped = false;
        while let Some(frame) = watch.message().await? {
            stopped |= frame.overview.as_ref().is_some_and(|overview| overview.shutting_down);
        }
        assert!(stopped);
        plugin.close().await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    })
    .await?
}
