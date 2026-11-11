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
/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[dokime_runtime::settings]
#[derive(Clone, Debug, Default, PartialEq)]
struct CaptureSettings {
    capture: Capture,
}

#[dokime_runtime::section(label = "File Capture")]
#[derive(Clone, Debug, Default, PartialEq)]
struct Capture {
    #[setting(label = "Capture Files", default = false)]
    enabled: bool,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

// An Inert Installable Sidecar Declaring The Capture Settings Catalog.
fn capture_bundle(owner: &str) -> Vec<u8> {
    use dokime_runtime::plugin::registration::{SETTINGS_NAMESPACE, SettingsRegistration, domain_schema};
    use dokime_runtime::plugin::settings::PluginSettings;
    use harmos_packaging::bundle::{Bundle, Executable, Facility, Identity, Limits, Source};

    let registration = harmos::sidecar::Registration {
        namespace: SETTINGS_NAMESPACE.into(),
        schema: domain_schema(SETTINGS_NAMESPACE),
        payload: serde_json::to_vec(&SettingsRegistration {
            catalog: CaptureSettings::catalog(),
            ..Default::default()
        })
        .unwrap(),
    };
    let policy = harmos::operations::OperationPolicy {
        timeout: std::time::Duration::from_secs(120),
        attempt_timeout: std::time::Duration::from_secs(120),
        cancellation_timeout: std::time::Duration::ZERO,
        retries: harmos::operations::RetryPolicy::Never,
    };
    let declaration =
        serde_json::json!({"kind":"plugin","facility":"sidecar","id":owner,"version":"1.0.0","description":"A fixture.","config":"()"});
    let lifecycle = serde_json::json!([
        {"action": "initialize", "identity": "harmos.operation.v1", "policy": policy},
        {"action": "shutdown", "identity": "harmos.operation.v1", "policy": policy},
    ]);
    let contracts =
        serde_json::json!({"kind":"contracts","facility":"sidecar","owner":owner,"entry":{"direction":"lifecycle","lifecycle":lifecycle}});
    let restart = serde_json::json!({"kind":"restart","facility":"sidecar","owner":owner,"policy":{"credit":3,"sustained_health_ms":30000,"backoff_base_ms":250,"backoff_cap_ms":10000}});
    let publications = serde_json::json!({"kind":"publications","facility":"sidecar","owner":owner,"streams":[]});
    let observers = serde_json::json!({"kind":"observers","facility":"sidecar","owner":owner,"observers":[]});
    let executable = format!(
        "inert{prefix}{declaration}\0{prefix}{contracts}\0{prefix}{restart}\0{prefix}{publications}\0{prefix}{observers}\0",
        prefix = std::str::from_utf8(harmos::artifacts::PREFIX).unwrap()
    );
    Bundle::build(
        Identity {
            name: "capture-fixture".into(),
            version: "1.0.0".parse().unwrap(),
            description: "A fixture.".into(),
            package: Default::default(),
        },
        &[Source {
            path: "bin/fixture",
            bytes: executable.as_bytes(),
            executable: Some(Executable {
                facility: Facility::Sidecar,
                target: dokime_runtime::plugins::NATIVE_TARGET.into(),
                registrations: vec![registration],
            }),
        }],
        Limits::default(),
    )
    .unwrap()
    .bytes()
    .to_vec()
}

/* === Tests === */

#[tokio::test]
async fn saved_settings_reach_restarted_program() {
    use dokime_domain::model::identity::Identifier;

    let root = tempfile::tempdir().unwrap();
    let config = || {
        dokime_runtime::RuntimeConfig::new(
            root.path(),
            dokime_runtime::Metadata::new(Identifier::new("capture.fixture").unwrap(), "Capture", None),
        )
        .with_credential_store(keyring_core::mock::Store::new().unwrap())
    };
    let id = Identifier::new("capture_fixture").unwrap();
    let mut first = dokime_runtime::Runtime::start(config()).await.unwrap();
    let handle = first.handle();
    handle.install_plugin(capture_bundle(id.as_str()), |_| Ok(())).await.unwrap();
    let mut values = handle.plugin_settings(&id).await.unwrap().values;
    values.insert("enabled".into(), dokime_runtime::Value::Bool(true));
    handle.save_plugin_settings(&id, dokime_runtime::domains::installed_plugins::PluginSettingsUpdate { values }).await.unwrap();
    drop(handle);
    first.shutdown().await.unwrap();

    // The Restarted Program Opens With What The Operator Saved.
    let mut restarted = dokime_runtime::Runtime::start(config()).await.unwrap();
    let bytes = restarted.prepared_plugin_configuration(id.as_str()).await.expect("the installed program is prepared at boot");
    let configuration: dokime_runtime::plugin::Configuration = serde_json::from_slice(&bytes).unwrap();
    assert!(configuration.settings::<CaptureSettings>().unwrap().capture.enabled);
    assert!(configuration.is_saved("enabled"));
    assert_eq!(restarted.handle().plugin_settings(&id).await.unwrap().values["enabled"], dokime_runtime::Value::Bool(true));
    restarted.shutdown().await.unwrap();
}
