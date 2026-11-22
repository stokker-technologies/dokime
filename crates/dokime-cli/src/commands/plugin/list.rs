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
use std::iter;

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
        println!("{}\t{}", summary(entry), placement(entry));
    }
    Ok(())
}

/* === Private Functions === */

pub(super) fn summary(entry: &PluginEntry) -> String {
    let version = entry.description.as_ref().map(|description| description.version.as_str()).unwrap_or("?");
    let mut summary = format!("{}\t{version}\t{}", entry.name, status(entry));
    if entry.state != PluginState::Ready as i32 || entry.failure.is_some() {
        let failure = iter::successors(entry.failure.as_ref(), |failure| failure.cause.as_deref()).last();
        let action = failure.map(|failure| if failure.action.is_empty() { failure.message.as_str() } else { failure.action.as_str() });
        summary.push('\t');
        let action = action.filter(|action| !action.is_empty()).unwrap_or("Enable or Restart the Plugin and Check Its Status");
        summary.push_str(&action.split_whitespace().collect::<Vec<_>>().join(" "));
    }
    summary
}

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

#[cfg(test)]
mod tests {
    use super::summary;
    use dokime_client_internal::host::{PluginDescription, PluginEntry, PluginState, PluginValue};

    #[test]
    fn plugin_summary_keeps_identity_state_and_recovery_without_declarations() {
        let mut entry = PluginEntry {
            name: "fixture".into(),
            description: Some(PluginDescription {
                version: "1.2.3".into(),
                ..Default::default()
            }),
            state: PluginState::Ready as i32,
            value: Some(PluginValue { enabled: true }),
            ..Default::default()
        };
        let line = summary(&entry);
        let fields = line.split('\t').collect::<Vec<_>>();
        assert_eq!(&fields[..2], [entry.name.as_str(), "1.2.3"]);
        assert_eq!(fields.len(), 3);
        assert_eq!(line.lines().count(), 1);

        entry.state = PluginState::Unavailable as i32;
        let line = summary(&entry);
        assert_eq!(line.split('\t').count(), 4);
        assert_eq!(line.lines().count(), 1);
    }
}
