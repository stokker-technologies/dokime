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

// Third Party Imports

// Local Imports
use super::*;
use dokime_domain::model;

use dokime_runtime::domains::installed_plugins::PluginSettingsUpdate;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
#[dokime_runtime::settings]
#[derive(Clone, Debug, Default, PartialEq)]
struct FixtureSettings {
    connection: FixtureConnection,
}

#[dokime_runtime::section(label = "Connection")]
#[derive(Clone, Debug, Default, PartialEq)]
struct FixtureConnection {
    #[setting(label = "Workspace ID")]
    workspace_id: Option<String>,

    #[setting(label = "Access Token")]
    token: Option<dokime_runtime::plugin::Secret>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

pub(crate) async fn boot_health_transaction() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    let boot_health_item_id = dokime_runtime::model::identity::new_id::<dokime_runtime::HealthItemId>();
    let boot_health_observed_at = dokime_runtime::model::time::now_utc();

    runtime
        .record_health_item(model::execution::HealthItem {
            health_item_id: boot_health_item_id.clone(),
            scope: dokime_runtime::RecordScope::default(),
            severity: model::execution::HealthSeverity::Error,
            message: "InstalledPlugin discovery failed.".to_owned(),
            detail: Some("fixture.missing: manifest missing".to_owned()),
            observed_at: boot_health_observed_at.clone(),
        })
        .await
        .unwrap();
    let evidence = runtime.evidence_snapshot().await.unwrap();

    let boot_health = evidence
        .health
        .iter()
        .find(|health| health.detail.as_deref().is_some_and(|detail| detail.contains("fixture.missing: manifest missing")))
        .unwrap();
    assert_eq!(boot_health.health_item_id, boot_health_item_id);
    assert_eq!(boot_health.observed_at, boot_health_observed_at);
    assert_eq!(evidence.health.len(), 1);
}

// install definition domains records through harmos transaction.

pub(crate) async fn install_definition_harmos() {
    let root = tempdir().unwrap();
    let mut registration = TestSuiteDefinition::new("fixture.suite", empty_suite()).unwrap();
    registration.record_contracts = vec![fixture_record_contract()];
    registration.telemetry_contracts = vec![fixture_telemetry_contract()];
    let artifact = dokime_runtime::InstalledPlugin {
        source: "fixture_install".into(),
        version: "test".into(),
        description: "fixture".into(),
        enabled: true,
        suites: dokime_runtime::definition::SuiteSet::new([registration]),
        ..Default::default()
    };
    let (_owner, runtime) = started(config(root.path()).with_installed_plugins([artifact])).await;
    let after = runtime.runtime_overview().await.expect("runtime overview query succeeds").revision;
    let suites = runtime.test_suites().await;

    assert!(after > 0);
    assert!(suites.iter().any(|suite| suite.source == "fixture_install"));
    assert!(runtime.record_contracts().await.iter().any(|contract| contract.record_type.as_str() == "fixture.event"));
    assert!(runtime.telemetry_contracts().await.iter().any(|contract| contract.stream.as_str() == "fixture.telemetry"));
}

pub(crate) async fn artifact_registration_visible() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    let mut registration = TestSuiteDefinition::new("fixture.suite", empty_suite()).unwrap();
    registration.record_contracts = vec![fixture_record_contract()];
    registration.telemetry_contracts = vec![fixture_telemetry_contract()];

    let register = runtime.set_suites("fixture", dokime_runtime::definition::SuiteSet::new([registration.clone()])).await;
    let duplicate = runtime.set_suites("fixture", dokime_runtime::definition::SuiteSet::new([registration])).await;
    let overview = runtime.runtime_overview().await.expect("runtime overview query succeeds");
    let record_contracts = runtime.record_contracts().await;
    let telemetry_contracts = runtime.telemetry_contracts().await;
    let suite_query = runtime
        .query_runtime_view(RuntimeViewQuery {
            scope: None,
            topic: RuntimeViewTopic::Suites,
            window: RuntimeWindow::Full,
            resolution: RuntimeResolution::Raw,
            cursor: None,
        })
        .await
        .unwrap();
    let record_definition_query = runtime
        .query_runtime_view(RuntimeViewQuery {
            scope: None,
            topic: RuntimeViewTopic::RecordDefinitions,
            window: RuntimeWindow::Full,
            resolution: RuntimeResolution::Raw,
            cursor: None,
        })
        .await
        .unwrap();
    let telemetry_definition_query = runtime
        .query_runtime_view(RuntimeViewQuery {
            scope: None,
            topic: RuntimeViewTopic::TelemetryDefinitions { stream: None },
            window: RuntimeWindow::Full,
            resolution: RuntimeResolution::Raw,
            cursor: None,
        })
        .await
        .unwrap();
    let suites = runtime.test_suites().await;
    let unregister = runtime.remove_suites("fixture").await;
    let after_suites = runtime.test_suites().await;
    let after_suite_query = runtime
        .query_runtime_view(RuntimeViewQuery {
            scope: None,
            topic: RuntimeViewTopic::Suites,
            window: RuntimeWindow::Full,
            resolution: RuntimeResolution::Raw,
            cursor: None,
        })
        .await
        .unwrap();

    assert!(register.is_ok());
    // Re-registering the same suite id is idempotent (reboot replays the suite
    // then boot re-installs it), so the duplicate is accepted as a no-op and the
    // suite is registered exactly once.
    assert!(duplicate.is_ok());
    assert_eq!(suites.len(), 1);
    assert_eq!(overview.test_suite_count, 1);
    assert_eq!(suites[0].source, "fixture");
    assert!(record_contracts.iter().any(|contract| contract.record_type.as_str() == "fixture.event"));
    assert!(telemetry_contracts.iter().any(|contract| contract.stream.as_str() == "fixture.telemetry"));
    let RuntimeViewPayload::Suites { frame } = suite_query.payload else {
        panic!("expected suite query frame");
    };
    assert_eq!(frame.items, vec!["fixture.suite"]);
    let RuntimeViewPayload::RecordDefinitions { frame } = record_definition_query.payload else {
        panic!("expected event definition query frame");
    };
    assert!(frame.items.iter().any(|item| item == "fixture.event"));
    let RuntimeViewPayload::TelemetryDefinitions { frame } = telemetry_definition_query.payload else {
        panic!("expected telemetry definition query frame");
    };
    assert!(frame.items.iter().any(|item| item == "fixture.telemetry"));
    assert!(unregister.is_ok());
    assert!(after_suites.is_empty());
    let RuntimeViewPayload::Suites { frame } = after_suite_query.payload else {
        panic!("expected suite query frame");
    };
    assert!(frame.items.is_empty());
}

pub(crate) async fn composed_plugin_process() {
    let root = tempdir().unwrap();
    let config = config(root.path()).with_installed_plugins([settings_artifact()]);
    let artifact_id = Identifier::new("settings_fixture").unwrap();
    let (mut first, runtime) = started(config.clone()).await;
    let mut values = runtime.plugin_settings(&artifact_id).await.unwrap().values;
    values.insert("workspace_id".to_owned(), dokime_runtime::Value::from("recovered-workspace"));

    runtime.save_plugin_settings(&artifact_id, PluginSettingsUpdate { values }).await.unwrap();
    drop(runtime);
    first.shutdown().await.unwrap();

    // A Composed Plugin Has No Install ID: Its Values Last One Process
    let (mut rebooted, runtime) = started(config).await;
    let recovered = runtime.plugin_settings(&artifact_id).await.unwrap();

    assert_ne!(
        recovered.values["workspace_id"],
        dokime_runtime::Value::from("recovered-workspace"),
        "values of plugins without an install ID are not persisted"
    );
    drop(runtime);
    rebooted.shutdown().await.unwrap();
}

fn settings_artifact() -> dokime_runtime::InstalledPlugin {
    use dokime_runtime::plugin::settings::PluginSettings;
    dokime_runtime::InstalledPlugin {
        source: "settings_fixture".into(),
        version: "test".into(),
        description: "Settings fixture".into(),
        enabled: true,
        settings: FixtureSettings::catalog(),
        ..Default::default()
    }
}

/* === Tests === */

#[tokio::test]
async fn non_admitted_settings() {
    let root = tempdir().unwrap();
    let runtime_config = config(root.path());
    let (mut owner, runtime) = started(runtime_config.clone()).await;
    let schema = settings_artifact().settings;
    let bytes = crate::service_support::bundles::bundle_with_settings("failed-settings", "settings_fixture", schema.clone());
    let entry = runtime.install_plugin(bytes, |_| Ok(())).await.unwrap();
    let store = runtime.plugin_store();
    store
        .record_boot(
            entry.installation.install_id,
            Some(dokime_domain::model::plugins::wire::PluginStartFailure {
                plugin_id: "fixture".into(),
                plugin_name: "Fixture".into(),
                message: "startup-refused".into(),
                code: None,
                missing_secret: None,
                consequences: Vec::new(),
                carrier: Vec::new(),
            }),
        )
        .await
        .unwrap();
    let id = Identifier::new("settings_fixture").unwrap();
    let frame = runtime
        .query_runtime_view(RuntimeViewQuery {
            topic: RuntimeViewTopic::InstalledPlugins,
            scope: None,
            window: RuntimeWindow::Full,
            resolution: RuntimeResolution::Raw,
            cursor: None,
        })
        .await
        .unwrap();
    let RuntimeViewPayload::InstalledPlugins { frame } = frame.payload else {
        panic!("unexpected payload");
    };
    assert!(frame.installed_plugins.iter().all(|plugin| plugin.plugin_id != id));
    let initial = runtime.plugin_settings(&id).await.unwrap();
    assert_eq!(initial.settings, schema);
    let mut values = initial.values;
    values.insert("workspace_id".into(), Value::from("corrected"));
    runtime.save_plugin_settings(&id, PluginSettingsUpdate { values }).await.unwrap();
    let snapshot = runtime
        .operations()
        .call::<dokime_runtime::operations::secrets::Put>(dokime_runtime::operations::secrets::Put {
            metadata: model::secrets::catalog::Metadata {
                identifier: "token".into(),
                label: "Token".into(),
                description: None,
            },
            value: Some("replacement".into()),
            generation: 0,
        })
        .await
        .unwrap();
    assert_eq!(snapshot.entries[0].known_uses, ["settings_fixture"]);
    let updated = runtime.plugin_settings(&id).await.unwrap();
    assert!(updated.secrets[0].available);
    assert!(!serde_json::to_string(&updated).unwrap().contains("replacement"));
    store.set_enabled(entry.installation.install_id, false).await.unwrap();
    drop(runtime);
    owner.shutdown().await.unwrap();

    let (mut owner, runtime) = started(runtime_config).await;
    let recovered = runtime.plugin_settings(&id).await.unwrap();
    let store = runtime.plugin_store();
    assert_eq!(recovered.values["workspace_id"], Value::from("corrected"));
    assert!(recovered.secrets[0].available);
    let mut invalid = recovered.values.clone();
    invalid.insert("workspace_id".into(), Value::from(true));
    assert!(runtime.save_plugin_settings(&id, PluginSettingsUpdate { values: invalid }).await.is_err());
    assert_eq!(runtime.plugin_settings(&id).await.unwrap().values, recovered.values);
    let path = store.path(entry.installation.install_id);
    let authorized = std::fs::read(&path).unwrap();
    std::fs::write(&path, b"unauthorized").unwrap();
    assert!(runtime.plugin_settings(&id).await.is_err());
    std::fs::write(&path, authorized).unwrap();
    store.remove(entry.installation.install_id).await.unwrap();
    assert!(runtime.plugin_settings(&id).await.is_err());
    drop(runtime);
    owner.shutdown().await.unwrap();
}
