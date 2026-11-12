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
        let mut plugin = dokime_runtime_next::PluginProcess::start(&super::support::thermal_fixture()?, true, &super::support::settings().placement).await?;
        let process = super::client_runs_thermal_suite_over_grpc::HostProcess::start(root.path()).await?;
        let mut plugins = v1::plugins_client::PluginsClient::connect(process.url.clone()).await?;
        let mut catalog = v1::catalog_client::CatalogClient::connect(process.url.clone()).await?;
        plugins.add(v1::AddRemotePlugin { url: plugin.endpoint().into() }).await?;

        // Read Suites, Templates, Record Contracts and Stream Rows
        let suites = catalog.list_suites(v1::CatalogListSuitesRequest { limit: 1, ..Default::default() }).await?.into_inner();
        assert!(suites.cursor.is_some());
        let suite = catalog.get_suite(v1::CatalogGetSuiteRequest { suite_id: suites.suites[0].suite_id.clone() }).await?.into_inner();
        assert!(!suite.templates.is_empty());
        let records = catalog.list_record_schemas(v1::CatalogListRecordSchemasRequest { limit: 1, ..Default::default() }).await?.into_inner();
        let record = catalog.get_record_schema(v1::CatalogGetRecordSchemaRequest { record_type: records.schemas[0].record_type.clone() }).await?.into_inner();
        assert!(!record.fields.is_empty());
        let streams = catalog.list_telemetry_schemas(v1::CatalogListTelemetrySchemasRequest { limit: 1, ..Default::default() }).await?.into_inner();
        let stream = catalog.get_telemetry_schema(v1::CatalogGetTelemetrySchemaRequest { stream: streams.schemas[0].stream.clone() }).await?.into_inner();
        assert!(!stream.columns.is_empty());
        assert!(catalog.get_suite(v1::CatalogGetSuiteRequest { suite_id: "missing".into() }).await.is_err());

        // A Cursor from Another Boot Cannot Continue This Catalog
        let old_cursor = suites.cursor;
        process.close().await?;
        let process = super::client_runs_thermal_suite_over_grpc::HostProcess::start(root.path()).await?;
        let mut catalog = v1::catalog_client::CatalogClient::connect(process.url.clone()).await?;
        assert!(catalog.list_suites(v1::CatalogListSuitesRequest { cursor: old_cursor, ..Default::default() }).await.is_err());
        process.close().await?;
        plugin.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    }).await?
}
