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
use dokime_runtime as runtime;

use dokime_domain::model::identity::Identifier;

use dokime_runtime::settings::{SettingValues, Settings, SettingsEntry};

use runtime::settings::{SecretSetting, TextArea};

use std::collections::BTreeSet;

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
async fn direct_secret_startup() {
    let root = tempfile::tempdir().unwrap();
    let config =
        runtime::RuntimeConfig::new(root.path(), runtime::Metadata::new(Identifier::new("secret.fixture").unwrap(), "Secrets", None))
            .with_credential_store(keyring_core::mock::Store::new().unwrap());
    let mut runtime = runtime::Runtime::start(config).await.unwrap();
    let handle = runtime.handle();
    let plugin = runtime::InstalledPlugin {
        source: "fixture".into(),
        settings: Settings {
            contributions: vec![
                SettingsEntry::TextArea(TextArea {
                    id: "workspace".into(),
                    title: "Workspace".into(),
                    description: None,
                    default: "default".into(),
                    required: true,
                }),
                SettingsEntry::Secret(SecretSetting {
                    id: "optional".into(),
                    title: "Optional".into(),
                    description: None,
                    required: false,
                }),
                SettingsEntry::Secret(SecretSetting {
                    id: "api_token".into(),
                    title: "API Token".into(),
                    description: None,
                    required: true,
                }),
            ],
        },
        ..Default::default()
    };
    let id = Identifier::new(&plugin.source).unwrap();
    runtime::testing::update_plugin_preferences(&handle, &id, |stored| {
        stored.values = SettingValues::from([
            ("workspace".into(), "saved".into()),
            (
                "api_token".into(),
                runtime::Value::from_plain_json(serde_json::json!({
                    "kind": "stored_secret", "secret_id": "fixture.api_token"
                }))
                .unwrap(),
            ),
            (
                "removed".into(),
                runtime::Value::from_plain_json(serde_json::json!({
                    "kind": "environment", "variable": "PATH"
                }))
                .unwrap(),
            ),
        ]);
        stored.saved = stored.values.keys().cloned().collect();
    })
    .await
    .unwrap();
    let configured = serde_json::json!({});
    let missing = runtime::testing::plugin_initialization(&handle, &plugin, &configured).await.unwrap_err();
    assert!(
        matches!(&missing, runtime::runtime::Error::Dokime(runtime::Error::UnavailableSecret(identifier)) if identifier == "api_token")
    );

    let store = runtime::testing::secret_store(&handle);
    // A Similar Plugin-Prefixed Identifier Never Satisfies The Declaration.
    store
        .apply(
            vec![runtime::testing::SecretChange {
                metadata: dokime_domain::model::secrets::catalog::Metadata {
                    identifier: "fixture.api_token".into(),
                    label: "Other".into(),
                    description: None,
                },
                value: Some("other".into()),
            }],
            &[],
            0,
        )
        .await
        .unwrap();
    assert!(matches!(runtime::testing::plugin_initialization(&handle, &plugin, &configured).await,
        Err(runtime::runtime::Error::Dokime(runtime::Error::UnavailableSecret(identifier))) if identifier == "api_token"));

    store
        .apply(
            vec![runtime::testing::SecretChange {
                metadata: dokime_domain::model::secrets::catalog::Metadata {
                    identifier: "api_token".into(),
                    label: "API Token".into(),
                    description: None,
                },
                value: Some("direct-value".into()),
            }],
            &[],
            1,
        )
        .await
        .unwrap();
    // Reload Reuses Initialization And Operator Values Override Launch Seeds.
    let retry = runtime::testing::plugin_initialization(&handle, &plugin, &serde_json::json!({"workspace": "launch-seed"})).await.unwrap();
    assert_eq!(retry.secret("api_token").unwrap().expose(), "direct-value");
    assert_eq!(retry.secrets["optional"], None);
    assert_eq!(retry.settings, SettingValues::from([("workspace".into(), "saved".into())]));
    assert_eq!(retry.saved, BTreeSet::from(["workspace".into()]));
    assert!(!format!("{retry:?}").contains("direct-value"));
    runtime.shutdown().await.unwrap();
}
