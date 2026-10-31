/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Stories Over The Built OBC Plugin Bundle, Which Only The Acceptance Lane Receives.

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
    std::fs::read(crate::service_support::bundles::plugin_obc_bundle()).unwrap()
}

/* === Tests === */

#[tokio::test]
async fn external_analysis_reload() {
    let root = tempfile::tempdir().unwrap();
    let bundle = crate::service_support::bundles::plugin_obc_bundle();
    let host = crate::service_support::bundles::host_plugin_bundle();
    let config = crate::service_support::config(root.path())
        .with_plugins([dokime_runtime::plugins::PluginBundle::new(host), dokime_runtime::plugins::PluginBundle::new(bundle)]);
    let mut owner = dokime_runtime::Runtime::start(config).await.unwrap();
    let runtime = owner.handle();
    runtime.publish_telemetry_batch(crate::service_support::evidence::samples("host.wifi.scan", 100, None)).await.unwrap();
    runtime.storage().flush().await.unwrap();
    let mut request = crate::service_support::plots::wifi_request(None, Some(1));
    request.analysis = Some(dokime_runtime::analysis::Selection {
        id: "plugin_obc.analysis.ripple".into(),
        version: 1,
        parameters: Default::default(),
    });
    for reload in [false, true] {
        if reload {
            owner.reinitialize_plugin("plugin_obc").await.unwrap();
        }
        let definitions = runtime.analysis_definitions().await.unwrap();
        assert_eq!(definitions.iter().filter(|definition| definition.id == "plugin_obc.analysis.ripple").count(), 1);
        let envelopes = runtime
            .query_plot_payloads(vec![request.clone()], dokime_runtime::runtime::query::RuntimePlotQueryMode::Snapshot)
            .await
            .unwrap();
        let computed = envelopes[0].payload.computed.as_ref().unwrap();
        assert_eq!(computed.selection.id, "plugin_obc.analysis.ripple");
        assert_eq!(computed.series[0].input_samples, 100);
        assert_eq!(computed.series[0].output.outputs[0].values, [99.0]);
        assert_eq!(computed.series[0].output.outputs[0].unit, "count");
    }
    drop(runtime);
    owner.shutdown().await.unwrap();
}

// Frozen Catalog Rejects Undeclared, Reserved And Duplicate Operations.
#[tokio::test]
async fn frozen_catalog_multiply() {
    use harmos_packaging::bundle::{Bundle, Limits, Source};
    let bundle = Bundle::inspect(&fixture_bytes(), Limits::default()).unwrap();
    let file = &bundle.manifest().files[0];
    // The analysis claims `plugin_obc.analysis.ripple` first, so the status probe naming
    // it is the second claim.
    for (namespace, pointer, method, reason) in [
        ("dokime.settings", "/status_entrypoint/name", "obc.bench.missing", "undeclared"),
        ("dokime.settings", "/status_entrypoint/name", "harmos.initialize", "reserved"),
        ("dokime.settings", "/status_entrypoint/name", "plugin_obc.analysis.ripple", "more than once"),
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
                dokime_runtime::paths::ApplicationPaths::new(root.path()),
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
                    dokime_runtime::paths::ApplicationPaths::new(root.path()),
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
