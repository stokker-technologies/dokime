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

#[tokio::test]
async fn client_inspects_health() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Open the Backend Health Page on a Fresh Host
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let process =
            super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &credentials.environment).await?;

        let empty = dokime_protocol::client::evidence::health_events_client::HealthEventsClient::connect(process.url.clone())
            .await?
            .list(dokime_protocol::client::evidence::HealthEventsListRequest::default())
            .await?
            .into_inner();
        assert!(empty.events.is_empty());
        assert!(empty.resolved_at.is_some());

        // Retain an Unavailable Entry and Show an Actionable Health Item
        let mut plugins = dokime_protocol::client::plugins_client::PluginsClient::connect(process.url.clone()).await?;
        let unavailable = plugins.add(dokime_protocol::client::AddRemotePlugin { url: "http://127.0.0.1:0".into() }).await?.into_inner();
        assert_eq!(unavailable.state, dokime_protocol::client::PluginState::Unavailable as i32);
        let page = dokime_protocol::client::evidence::health_events_client::HealthEventsClient::connect(process.url.clone())
            .await?
            .list(dokime_protocol::client::evidence::HealthEventsListRequest::default())
            .await?
            .into_inner();
        assert_eq!(page.events[0].health_item_id, unavailable.entry);
        assert!(!page.events[0].message.is_empty());
        let retained = dokime_protocol::client::evidence::health_events_client::HealthEventsClient::connect(process.url.clone())
            .await?
            .list(dokime_protocol::client::evidence::HealthEventsListRequest {
                at: empty.resolved_at,
                ..Default::default()
            })
            .await?
            .into_inner();
        assert!(retained.events.is_empty());
        dokime_protocol::client::plugins_client::PluginsClient::connect(process.url.clone())
            .await?
            .remove(dokime_protocol::client::RemovePlugin { entry: unavailable.entry })
            .await?;
        assert!(
            dokime_protocol::client::evidence::health_events_client::HealthEventsClient::connect(process.url.clone())
                .await?
                .list(dokime_protocol::client::evidence::HealthEventsListRequest::default())
                .await?
                .into_inner()
                .events
                .is_empty()
        );
        process.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
