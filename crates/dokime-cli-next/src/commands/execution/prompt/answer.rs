/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Answer a Pending Operator Prompt

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    execution: String,

    #[arg(long)]
    prompt: Option<String>,

    #[arg(long)]
    action: String,

    #[arg(long)]
    accepted: bool,

    #[arg(long)]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let channel = settings.channel(endpoint, false).await?;
    let client = dokime::client::HostClient::new(channel, settings.client())?;
    let identifier = match args.prompt {
        Some(identifier) => identifier,
        None => {
            let pending = client.prompts.pending(args.execution).await?;
            if pending.prompts.len() != 1 {
                return Err("Answer Prompt: Supply --prompt When There Is Not Exactly One Pending Prompt".into());
            }
            pending.prompts[0].prompt_id.clone()
        }
    };
    let value = client.prompts.respond(identifier, args.action, args.accepted).await?;
    crate::output::print(&value, args.json)
}

/* === Private Functions === */

/* === Tests === */
