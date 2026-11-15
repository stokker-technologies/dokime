/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Manages Plugin Entries

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use super::client_runs_thermal_suite_over_grpc::HostProcess;
use super::support;
use dokime::Client;
use dokime::client::PluginsListOptions;
use dokime_runtime::PluginProcess;
use harmos_protocol::host::plugin_placement::Kind as PlacementKind;
use harmos_protocol::host::{PluginPlacement, PluginRemote, PluginState, PluginValue};
use std::error::Error;
use tonic::transport::Endpoint;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */

#[test]
fn client_manages_plugin_entries() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(support::SYSTEM_HANG_GUARD, async {
            // Create a Remote Entry Through the Composed Library Resource
            let root = support::tempdir()?;
            let credential_root = support::tempdir()?;
            let credentials = support::CredentialStore::start(credential_root.path()).await?;
            let mut plugin = PluginProcess::start(&support::thermal_fixture()?, true, &support::settings().placement).await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, support::client_settings())?;
            let placement = PluginPlacement {
                kind: Some(PlacementKind::Remote(PluginRemote { url: plugin.endpoint().into() })),
            };
            let entry = client.plugins.create("thermal", placement.clone(), PluginValue { enabled: true }).await?.entry.unwrap();
            let entry = super::support::plugin_state(&client.plugins, entry, harmos_protocol::host::PluginState::Ready).await?;
            assert_eq!(entry.name, "thermal");
            assert_eq!(entry.state, PluginState::Ready as i32);
            assert_eq!(entry.placement, Some(placement.clone()));
            let admitted = client.plugins.list().await?;
            assert_eq!(admitted.entries, vec![entry.clone()]);
            let invalid = PluginPlacement {
                kind: Some(PlacementKind::Remote(PluginRemote { url: "not a URL".into() })),
            };
            assert!(client.plugins.create("invalid", invalid, PluginValue { enabled: true }).await.is_err());
            process.close().await?;

            // Retain the Same Entry Across Restart and Fence Old Read Cuts
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, support::client_settings())?;
            let saved = client.plugins.get("thermal").await?.entry.unwrap();
            assert_eq!(saved.name, entry.name);
            assert_eq!(saved.placement, entry.placement);
            let restarted = client.plugins.restart(saved.name.clone(), saved.revision.clone()).await?;
            let restarted = support::plugin_state(&client.plugins, restarted, PluginState::Ready).await?;
            assert_eq!(restarted.state, PluginState::Ready as i32);
            let options = PluginsListOptions {
                at: admitted.resolved_at,
                ..Default::default()
            };
            assert!(client.plugins.list_with(options).await.is_err());

            // Deletion Leaves the User-Owned Remote Process Available for Reattachment
            client.plugins.delete(restarted.name, restarted.revision).await?;
            assert!(client.plugins.list().await?.entries.is_empty());
            let readded = client.plugins.create("thermal", placement, PluginValue { enabled: true }).await?.entry.unwrap();
            let readded = super::support::plugin_state(&client.plugins, readded, harmos_protocol::host::PluginState::Ready).await?;
            assert_eq!(readded.name, entry.name);
            assert_eq!(readded.state, PluginState::Ready as i32);
            plugin.close().await?;
            let unavailable = client.plugins.get("thermal").await?.entry.unwrap();
            assert_eq!(unavailable.state, PluginState::Unavailable as i32);
            assert_eq!(unavailable.description, readded.description);
            client.plugins.delete(unavailable.name, unavailable.revision).await?;
            process.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}
