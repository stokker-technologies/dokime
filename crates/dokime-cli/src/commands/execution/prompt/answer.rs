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
use dokime_protocol::client;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    #[arg(long, help = "Reuse This Identity When Retrying the Same Mutation")]
    request_id: Option<String>,

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
    let client = settings.sdk(endpoint, false).await?;
    let identifier = match args.prompt {
        Some(identifier) => identifier,
        None => {
            let request = client::PromptsListRequest {
                execution_id: Some(args.execution),
                pending_only: true,
                ..Default::default()
            };
            let pending = client.prompts.list(settings.request(request)).await?.into_inner();
            if pending.prompts.len() != 1 {
                return Err("Answer Prompt: Supply --prompt When There Is Not Exactly One Pending Prompt".into());
            }
            pending.prompts[0].prompt.as_ref().ok_or("Answer Prompt: Missing Prompt Identity")?.prompt_id.clone()
        }
    };
    let request = client::prompts::AnswersCreateRequest {
        parent: identifier,
        action: args.action,
        accepted: args.accepted,
        request_id: settings.request_id(args.request_id)?,
        ..Default::default()
    };
    let value = client.prompts.answers.create(settings.request(request)).await?.into_inner();
    crate::output::print(&value, args.json)
}

/* === Private Functions === */

/* === Tests === */
