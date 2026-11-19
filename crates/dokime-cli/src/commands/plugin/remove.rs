/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Remove an Entry Through the SDK Service

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_client_internal::{PluginsDeleteOptions, PluginsGetOptions};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    entry: String,

    #[arg(long)]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let client = settings.sdk(endpoint).await?;
    let entry = client
        .plugins
        .get_with(
            &args.entry,
            PluginsGetOptions {
                context: settings.context()?,
                ..Default::default()
            },
        )
        .await?
        .entry
        .ok_or("Remove Plugin: Missing Entry")?;
    let value = client.plugins.delete_with(entry.name, entry.revision, PluginsDeleteOptions { context: settings.context()? }).await?;
    crate::output::print(&value, args.json)
}

/* === Private Functions === */

/* === Tests === */
