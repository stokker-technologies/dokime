/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Await an Operator Prompt Without Polling

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::client::{ExecutionsProgressWatchOptions, PromptView, PromptsListOptions};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    execution: String,

    #[arg(long)]
    json: bool,

    #[command(flatten)]
    id_file: crate::commands::id_file::Args,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let id_file = args.id_file.open()?;
    let client = settings.sdk(endpoint).await?;
    let pending = client
        .prompts
        .list_with(
            &args.execution,
            true,
            PromptsListOptions {
                context: settings.context()?,
                ..Default::default()
            },
        )
        .await?;
    if let Some(prompt) = pending.prompts.first() {
        return settle(prompt, id_file, args.json);
    }
    let mut events = client
        .executions
        .progress
        .watch_with(
            &args.execution,
            ExecutionsProgressWatchOptions {
                context: settings.context()?,
                ..Default::default()
            },
        )
        .await?;
    while let Some(frame) = events.next().await? {
        if frame.phase == "prompt" || frame.finished {
            let pending = client
                .prompts
                .list_with(
                    &args.execution,
                    true,
                    PromptsListOptions {
                        context: settings.context()?,
                        ..Default::default()
                    },
                )
                .await?;
            if let Some(prompt) = pending.prompts.first() {
                return settle(prompt, id_file, args.json);
            }
        }
        if frame.finished {
            return Err("Await Prompt: Execution Finished Without a Pending Prompt".into());
        }
    }
    Err("Await Prompt: Progress Watch Closed Without Settlement".into())
}

/* === Private Functions === */

fn settle(prompt: &PromptView, id_file: Option<crate::commands::id_file::IdFile>, json: bool) -> crate::Result<()> {
    if let Some(file) = id_file {
        file.write(prompt.prompt.iter().map(|instance| &instance.prompt_id))?;
    }
    crate::output::print(prompt, json)
}

/* === Tests === */
