/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Installs And Removes Managed Plugins

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use super::client_runs_thermal_suite_over_grpc::HostProcess;
use super::support;
use super::support::bundles::{bundle, upload};
use dokime::Client;
use harmos_protocol::host::plugin_placement::Kind as PlacementKind;
use harmos_protocol::host::{PluginManaged, PluginPlacement, PluginState, PluginValue};
use sha2::{Digest, Sha256};
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
fn client_installs_and_removes_managed_plugins() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(support::SYSTEM_HANG_GUARD, async {
            // Upload a Verified Bundle Before Creating the Library Entry
            let executable = std::fs::read(support::thermal_fixture()?)?;
            let first_archive = bundle(&executable, "thermal", "1.0.0")?;
            let root = support::tempdir()?;
            let credential_root = support::tempdir()?;
            let credentials = support::CredentialStore::start(credential_root.path()).await?;
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, support::client_settings())?;
            let content = upload(&client, first_archive.clone()).await?;
            let placement = PluginPlacement {
                kind: Some(PlacementKind::Managed(PluginManaged {
                    content: Some(content),
                    sha256: Sha256::digest(&first_archive).to_vec(),
                })),
            };
            let installed = client.plugins.create("thermal", placement, PluginValue { enabled: true }).await?.entry.unwrap();
            let installed = super::support::plugin_state(&client.plugins, installed, harmos_protocol::host::PluginState::Ready).await?;
            assert_eq!(installed.state, PluginState::Ready as i32);
            assert!(installed.process.is_some());
            assert_eq!(client.plugins.get("thermal").await?.entry, Some(installed.clone()));
            assert_eq!(client.plugins.list().await?.entries, vec![installed.clone()]);

            // Disabled Replacement Keeps Its Identity and Enablement
            let disabled = client.plugins.set("thermal", PluginValue { enabled: false }, installed.revision).await?.entry.unwrap();
            assert!(!disabled.value.as_ref().unwrap().enabled);
            let second_archive = bundle(&executable, "thermal", "2.0.0")?;
            let content = upload(&client, second_archive.clone()).await?;
            let placement = PluginPlacement {
                kind: Some(PlacementKind::Managed(PluginManaged {
                    content: Some(content),
                    sha256: Sha256::digest(&second_archive).to_vec(),
                })),
            };
            let replacement = client.plugins.replace("thermal", placement, disabled.revision).await?.entry.unwrap();
            assert_eq!(replacement.name, disabled.name);
            assert!(!replacement.value.as_ref().unwrap().enabled);
            let enabled = client.plugins.set("thermal", PluginValue { enabled: true }, replacement.revision).await?.entry.unwrap();
            let enabled = support::plugin_state(&client.plugins, enabled, PluginState::Ready).await?;
            assert_eq!(enabled.state, PluginState::Ready as i32);

            // A Bad Bundle Cannot Replace the Serving Entry
            let content = upload(&client, b"invalid bundle".to_vec()).await?;
            let invalid = PluginPlacement {
                kind: Some(PlacementKind::Managed(PluginManaged {
                    content: Some(content),
                    sha256: Sha256::digest(b"invalid bundle").to_vec(),
                })),
            };
            assert!(client.plugins.replace("thermal", invalid, enabled.revision.clone()).await.is_err());
            assert_eq!(client.plugins.get("thermal").await?.entry, Some(enabled));
            process.close().await?;

            // Restart Activates the Durable Managed Entry and Deletion Retires It
            let process = HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
            let channel = Endpoint::from_shared(process.url.clone())?.connect().await?;
            let client = Client::from_channel(channel, support::client_settings())?;
            let restarted = client.plugins.get("thermal").await?.entry.unwrap();
            let restarted = support::plugin_state(&client.plugins, restarted, PluginState::Ready).await?;
            assert_eq!(restarted.state, PluginState::Ready as i32);
            assert!(restarted.process.is_some());
            client.plugins.delete(restarted.name, restarted.revision).await?;
            assert!(client.plugins.list().await?.entries.is_empty());
            process.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}
