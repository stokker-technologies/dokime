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
use std::error::Error;

// Third Party Imports
use harmos_protocol::host::plugin_placement;
use tonic::transport::Endpoint;

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime::Client;
use dokime::MapValue;
use dokime::Metadata;
use dokime::Termination;
use dokime::client::CasesCreateOptions;
use dokime::client::CasesListOptions;
use dokime::client::CyclesCreateOptions;
use dokime::client::CyclesGetOptions;
use dokime::client::CyclesSealOptions;
use dokime::client::CyclesSetOptions;
use dokime::client::ExecutionBinding;
use dokime::client::ExecutionMode;
use dokime::client::ExecutionParameterSet;
use dokime::client::ExecutionState;
use dokime::client::ExecutionsCancelOptions;
use dokime::client::ExecutionsCommandsCreateOptions;
use dokime::client::ExecutionsCommandsGetOptions;
use dokime::client::ExecutionsCreateOptions;
use dokime::client::ExecutionsExecutionCommandAction;
use dokime::client::ExecutionsExecutionCommandPhase;
use dokime::client::ExecutionsGetOptions;
use dokime::client::ExecutionsProgressWatchOptions;
use dokime::client::ExecutionsSetOptions;
use dokime::client::ExecutionsStartOptions;
use dokime::client::PluginPlacement;
use dokime::client::PluginRemote;
use dokime::client::PluginValue;
use dokime::client::RegisteredSelection;
use dokime::client::SelectionPlan;
use dokime_protocol::host::execution_outcome;
use dokime_protocol::host::executions::CommandsCreateRequest;
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
fn client_controls_execution_lifecycle() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Create a Mutable Cycle and Register Its Cases Without Running
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
        super::support::plugin_state(&client.plugins, entry, harmos_protocol::host::PluginState::Ready).await?;
        let execution = Client::from_channel(channel.clone(), super::support::client_settings())?.executions.clone();
        let cycle = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .cycles
            .clone()
            .create_with(
                Default::default(),
                Metadata {
                    label: "Operator Cycle".into(),
                    ..Default::default()
                },
                Default::default(),
                String::new(),
                CyclesCreateOptions {
                    request_id: Some(uuid::Uuid::now_v7().to_string()),
                    ..Default::default()
                },
            )
            .await?;
        let cycle = cycle.cycle.unwrap();
        let registered = Client::from_channel(channel.clone(), super::support::client_settings())?
            .cases
            .clone()
            .create_with(
                SelectionPlan {
                    cycle_id: cycle.cycle_id.clone(),
                    selections: vec![RegisteredSelection {
                        suite_id: "thermal".into(),
                        template_id: "thermal.cycle".into(),
                        ..Default::default()
                    }; 2],
                    ..Default::default()
                },
                false,
                CasesCreateOptions {
                    request_id: Some("lifecycle-plan".into()),
                    ..Default::default()
                },
            )
            .await?;
        let cases = registered.cases.clone();
        assert_eq!(registered.receipt.as_ref().unwrap().request_id, "lifecycle-plan");
        let case_id = cases[0].case_id.clone();
        let listed = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .cases
            .clone()
            .list(cycle.cycle_id.clone())
            .await?;
        assert_eq!(listed.cases.len(), 2);

        // Delete Only the Unexecuted Case and Preserve Its Retained Cut.
        let deleted = client.cases.delete(cases[1].case_id.clone(), cycle.cycle_id.clone()).await?;
        assert!(deleted.completed);
        assert_eq!(deleted.accepted_at, deleted.durable_at);
        let current = client.cases.list(cycle.cycle_id.clone()).await?;
        assert_eq!(current.cases.iter().map(|case| &case.case_id).collect::<Vec<_>>(), vec![&case_id]);
        let retained = client.cases.list_with(cycle.cycle_id.clone(), CasesListOptions {
            at: listed.resolved_at.clone(),
            ..Default::default()
        }).await?;
        assert_eq!(retained, listed);
        assert!(client.cases.get(cases[1].case_id.clone(), cycle.cycle_id.clone()).await.is_err());

        // Replaying Registration Returns the Original Receipt Without Recreating a Deleted Case.
        let replayed = client.cases.create_with(SelectionPlan {
            cycle_id: cycle.cycle_id.clone(),
            selections: vec![RegisteredSelection {
                suite_id: "thermal".into(),
                template_id: "thermal.cycle".into(),
                ..Default::default()
            }; 2],
            ..Default::default()
        }, false, CasesCreateOptions {
            request_id: Some("lifecycle-plan".into()),
            ..Default::default()
        }).await?;
        assert_eq!(replayed, registered);
        let current = client.cases.list(cycle.cycle_id.clone()).await?;
        assert_eq!(current.cases.iter().map(|case| &case.case_id).collect::<Vec<_>>(), vec![&case_id]);
        assert_eq!(
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .cases
                .clone()
                .get(case_id.clone(), cycle.cycle_id.clone())
                .await?
                .value
                .unwrap()
                .parameter_count,
            1
        );
        assert_eq!(
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .cases
                .clone()
                .parameters(case_id.clone(), cycle.cycle_id.clone())
                .await?
                .parameters[0]
                .name,
            "limit"
        );

        // Append an Explicit Execution and Inspect Its Start Capability
        let identifier = "019a0000-0000-7000-8000-000000000001".to_owned();
        Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .executions
            .clone()
            .create_with(
                cycle.cycle_id.clone(),
                vec![ExecutionBinding {
                    execution_id: identifier.clone(),
                    case_id: case_id.clone(),
                }],
                ExecutionsCreateOptions {
                    request_id: Some(uuid::Uuid::now_v7().to_string()),
                    ..Default::default()
                },
            )
            .await?;
        assert!(
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .executions
                .clone()
                .get(identifier.clone())
                .await?
                .execution
                .unwrap()
                .controls
                .unwrap()
                .can_start
        );
        assert!(
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .cases
                .clone()
                .delete(case_id.clone(), cycle.cycle_id.clone())
                .await
                .is_err()
        );
        let started = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .executions
            .clone()
            .start_with(
                identifier.clone(),
                ExecutionMode::try_from(ExecutionMode::Interactive as i32)?,
                ExecutionsStartOptions {
                    request_id: Some(uuid::Uuid::now_v7().to_string()),
                    ..Default::default()
                },
            )
            .await?;
        assert!(started.receipt.unwrap().completed);
        let mut progress =
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .executions
                .progress
                .clone()
                .watch_with(identifier.clone(), ExecutionsProgressWatchOptions { at: None, ..Default::default() })
                .await?;

        // Pause and Resume Between Interactive Commands
        let current = execution.get_with(identifier.clone(), ExecutionsGetOptions { at: None, ..Default::default() }).await?;
        let mut value = current.value.unwrap();
        value.paused = true;
        let paused = execution
            .set_with(
                identifier.clone(),
                value.clone(),
                current.revision,
                ExecutionsSetOptions {
                    request_id: Some("pause".into()),
                    ..Default::default()
                },
            )
            .await?;
        assert!(paused.value.as_ref().unwrap().paused);
        value.paused = false;
        execution
            .set_with(
                identifier.clone(),
                value,
                paused.revision,
                ExecutionsSetOptions {
                    request_id: Some("resume".into()),
                    ..Default::default()
                },
            )
            .await?;
        let mut runtime =
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .backend
                .overview
                .clone()
                .watch()
                .await?;
        runtime.next().await?.unwrap();

        // Run One Step and Resolve Its Receipt Through State Events
        let command = CommandsCreateRequest {
            request_id: identifier.clone(),
            parent: identifier.clone(),
            command_id: "ramp-command".into(),
            action: ExecutionsExecutionCommandAction::Next as i32,
            step_id: None,
        };
        let admitted = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .executions
            .commands
            .clone()
            .create_with(
                (command.clone()).parent,
                (command.clone()).command_id,
                ExecutionsExecutionCommandAction::try_from((command.clone()).action)?,
                ExecutionsCommandsCreateOptions {
                    request_id: Some((command.clone()).request_id),
                    step_id: (command.clone()).step_id,
                    ..Default::default()
                },
            )
            .await?;
        loop {
            let receipt =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .executions
                    .commands
                    .clone()
                    .get(command.command_id.clone(), identifier.clone())
                    .await?
                    .command
                    .unwrap();
            if receipt.phase == ExecutionsExecutionCommandPhase::Completed as i32 {
                break;
            }
            runtime.next().await?.unwrap();
        }
        // Retain the Command State at Its Native Admission Cut
        let retained = client.executions.commands.get_with(command.command_id.clone(), identifier.clone(), ExecutionsCommandsGetOptions {
            at: admitted.receipt.as_ref().and_then(|receipt| receipt.durable_at.clone()),
            ..Default::default()
        }).await?;
        assert_eq!(retained.command, admitted.command);

        let repeated = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .executions
            .commands
            .clone()
            .create_with(
                (command.clone()).parent,
                (command.clone()).command_id,
                ExecutionsExecutionCommandAction::try_from((command.clone()).action)?,
                ExecutionsCommandsCreateOptions {
                    request_id: Some((command.clone()).request_id),
                    step_id: (command.clone()).step_id,
                    ..Default::default()
                },
            )
            .await?
            .command
            .unwrap();
        assert_eq!(repeated.phase, ExecutionsExecutionCommandPhase::Completed as i32);
        assert_eq!(
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .executions
                .commands
                .clone()
                .create_with(
                    identifier.clone(),
                    "repeat-ramp",
                    ExecutionsExecutionCommandAction::try_from(ExecutionsExecutionCommandAction::RunStep as i32)?,
                    ExecutionsCommandsCreateOptions {
                        request_id: Some(identifier.clone()),
                        step_id: Some("ramp".into()),
                        ..Default::default()
                    }
                )
                .await?
                .command
                .unwrap()
                .phase,
            ExecutionsExecutionCommandPhase::Refused as i32
        );

        // Run the Prompt Step and Finish After Answering
        Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .executions
            .commands
            .clone()
            .create_with(
                identifier.clone(),
                "hold-command",
                ExecutionsExecutionCommandAction::try_from(ExecutionsExecutionCommandAction::Next as i32)?,
                ExecutionsCommandsCreateOptions {
                    request_id: Some(identifier.clone()),
                    step_id: None,
                    ..Default::default()
                },
            )
            .await?;
        while let Some(frame) = progress.next().await? {
            if frame.phase == "prompt" {
                break;
            }
        }
        let prompt = client.prompts.list(identifier.clone(), true).await?.prompts[0].prompt.as_ref().unwrap().prompt_id.clone();
        client.prompts.answers.create(prompt, "yes", MapValue::default(), true).await?;
        loop {
            let receipt =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .executions
                    .commands
                    .clone()
                    .get("hold-command", identifier.clone())
                    .await?
                    .command
                    .unwrap();
            if receipt.phase == ExecutionsExecutionCommandPhase::Completed as i32 {
                break;
            }
            runtime.next().await?.unwrap();
        }
        let exhausted = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .executions
            .commands
            .clone()
            .create_with(
                identifier.clone(),
                "no-next-step",
                ExecutionsExecutionCommandAction::try_from(ExecutionsExecutionCommandAction::Next as i32)?,
                ExecutionsCommandsCreateOptions {
                    request_id: Some(identifier.clone()),
                    step_id: None,
                    ..Default::default()
                },
            )
            .await?
            .command
            .unwrap();
        assert_eq!(exhausted.phase, ExecutionsExecutionCommandPhase::Refused as i32);
        assert!(
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .executions
                .clone()
                .get(identifier.clone())
                .await?
                .execution
                .unwrap()
                .controls
                .unwrap()
                .can_cancel
        );
        Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .executions
            .commands
            .clone()
            .create_with(
                identifier.clone(),
                "finish-command",
                ExecutionsExecutionCommandAction::try_from(ExecutionsExecutionCommandAction::Finish as i32)?,
                ExecutionsCommandsCreateOptions {
                    request_id: Some(identifier.clone()),
                    step_id: None,
                    ..Default::default()
                },
            )
            .await?;
        while progress.next().await?.is_some() {}

        // Inspect the Sealed Run and Its Parameters and Ordered Steps
        let detail = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .executions
            .clone()
            .get(identifier.clone())
            .await?
            .execution
            .unwrap();
        assert_eq!(detail.summary.unwrap().state, ExecutionState::Sealed as i32);
        let steps = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .executions
            .clone()
            .steps(identifier.clone())
            .await?;
        assert_eq!(steps.steps.iter().map(|step| step.step_id.as_str()).collect::<Vec<_>>(), vec!["ramp", "hold"]);
        assert_eq!(
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .executions
                .clone()
                .parameters(identifier.clone(), ExecutionParameterSet::try_from(ExecutionParameterSet::Inputs as i32)?)
                .await?
                .parameters
                .len(),
            1
        );
        assert_eq!(
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .executions
                .clone()
                .list(cycle.cycle_id.clone(), Default::default(), Default::default(), Default::default())
                .await?
                .executions
                .len(),
            1
        );

        // Rename and Archive Without Changing the Execution Seal
        let current = execution.get_with(identifier.clone(), ExecutionsGetOptions { at: None, ..Default::default() }).await?;
        let seal = current.execution.as_ref().unwrap().seal.clone();
        let mut value = current.value.unwrap();
        value.label = uuid::Uuid::now_v7().to_string();
        value.archived = true;
        execution
            .set_with(
                identifier.clone(),
                value,
                current.revision,
                ExecutionsSetOptions {
                    request_id: Some("review-execution".into()),
                    ..Default::default()
                },
            )
            .await?;
        assert_eq!(
            execution.get_with(identifier.clone(), ExecutionsGetOptions { at: None, ..Default::default() }).await?.execution.unwrap().seal,
            seal
        );
        let cycles = Client::from_channel(channel.clone(), super::support::client_settings())?.cycles.clone();
        let current = cycles.get_with(cycle.cycle_id.clone(), CyclesGetOptions { at: None, ..Default::default() }).await?;
        let mut value = current.value.unwrap();
        value.label = uuid::Uuid::now_v7().to_string();
        value.archived = true;
        cycles
            .set_with(
                cycle.cycle_id.clone(),
                value,
                current.revision,
                CyclesSetOptions {
                    request_id: Some("review-cycle".into()),
                    ..Default::default()
                },
            )
            .await?;
        cycles
            .seal_with(
                cycle.cycle_id.clone(),
                CyclesSealOptions {
                    request_id: Some("seal-cycle".into()),
                    ..Default::default()
                },
            )
            .await?;
        let sealed = cycles.get_with(cycle.cycle_id.clone(), CyclesGetOptions { at: None, ..Default::default() }).await?.cycle.unwrap();
        assert!(sealed.sealed && sealed.archived);

        // Rerun Frozen Origins into a New Cycle and Cancel Its Active Run
        let rerun = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .cycles
            .clone()
            .create_with(
                Default::default(),
                Metadata { label: "Repeated Cycle".into(), ..Default::default() },
                true,
                cycle.cycle_id.clone(),
                CyclesCreateOptions {
                    request_id: Some("rerun".into()),
                    ..Default::default()
                },
            )
            .await?;
        let rerun = rerun.cycle.unwrap();
        let run = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .executions
            .clone()
            .list(rerun.cycle_id.clone(), Default::default(), Default::default(), Default::default())
            .await?
            .executions[0]
            .summary
            .as_ref()
            .unwrap()
            .execution_id
            .clone();
        let mut watch = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .executions
            .progress
            .clone()
            .watch_with(run.clone(), ExecutionsProgressWatchOptions { at: None, ..Default::default() })
            .await?;
        while let Some(frame) = watch.next().await? {
            if frame.phase == "prompt" {
                break;
            }
        }
        Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .executions
            .clone()
            .cancel_with(
                run.clone(),
                ExecutionsCancelOptions {
                    request_id: Some(uuid::Uuid::now_v7().to_string()),
                    ..Default::default()
                },
            )
            .await?;
        while watch.next().await?.is_some() {}
        assert!(
            matches!(Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?.executions.clone().get(run).await?.execution.unwrap().outcome.unwrap().attribution, Some(execution_outcome::Attribution::NotEvaluated(outcome)) if outcome.termination == Termination::Cancelled as i32)
        );
        drop(runtime);
        process.close().await?;

        // Read the Durable Command After Restarting the Host
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let client = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?;
        let command = client.executions.commands.get("finish-command", identifier).await?.command.unwrap();
        assert_eq!(command.phase, ExecutionsExecutionCommandPhase::Completed as i32);
        process.close().await?;
        plugin.close().await?;
        Ok::<_, Box<dyn Error>>(())
    }).await?
    }))
}

#[test]
fn unfinished_command_receipts_interrupt_after_restart() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Start the Host and Thermal Plugin
            let root = super::support::tempdir()?;
            let credential_root = super::support::tempdir()?;
            let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
            let settings = super::support::settings();
            let mut plugin = PluginProcess::start(&super::support::thermal_fixture()?, true, &settings.placement).await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, super::support::client_settings())?;
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
            super::support::plugin_state(&client.plugins, entry, harmos_protocol::host::PluginState::Ready).await?;

            // Admit an Interactive Execution
            let cases = super::support::materialize(&client, "thermal", "thermal.cycle", Default::default()).await?;
            let cycle = client
                .cycles
                .create_with(
                    vec![cases[0].origin.clone().unwrap()],
                    Metadata {
                        label: "Interrupted Command".into(),
                        ..Default::default()
                    },
                    false,
                    String::new(),
                    CyclesCreateOptions::default(),
                )
                .await?
                .cycle
                .unwrap();
            let identifier =
                client.executions.list(cycle.cycle_id, Default::default(), Default::default(), Default::default()).await?.executions[0]
                    .summary
                    .as_ref()
                    .unwrap()
                    .execution_id
                    .clone();
            client.executions.start(identifier.clone(), ExecutionMode::Interactive).await?;
            let mut progress = client.executions.progress.watch(identifier.clone()).await?;

            // Run a Command Until Its Operator Prompt Is Observed
            let admitted = client
                .executions
                .commands
                .create(identifier.clone(), "interrupted-command", ExecutionsExecutionCommandAction::Next)
                .await?;
            let mut prompted = false;
            while let Some(frame) = progress.next().await? {
                if frame.phase == "prompt" {
                    prompted = true;
                    break;
                }
            }
            assert!(prompted);
            let running = client.executions.commands.get("interrupted-command", identifier.clone()).await?.command.unwrap();
            assert_eq!(running.phase, ExecutionsExecutionCommandPhase::Running as i32);
            drop(progress);

            // Interrupt the Process and Recover the Command Receipt
            process.kill().await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, super::support::client_settings())?;
            let recovered = client.executions.commands.get("interrupted-command", identifier.clone()).await?.command.unwrap();
            assert_eq!(recovered.phase, ExecutionsExecutionCommandPhase::Interrupted as i32);
            assert_eq!(recovered.execution_id, running.execution_id);
            assert_eq!(recovered.command_id, running.command_id);
            assert_eq!(recovered.action, running.action);
            assert_eq!(recovered.accepted_at, running.accepted_at);

            // Preserve the Native Admission Cut Across Recovery
            let retained = client
                .executions
                .commands
                .get_with(
                    "interrupted-command",
                    identifier,
                    ExecutionsCommandsGetOptions {
                        at: admitted.receipt.and_then(|receipt| receipt.durable_at),
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(retained.command, admitted.command);
            process.close().await?;
            plugin.close().await?;
            Ok::<_, Box<dyn Error>>(())
        })
        .await?
    }))
}
