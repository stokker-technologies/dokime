/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Discover A Pending Prompt For One Execution

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::Duration;

// Third Party Imports

// Local Imports
use dokime_domain::model::identity::TestExecutionId;
use dokime_domain::model::prompts::wire::PromptInstance;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    #[arg(help = "Execution UUID Whose Pending Prompt Is Required.")]
    execution_id: String,

    #[arg(long, value_parser = clap::value_parser!(u64).range(1..=86400), help = "Wait Up To This Many Seconds; Omitted Reads Once.")]
    wait_seconds: Option<u64>,

    #[command(flatten)]
    connection: crate::connection::Args,

    #[command(flatten)]
    recovery: super::super::recovery::Args,

    #[arg(long, help = "Emit The Admitted Pending Prompt As JSON.")]
    json: bool,

    #[arg(long, conflicts_with = "json", help = "Print Only The Pending Prompt UUID For Scripts.")]
    id_only: bool,

    #[command(flatten)]
    id_file: crate::commands::id_file::Args,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args) -> crate::Result<()> {
    let id = TestExecutionId::from_uuid(&args.execution_id).map_err(|error| crate::Error::Input(error.to_string()))?;
    let recovery = args.recovery.prepare(&id)?;
    let id_file = args.id_file.open()?;
    let client = args.connection.connect().await?;
    let execution = client.execution(id);
    let read = async {
        match args.wait_seconds {
            Some(seconds) => tokio::time::timeout(Duration::from_secs(seconds), find(&execution, true)).await.map_err(|_| {
                crate::Error::Setup(format!("No pending prompt observed before the deadline for execution {}.", args.execution_id))
            })?,
            None => find(&execution, false).await,
        }
    };
    let result = super::super::recovery::observe(&client, &execution, recovery.as_ref(), "Prompt", read).await;
    client.close_and_wait().await;
    let prompt = result?;
    if let Some(file) = id_file {
        file.write([&prompt.prompt_id])?;
    }

    if args.id_only {
        println!("{}", prompt.prompt_id);
    } else if args.json {
        println!("{}", serde_json::to_string(&prompt)?);
    } else {
        let title = prompt.definition.as_ref().map_or("", |definition| definition.title.as_str());
        println!("Prompt {} For Execution {}: {title}", prompt.prompt_id, args.execution_id);
    }
    Ok(())
}

/* === Private Functions === */

async fn find(execution: &dokime_client::TestExecutionHandle, wait: bool) -> crate::Result<PromptInstance> {
    loop {
        let pending = execution.pending_prompts().await?;
        let prompt = pending.message().await;
        pending.close().await;
        if let Some(prompt) = prompt? {
            return Ok(prompt);
        }
        if !wait {
            return Err(crate::Error::Setup(format!("Execution {} has no pending prompt.", execution.id())));
        }
        match execution.checkpoint().await {
            Ok(_) => return Err(crate::Error::Setup(format!("Execution {} is sealed and has no pending prompt.", execution.id()))),
            Err(dokime_client::Error::Rpc(status)) if status.code() == dokime_client::RpcCode::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

/* === Tests === */
