/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Answer An Execution Prompt By Identity

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_domain::model::identity::{PromptId, TestExecutionId};
use dokime_domain::model::values::{Value, wire};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

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

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args) -> crate::Result<()> {
    let execution = TestExecutionId::from_uuid(&args.execution_id).map_err(|error| crate::Error::Input(error.to_string()))?;
    let prompt = PromptId::from_uuid(&args.prompt_id).map_err(|error| crate::Error::Input(error.to_string()))?;
    let action = args.action.trim().to_owned();
    if action.is_empty() || action.len() > 256 {
        return Err(crate::Error::Input("prompt action must contain 1 to 256 bytes".into()));
    }
    let values = answer_values(&args.values_json)?;
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

/* === Private Functions === */

fn answer_values(input: &str) -> crate::Result<wire::Value> {
    if input.len() > 1024 * 1024 {
        return Err(crate::Error::Input("prompt values exceed 1 MiB".into()));
    }
    let json: serde_json::Value = serde_json::from_str(input).map_err(|error| crate::Error::Input(error.to_string()))?;
    if !json.is_object() {
        return Err(crate::Error::Input("prompt values must be a JSON object".into()));
    }
    let value = Value::from_plain_json(json).map_err(|error| crate::Error::Input(error.to_string()))?;
    Ok(wire::Value::from(value))
}

/* === Tests === */
