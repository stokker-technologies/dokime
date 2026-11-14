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
use dokime::client::PluginsCreateOptions;
use dokime::client::plugin_placement::Kind;
use dokime::client::{PluginPlacement, PluginRemote, PluginValue};

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

pub(crate) async fn run(args: Args, endpoint: Option<String>, settings: crate::settings::Settings) -> crate::Result<()> {
    let description = dokime_runtime::describe_plugin(&args.url, &settings.placement()).await?;
    let client = settings.sdk(endpoint).await?;
    let placement = PluginPlacement {
        kind: Some(Kind::Remote(PluginRemote { url: args.url })),
    };
    let value = client
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
    crate::output::print(&value, args.json)
}

/* === Private Functions === */

/* === Tests === */
