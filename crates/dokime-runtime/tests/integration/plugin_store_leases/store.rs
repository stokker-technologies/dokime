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
    let settings = Settings::open(dokime_runtime::paths::ApplicationPaths::new(root), database.clone()).await.unwrap();
    (PluginStore::with_archive_limit(settings, Default::default(), 285_212_672), database)
}

fn saved_metadata(root: &Path) -> serde_json::Value {
    let connection =
        rusqlite::Connection::open_with_flags(root.join("data/local.sqlite"), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let body: Vec<u8> = connection.query_row("SELECT body FROM application_metadata WHERE id=1", [], |row| row.get(0)).unwrap();
    serde_json::from_slice(&body).unwrap()
}

fn installer_metadata(path: &Path, version: &str, bytes: &[u8]) {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let files = serde_json::json!({"plugins/plugin-host.dokime": digest});
    let metadata = serde_json::json!({"schema":1,"version":version,"files":files});
    fs::write(path, serde_json::to_vec(&metadata).unwrap()).unwrap();
}

fn large_bundle(package: &str, owner: &str, version: &str, resource_bytes: usize) -> Vec<u8> {
    let identity = Identity {
        name: package.into(),
        version: version.parse().unwrap(),
        description: "A fixture.".into(),
        package: Default::default(),
    };
    let declaration =
        serde_json::json!({"kind":"plugin","facility":"sidecar","id":owner,"version":version,"description":"A fixture.","config":"()"});
    let policy = harmos::operations::OperationPolicy {
        timeout: std::time::Duration::from_secs(120),
        attempt_timeout: std::time::Duration::from_secs(120),
        retries: harmos::operations::RetryPolicy::Never,
    };
    // Inert artifact metadata names each lifecycle action's identity scheme rather than its fingerprint.
    let lifecycle = serde_json::json!([
        {"action": "initialize", "identity": "harmos.operation.v1", "policy": policy},
        {"action": "shutdown", "identity": "harmos.operation.v1", "policy": policy},
    ]);
    let contracts =
        serde_json::json!({"kind":"contracts","facility":"sidecar","owner":owner,"entry":{"direction":"lifecycle","lifecycle":lifecycle}});
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
            Source {
                path: "resources/payload.bin",
                bytes: &resource,
                executable: None,
            },
        ],
        Limits::default(),
    )
    .unwrap()
    .bytes()
    .to_vec()
}

/* === Tests === */

#[tokio::test]
async fn boot_failure_persists_plugin_and_carrier() {
    let root = tempfile::tempdir().unwrap();
    let (store, database) = store(root.path()).await;
    let id = store.install(&bundle("fixture", "fixture", "1.0.0"), &[], &BTreeSet::new()).await.unwrap().installation.install_id;
    let carrier = dokime_runtime::harmos::domain::Failure::new(dokime_runtime::harmos::operations::FailureKind::Permanent, "api_token")
        .with_detail("dokime.secret", "api_token")
        .encode();
    let failure = dokime_domain::model::plugins::wire::PluginStartFailure {
        plugin_id: "fixture".into(),
        plugin_name: "Fixture".into(),
        message: String::new(),
        code: Some(dokime_domain::model::plugins::error::MISSING_SECRET_BINDING_CODE.to_string()),
        missing_secret: Some("api_token".into()),
        consequences: vec!["workflow is unavailable".into()],
        carrier: carrier.clone(),
    };
    store.record_boot(id, Some(failure.clone())).await.unwrap();
    database.close().await.unwrap();
    let (reopened, database) = self::store(root.path()).await;
    let row = reopened.get(id).await.unwrap();
    let Some(PluginLastBoot::Failed { failure: retained, .. }) = row.installation.last_boot else { panic!() };
    assert_eq!(retained, failure);
    assert_eq!(retained.plugin_id, "fixture");
    assert_eq!(retained.missing_secret.as_deref(), Some("api_token"));
    assert_eq!(
        dokime_runtime::harmos::domain::Failure::decode(&retained.carrier).unwrap().code(),
        dokime_runtime::harmos::operations::FailureKind::Permanent
    );
    database.close().await.unwrap();
}

#[tokio::test]
async fn schema_budget_installation() {
    use dokime_runtime::config::{Config, RuntimeConfig, schema_defaults};
    use std::num::NonZeroUsize;
    let root = tempfile::tempdir().unwrap();
    let database = dokime_runtime::storage::sqlite::State::open(&root.path().join("data")).await.unwrap();
    let saved = Settings::open(dokime_runtime::paths::ApplicationPaths::new(root.path()), database.clone()).await.unwrap();
    let defaults = schema_defaults();
    assert_eq!(Config::local(root.path()).schema.preparation_bytes, defaults.preparation_bytes);
    let config = RuntimeConfig::new(
        root.path(),
        dokime_domain::model::identity::Metadata::new(dokime_domain::model::identity::Identifier::new("fixture").unwrap(), "Fixture", None),
    );
    assert_eq!(config.schema.preparation_work, defaults.preparation_work);
    let bytes = bundle("budget-fixture", "fixture", "1.0.0");
    for budget in ["memory bytes", "work"] {
        let mut schema = defaults;
        if budget == "work" {
            schema.preparation_work = NonZeroUsize::new(1).unwrap();
        } else {
            schema.preparation_bytes = NonZeroUsize::new(1).unwrap();
        }
        let store = PluginStore::new(saved.clone(), schema);
        let error = store.install(&bytes, &[], &BTreeSet::new()).await.unwrap_err().to_string();
        assert!(error.contains("budget-fixture") && error.contains(budget) && error.contains("allowed 1"), "{error}");
        assert!(store.list().await.unwrap().is_empty());
        assert!(!store.root().exists());
    }
    let store = PluginStore::new(saved, defaults);
    let installed = store.install(&bytes, &[], &BTreeSet::new()).await.unwrap();
    assert_eq!(installed.metadata.unwrap().identity.name, "budget-fixture");
    drop(store);
    database.close().await.unwrap();
}

#[tokio::test]
async fn bundled_upgrades() {
    let root = tempfile::tempdir().unwrap();
    let (store, database) = store(root.path()).await;
    let first_bytes = bundle("organization-host", "host", "1.0.0");
    let first = store.install_bundled(&first_bytes, &[], &BTreeSet::new()).await.unwrap();
    let first_id = first.installation.install_id;
    assert_eq!(first.metadata.unwrap().identity.name, "organization-host");
    assert_eq!(fs::read(store.path(first_id)).unwrap(), first_bytes);
    let repeated = store.install_bundled(&first_bytes, &[], &BTreeSet::new()).await.unwrap();
    assert_eq!(repeated.installation.install_id, first_id);
    let settings = Settings::open(dokime_runtime::paths::ApplicationPaths::new(root.path()), database.clone()).await.unwrap();
    settings
        .edit(move |document| {
            let plugin = document.plugins.get_mut(&first_id).unwrap();
            plugin.programs.entry("host".into()).or_default().saved.insert("calibration".into());
            Ok::<_, PluginError>(())
        })
        .await
        .unwrap();
    drop(settings);

    let upgraded_bytes = bundle("organization-host", "host", "2.0.0");
    let upgraded = store.install_bundled(&upgraded_bytes, &[], &BTreeSet::new()).await.unwrap();
    let upgraded_id = upgraded.installation.install_id;
    assert_ne!(upgraded_id, first_id);
    assert_eq!(store.get(first_id).await.unwrap().installation.state, PluginState::RemovalPending);
    assert_eq!(store.get(upgraded_id).await.unwrap().metadata.unwrap().identity.version, "2.0.0");
    assert_eq!(
        saved_metadata(root.path())["plugins"][upgraded_id.to_string()]["programs"]["host"]["saved"],
        serde_json::json!(["calibration"])
    );
    store.set_enabled(upgraded_id, false).await.unwrap();
    drop(store);
    database.close().await.unwrap();

    // A Fresh Application Start Retains A Newer Installed Version And Its Disabled State.
    let (store, database) = self::store(root.path()).await;
    let kept = store.install_bundled(&first_bytes, &[], &BTreeSet::new()).await.unwrap();
    assert_eq!(kept.installation.install_id, upgraded_id);
    assert_eq!(kept.installation.state, PluginState::Disabled);
    assert_eq!(fs::read(store.path(upgraded_id)).unwrap(), upgraded_bytes);
    let newer = bundle("organization-host", "host", "3.0.0");
    let upgraded = store.install_bundled(&newer, &[], &BTreeSet::new()).await.unwrap();
    assert_eq!(upgraded.installation.state, PluginState::Disabled);
    assert_eq!(upgraded.metadata.unwrap().identity.version, "3.0.0");

    let fourth = bundle("organization-host", "host", "4.0.0");
    let fifth = bundle("organization-host", "host", "5.0.0");
    let sources = BTreeSet::new();
    let (left, right) = tokio::join!(store.install_bundled(&fifth, &[], &sources), store.install_bundled(&fourth, &[], &sources));
    left.unwrap();
    right.unwrap();
    let current = store.list().await.unwrap().into_iter().find(|entry| entry.installation.state == PluginState::Disabled).unwrap();
    assert_eq!(current.metadata.unwrap().identity.version, "5.0.0");
    database.close().await.unwrap();
}

#[tokio::test]
async fn bundled_sources_survive() {
    for identifier in [dokime_runtime::paths::DEFAULT_APPLICATION_ID, "com.hilti.testhub"] {
        let root = tempfile::tempdir().unwrap();
        let application = root.path().join(identifier);
        let plugins = application.join("plugins");
        fs::create_dir_all(&plugins).unwrap();
        let source = plugins.join("plugin-host.dokime");
        let bytes = bundle("organization-host", "host", "1.0.0");
        fs::write(&source, &bytes).unwrap();
        let metadata = application.join("installer.json");
        let mut installation = None;
        for version in ["1.0.0", "2.0.0"] {
            installer_metadata(&metadata, version, &bytes);
            let (store, database) = self::store(&application).await;
            assert_eq!(store.root(), application.join("data/plugins"));
            store.sweep().await.unwrap();
            store.install_distribution(&plugins, &[], &BTreeSet::new()).await.unwrap();
            let installed = store.installed().await.unwrap();
            assert_eq!(installed.len(), 1);
            assert_eq!(installed[0].path.parent(), Some(store.root()));
            assert_eq!(fs::read(&installed[0].path).unwrap(), bytes);
            assert!(source.is_file());
            assert_eq!(fs::read(&source).unwrap(), bytes);
            let current = store.list().await.unwrap().pop().unwrap();
            assert_eq!(current.metadata.unwrap().identity.name, "organization-host");
            if let Some(previous) = installation {
                assert_eq!(current.installation.install_id, previous);
            }
            installation = Some(current.installation.install_id);
            drop(store);
            database.close().await.unwrap();
        }
    }
}

#[tokio::test]
async fn bundled_distribution() {
    let root = tempfile::tempdir().unwrap();
    let installer = tempfile::tempdir().unwrap();
    let plugins = installer.path().join("plugins");
    fs::create_dir(&plugins).unwrap();
    let distribution = installer.path().join("installer.json");
    fs::write(installer.path().join("distribution.json"), br#"{"schema":1,"identifier":"com.hilti.dokime"}"#).unwrap();
    let bytes = bundle("organization-host", "host", "1.0.0");
    installer_metadata(&distribution, "1.0.0", &bytes);
    fs::write(plugins.join("plugin-host.dokime"), &bytes).unwrap();
    fs::write(plugins.join("leftover.dokime"), b"unindexed older installer resource").unwrap();
    let sources = BTreeSet::new();
    let (store, database) = self::store(root.path()).await;
    store.install_distribution(&plugins, &[], &sources).await.unwrap();
    let first = store.installed().await.unwrap();
    assert_eq!(first.len(), 1);
    let first = store.list().await.unwrap().pop().unwrap();
    store.remove(first.installation.install_id).await.unwrap();
    store.sweep().await.unwrap();
    drop(store);
    database.close().await.unwrap();

    let (store, database) = self::store(root.path()).await;
    store.install_distribution(&plugins, &[], &sources).await.unwrap();
    assert!(store.installed().await.unwrap().is_empty());
    installer_metadata(&distribution, "2.0.0", &bytes);
    store.install_distribution(&plugins, &[], &sources).await.unwrap();
    assert_eq!(store.installed().await.unwrap().len(), 1);
    let newer = store.install(&bundle("organization-host", "host", "3.0.0"), &[], &sources).await.unwrap();
    installer_metadata(&distribution, "3.0.0", &bytes);
    store.install_distribution(&plugins, &[], &sources).await.unwrap();
    assert_eq!(fs::read(store.path(newer.installation.install_id)).unwrap(), bundle("organization-host", "host", "3.0.0"));

    let before = saved_metadata(root.path())["bundled_distribution"].clone();
    installer_metadata(&distribution, "4.0.0", &bytes);
    fs::write(plugins.join("plugin-host.dokime"), b"modified archive").unwrap();
    let refused = store.install_distribution(&plugins, &[], &sources).await.unwrap_err();
    assert!(refused.to_string().contains("plugin-host.dokime"));
    assert_eq!(saved_metadata(root.path())["bundled_distribution"], before);
    fs::write(plugins.join("plugin-host.dokime"), &bytes).unwrap();
    store.install_distribution(&plugins, &[], &sources).await.unwrap();
    assert_ne!(saved_metadata(root.path())["bundled_distribution"], before);
    database.close().await.unwrap();
}

pub(crate) async fn direct_installation() {
    let bytes = large_bundle("fixture", "fixture", "1.0.0", 70 * 1024 * 1024);
    let root = tempfile::tempdir().unwrap();
    let (store, store_database) = store(root.path()).await;
    inspect(&bytes, &Default::default()).unwrap();
    let limited_settings = Settings::open(dokime_runtime::paths::ApplicationPaths::new(root.path()), store_database.clone()).await.unwrap();
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

pub(crate) async fn replacement_preserves_bytes() {
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

pub(crate) async fn collisions_preserve_installations() {
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

pub(crate) async fn refused_result_preserved() {
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

pub(crate) async fn unavailable_bytes_metadata() {
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

pub(crate) async fn installed_survive_cleanup() {
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
pub(crate) async fn failed_settings_bytes() {
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

pub(crate) async fn removal_cleanup_retry() {
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

pub(crate) async fn disabled_reserves_owners() {
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

pub(crate) async fn bounded_pages_unknown() {
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

pub(crate) async fn startup_reclaims_extraction() {
    let root = tempfile::tempdir().unwrap();
    let config = crate::service_support::config(root.path());
    let extraction = config.application.root.join("cache/plugins");
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

pub(crate) async fn app_roots_independent() {
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

pub(crate) async fn unauthorized_files_kept() {
    let root = tempfile::tempdir().unwrap();
    let plugins = dokime_runtime::paths::ApplicationPaths::new(root.path()).plugins();
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
    assert!(Settings::open(dokime_runtime::paths::ApplicationPaths::new(root.path()), database.clone()).await.is_err());
    let body: Vec<u8> = database
        .call(|connection| connection.query_row("SELECT body FROM application_metadata WHERE id=1", [], |row| row.get(0)))
        .await
        .unwrap();
    assert_eq!(body, b"{");
    database.close().await.unwrap();
}
