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
use sha2::{Digest, Sha256};

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
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

/// Runs one of the generated plugin's own mise tasks, offline, against this tree.
fn task(plugin: &Path, name: &str) {
    let output = Command::new("mise")
        .arg("run")
        .arg(name)
        .current_dir(plugin)
        .env("MISE_TRUSTED_CONFIG_PATHS", plugin)
        .env("MISE_YES", "1")
        .env_remove("UV_PROJECT_ENVIRONMENT")
        .env_remove("UV_PROJECT")
        .env_remove("UV_WORKING_DIR")
        .env("UV_OFFLINE", "true")
        .env("UV_CONFIG_FILE", plugin.join("uv.toml"))
        .env("UV_FIND_LINKS", workspace().join("target/dependencies/wheels"))
        .env("UV_CACHE_DIR", plugin.join("target/uv-cache"))
        .env("CARGO_NET_OFFLINE", "true")
        .env_remove("CARGO_TARGET_DIR")
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
    let output = Command::new(super::support::tool()).args(["dokime", "new"]).arg(&plugin).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    for file in [
        "mise.toml",
        "stkr.toml",
        ".github/workflows/ci.yml",
        ".gitlab-ci.yml",
        "tools/python/pyproject.toml",
        "tools/python/uv.lock",
        "tools/python/src/project_tools/test/run.py",
        "tools/python/src/project_tools/test/archive.py",
    ] {
        assert!(plugin.join(file).is_file(), "{file} is generated");
    }

    // Managed Assets Keep Their Digest After Product Placeholder Expansion.
    for file in ["mise.toml", "tools/python/src/project_tools/test/run.py"] {
        let contents = std::fs::read_to_string(plugin.join(file)).unwrap();
        let (marker, body) = contents.split_once('\n').unwrap();
        assert!(marker.starts_with("# Rendered by stkr project update from "));
        let (_, recorded) = marker.rsplit_once("; sha256:").unwrap();
        assert_eq!(recorded, format!("{:x}", Sha256::digest(body.as_bytes())));
        assert!(!contents.contains("{{package}}"), "{file} has no unresolved package placeholder");
    }

    // Build It Against This Tree, Offline
    let sdk = workspace().join("crates/dokime-runtime");
    std::fs::create_dir(plugin.join(".cargo")).unwrap();
    let dependencies = workspace().join("target/dependencies");
    assert!(dependencies.join("wheels").is_dir(), "build stages the locked starter wheelhouse");
    let mut cargo = std::fs::read_to_string(dependencies.join("cargo-config.toml"))
        .expect("build exports the locked Cargo source closure")
        .parse::<toml_edit::DocumentMut>()
        .unwrap();
    let mut redirected = false;
    for (_, source) in cargo["source"].as_table_mut().unwrap().iter_mut() {
        if let Some(directory) = source.as_table_like_mut().and_then(|table| table.get_mut("directory")) {
            *directory = toml_edit::value(dependencies.join("vendor").display().to_string());
            redirected = true;
        }
    }
    assert!(redirected, "the exported source configuration names its vendor directory");
    let patched = format!("{cargo}\n[patch.{REPOSITORY:?}]\ndokime-runtime = {{ path = {:?} }}\n", sdk.display().to_string(),);
    std::fs::write(plugin.join(".cargo/config.toml"), patched).unwrap();
    std::fs::write(plugin.join("uv.toml"), "no-index = true\n").unwrap();
    // The workspace lockfile keeps Harmos at the revision this tree pins.
    std::fs::copy(workspace().join("Cargo.lock"), plugin.join("Cargo.lock")).unwrap();

    task(&plugin, "check");
    // Exercise The Generated Project's Actual Archive Producer Before Its
    // Unit Lane Reads That Archive. This Is Authoring Behavior Under Test.
    task(&plugin, "build");
    task(&plugin, "test:unit");
}
