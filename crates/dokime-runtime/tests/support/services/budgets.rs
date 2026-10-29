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

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::path::PathBuf;
// Third Party Imports

// Local Imports
use dokime_runtime::Metadata;
use dokime_runtime::RuntimeConfig;
use dokime_runtime::config::StorageSettings;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn runtime_config(root: impl Into<PathBuf>, hub_metadata: Metadata) -> RuntimeConfig {
    RuntimeConfig::new(root, hub_metadata)
        .with_credential_store(keyring_core::mock::Store::new().expect("isolated fixture credential store"))
}

pub fn story_settings() -> StorageSettings {
    dokime_runtime::config::storage_defaults()
}

/* === Private Functions === */
/* === Tests === */
