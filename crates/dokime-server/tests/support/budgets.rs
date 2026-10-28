/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Each test binary includes this file and uses its own subset.
#![allow(dead_code)]

use std::path::PathBuf;

use dokime::Metadata;
use dokime::RuntimeConfig;
use dokime::config::StorageSettings;

pub fn runtime_config(root: impl Into<PathBuf>, hub_metadata: Metadata) -> RuntimeConfig {
    RuntimeConfig::new(root, hub_metadata)
        .with_credential_store(keyring_core::mock::Store::new().expect("isolated fixture credential store"))
}

pub fn story_settings() -> StorageSettings {
    dokime::config::storage_defaults()
}
