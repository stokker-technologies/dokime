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
use std::error::Error;

// Third Party Imports
use harmos_protocol::host::plugin_placement;
use tonic::transport::Endpoint;

// Local Imports
use dokime_client_internal::Client;
use dokime_client_internal::PluginPlacement;
use dokime_client_internal::PluginRemote;
use dokime_client_internal::PluginValue;
use dokime_client_internal::SuitesListOptions;
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
fn client_browses_catalog() -> Result<(), Box<dyn Error>> {
    super::support::run(Box::pin(async {
        tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
            // Start a Real Host and Admit the Shared Plugin
            let root = super::support::tempdir()?;
            let credential_root = super::support::tempdir()?;
            let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
            let mut plugin = PluginProcess::start(&super::support::thermal_fixture()?, true, &super::support::settings().placement).await?;
            let process =
                super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &credentials.environment)
                    .await?;
            let plugins =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .plugins
                    .clone();

            let entry = plugins
                .create(
                    "thermal",
                    PluginPlacement {
                        kind: Some(plugin_placement::Kind::Remote(PluginRemote { url: plugin.endpoint().into() })),
                    },
                    PluginValue { enabled: true },
                )
                .await?
                .entry
                .unwrap();
            let entry = super::support::plugin_state(&plugins, entry, harmos_protocol::host::PluginState::Ready).await?;
            assert_eq!(entry.name, "thermal");

            // Read Suites, Templates, Record Contracts and Stream Rows
            let suites =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .suites
                    .clone()
                    .list(String::new())
                    .await?;
            let suite =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .suites
                    .clone()
                    .get(suites.values[0].metadata.as_ref().unwrap().identifier.clone(), String::new(), Default::default())
                    .await?;
            assert!(!suite.value.unwrap().templates.is_empty());
            let records =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .record_schemas
                    .clone()
                    .list(String::new())
                    .await?;
            let record =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .record_schemas
                    .clone()
                    .get(records.values[0].record_type.clone(), String::new(), Default::default())
                    .await?;
            assert!(!record.value.unwrap().fields.is_empty());
            let streams =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .stream_schemas
                    .clone()
                    .list(String::new())
                    .await?;
            let stream =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .stream_schemas
                    .clone()
                    .get(streams.values[0].stream.clone(), String::new(), Default::default())
                    .await?;
            assert!(!stream.value.unwrap().columns.is_empty());
            assert!(
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .suites
                    .clone()
                    .get("missing", String::new(), Default::default())
                    .await
                    .is_err()
            );

            // The Complete Catalog Remains Readable After Restart
            process.close().await?;
            let process =
                super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &credentials.environment)
                    .await?;

            let plugins =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .plugins
                    .clone();
            // Read the Retained Catalog Before Reconnecting Its Provider
            let retained =
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .suites
                    .list_with(
                        String::new(),
                        SuitesListOptions {
                            at: suites.resolved_at.clone(),
                            ..Default::default()
                        },
                    )
                    .await?;
            assert_eq!(retained.values, suites.values);
            assert_eq!(retained.resolved_at, suites.resolved_at);
            let restarted = plugins.restart("thermal", plugins.get("thermal").await?.entry.unwrap().revision).await?;
            super::support::plugin_state(&plugins, restarted, harmos_protocol::host::PluginState::Ready).await?;
            assert_eq!(
                Client::from_channel(Endpoint::from_shared(process.url.clone())?.connect().await?, super::support::client_settings())?
                    .suites
                    .clone()
                    .list(String::new())
                    .await?
                    .values,
                suites.values
            );
            process.close().await?;
            plugin.close().await?;
            Ok::<(), Box<dyn Error>>(())
        })
        .await?
    }))
}
