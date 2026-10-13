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
use dokime_app::plugins::PluginStore;

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
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/fixtures/dokime-route-plugin/dokime-route-plugin.dokime"),
    )
    .expect("run mise run build:plugins:dev to build the standalone fixture")
}

/* === Tests === */
// frozen catalog cannot bind missing reserved or multiply claimed routes.

pub(crate) fn frozen_catalog_multiply() {
    use harmos::plugin::bundle::{Bundle, Limits, Source};
    let bundle = Bundle::inspect(&fixture_bytes(), Limits::default()).unwrap();
    let file = &bundle.manifest().files[0];
    for (namespace, pointer, method, reason) in [
        (
            "dokime.options",
            "/0/source/entrypoint/name",
            "route-fixture.missing",
            "undeclared",
        ),
        (
            "dokime.settings",
            "/status_entrypoint/name",
            "route-fixture.missing",
            "undeclared",
        ),
        (
            "dokime.settings",
            "/status_entrypoint/name",
            "harmos.initialize",
            "reserved",
        ),
        (
            "dokime.settings",
            "/status_entrypoint/name",
            "route-fixture.list_choices",
            "more than once",
        ),
    ] {
        let mut executable = file.executable.clone().unwrap();
        let registration = executable
            .registrations
            .iter_mut()
            .find(|entry| entry.namespace == namespace)
            .unwrap();
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
        let store = PluginStore::new(dokime_app::config::Settings::load(root.path()).unwrap());
        assert!(
            store
                .stage(forged.bytes())
                .unwrap_err()
                .to_string()
                .contains(reason)
        );
        assert!(store.list().unwrap().is_empty());
    }
}

// legacy and unknown domain carriers are refused before admission.

pub(crate) fn legacy_unknown_refused() {
    use harmos::plugin::bundle::{Bundle, Limits, Source};
    let bundle = Bundle::inspect(&fixture_bytes(), Limits::default()).unwrap();
    let file = &bundle.manifest().files[0];
    for namespace in ["dokime.options", "dokime.settings"] {
        for schema in [1, 4] {
            let mut executable = file.executable.clone().unwrap();
            let registration = executable
                .registrations
                .iter_mut()
                .find(|entry| entry.namespace == namespace)
                .unwrap();
            assert_eq!(
                registration.schema,
                dokime::plugin::registration::domain_schema(namespace)
            );
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
            let store = PluginStore::new(dokime_app::config::Settings::load(root.path()).unwrap());
            let error = store.stage(forged.bytes()).unwrap_err().to_string();
            assert!(
                error.contains(&format!("unsupported `{namespace}` schema {schema}")),
                "{error}"
            );
            assert!(store.list().unwrap().is_empty());
        }
    }
}
