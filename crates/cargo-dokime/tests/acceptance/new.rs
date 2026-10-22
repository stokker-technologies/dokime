/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

use super::support;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};
// Third Party Imports
use sha2::{Digest, Sha256};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn generate(root: &Path, args: &[&str]) -> Output {
    Command::new(super::support::tool())
        .current_dir(root)
        .args(["dokime", "new"])
        .args(args)
        .env("CARGO_NET_OFFLINE", "true")
        .env("PATH", "")
        .output()
        .unwrap()
}

fn succeeded(output: Output) -> String {
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stderr).unwrap()
}

/* === Tests === */

#[test]
fn standalone_scaffold_offline() {
    let root = support::standalone_directory();
    succeeded(generate(root.path(), &["my-plugin"]));
    let manifest = fs::read_to_string(root.path().join("my-plugin/Cargo.toml")).unwrap();
    let document = manifest.parse::<toml_edit::DocumentMut>().unwrap();
    let sdk = &document["dependencies"]["dokime"];
    assert_eq!(sdk["git"].as_str(), Some("https://github.com/stokker-technologies/dokime"));
    assert_eq!(sdk["branch"].as_str(), Some("dev"));
    assert!(sdk.get("rev").is_none());
    assert_eq!(document["profile"]["dev"]["debug"].as_str(), Some("line-tables-only"));
    assert_eq!(document["profile"]["dev"]["split-debuginfo"].as_str(), Some("unpacked"));
    assert!(!manifest.contains("workspace"));
    assert!(!manifest.contains("harmos"));
    assert!(root.path().join("my-plugin/src/suites/soak/hold.rs").is_file());
    assert!(root.path().join("my-plugin/tests/soak.rs").is_file());
    assert!(!root.path().join("my-plugin/target").exists());
    assert!(fs::read_to_string(root.path().join("my-plugin/src/main.rs")).unwrap().contains("my_plugin::Plugin"));
    // Declared ids come from the owner namespace rather than the crate name, so
    // a package whose name carries an underscore still declares addressable ids.
    assert!(fs::read_to_string(root.path().join("my-plugin/src/suites/soak/hold.rs")).unwrap().contains("id = \"my-plugin.soak.hold\""));

    // The Shipped Starter Includes Its Own Native Workflow And Locked Tooling
    let plugin = root.path().join("my-plugin");
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
    for file in ["mise.toml", "tools/python/src/project_tools/test/run.py"] {
        let contents = fs::read_to_string(plugin.join(file)).unwrap();
        let (marker, body) = contents.split_once('\n').unwrap();
        assert!(marker.starts_with("# Rendered by stkr project update from "));
        let (_, recorded) = marker.rsplit_once("; sha256:").unwrap();
        let digest = Sha256::digest(body.as_bytes());
        let encoded = digest.iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        assert_eq!(recorded, encoded);
        assert!(!contents.contains("{{package}}"), "{file} has no unresolved package placeholder");
    }
}

#[test]
fn workspace_scaffold_preserved() {
    for members in ["[]", "[\"plugins/*\"]", "[\"./plugins/*\"]"] {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("plugins")).unwrap();
        let original = format!(
            "# Keep this comment\n[workspace]\nresolver = \"3\"\nmembers = {members} # members comment\ndefault-members = [] # deliberate\n"
        );
        fs::write(root.path().join("Cargo.toml"), &original).unwrap();
        let output = succeeded(generate(root.path(), &["plugins/smoke-plugin"]));
        assert!(output.contains("cargo dokime dev -p smoke-plugin"));
        let member = fs::read_to_string(root.path().join("plugins/smoke-plugin/Cargo.toml")).unwrap();
        assert!(member.parse::<toml_edit::DocumentMut>().unwrap().get("profile").is_none());
        let updated = fs::read_to_string(root.path().join("Cargo.toml")).unwrap();
        assert!(updated.contains("# Keep this comment"));
        assert!(updated.contains("# members comment"));
        assert!(updated.contains("default-members = [] # deliberate"));
        if members.contains('*') {
            assert_eq!(updated, original);
        } else {
            assert!(updated.contains("\"plugins/smoke-plugin\""));
        }
    }
}

#[test]
fn excluded_scaffold_independent() {
    let root = tempfile::tempdir().unwrap();
    fs::create_dir(root.path().join("plugins")).unwrap();
    let original = "[workspace]\nmembers = [\"plugins/*\"]\nexclude = [\"plugins/private-*\"]\n";
    fs::write(root.path().join("Cargo.toml"), original).unwrap();
    let output = succeeded(generate(root.path(), &["plugins/private-demo"]));
    assert!(!output.contains(" -p "));
    assert_eq!(fs::read_to_string(root.path().join("Cargo.toml")).unwrap(), original);
    assert!(fs::read_to_string(root.path().join("plugins/private-demo/Cargo.toml")).unwrap().contains("[workspace]"));
}

#[test]
fn invalid_scaffold_refused() {
    let root = tempfile::tempdir().unwrap();
    for name in ["self", "gen", "con", "lpt1", "has space", "9plugin", "Bad", "test"] {
        assert!(!generate(root.path(), &["plugin", "--name", name]).status.success());
        assert!(!root.path().join("plugin").exists());
    }
    fs::create_dir(root.path().join("existing")).unwrap();
    fs::write(root.path().join("existing/keep"), "original").unwrap();
    assert!(!generate(root.path(), &["existing"]).status.success());
    assert_eq!(fs::read_to_string(root.path().join("existing/keep")).unwrap(), "original");
    let malformed = "[workspace]\nmembers = 42\n";
    fs::write(root.path().join("Cargo.toml"), malformed).unwrap();
    assert!(!generate(root.path(), &["plugin"]).status.success());
    assert!(!root.path().join("plugin").exists());
    assert_eq!(fs::read_to_string(root.path().join("Cargo.toml")).unwrap(), malformed);
}
