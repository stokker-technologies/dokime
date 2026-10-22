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
use dokime::Metadata;
use dokime::RuntimeConfig;
use dokime::config::StorageSettings;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn runtime_config(root: impl Into<PathBuf>, hub_metadata: Metadata) -> RuntimeConfig {
    RuntimeConfig::new(root, hub_metadata).with_test_secret_store(dokime::SecretTestStore::default())
}

pub fn story_settings() -> StorageSettings {
    dokime::config::storage_defaults()
}

/* === Private Functions === */

/* === Tests === */
