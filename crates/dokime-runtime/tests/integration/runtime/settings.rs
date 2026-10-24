/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, fs, sync::Arc};
// Third Party Imports
use dokime_runtime::domains::{ActionDispatcher, ActionDispatcherError, ActionDispatcherFuture, StatusResponse};
use dokime_runtime::{HostCardBlock, HostCardTone, Identifier, Metadata, RedactedSecretBinding, SecretBackendKind, Value};
// Local Imports
use dokime_runtime::RuntimeHandle;
use dokime_runtime::domains::settings::RuntimeSettingsCardState;

use dokime_runtime::domains::installed_plugins::{PluginSecretUpdate, PluginSettingsUpdate};
use dokime_runtime::settings::{SecretReference, SecretSetting, SettingValues, Settings, SettingsEntry, TextArea};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[derive(Clone)]
struct FixtureStatusDispatcher {
    status: Result<StatusResponse, ActionDispatcherError>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl ActionDispatcher for FixtureStatusDispatcher {
    fn status<'a>(&'a self) -> ActionDispatcherFuture<'a, StatusResponse> {
        Box::pin(async move { self.status.clone() })
    }

    fn query_options<'a>(&'a self, entrypoint: &'a str) -> ActionDispatcherFuture<'a, Vec<dokime_runtime::domains::OptionProviderOption>> {
        Box::pin(async move { Err(ActionDispatcherError::UnknownEntrypoint(entrypoint.to_owned())) })
    }
}

/* === Public Functions === */

/* === Private Functions === */
async fn fixture_runtime(module: Option<dokime_runtime::InstalledPlugin>) -> (tempfile::TempDir, dokime_runtime::Runtime, RuntimeHandle) {
    fixture_runtime_with_dispatcher(module, None).await
}

async fn fixture_runtime_with_dispatcher(
    module: Option<dokime_runtime::InstalledPlugin>, dispatcher: Option<Arc<dyn ActionDispatcher>>,
) -> (tempfile::TempDir, dokime_runtime::Runtime, RuntimeHandle) {
    let root = tempfile::tempdir().unwrap();
    let artifact_id = module.as_ref().map(|module| Identifier::new(&module.source).unwrap());
    let mut config =
        crate::service_support::budgets::runtime_config(root.path(), metadata("runtime.fixture", "Runtime")).with_installed_plugins(module);
    if let (Some(artifact_id), Some(dispatcher)) = (artifact_id, dispatcher) {
        config = config.with_action_dispatcher(artifact_id, dispatcher);
    }
    let runtime = {
        let runtime_config = config;
        dokime_runtime::Runtime::start(runtime_config)
    }
    .await
    .unwrap();
    let handle = runtime.handle();
    (root, runtime, handle)
}

fn fixture_module() -> dokime_runtime::InstalledPlugin {
    let mut module = installed_plugin("fixture.artifact");
    module.settings_status = Some(dokime_runtime::domains::PluginEntrypoint { name: "fixture.status".into() });
    module
}

fn installed_plugin(identifier: &str) -> dokime_runtime::InstalledPlugin {
    dokime_runtime::InstalledPlugin {
        source: identifier.to_owned(),
        version: "test".to_owned(),
        description: "Fixture Settings".to_owned(),
        enabled: true,
        settings: dokime_runtime::settings::Settings {
            contributions: vec![dokime_runtime::settings::SettingsEntry::TextArea(dokime_runtime::settings::TextArea {
                id: "workspace".to_owned(),
                title: "Workspace".to_owned(),
                description: None,
                default: "declared-workspace".to_owned(),
                required: true,
            })],
        },
        ..Default::default()
    }
}

fn metadata(identifier: &str, label: &str) -> Metadata {
    Metadata::new(Identifier::new(identifier).unwrap(), label, None)
}

/* === Tests === */

// live status supplies the state and every row the probe answered.

#[tokio::test]
async fn live_status_answered() {
    let dispatcher = FixtureStatusDispatcher {
        status: Ok(StatusResponse {
            state: "connected".to_owned(),
            fields: BTreeMap::from([("Workspace".to_owned(), "Live workspace".into()), ("Sync".to_owned(), "Enabled".into())]),
        }),
    };
    let (_root, mut owner, runtime) = fixture_runtime_with_dispatcher(Some(fixture_module()), Some(Arc::new(dispatcher))).await;

    let frame = runtime.settings_frame().await;
    let card = &frame.cards[0];

    assert_eq!(card.state, RuntimeSettingsCardState::Ready);
    assert!(matches!(
        &card.body.blocks[0],
        HostCardBlock::Status { label, tone, detail }
            if label == "Connected"
                && *tone == HostCardTone::Success
                && detail.as_deref() == Some("Live sidecar state: connected.")
    ));
    assert!(matches!(
        &card.body.blocks[1],
        HostCardBlock::KeyValueList { items }
            if items.len() == 2
                && items.iter().all(|item| item.detail.is_none())
                && items.iter().any(|item| item.label == "Workspace" && item.value == "Live workspace")
                && items.iter().any(|item| item.label == "Sync" && item.value == "Enabled")
    ));
    drop(runtime);
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn dispatcher_alone_probe() {
    let mut module = fixture_module();
    module.settings_status = None;
    let dispatcher = FixtureStatusDispatcher { status: Err(ActionDispatcherError::Unavailable) };
    let (_root, mut owner, runtime) = fixture_runtime_with_dispatcher(Some(module), Some(Arc::new(dispatcher))).await;

    let frame = runtime.settings_frame().await;

    assert!(frame.cards.is_empty(), "the plugin's own section is what declares the probe, not the host's dispatcher");
    drop(runtime);
    owner.shutdown().await.unwrap();
}

// discovered settings are stored beside saved values until they are saved.

#[tokio::test]
async fn discovered_settings_until() {
    let (_root, mut owner, runtime) = fixture_runtime(Some(fixture_module())).await;
    let artifact_id = Identifier::new("fixture.artifact").unwrap();

    // Nothing discovered yet: the card shows what the plugin declared.
    let settings = runtime.plugin_settings(&artifact_id).await.unwrap();
    assert!(settings.discovered.is_empty());
    assert_eq!(settings.values["workspace"], "declared-workspace".into());

    // The plugin discovered one, and the host stored it with its marker.
    runtime
        .record_discovered_settings(&artifact_id, BTreeMap::from([("workspace".to_owned(), Value::from("found-workspace"))]))
        .await
        .unwrap();
    let settings = runtime.plugin_settings(&artifact_id).await.unwrap();
    assert_eq!(settings.discovered["workspace"], Value::from("found-workspace"));
    assert_eq!(settings.values["workspace"], "declared-workspace".into(), "a discovered value is used by no run until it is saved");

    // Saving it as it stands is what makes it an edited value: it moves into
    // the saved values and stops being offered as a suggestion.
    runtime
        .save_plugin_settings(
            &artifact_id,
            dokime_runtime::domains::installed_plugins::PluginSettingsUpdate {
                values: BTreeMap::from([("workspace".to_owned(), Value::from("found-workspace"))]),
            },
        )
        .await
        .unwrap();
    let settings = runtime.plugin_settings(&artifact_id).await.unwrap();
    assert!(settings.discovered.is_empty());
    assert_eq!(settings.values["workspace"], Value::from("found-workspace"));
    drop(runtime);
    owner.shutdown().await.unwrap();
}

#[tokio::test]
async fn resolves_one_record() {
    let artifact_id = Identifier::new("fabric").unwrap();
    let settings = Settings {
        contributions: vec![
            SettingsEntry::TextArea(TextArea {
                id: "workspace".into(),
                title: "Workspace".into(),
                description: None,
                required: true,
                default: "default-workspace".into(),
            }),
            SettingsEntry::SecretReference(SecretSetting {
                id: "access_token".into(),
                title: "Access token".into(),
                description: None,
                required: true,
                default: None,
            }),
        ],
    };
    let mut artifact = installed_plugin("fabric");
    artifact.settings = settings;
    let (_root, mut runtime, handle) = fixture_runtime(Some(artifact)).await;
    handle.save_plugin_secret(&artifact_id, "access_token", PluginSecretUpdate { secret: "secret-token".into() }).await.unwrap();
    handle
        .save_plugin_settings(
            &artifact_id,
            PluginSettingsUpdate {
                values: BTreeMap::from([
                    ("workspace".into(), Value::from("team-prod")),
                    ("access_token".into(), SecretReference::StoredSecret { secret_id: "fabric.access_token".into() }.into()),
                ]),
            },
        )
        .await
        .unwrap();

    let resolved = handle.plugin_settings(&artifact_id).await.unwrap();

    assert_eq!(resolved.values["workspace"], Value::from("team-prod"));
    assert_eq!(
        resolved.secret_bindings.bindings["access_token"],
        RedactedSecretBinding { secret_id: "fabric.access_token".to_owned(), backend: SecretBackendKind::OsKeyring, configured: true }
    );
    assert!(resolved.secret_bindings.missing_required.is_empty());
    runtime.shutdown().await.unwrap();
}

// saves only references and rejects secret material without changing values.

#[tokio::test]
async fn secret_references_file() {
    let id = Identifier::new("fixture").unwrap();
    let catalog = Settings {
        contributions: vec![
            SettingsEntry::SecretReference(SecretSetting {
                id: "token".into(),
                title: "Token".into(),
                description: None,
                default: None,
                required: true,
            }),
            SettingsEntry::TextArea(TextArea {
                id: "name".into(),
                title: "Name".into(),
                description: None,
                default: "default-name".into(),
                required: true,
            }),
        ],
    };
    let mut artifact = installed_plugin("fixture");
    artifact.settings = catalog;
    let (root, mut runtime, handle) = fixture_runtime(Some(artifact)).await;
    handle.save_plugin_secret(&id, "token", PluginSecretUpdate { secret: "never-persist-plaintext-here".into() }).await.unwrap();
    let resolved = handle.plugin_settings(&id).await.unwrap();
    assert_eq!(resolved.values["name"], "default-name".into());
    assert!(resolved.secret_bindings.missing_required.is_empty());
    handle.save_plugin_settings(&id, PluginSettingsUpdate { values: resolved.values.clone() }).await.unwrap();
    let settings_file = root.path().join("application/settings.json");
    let saved = handle.plugin_settings(&id).await.unwrap();
    assert!(!fs::read_to_string(&settings_file).unwrap().contains("never-persist-plaintext-here"));
    let mut invalid = resolved.values;
    invalid.insert("token".into(), "never-persist-plaintext-here".into());
    assert!(handle.save_plugin_settings(&id, PluginSettingsUpdate { values: invalid }).await.is_err());
    assert_eq!(handle.plugin_settings(&id).await.unwrap().values, saved.values);
    let environment = SettingValues::from([(
        "token".into(),
        SecretReference::Environment { variable: "DOKIME_TEST_ABSENT_ENV_REFERENCE".into() }.into(),
    )]);
    handle.save_plugin_settings(&id, PluginSettingsUpdate { values: environment }).await.unwrap();
    let resolved = handle.plugin_settings(&id).await.unwrap();
    assert_eq!(
        SecretReference::try_from(&resolved.values["token"]).unwrap(),
        SecretReference::Environment { variable: "DOKIME_TEST_ABSENT_ENV_REFERENCE".into() }
    );
    assert!(!fs::read_to_string(settings_file).unwrap().contains("never-persist-plaintext-here"));
    runtime.shutdown().await.unwrap();
}

#[tokio::test]
async fn stored_secret_bindings() {
    use dokime_domain::{model::secrets::catalog, operations::secrets};
    let root = tempfile::tempdir().unwrap();
    let id = Identifier::new("zephyr").unwrap();
    let mut artifact = installed_plugin(id.as_str());
    artifact.settings = Settings {
        contributions: vec![SettingsEntry::SecretReference(SecretSetting {
            id: "api_token".into(),
            title: "API Token".into(),
            description: None,
            required: true,
            default: None,
        })],
    };
    let config = crate::service_support::budgets::runtime_config(root.path(), metadata("runtime.secrets", "Secrets"))
        .with_installed_plugins([artifact]);
    let mut runtime = dokime_runtime::Runtime::start(config.clone()).await.unwrap();
    let handle = runtime.handle();
    let snapshot = handle
        .operations()
        .call(secrets::Put {
            metadata: catalog::Metadata { name: "zephyr.api_token".into(), label: "API Token".into(), description: None },
            value: Some("secret-token".into()),
            generation: 0,
        })
        .await
        .unwrap();
    assert_eq!(snapshot.generation, 1);
    assert!(!serde_json::to_string(&*snapshot).unwrap().contains("secret-token"));
    let value = handle.operations().call(secrets::Lookup { name: "zephyr.api_token".into() }).await.unwrap();
    assert_eq!(value.value.as_deref(), Some("secret-token"));
    let reference = SecretReference::StoredSecret { secret_id: "zephyr.api_token".into() };
    let values = BTreeMap::from([("api_token".into(), reference.into())]);
    handle.save_plugin_settings(&id, PluginSettingsUpdate { values }).await.unwrap();
    assert!(handle.plugin_settings(&id).await.unwrap().secret_bindings.missing_required.is_empty());

    let replacement =
        secrets::Put { metadata: snapshot.entries[0].metadata.clone(), value: Some("replacement-token".into()), generation: 1 };
    handle.operations().call(replacement.clone()).await.unwrap();
    assert!(handle.operations().call(replacement).await.is_err());
    let fresh = handle.operations().call(secrets::Lookup { name: "zephyr.api_token".into() }).await.unwrap();
    assert_eq!(fresh.value.as_deref(), Some("replacement-token"));
    assert_eq!(value.value.as_deref(), Some("secret-token"));
    drop(handle);
    runtime.shutdown().await.unwrap();

    let mut reopened = dokime_runtime::Runtime::start(config).await.unwrap();
    assert!(!fs::read_to_string(root.path().join("application/settings.json")).unwrap().contains("secret-token"));
    let handle = reopened.handle();
    assert_eq!(
        handle.operations().call(secrets::Lookup { name: "zephyr.api_token".into() }).await.unwrap().value.as_deref(),
        Some("replacement-token")
    );
    handle.operations().call(secrets::Remove { names: vec!["zephyr.api_token".into()], generation: 2 }).await.unwrap();
    assert!(handle.operations().call(secrets::Lookup { name: "zephyr.api_token".into() }).await.is_err());
    assert_eq!(handle.plugin_settings(&id).await.unwrap().secret_bindings.missing_required, vec!["api_token"]);
    drop(handle);
    reopened.shutdown().await.unwrap();
}

// source settings persist order and enabled state without transferring evidence.

#[tokio::test]
async fn source_settings_absent() {
    let root = tempfile::tempdir().unwrap();
    let config = crate::service_support::budgets::runtime_config(
        root.path(),
        dokime_runtime::Metadata::new(dokime_runtime::Identifier::new("storage.settings").unwrap(), "Storage Settings", None),
    );
    {
        let state = dokime_runtime::storage::sqlite::State::open(root.path()).await.unwrap();
        state.register_source("fabric", "microsoft.fabric").await.unwrap();
    }
    let mut runtime = dokime_runtime::Runtime::with_execution_fixtures_without_builtins(config.clone(), std::iter::empty()).await.unwrap();
    let handle = runtime.handle();
    handle.reorder_storage_sources(vec!["fabric".into(), "local".into()]).await.unwrap();
    handle.set_storage_source_enabled("fabric", false).await.unwrap();
    let expected = handle.storage_sources().await.unwrap();
    assert_eq!(expected[0].identifier, "fabric");
    assert!(!expected[0].enabled);
    assert_eq!(expected[1].priority, 1);
    assert!(handle.reorder_storage_sources(vec!["local".into()]).await.is_err());
    assert!(handle.set_storage_source_enabled("local", false).await.is_err());
    assert_eq!(handle.storage_sources().await.unwrap(), expected);
    assert!(handle.browse_executions(Default::default()).await.unwrap().entries.is_empty());
    drop(handle);
    runtime.shutdown().await.unwrap();

    let mut runtime = dokime_runtime::Runtime::with_execution_fixtures_without_builtins(config, std::iter::empty()).await.unwrap();
    let handle = runtime.handle();
    assert_eq!(handle.storage_sources().await.unwrap(), expected);
    handle.set_storage_source_enabled("fabric", true).await.unwrap();
    assert!(handle.storage_sources().await.unwrap()[0].enabled);
    drop(handle);
    runtime.shutdown().await.unwrap();
}
