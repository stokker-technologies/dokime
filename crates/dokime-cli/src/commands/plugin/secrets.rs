/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Manage Declared Plugin Secret Bindings

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::settings::Settings;
use crate::client::PluginsSecretsListOptions;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    #[command(subcommand)]
    command: Command,

    #[arg(long, global = true)]
    json: bool,
}

/* === Enums === */

#[derive(clap::Subcommand)]
enum Command {
    /// List Each Declared Secret Identifier and Whether Secret Storage Holds It
    List { name: String },
}

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: Settings) -> crate::Result<()> {
    let client = settings.sdk(endpoint).await?;
    match args.command {
        Command::List { name } => {
            let value = client
                .plugins
                .secrets
                .list_with(PluginsSecretsListOptions {
                    context: settings.context()?,
                    name: Some(name),
                    ..Default::default()
                })
                .await?;
            crate::output::print(&value, args.json)
        }
    }
}

/* === Private Functions === */

/* === Tests === */
