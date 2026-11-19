/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read an Operator Prompt

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_client_internal::PromptsGetOptions;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    prompt: String,

    #[arg(long)]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let client = settings.sdk(endpoint).await?;
    let value = client
        .prompts
        .get_with(
            args.prompt,
            PromptsGetOptions {
                context: settings.context()?,
                ..Default::default()
            },
        )
        .await?;
    crate::output::print(&value, args.json)
}

/* === Private Functions === */

/* === Tests === */
