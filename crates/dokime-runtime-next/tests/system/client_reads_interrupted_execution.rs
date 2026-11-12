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
use dokime_protocol::v1;
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
async fn client_reads_interrupted_execution() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Start an Execution in the Real Host and Wait for Its Prompt
        let root = tempfile::tempdir()?;
        let settings = super::support::settings();
        let mut plugin = PluginProcess::start(&super::support::execution_fixture()?, true, &settings.placement).await?;
        let process = HostProcess::start(root.path()).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let mut plugins = v1::plugins_client::PluginsClient::new(channel.clone());
        let mut planning = v1::planning_client::PlanningClient::new(channel.clone());
        let mut execution = v1::execution_client::ExecutionClient::new(channel.clone());
        let mut observation = v1::observation_client::ObservationClient::new(channel.clone());
        let mut telemetry = v1::telemetry_client::TelemetryClient::new(channel);
        let entry = plugins.add(v1::AddRemotePlugin { url: plugin.endpoint().into() }).await?.into_inner();
        let planned = planning
            .materialize_template(v1::PlanningMaterializeTemplateRequest {
                selection: Some(v1::RegisteredSelection {
                    suite_id: "execution".into(),
                    template_id: "execution.capture".into(),
                    parameters: super::support::parameters(19.0, 0, true)
                        .into_iter()
                        .map(|(key, value)| v1::RegisteredSelectionParametersEntry {
                            key,
                            value: Some(v1::SelectionValues { values: vec![value] }),
                        })
                        .collect(),
                    ..Default::default()
                }),
            })
            .await?
            .into_inner();
        let cycle = execution
            .create_cycle(v1::ExecutionCreateCycleRequest {
                origins: planned.cases.into_iter().map(|case| case.origin.unwrap()).collect(),
                continue_automatically: true,
                ..Default::default()
            })
            .await?
            .into_inner();
        let identifier = cycle.execution_ids[0].clone();
        let mut progress = observation
            .watch_execution_progress(v1::ObservationWatchExecutionProgressRequest { test_execution_id: identifier.clone() })
            .await?
            .into_inner();
        let mut prompted = false;
        while let Some(frame) = progress.message().await? {
            if let Some(v1::observation_watch_execution_progress_output::Value::Item(frame)) = frame.value {
                if frame.progress.unwrap().phase == "prompt" {
                    prompted = true;
                    break;
                }
            }
        }
        assert!(prompted);
        let before = execution.get_execution(v1::ExecutionGetExecutionRequest { execution_id: identifier.clone() }).await?.into_inner();
        assert!(before.controls.unwrap().can_cancel);
        let request = v1::TelemetryListSamplesRequest {
            scope: Some(v1::EvidenceScope {
                execution_id: Some(identifier.clone()),
                ..Default::default()
            }),
            ..Default::default()
        };
        let samples = telemetry.list_samples(request.clone()).await?.into_inner();
        assert!(!samples.samples.is_empty());
        drop(progress);

        // Kill the Host and Inspect the Interrupted Evidence Through the Same Reads
        process.kill().await?;
        let process = HostProcess::start(root.path()).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let mut execution = v1::execution_client::ExecutionClient::new(channel.clone());
        let mut query = v1::query_client::QueryClient::new(channel.clone());
        let mut telemetry = v1::telemetry_client::TelemetryClient::new(channel.clone());
        let mut plugins = v1::plugins_client::PluginsClient::new(channel.clone());
        let mut observation = v1::observation_client::ObservationClient::new(channel);
        let recovered = execution.get_execution(v1::ExecutionGetExecutionRequest { execution_id: identifier.clone() }).await?.into_inner();
        assert_eq!(recovered.summary.as_ref().unwrap().state, v1::ExecutionState::Sealed as i32);
        assert!(matches!(recovered.outcome.as_ref().unwrap().attribution.as_ref(),
            Some(v1::execution_outcome::Attribution::NotEvaluated(outcome)) if outcome.termination == v1::Termination::Interrupted as i32));
        let controls = recovered.controls.as_ref().unwrap();
        assert!(controls.can_rerun && controls.can_inspect && controls.can_export);
        assert!(!controls.can_start && !controls.can_cancel);
        assert_eq!(telemetry.list_samples(request).await?.into_inner(), samples);
        let page = query.list_executions(v1::QueryListExecutionsRequest::default()).await?.into_inner();
        assert_eq!(page.rows, vec![recovered.summary.clone().unwrap()]);
        let counts = page.outcome_counts.unwrap();
        assert_eq!((counts.evaluated_count, counts.not_evaluated_count, counts.declared_count), (0, 1, 0));
        assert_eq!(counts.result_status, None);
        assert_eq!(recovered.cycle_counts.as_ref(), Some(&counts));
        let saved = plugins.list(v1::ListPlugins {}).await?.into_inner();
        assert_eq!(saved.plugins.len(), 1);
        assert_eq!(saved.plugins[0].entry, entry.entry);
        assert_eq!(saved.plugins[0].state, v1::PluginState::Unavailable as i32);
        assert_eq!(saved.plugins[0].description, entry.description);
        let mut progress = observation
            .watch_execution_progress(v1::ObservationWatchExecutionProgressRequest { test_execution_id: identifier.clone() })
            .await?
            .into_inner();
        let mut terminal = false;
        while let Some(frame) = progress.message().await? {
            if matches!(frame.value, Some(v1::observation_watch_execution_progress_output::Value::Response(_))) {
                terminal = true;
            }
        }
        assert!(terminal);

        // Reattach the Plugin Without Replaying or Changing Recovered Evidence
        plugins.add(v1::AddRemotePlugin { url: plugin.endpoint().into() }).await?;
        assert_eq!(plugins.list(v1::ListPlugins {}).await?.into_inner().plugins[0].state, v1::PluginState::Ready as i32);
        assert_eq!(execution.get_execution(v1::ExecutionGetExecutionRequest { execution_id: identifier }).await?.into_inner(), recovered);
        process.close().await?;
        plugin.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
