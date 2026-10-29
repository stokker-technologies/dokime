/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Answer An Execution Prompt By Identity

use dokime::domain::model::identity::{PromptId, TestExecutionId};
use dokime::domain::model::values::{Value, wire};

const PROMPT_ACTION_BYTES: u64 = 256;
const VALUE_INPUT_BYTES: u64 = 1_048_576;

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[arg(help = "Execution UUID Owning The Prompt.")]
    execution_id: String,

    #[arg(help = "Prompt UUID To Answer Once.")]
    prompt_id: String,

    #[arg(long, help = "Declared Action ID; cancel And abort Submit A Cancelled Response.")]
    action: String,

    #[arg(long, default_value = "{}", help = "Answer Fields As A Plain JSON Object; Empty For A Unit Confirmation.")]
    values_json: String,

    #[command(flatten)]
    connection: crate::connection::Args,

    #[command(flatten)]
    recovery: super::super::recovery::Args,

    #[arg(long, help = "Emit The Committed Prompt Receipt As JSON.")]
    json: bool,
}

pub(crate) async fn run(mut args: Args, settings: &dokime::domain::model::settings::application::ClientPreferences) -> crate::Result<()> {
    args.connection = args.connection.with_settings(*settings);
    let execution = TestExecutionId::from_uuid(&args.execution_id).map_err(|error| crate::Error::Input(error.to_string()))?;
    let prompt = PromptId::from_uuid(&args.prompt_id).map_err(|error| crate::Error::Input(error.to_string()))?;
    let action = args.action.trim().to_owned();
    if action.is_empty() || action.len() as u64 > PROMPT_ACTION_BYTES {
        return Err(crate::Error::Input("prompt action must be nonempty and within the configured byte limit".into()));
    }
    let values = answer_values(&args.values_json, VALUE_INPUT_BYTES as usize)?;
    let accepted = !["cancel", "abort"].contains(&action.as_str());

    let recovery = args.recovery.prepare(&execution)?;
    let client = args.connection.connect().await?;
    let execution = client.execution(execution);
    let operation = async { execution.respond_prompt(&prompt, action, values, accepted).await.map_err(crate::Error::from) };
    let result = super::super::recovery::observe(&client, &execution, recovery.as_ref(), "Prompt Answer", operation).await;
    client.close_and_wait().await;
    let receipt = result?;
    if args.json {
        println!("{}", serde_json::to_string(&receipt)?);
    } else {
        println!("Answered Prompt {} For Execution {}", args.prompt_id, args.execution_id);
    }
    Ok(())
}

fn answer_values(input: &str, maximum: usize) -> crate::Result<wire::Value> {
    if input.len() > maximum {
        return Err(crate::Error::Input("prompt values exceed the configured byte limit".into()));
    }
    let json: serde_json::Value = serde_json::from_str(input).map_err(|error| crate::Error::Input(error.to_string()))?;
    if !json.is_object() {
        return Err(crate::Error::Input("prompt values must be a JSON object".into()));
    }
    let value = Value::from_plain_json(json).map_err(|error| crate::Error::Input(error.to_string()))?;
    wire::Value::try_from(value).map_err(|error| crate::Error::Input(error.to_string()))
}
