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
use dokime_app::core::{Identifier, Metadata};
use dokime_app::domains::exports::storage::DiagnosticPathKey;
use dokime_app::{Runtime, RuntimeConfig};
use tempfile::tempdir;

use dokime_server::diagnostics_assembly::{
    DiagnosticCheckStatus, DiagnosticsSnapshot, StartupDoctorCheck, StartupDoctorCheckKey,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn storage_root_check(diagnostics: &DiagnosticsSnapshot) -> &StartupDoctorCheck {
    diagnostics
        .startup_doctor
        .checks
        .iter()
        .find(|check| check.key == StartupDoctorCheckKey::StorageRoot)
        .expect("the storage root is reported")
}

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
    let present = storage_root_check(&diagnostics);
    assert_eq!(present.status, DiagnosticCheckStatus::Ok);
    assert_eq!(present.message, "storage root exists");
    assert_eq!(
        present.detail.as_deref(),
        Some(root.path().to_string_lossy().as_ref())
    );
    let accepting = diagnostics
        .startup_doctor
        .checks
        .iter()
        .find(|check| check.key == StartupDoctorCheckKey::ShutdownState)
        .expect("the shutdown state is reported");
    assert_eq!(accepting.status, DiagnosticCheckStatus::Ok);
    assert_eq!(accepting.message, "runtime accepts commands");

    // An absent root under an existing parent warns that it will be created.
    let parent = tempdir().unwrap();
    let pending = DiagnosticsSnapshot::from_runtime(
        runtime.runtime_overview().await.unwrap(),
        &parent.path().join("absent"),
        &runtime.application_root(),
    );
    assert_eq!(
        pending.startup_doctor.status,
        DiagnosticCheckStatus::Warning
    );
    let created = storage_root_check(&pending);
    assert_eq!(created.status, DiagnosticCheckStatus::Warning);
    assert_eq!(
        created.message,
        "storage root will be created on first write"
    );

    // A root without a parent fails the doctor.
    let orphan = DiagnosticsSnapshot::from_runtime(
        runtime.runtime_overview().await.unwrap(),
        &parent.path().join("missing").join("absent"),
        &runtime.application_root(),
    );
    assert_eq!(orphan.startup_doctor.status, DiagnosticCheckStatus::Failed);
    let refused = storage_root_check(&orphan);
    assert_eq!(refused.status, DiagnosticCheckStatus::Failed);
    assert_eq!(refused.message, "storage root parent does not exist");

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
