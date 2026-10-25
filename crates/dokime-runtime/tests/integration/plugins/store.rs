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
        timeout: std::time::Duration::from_secs(120),
        attempt_timeout: std::time::Duration::from_secs(120),
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
fn direct_installation() {
    let bytes = large_bundle("fixture", "fixture", "1.0.0", 70 * 1024 * 1024);
    let root = tempfile::tempdir().unwrap();
    let store = store(root.path());
    let installed = store.install(&bytes, &[], &BTreeSet::new()).unwrap();
    assert_eq!(installed.installation.state, PluginState::Installed);
    assert!(installed.restart_required);
    assert!(!installed.active);
    let queried = store.get(installed.installation.install_id).unwrap();
    assert_eq!(queried.installation.sha256, installed.installation.sha256);
    assert!(queried.metadata.is_some());
    assert_eq!(store.installed().unwrap().len(), 1);
}

#[test]
fn replacement_preserves_bytes() {
    let root = tempfile::tempdir().unwrap();
    let store = store(root.path());
    let first = store.install(&bundle("fixture", "fixture", "1.0.0"), &[], &BTreeSet::new()).unwrap().installation.install_id;
    let original = fs::read(store.path(first)).unwrap();
    let second = store.install(&bundle("fixture", "fixture", "2.0.0"), &[], &BTreeSet::new()).unwrap().installation.install_id;
    assert_eq!(fs::read(store.path(first)).unwrap(), original);
    assert_eq!(store.get(first).unwrap().installation.state, PluginState::RemovalPending);
    assert_eq!(store.get(second).unwrap().installation.state, PluginState::Installed);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(fs::metadata(store.path(second)).unwrap().permissions().mode() & 0o111, 0o111);
    }
    let saved = settings_json(root.path());
    assert!(saved["plugins"][second.to_string()].get("metadata").is_none());
    assert!(saved["plugins"][second.to_string()].get("installed_at").is_some());
    assert!(!saved.to_string().contains(".dokime"));
    store.sweep().unwrap();
    assert!(!store.path(first).exists());
    assert!(store.path(second).is_file());
    assert_eq!(store.list().unwrap().len(), 1);
}

#[test]
fn collisions_preserve_installations() {
    let root = tempfile::tempdir().unwrap();
    let left = store(root.path());
    let bytes = bundle("left", "shared", "1.0.0");
    assert!(matches!(left.install(&bytes, &[], &BTreeSet::from(["shared".into()])), Err(PluginError::Collision(_))));
    assert!(left.list().unwrap().is_empty());
    let installed = left.install(&bytes, &[], &BTreeSet::new()).unwrap();
    let right = store(root.path());
    let before = settings_json(root.path());
    assert!(matches!(right.install(&bundle("right", "shared", "1.0.0"), &[], &BTreeSet::new()), Err(PluginError::Collision(_))));
    let configured = inspect(&bundle("configured", "external", "1.0.0")).unwrap().1;
    assert!(matches!(
        right.install(&bundle("configured", "unique", "2.0.0"), &[configured], &BTreeSet::new()),
        Err(PluginError::Collision(_))
    ));
    assert!(right.install(b"invalid archive", &[], &BTreeSet::new()).is_err());
    assert_eq!(settings_json(root.path()), before);
    assert_eq!(right.list().unwrap().len(), 1);
    assert_eq!(fs::read_dir(left.root()).unwrap().count(), 1);
    assert_eq!(right.get(installed.installation.install_id).unwrap().installation.state, PluginState::Installed);
}

#[test]
fn refused_result_preserved() {
    let root = tempfile::tempdir().unwrap();
    let store = store(root.path());
    let first = store.install(&bundle("fixture", "fixture", "1.0.0"), &[], &BTreeSet::new()).unwrap().installation.install_id;
    let original = fs::read(store.path(first)).unwrap();
    let before = settings_json(root.path());
    let mut checked = false;
    let result = store.install_checked(&bundle("fixture", "fixture", "2.0.0"), &[], &BTreeSet::new(), |entry| {
        checked = true;
        assert_eq!(entry.installation.state, PluginState::Installed);
        Err(PluginError::ResultLimit)
    });
    assert!(checked);
    assert!(matches!(result, Err(PluginError::ResultLimit)));
    assert_eq!(settings_json(root.path()), before);
    assert_eq!(fs::read(store.path(first)).unwrap(), original);
    assert_eq!(fs::read_dir(store.root()).unwrap().count(), 1);
}

#[test]
fn unavailable_bytes_metadata() {
    let root = tempfile::tempdir().unwrap();
    let store = store(root.path());
    let id = store.install(&bundle("fixture", "fixture", "1.0.0"), &[], &BTreeSet::new()).unwrap().installation.install_id;
    fs::write(store.path(id), bundle("untrusted", "other", "2.0.0")).unwrap();
    let entry = store.get(id).unwrap();
    assert!(entry.metadata.is_none());
    assert!(entry.error.is_some());
    let before = settings_json(root.path());
    assert!(matches!(store.install(&bundle("fixture", "fixture", "2.0.0"), &[], &BTreeSet::new()), Err(PluginError::Changed(_))));
    assert_eq!(settings_json(root.path()), before);
    store.remove(id).unwrap();
    assert_eq!(store.get(id).unwrap().installation.state, PluginState::RemovalPending);
    store.sweep().unwrap();
    assert!(store.list().unwrap().is_empty());
}

#[test]
fn installed_survive_cleanup() {
    let root = tempfile::tempdir().unwrap();
    let store = store(root.path());
    let installed = store.install(&bundle("fixture", "fixture", "1.0.0"), &[], &BTreeSet::new()).unwrap().installation.install_id;
    let bytes = fs::read(store.path(installed)).unwrap();
    fs::write(store.root().join("orphan.dokime"), b"unreferenced installation").unwrap();
    store.sweep().unwrap();
    assert_eq!(fs::read_dir(store.root()).unwrap().count(), 1);
    assert_eq!(fs::read(store.path(installed)).unwrap(), bytes);
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
    let id = store.install(&bundle("fixture", "fixture", "1.0.0"), &[], &BTreeSet::new()).unwrap().installation.install_id;
    let bytes = fs::read(store.path(id)).unwrap();
    let before = fs::read(root.path().join("settings.json")).unwrap();
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o555)).unwrap();
    let result = store.install(&bundle("fixture", "fixture", "2.0.0"), &[], &BTreeSet::new());
    fs::set_permissions(root.path(), fs::Permissions::from_mode(0o755)).unwrap();
    assert!(result.is_err());
    assert_eq!(fs::read(store.path(id)).unwrap(), bytes);
    assert_eq!(fs::read(root.path().join("settings.json")).unwrap(), before);
    store.sweep().unwrap();
    assert_eq!(fs::read_dir(store.root()).unwrap().count(), 1);
}

#[test]
fn removal_cleanup_retry() {
    let root = tempfile::tempdir().unwrap();
    let store = store(root.path());
    let id = store.install(&bundle("fixture", "fixture", "1.0.0"), &[], &BTreeSet::new()).unwrap().installation.install_id;
    store.remove(id).unwrap();
    assert!(store.path(id).is_file());
    fs::remove_file(store.path(id)).unwrap();
    fs::create_dir(store.path(id)).unwrap();
    store.sweep().unwrap();
    let row = store.get(id).unwrap();
    assert_eq!(row.installation.state, PluginState::RemovalPending);
    assert!(matches!(row.installation.last_boot, Some(PluginLastBoot::Failed { .. })));
    fs::remove_dir(store.path(id)).unwrap();
    let restarted = self::store(root.path());
    restarted.sweep().unwrap();
    assert!(restarted.list().unwrap().is_empty());
}

#[test]
fn disabled_reserves_owners() {
    let root = tempfile::tempdir().unwrap();
    let store = store(root.path());
    let id = store.install(&bundle("fixture", "fixture", "1.0.0"), &[], &BTreeSet::new()).unwrap().installation.install_id;
    store.set_enabled(id, false).unwrap();
    assert!(store.installed().unwrap().is_empty());
    assert!(store.path(id).is_file());
    assert!(matches!(store.install(&bundle("other", "fixture", "1.0.0"), &[], &BTreeSet::new()), Err(PluginError::Collision(_))));
    store.set_enabled(id, true).unwrap();
    assert_eq!(store.installed().unwrap().len(), 1);
    store.remove(id).unwrap();
    assert!(store.set_enabled(id, true).is_err());
}

#[test]
fn bounded_pages_unknown() {
    let root = tempfile::tempdir().unwrap();
    let store = store(root.path());
    let ids = (0..3)
        .map(|i| {
            store
                .install(&bundle(&format!("package{i}"), &format!("owner{i}"), "1.0.0"), &[], &BTreeSet::new())
                .unwrap()
                .installation
                .install_id
        })
        .collect::<Vec<_>>();
    let first = store.page(None, 1).unwrap();
    assert_eq!(first.total, 3);
    assert_eq!(first.entries[0].installation.install_id, ids[0]);
    let second = store.page(Some((first.next.unwrap(), &first.manifest_sha256)), 1).unwrap();
    assert_eq!(second.entries[0].installation.install_id, ids[1]);
    assert!(matches!(store.page(Some((uuid::Uuid::from_u128(7), &first.manifest_sha256)), 1), Err(PluginError::Cursor)));
    store.remove(ids[2]).unwrap();
    assert!(matches!(store.page(Some((first.next.unwrap(), &first.manifest_sha256)), 1), Err(PluginError::Cursor)));
    assert!(matches!(store.page(None, 101), Err(PluginError::Limit)));
}

#[test]
fn obsolete_settings_preserved() {
    let root = tempfile::tempdir().unwrap();
    let store = store(root.path());
    let id = store.install(&bundle("fixture", "fixture", "1.0.0"), &[], &BTreeSet::new()).unwrap().installation.install_id;
    let mut saved = settings_json(root.path());
    let entry = saved["plugins"][id.to_string()].as_object_mut().unwrap();
    let installed_at = entry.remove("installed_at").unwrap();
    entry.insert("staged_at".into(), installed_at);
    entry.insert("state".into(), "staged".into());
    entry.insert("consented_at".into(), serde_json::Value::Null);
    let bytes = serde_json::to_vec(&saved).unwrap();
    fs::write(root.path().join("settings.json"), &bytes).unwrap();
    assert!(Settings::load(root.path()).is_err());
    assert_eq!(fs::read(root.path().join("settings.json")).unwrap(), bytes);
}

#[tokio::test]
async fn startup_reclaims_extraction() {
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
    let a = first.install(&bundle("one", "one", "1.0.0"), &[], &BTreeSet::new()).unwrap().installation.install_id;
    let second = store(&root.path().join("two"));
    let b = second.install(&bundle("two", "two", "1.0.0"), &[], &BTreeSet::new()).unwrap().installation.install_id;
    assert_eq!(first.list().unwrap()[0].installation.install_id, a);
    assert_eq!(second.list().unwrap()[0].installation.install_id, b);
}
