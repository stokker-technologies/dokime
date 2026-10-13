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
use dokime_app::capabilities::exports::storage::DiagnosticPathKey;
use dokime_app::core::{Identifier, Metadata};
use dokime_app::{Runtime, RuntimeConfig};
use tempfile::tempdir;

use dokime_server::diagnostics_assembly::{DiagnosticCheckStatus, DiagnosticsSnapshot};

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

pub(crate) async fn assembles_counts_runtime() {
    let root = tempdir().unwrap();
    let mut runtime_owner = {
        let runtime_config = RuntimeConfig::new(
            root.path(),
            Metadata::new(Identifier::new("runtime.hub").unwrap(), "Runtime Hub", None),
        );
        Runtime::start(runtime_config)
    }
    .await
    .unwrap();
    let runtime = runtime_owner.handle();

    let diagnostics = DiagnosticsSnapshot::from_runtime(
        runtime.runtime_overview().await.unwrap(),
        root.path(),
        &runtime.application_root(),
    );

    // Counts derive from the authoritative runtime overview.
    assert!(!diagnostics.identity.node.node_id.as_str().is_empty());
    assert_eq!(
        diagnostics.counts.installed_plugin_errors,
        diagnostics.installed_plugin_errors.len()
    );

    // The startup-doctor passes on a healthy, present storage root.
    assert_eq!(diagnostics.startup_doctor.status, DiagnosticCheckStatus::Ok);
    let baseline = diagnostics
        .startup_doctor
        .checks
        .iter()
        .find(|check| check.message == "storage baseline is compatible")
        .expect("an existing root reports its baseline");
    assert_eq!(baseline.status, DiagnosticCheckStatus::Ok);
    assert_eq!(
        baseline.detail.as_deref(),
        Some("schema version 1; data format dokime.sqlite.storage.v1")
    );

    // A root from another baseline fails the doctor and names the mismatch.
    let foreign = tempdir().unwrap();
    dokime::storage::sqlite::initialize_sqlite().unwrap();
    dokime::storage::sqlite::rusqlite::Connection::open(foreign.path().join("local.sqlite"))
        .unwrap()
        .execute_batch("PRAGMA user_version=7; CREATE TABLE foreign_evidence(payload BLOB);")
        .unwrap();
    let refused = DiagnosticsSnapshot::from_runtime(
        runtime.runtime_overview().await.unwrap(),
        foreign.path(),
        &runtime.application_root(),
    );
    assert_eq!(refused.startup_doctor.status, DiagnosticCheckStatus::Failed);
    let baseline = refused
        .startup_doctor
        .checks
        .iter()
        .find(|check| check.status == DiagnosticCheckStatus::Failed)
        .expect("the incompatible baseline is reported");
    assert_eq!(
        baseline.message,
        "storage baseline is incompatible with this build"
    );
    assert!(
        baseline
            .detail
            .as_deref()
            .is_some_and(|detail| detail.contains("schema version 7")),
        "{:?}",
        baseline.detail
    );

    // An absent root has no baseline row to report.
    let absent = DiagnosticsSnapshot::from_runtime(
        runtime.runtime_overview().await.unwrap(),
        &foreign.path().join("absent"),
        &runtime.application_root(),
    );
    assert!(
        absent
            .startup_doctor
            .checks
            .iter()
            .all(|check| !check.message.starts_with("storage baseline"))
    );

    // The bundle carries the runtime's storage-layout contract.
    assert_eq!(
        diagnostics.storage_root,
        root.path().to_string_lossy().into_owned()
    );
    let hub_log = diagnostics
        .path_conventions
        .iter()
        .find(|path| path.key == DiagnosticPathKey::HubLog)
        .expect("the hub log convention is present");
    assert_eq!(hub_log.relative_path, "logs/dokime.jsonl");
    assert!(
        hub_log
            .absolute_path
            .as_ref()
            .unwrap()
            .starts_with(&runtime.application_root().to_string_lossy().into_owned()),
        "logs belong to the application root"
    );

    runtime_owner.shutdown().await.unwrap();
}

fn fresh_runtime(
    multithread: bool,
    threads: Option<usize>,
    paused: bool,
) -> tokio::runtime::Runtime {
    let _ = multithread;
    let mut builder = tokio::runtime::Builder::new_current_thread();
    builder.enable_all();
    if let Some(threads) = threads {
        builder.worker_threads(threads);
    }
    let _ = paused;
    builder.build().expect("scenario runtime")
}

#[test]
fn domain_contract() {
    {
        eprintln!("Scenario: assembles_counts_runtime");
        run_scenario(assembles_counts_runtime, false, None, false);
    }
}

#[inline(never)]
fn run_scenario<F: std::future::Future>(
    factory: impl FnOnce() -> F,
    multithread: bool,
    threads: Option<usize>,
    paused: bool,
) -> F::Output {
    fresh_runtime(multithread, threads, paused).block_on(Box::pin(factory()))
}
