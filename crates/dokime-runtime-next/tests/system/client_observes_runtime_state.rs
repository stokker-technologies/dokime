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
        let root = tempfile::tempdir()?;
        let credential_root = tempfile::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let settings = super::support::settings();
        let mut plugin = PluginProcess::start(&super::support::thermal_fixture()?, true, &settings.placement).await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = HostClient::new(channel.clone(), super::support::client_settings())?;
        let mut observation = v1::observation_client::ObservationClient::new(channel.clone());
        let mut watch = observation.watch_runtime(v1::ObservationWatchRuntimeRequest::default()).await?.into_inner();
        let Some(v1::observation_watch_runtime_output::Value::Item(initial)) = watch.message().await?.unwrap().value else { panic!("Missing Initial Runtime State"); };
        assert!(initial.resynchronized);
        assert_eq!(initial.cycle_count, 0);
        let mut future = initial.cursor.clone().unwrap();
        future.position = u64::MAX;
        assert_eq!(observation.watch_runtime(v1::ObservationWatchRuntimeRequest { cursor: Some(future) }).await.unwrap_err().code(), tonic::Code::FailedPrecondition);
        client.plugins.add(plugin.endpoint()).await?;
        let Some(v1::observation_watch_runtime_output::Value::Item(changed)) = watch.message().await?.unwrap().value else { panic!("Missing Plugin State Change"); };
        assert!(!changed.resynchronized);
        assert!(changed.cursor.unwrap().position > initial.cursor.unwrap().position);

        // Start a Cycle and Await Progress Through the Bidirectional Service
        let cases = client.templates.materialize("thermal", "thermal.cycle", std::collections::HashMap::new()).await?;
        let cycle = client.cycles.start(cases).await?;
        let identifier = cycle.executions[0].clone();
        let mut progress = observation.run_watch_execution_progress(tokio_stream::iter([
            v1::ObservationWatchExecutionProgressRunInput { value: Some(v1::observation_watch_execution_progress_run_input::Value::Request(v1::ObservationWatchExecutionProgressRequest { test_execution_id: identifier.clone() })) },
            v1::ObservationWatchExecutionProgressRunInput { value: Some(v1::observation_watch_execution_progress_run_input::Value::Finish(v1::ClientEmpty {})) },
        ])).await?.into_inner();
        while let Some(frame) = progress.message().await? { if matches!(frame.value, Some(v1::observation_watch_execution_progress_run_output::Value::Item(frame)) if frame.progress.as_ref().is_some_and(|progress| progress.phase == "prompt")) { break; } }
        let shell = observation.get_runtime_shell(v1::ObservationGetRuntimeShellRequest::default()).await?.into_inner();
        assert_eq!(shell.test_suite_count, 1);
        assert_eq!(shell.cycle_summaries[0].cycle_id, cycle.identifier);
        assert_eq!(shell.evidence_counts.unwrap().prompts, 1);
        let traceability = observation.get_traceability(v1::ObservationGetTraceabilityRequest { scope: Some(v1::EvidenceScope { execution_id: Some(identifier.clone()), ..Default::default() }), ..Default::default() }).await?.into_inner();
        assert!(traceability.complete && traceability.snapshots.is_empty());

        // Cancel a Watch and Verify That the Execution Still Waits for Its Answer
        let (sender,input) = tokio::sync::mpsc::channel(2);
        sender.send(v1::ObservationWatchRuntimeRunInput { value: Some(v1::observation_watch_runtime_run_input::Value::Request(v1::ObservationWatchRuntimeRequest::default())) }).await?;
        let mut run = observation.run_watch_runtime(tokio_stream::wrappers::ReceiverStream::new(input)).await?.into_inner();
        run.message().await?.unwrap();
        sender.send(v1::ObservationWatchRuntimeRunInput { value: Some(v1::observation_watch_runtime_run_input::Value::Cancel(v1::ClientEmpty {})) }).await?;
        assert_eq!(run.message().await.unwrap_err().code(), tonic::Code::Cancelled);
        assert!(client.executions.get(identifier.clone()).await?.controls.unwrap().can_cancel);
        let prompt = client.prompts.pending(identifier.clone()).await?.prompts[0].prompt_id.clone();
        client.prompts.respond(prompt,"yes",true).await?;
        let mut terminal = false;
        while let Some(frame) = progress.message().await? { if matches!(frame.value, Some(v1::observation_watch_execution_progress_run_output::Value::Response(_))) { terminal = true; } }
        assert!(terminal);

        // Observe Graceful Shutdown Through the Final Runtime Response
        process.close().await?;
        let mut stopped = false;
        let mut terminal = false;
        while let Some(frame) = watch.message().await? { match frame.value { Some(v1::observation_watch_runtime_output::Value::Item(frame)) => stopped |= frame.shutting_down, Some(v1::observation_watch_runtime_output::Value::Response(_)) => terminal = true, _ => {} } }
        assert!(stopped && terminal);
        plugin.close().await?;
        Ok::<_,Box<dyn std::error::Error>>(())
    }).await?
}
