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
use std::path::{Path, PathBuf};
// Third Party Imports

// Local Imports

/* === Constants === */
const PLATFORM: &str = if cfg!(windows) { "windows" } else { "linux" };
const EXECUTABLE_SUFFIX: &str = if cfg!(windows) { ".exe" } else { "" };

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub fn host_binary() -> PathBuf {
    artifact(&format!("bin/dokime-server{EXECUTABLE_SUFFIX}"))
}

pub fn obc_plugin_bundle() -> PathBuf {
    artifact("plugins/obc-plugin.dokime")
}

/* === Private Functions === */
fn artifact(relative: &str) -> PathBuf {
    let path = build_root().join(relative);
    assert!(path.is_file(), "missing built artifact {}; run `mise run build`", path.display());
    path
}

fn build_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|directory| directory.join("Cargo.lock").is_file())
        .expect("the workspace root holds Cargo.lock")
        .join("dist/build")
        .join(PLATFORM)
}

/* === Tests === */
