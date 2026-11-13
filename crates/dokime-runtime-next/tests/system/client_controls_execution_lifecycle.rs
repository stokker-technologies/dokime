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
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let settings = super::support::settings();
        let mut plugin = PluginProcess::start(&super::support::thermal_fixture()?, true, &settings.placement).await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = HostClient::new(channel.clone(), super::support::client_settings())?;
        client.plugins.add(plugin.endpoint()).await?;
        let mut execution = v1::executions_client::ExecutionsClient::new(channel.clone());
        let cycle = v1::cycles_client::CyclesClient::connect(process.url.clone()).await?.create(v1::CyclesCreateRequest {request_id:uuid::Uuid::now_v7().to_string(), metadata: Some(v1::Metadata { label: "Operator Cycle".into(), ..Default::default() }), ..Default::default() }).await?.into_inner();
        let cycle=cycle.cycle.unwrap();
        let cases=v1::cases_client::CasesClient::new(channel.clone()).create(v1::CasesCreateRequest {request_id:"lifecycle-plan".into(),plan:Some(v1::SelectionPlan {cycle_id:cycle.cycle_id.clone(),selections:vec![v1::RegisteredSelection {suite_id:"thermal".into(),template_id:"thermal.cycle".into(),..Default::default()}],..Default::default()}),validate_only:false}).await?.into_inner().cases;
        let case_id = cases[0].case_id.clone();
        let listed = v1::cases_client::CasesClient::connect(process.url.clone()).await?.list(v1::CasesListRequest { parent: cycle.cycle_id.clone(), ..Default::default()}).await?.into_inner();
        assert_eq!(listed.cases.len(), 1);
        assert_eq!(v1::cases_client::CasesClient::connect(process.url.clone()).await?.get(v1::CasesGetRequest { parent: cycle.cycle_id.clone(), name: case_id.clone(), ..Default::default()}).await?.into_inner().value.unwrap().parameter_count, 1);
        assert_eq!(v1::cases_client::CasesClient::connect(process.url.clone()).await?.parameters(v1::CasesParametersRequest { parent: cycle.cycle_id.clone(), name: case_id.clone(), ..Default::default()}).await?.into_inner().parameters[0].name, "limit");

        // Append an Explicit Execution and Inspect Its Start Capability
        let identifier = "019a0000-0000-7000-8000-000000000001".to_owned();
        v1::executions_client::ExecutionsClient::connect(process.url.clone()).await?.create(v1::ExecutionsCreateRequest {request_id:uuid::Uuid::now_v7().to_string(), parent: cycle.cycle_id.clone(), executions: vec![v1::ExecutionBinding { execution_id: identifier.clone(), case_id: case_id.clone() }], }).await?;
        assert!(v1::executions_client::ExecutionsClient::connect(process.url.clone()).await?.get(v1::ExecutionsGetRequest { name: identifier.clone(), ..Default::default()}).await?.into_inner().execution.unwrap().controls.unwrap().can_start);
        assert!(v1::cases_client::CasesClient::connect(process.url.clone()).await?.delete(v1::CasesDeleteRequest { parent: cycle.cycle_id.clone(), name: case_id.clone(), ..Default::default() }).await.is_err());
        let started = v1::executions_client::ExecutionsClient::connect(process.url.clone()).await?.start(v1::ExecutionsStartRequest {request_id:uuid::Uuid::now_v7().to_string(), name: identifier.clone(), mode: v1::ExecutionMode::Interactive as i32, }).await?.into_inner();
        assert!(started.receipt.unwrap().completed);
        let mut progress = v1::executions::progress_client::ProgressClient::connect(process.url.clone()).await?.watch(v1::executions::ProgressWatchRequest { name: identifier.clone() , at: None, }).await?.into_inner();

        // Pause and Resume Between Interactive Commands
        let current=execution.get(v1::ExecutionsGetRequest {name:identifier.clone(),at:None}).await?.into_inner();let mut value=current.value.unwrap();value.paused=true;
        let paused=execution.set(v1::ExecutionsSetRequest {name:identifier.clone(),value:Some(value.clone()),expected_revision:current.revision,request_id:"pause".into()}).await?.into_inner();assert!(paused.value.as_ref().unwrap().paused);
        value.paused=false;execution.set(v1::ExecutionsSetRequest {name:identifier.clone(),value:Some(value),expected_revision:paused.revision,request_id:"resume".into()}).await?;
        let mut runtime = v1::backend::overview_client::OverviewClient::connect(process.url.clone()).await?.watch(v1::backend::OverviewWatchRequest::default()).await?.into_inner();
        runtime.message().await?.unwrap();

        // Run One Step and Resolve Its Receipt Through State Events
        let command = v1::executions::CommandsCreateRequest { request_id: identifier.clone(), parent: identifier.clone(), command_id: "ramp-command".into(), action: v1::ExecutionCommandAction::Next as i32, step_id: None, };
        v1::executions::commands_client::CommandsClient::connect(process.url.clone()).await?.create(command.clone()).await?;
        loop {
            let receipt = v1::executions::commands_client::CommandsClient::connect(process.url.clone()).await?.get(v1::executions::CommandsGetRequest { parent: identifier.clone(), name: command.command_id.clone(), ..Default::default()}).await?.into_inner().command.unwrap();
            if receipt.phase == v1::ExecutionCommandPhase::Completed as i32 { break; }
            runtime.message().await?.unwrap();
        }
        let repeated = v1::executions::commands_client::CommandsClient::connect(process.url.clone()).await?.create(command).await?.into_inner().command.unwrap();
        assert_eq!(repeated.phase, v1::ExecutionCommandPhase::Completed as i32);
        assert_eq!(v1::executions::commands_client::CommandsClient::connect(process.url.clone()).await?.create(v1::executions::CommandsCreateRequest { request_id: identifier.clone(), parent: identifier.clone(), command_id: "repeat-ramp".into(), action: v1::ExecutionCommandAction::RunStep as i32, step_id: Some("ramp".into()), }).await?.into_inner().command.unwrap().phase,v1::ExecutionCommandPhase::Refused as i32);

        // Run the Prompt Step and Finish After Answering
        v1::executions::commands_client::CommandsClient::connect(process.url.clone()).await?.create(v1::executions::CommandsCreateRequest { request_id: identifier.clone(), parent: identifier.clone(), command_id: "hold-command".into(), action: v1::ExecutionCommandAction::Next as i32, step_id: None, }).await?;
        while let Some(frame) = progress.message().await? { if frame.phase == "prompt" { break; } }
        let prompt = client.prompts.pending(identifier.clone()).await?.prompts[0].prompt.as_ref().unwrap().prompt_id.clone();
        client.prompts.respond(prompt, "yes", true).await?;
        loop {
            let receipt = v1::executions::commands_client::CommandsClient::connect(process.url.clone()).await?.get(v1::executions::CommandsGetRequest { parent: identifier.clone(), name: "hold-command".into(), ..Default::default()}).await?.into_inner().command.unwrap();
            if receipt.phase == v1::ExecutionCommandPhase::Completed as i32 { break; }
            runtime.message().await?.unwrap();
        }
        let exhausted = v1::executions::commands_client::CommandsClient::connect(process.url.clone()).await?.create(v1::executions::CommandsCreateRequest { request_id: identifier.clone(),parent: identifier.clone(),command_id: "no-next-step".into(),action: v1::ExecutionCommandAction::Next as i32,step_id: None, }).await?.into_inner().command.unwrap();
        assert_eq!(exhausted.phase,v1::ExecutionCommandPhase::Refused as i32);
        assert!(v1::executions_client::ExecutionsClient::connect(process.url.clone()).await?.get(v1::ExecutionsGetRequest { name: identifier.clone(), ..Default::default()}).await?.into_inner().execution.unwrap().controls.unwrap().can_cancel);
        v1::executions::commands_client::CommandsClient::connect(process.url.clone()).await?.create(v1::executions::CommandsCreateRequest { request_id: identifier.clone(), parent: identifier.clone(), command_id: "finish-command".into(), action: v1::ExecutionCommandAction::Finish as i32, step_id: None, }).await?;
        while progress.message().await?.is_some() {}

        // Inspect the Sealed Run and Its Parameters and Ordered Steps
        let detail = v1::executions_client::ExecutionsClient::connect(process.url.clone()).await?.get(v1::ExecutionsGetRequest { name: identifier.clone(), ..Default::default()}).await?.into_inner().execution.unwrap();
        assert_eq!(detail.summary.unwrap().state, v1::ExecutionState::Sealed as i32);
        let steps = v1::executions_client::ExecutionsClient::connect(process.url.clone()).await?.steps(v1::ExecutionsStepsRequest { name: identifier.clone(), ..Default::default()}).await?.into_inner();
        assert_eq!(steps.steps.iter().map(|step| step.step_id.as_str()).collect::<Vec<_>>(), vec!["ramp", "hold"]);
        assert_eq!(v1::executions_client::ExecutionsClient::connect(process.url.clone()).await?.parameters(v1::ExecutionsParametersRequest { name: identifier.clone(), set: v1::ExecutionParameterSet::Inputs as i32, ..Default::default()}).await?.into_inner().parameters.len(), 1);
        assert_eq!(v1::executions_client::ExecutionsClient::connect(process.url.clone()).await?.list(v1::ExecutionsListRequest { parent: cycle.cycle_id.clone(), ..Default::default() }).await?.into_inner().executions.len(), 1);

        // Rename and Archive Without Changing the Execution Seal
        let current=execution.get(v1::ExecutionsGetRequest {name:identifier.clone(),at:None}).await?.into_inner();let seal=current.execution.as_ref().unwrap().seal.clone();let mut value=current.value.unwrap();value.label=uuid::Uuid::now_v7().to_string();value.archived=true;execution.set(v1::ExecutionsSetRequest {name:identifier.clone(),value:Some(value),expected_revision:current.revision,request_id:"review-execution".into()}).await?;
        assert_eq!(execution.get(v1::ExecutionsGetRequest {name:identifier.clone(),at:None}).await?.into_inner().execution.unwrap().seal,seal);
        let mut cycles=v1::cycles_client::CyclesClient::new(channel.clone());let current=cycles.get(v1::CyclesGetRequest {name:cycle.cycle_id.clone(),at:None}).await?.into_inner();let mut value=current.value.unwrap();value.label=uuid::Uuid::now_v7().to_string();value.archived=true;cycles.set(v1::CyclesSetRequest {name:cycle.cycle_id.clone(),value:Some(value),expected_revision:current.revision,request_id:"review-cycle".into()}).await?;
        cycles.seal(v1::CyclesSealRequest {name:cycle.cycle_id.clone(),request_id:"seal-cycle".into()}).await?;
        let sealed=cycles.get(v1::CyclesGetRequest {name:cycle.cycle_id.clone(),at:None}).await?.into_inner().cycle.unwrap();assert!(sealed.sealed && sealed.archived);

        // Rerun Frozen Origins into a New Cycle and Cancel Its Active Run
        let rerun = v1::cycles_client::CyclesClient::connect(process.url.clone()).await?.create(v1::CyclesCreateRequest { source_cycle: cycle.cycle_id.clone(),request_id:"rerun".into(),continue_automatically:true, ..Default::default() }).await?.into_inner();
        let rerun=rerun.cycle.unwrap();
        let run = v1::executions_client::ExecutionsClient::connect(process.url.clone()).await?.list(v1::ExecutionsListRequest { parent: rerun.cycle_id.clone(), ..Default::default() }).await?.into_inner().executions[0].summary.as_ref().unwrap().execution_id.clone();
        let mut watch = v1::executions::progress_client::ProgressClient::connect(process.url.clone()).await?.watch(v1::executions::ProgressWatchRequest { name: run.clone() , at: None, }).await?.into_inner();
        while let Some(frame) = watch.message().await? { if frame.phase == "prompt" { break; } }
        v1::executions_client::ExecutionsClient::connect(process.url.clone()).await?.cancel(v1::ExecutionsCancelRequest {request_id:uuid::Uuid::now_v7().to_string(), name: run.clone(),  }).await?;
        while watch.message().await?.is_some() {}
        assert!(matches!(v1::executions_client::ExecutionsClient::connect(process.url.clone()).await?.get(v1::ExecutionsGetRequest { name: run, ..Default::default()}).await?.into_inner().execution.unwrap().outcome.unwrap().attribution, Some(v1::execution_outcome::Attribution::NotEvaluated(outcome)) if outcome.termination == v1::Termination::Cancelled as i32));
        drop(runtime);
        process.close().await?;
        plugin.close().await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    }).await?
}
