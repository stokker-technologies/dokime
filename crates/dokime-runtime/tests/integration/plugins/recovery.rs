/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Recover Installed Bundles Independently Of Execution Storage.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeSet;

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

fn recovery_bundle(owner: &str) -> Vec<u8> {
    use dokime_runtime::settings::{Boolean, Settings, SettingsEntry};
    let settings = Settings {
        contributions: vec![SettingsEntry::Boolean(Boolean {
            id: "enabled".into(),
            title: "Capture".into(),
            description: None,
            default: false,
        })],
    };
    crate::service_support::bundles::bundle_with_settings("capture-fixture", owner, settings)
}

/* === Tests === */

#[dokime_tracing::trace_test]
#[tokio::test]
async fn backup_reset_preserves_bundle_state_settings_and_secrets() {
    use dokime_domain::model::identity::Identifier;
    use dokime_domain::model::secrets::catalog::{Metadata, Slot};
    use dokime_runtime::plugins::{PluginRecovery, PluginSettings, PluginState, ProgramValues};

    for (enabled, corrupt_settings) in [(false, false), (true, false), (false, true)] {
        let root = tempfile::tempdir().unwrap();
        let application = dokime_runtime::paths::ApplicationPaths::new(root.path());
        let data = application.default_data();
        let config = || {
            let mut config = dokime_runtime::RuntimeConfig::new(
                &data,
                dokime_runtime::Metadata::new(Identifier::new("recovery.fixture").unwrap(), "Fixture", None),
            )
            .with_credential_store(keyring_core::mock::Store::new().unwrap());
            config.application = application.clone();
            config
        };
        let mut original = dokime_runtime::Runtime::start(config()).await.unwrap();
        let handle = original.handle();
        let bytes = recovery_bundle("recovery_fixture");
        let store = handle.plugin_store();
        let id = store.install(&bytes, &[], &BTreeSet::new()).await.unwrap().installation.install_id;
        store.set_enabled(id, enabled).await.unwrap();
        let removed_id = uuid::Uuid::now_v7();
        let orphan_id = uuid::Uuid::now_v7();
        handle
            .settings()
            .edit(move |metadata| {
                let plugin = metadata.plugins.get_mut(&id).unwrap();
                plugin.programs.insert(
                    "recovery_fixture".into(),
                    ProgramValues {
                        values: [
                            ("enabled".into(), dokime_runtime::Value::Bool(true)),
                            ("removed".into(), dokime_runtime::Value::Bool(false)),
                        ]
                        .into(),
                        saved: ["enabled".into(), "removed".into()].into(),
                    },
                );
                let mut removed = plugin.clone();
                removed.state = PluginState::RemovalPending;
                metadata.plugins.insert(removed_id, removed);
                metadata.secrets.entries.insert(
                    "recovery_token".into(),
                    Slot {
                        metadata: Metadata {
                            identifier: "recovery_token".into(),
                            label: "Fixture".into(),
                            description: None,
                        },
                        slot: uuid::Uuid::now_v7().to_string(),
                    },
                );
                Ok::<_, dokime_runtime::storage::Error>(())
            })
            .await
            .unwrap();
        std::fs::write(store.path(removed_id), &bytes).unwrap();
        std::fs::write(store.path(orphan_id), &bytes).unwrap();
        let saved = handle.settings().read(Clone::clone).await.unwrap();
        drop(handle);
        original.shutdown().await.unwrap();

        // Capture Still Works When The Execution Schema Cannot Be Opened.
        let database = rusqlite::Connection::open(data.join("local.sqlite")).unwrap();
        database.pragma_update(None, "user_version", 999).unwrap();
        if corrupt_settings {
            let mut metadata = serde_json::to_value(&saved).unwrap();
            metadata["plugins"][id.to_string()]["programs"] = serde_json::json!("invalid");
            database.execute("UPDATE application_metadata SET body=?1 WHERE id=1", [serde_json::to_vec(&metadata).unwrap()]).unwrap();
        }
        drop(database);
        let recovery = PluginRecovery::capture(&data, &application).unwrap();
        assert!(data.starts_with(root.path()));
        std::fs::remove_dir_all(&data).unwrap();
        recovery.restore(&data, &application).await.unwrap();
        let mut restarted = dokime_runtime::Runtime::start(config()).await.unwrap();
        let handle = restarted.handle();
        let entry = handle.plugin_store().get(id).await.unwrap();
        assert_eq!(handle.plugin_store().list().await.unwrap().len(), 1);
        assert!(!handle.plugin_store().path(removed_id).exists());
        assert!(!handle.plugin_store().path(orphan_id).exists());
        assert_eq!(entry.installation.state, if enabled { PluginState::Installed } else { PluginState::Disabled });
        assert!(entry.metadata.is_some());
        assert_eq!(std::fs::read(handle.plugin_store().path(id)).unwrap(), bytes);
        let restored = handle.settings().read(Clone::clone).await.unwrap();
        assert_eq!(restored.identity, saved.identity);
        assert_eq!(restored.secrets, saved.secrets);
        let PluginSettings { programs, .. } = &restored.plugins[&id];
        if corrupt_settings {
            assert!(programs.is_empty());
        } else {
            assert_eq!(programs["recovery_fixture"].values, [("enabled".into(), dokime_runtime::Value::Bool(true))].into());
            assert_eq!(programs["recovery_fixture"].saved, ["enabled".into()].into());
        }
        drop(handle);
        restarted.shutdown().await.unwrap();
    }
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn backup_reset_keeps_refused_bundles_without_blocking_startup() {
    use dokime_domain::model::identity::Identifier;
    use dokime_runtime::plugins::PluginRecovery;

    let root = tempfile::tempdir().unwrap();
    let application = dokime_runtime::paths::ApplicationPaths::new(root.path());
    let data = application.default_data();
    let config = || {
        let mut config = dokime_runtime::RuntimeConfig::new(
            &data,
            dokime_runtime::Metadata::new(Identifier::new("recovery.fixture").unwrap(), "Fixture", None),
        )
        .with_credential_store(keyring_core::mock::Store::new().unwrap());
        config.application = application.clone();
        config
    };
    let mut original = dokime_runtime::Runtime::start(config()).await.unwrap();
    let store = original.handle().plugin_store();
    let id = store.install(&recovery_bundle("refused_fixture"), &[], &BTreeSet::new()).await.unwrap().installation.install_id;
    original.shutdown().await.unwrap();
    std::fs::write(store.path(id), b"refused bundle").unwrap();
    drop(store);
    let recovery = PluginRecovery::capture(&data, &application).unwrap();
    assert!(data.starts_with(root.path()));
    std::fs::remove_dir_all(&data).unwrap();
    recovery.restore(&data, &application).await.unwrap();
    let mut restarted = dokime_runtime::Runtime::start(config()).await.unwrap();
    let handle = restarted.handle();
    let store = handle.plugin_store();
    let mut entry = store.get(id).await.unwrap();
    handle.plugin_process_state(&mut entry);
    assert!(entry.error.is_some());
    assert!(entry.boot_failed);
    assert!(!entry.active);
    // An Unreadable Recovered Bundle Cannot Block Subsequent Bundled Installation.
    let installed = store.install_bundled(&recovery_bundle("bundled_fixture"), &[], &BTreeSet::new()).await.unwrap();
    assert_ne!(installed.installation.install_id, id);
    assert_eq!(store.list().await.unwrap().len(), 2);
    drop(handle);
    drop(store);
    restarted.shutdown().await.unwrap();
}
