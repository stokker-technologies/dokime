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
        use dokime::v1;

        // Open the Backend Health Page on a Fresh Host
        let root = tempfile::tempdir()?;
        let process = super::client_runs_thermal_suite_over_grpc::HostProcess::start(root.path()).await?;
        let mut health = v1::health_client::HealthClient::connect(process.url.clone()).await?;
        let empty = health.list_health(v1::HealthListHealthRequest::default()).await?.into_inner();
        assert!(empty.items.is_empty());
        assert!(empty.cursor.is_some());

        // Retain an Unavailable Entry and Show an Actionable Health Item
        let mut plugins = v1::plugins_client::PluginsClient::connect(process.url.clone()).await?;
        let unavailable = plugins.add(v1::AddRemotePlugin { url: "http://127.0.0.1:0".into() }).await?.into_inner();
        assert_eq!(unavailable.state, v1::PluginState::Unavailable as i32);
        let page = health.list_health(v1::HealthListHealthRequest::default()).await?.into_inner();
        assert_eq!(page.items[0].health_item_id, unavailable.entry);
        assert!(!page.items[0].message.is_empty());
        assert!(
            health
                .list_health(v1::HealthListHealthRequest {
                    cursor: empty.cursor,
                    ..Default::default()
                })
                .await
                .is_err()
        );
        plugins.remove(v1::RemovePlugin { entry: unavailable.entry }).await?;
        assert!(health.list_health(v1::HealthListHealthRequest::default()).await?.into_inner().items.is_empty());
        process.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
