/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Controls Execution Lifecycle

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
async fn client_controls_execution_lifecycle() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Create a Mutable Cycle and Register Its Cases Without Running
        let root = tempfile::tempdir()?;
        let credential_root = tempfile::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let settings = super::support::settings();
        let mut plugin = PluginProcess::start(&super::support::thermal_fixture()?, true, &settings.placement).await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = HostClient::new(channel.clone());
        client.plugins.add(plugin.endpoint()).await?;
        let mut execution = v1::execution_client::ExecutionClient::new(channel.clone());
        let mut observation = v1::observation_client::ObservationClient::new(channel.clone());
        let cycle = execution.create_cycle(v1::ExecutionCreateCycleRequest { metadata: Some(v1::Metadata { label: "Operator Cycle".into(), ..Default::default() }), ..Default::default() }).await?.into_inner();
        let cases = v1::planning_client::PlanningClient::new(channel.clone()).register_selection(v1::PlanningRegisterSelectionRequest { idempotency_key: "lifecycle-plan".into(), plan: Some(v1::SelectionPlan { cycle_id: cycle.cycle_id.clone(), selections: vec![v1::RegisteredSelection { suite_id: "thermal".into(), template_id: "thermal.cycle".into(), ..Default::default() }], ..Default::default() }) }).await?.into_inner().cases;
        let case_id = cases[0].case_id.clone();
        let listed = execution.list_cases(v1::ExecutionListCasesRequest { cycle_id: cycle.cycle_id.clone(), ..Default::default() }).await?.into_inner();
        assert_eq!(listed.cases.len(), 1);
        assert_eq!(execution.get_case(v1::ExecutionGetCaseRequest { cycle_id: cycle.cycle_id.clone(), case_id: case_id.clone() }).await?.into_inner().parameter_count, 1);
        assert_eq!(execution.list_case_parameters(v1::ExecutionListCaseParametersRequest { cycle_id: cycle.cycle_id.clone(), case_id: case_id.clone(), ..Default::default() }).await?.into_inner().parameters[0].name, "limit");

        // Append an Explicit Execution and Inspect Its Start Capability
        let identifier = "019a0000-0000-7000-8000-000000000001".to_owned();
        execution.append_executions(v1::ExecutionAppendExecutionsRequest { cycle_id: cycle.cycle_id.clone(), executions: vec![v1::ExecutionBinding { execution_id: identifier.clone(), case_id: case_id.clone() }] }).await?;
        assert!(execution.get_execution(v1::ExecutionGetExecutionRequest { execution_id: identifier.clone() }).await?.into_inner().controls.unwrap().can_start);
        assert!(execution.delete_case(v1::ExecutionDeleteCaseRequest { cycle_id: cycle.cycle_id.clone(), case_id: case_id.clone(), ..Default::default() }).await.is_err());
        let started = execution.start_execution(v1::ExecutionStartExecutionRequest { execution_id: identifier.clone(), mode: v1::ExecutionMode::Interactive as i32 }).await?.into_inner();
        assert!(matches!(started.outcome.unwrap().kind, Some(v1::start_execution_outcome::Kind::Accepted(_))));
        let mut progress = observation.watch_execution_progress(v1::ObservationWatchExecutionProgressRequest { test_execution_id: identifier.clone() }).await?.into_inner();

        // Pause and Resume Between Interactive Commands
        execution.pause_execution(v1::ExecutionPauseExecutionRequest { execution_id: identifier.clone(), ..Default::default() }).await?;
        assert!(execution.get_execution(v1::ExecutionGetExecutionRequest { execution_id: identifier.clone() }).await?.into_inner().controls.unwrap().paused);
        execution.resume_execution(v1::ExecutionResumeExecutionRequest { execution_id: identifier.clone(), ..Default::default() }).await?;
        let mut runtime = observation.watch_runtime(v1::ObservationWatchRuntimeRequest::default()).await?.into_inner();
        runtime.message().await?.unwrap();

        // Run One Step and Resolve Its Receipt Through State Events
        let command = v1::ExecutionSubmitExecutionCommandRequest { request_id: identifier.clone(), execution_id: identifier.clone(), command_id: "ramp-command".into(), action: v1::ExecutionCommandAction::Next as i32, step_id: None };
        execution.submit_execution_command(command.clone()).await?;
        loop {
            let receipt = execution.get_execution_command(v1::ExecutionGetExecutionCommandRequest { request_id: identifier.clone(), command_id: command.command_id.clone() }).await?.into_inner();
            if receipt.phase == v1::ExecutionCommandPhase::Completed as i32 { break; }
            runtime.message().await?.unwrap();
        }
        let repeated = execution.submit_execution_command(command).await?.into_inner();
        assert_eq!(repeated.phase, v1::ExecutionCommandPhase::Completed as i32);
        assert_eq!(execution.submit_execution_command(v1::ExecutionSubmitExecutionCommandRequest { request_id: identifier.clone(), execution_id: identifier.clone(), command_id: "repeat-ramp".into(), action: v1::ExecutionCommandAction::RunStep as i32, step_id: Some("ramp".into()) }).await?.into_inner().phase,v1::ExecutionCommandPhase::Refused as i32);

        // Run the Prompt Step and Finish After Answering
        execution.submit_execution_command(v1::ExecutionSubmitExecutionCommandRequest { request_id: identifier.clone(), execution_id: identifier.clone(), command_id: "hold-command".into(), action: v1::ExecutionCommandAction::Next as i32, step_id: None }).await?;
        while let Some(frame) = progress.message().await? { if matches!(frame.value, Some(v1::observation_watch_execution_progress_output::Value::Item(frame)) if frame.progress.as_ref().is_some_and(|progress| progress.phase == "prompt")) { break; } }
        let prompt = client.prompts.pending(identifier.clone()).await?.prompts[0].prompt_id.clone();
        client.prompts.respond(prompt, "yes", true).await?;
        loop {
            let receipt = execution.get_execution_command(v1::ExecutionGetExecutionCommandRequest { request_id: identifier.clone(), command_id: "hold-command".into() }).await?.into_inner();
            if receipt.phase == v1::ExecutionCommandPhase::Completed as i32 { break; }
            runtime.message().await?.unwrap();
        }
        let exhausted = execution.submit_execution_command(v1::ExecutionSubmitExecutionCommandRequest { request_id: identifier.clone(),execution_id: identifier.clone(),command_id: "no-next-step".into(),action: v1::ExecutionCommandAction::Next as i32,step_id: None }).await?.into_inner();
        assert_eq!(exhausted.phase,v1::ExecutionCommandPhase::Refused as i32);
        assert!(execution.get_execution(v1::ExecutionGetExecutionRequest { execution_id: identifier.clone() }).await?.into_inner().controls.unwrap().can_cancel);
        execution.submit_execution_command(v1::ExecutionSubmitExecutionCommandRequest { request_id: identifier.clone(), execution_id: identifier.clone(), command_id: "finish-command".into(), action: v1::ExecutionCommandAction::Finish as i32, step_id: None }).await?;
        while progress.message().await?.is_some() {}

        // Inspect the Sealed Run and Its Parameters and Ordered Steps
        let detail = execution.get_execution(v1::ExecutionGetExecutionRequest { execution_id: identifier.clone() }).await?.into_inner();
        assert_eq!(detail.summary.unwrap().state, v1::ExecutionState::Sealed as i32);
        let steps = execution.list_execution_steps(v1::ExecutionListExecutionStepsRequest { execution_id: identifier.clone(), ..Default::default() }).await?.into_inner();
        assert_eq!(steps.steps.iter().map(|step| step.step_id.as_str()).collect::<Vec<_>>(), vec!["ramp", "hold"]);
        assert_eq!(execution.list_execution_parameters(v1::ExecutionListExecutionParametersRequest { execution_id: identifier.clone(), set: v1::ExecutionParameterSet::Inputs as i32, ..Default::default() }).await?.into_inner().parameters.len(), 1);
        assert_eq!(execution.list_executions(v1::ExecutionListExecutionsRequest { cycle_id: cycle.cycle_id.clone(), ..Default::default() }).await?.into_inner().executions.len(), 1);

        // Rename and Archive Without Changing the Execution Seal
        execution.rename_execution(v1::ExecutionRenameExecutionRequest { execution_id: identifier.clone(), label: "Accepted Run".into(), ..Default::default() }).await?;
        execution.set_execution_archived(v1::ExecutionSetExecutionArchivedRequest { execution_id: identifier.clone(), archived: true, ..Default::default() }).await?;
        execution.rename_cycle(v1::ExecutionRenameCycleRequest { cycle_id: cycle.cycle_id.clone(), label: "Reviewed Cycle".into(), ..Default::default() }).await?;
        execution.seal_cycle(v1::ExecutionSealCycleRequest { cycle_id: cycle.cycle_id.clone(), ..Default::default() }).await?;
        execution.set_cycle_archived(v1::ExecutionSetCycleArchivedRequest { cycle_id: cycle.cycle_id.clone(), archived: true, ..Default::default() }).await?;
        let sealed = execution.get_cycle(v1::ExecutionGetCycleRequest { cycle_id: cycle.cycle_id.clone() }).await?.into_inner();
        assert!(sealed.sealed && sealed.archived);
        assert_eq!(sealed.metadata.unwrap().label, "Reviewed Cycle");

        // Rerun Frozen Origins into a New Cycle and Cancel Its Active Run
        let rerun = execution.rerun_cycle(v1::ExecutionRerunCycleRequest { source_cycle_id: cycle.cycle_id.clone(), ..Default::default() }).await?.into_inner();
        let run = execution.list_executions(v1::ExecutionListExecutionsRequest { cycle_id: rerun.cycle_id.clone(), ..Default::default() }).await?.into_inner().executions[0].execution_id.clone();
        let mut watch = observation.watch_execution_progress(v1::ObservationWatchExecutionProgressRequest { test_execution_id: run.clone() }).await?.into_inner();
        while let Some(frame) = watch.message().await? { if matches!(frame.value, Some(v1::observation_watch_execution_progress_output::Value::Item(frame)) if frame.progress.as_ref().is_some_and(|progress| progress.phase == "prompt")) { break; } }
        execution.cancel_execution(v1::ExecutionCancelExecutionRequest { execution_id: run.clone(), ..Default::default() }).await?;
        while watch.message().await?.is_some() {}
        assert!(matches!(execution.get_execution(v1::ExecutionGetExecutionRequest { execution_id: run }).await?.into_inner().outcome.unwrap().attribution, Some(v1::execution_outcome::Attribution::NotEvaluated(outcome)) if outcome.termination == v1::Termination::Cancelled as i32));
        drop(runtime);
        process.close().await?;
        plugin.close().await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    }).await?
}
