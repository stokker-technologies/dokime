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
        let root = super::support::tempdir()?;
        let credential_root = super::support::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
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
        let process =
            super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let mut settings = dokime_protocol::client::settings_client::SettingsClient::connect(process.url.clone()).await?;
        let frame = settings.schema(dokime_protocol::client::SettingsSchemaRequest::default()).await?.into_inner().schema.unwrap();
        assert_eq!(std::path::Path::new(&frame.bootstrap_paths.unwrap().storage), root.path());
        let before = settings.get(dokime_protocol::client::SettingsGetRequest::default()).await?.into_inner();
        let mut value = before.value.clone().unwrap();
        value.report.as_mut().unwrap().plot_points = 321;
        let update = dokime_protocol::client::SettingsSetRequest {
            value: Some(value.clone()),
            expected_revision: before.revision.clone(),
            request_id: "preferences".into(),
        };
        let saved = settings.set(update.clone()).await?.into_inner();
        assert_eq!(saved.value, Some(value));
        assert_ne!(saved.revision, before.revision);
        assert_eq!(settings.set(update.clone()).await?.into_inner(), saved);
        let mut stale = update.clone();
        stale.request_id = "stale".into();
        assert_eq!(settings.set(stale).await.unwrap_err().code(), tonic::Code::Aborted);
        let mut invalid = saved.value.clone().unwrap();
        invalid.client.as_mut().unwrap().connect_timeout_ms = 0;
        assert!(
            settings
                .set(dokime_protocol::client::SettingsSetRequest {
                    value: Some(invalid),
                    expected_revision: saved.revision.clone(),
                    request_id: "invalid".into(),
                })
                .await
                .is_err()
        );
        let current = settings.get(dokime_protocol::client::SettingsGetRequest::default()).await?.into_inner();
        assert_eq!(current.value, saved.value);
        assert_eq!(current.revision, saved.revision);
        let pinned = settings.get(dokime_protocol::client::SettingsGetRequest { at: before.resolved_at }).await?.into_inner();
        assert_eq!(pinned.value, before.value);

        // Whole Source and Plugin Values Use Their Own Revisions.
        let mut sources = dokime_protocol::client::storage::sources_client::SourcesClient::connect(process.url.clone()).await?;
        let current = sources.get(dokime_protocol::client::storage::SourcesGetRequest::default()).await?.into_inner();
        let saved_sources = sources
            .set(dokime_protocol::client::storage::SourcesSetRequest {
                value: current.value.clone(),
                expected_revision: current.revision,
                request_id: "sources".into(),
            })
            .await?
            .into_inner();
        assert_eq!(saved_sources.value, current.value);
        let mut plugins = dokime_protocol::client::plugins::settings_client::SettingsClient::connect(process.url.clone()).await?;
        assert!(
            plugins
                .get(dokime_protocol::client::plugins::SettingsGetRequest {
                    name: "missing".into(),
                    ..Default::default()
                })
                .await
                .is_err()
        );
        let before = plugins
            .get(dokime_protocol::client::plugins::SettingsGetRequest {
                name: "settings.fixture".into(),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(before.value.as_ref().unwrap().values["enabled"].kind, Some(v1::value::Kind::Boolean(true)));
        let value = v1::MapValue {
            values: std::collections::HashMap::from([(
                "enabled".into(),
                v1::Value {
                    kind: Some(v1::value::Kind::Boolean(false)),
                },
            )]),
        };
        let saved_plugin = plugins
            .set(dokime_protocol::client::plugins::SettingsSetRequest {
                name: "settings.fixture".into(),
                value: Some(value.clone()),
                expected_revision: before.revision,
                request_id: "plugin-value".into(),
            })
            .await?
            .into_inner();
        assert_eq!(saved_plugin.value, Some(value));
        let invalid = v1::MapValue {
            values: std::collections::HashMap::from([(
                "enabled".into(),
                v1::Value {
                    kind: Some(v1::value::Kind::Text("invalid".into())),
                },
            )]),
        };
        assert!(
            plugins
                .set(dokime_protocol::client::plugins::SettingsSetRequest {
                    name: "settings.fixture".into(),
                    value: Some(invalid),
                    expected_revision: saved_plugin.revision.clone(),
                    request_id: "invalid-plugin".into(),
                })
                .await
                .is_err()
        );
        process.close().await?;

        // Reopening Preserves the Exact Writable Values and Revisions.
        let process =
            super::client_runs_thermal_suite_over_grpc::HostProcess::start_with_environment(root.path(), &credentials.environment).await?;
        let mut settings = dokime_protocol::client::settings_client::SettingsClient::connect(process.url.clone()).await?;
        let reopened = settings.get(dokime_protocol::client::SettingsGetRequest::default()).await?.into_inner();
        assert_eq!(reopened.value, saved.value);
        assert_eq!(reopened.revision, saved.revision);
        let mut plugins = dokime_protocol::client::plugins::settings_client::SettingsClient::connect(process.url.clone()).await?;
        let reopened = plugins
            .get(dokime_protocol::client::plugins::SettingsGetRequest {
                name: "settings.fixture".into(),
                ..Default::default()
            })
            .await?
            .into_inner();
        assert_eq!(reopened.value, saved_plugin.value);
        assert_eq!(reopened.revision, saved_plugin.revision);
        process.close().await?;
        Ok::<(), Box<dyn std::error::Error>>(())
    })
    .await?
}
