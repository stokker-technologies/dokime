/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use dokime_runtime::storage::sqlite::Backend;
use harmos::Runtime;
use harmos::storage::*;
use std::sync::{Arc, OnceLock};

use super::{Model, State};

pub struct Fixture {
    pub settings: StorageSettings,
    _directory: tempfile::TempDir,
    pub storage: StorageManager,
    pub runtime: Runtime<Model>,
}

impl Fixture {
    pub async fn new() -> Self {
        let directory = tempfile::tempdir().unwrap();
        let settings = settings();
        let backends = backends(directory.path()).await;
        let storage = StorageManager::new(backends, settings.clone()).unwrap();
        let runtime = Runtime::<Model>::builder(State::default(), ())
            .register(super::Transactions::catalog())
            .register(super::Records::catalog())
            .runtime_with_storage(&storage)
            .await
            .unwrap();
        Self {
            settings,
            _directory: directory,
            storage,
            runtime,
        }
    }

    pub async fn finish(self) {
        self.storage.close().await.unwrap();
        self.runtime.stop().await.unwrap();
        drop(self.runtime);
        drop(self.storage);
        self._directory.close().unwrap();
    }
}

pub fn settings() -> StorageSettings {
    static SETTINGS: OnceLock<StorageSettings> = OnceLock::new();
    SETTINGS
        .get_or_init(|| {
            let settings = dokime_runtime::config::storage_defaults();
            eprintln!("Dokime SQLite Storage Defaults: {settings:?}");
            settings
        })
        .clone()
}

pub async fn backends(root: &std::path::Path) -> StorageBackends {
    let primary = Arc::new(Backend::<()>::primary(root.to_owned()).await.unwrap());
    let state = harmos::storage::state::file::FileBackend::new(
        root.to_owned(),
        primary.history().clone(),
        dokime_runtime::config::storage_state_defaults(),
    );
    StorageBackends { primary, state: Arc::new(state) }
}
