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
use dokime_protocol::client;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    execution: String,

    #[arg(long)]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let client = settings.sdk(endpoint, true).await?;
    let request = client::PromptsListRequest {
        execution_id: Some(args.execution.clone()),
        pending_only: true,
        ..Default::default()
    };
    let pending = client.prompts.list(settings.request(request.clone())).await?.into_inner();
    if let Some(prompt) = pending.prompts.first() {
        return crate::output::print(prompt, args.json);
    }
    let progress = client::executions::ProgressWatchRequest { name: args.execution, at: None };
    let mut events = client.executions.progress.watch(settings.request(progress)).await?;
    while let Some(frame) = events.next().await? {
        if frame.phase == "prompt" || frame.finished {
            let pending = client.prompts.list(settings.request(request.clone())).await?.into_inner();
            if let Some(prompt) = pending.prompts.first() {
                return crate::output::print(prompt, args.json);
            }
        }
        if frame.finished {
            return Err("Await Prompt: Execution Finished Without a Pending Prompt".into());
        }
    }
    Err("Await Prompt: Progress Watch Closed Without Settlement".into())
}

/* === Private Functions === */

/* === Tests === */
