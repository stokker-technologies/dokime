/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// System Benchmarks: The Runtime Driving A Built Sidecar Plugin

/* === Modules === */
mod report;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::path::{Path, PathBuf};

// Third Party Imports
use criterion::{criterion_group, criterion_main};

// Local Imports

/* === Constants === */
const PLATFORM: &str = if cfg!(windows) { "windows" } else { "linux" };

/* === Macros === */
criterion_group!(benches, report::benchmark);
criterion_main!(benches);

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn plugin_bundle() -> PathBuf {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|directory| directory.join("Cargo.lock").is_file())
        .expect("the workspace root holds Cargo.lock")
        .join("target/build/bench")
        .join(format!("{PLATFORM}-{}", std::env::consts::ARCH))
        .join("fixtures/plugins/obc-plugin.dokime");
    assert!(path.is_file(), "missing built artifact {}; run `mise run build`", path.display());
    path
}

/* === Private Functions === */

/* === Tests === */
