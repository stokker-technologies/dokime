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
    let mut client = dokime::v1::prompts_client::PromptsClient::with_interceptor(channel, settings.interceptor());
    let identifier = match args.prompt {
        Some(identifier) => identifier,
        None => {
            let pending = client.list_pending(dokime::v1::PromptsListPendingRequest { execution_id: Some(args.execution), ..Default::default() }).await?.into_inner();
            if pending.prompts.len() != 1 {
                return Err("Answer Prompt: Supply --prompt When There Is Not Exactly One Pending Prompt".into());
            }
            pending.prompts[0].prompt_id.clone()
        }
    };
    let value = client.respond(dokime::v1::PromptsRespondRequest { prompt_id: identifier, action: args.action, accepted: args.accepted, ..Default::default() }).await?.into_inner();
    crate::output::print(&value, args.json)
}

/* === Private Functions === */

/* === Tests === */
