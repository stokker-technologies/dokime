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
use dokime_runtime::plugins::PluginStore;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn fixture_bytes() -> Vec<u8> {
    std::fs::read(crate::service_support::bundles::obc_plugin_bundle()).unwrap()
}

/* === Tests === */
// Frozen Catalog Rejects Undeclared, Reserved And Duplicate Operations.

#[tokio::test]
async fn frozen_catalog_multiply() {
    use harmos_packaging::bundle::{Bundle, Limits, Source};
    let bundle = Bundle::inspect(&fixture_bytes(), Limits::default()).unwrap();
    let file = &bundle.manifest().files[0];
    // The analysis claims `obc-plugin.ripple` first, so the status probe naming
    // it is the second claim.
    for (namespace, pointer, method, reason) in [
        ("dokime.settings", "/status_entrypoint/name", "obc-plugin.missing", "undeclared"),
        ("dokime.settings", "/status_entrypoint/name", "harmos.initialize", "reserved"),
        ("dokime.settings", "/status_entrypoint/name", "obc-plugin.ripple", "more than once"),
    ] {
        let mut executable = file.executable.clone().unwrap();
        let registration = executable.registrations.iter_mut().find(|entry| entry.namespace == namespace).unwrap();
        let mut payload: serde_json::Value = serde_json::from_slice(&registration.payload).unwrap();
        *payload.pointer_mut(pointer).unwrap() = method.into();
        registration.payload = serde_json::to_vec(&payload).unwrap();
        let forged = Bundle::build(
            bundle.manifest().identity.clone(),
            &[Source {
                path: &file.path,
                bytes: bundle.file(&file.path).unwrap(),
                executable: Some(executable),
            }],
            Limits::default(),
        )
        .unwrap();
        let root = tempfile::tempdir().unwrap();
        let store = PluginStore::with_archive_limit(
            dokime_runtime::config::Settings::open(
                root.path(),
                dokime_runtime::storage::sqlite::State::open(&root.path().join("data")).await.unwrap(),
            )
            .await
            .unwrap(),
            Default::default(),
            285_212_672_usize,
        );
        assert!(store.install(forged.bytes(), &[], &Default::default()).await.unwrap_err().to_string().contains(reason));
        assert!(store.list().await.unwrap().is_empty());
    }
}

// Unsupported Domain Schemas Are Refused Before Admission.

#[tokio::test]
async fn unsupported_schema_refused() {
    use harmos_packaging::bundle::{Bundle, Limits, Source};
    let bundle = Bundle::inspect(&fixture_bytes(), Limits::default()).unwrap();
    let file = &bundle.manifest().files[0];
    for namespace in ["dokime.settings", "dokime.suites"] {
        for schema in [1, 4] {
            let mut executable = file.executable.clone().unwrap();
            let registration = executable.registrations.iter_mut().find(|entry| entry.namespace == namespace).unwrap();
            assert_eq!(registration.schema, dokime_runtime::plugin::registration::domain_schema(namespace));
            registration.schema = schema;
            let forged = Bundle::build(
                bundle.manifest().identity.clone(),
                &[Source {
                    path: &file.path,
                    bytes: bundle.file(&file.path).unwrap(),
                    executable: Some(executable),
                }],
                Limits::default(),
            )
            .unwrap();
            let root = tempfile::tempdir().unwrap();
            let store = PluginStore::with_archive_limit(
                dokime_runtime::config::Settings::open(
                    root.path(),
                    dokime_runtime::storage::sqlite::State::open(&root.path().join("data")).await.unwrap(),
                )
                .await
                .unwrap(),
                Default::default(),
                285_212_672_usize,
            );
            let error = store.install(forged.bytes(), &[], &Default::default()).await.unwrap_err().to_string();
            assert!(error.contains(&format!("unsupported `{namespace}` schema {schema}")), "{error}");
            assert!(store.list().await.unwrap().is_empty());
        }
    }
}
