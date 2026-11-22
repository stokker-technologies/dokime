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
use dokime_protocol::host::plugin_placement;
use tokio::time;
use tonic::Code;
use tonic::transport::Endpoint;
use uuid::Uuid;

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime::MapValue;
use dokime::Metadata;
use dokime::Termination;
use dokime_client_internal::CasesCreateOptions;
use dokime_client_internal::CasesListOptions;
use dokime_client_internal::Client;
use dokime_client_internal::CyclesCreateOptions;
use dokime_client_internal::CyclesGetOptions;
use dokime_client_internal::CyclesSealOptions;
use dokime_client_internal::CyclesSetOptions;
use dokime_client_internal::ExecutionBinding;
use dokime_client_internal::ExecutionCommandAction;
use dokime_client_internal::ExecutionCommandPhase;
use dokime_client_internal::ExecutionMode;
use dokime_client_internal::ExecutionParameterSet;
use dokime_client_internal::ExecutionState;
use dokime_client_internal::ExecutionsCancelOptions;
use dokime_client_internal::ExecutionsCommandsCreateOptions;
use dokime_client_internal::ExecutionsCommandsGetOptions;
use dokime_client_internal::ExecutionsCreateOptions;
use dokime_client_internal::ExecutionsGetOptions;
use dokime_client_internal::ExecutionsProgressWatchOptions;
use dokime_client_internal::ExecutionsSetOptions;
use dokime_client_internal::ExecutionsStartOptions;
use dokime_client_internal::RegisteredSelection;
use dokime_client_internal::SelectionPlan;
use dokime_client_internal::host::PluginPlacement;
use dokime_client_internal::host::PluginRemote;
use dokime_client_internal::host::PluginValue;
use dokime_client_internal::{ArchiveScope, CycleFilter, CyclePhase, CycleSelection};
use dokime_protocol::host::PluginState;
use dokime_protocol::host::execution_outcome;
use dokime_protocol::host::executions::CommandsCreateRequest;
use dokime_runtime::{PluginProcess, services};

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
        time::timeout(super::support::SYSTEM_HANG_GUARD, async {
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
        super::support::plugin_state(&client.plugins, entry, PluginState::Ready).await?;
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
                    request_id: Some(Uuid::now_v7().to_string()),
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
                        suite_id: "thermal.thermal".into(),
                        template_id: "thermal.thermal.cycle".into(),
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
                suite_id: "thermal.thermal".into(),
                template_id: "thermal.thermal.cycle".into(),
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
                    request_id: Some(Uuid::now_v7().to_string()),
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
                    request_id: Some(Uuid::now_v7().to_string()),
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
            action: ExecutionCommandAction::Next as i32,
            step_id: None,
        };
        let admitted = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .executions
            .commands
            .clone()
            .create_with(
                (command.clone()).parent,
                (command.clone()).command_id,
                ExecutionCommandAction::try_from((command.clone()).action)?,
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
            if receipt.phase == ExecutionCommandPhase::Completed as i32 {
                break;
            }
            runtime.next().await?.unwrap();
        }
        let repeated = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .executions
            .commands
            .clone()
            .create_with(
                (command.clone()).parent,
                (command.clone()).command_id,
                ExecutionCommandAction::try_from((command.clone()).action)?,
                ExecutionsCommandsCreateOptions {
                    request_id: Some((command.clone()).request_id),
                    step_id: (command.clone()).step_id,
                    ..Default::default()
                },
            )
            .await?;
        assert_eq!(repeated.receipt, admitted.receipt);
        let repeated = repeated.command.unwrap();
        assert_eq!(repeated.phase, ExecutionCommandPhase::Completed as i32);
        assert_eq!(
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .executions
                .commands
                .clone()
                .create_with(
                    identifier.clone(),
                    "repeat-ramp",
                    ExecutionCommandAction::try_from(ExecutionCommandAction::RunStep as i32)?,
                    ExecutionsCommandsCreateOptions {
                        request_id: Some(identifier.clone()),
                        step_id: Some("thermal.thermal.cycle.ramp".into()),
                        ..Default::default()
                    }
                )
                .await?
                .command
                .unwrap()
                .phase,
            ExecutionCommandPhase::Refused as i32
        );

        // Run the Prompt Step and Finish After Answering
        Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .executions
            .commands
            .clone()
            .create_with(
                identifier.clone(),
                "hold-command",
                ExecutionCommandAction::try_from(ExecutionCommandAction::Next as i32)?,
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
        let held = client.executions.commands.get("hold-command", identifier.clone()).await?;
        let prompt = client.prompts.list(identifier.clone(), true).await?.prompts[0].prompt.as_ref().unwrap().prompt_id.clone();
        client.prompts.answers.create(prompt, "0", MapValue::default(), true).await?;
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
            if receipt.phase == ExecutionCommandPhase::Completed as i32 {
                break;
            }
            runtime.next().await?.unwrap();
        }
        // Retain the Running Command at Its Issued Read Cut
        let retained = client.executions.commands.get_with("hold-command", identifier.clone(), ExecutionsCommandsGetOptions {
            at: held.resolved_at,
            ..Default::default()
        }).await?;
        assert_eq!(retained.command, held.command);

        let exhausted = Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
            .executions
            .commands
            .clone()
            .create_with(
                identifier.clone(),
                "no-next-step",
                ExecutionCommandAction::try_from(ExecutionCommandAction::Next as i32)?,
                ExecutionsCommandsCreateOptions {
                    request_id: Some(identifier.clone()),
                    step_id: None,
                    ..Default::default()
                },
            )
            .await?
            .command
            .unwrap();
        assert_eq!(exhausted.phase, ExecutionCommandPhase::Refused as i32);
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
                ExecutionCommandAction::try_from(ExecutionCommandAction::Finish as i32)?,
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
        assert_eq!(steps.steps.iter().map(|step| step.step_id.as_str()).collect::<Vec<_>>(), vec!["thermal.thermal.cycle.ramp", "thermal.thermal.cycle.hold"]);
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
        value.label = Uuid::now_v7().to_string();
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
        value.label = Uuid::now_v7().to_string();
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
        let mut completion = cycles.watch(CycleSelection { cycle_ids: vec![cycle.cycle_id.clone()], ..Default::default() }, CycleFilter { archive: Some(ArchiveScope::All as i32), ..Default::default() }, Default::default()).await?;
        while let Some(frame) = completion.next().await? {
            assert!(frame.resync_required.is_none());
            let cycle = &frame.cycles[0];
            assert_ne!(cycle.phase, CyclePhase::SealFailed as i32);
            if cycle.sealed { break; }
        }
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
                    request_id: Some(Uuid::now_v7().to_string()),
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
        assert_eq!(command.phase, ExecutionCommandPhase::Completed as i32);
        process.close().await?;
        plugin.close().await?;
        Ok::<_, Box<dyn Error>>(())
    }).await?
    }))
}

#[test]
fn unfinished_command_receipts_interrupt_after_restart() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        time::timeout(super::support::SYSTEM_HANG_GUARD, async {
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
            super::support::plugin_state(&client.plugins, entry, PluginState::Ready).await?;

            // Admit an Interactive Execution
            let cases = super::support::materialize(&client, "thermal.thermal", "thermal.thermal.cycle", Default::default()).await?;
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
                .create_with(
                    identifier.clone(),
                    "interrupted-command",
                    ExecutionCommandAction::RunStep,
                    ExecutionsCommandsCreateOptions {
                        step_id: Some("thermal.thermal.cycle.hold".into()),
                        ..Default::default()
                    },
                )
                .await?;
            let mut prompted = false;
            while let Some(frame) = progress.next().await? {
                if frame.phase == "prompt" {
                    prompted = true;
                    break;
                }
            }
            assert!(prompted);
            let held = client.executions.commands.get("interrupted-command", identifier.clone()).await?;
            let running = held.command.clone().unwrap();
            assert_eq!(running.phase, ExecutionCommandPhase::Running as i32);
            drop(progress);

            // Retain the Native Admission Cut While Its History Is Available
            let retained = client
                .executions
                .commands
                .get_with(
                    "interrupted-command",
                    identifier.clone(),
                    ExecutionsCommandsGetOptions {
                        at: held.resolved_at,
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(retained.command, held.command);

            // Interrupt the Process and Recover the Command Receipt
            process.kill().await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, super::support::client_settings())?;
            let recovered = client.executions.commands.get("interrupted-command", identifier.clone()).await?.command.unwrap();
            assert_eq!(recovered.phase, ExecutionCommandPhase::Interrupted as i32);
            assert_eq!(recovered.execution_id, running.execution_id);
            assert_eq!(recovered.command_id, running.command_id);
            assert_eq!(recovered.action, running.action);
            assert_eq!(recovered.accepted_at, running.accepted_at);

            // Replay the Original Durable Admission Receipt After Recovery
            let replayed = client
                .executions
                .commands
                .create_with(
                    identifier,
                    "interrupted-command",
                    ExecutionCommandAction::RunStep,
                    ExecutionsCommandsCreateOptions {
                        request_id: Some(running.request_id),
                        step_id: Some("thermal.thermal.cycle.hold".into()),
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(replayed.command, Some(recovered));
            assert_eq!(replayed.receipt, admitted.receipt);

            process.close().await?;
            plugin.close().await?;
            Ok::<_, Box<dyn Error>>(())
        })
        .await?
    }))
}

#[test]
fn cycle_origins_remain_registered_for_another_execution() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            let root = super::support::tempdir()?;
            let settings = super::support::settings();
            let mut plugin = PluginProcess::start(&super::support::thermal_fixture()?, true, &settings.placement).await?;
            let server = services::serve("127.0.0.1:0".parse()?, root.path(), settings, super::support::service_settings()).await?;
            let channel = Endpoint::from_shared(format!("http://{}", server.endpoint()))?.connect().await?;
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
            super::support::plugin_state(&client.plugins, entry, PluginState::Ready).await?;
            let cases = client
                .cases
                .create_with(
                    SelectionPlan {
                        selections: vec![RegisteredSelection {
                            suite_id: "thermal.thermal".into(),
                            template_id: "thermal.thermal.cycle".into(),
                            ..Default::default()
                        }],
                        ..Default::default()
                    },
                    false,
                    CasesCreateOptions::default(),
                )
                .await?
                .cases;

            for automatic in [false, true] {
                // Both Cycle Creation Modes Register the Origin Before Admitting Its First Run.
                let cycle = client
                    .cycles
                    .create_with(
                        vec![cases[0].origin.clone().unwrap()],
                        Metadata {
                            label: "Repeated Case".into(),
                            ..Default::default()
                        },
                        automatic,
                        String::new(),
                        CyclesCreateOptions::default(),
                    )
                    .await?
                    .cycle
                    .unwrap();
                let first = client
                    .executions
                    .list(cycle.cycle_id.clone(), Default::default(), Default::default(), Default::default())
                    .await?
                    .executions
                    .remove(0)
                    .summary
                    .unwrap();
                let registered = client.cases.list(cycle.cycle_id.clone()).await?.cases;
                assert_eq!(registered.len(), 1);
                assert_eq!(registered[0].case_id, first.case_id);
                assert_eq!(registered[0].template_id, "thermal.thermal.cycle");
                assert_eq!(registered[0].suite_id, "thermal.thermal");
                assert_eq!(client.cases.get(first.case_id.clone(), cycle.cycle_id.clone()).await?.value.unwrap().parameter_count, 1);
                if !automatic {
                    client.executions.start(first.execution_id.clone(), ExecutionMode::Automatic).await?;
                }
                let mut progress = client.executions.progress.watch(first.execution_id.clone()).await?;
                while let Some(frame) = progress.next().await? {
                    if frame.phase == "prompt" {
                        break;
                    }
                }
                client.executions.cancel(first.execution_id).await?;
                while progress.next().await?.is_some() {}

                // The Visible Case Is Runnable Again in the Same Cycle After Interruption.
                let next = Uuid::now_v7().to_string();
                let created = client
                    .executions
                    .create_with(
                        cycle.cycle_id.clone(),
                        vec![ExecutionBinding {
                            execution_id: next.clone(),
                            case_id: first.case_id.clone(),
                        }],
                        ExecutionsCreateOptions::default(),
                    )
                    .await?;
                assert_eq!(created.executions[0].summary.as_ref().unwrap().case_id, first.case_id);
                assert_eq!(client.cases.list(cycle.cycle_id.clone()).await?.cases[0].attempt_count, 2);
                if automatic {
                    let mut progress = client.executions.progress.watch(next.clone()).await?;
                    while let Some(frame) = progress.next().await? {
                        if frame.phase == "prompt" {
                            break;
                        }
                    }
                    client.executions.cancel(next).await?;
                    while progress.next().await?.is_some() {}
                }

                let missing = Uuid::now_v7().to_string();
                let refused = client
                    .executions
                    .create_with(
                        cycle.cycle_id,
                        vec![ExecutionBinding {
                            execution_id: Uuid::now_v7().to_string(),
                            case_id: missing.clone(),
                        }],
                        ExecutionsCreateOptions::default(),
                    )
                    .await
                    .unwrap_err();
                assert_eq!(refused.code(), Code::NotFound);
                assert!(refused.message().contains(&missing));
            }
            // A Case Registered Before the Cycle Also Remains Visible Through Its Binding.
            let cycle = client
                .cycles
                .create_with(
                    Vec::new(),
                    Metadata {
                        label: "Bound Case".into(),
                        ..Default::default()
                    },
                    false,
                    String::new(),
                    CyclesCreateOptions::default(),
                )
                .await?
                .cycle
                .unwrap();
            client
                .executions
                .create_with(
                    cycle.cycle_id.clone(),
                    vec![ExecutionBinding {
                        execution_id: Uuid::now_v7().to_string(),
                        case_id: cases[0].case_id.clone(),
                    }],
                    ExecutionsCreateOptions::default(),
                )
                .await?;
            assert_eq!(client.cases.list(cycle.cycle_id.clone()).await?.cases[0].case_id, cases[0].case_id);
            assert_eq!(client.cases.get(cases[0].case_id.clone(), cycle.cycle_id).await?.value.unwrap().parameter_count, 1);
            server.close().await?;
            plugin.close().await?;
            Ok::<_, Box<dyn Error>>(())
        })
        .await?
    }))
}
