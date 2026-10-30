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
    artifact(&format!("bin/dokime{EXECUTABLE_SUFFIX}"))
}

pub fn host_plugin_bundle() -> PathBuf {
    artifact("plugins/plugin-host.dokime")
}

pub fn plugin_obc_bundle() -> PathBuf {
    artifact("fixtures/plugins/plugin-obc.dokime")
}

pub fn workspace_root() -> PathBuf {
    std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").expect("nextest sets the manifest directory"))
        .ancestors()
        .find(|directory| directory.join("Cargo.lock").is_file())
        .expect("the workspace root holds Cargo.lock")
        .to_owned()
}

/* === Private Functions === */
fn artifact(relative: &str) -> PathBuf {
    let path = build_root().join(relative);
    assert!(path.is_file(), "missing built artifact {}; run `mise run build`", path.display());
    path
}

fn build_root() -> PathBuf {
    let profile = std::env::var("STKR_TEST_PROFILE").unwrap_or_else(|_| "dev".to_owned());
    assert!(matches!(profile.as_str(), "dev" | "release"), "invalid graph profile");
    let target = format!("{PLATFORM}-{}", std::env::consts::ARCH);
    workspace_root().join("target/build").join(profile).join(target)
}

/* === Tests === */
