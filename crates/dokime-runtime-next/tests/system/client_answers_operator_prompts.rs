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

use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime::{client::HostClient, v1};
use dokime_runtime_next::PluginProcess;

// Standard Library Imports

// Third Party Imports

// Local Imports


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
        let root = tempfile::tempdir()?;
        let settings = super::support::settings();
        let mut plugin = PluginProcess::start(&super::support::thermal_fixture()?, true, &settings.placement).await?;
        let process = HostProcess::start(root.path()).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let client = HostClient::new(channel.clone());
        let mut prompts = v1::prompts_client::PromptsClient::new(channel.clone());
        client.plugins.add(plugin.endpoint()).await?;

        // Inspect the Registered Prompt Catalog Through Both Call Forms
        let definitions = prompts.list_definitions(v1::PromptsListDefinitionsRequest::default()).await?.into_inner();
        assert_eq!(definitions.definitions.len(), 1);
        let definition = prompts.get_definition(v1::PromptsGetDefinitionRequest { prompt_type: "thermal.door".into() }).await?.into_inner();
        assert_eq!(definition.metadata.as_ref().unwrap().identifier, definitions.definitions[0].prompt_type);
        let mut run = prompts.run_get_definition(tokio_stream::iter([
            v1::PromptsGetDefinitionRunInput { value: Some(v1::prompts_get_definition_run_input::Value::Request(v1::PromptsGetDefinitionRequest { prompt_type: "thermal.door".into() })) },
            v1::PromptsGetDefinitionRunInput { value: Some(v1::prompts_get_definition_run_input::Value::Finish(v1::ClientEmpty {})) },
        ])).await?.into_inner();
        assert_eq!(run.message().await?.unwrap().value, Some(v1::prompts_get_definition_run_output::Value::Response(definition)));
        assert!(run.message().await?.is_none());
        let missing = prompts.get_definition(v1::PromptsGetDefinitionRequest { prompt_type: "missing".into() }).await.unwrap_err();
        assert_eq!(missing.code(), tonic::Code::NotFound);

        // Await a Prompt and Inspect the Same Pending Selection
        let cases = client.templates.materialize("thermal", "thermal.cycle", std::collections::HashMap::from([
            ("limit".into(), vec![v1::Value { kind: Some(v1::value::Kind::Number(85.0)) }]),
        ])).await?;
        let cycle = client.cycles.start(cases).await?;
        let execution = cycle.executions[0].clone();
        let mut watch = cycle.watch();
        while let Some(frame) = watch.next().await? { if frame.phase == "prompt" { break; } }
        let pending = prompts.list_pending(v1::PromptsListPendingRequest { execution_id: Some(execution.clone()), limit: 1, ..Default::default() }).await?.into_inner();
        assert_eq!(pending.prompts.len(), 1);
        let prompt_id = pending.prompts[0].prompt_id.clone();
        let stale = prompts.list_pending(v1::PromptsListPendingRequest { execution_id: Some("another".into()), cursor: pending.cursor.clone(), ..Default::default() }).await.unwrap_err();
        assert_eq!(stale.code(), tonic::Code::FailedPrecondition);

        // Reject an Unknown Action Without Settling the Question
        let invalid = prompts.respond(v1::PromptsRespondRequest { prompt_id: prompt_id.clone(), action: "unknown".into(), accepted: true, ..Default::default() }).await.unwrap_err();
        assert_eq!(invalid.code(), tonic::Code::InvalidArgument);
        assert!(prompts.get_prompt(v1::PromptsGetPromptRequest { prompt_id: prompt_id.clone() }).await?.into_inner().response.is_none());

        // Answer Once and Resolve the Accepted Answer After Settlement
        let receipt = prompts.respond(v1::PromptsRespondRequest { prompt_id: prompt_id.clone(), action: "yes".into(), accepted: true, context: Some(v1::MutationContext { correlation_id: Some("operator-answer".into()) }), ..Default::default() }).await?.into_inner();
        assert_eq!(receipt.prompt_id, prompt_id);
        while watch.next().await?.is_some() {}
        let answered = prompts.get_prompt(v1::PromptsGetPromptRequest { prompt_id: prompt_id.clone() }).await?.into_inner();
        assert_eq!(answered.response.as_ref().unwrap().action, "yes");
        assert_eq!(answered.response.as_ref().unwrap().responder, "local");
        assert!(prompts.respond(v1::PromptsRespondRequest { prompt_id: prompt_id.clone(), action: "no".into(), ..Default::default() }).await.is_err());
        assert!(prompts.list_pending(v1::PromptsListPendingRequest::default()).await?.into_inner().prompts.is_empty());

        // Reopen the Host and Resolve the Frozen Question and Answer
        process.close().await?;
        let process = HostProcess::start(root.path()).await?;
        let channel = tonic::transport::Endpoint::from_shared(process.url.clone())?.connect().await?;
        let reopened = v1::prompts_client::PromptsClient::new(channel).get_prompt(v1::PromptsGetPromptRequest { prompt_id }).await?.into_inner();
        assert_eq!(reopened.prompt, answered.prompt);
        assert_eq!(reopened.response, answered.response);
        process.close().await?;
        plugin.close().await?;
        Ok::<_, Box<dyn std::error::Error>>(())
    }).await?
}
