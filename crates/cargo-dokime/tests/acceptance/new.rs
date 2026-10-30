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
use std::{fs, path::Path, process::Output};
// Third Party Imports

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
    let mut command = dokime::domain::process::command(super::support::tool());
    command
        .current_dir(root)
        .args(["dokime", "sidecar", "new"])
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
    assert_eq!(sdk["tag"].as_str(), Some(concat!("v", env!("CARGO_PKG_VERSION"))));
    let host = &document["dependencies"]["plugin-host"];
    assert_eq!(host["git"].as_str(), sdk["git"].as_str());
    assert_eq!(host["tag"].as_str(), sdk["tag"].as_str());
    assert_eq!(host["default-features"].as_bool(), Some(false));
    assert!(sdk.get("rev").is_none());
    assert!(document.get("profile").is_none(), "the starter uses Cargo profile defaults");
    assert!(!manifest.contains("workspace"));
    assert!(!manifest.contains("harmos"));
    assert!(root.path().join("my-plugin/src/suites/soak/hold.rs").is_file());
    assert!(root.path().join("my-plugin/tests/soak.rs").is_file());
    assert!(!root.path().join("my-plugin/target").exists());
    assert!(fs::read_to_string(root.path().join("my-plugin/src/main.rs")).unwrap().contains("my_plugin::Plugin"));
    // The plugin id is the crate name, one Harmos owner segment, and every
    // declared id is scoped under it, so a hyphenated package stays addressable.
    assert!(fs::read_to_string(root.path().join("my-plugin/src/lib.rs")).unwrap().contains("id = \"my_plugin\""));
    assert!(fs::read_to_string(root.path().join("my-plugin/src/suites/soak/hold.rs")).unwrap().contains("id = \"my_plugin.soak.hold\""));

    // The Shipped Starter Includes Its Own Native Workflow And Locked Tooling
    let plugin = root.path().join("my-plugin");
    for file in [
        "mise.toml",
        ".stkr/project.toml",
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
    assert!(!plugin.join("tools/python/uv.lock").exists(), "the tooling uses the root workspace lock");
    let python = fs::read_to_string(plugin.join("pyproject.toml")).unwrap().parse::<toml_edit::DocumentMut>().unwrap();
    let members = python["tool"]["uv"]["workspace"]["members"].as_array().unwrap();
    assert!(members.iter().any(|member| member.as_str() == Some("tools/python")));
    for file in ["mise.toml", "tools/python/src/project_tools/test/run.py"] {
        let contents = fs::read_to_string(plugin.join(file)).unwrap();
        let public_contents = contents.replace(".stkr", "").to_ascii_lowercase();
        for word in public_contents.split(|character: char| !character.is_ascii_alphanumeric() && character != '_') {
            assert!(!matches!(word, "hub" | "stkr"), "{file} has no internal tooling mentions");
        }
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
        assert!(output.contains("cargo dokime sidecar dev -p smoke-plugin"));
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

#[test]
fn sidecar_commands_grouped() {
    let root = tempfile::tempdir().unwrap();
    for leaf in ["new", "build", "package", "dev", "inspect"] {
        for cargo_prefix in [false, true] {
            let mut command = dokime::domain::process::command(support::tool());
            command.current_dir(root.path()).env("PATH", "");
            if cargo_prefix {
                command.arg("dokime");
            }
            let output = command.args(["sidecar", leaf, "--help"]).output().unwrap();
            assert!(output.status.success(), "{leaf}: {}", String::from_utf8_lossy(&output.stderr));
        }
        let mut command = dokime::domain::process::command(support::tool());
        let output = command.current_dir(root.path()).args(["dokime", leaf, "--help"]).output().unwrap();
        assert_eq!(output.status.code(), Some(2), "top-level {leaf} must be rejected");
    }
}
