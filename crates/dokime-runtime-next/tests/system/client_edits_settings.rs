/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client Edits Settings

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
async fn client_edits_settings() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        use dokime::v1;

        // Open Settings and Save a Revision-Checked Preference
        let root = tempfile::tempdir()?;
        // An Unavailable Plugin Retains Editable Settings Without Startup Injection
        let catalog = dokime::Plugin::new("settings.fixture", "0.1.0")
            .setting(v1::Setting {
                metadata: Some(v1::Metadata {
                    identifier: "enabled".into(),
                    label: "Enabled".into(),
                    ..Default::default()
                }),
                kind: Some(v1::setting::Kind::Boolean(v1::BooleanSetting { default_value: true })),
            })
            .catalog()?;
        let store = dokime_storage::Store::open(root.path())?;
        let description = harmos_protocol::plugin::InstanceGetResponse {
            id: "settings.fixture".into(),
            catalog: Some(catalog),
            ..Default::default()
        };
        store.save_plugin("settings.fixture", &prost::Message::encode_to_vec(&description))?;
        drop(store);
        let process = super::client_runs_thermal_suite_over_grpc::HostProcess::start(root.path()).await?;
        let mut settings = v1::settings_client::SettingsClient::connect(process.url.clone()).await?;
        let frame = settings.get_settings(v1::SettingsGetSettingsRequest {}).await?.into_inner();
        assert_eq!(std::path::Path::new(&frame.bootstrap_paths.unwrap().storage), root.path());
        let before = settings.read_application_settings(v1::SettingsReadApplicationSettingsRequest {}).await?.into_inner();
        let mut changed = before.settings.clone().unwrap();
        changed.report.as_mut().unwrap().plot_points = 321;
        let update = v1::UpdateSettings {
            expected_revision: before.revision,
            settings: Some(changed.clone()),
        };
        let saved = settings
            .update_application_settings(v1::SettingsUpdateApplicationSettingsRequest { update: Some(update.clone()) })
            .await?
            .into_inner();
        assert_eq!(saved.settings, Some(changed));
        assert!(saved.revision > before.revision);
        assert_eq!(
            settings
                .update_application_settings(v1::SettingsUpdateApplicationSettingsRequest { update: Some(update) })
                .await
                .unwrap_err()
                .code(),
            tonic::Code::Aborted
        );

        // Validate Before Committing and Preserve the Previous Revision
        let mut invalid = saved.settings.clone().unwrap();
        invalid.client.as_mut().unwrap().connect_timeout_ms = 0;
        assert!(
            settings
                .update_application_settings(v1::SettingsUpdateApplicationSettingsRequest {
                    update: Some(v1::UpdateSettings {
                        expected_revision: saved.revision,
                        settings: Some(invalid)
                    })
                })
                .await
                .is_err()
        );
        assert_eq!(settings.read_application_settings(v1::SettingsReadApplicationSettingsRequest {}).await?.into_inner(), saved);
        let sources = settings.list_storage_sources(v1::SettingsListStorageSourcesRequest {}).await?.into_inner();
        assert_eq!(
            settings
                .reorder_storage_sources(v1::SettingsReorderStorageSourcesRequest {
                    identifiers: sources.sources.iter().map(|source| source.identifier.clone()).collect()
                })
                .await?
                .into_inner(),
            sources
        );
        assert!(settings.get_plugin_settings(v1::SettingsGetPluginSettingsRequest { plugin_id: "missing".into() }).await.is_err());
        let plugin_before =
            settings.get_plugin_settings(v1::SettingsGetPluginSettingsRequest { plugin_id: "settings.fixture".into() }).await?.into_inner();
        assert_eq!(plugin_before.values.unwrap().values["enabled"].kind, Some(v1::value::Kind::Boolean(true)));
        let values = v1::MapValue {
            values: std::collections::HashMap::from([(
                "enabled".into(),
                v1::Value {
                    kind: Some(v1::value::Kind::Boolean(false)),
                },
            )]),
        };
        let plugin_saved = settings
            .update_plugin_settings(v1::SettingsUpdatePluginSettingsRequest {
                plugin_id: "settings.fixture".into(),
                values: Some(values.clone()),
            })
            .await?
            .into_inner();
        assert_eq!(plugin_saved.values, Some(values));
        assert!(
            settings
                .update_plugin_settings(v1::SettingsUpdatePluginSettingsRequest {
                    plugin_id: "settings.fixture".into(),
                    values: Some(v1::MapValue {
                        values: std::collections::HashMap::from([(
                            "enabled".into(),
                            v1::Value {
                                kind: Some(v1::value::Kind::Text("invalid".into()))
                            }
                        )])
                    })
                })
                .await
                .is_err()
        );
        process.close().await?;

        // Reopen Settings Without Losing the Saved Revision
        let process = super::client_runs_thermal_suite_over_grpc::HostProcess::start(root.path()).await?;
        let mut settings = v1::settings_client::SettingsClient::connect(process.url.clone()).await?;
        assert_eq!(settings.read_application_settings(v1::SettingsReadApplicationSettingsRequest {}).await?.into_inner(), saved);
        assert_eq!(
            settings.get_plugin_settings(v1::SettingsGetPluginSettingsRequest { plugin_id: "settings.fixture".into() }).await?.into_inner(),
            plugin_saved
        );
        process.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
