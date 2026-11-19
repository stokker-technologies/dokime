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

// Standard Library Imports
use std::error::Error;

// Third Party Imports
use tonic::transport::Endpoint;

// Local Imports
use super::client_runs_thermal_suite_over_grpc::HostProcess;
use super::support;
use dokime_client_internal::Client;
use dokime_client_internal::PluginsListOptions;
use dokime_protocol::host::plugin_placement;
use dokime_protocol::host::{PluginPlacement, PluginRemote, PluginState, PluginValue};
use dokime_runtime::PluginProcess;

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
            // Create a Remote Entry Through Product Policy
            let root = support::tempdir()?;
            let credential_root = support::tempdir()?;
            let credentials = support::CredentialStore::start(credential_root.path()).await?;
            let mut plugin = PluginProcess::start(&support::thermal_fixture()?, true, &support::settings().placement).await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, support::client_settings())?;
            let placement = PluginPlacement {
                kind: Some(plugin_placement::Kind::Remote(PluginRemote { url: plugin.endpoint().into() })),
            };
            let entry = client.plugins.create("thermal", placement.clone(), PluginValue { enabled: true }).await?.entry.unwrap();
            let entry = super::support::plugin_state(&client.plugins, entry, dokime_protocol::host::PluginState::Ready).await?;
            assert_eq!(entry.name, "thermal");
            assert_eq!(entry.state, PluginState::Ready as i32);
            assert_eq!(entry.placement, Some(placement.clone()));
            let admitted = client.plugins.list().await?;
            assert_eq!(admitted.entries, vec![entry.clone()]);
            let invalid = PluginPlacement {
                kind: Some(plugin_placement::Kind::Remote(PluginRemote { url: "not a URL".into() })),
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
            let readded = super::support::plugin_state(&client.plugins, readded, dokime_protocol::host::PluginState::Ready).await?;
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

#[test]
fn product_plugin_policy_preserves_origin_cuts_and_replay() -> Result<(), Box<dyn Error>> {
    support::run(Box::pin(async {
        // Create a Disabled Entry Without Owning a Remote Process.
        let root = support::tempdir()?;
        let server =
            dokime_runtime::services::serve("127.0.0.1:0".parse()?, root.path(), support::settings(), support::service_settings()).await?;
        let channel = Endpoint::from_shared(format!("http://{}", server.endpoint()))?.connect().await?;
        let client = Client::from_channel(channel, support::client_settings())?;
        let placement = PluginPlacement {
            kind: Some(plugin_placement::Kind::Remote(PluginRemote { url: "http://127.0.0.1:1".into() })),
        };
        let options = dokime_client_internal::PluginsCreateOptions {
            origin: Some("fixture".into()),
            request_id: Some("origin-round-trip".into()),
            ..Default::default()
        };
        let created = client.plugins.create_with("fixture", placement.clone(), PluginValue { enabled: false }, options.clone()).await?;
        let entry = created.entry.as_ref().unwrap();
        assert_eq!(entry.origin, "fixture");
        assert_eq!(entry.placement, Some(placement.clone()));
        assert_eq!(client.plugins.get("fixture").await?.entry.as_ref(), Some(entry));
        let listed = client.plugins.list().await?;
        assert_eq!(listed.entries, vec![entry.clone()]);
        let replayed = client.plugins.create_with("fixture", placement.clone(), PluginValue { enabled: false }, options.clone()).await?;
        assert_eq!(replayed, created);
        let conflict = dokime_client_internal::PluginsCreateOptions {
            origin: Some("changed".into()),
            ..options
        };
        assert!(client.plugins.create_with("fixture", placement, PluginValue { enabled: false }, conflict).await.is_err());

        // The Product Watch Returns the Same Entry and Retained Read Boundary.
        let mut watch = client
            .plugins
            .watch_with(dokime_client_internal::PluginsWatchOptions {
                name: Some("fixture".into()),
                at: listed.resolved_at.clone(),
                ..Default::default()
            })
            .await?;
        let opening = watch.next().await?.ok_or("Watch Plugin: Return Its Opening Snapshot")?;
        let Some(dokime_protocol::host::plugins_watch_response::Event::Snapshot(snapshot)) = opening.event else {
            return Err("Watch Plugin: Return Its Opening Snapshot".into());
        };
        assert_eq!(snapshot.entries, vec![entry.clone()]);
        assert_eq!(opening.resolved_at, listed.resolved_at);
        drop(watch);

        // Settings Admission and Conditional Mutations Share the Native Policy Owner.
        assert!(
            client
                .plugins
                .settings
                .list_with(dokime_client_internal::PluginsSettingsListOptions {
                    name: Some("fixture".into()),
                    ..Default::default()
                })
                .await
                .is_err()
        );
        assert!(client.plugins.delete("fixture", vec![0]).await.is_err());
        let updated = client.plugins.set("fixture", PluginValue { enabled: false }, entry.revision.clone()).await?.entry.unwrap();
        assert_eq!(updated.origin, entry.origin);
        client.plugins.delete("fixture", updated.revision).await?;
        assert!(client.plugins.list().await?.entries.is_empty());
        server.close().await?;
        Ok(())
    }))
}
