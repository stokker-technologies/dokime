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
        use dokime::v1;

        // Start a Real Host and Admit the Shared Plugin
        let root = tempfile::tempdir()?;
        let credential_root = tempfile::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let mut plugin =
            dokime_runtime_next::PluginProcess::start(&super::support::thermal_fixture()?, true, &super::support::settings().placement)
                .await?;
        let process =
            super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let mut plugins = v1::plugins_client::PluginsClient::connect(process.url.clone()).await?;
        let mut catalog = v1::catalog_client::CatalogClient::connect(process.url.clone()).await?;
        let entry = plugins.add(v1::AddRemotePlugin { url: plugin.endpoint().into() }).await?.into_inner();

        // Read Suites, Templates, Record Contracts and Stream Rows
        let suites = catalog.list_suites(v1::CatalogListSuitesRequest {}).await?.into_inner();
        let suite = catalog
            .get_suite(v1::CatalogGetSuiteRequest {
                suite_id: suites.suites[0].suite_id.clone(),
            })
            .await?
            .into_inner();
        assert!(!suite.templates.is_empty());
        let records = catalog.list_record_schemas(v1::CatalogListRecordSchemasRequest {}).await?.into_inner();
        let record = catalog
            .get_record_schema(v1::CatalogGetRecordSchemaRequest {
                record_type: records.schemas[0].record_type.clone(),
            })
            .await?
            .into_inner();
        assert!(!record.fields.is_empty());
        let streams = catalog.list_telemetry_schemas(v1::CatalogListTelemetrySchemasRequest {}).await?.into_inner();
        let stream = catalog
            .get_telemetry_schema(v1::CatalogGetTelemetrySchemaRequest {
                stream: streams.schemas[0].stream.clone(),
            })
            .await?
            .into_inner();
        assert!(!stream.columns.is_empty());
        assert!(catalog.get_suite(v1::CatalogGetSuiteRequest { suite_id: "missing".into() }).await.is_err());

        // The Complete Catalog Remains Readable After Restart
        process.close().await?;
        let process =
            super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let mut catalog = v1::catalog_client::CatalogClient::connect(process.url.clone()).await?;
        let mut plugins = v1::plugins_client::PluginsClient::connect(process.url.clone()).await?;
        plugins
            .reload(v1::PluginsReloadRequest {
                plugin_id: entry.description.map(|description| description.id),
                ..Default::default()
            })
            .await?;
        assert_eq!(catalog.list_suites(v1::CatalogListSuitesRequest {}).await?.into_inner(), suites);
        process.close().await?;
        plugin.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
