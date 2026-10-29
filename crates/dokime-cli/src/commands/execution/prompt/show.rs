/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Display An Execution Prompt By Identity

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::domain::model::identity::{PromptId, TestExecutionId};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[arg(help = "Execution UUID Owning The Prompt.")]
    execution_id: String,

    #[arg(help = "Prompt UUID Returned By Execution Prompt Next.")]
    prompt_id: String,

    #[command(flatten)]
    connection: crate::connection::Args,

    #[command(flatten)]
    recovery: super::super::recovery::Args,

    #[arg(long, help = "Emit The Admitted Prompt And Any Committed Response As JSON.")]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(mut args: Args, settings: &dokime::domain::model::settings::application::ClientPreferences) -> crate::Result<()> {
    args.connection = args.connection.with_settings(*settings);
    let execution = TestExecutionId::from_uuid(&args.execution_id).map_err(|error| crate::Error::Input(error.to_string()))?;
    let prompt = PromptId::from_uuid(&args.prompt_id).map_err(|error| crate::Error::Input(error.to_string()))?;
    let recovery = args.recovery.prepare(&execution)?;
    let client = args.connection.connect().await?;
    let execution = client.execution(execution);
    let operation = async { execution.prompt(&prompt).await.map_err(crate::Error::from) };
    let result = super::super::recovery::observe(&client, &execution, recovery.as_ref(), "Prompt Display", operation).await;
    client.close_and_wait().await;
    let view = result?;

    if args.json {
        println!("{}", serde_json::to_string(&view)?);
        return Ok(());
    }
    let prompt = view.prompt.as_ref().ok_or_else(|| crate::Error::Setup("prompt view is empty".into()))?;
    let definition = prompt.definition.as_ref().ok_or_else(|| crate::Error::Setup("prompt definition is empty".into()))?;
    println!("Prompt {} For Execution {}", args.prompt_id, args.execution_id);
    println!("{}", definition.title);
    if let Some(message) = &definition.message {
        println!("{message}");
    }
    for action in &definition.actions {
        println!("Action {}: {}", action.id, action.label);
    }
    for field in &definition.fields {
        if let Some(metadata) = &field.metadata {
            println!("Field {}: {}", metadata.identifier, metadata.label);
        }
    }
    if let Some(response) = &view.response {
        println!("Answered: {} (Accepted: {})", response.action, response.accepted);
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
