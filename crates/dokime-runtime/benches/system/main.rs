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
use std::path::PathBuf;

// Third Party Imports
use criterion::{criterion_group, criterion_main};

// Local Imports

/* === Constants === */

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
    let path = std::env::current_dir()
        .expect("benchmark runner starts in the package")
        .ancestors()
        .find(|directory| directory.join("Cargo.lock").is_file())
        .expect("the workspace root holds Cargo.lock")
        .join("target/build/bench")
        .join(format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH))
        .join("plugins/obc-plugin.dokime");
    assert!(path.is_file(), "missing built artifact {}; run `mise run build`", path.display());
    path
}

/* === Private Functions === */

/* === Tests === */
