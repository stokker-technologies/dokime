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
use dokime_client_internal::PromptsAnswersCreateOptions;
use dokime_client_internal::PromptsListOptions;

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
    let client = settings.sdk(endpoint).await?;
    let identifier = match args.prompt {
        Some(identifier) => identifier,
        None => {
            let pending = client
                .prompts
                .list_with(
                    args.execution,
                    true,
                    PromptsListOptions {
                        context: settings.context()?,
                        ..Default::default()
                    },
                )
                .await?;
            if pending.prompts.len() != 1 {
                return Err("Answer Prompt: Supply --prompt When There Is Not Exactly One Pending Prompt".into());
            }
            pending.prompts[0].prompt.as_ref().ok_or("Answer Prompt: Missing Prompt Identity")?.prompt_id.clone()
        }
    };
    let options = PromptsAnswersCreateOptions {
        context: settings.context()?,
        request_id: Some(settings.request_id(args.request_id)?),
    };
    let value = client.prompts.answers.create_with(identifier, args.action, Default::default(), args.accepted, options).await?;
    crate::output::print(&value, args.json)
}

/* === Private Functions === */

/* === Tests === */
