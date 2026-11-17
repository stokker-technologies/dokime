/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Answers Operator Prompts

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::HashMap;
use std::error::Error;

// Third Party Imports
use harmos_protocol::host::plugin_placement;
use tonic::Code;
use tonic::transport::Endpoint;

// Local Imports
use dokime::Client;
use dokime::MapValue;
use dokime::Metadata;
use dokime::client::CyclesCreateOptions;
use dokime::client::ExecutionCommandAction;
use dokime::client::ExecutionCommandPhase;
use dokime::client::ExecutionMode;
use dokime::client::ExecutionsCommandsCreateOptions;
use dokime::client::ExecutionsProgressWatchOptions;
use dokime::client::ExecutionsStartOptions;
use dokime::client::ExecutionsStopOptions;
use dokime::client::PluginPlacement;
use dokime::client::PluginRemote;
use dokime::client::PluginValue;
use dokime::client::PromptsAnswersCreateOptions;
use dokime::client::PromptsListOptions;
use dokime_protocol::host::prompts::AnswersCreateRequest;
use dokime_protocol::model::origin;
use dokime_runtime::PluginProcess;
use super::client_runs_thermal_suite_over_grpc::HostProcess;

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
fn client_answers_operator_prompts() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Connect the Operator to a Real Host and Plugin
            let root = super::support::tempdir()?;
            let credential_root = super::support::tempdir()?;
            let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
            let settings = super::support::settings();
            let mut plugin = PluginProcess::start(&super::support::thermal_fixture()?, true, &settings.placement).await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel.clone(), super::support::client_settings())?;
            let prompts = Client::from_channel(channel.clone(), super::support::client_settings())?.prompts.clone();
            let answers = Client::from_channel(channel.clone(), super::support::client_settings())?.prompts.answers.clone();
            let definitions = Client::from_channel(channel.clone(), super::support::client_settings())?.prompt_definitions.clone();
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
            let catalog = definitions.list(String::new()).await?;
            assert_eq!(catalog.values.len(), 1);
            let definition = definitions.get("thermal.door", String::new(), Default::default()).await?.value.unwrap();
            assert_eq!(definition.metadata.as_ref().unwrap().identifier, catalog.values[0].metadata.as_ref().unwrap().identifier);
            assert_eq!(definitions.get("missing", String::new(), Default::default()).await.unwrap_err().code(), Code::NotFound);
            let cases = super::support::materialize(&client, "thermal", "thermal.cycle", HashMap::new()).await?;
            let mut repeated_origin = cases[0].origin.clone().unwrap();
            if let Some(origin::Kind::PluginRun(run)) = repeated_origin.kind.as_mut() {
                run.test_case.as_mut().unwrap().steps[1].repeatable = true;
            }
            let cycle = super::support::admit(&client, cases).await?;
            let execution = cycle[0].clone();
            let mut watch = client.executions.progress.watch(cycle[0].clone()).await?;
            while let Some(frame) = watch.next().await? {
                if frame.phase == "prompt" {
                    break;
                }
            }
            let pending = prompts
                .list_with(
                    String::new(),
                    true,
                    PromptsListOptions {
                        execution_id: Some(execution.clone()),
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(pending.prompts.len(), 1);
            let prompt_id = pending.prompts[0].prompt.as_ref().unwrap().prompt_id.clone();
            assert!(
                prompts
                    .list_with(
                        String::new(),
                        true,
                        PromptsListOptions {
                            execution_id: Some("another".into()),
                            ..Default::default()
                        }
                    )
                    .await?
                    .prompts
                    .is_empty()
            );
            assert!(
                answers
                    .create_with(
                        prompt_id.clone(),
                        "unknown",
                        Default::default(),
                        true,
                        PromptsAnswersCreateOptions {
                            request_id: Some("invalid".into()),
                            ..Default::default()
                        }
                    )
                    .await
                    .is_err()
            );
            assert!(prompts.get(prompt_id.clone()).await?.prompt.unwrap().response.is_none());
            let request = AnswersCreateRequest {
                parent: prompt_id.clone(),
                action: "yes".into(),
                accepted: true,
                request_id: "operator-answer".into(),
                ..Default::default()
            };
            let receipt = answers
                .create_with(
                    (request.clone()).parent,
                    (request.clone()).action,
                    ((request.clone()).values).unwrap_or_default(),
                    (request.clone()).accepted,
                    PromptsAnswersCreateOptions {
                        request_id: Some((request.clone()).request_id),
                        ..Default::default()
                    },
                )
                .await?;
            assert_eq!(
                answers
                    .create_with(
                        (request.clone()).parent,
                        (request.clone()).action,
                        ((request.clone()).values).unwrap_or_default(),
                        (request.clone()).accepted,
                        PromptsAnswersCreateOptions {
                            request_id: Some((request.clone()).request_id),
                            ..Default::default()
                        }
                    )
                    .await?,
                receipt
            );
            assert_eq!(receipt.answer.as_ref().unwrap().prompt_id, prompt_id);
            while watch.next().await?.is_some() {}
            let answered = prompts.get(prompt_id.clone()).await?.prompt.unwrap();
            assert!(answered.response.as_ref().unwrap().accepted);
            assert!(
                answers
                    .create_with(
                        prompt_id.clone(),
                        "no",
                        Default::default(),
                        Default::default(),
                        PromptsAnswersCreateOptions {
                            request_id: Some("another-answer".into()),
                            ..Default::default()
                        }
                    )
                    .await
                    .is_err()
            );
            assert!(prompts.list(String::new(), true).await?.prompts.is_empty());

            // Repeated Questions Keep Distinct Identities and Durable Answers.
            let cycles = Client::from_channel(channel.clone(), super::support::client_settings())?.cycles.clone();
            let admitted = cycles
                .create_with(
                    vec![repeated_origin],
                    Metadata {
                        label: "Repeated Questions".into(),
                        ..Default::default()
                    },
                    Default::default(),
                    String::new(),
                    CyclesCreateOptions {
                        request_id: Some("repeat-cycle".into()),
                        ..Default::default()
                    },
                )
                .await?;
            let parent = admitted.cycle.unwrap().cycle_id;
            let executions = Client::from_channel(channel.clone(), super::support::client_settings())?.executions.clone();
            let repeated = executions.list(parent, Default::default(), Default::default(), Default::default()).await?.executions[0]
                .summary
                .as_ref()
                .unwrap()
                .execution_id
                .clone();
            executions
                .start_with(
                    repeated.clone(),
                    ExecutionMode::try_from(ExecutionMode::Interactive as i32)?,
                    ExecutionsStartOptions {
                        request_id: Some("interactive".into()),
                        ..Default::default()
                    },
                )
                .await?;
            let mut runtime =
                Client::from_channel(channel.clone(), super::support::client_settings())?.backend.overview.clone().watch().await?;
            runtime.next().await?.unwrap();
            let commands = Client::from_channel(channel.clone(), super::support::client_settings())?.executions.commands.clone();
            let mut progress = Client::from_channel(channel, super::support::client_settings())?
                .executions
                .progress
                .clone()
                .watch_with(repeated.clone(), ExecutionsProgressWatchOptions { at: None, ..Default::default() })
                .await?;
            let mut questions = Vec::new();
            for command in ["first-question", "second-question"] {
                commands
                    .create_with(
                        repeated.clone(),
                        command,
                        ExecutionCommandAction::try_from(ExecutionCommandAction::RunStep as i32)?,
                        ExecutionsCommandsCreateOptions {
                            request_id: Some(command.into()),
                            step_id: Some("hold".into()),
                            ..Default::default()
                        },
                    )
                    .await?;
                let question = loop {
                    let pending = client.prompts.list(repeated.clone(), true).await?;
                    if let Some(view) = pending.prompts.first() {
                        break view.prompt.as_ref().unwrap().prompt_id.clone();
                    }
                    runtime.next().await?.unwrap();
                };
                assert!(!questions.contains(&question));
                client.prompts.answers.create(question.clone(), "yes", MapValue::default(), true).await?;
                questions.push(question);
                loop {
                    let saved = commands.get(command, repeated.clone()).await?.command.unwrap();
                    if saved.phase == ExecutionCommandPhase::Completed as i32 {
                        break;
                    }
                    runtime.next().await?.unwrap();
                }
            }
            for question in questions {
                assert!(client.prompts.get(question).await?.prompt.unwrap().response.unwrap().accepted);
            }
            executions
                .stop_with(
                    repeated,
                    ExecutionsStopOptions {
                        request_id: Some("stop".into()),
                        ..Default::default()
                    },
                )
                .await?;
            while progress.next().await?.is_some() {}
            drop(runtime);
            process.close().await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let prompts =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .prompts
                    .clone();
            let reopened = prompts.get(prompt_id).await?.prompt.unwrap();
            assert_eq!(reopened, answered);
            process.close().await?;
            plugin.close().await?;
            Ok::<_, Box<dyn Error>>(())
        })
        .await?
    }))
}
