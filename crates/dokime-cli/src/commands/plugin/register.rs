/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Register a Remote gRPC Plugin

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::{Result, output, settings::Settings};
use dokime_client_internal::PluginsCreateOptions;
use dokime_client_internal::host::plugin_placement::Kind;
use dokime_client_internal::host::{PluginPlacement, PluginRemote, PluginValue};

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
    let entry = value.entry.take().ok_or("Register Plugin: Missing Registered Entry")?;
    value.entry = Some(super::await_ready(&client, entry, settings, "Register Plugin").await?);
    if let Some(json) = output {
        if json {
            output::print(&value, true)?;
        } else {
            println!("{}", super::list::summary(value.entry.as_ref().unwrap()));
        }
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */
