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
use std::{collections::BTreeSet, fs, path::Path};
// Third Party Imports
use harmos_packaging::bundle::{Bundle, Executable, Facility, Identity, Limits, Source};
// Local Imports
use crate::service_support::bundles::bundle;
use dokime_runtime::config::Settings;
use dokime_runtime::plugins::{PluginError, PluginLastBoot, PluginState, PluginStore, inspect};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn store(root: &Path) -> PluginStore {
    PluginStore::new(Settings::load(root).unwrap())
}

fn settings_json(root: &Path) -> serde_json::Value {
    serde_json::from_slice(&fs::read(root.join("settings.json")).unwrap()).unwrap()
}

fn large_bundle(package: &str, owner: &str, version: &str, resource_bytes: usize) -> Vec<u8> {
    let identity =
        Identity { name: package.into(), version: version.parse().unwrap(), description: "A fixture.".into(), package: Default::default() };
    let declaration =
        serde_json::json!({"kind":"plugin","facility":"sidecar","id":owner,"version":version,"description":"A fixture.","config":"()"});
    let policy = harmos::operations::OperationPolicy {
        timeout: std::time::Duration::from_secs(30),
        attempt_timeout: std::time::Duration::from_secs(30),
        retries: harmos::operations::RetryPolicy::Never,
        restart: harmos::operations::RestartPolicy::Never,
    };
    let lifecycle = [
        harmos::operations::lifecycle::Descriptor::new(harmos::operations::lifecycle::Action::Initialize, policy).unwrap(),
        harmos::operations::lifecycle::Descriptor::new(harmos::operations::lifecycle::Action::Shutdown, policy).unwrap(),
    ];
    let contracts =
        serde_json::json!({"kind":"contracts","facility":"sidecar","owner":owner,"provides":[],"requires":[],"lifecycle":lifecycle});
    let restart = serde_json::json!({"kind":"restart","facility":"sidecar","owner":owner,"policy":{"credit":3,"sustained_health_ms":30000,"backoff_base_ms":250,"backoff_cap_ms":10000}});
    let publications = serde_json::json!({"kind":"publications","facility":"sidecar","owner":owner,"streams":[]});
    let observers = serde_json::json!({"kind":"observers","facility":"sidecar","owner":owner,"observers":[]});
    let executable = format!(
        "inert{prefix}{declaration}\0{prefix}{contracts}\0{prefix}{restart}\0{prefix}{publications}\0{prefix}{observers}\0",
        prefix = std::str::from_utf8(harmos::artifacts::PREFIX).unwrap()
    );
    let resource = vec![0xABu8; resource_bytes];
    Bundle::build(
        identity,
        &[
            Source {
                path: "bin/fixture",
                bytes: executable.as_bytes(),
                executable: Some(Executable {
                    facility: Facility::Sidecar,
                    target: env!("DOKIME_NATIVE_TARGET").into(),
                    registrations: vec![],
                }),
            },
            Source { path: "resources/payload.bin", bytes: &resource, executable: None },
        ],
        Limits::default(),
    )
    .unwrap()
    .bytes()
    .to_vec()
}

/* === Tests === */

#[test]
fn inspect_stage_bundle() {
    // A Bundle Past The Default Resource Limit Still Inspects And Stages.
    let bytes = large_bundle("fixture", "fixture", "1.0.0", 70 * 1024 * 1024);
    let inspected = std::time::Instant::now();
    inspect(&bytes).unwrap();
    eprintln!("Large Plugin Inspection: {:?}", inspected.elapsed());

    let root = tempfile::tempdir().unwrap();
    let store = store(root.path());
    let started = std::time::Instant::now();
    let staged = store.stage(&bytes).unwrap();
    eprintln!("Large Plugin Stage: {:?}", started.elapsed());
    let started = std::time::Instant::now();
    let queried = store.get(staged.installation.install_id).unwrap();
    eprintln!("Large Plugin Query: {:?}", started.elapsed());
    assert_eq!(queried.installation.sha256, staged.installation.sha256);
    assert!(queried.metadata.is_some());
    let started = std::time::Instant::now();
    let installed = store.consent(staged.installation.install_id, &[], &BTreeSet::new()).unwrap();
    eprintln!("Large Plugin Consent: {:?}", started.elapsed());
    assert_eq!(installed.installation.state, PluginState::Installed);
    assert_eq!(installed.installation.sha256, staged.installation.sha256);
    assert!(installed.installation.consented_at.is_some());
    assert_eq!(store.list().unwrap().len(), 1);
}

#[test]
fn consent_refusal_preserves_authorization() {
    let root = tempfile::tempdir().unwrap();
    let store = store(root.path());
    let first = store.stage(&bundle("fixture", "fixture", "1.0.0")).unwrap();
    store.consent(first.installation.install_id, &[], &BTreeSet::new()).unwrap();
    let staged = store.stage(&bundle("fixture", "fixture", "2.0.0")).unwrap();
    let before = settings_json(root.path());
    let result = store.consent_checked(staged.installation.install_id, &[], &BTreeSet::new(), |reviewed| {
        assert_eq!(reviewed.installation.sha256, staged.installation.sha256);
        assert!(reviewed.metadata.is_some());
        Err::<(), _>(PluginError::Invalid("disclosure refused".into()))
    });
    assert!(matches!(result, Err(PluginError::Invalid(message)) if message == "disclosure refused"));
    assert_eq!(settings_json(root.path()), before);
    let installed = store.consent(staged.installation.install_id, &[], &BTreeSet::new()).unwrap();
    assert_eq!(installed.installation.state, PluginState::Installed);
    assert_eq!(store.get(first.installation.install_id).unwrap().installation.state, PluginState::RemovalPending);
}

#[test]
fn immutable_executable_survive() {
    let root = tempfile::tempdir().unwrap();
    let store = store(root.path());
    let first = store.stage(&bundle("fixture", "fixture", "1.0.0")).unwrap();
    let first_id = first.installation.install_id;
    store.consent(first_id, &[], &BTreeSet::new()).unwrap();
    let original = fs::read(store.path(first_id)).unwrap();
    let second = store.stage(&bundle("fixture", "fixture", "2.0.0")).unwrap();
    let second_id = second.installation.install_id;
    store.consent(second_id, &[], &BTreeSet::new()).unwrap();
    store.consent(second_id, &[], &BTreeSet::new()).unwrap();
    assert_eq!(fs::read(store.path(first_id)).unwrap(), original);
    let rows = store.list().unwrap();
    assert_eq!(rows[0].installation.state, PluginState::RemovalPending);
    assert_eq!(rows[1].installation.state, PluginState::Installed);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(store.path(second_id)).unwrap().permissions().mode();
        assert_eq!(mode & 0o111, 0o111, "installed bundles are executable");
    }

    // Settings Hold Authorization, Never Author Metadata Or Paths
    let saved = settings_json(root.path());
    let entry = &saved["plugins"][first_id.to_string()];
    assert_eq!(entry["state"], "removal_pending");
    assert!(entry.get("metadata").is_none());
    assert!(entry.get("version").is_none());
    assert!(!saved.to_string().contains(".dokime"));
    assert!(
        fs::read_dir(store.root())
            .unwrap()
            .all(|entry| { entry.unwrap().path().extension().is_some_and(|extension| extension == "dokime") })
    );
    store.sweep().unwrap();
    assert!(!store.path(first_id).exists());
    assert_eq!(store.list().unwrap().len(), 1);
}

// consent rechecks staged collisions and host claims under the settings lock.

#[test]
fn consent_rechecks_host() {
    let root = tempfile::tempdir().unwrap();
    let left = store(root.path());
    let right = store(root.path());
    let a = left.stage(&bundle("left", "shared", "1.0.0")).unwrap().installation.install_id;
    let b = right.stage(&bundle("right", "shared", "1.0.0")).unwrap().installation.install_id;
    assert!(matches!(left.consent(a, &[], &BTreeSet::from(["shared".into()])), Err(PluginError::Collision(_))));
    left.consent(a, &[], &BTreeSet::new()).unwrap();
    assert!(matches!(right.consent(b, &[], &BTreeSet::new()), Err(PluginError::Collision(_))));
    assert_eq!(right.list().unwrap().len(), 2);
}

#[test]
fn unavailable_bytes_metadata() {
    let root = tempfile::tempdir().unwrap();
    let store = store(root.path());
    let id = store.stage(&bundle("fixture", "fixture", "1.0.0")).unwrap().installation.install_id;
    fs::write(store.path(id), bundle("untrusted", "other", "2.0.0")).unwrap();
    let entry = store.list().unwrap().remove(0);
    assert_eq!(entry.installation.install_id, id);
    assert!(entry.metadata.is_none());
    assert!(entry.error.is_some());
    assert!(matches!(store.consent(id, &[], &BTreeSet::new()), Err(PluginError::Changed(_))));
    fs::remove_file(store.path(id)).unwrap();
    assert!(store.list().unwrap()[0].metadata.is_none());
    store.remove(id).unwrap();
    assert!(store.list().unwrap().is_empty());
}

// unauthorized files are swept and a corrupt settings file is never replaced.

#[test]
fn unauthorized_files_kept() {
    let root = tempfile::tempdir().unwrap();
    let plugins = root.path().join("plugins");
    fs::create_dir_all(&plugins).unwrap();
    fs::write(plugins.join("manual.dokime"), bundle("manual", "manual", "1.0.0")).unwrap();
    fs::write(plugins.join(".tmpABC123"), b"interrupted stage").unwrap();
    let store = store(root.path());
    assert!(store.list().unwrap().is_empty());
    assert!(store.installed().unwrap().is_empty());
    store.sweep().unwrap();
    assert_eq!(fs::read_dir(&plugins).unwrap().count(), 0);

    fs::write(root.path().join("settings.json"), b"{").unwrap();
    assert!(Settings::load(root.path()).is_err());
    assert_eq!(fs::read(root.path().join("settings.json")).unwrap(), b"{");
}

#[cfg(unix)]
#[test]
fn failed_settings_bytes() {
    use std::os::unix::fs::PermissionsExt;

    let root = tempfile::tempdir().unwrap();
    let store = store(root.path());
    let id = store.stage(&bundle("fixture", "fixture", "1.0.0")).unwrap().installation.install_id;
    let bytes = fs::read(store.path(id)).unwrap();
    let before = fs::read(root.path().join("settings.json")).unwrap();

    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o555)).unwrap();
    let staged = store.stage(&bundle("fixture", "fixture", "2.0.0"));
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o755)).unwrap();

    assert!(staged.is_err());
    assert_eq!(fs::read(store.path(id)).unwrap(), bytes);
    assert_eq!(fs::read(root.path().join("settings.json")).unwrap(), before);
    assert_eq!(store.list().unwrap().len(), 1);
    store.sweep().unwrap();
    assert_eq!(fs::read_dir(store.root()).unwrap().count(), 1);
}

#[test]
fn failed_cleanup_retry() {
    let root = tempfile::tempdir().unwrap();
    let store = store(root.path());
    let id = store.stage(&bundle("fixture", "fixture", "1.0.0")).unwrap().installation.install_id;
    store.consent(id, &[], &BTreeSet::new()).unwrap();
    store.remove(id).unwrap();
    fs::remove_file(store.path(id)).unwrap();
    fs::create_dir(store.path(id)).unwrap();
    store.sweep().unwrap();
    let row = store.list().unwrap().remove(0);
    assert_eq!(row.installation.state, PluginState::RemovalPending);
    assert!(matches!(row.installation.last_boot, Some(PluginLastBoot::Failed { .. })));
    fs::remove_dir(store.path(id)).unwrap();
    store.sweep().unwrap();
    assert!(store.list().unwrap().is_empty());
}

// removal pending keeps its meaning when settings are read again after restart.

#[test]
fn removal_pending_sweep() {
    let root = tempfile::tempdir().unwrap();
    let id = {
        let store = store(root.path());
        let id = store.stage(&bundle("fixture", "fixture", "1.0.0")).unwrap().installation.install_id;
        store.consent(id, &[], &BTreeSet::new()).unwrap();
        store.remove(id).unwrap();
        assert!(store.path(id).is_file(), "consented bytes stop next boot");
        id
    };

    let restarted = store(root.path());
    assert_eq!(restarted.get(id).unwrap().installation.state, PluginState::RemovalPending);
    assert!(restarted.installed().unwrap().is_empty());
    restarted.sweep().unwrap();
    assert!(!restarted.path(id).exists());
    assert!(restarted.list().unwrap().is_empty());
}

// discarding unconsented replacement is immediate and failed cleanup can retry.

#[test]
fn discarding_unconsented_failed() {
    let root = tempfile::tempdir().unwrap();
    let store = store(root.path());
    let original = store.stage(&bundle("fixture", "fixture", "1.0.0")).unwrap().installation.install_id;
    store.consent(original, &[], &BTreeSet::new()).unwrap();
    store.record_boot(original, None).unwrap();
    let before = fs::read(store.path(original)).unwrap();
    let staged = store.stage(&bundle("fixture", "fixture", "2.0.0")).unwrap().installation.install_id;
    store.remove(staged).unwrap();
    assert!(!store.path(staged).exists());
    let rows = store.list().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].installation.install_id, original);
    assert!(matches!(rows[0].installation.last_boot, Some(PluginLastBoot::Loaded { .. })));
    assert_eq!(fs::read(store.path(original)).unwrap(), before);

    let blocked = store.stage(&bundle("other", "other", "1.0.0")).unwrap().installation.install_id;
    fs::remove_file(store.path(blocked)).unwrap();
    fs::create_dir(store.path(blocked)).unwrap();
    store.remove(blocked).unwrap();
    let failed = store.get(blocked).unwrap();
    assert_eq!(failed.installation.state, PluginState::RemovalPending);
    assert!(failed.installation.consented_at.is_none());
    assert!(matches!(failed.installation.last_boot, Some(PluginLastBoot::Failed { .. })));
    assert!(store.consent(blocked, &[], &BTreeSet::new()).is_err());
    fs::remove_dir(store.path(blocked)).unwrap();
    store.remove(blocked).unwrap();
    assert_eq!(store.list().unwrap().len(), 1);
}

// disabling retains authorized bytes and reservation for next boot.

#[test]
fn disabling_retains_next() {
    let root = tempfile::tempdir().unwrap();
    let store = store(root.path());
    let row = store.stage(&bundle("fixture", "fixture", "1.0.0")).unwrap();
    let id = row.installation.install_id;
    assert!(store.set_enabled(id, true).is_err());
    store.consent(id, &[], &BTreeSet::new()).unwrap();
    store.set_enabled(id, false).unwrap();
    assert!(store.installed().unwrap().is_empty());
    assert!(store.path(id).is_file());
    assert_eq!(store.list().unwrap()[0].installation.state, PluginState::Disabled);
    let collision = store.stage(&bundle("other", "fixture", "1.0.0")).unwrap();
    assert!(matches!(store.consent(collision.installation.install_id, &[], &BTreeSet::new()), Err(PluginError::Collision(_))));
    store.set_enabled(id, true).unwrap();
    assert_eq!(store.installed().unwrap().len(), 1);
    store.remove(id).unwrap();
    assert!(store.set_enabled(id, true).is_err());
}

// bounded pages preserve order and refuse changed or unknown cursors.

#[test]
fn bounded_pages_unknown() {
    let root = tempfile::tempdir().unwrap();
    let store = store(root.path());
    let ids = (0..3)
        .map(|i| store.stage(&bundle(&format!("package{i}"), &format!("owner{i}"), "1.0.0")).unwrap().installation.install_id)
        .collect::<Vec<_>>();
    let first = store.page(None, 1).unwrap();
    assert_eq!(first.total, 3);
    assert_eq!(first.entries[0].installation.install_id, ids[0]);
    let second = store.page(Some((first.next.unwrap(), &first.manifest_sha256)), 1).unwrap();
    assert_eq!(second.entries[0].installation.install_id, ids[1]);
    assert_eq!(store.get(ids[2]).unwrap().installation.install_id, ids[2]);
    assert!(matches!(store.page(Some((uuid::Uuid::from_u128(7), &first.manifest_sha256)), 1), Err(PluginError::Cursor)));
    store.remove(ids[2]).unwrap();
    assert!(matches!(store.page(Some((first.next.unwrap(), &first.manifest_sha256)), 1), Err(PluginError::Cursor)));
    assert!(matches!(store.page(None, 101), Err(PluginError::Limit)));
}

// Expiry decisions are unit-tested at the exact boundary; the sweep applies them.

#[test]
fn sweep_expires_cleanup() {
    let root = tempfile::tempdir().unwrap();
    let store = store(root.path());
    let fresh = store.stage(&bundle("fresh", "fresh", "1.0.0")).unwrap();
    let installed = store.stage(&bundle("kept", "kept", "1.0.0")).unwrap();
    let expired = store.stage(&bundle("old", "old", "1.0.0")).unwrap();
    let blocked = store.stage(&bundle("stuck", "stuck", "1.0.0")).unwrap();
    let [fresh, installed, expired, blocked] = [fresh, installed, expired, blocked].map(|entry| entry.installation.install_id);
    store.consent(installed, &[], &BTreeSet::new()).unwrap();
    fs::remove_file(store.path(blocked)).unwrap();
    fs::create_dir(store.path(blocked)).unwrap();

    // Age Two Stages Past Their Expiry
    let mut saved = settings_json(root.path());
    for id in [expired, blocked] {
        saved["plugins"][id.to_string()]["expires_at"] = "2000-01-01T00:00:00Z".into();
    }
    fs::write(root.path().join("settings.json"), serde_json::to_vec(&saved).unwrap()).unwrap();

    let store = self::store(root.path());
    store.sweep().unwrap();
    assert!(!store.path(expired).exists());
    assert!(store.path(fresh).is_file());
    assert!(store.path(installed).is_file());
    assert_eq!(store.list().unwrap().len(), 3);
    assert_eq!(store.get(fresh).unwrap().installation.state, PluginState::Staged);
    let stuck = store.get(blocked).unwrap().installation;
    assert_eq!(stuck.state, PluginState::RemovalPending);
    assert!(matches!(stuck.last_boot, Some(PluginLastBoot::Failed { .. })));
    fs::remove_dir(store.path(blocked)).unwrap();
    store.sweep().unwrap();
    assert_eq!(store.list().unwrap().len(), 2);
}

#[tokio::test]
async fn startup_reclaims_staging() {
    let root = tempfile::tempdir().unwrap();
    let config = crate::service_support::config(root.path());
    let extraction = config.application_root.join("cache/plugins");
    let orphan = extraction.join(uuid::Uuid::from_u128(2).to_string());
    let staging = extraction.join("harmos-plugin-not-a-uuid");
    fs::create_dir_all(&orphan).unwrap();
    fs::create_dir_all(&staging).unwrap();
    fs::write(orphan.join("stale"), b"orphaned bytes").unwrap();
    fs::write(staging.join("canary"), b"active staging").unwrap();

    let mut runtime = dokime_runtime::Runtime::start(config).await.unwrap();

    assert!(!orphan.exists());
    assert_eq!(fs::read(staging.join("canary")).unwrap(), b"active staging");
    runtime.shutdown().await.unwrap();
}

#[test]
fn app_roots_independent() {
    let root = tempfile::tempdir().unwrap();
    let first = store(&root.path().join("one"));
    let a = first.stage(&bundle("one", "one", "1.0.0")).unwrap().installation.install_id;
    let second = store(&root.path().join("two"));
    let b = second.stage(&bundle("two", "two", "1.0.0")).unwrap().installation.install_id;
    let ids = |store: &PluginStore| store.list().unwrap().iter().map(|entry| entry.installation.install_id).collect::<Vec<_>>();
    assert_eq!(ids(&first), [a]);
    assert_eq!(ids(&second), [b]);
}
