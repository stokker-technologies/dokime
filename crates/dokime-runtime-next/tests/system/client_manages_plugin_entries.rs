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
async fn client_manages_plugin_entries() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        use dokime::v1;
        
        // Add a Remote Plugin and Read Its Complete Entry
        let root = tempfile::tempdir()?;
        let mut plugin = dokime_runtime_next::PluginProcess::start(&super::support::thermal_fixture()?, true, &super::support::settings().placement).await?;
        let process = super::client_runs_thermal_suite_over_grpc::HostProcess::start(root.path()).await?;
        let mut plugins = v1::plugins_client::PluginsClient::connect(process.url.clone()).await?;
        let entry = plugins.add(v1::AddRemotePlugin { url: plugin.endpoint().into() }).await?.into_inner();
        assert_eq!(entry.state, v1::PluginState::Ready as i32);
        assert!(matches!(entry.placement, Some(v1::plugin::Placement::Remote(_))));
        assert_eq!(plugins.list(v1::ListPlugins {}).await?.into_inner().plugins, vec![entry.clone()]);
        let admitted = plugins.list_admitted_plugins(v1::PluginsListAdmittedPluginsRequest { page_size: 1, after: None }).await?.into_inner();
        assert_eq!(admitted.total_count, 1);
        assert_eq!(admitted.plugins[0].plugin.as_ref(), Some(&entry));
        assert!(plugins.add(v1::AddRemotePlugin { url: "not a URL".into() }).await.is_err());
        process.close().await?;

        // Retain Placement While Unavailable and Explicitly Reconnect
        let process = super::client_runs_thermal_suite_over_grpc::HostProcess::start(root.path()).await?;
        let mut plugins = v1::plugins_client::PluginsClient::connect(process.url.clone()).await?;
        let saved = plugins.list(v1::ListPlugins {}).await?.into_inner().plugins.remove(0);
        assert_eq!(saved.entry, entry.entry);
        assert_eq!(saved.placement, entry.placement);
        assert_eq!(saved.state, v1::PluginState::Unavailable as i32);
        plugins.reload(v1::PluginsReloadRequest { plugin_id: Some(entry.entry.clone()), install_id: None }).await?;
        assert_eq!(plugins.list(v1::ListPlugins {}).await?.into_inner().plugins[0].state, v1::PluginState::Ready as i32);
        assert!(plugins.list_admitted_plugins(v1::PluginsListAdmittedPluginsRequest { page_size: 1, after: Some(v1::AdmittedPluginCursor { epoch: admitted.epoch, revision: admitted.revision, after_plugin_id: entry.entry.clone() }) }).await.is_err());

        // Removing a Remote Entry Leaves Its User-Owned Process Running
        plugins.remove(v1::RemovePlugin { entry: entry.entry.clone() }).await?;
        assert!(plugins.list(v1::ListPlugins {}).await?.into_inner().plugins.is_empty());
        let readded = plugins.add(v1::AddRemotePlugin { url: plugin.endpoint().into() }).await?.into_inner();
        assert_eq!(readded.entry, entry.entry);
        plugins.remove(v1::RemovePlugin { entry: entry.entry }).await?;
        process.close().await?;
        plugin.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    }).await?
}
