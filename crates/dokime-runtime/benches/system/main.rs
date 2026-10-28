/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// System Benchmarks: The Runtime Driving A Built Sidecar Plugin

mod report;

use std::path::PathBuf;

use criterion::{criterion_group, criterion_main};

criterion_group!(benches, report::benchmark);
criterion_main!(benches);

pub fn plugin_bundle() -> PathBuf {
    let path = std::env::current_dir()
        .expect("benchmark runner starts in the package")
        .ancestors()
        .find(|directory| directory.join("Cargo.lock").is_file())
        .expect("the workspace root holds Cargo.lock")
        .join("target/build/bench")
        .join(format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH))
        .join("fixtures/plugins/obc-plugin.dokime");
    assert!(path.is_file(), "missing built artifact {}; run `mise run build:rust:bench`", path.display());
    path
}
