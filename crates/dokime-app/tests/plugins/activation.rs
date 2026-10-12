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
use tempfile::tempdir;
// Local Imports
use super::*;
use crate::support::plugins::fixture_bundle;
use dokime_app::Runtime;
use dokime_app::plugins::{PluginBundle, PluginLastBoot};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

pub(crate) async fn broken_installed_skipped() {
    let root = tempdir().unwrap();
    let store = crate::support::plugins::plugin_store(root.path());
    let fixture = fixture_bundle(root.path(), "fixture-suites");
    let bytes = std::fs::read(fixture.path).unwrap();
    let staged = store.stage(&bytes).unwrap();
    let installed = store
        .consent(staged.installation.install_id, &[], &Default::default())
        .unwrap();
    std::fs::write(
        store.path(installed.installation.install_id),
        b"damaged after consent",
    )
    .unwrap();

    let mut runtime = {
        let runtime_config = config(root.path());
        Runtime::start(runtime_config)
    }
    .await
    .expect("an invalid sideload must not fail the trusted runtime boot");

    let reloaded = crate::support::plugins::plugin_store(root.path());
    assert!(matches!(
        reloaded.list().unwrap()[0].installation.last_boot,
        Some(PluginLastBoot::Failed { .. })
    ));
    runtime.shutdown().await.unwrap();
}

pub(crate) async fn broken_configured_loud() {
    let root = tempdir().unwrap();
    let invalid = root.path().join("invalid-sidecar");
    std::fs::write(&invalid, b"not a sidecar").unwrap();

    let result = {
        let runtime_config = config(root.path()).with_plugins([PluginBundle::new(invalid)]);
        Runtime::start(runtime_config)
    }
    .await;

    assert!(result.is_err());
}

fn difference(left: &serde_json::Value, right: &serde_json::Value, path: &str) -> String {
    if let (Some(left), Some(right)) = (left.as_object(), right.as_object()) {
        for (key, value) in left {
            if Some(value) != right.get(key) {
                return difference(value, &right[key], &format!("{path}/{key}"));
            }
        }
    }
    if let (Some(left), Some(right)) = (left.as_array(), right.as_array()) {
        for (index, (left, right)) in left.iter().zip(right).enumerate() {
            if left != right {
                return difference(left, right, &format!("{path}/{index}"));
            }
        }
    }
    format!("{path}: {left} != {right}")
}

// multi executable failure leaves no live siblings or partial catalogs.

pub(crate) async fn multi_executable_partial() {
    let root = tempdir().unwrap();
    let suite = fixture_bundle(root.path(), "fixture-suites");
    let telemetry = fixture_bundle(root.path(), "fixture-telemetry");
    let first =
        harmos::plugin::Bundle::inspect(&std::fs::read(suite.path).unwrap(), Default::default())
            .unwrap();
    let second = harmos::plugin::Bundle::inspect(
        &std::fs::read(telemetry.path).unwrap(),
        Default::default(),
    )
    .unwrap();
    let bundle = harmos::plugin::Bundle::build(
        harmos::plugin::Identity {
            name: "runtime-multi".into(),
            ..first.manifest().identity.clone()
        },
        &[
            harmos::plugin::Source {
                path: "bin/suite",
                bytes: first.file(&first.manifest().files[0].path).unwrap(),
                executable: first.manifest().files[0].executable.clone(),
            },
            harmos::plugin::Source {
                path: "bin/telemetry",
                bytes: second.file(&second.manifest().files[0].path).unwrap(),
                executable: second.manifest().files[0].executable.clone(),
            },
        ],
        Default::default(),
    )
    .unwrap();
    let path = root.path().join("multi.dokime");
    std::fs::write(&path, bundle.bytes()).unwrap();
    let lifecycle = root.path().join("lifecycle");
    let input = PluginBundle::new(path)
        .configured("fixture-suites", serde_json::json!({"refuse":true}))
        .unwrap()
        .configured(
            "validation-telemetry",
            serde_json::json!({"lifecycle":lifecycle}),
        )
        .unwrap();
    let config = config(root.path());
    let failed = {
        let runtime_config = config.clone().with_plugins([input]);
        Runtime::start(runtime_config)
    }
    .await;
    assert!(failed.is_err());
    // Dependency-ordered startup can fail before an independent sibling starts.
    // Any sibling that did start must finish cleanup before the boot fails.
    if lifecycle.exists() {
        assert_eq!(std::fs::read(&lifecycle).unwrap(), b"finalized");
    }
    assert_eq!(
        std::fs::read_dir(root.path().join("application/cache/plugins"))
            .unwrap()
            .count(),
        0
    );
    let (mut recovered, _) = started(config).await;
    assert!(
        recovered
            .handle()
            .test_suites()
            .await
            .iter()
            .all(|suite| suite.source != "fixture-suites")
    );
    recovered.shutdown().await.unwrap();
}

pub(crate) async fn mismatched_unknown_initialize() {
    let root = tempdir().unwrap();
    let fixture = fixture_bundle(root.path(), "fixture-suites");
    let inspected =
        harmos::plugin::Bundle::inspect(&std::fs::read(&fixture.path).unwrap(), Default::default())
            .unwrap();
    let mut executable = inspected.manifest().files[0].executable.clone().unwrap();
    executable
        .registrations
        .push(harmos::sidecar::Registration {
            namespace: "fixture.unknown".into(),
            schema: 1,
            payload: br#"{"value":"frozen"}"#.to_vec(),
        });
    let changed = harmos::plugin::Bundle::build(
        inspected.manifest().identity.clone(),
        &[harmos::plugin::Source {
            path: "bin/fixture",
            bytes: inspected.file(&inspected.manifest().files[0].path).unwrap(),
            executable: Some(executable),
        }],
        Default::default(),
    )
    .unwrap();
    std::fs::write(&fixture.path, changed.bytes()).unwrap();
    let marker = root.path().join("initialized");
    let input = fixture
        .configured("fixture-suites", serde_json::json!({"marker":marker}))
        .unwrap();
    let result = {
        let runtime_config = config(root.path()).with_plugins([input]);
        Runtime::start(runtime_config)
    }
    .await;
    assert!(result.is_err());
    assert!(
        !marker.exists(),
        "catalog mismatch must precede authored initialization"
    );
}

// independently built mini bundle installs without a host import.

pub(crate) async fn independently_built_import() {
    let root = tempdir().unwrap();
    let bundle = crate::support::plugins::prepared_bundle("mini-sidecar");
    let bytes = std::fs::read(bundle)
        .expect("mise run build:plugins:dev builds the independent Mini bundle");
    let store = crate::support::plugins::plugin_store(root.path());
    let staged = store.stage(&bytes).unwrap();
    let id = staged.installation.install_id;
    store.consent(id, &[], &Default::default()).unwrap();
    let config = config(root.path());
    let (mut runtime, _) = started(config.clone()).await;
    assert!(
        runtime
            .handle()
            .test_suites()
            .await
            .iter()
            .any(|suite| suite.source == "mini-sidecar")
    );
    assert!(matches!(
        store.list().unwrap()[0].installation.last_boot,
        Some(PluginLastBoot::Loaded { .. })
    ));
    assert_eq!(std::fs::read(store.path(id)).unwrap(), bytes);
    assert_eq!(
        std::fs::read_dir(root.path().join("application/cache/plugins"))
            .unwrap()
            .count(),
        1
    );
    store.remove(id).unwrap();
    assert!(
        runtime
            .handle()
            .test_suites()
            .await
            .iter()
            .any(|suite| suite.source == "mini-sidecar")
    );
    runtime.shutdown().await.unwrap();
    // An installed bundle's extraction is kept at a stable, install-id-keyed
    // path that survives ordinary shutdown, so an unchanged bundle can skip
    // re-extracting it on the next boot (see `Extraction::Cached`). Removing
    // it from the store does not delete it immediately: the next boot's
    // `install_plugins` reclaims it once the id no longer appears among the
    // currently installed plugins (see `sweep_orphaned_extractions`).
    assert_eq!(
        std::fs::read_dir(root.path().join("application/cache/plugins"))
            .unwrap()
            .count(),
        1
    );
    let (mut restarted, _) = started(config).await;
    assert!(
        !restarted
            .handle()
            .test_suites()
            .await
            .iter()
            .any(|suite| suite.source == "mini-sidecar")
    );
    assert!(store.list().unwrap().is_empty());
    assert!(!store.path(id).exists());
    // The orphaned extraction is reclaimed by the boot that discovered the
    // plugin is no longer installed, not left behind indefinitely.
    assert_eq!(
        std::fs::read_dir(root.path().join("application/cache/plugins"))
            .unwrap()
            .count(),
        0
    );
    restarted.shutdown().await.unwrap();
}

// queued plugin changes preserve active catalog until each restart.

pub(crate) async fn queued_plugin_until() {
    let root = tempdir().unwrap();
    let config = config(root.path());
    let store = crate::support::plugins::plugin_store(root.path());
    let bytes = std::fs::read(fixture_bundle(root.path(), "fixture-suites").path).unwrap();
    let (mut first, _) = started(config.clone()).await;
    let staged = store.stage(&bytes).unwrap();
    let id = staged.installation.install_id;
    let consented = first.handle().consent_plugin(id).await.unwrap();
    let mut row = consented;
    first.handle().plugin_process_state(&mut row);
    assert!(!row.active);
    assert!(row.restart_required);
    assert!(
        first
            .handle()
            .test_suites()
            .await
            .iter()
            .all(|suite| suite.source != "fixture-suites")
    );
    first.shutdown().await.unwrap();

    let (mut active, _) = started(config.clone()).await;
    let mut row = store.list().unwrap().remove(0);
    active.handle().plugin_process_state(&mut row);
    assert!(row.active);
    assert!(!row.restart_required);
    store.set_enabled(id, false).unwrap();
    let mut row = store.list().unwrap().remove(0);
    active.handle().plugin_process_state(&mut row);
    assert!(row.active && row.restart_required);
    assert!(active.handle().test_suites().await.iter().any(|suite| {
        suite.source == "fixture-suites"
            && suite.suite_id == "fixture.checks"
            && suite
                .suite
                .test_case_templates
                .iter()
                .any(|template| template.metadata.identifier.as_str() == "fixture.smoke")
    }));
    active.shutdown().await.unwrap();

    let (mut disabled, _) = started(config.clone()).await;
    let mut row = store.list().unwrap().remove(0);
    disabled.handle().plugin_process_state(&mut row);
    assert!(!row.active && !row.restart_required);
    assert!(matches!(
        row.installation.last_boot,
        Some(PluginLastBoot::Loaded { .. })
    ));
    assert!(
        disabled
            .handle()
            .test_suites()
            .await
            .iter()
            .all(|suite| suite.source != "fixture-suites")
    );
    store.set_enabled(id, true).unwrap();
    let mut row = store.list().unwrap().remove(0);
    disabled.handle().plugin_process_state(&mut row);
    assert!(!row.active && row.restart_required);
    disabled.shutdown().await.unwrap();

    let (mut active, _) = started(config.clone()).await;
    store.remove(id).unwrap();
    let mut row = store.list().unwrap().remove(0);
    active.handle().plugin_process_state(&mut row);
    assert!(row.active && row.restart_required);
    assert!(store.path(id).is_file());
    assert!(
        active
            .handle()
            .test_suites()
            .await
            .iter()
            .any(|suite| suite.source == "fixture-suites")
    );
    active.shutdown().await.unwrap();
    let (mut removed, _) = started(config).await;
    assert!(store.list().unwrap().is_empty());
    assert!(!store.path(id).exists());
    assert!(
        removed
            .handle()
            .test_suites()
            .await
            .iter()
            .all(|suite| suite.source != "fixture-suites")
    );
    removed.shutdown().await.unwrap();
}

/* === Tests === */

pub(crate) fn fixture_catalog_repeatable() {
    let export = || {
        let executable = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target")
            .join(env!("DOKIME_NATIVE_TARGET"))
            .join("debug")
            .join(format!(
                "dokime-app-suite-sidecar-fixture{}",
                std::env::consts::EXE_SUFFIX,
            ));
        let output = std::process::Command::new(executable)
            .arg(harmos::sidecar::CATALOG_EXPORT_FLAG)
            .output()
            .expect("mise run fixtures:prepare prepares the native fixture");
        assert!(output.status.success());
        serde_json::from_slice::<Vec<harmos::sidecar::Registration>>(&output.stdout).unwrap()
    };
    let first = export();
    let second = export();
    for (left, right) in first.iter().zip(&second) {
        if left.payload != right.payload {
            let left: serde_json::Value = serde_json::from_slice(&left.payload).unwrap();
            let right: serde_json::Value = serde_json::from_slice(&right.payload).unwrap();
            panic!(
                "catalog export differs at {}",
                difference(&left, &right, "")
            );
        }
    }
    assert_eq!(first, second);
}
