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
async fn client_answers_operator_prompts() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Connect the Operator to a Real Host and Plugin
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let settings = super::support::settings();
        let mut plugin = PluginProcess::start(&super::support::thermal_fixture()?, true, &settings.placement).await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = HostClient::new(channel.clone(), super::support::client_settings())?;
        let mut prompts = v1::prompts_client::PromptsClient::new(channel.clone());
        let mut answers = v1::prompts::answers_client::AnswersClient::new(channel.clone());
        let mut definitions = v1::catalog::prompt_definitions_client::PromptDefinitionsClient::new(channel.clone());
        client.plugins.add(plugin.endpoint()).await?;
        let catalog = definitions.list(v1::catalog::PromptDefinitionsListRequest::default()).await?.into_inner();
        assert_eq!(catalog.values.len(), 1);
        let definition = definitions
            .get(v1::catalog::PromptDefinitionsGetRequest {
                name: "thermal.door".into(),
                ..Default::default()
            })
            .await?
            .into_inner()
            .value
            .unwrap();
        assert_eq!(definition.metadata.as_ref().unwrap().identifier, catalog.values[0].metadata.as_ref().unwrap().identifier);
        assert_eq!(
            definitions
                .get(v1::catalog::PromptDefinitionsGetRequest {
                    name: "missing".into(),
                    ..Default::default()
                })
                .await
                .unwrap_err()
                .code(),
            tonic::Code::NotFound
        );
        let cases = client.templates.materialize("thermal", "thermal.cycle", std::collections::HashMap::new()).await?;
        let mut repeated_origin = cases[0].origin.clone().unwrap();
        if let Some(v1::origin::Kind::PluginRun(run)) = repeated_origin.kind.as_mut() {
            run.test_case.as_mut().unwrap().steps[1].repeatable = true;
        }
        let cycle = client.cycles.start(cases).await?;
        let execution = cycle.executions[0].clone();
        let mut watch = cycle.watch();
        while let Some(frame) = watch.next().await? {
            if frame.phase == "prompt" {
                break;
            }
        }
        let pending = prompts
            .list(v1::PromptsListRequest {
                execution_id: Some(execution.clone()),
                pending_only: true,
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(pending.prompts.len(), 1);
        let prompt_id = pending.prompts[0].prompt.as_ref().unwrap().prompt_id.clone();
        assert!(
            prompts
                .list(v1::PromptsListRequest {
                    execution_id: Some("another".into()),
                    pending_only: true,
                    ..Default::default()
                })
                .await?
                .into_inner()
                .prompts
                .is_empty()
        );
        assert!(
            answers
                .create(v1::prompts::AnswersCreateRequest {
                    parent: prompt_id.clone(),
                    action: "unknown".into(),
                    accepted: true,
                    request_id: "invalid".into(),
                    ..Default::default()
                })
                .await
                .is_err()
        );
        assert!(
            prompts
                .get(v1::PromptsGetRequest {
                    name: prompt_id.clone(),
                    ..Default::default()
                })
                .await?
                .into_inner()
                .prompt
                .unwrap()
                .response
                .is_none()
        );
        let request = v1::prompts::AnswersCreateRequest {
            parent: prompt_id.clone(),
            action: "yes".into(),
            accepted: true,
            request_id: "operator-answer".into(),
            ..Default::default()
        };
        let receipt = answers.create(request.clone()).await?.into_inner();
        assert_eq!(answers.create(request).await?.into_inner(), receipt);
        assert_eq!(receipt.answer.as_ref().unwrap().prompt_id, prompt_id);
        while watch.next().await?.is_some() {}
        let answered = prompts
            .get(v1::PromptsGetRequest {
                name: prompt_id.clone(),
                ..Default::default()
            })
            .await?
            .into_inner()
            .prompt
            .unwrap();
        assert!(answered.response.as_ref().unwrap().accepted);
        assert!(
            answers
                .create(v1::prompts::AnswersCreateRequest {
                    parent: prompt_id.clone(),
                    action: "no".into(),
                    request_id: "another-answer".into(),
                    ..Default::default()
                })
                .await
                .is_err()
        );
        assert!(prompts.list(v1::PromptsListRequest { pending_only: true, ..Default::default() }).await?.into_inner().prompts.is_empty());

        // Repeated Questions Keep Distinct Identities and Durable Answers.
        let mut cycles = v1::cycles_client::CyclesClient::new(channel.clone());
        let admitted = cycles
            .create(v1::CyclesCreateRequest {
                origins: vec![repeated_origin],
                request_id: "repeat-cycle".into(),
                ..Default::default()
            })
            .await?
            .into_inner();
        let parent = admitted.cycle.unwrap().cycle_id;
        let mut executions = v1::executions_client::ExecutionsClient::new(channel.clone());
        let repeated = executions.list(v1::ExecutionsListRequest { parent, ..Default::default() }).await?.into_inner().executions[0]
            .summary
            .as_ref()
            .unwrap()
            .execution_id
            .clone();
        executions
            .start(v1::ExecutionsStartRequest {
                name: repeated.clone(),
                request_id: "interactive".into(),
                mode: v1::ExecutionMode::Interactive as i32,
            })
            .await?;
        let mut runtime = v1::backend::overview_client::OverviewClient::new(channel.clone())
            .watch(v1::backend::OverviewWatchRequest::default())
            .await?
            .into_inner();
        runtime.message().await?.unwrap();
        let mut commands = v1::executions::commands_client::CommandsClient::new(channel.clone());
        let mut progress = v1::executions::progress_client::ProgressClient::new(channel)
            .watch(v1::executions::ProgressWatchRequest { name: repeated.clone(), at: None })
            .await?
            .into_inner();
        let mut questions = Vec::new();
        for command in ["first-question", "second-question"] {
            commands
                .create(v1::executions::CommandsCreateRequest {
                    parent: repeated.clone(),
                    request_id: command.into(),
                    command_id: command.into(),
                    action: v1::ExecutionCommandAction::RunStep as i32,
                    step_id: Some("hold".into()),
                })
                .await?;
            let question = loop {
                let pending = client.prompts.pending(repeated.clone()).await?;
                if let Some(view) = pending.prompts.first() {
                    break view.prompt.as_ref().unwrap().prompt_id.clone();
                }
                runtime.message().await?.unwrap();
            };
            assert!(!questions.contains(&question));
            client.prompts.respond(question.clone(), "yes", true).await?;
            questions.push(question);
            loop {
                let saved = commands
                    .get(v1::executions::CommandsGetRequest {
                        parent: repeated.clone(),
                        name: command.into(),
                        ..Default::default()
                    })
                    .await?
                    .into_inner()
                    .command
                    .unwrap();
                if saved.phase == v1::ExecutionCommandPhase::Completed as i32 {
                    break;
                }
                runtime.message().await?.unwrap();
            }
        }
        for question in questions {
            assert!(client.prompts.get(question).await?.prompt.unwrap().response.unwrap().accepted);
        }
        executions
            .stop(v1::ExecutionsStopRequest {
                name: repeated,
                request_id: "stop".into(),
            })
            .await?;
        while progress.message().await?.is_some() {}
        drop(runtime);
        process.close().await?;
        let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let mut prompts = v1::prompts_client::PromptsClient::connect(process.url.clone()).await?;
        let reopened = prompts.get(v1::PromptsGetRequest { name: prompt_id, ..Default::default() }).await?.into_inner().prompt.unwrap();
        assert_eq!(reopened, answered);
        process.close().await?;
        plugin.close().await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    })
    .await?
}
