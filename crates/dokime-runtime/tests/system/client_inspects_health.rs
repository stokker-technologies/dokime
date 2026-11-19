/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Inspects Health

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime_client_internal::Client;
use dokime_client_internal::EvidenceHealthEventsListOptions;
use dokime_client_internal::PluginPlacement;
use dokime_client_internal::PluginRemote;
use dokime_client_internal::PluginState;
use dokime_client_internal::PluginValue;
use harmos_protocol::host::plugin_placement;
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
fn client_inspects_health() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Open the Backend Health Page on a Fresh Host
            let root = super::support::tempdir()?;
            let credential_root = super::support::tempdir()?;
            let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
            let process =
                super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &credentials.environment)
                    .await?;

            let empty =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .evidence
                    .health_events
                    .clone()
                    .list(Default::default())
                    .await?;
            assert!(empty.events.is_empty());
            assert!(empty.resolved_at.is_some());

            // Retain an Unavailable Entry and Show an Actionable Health Item
            let plugins =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .plugins
                    .clone();
            let unavailable = plugins
                .create(
                    "unavailable",
                    PluginPlacement {
                        kind: Some(plugin_placement::Kind::Remote(PluginRemote { url: "http://127.0.0.1:0".into() })),
                    },
                    PluginValue { enabled: true },
                )
                .await?
                .entry
                .unwrap();
            let unavailable = super::support::plugin_state(&plugins, unavailable, PluginState::Unavailable).await?;
            assert_eq!(unavailable.state, PluginState::Unavailable as i32);
            let page =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .evidence
                    .health_events
                    .clone()
                    .list(Default::default())
                    .await?;
            assert_eq!(page.events[0].health_item_id, unavailable.name);
            assert!(!page.events[0].message.is_empty());
            let retained =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .evidence
                    .health_events
                    .clone()
                    .list_with(
                        Default::default(),
                        EvidenceHealthEventsListOptions {
                            at: empty.resolved_at,
                            ..Default::default()
                        },
                    )
                    .await?;
            assert!(retained.events.is_empty());
            Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                .plugins
                .clone()
                .delete(unavailable.name.clone(), unavailable.revision.clone())
                .await?;
            assert!(
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .evidence
                    .health_events
                    .clone()
                    .list(Default::default())
                    .await?
                    .events
                    .is_empty()
            );
            process.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}
