/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Browses Catalog

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
async fn client_browses_catalog() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Start a Real Host and Admit the Shared Plugin
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let mut plugin =
            dokime_runtime_next::PluginProcess::start(&super::support::thermal_fixture()?, true, &super::support::settings().placement)
                .await?;
        let process =
            super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let mut plugins = dokime_protocol::client::plugins_client::PluginsClient::connect(process.url.clone()).await?;

        let entry = plugins.add(dokime_protocol::client::AddRemotePlugin { url: plugin.endpoint().into() }).await?.into_inner();

        // Read Suites, Templates, Record Contracts and Stream Rows
        let suites = dokime_protocol::client::suites_client::SuitesClient::connect(process.url.clone())
            .await?
            .list(dokime_protocol::client::SuitesListRequest { ..Default::default() })
            .await?
            .into_inner();
        let suite = dokime_protocol::client::suites_client::SuitesClient::connect(process.url.clone())
            .await?
            .get(dokime_protocol::client::SuitesGetRequest {
                name: suites.values[0].metadata.as_ref().unwrap().identifier.clone(),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert!(!suite.value.unwrap().templates.is_empty());
        let records = dokime_protocol::client::record_schemas_client::RecordSchemasClient::connect(process.url.clone())
            .await?
            .list(dokime_protocol::client::RecordSchemasListRequest { ..Default::default() })
            .await?
            .into_inner();
        let record = dokime_protocol::client::record_schemas_client::RecordSchemasClient::connect(process.url.clone())
            .await?
            .get(dokime_protocol::client::RecordSchemasGetRequest {
                name: records.values[0].record_type.clone(),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert!(!record.value.unwrap().fields.is_empty());
        let streams = dokime_protocol::client::stream_schemas_client::StreamSchemasClient::connect(process.url.clone())
            .await?
            .list(dokime_protocol::client::StreamSchemasListRequest { ..Default::default() })
            .await?
            .into_inner();
        let stream = dokime_protocol::client::stream_schemas_client::StreamSchemasClient::connect(process.url.clone())
            .await?
            .get(dokime_protocol::client::StreamSchemasGetRequest {
                name: streams.values[0].stream.clone(),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert!(!stream.value.unwrap().columns.is_empty());
        assert!(
            dokime_protocol::client::suites_client::SuitesClient::connect(process.url.clone())
                .await?
                .get(dokime_protocol::client::SuitesGetRequest {
                    name: "missing".into(),
                    ..Default::default()
                })
                .await
                .is_err()
        );

        // The Complete Catalog Remains Readable After Restart
        process.close().await?;
        let process =
            super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &credentials.environment).await?;

        let mut plugins = dokime_protocol::client::plugins_client::PluginsClient::connect(process.url.clone()).await?;
        plugins
            .reload(dokime_protocol::client::PluginsReloadRequest {
                plugin_id: entry.description.map(|description| description.id),
                ..Default::default()
            })
            .await?;
        assert_eq!(
            dokime_protocol::client::suites_client::SuitesClient::connect(process.url.clone())
                .await?
                .list(dokime_protocol::client::SuitesListRequest { ..Default::default() })
                .await?
                .into_inner()
                .values,
            suites.values
        );
        process.close().await?;
        plugin.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
