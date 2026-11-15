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
use dokime::client::{PluginState, PluginsGetOptions, PluginsWatchOptions};
use tonic::Status;

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
    let mut entry = value.entry.take().ok_or("Add Plugin: Missing Registered Entry")?;
    let mut watch = client
        .plugins
        .watch_with(PluginsWatchOptions {
            name: Some(entry.name.clone()),
            context: settings.context()?,
            ..Default::default()
        })
        .await?;
    while entry.state != PluginState::Ready as i32 {
        if matches!(PluginState::try_from(entry.state), Ok(PluginState::Unavailable | PluginState::Stopped)) {
            return Err(Status::failed_precondition(format!(
                "Add Plugin {}: {}",
                entry.name,
                entry.failure.map_or_else(|| "Activation Stopped".into(), |failure| failure.message)
            ))
            .into());
        }
        watch.next().await?.ok_or("Add Plugin: Plugin Watch Closed Before Readiness")?;
        entry = client
            .plugins
            .get_with(
                entry.name,
                PluginsGetOptions {
                    context: settings.context()?,
                    ..Default::default()
                },
            )
            .await?
            .entry
            .ok_or("Add Plugin: Plugin Entry Was Removed")?;
    }
    drop(watch);
    value.entry = Some(entry);
    crate::output::print(&value, args.json)
}

/* === Private Functions === */

/* === Tests === */
