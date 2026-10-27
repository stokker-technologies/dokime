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
    use crate::fixtures::{PREPARE, prepared_directory, require_current};
    let directory = prepared_directory();
    require_current(&directory, PREPARE);
    let bundle = directory.join("obc-plugin/obc-plugin.dokime");
    std::fs::read(&bundle).unwrap_or_else(|error| panic!("{PREPARE}: {}: {error}", bundle.display()))
}

/* === Tests === */
// frozen catalog cannot bind missing, reserved or multiply claimed operations.

pub(crate) async fn frozen_catalog_multiply() {
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
            &[Source { path: &file.path, bytes: bundle.file(&file.path).unwrap(), executable: Some(executable) }],
            Limits::default(),
        )
        .unwrap();
        let root = tempfile::tempdir().unwrap();
        let store = PluginStore::with_archive_limit(
            dokime_runtime::config::Settings::open(
                root.path(),
                dokime_runtime::storage::sqlite::State::open(&root.path().join("data"), &Default::default()).await.unwrap(),
            )
            .await
            .unwrap(),
            Default::default(),
            285_212_672 as usize,
        );
        assert!(store.stage(forged.bytes()).await.unwrap_err().to_string().contains(reason));
        assert!(store.list().await.unwrap().is_empty());
    }
}

// legacy and unknown domain carriers are refused before admission.

pub(crate) async fn legacy_unknown_refused() {
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
                &[Source { path: &file.path, bytes: bundle.file(&file.path).unwrap(), executable: Some(executable) }],
                Limits::default(),
            )
            .unwrap();
            let root = tempfile::tempdir().unwrap();
            let store = PluginStore::with_archive_limit(
                dokime_runtime::config::Settings::open(
                    root.path(),
                    dokime_runtime::storage::sqlite::State::open(&root.path().join("data"), &Default::default()).await.unwrap(),
                )
                .await
                .unwrap(),
                Default::default(),
                285_212_672 as usize,
            );
            let error = store.stage(forged.bytes()).await.unwrap_err().to_string();
            assert!(error.contains(&format!("unsupported `{namespace}` schema {schema}")), "{error}");
            assert!(store.list().await.unwrap().is_empty());
        }
    }
}
