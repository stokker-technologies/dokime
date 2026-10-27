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
async fn store(root: &Path) -> (PluginStore, dokime_runtime::storage::sqlite::State) {
    let database = dokime_runtime::storage::sqlite::State::open(&root.join("data")).await.unwrap();
    let settings = Settings::open(root, database.clone()).await.unwrap();
    (PluginStore::with_archive_limit(settings, Default::default(), 285_212_672), database)
}

fn saved_metadata(root: &Path) -> serde_json::Value {
    let connection =
        rusqlite::Connection::open_with_flags(root.join("data/local.sqlite"), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let body: Vec<u8> = connection.query_row("SELECT body FROM application_metadata WHERE id=1", [], |row| row.get(0)).unwrap();
    serde_json::from_slice(&body).unwrap()
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

#[tokio::test]
async fn direct_installation() {
    let bytes = large_bundle("fixture", "fixture", "1.0.0", 70 * 1024 * 1024);
    let root = tempfile::tempdir().unwrap();
    let (store, store_database) = store(root.path()).await;
    inspect(&bytes, &Default::default()).unwrap();
    let limited_settings = Settings::open(root.path(), store_database.clone()).await.unwrap();
    let limited = PluginStore::with_archive_limit(limited_settings, Default::default(), bytes.len() - 1);
    assert!(limited.install(&bytes, &[], &BTreeSet::new()).await.is_err());
    assert!(store.list().await.unwrap().is_empty());
    let installed = store.install(&bytes, &[], &BTreeSet::new()).await.unwrap();
    assert_eq!(installed.installation.state, PluginState::Installed);
    assert!(installed.restart_required);
    assert!(!installed.active);
    let queried = store.get(installed.installation.install_id).await.unwrap();
    assert_eq!(queried.installation.sha256, installed.installation.sha256);
    assert!(queried.metadata.is_some());
    assert_eq!(store.installed().await.unwrap().len(), 1);
    store_database.close().await.unwrap();
}

#[tokio::test]
async fn replacement_preserves_bytes() {
    let root = tempfile::tempdir().unwrap();
    let (store, store_database) = store(root.path()).await;
    let first = store.install(&bundle("fixture", "fixture", "1.0.0"), &[], &BTreeSet::new()).await.unwrap().installation.install_id;
    let original = fs::read(store.path(first)).unwrap();
    let second = store.install(&bundle("fixture", "fixture", "2.0.0"), &[], &BTreeSet::new()).await.unwrap().installation.install_id;
    assert_eq!(fs::read(store.path(first)).unwrap(), original);
    assert_eq!(store.get(first).await.unwrap().installation.state, PluginState::RemovalPending);
    assert_eq!(store.get(second).await.unwrap().installation.state, PluginState::Installed);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(fs::metadata(store.path(second)).unwrap().permissions().mode() & 0o111, 0o111);
    }
    let saved = saved_metadata(root.path());
    assert!(saved["plugins"][second.to_string()].get("metadata").is_none());
    assert!(saved["plugins"][second.to_string()].get("installed_at").is_some());
    assert!(!saved.to_string().contains(".dokime"));
    store.sweep().await.unwrap();
    assert!(!store.path(first).exists());
    assert!(store.path(second).is_file());
    assert_eq!(store.list().await.unwrap().len(), 1);
    store_database.close().await.unwrap();
}

#[tokio::test]
async fn collisions_preserve_installations() {
    let root = tempfile::tempdir().unwrap();
    let (left, left_database) = store(root.path()).await;
    let bytes = bundle("left", "shared", "1.0.0");
    assert!(matches!(left.install(&bytes, &[], &BTreeSet::from(["shared".into()])).await, Err(PluginError::Collision(_))));
    assert!(left.list().await.unwrap().is_empty());
    let installed = left.install(&bytes, &[], &BTreeSet::new()).await.unwrap();
    let right = left.clone();
    let before = saved_metadata(root.path());
    assert!(matches!(right.install(&bundle("right", "shared", "1.0.0"), &[], &BTreeSet::new()).await, Err(PluginError::Collision(_))));
    let configured = inspect(&bundle("configured", "external", "1.0.0"), &Default::default()).unwrap().1;
    assert!(matches!(
        right.install(&bundle("configured", "unique", "2.0.0"), &[configured], &BTreeSet::new()).await,
        Err(PluginError::Collision(_))
    ));
    assert!(right.install(b"invalid archive", &[], &BTreeSet::new()).await.is_err());
    assert_eq!(saved_metadata(root.path()), before);
    assert_eq!(right.list().await.unwrap().len(), 1);
    assert_eq!(fs::read_dir(left.root()).unwrap().count(), 1);
    assert_eq!(right.get(installed.installation.install_id).await.unwrap().installation.state, PluginState::Installed);
    left_database.close().await.unwrap();
}

#[tokio::test]
async fn refused_result_preserved() {
    let root = tempfile::tempdir().unwrap();
    let (store, store_database) = store(root.path()).await;
    let first = store.install(&bundle("fixture", "fixture", "1.0.0"), &[], &BTreeSet::new()).await.unwrap().installation.install_id;
    let original = fs::read(store.path(first)).unwrap();
    let before = saved_metadata(root.path());
    let checked = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let observed = checked.clone();
    let result = store
        .install_checked(&bundle("fixture", "fixture", "2.0.0"), &[], &BTreeSet::new(), move |entry| {
            observed.store(true, std::sync::atomic::Ordering::SeqCst);
            assert_eq!(entry.installation.state, PluginState::Installed);
            Err(PluginError::ResultLimit)
        })
        .await;
    assert!(checked.load(std::sync::atomic::Ordering::SeqCst));
    assert!(matches!(result, Err(PluginError::ResultLimit)));
    assert_eq!(saved_metadata(root.path()), before);
    assert_eq!(fs::read(store.path(first)).unwrap(), original);
    assert_eq!(fs::read_dir(store.root()).unwrap().count(), 1);
    store_database.close().await.unwrap();
}

#[tokio::test]
async fn unavailable_bytes_metadata() {
    let root = tempfile::tempdir().unwrap();
    let (store, store_database) = store(root.path()).await;
    let id = store.install(&bundle("fixture", "fixture", "1.0.0"), &[], &BTreeSet::new()).await.unwrap().installation.install_id;
    fs::write(store.path(id), bundle("untrusted", "other", "2.0.0")).unwrap();
    let entry = store.get(id).await.unwrap();
    assert!(entry.metadata.is_none());
    assert!(entry.error.is_some());
    let before = saved_metadata(root.path());
    assert!(matches!(store.install(&bundle("fixture", "fixture", "2.0.0"), &[], &BTreeSet::new()).await, Err(PluginError::Changed(_))));
    assert_eq!(saved_metadata(root.path()), before);
    store.remove(id).await.unwrap();
    assert_eq!(store.get(id).await.unwrap().installation.state, PluginState::RemovalPending);
    store.sweep().await.unwrap();
    assert!(store.list().await.unwrap().is_empty());
    store_database.close().await.unwrap();
}

#[tokio::test]
async fn installed_survive_cleanup() {
    let root = tempfile::tempdir().unwrap();
    let (store, store_database) = store(root.path()).await;
    let installed = store.install(&bundle("fixture", "fixture", "1.0.0"), &[], &BTreeSet::new()).await.unwrap().installation.install_id;
    let bytes = fs::read(store.path(installed)).unwrap();
    fs::write(store.root().join("orphan.dokime"), b"unreferenced installation").unwrap();
    store.sweep().await.unwrap();
    assert_eq!(fs::read_dir(store.root()).unwrap().count(), 1);
    assert_eq!(fs::read(store.path(installed)).unwrap(), bytes);

    store_database.close().await.unwrap();
}

#[cfg(unix)]
#[cfg(unix)]
#[tokio::test]
async fn failed_settings_bytes() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    let (store, store_database) = store(root.path()).await;
    let id = store.install(&bundle("fixture", "fixture", "1.0.0"), &[], &BTreeSet::new()).await.unwrap().installation.install_id;
    let bytes = fs::read(store.path(id)).unwrap();
    let before = saved_metadata(root.path());

    fs::set_permissions(store.root(), fs::Permissions::from_mode(0o555)).unwrap();
    let staged = store.install(&bundle("fixture", "fixture", "2.0.0"), &[], &BTreeSet::new()).await;
    fs::set_permissions(store.root(), fs::Permissions::from_mode(0o755)).unwrap();

    assert!(staged.is_err());
    assert_eq!(fs::read(store.path(id)).unwrap(), bytes);
    assert_eq!(saved_metadata(root.path()), before);
    assert_eq!(store.list().await.unwrap().len(), 1);
    store.sweep().await.unwrap();
    assert_eq!(fs::read_dir(store.root()).unwrap().count(), 1);
    store_database.close().await.unwrap();
}

#[tokio::test]
async fn removal_cleanup_retry() {
    let root = tempfile::tempdir().unwrap();
    let (store, store_database) = store(root.path()).await;
    let id = store.install(&bundle("fixture", "fixture", "1.0.0"), &[], &BTreeSet::new()).await.unwrap().installation.install_id;
    store.remove(id).await.unwrap();
    assert!(store.path(id).is_file());
    fs::remove_file(store.path(id)).unwrap();
    fs::create_dir(store.path(id)).unwrap();
    store.sweep().await.unwrap();
    let row = store.get(id).await.unwrap();
    assert_eq!(row.installation.state, PluginState::RemovalPending);
    assert!(matches!(row.installation.last_boot, Some(PluginLastBoot::Failed { .. })));
    fs::remove_dir(store.path(id)).unwrap();
    store_database.close().await.unwrap();
    let (restarted, restarted_database) = self::store(root.path()).await;
    restarted.sweep().await.unwrap();
    assert!(restarted.list().await.unwrap().is_empty());
    restarted_database.close().await.unwrap();
}

#[tokio::test]
async fn disabled_reserves_owners() {
    let root = tempfile::tempdir().unwrap();
    let (store, store_database) = store(root.path()).await;
    let id = store.install(&bundle("fixture", "fixture", "1.0.0"), &[], &BTreeSet::new()).await.unwrap().installation.install_id;
    store.set_enabled(id, false).await.unwrap();
    assert!(store.installed().await.unwrap().is_empty());
    assert!(store.path(id).is_file());
    assert!(matches!(store.install(&bundle("other", "fixture", "1.0.0"), &[], &BTreeSet::new()).await, Err(PluginError::Collision(_))));
    store.set_enabled(id, true).await.unwrap();
    assert_eq!(store.installed().await.unwrap().len(), 1);
    store.remove(id).await.unwrap();
    assert!(store.set_enabled(id, true).await.is_err());
    store_database.close().await.unwrap();
}

#[tokio::test]
async fn bounded_pages_unknown() {
    let root = tempfile::tempdir().unwrap();
    let (store, store_database) = store(root.path()).await;
    let mut ids = Vec::new();
    for index in 0..3 {
        let entry =
            store.install(&bundle(&format!("package{index}"), &format!("owner{index}"), "1.0.0"), &[], &BTreeSet::new()).await.unwrap();
        ids.push(entry.installation.install_id);
    }
    let first = store.page(None, 1).await.unwrap();
    assert_eq!(first.total, 3);
    assert_eq!(first.entries[0].installation.install_id, ids[0]);
    let second = store.page(Some((first.next.unwrap(), &first.manifest_sha256)), 1).await.unwrap();
    assert_eq!(second.entries[0].installation.install_id, ids[1]);
    assert!(matches!(store.page(Some((uuid::Uuid::from_u128(7), &first.manifest_sha256)), 1).await, Err(PluginError::Cursor)));
    store.remove(ids[2]).await.unwrap();
    assert!(matches!(store.page(Some((first.next.unwrap(), &first.manifest_sha256)), 1).await, Err(PluginError::Cursor)));
    assert!(matches!(store.page(None, 501).await, Err(PluginError::Limit)));
    store_database.close().await.unwrap();
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

#[tokio::test]
async fn app_roots_independent() {
    let root = tempfile::tempdir().unwrap();
    let (first, first_database) = store(&root.path().join("one")).await;
    let a = first.install(&bundle("one", "one", "1.0.0"), &[], &BTreeSet::new()).await.unwrap().installation.install_id;
    let (second, second_database) = store(&root.path().join("two")).await;
    let b = second.install(&bundle("two", "two", "1.0.0"), &[], &BTreeSet::new()).await.unwrap().installation.install_id;
    assert_eq!(first.list().await.unwrap()[0].installation.install_id, a);
    assert_eq!(second.list().await.unwrap()[0].installation.install_id, b);
    first_database.close().await.unwrap();
    second_database.close().await.unwrap();
}

#[tokio::test]
async fn unauthorized_files_kept() {
    let root = tempfile::tempdir().unwrap();
    let plugins = root.path().join("plugins");
    fs::create_dir_all(&plugins).unwrap();
    fs::write(plugins.join("manual.dokime"), bundle("manual", "manual", "1.0.0")).unwrap();
    fs::write(plugins.join(".tmpABC123"), b"interrupted stage").unwrap();
    let (store, store_database) = store(root.path()).await;
    assert!(store.list().await.unwrap().is_empty());
    assert!(store.installed().await.unwrap().is_empty());
    store.sweep().await.unwrap();
    assert_eq!(fs::read_dir(&plugins).unwrap().count(), 0);

    store_database.close().await.unwrap();
    drop(store);
    let database = dokime_runtime::storage::sqlite::State::open(&root.path().join("data")).await.unwrap();
    database.call(|connection| connection.execute("UPDATE application_metadata SET body=X'7b' WHERE id=1", [])).await.unwrap();
    assert!(Settings::open(root.path(), database.clone()).await.is_err());
    let body: Vec<u8> = database
        .call(|connection| connection.query_row("SELECT body FROM application_metadata WHERE id=1", [], |row| row.get(0)))
        .await
        .unwrap();
    assert_eq!(body, b"{");
    database.close().await.unwrap();
}
