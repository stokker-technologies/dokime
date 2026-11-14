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
use std::{collections::BTreeMap, fs::File, io::Read, path::PathBuf};

// Third Party Imports

// Local Imports
use crate::error::Error;
use crate::settings::Settings;
use dokime::client::{PluginsSecretBindingValue, PluginsSecretsListOptions};
use dokime::client::{PluginsSecretsDeleteOptions, PluginsSecretsSetOptions};

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
    List {
        name: Option<String>,
    },
    Set {
        name: String,

        #[arg(long, help = "JSON Object Mapping Declared Identifiers to Stored Secret Names")]
        bindings: PathBuf,
    },
    Delete {
        name: String,

        #[arg(required = true)]
        identifiers: Vec<String>,
    },
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
                    name,
                    ..Default::default()
                })
                .await?;
            crate::output::print(&value, args.json)
        }
        Command::Set { name, bindings } => {
            let file = File::open(bindings)?;
            let mut bytes = Vec::new();
            file.take(settings.retained_bytes.get() as u64 + 1).read_to_end(&mut bytes)?;
            if bytes.len() > settings.retained_bytes.get() {
                return Err(Error::Capacity("Set Plugin Secrets: Select a Smaller Binding Map".into()));
            }
            let bindings: BTreeMap<String, String> = serde_json::from_slice(&bytes)?;
            let bindings = bindings.into_iter().map(|(identifier, secret)| PluginsSecretBindingValue { identifier, secret }).collect();
            let current = client
                .plugins
                .secrets
                .list_with(PluginsSecretsListOptions {
                    context: settings.context()?,
                    name: Some(name.clone()),
                    ..Default::default()
                })
                .await?;
            let value = client
                .plugins
                .secrets
                .set_with(
                    name,
                    bindings,
                    current.revision,
                    PluginsSecretsSetOptions {
                        context: settings.context()?,
                        ..Default::default()
                    },
                )
                .await?;
            crate::output::print(&value, args.json)
        }
        Command::Delete { name, identifiers } => {
            let current = client
                .plugins
                .secrets
                .list_with(PluginsSecretsListOptions {
                    context: settings.context()?,
                    name: Some(name.clone()),
                    ..Default::default()
                })
                .await?;
            let value = client
                .plugins
                .secrets
                .delete_with(
                    name,
                    identifiers,
                    current.revision,
                    PluginsSecretsDeleteOptions {
                        context: settings.context()?,
                        ..Default::default()
                    },
                )
                .await?;
            crate::output::print(&value, args.json)
        }
    }
}

/* === Private Functions === */

/* === Tests === */
