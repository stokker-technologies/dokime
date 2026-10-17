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
use std::process::Command;

// Third Party Imports

// Local Imports

/* === Constants === */
const REPOSITORY: &str = "https://github.com/stokker-technologies/dokime";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

/// Runs one of the generated plugin's own mise tasks, offline, against this tree.
fn task(plugin: &Path, name: &str) {
    let output = Command::new("mise")
        .arg("run")
        .arg(name)
        .current_dir(plugin)
        .env("MISE_TRUSTED_CONFIG_PATHS", plugin)
        .env("MISE_YES", "1")
        .env("CARGO_NET_OFFLINE", "true")
        .env("CARGO_TARGET_DIR", workspace().join("target/scaffold"))
        .output()
        .expect("mise runs the generated plugin's tasks");
    assert!(
        output.status.success(),
        "mise run {name} failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/* === Tests === */
// a generated plugin passes its own check and unit tasks against this tree.

pub(super) fn generated_plugin_tasks() {
    let root = super::support::standalone_directory();
    let plugin = root.path().join("bench-plugin");

    // Generate The Plugin
    let output = Command::new(super::support::tool())
        .args(["dokime", "new"])
        .arg(&plugin)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    for file in [
        "mise.toml",
        "stkr.toml",
        ".github/workflows/ci.yml",
        ".gitlab-ci.yml",
    ] {
        assert!(plugin.join(file).is_file(), "{file} is generated");
    }

    // Build It Against This Tree, Offline
    let sdk = workspace().join("crates/dokime");
    std::fs::create_dir(plugin.join(".cargo")).unwrap();
    std::fs::write(
        plugin.join(".cargo/config.toml"),
        format!(
            "[patch.{REPOSITORY:?}]\ndokime = {{ path = {:?} }}\n",
            sdk.display().to_string()
        ),
    )
    .unwrap();
    // The workspace lockfile keeps Harmos at the revision this tree pins.
    std::fs::copy(workspace().join("Cargo.lock"), plugin.join("Cargo.lock")).unwrap();

    task(&plugin, "check");
    task(&plugin, "test:unit");
}
