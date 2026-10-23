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
use std::path::PathBuf;
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

pub fn workspace_root() -> PathBuf {
    std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("nextest sets the manifest directory"))
        .ancestors()
        .find(|directory| directory.join("Cargo.lock").is_file())
        .expect("the workspace root holds Cargo.lock").to_owned()
}

/* === Private Functions === */
fn artifact(relative: &str) -> PathBuf {
    let path = build_root().join(relative);
    assert!(path.is_file(), "missing built artifact {}; run `mise run build`", path.display());
    path
}

fn build_root() -> PathBuf {
    let profile = if cfg!(debug_assertions) { "dev" } else { "release" };
    let target = format!("{PLATFORM}-{}", std::env::consts::ARCH);
    workspace_root().join("target/build").join(profile).join(target)
}

/* === Tests === */
