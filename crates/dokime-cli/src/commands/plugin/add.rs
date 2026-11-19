/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Read or Update the Host Through the SDK

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, output, settings::Settings};
use crate::client::PluginsCreateOptions;
use crate::client::plugin_placement::Kind;
use crate::client::{PluginPlacement, PluginRemote, PluginValue};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
    url: String,

    #[arg(long)]
    json: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: Settings) -> Result<()> {
    attach(args.url, endpoint, &settings, Some(args.json)).await
}

pub(crate) async fn attach(url: String, endpoint: Option<String>, settings: &Settings, output: Option<bool>) -> Result<()> {
    let description = dokime_runtime::describe_plugin(&url, &settings.placement()).await?;
    let client = settings.sdk(endpoint).await?;
    let placement = PluginPlacement {
        kind: Some(Kind::Remote(PluginRemote { url })),
    };
    let mut value = client
        .plugins
        .create_with(
            description.id,
            placement,
            PluginValue { enabled: true },
            PluginsCreateOptions {
                context: settings.context()?,
                ..Default::default()
            },
        )
        .await?;
    let entry = value.entry.take().ok_or("Add Plugin: Missing Registered Entry")?;
    value.entry = Some(super::await_ready(&client, entry, settings, "Add Plugin").await?);
    if let Some(json) = output {
        output::print(&value, json)?;
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
