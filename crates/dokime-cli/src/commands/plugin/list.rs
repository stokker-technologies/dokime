/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Inspect Host-Owned State Through the SDK

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_client_internal::PluginsListOptions;
use dokime_client_internal::host::plugin_placement::Kind;
use dokime_client_internal::host::{PluginEntry, PluginState};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(clap::Args)]
pub(crate) struct Args {
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
        .plugins
        .list_with(PluginsListOptions {
            context: settings.context()?,
            ..Default::default()
        })
        .await?;
    if args.json {
        return crate::output::print(&value, true);
    }
    for entry in &value.entries {
        println!("{}\t{}\t{}", entry.name, placement(entry), status(entry));
    }
    Ok(())
}

/* === Private Functions === */

/// Managed Entries Are Started by the Host; Unmanaged Ones by Their Author.
fn placement(entry: &PluginEntry) -> &'static str {
    match entry.placement.as_ref().and_then(|placement| placement.kind.as_ref()) {
        Some(Kind::Remote(_)) => "Unmanaged",
        Some(Kind::Managed(_) | Kind::Guest(_)) | None => "Managed",
    }
}

fn status(entry: &PluginEntry) -> &'static str {
    if entry.failure.is_some() {
        return "Failed";
    }
    if !entry.value.as_ref().is_some_and(|value| value.enabled) {
        return "Disabled";
    }
    match PluginState::try_from(entry.state) {
        Ok(PluginState::Ready) => "Ready",
        Ok(PluginState::Connecting) => "Starting",
        Ok(PluginState::Offline) => "Offline",
        Ok(PluginState::Stopping) => "Stopping",
        Ok(PluginState::Stopped) => "Stopped",
        Ok(PluginState::Unavailable | PluginState::Unspecified) | Err(_) => "Failed",
    }
}

/* === Tests === */
