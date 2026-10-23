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
#[cfg(windows)]
use std::os::windows::process::CommandExt;
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
// Every locked package with its version, whatever source the lock names.
fn pins(lock: &str) -> Vec<(String, String)> {
    let document = lock.parse::<toml_edit::DocumentMut>().unwrap();
    let mut pins = document["package"]
        .as_array_of_tables()
        .unwrap()
        .iter()
        .map(|package| (package["name"].as_str().unwrap().to_owned(), package["version"].as_str().unwrap().to_owned()))
        .collect::<Vec<_>>();
    pins.sort();
    pins
}

fn workspace() -> PathBuf {
    super::support::workspace_root()
}

// The offline closure the build stages beside its other outputs, so a lane runner's artifact carries it.
fn dependencies() -> PathBuf {
    super::support::build_root().join("fixtures/dependencies")
}

fn task(plugin: &Path, name: &str) {
    run(plugin, "mise", &["run", name]);
}

fn run(plugin: &Path, program: &str, arguments: &[&str]) {
    let name = format!("{program} {}", arguments.join(" "));
    let mut command = Command::new(program);
    #[cfg(windows)]
    command.creation_flags(0x0800_0000);
    // The Starter Runs As Its Author Would, Outside The Lane That Runs This Story.
    for (variable, _) in std::env::vars_os() {
        if variable.to_str().is_some_and(|variable| {
            variable.starts_with("NEXTEST") || matches!(variable, "VIRTUAL_ENV" | "MISE_LOCKED" | "MISE_ENABLE_TOOLS")
        }) {
            command.env_remove(variable);
        }
    }
    let tool = super::support::tool();
    let mut paths = vec![tool.parent().unwrap().to_owned()];
    paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()));
    let output = command
        .args(arguments)
        .current_dir(plugin)
        .env("MISE_TRUSTED_CONFIG_PATHS", plugin)
        .env("MISE_YES", "1")
        .env("PATH", std::env::join_paths(paths).unwrap())
        .env_remove("UV_PROJECT_ENVIRONMENT")
        .env_remove("UV_PROJECT")
        .env_remove("UV_WORKING_DIR")
        .env("UV_OFFLINE", "true")
        .env("UV_CONFIG_FILE", plugin.join("uv.toml"))
        .env("UV_FIND_LINKS", dependencies().join("wheels"))
        .env("UV_CACHE_DIR", plugin.join("target/uv-cache"))
        .env("CARGO_NET_OFFLINE", "true")
        .env_remove("CARGO_TARGET_DIR")
        .output()
        .expect("mise runs the generated plugin's tasks");
    assert!(
        output.status.success(),
        "{name} failed:\n{}\n{}",
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
    let mut command = Command::new(super::support::tool());
    #[cfg(windows)]
    command.creation_flags(0x0800_0000);
    let output = command.args(["dokime", "new"]).arg(&plugin).output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    for file in [
        "mise.toml",
        "stkr.toml",
        ".github/workflows/ci.yml",
        ".gitlab-ci.yml",
        "tools/python/pyproject.toml",
        "pyproject.toml",
        "uv.lock",
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
        assert_eq!(recorded, Sha256::digest(body.as_bytes()).iter().map(|byte| format!("{byte:02x}")).collect::<String>());
        assert!(!contents.contains("{{package}}"), "{file} has no unresolved package placeholder");
    }

    // Build It Against This Tree, Offline
    let sdk = workspace().join("crates/dokime-runtime");
    std::fs::create_dir(plugin.join(".cargo")).unwrap();
    let dependencies = dependencies();
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
    std::fs::write(plugin.join(".cargo/config.toml"), cargo.to_string()).unwrap();
    // The SDK Comes From This Tree: A `[patch]` Would Still Resolve The Published Git Source First,
    // Which An Offline Runner Without A Cached Checkout Cannot Do.
    let manifest = plugin.join("Cargo.toml");
    let mut authored = std::fs::read_to_string(&manifest).unwrap().parse::<toml_edit::DocumentMut>().unwrap();
    assert_eq!(authored["dependencies"]["dokime-runtime"]["git"].as_str(), Some(REPOSITORY), "the starter depends on the published SDK");
    let mut local = toml_edit::InlineTable::new();
    local.insert("path", sdk.display().to_string().into());
    authored["dependencies"]["dokime-runtime"] = toml_edit::value(local);
    std::fs::write(&manifest, authored.to_string()).unwrap();
    std::fs::write(plugin.join("uv.toml"), "no-index = true\n").unwrap();
    // The Staged Wheels Replace The Public Index: Rebind The Shipped Lock To Them Without Moving A Pin.
    let lock = plugin.join("uv.lock");
    let shipped = pins(&std::fs::read_to_string(&lock).unwrap());
    run(&plugin, "mise", &["exec", "--", "uv", "lock", "--offline", "--project", "tools/python"]);
    assert_eq!(pins(&std::fs::read_to_string(&lock).unwrap()), shipped, "the staged wheels satisfy the shipped Python pins");
    // The workspace lockfile keeps Harmos at the revision this tree pins.
    std::fs::copy(workspace().join("Cargo.lock"), plugin.join("Cargo.lock")).unwrap();
    // The Archive Producer Binds A Committed Tree, As The Author's Repository Would Be.
    run(&plugin, "git", &["init", "--initial-branch=feat/starter"]);
    run(&plugin, "git", &["add", "."]);
    run(
        &plugin,
        "git",
        &[
            "-c",
            "user.name=Dokime Fixture",
            "-c",
            "user.email=fixture@dokime.invalid",
            "commit",
            "-q",
            "-m",
            "feat: create authored plugin",
        ],
    );

    task(&plugin, "check");
    assert!(plugin.join(".venv").is_dir(), "native uv workspace preparation creates the root environment");
    assert!(!plugin.join("tools/python/.venv").exists(), "tooling does not create a nested environment");
    // Exercise The Generated Project's Actual Archive Producer Before Its
    // Unit Lane Reads That Archive. This Is Authoring Behavior Under Test.
    task(&plugin, "build");
    task(&plugin, "build:rust:tests");
    task(&plugin, "test:rust:unit");
    task(&plugin, "package:rust");
}
