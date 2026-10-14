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
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
// Third Party Imports
use sha2::{Digest, Sha256};
// Local Imports

/* === Constants === */
// `dokime_tools.fixtures` writes this stamp beside every bundle set it compiles.
pub const STAMP: &str = "fixtures.stamp";
pub const PREPARE: &str = "fixtures predate this tree: run mise run fixtures:prepare";
pub const DEV_PLUGINS: &str = "fixtures predate this tree: run mise run build:plugins:dev";
// Mirrors `dokime_tools.bindings.CONTRACT_SET`: hashed in this order.
const CONTRACT_SET: [&str; 2] = ["contracts.json", "descriptors.bin"];

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
/// The prepared bundle directory `mise run fixtures:prepare` fills.
pub fn prepared_directory() -> PathBuf {
    repository().join("target/fixtures")
}

/// Panics with `hint` unless `directory` carries a stamp for this tree's contracts.
pub fn require_current(directory: &Path, hint: &str) {
    let stamp = std::fs::read_to_string(directory.join(STAMP)).ok();
    if !current(stamp.as_deref(), contract_fingerprint()) {
        panic!("{hint}");
    }
}

/// Whether a stamp's recorded contract fingerprint is this tree's.
pub fn current(stamp: Option<&str>, contracts: &str) -> bool {
    stamp
        .into_iter()
        .flat_map(str::lines)
        .filter_map(|line| line.strip_prefix("contracts "))
        .any(|recorded| recorded == contracts)
}

/// The SHA-256 of the committed contract set, as `dokime_tools.bindings` computes it.
pub fn contract_fingerprint() -> &'static str {
    static FINGERPRINT: OnceLock<String> = OnceLock::new();
    FINGERPRINT.get_or_init(|| {
        let generated = repository().join("packages/dokime/src/dokime");
        let mut digest = Sha256::new();
        for name in CONTRACT_SET {
            let path = generated.join(name);
            digest.update(std::fs::read(&path).unwrap_or_else(|error| {
                panic!("contract set {} is unreadable: {error}", path.display())
            }));
        }
        digest
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect()
    })
}

/* === Private Functions === */
fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/* === Tests === */
