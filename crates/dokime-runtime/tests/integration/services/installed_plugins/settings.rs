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

// Installed-plugin runtime contract tests.
use dokime_runtime::{
    RuntimeCredentialConfig,
    domains::installed_plugins::{PluginCredentialSecretUpdate, PluginSettingsUpdate},
};

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

#[tokio::test]
async fn boot_health_transaction() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    let boot_health_item_id = dokime_runtime::core::HealthItemId::new();
    let boot_health_observed_at = dokime_runtime::core::Timestamp::now_utc();

    runtime
        .record_health_item(dokime_runtime::execution::HealthItem {
            health_item_id: boot_health_item_id.clone(),
            scope: dokime_runtime::core::RecordScope::default(),
            severity: dokime_runtime::execution::HealthSeverity::Error,
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
        .find(|health| {
            health
                .detail
                .as_deref()
                .is_some_and(|detail| detail.contains("fixture.missing: manifest missing"))
        })
        .unwrap();
    assert_eq!(boot_health.health_item_id, boot_health_item_id);
    assert_eq!(boot_health.observed_at, boot_health_observed_at);
    assert_eq!(evidence.health.len(), 1);
}

// install definition domains records through harmos transaction.

#[tokio::test]
async fn install_definition_harmos() {
    let root = tempdir().unwrap();
    let mut registration = TestSuiteDefinition::new("fixture.suite", empty_suite()).unwrap();
    registration.record_contracts = vec![fixture_record_contract()];
    registration.telemetry_contracts = vec![fixture_telemetry_contract()];
    let artifact = dokime_runtime::InstalledPlugin {
        source: "fixture.install".into(),
        version: "test".into(),
        description: "fixture".into(),
        enabled: true,
        suites: dokime_runtime::definition::SuiteSet::new([registration]),
        ..Default::default()
    };
    let (_owner, runtime) = started(config(root.path()).with_installed_plugins([artifact])).await;
    let after = runtime
        .runtime_overview()
        .await
        .expect("runtime overview query succeeds")
        .revision;
    let suites = runtime.test_suites().await;

    assert!(after > 0);
    assert!(suites.iter().any(|suite| suite.source == "fixture.install"));
    assert!(
        runtime
            .record_contracts()
            .await
            .iter()
            .any(|contract| contract.record_type.as_str() == "fixture.event")
    );
    assert!(
        runtime
            .telemetry_contracts()
            .await
            .iter()
            .any(|contract| contract.stream.as_str() == "fixture.telemetry")
    );
}

#[tokio::test]
async fn artifact_registration_visible() {
    let root = tempdir().unwrap();
    let (_owner, runtime) = booted(root.path()).await;
    let mut registration = TestSuiteDefinition::new("fixture.suite", empty_suite()).unwrap();
    registration.record_contracts = vec![fixture_record_contract()];
    registration.telemetry_contracts = vec![fixture_telemetry_contract()];

    let register = runtime
        .set_suites(
            "fixture",
            dokime_runtime::definition::SuiteSet::new([registration.clone()]),
        )
        .await;
    let duplicate = runtime
        .set_suites("fixture", dokime_runtime::definition::SuiteSet::new([registration]))
        .await;
    let overview = runtime
        .runtime_overview()
        .await
        .expect("runtime overview query succeeds");
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
    assert!(
        record_contracts
            .iter()
            .any(|contract| contract.record_type.as_str() == "fixture.event")
    );
    assert!(
        telemetry_contracts
            .iter()
            .any(|contract| contract.stream.as_str() == "fixture.telemetry")
    );
    let RuntimeViewPayload::Suites { frame } = suite_query.payload else {
        panic!("expected suite query frame");
    };
    assert_eq!(frame.items, vec!["fixture.suite"]);
    let RuntimeViewPayload::RecordDefinitions { frame } = record_definition_query.payload else {
        panic!("expected event definition query frame");
    };
    assert!(frame.items.iter().any(|item| item == "fixture.event"));
    let RuntimeViewPayload::TelemetryDefinitions { frame } = telemetry_definition_query.payload
    else {
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

#[tokio::test]
async fn composed_plugin_process() {
    let root = tempdir().unwrap();
    let credential_file = root.path().join("credentials.json");
    let config = config(root.path())
        .with_installed_plugins([settings_artifact()])
        .with_credential_backend(RuntimeCredentialConfig {
            backend: dokime_runtime::core::CredentialBackendKind::LocalFile,
            local_file: Some(credential_file),
        });
    let artifact_id = Identifier::new("settings-fixture").unwrap();
    let (mut first, runtime) = started(config.clone()).await;
    let mut values = runtime.plugin_settings(&artifact_id).await.unwrap().values;
    values.insert(
        "workspace_id".to_owned(),
        dokime_runtime::core::Value::from("recovered-workspace"),
    );

    runtime
        .save_plugin_settings(&artifact_id, PluginSettingsUpdate { values })
        .await
        .unwrap();
    runtime
        .save_plugin_credential_secret(
            &artifact_id,
            "token",
            PluginCredentialSecretUpdate {
                secret: "first-run-secret".to_owned(),
            },
        )
        .await
        .unwrap();
    drop(runtime);
    first.shutdown().await.unwrap();

    // A Composed Plugin Has No Install ID: Its Values Last One Process
    let (mut rebooted, runtime) = started(config).await;
    let recovered = runtime.plugin_settings(&artifact_id).await.unwrap();

    assert_ne!(
        recovered.values["workspace_id"],
        dokime_runtime::core::Value::from("recovered-workspace"),
        "values of plugins without an install ID are not written to settings.json"
    );
    drop(runtime);
    rebooted.shutdown().await.unwrap();
}

fn settings_artifact() -> dokime_runtime::InstalledPlugin {
    use dokime_runtime::plugin::settings::PluginSettings;
    dokime_runtime::InstalledPlugin {
        source: "settings-fixture".into(),
        version: "test".into(),
        description: "Settings fixture".into(),
        enabled: true,
        settings: FixtureSettings::catalog(),
        ..Default::default()
    }
}

/* === Tests === */
