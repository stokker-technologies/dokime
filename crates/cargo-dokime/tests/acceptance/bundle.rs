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
use std::path::Path;
use std::process::{Command, Output};
// Third Party Imports
use harmos::plugin::bundle::{Bundle, Limits};
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
fn cli(root: &Path) -> Command {
    let mut command = Command::new(super::support::tool());
    command.current_dir(root).arg("dokime");
    command
}

fn success(output: Output) -> Output {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/* === Tests === */
fn help_and_parse_errors_need_no_cargo_workspace() {
    let directory = tempfile::tempdir().unwrap();
    success(cli(directory.path()).arg("--help").output().unwrap());
    success(
        cli(directory.path())
            .args(["build", "--help"])
            .output()
            .unwrap(),
    );
    let output = cli(directory.path()).arg("unknown").output().unwrap();
    assert!(!output.status.success());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("Cargo.toml"));
}

#[cfg(unix)]
fn cargo_deadlines_identify_the_operation() {
    use std::os::unix::fs::PermissionsExt;

    let directory = tempfile::tempdir().unwrap();
    let cargo = directory.path().join("cargo");
    std::fs::write(&cargo, "#!/bin/sh\nexec sleep 30\n").unwrap();
    std::fs::set_permissions(&cargo, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::write(directory.path().join("Cargo.toml"), "").unwrap();
    let output = cli(directory.path())
        .args(["build", "--cargo-timeout", "1"])
        .env("CARGO", &cargo)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("package metadata"), "{stderr}");
    assert!(
        stderr.contains("exceeded its execution timeout"),
        "{stderr}"
    );
    std::fs::write(
        directory.path().join("Cargo.toml"),
        r#"[package]
name = "deadline-fixture"
version = "0.1.0"
[workspace]
[[bin]]
name = "deadline-fixture"
path = "main.rs"
"#,
    )
    .unwrap();
    std::fs::write(directory.path().join("main.rs"), "fn main() {}").unwrap();
    std::fs::write(
        &cargo,
        r#"#!/bin/sh
if [ "$1" = metadata ]; then exec "$REAL_CARGO" "$@"; fi
exec sleep 30
"#,
    )
    .unwrap();
    let output = cli(directory.path())
        .args(["build", "--cargo-timeout", "1"])
        .env("REAL_CARGO", env!("CARGO"))
        .env("CARGO", &cargo)
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Cargo compilation exceeded its execution timeout"),
        "{stderr}"
    );
}

fn native_package_builds_and_inspects_without_lifecycle() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    std::fs::create_dir(root.join("src")).unwrap();
    std::fs::create_dir(root.join("assets")).unwrap();
    let sdk = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../dokime")
        .canonicalize()
        .unwrap();
    let manifest = format!(
        r#"[package]
name = "independent-plugin"
version.workspace = true
edition = "2024"
description.workspace = true
authors.workspace = true
license.workspace = true
repository.workspace = true
homepage.workspace = true
documentation.workspace = true
keywords.workspace = true
categories.workspace = true
[dependencies]
dokime = {{ path = {:?} }}
[workspace]
[workspace.package]
version = "1.2.3"
description = "Cargo owns the bundle identity"
authors = ["Fixture Author"]
license = "MIT"
repository = "https://example.com/repository"
homepage = "https://example.com/home"
documentation = "https://example.com/docs"
keywords = ["fixture"]
categories = ["development-tools"]
"#,
        sdk
    );
    std::fs::write(root.join("Cargo.toml"), &manifest).unwrap();
    std::fs::write(
        root.join("src/main.rs"),
        include_str!("../fixtures/plugin.rs"),
    )
    .unwrap();
    std::fs::write(root.join("assets/calibration.bin"), [0, 1, 255]).unwrap();
    let target = super::support::target_directory().join("plugin-cli-fixture");
    let marker = root.join("exported");
    let build = || {
        let mut command = cli(root);
        command
            .args(["build", "--target-dir"])
            .arg(&target)
            .args(["-o", "plugin.dokime"])
            .env("FIXTURE_EXPORT_MARKER", &marker);
        command
    };
    success(build().output().unwrap());
    assert_eq!(std::fs::read(&marker).unwrap(), b"registered");
    let bytes = std::fs::read(root.join("plugin.dokime")).unwrap();
    let bundle = Bundle::inspect(&bytes, Limits::default()).unwrap();
    assert_eq!(bundle.manifest().identity.name, "independent-plugin");
    assert_eq!(bundle.manifest().identity.version.to_string(), "1.2.3");
    assert_eq!(
        bundle.manifest().identity.description,
        "Cargo owns the bundle identity"
    );
    let identity = &bundle.manifest().identity.package;
    assert_eq!(identity.authors, ["Fixture Author"]);
    assert_eq!(identity.license.as_deref(), Some("MIT"));
    assert_eq!(
        identity.repository.as_deref(),
        Some("https://example.com/repository")
    );
    assert_eq!(
        identity.homepage.as_deref(),
        Some("https://example.com/home")
    );
    assert_eq!(
        identity.documentation.as_deref(),
        Some("https://example.com/docs")
    );
    assert_eq!(identity.keywords, ["fixture"]);
    assert_eq!(identity.categories, ["development-tools"]);
    assert_eq!(bundle.file("assets/calibration.bin").unwrap(), &[0, 1, 255]);
    let executable = bundle
        .manifest()
        .files
        .iter()
        .find(|file| file.executable.is_some())
        .unwrap();
    assert!(
        !executable
            .executable
            .as_ref()
            .unwrap()
            .registrations
            .is_empty()
    );
    std::fs::remove_file(&marker).unwrap();
    let inspection = success(
        cli(root)
            .args(["inspect", "plugin.dokime"])
            .env("FIXTURE_EXPORT_MARKER", &marker)
            .output()
            .unwrap(),
    );
    let inspection: serde_json::Value = serde_json::from_slice(&inspection.stdout).unwrap();
    assert_eq!(
        inspection["declarations"][&executable.path]["id"],
        "independent-owner"
    );
    assert!(!marker.exists(), "inspection must not execute registration");
    assert!(
        !inspection["declarations"][&executable.path]["operations"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    #[cfg(unix)]
    {
        let assets = root.join("assets");
        let saved_assets = root.join("saved-assets");
        std::fs::rename(&assets, &saved_assets).unwrap();
        std::os::unix::fs::symlink(root.join("missing-assets"), &assets).unwrap();

        let output = build().output().unwrap();
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("no links"), "{stderr}");
        assert_eq!(
            std::fs::read(root.join("plugin.dokime")).unwrap(),
            bytes,
            "a dangling asset root must not replace the existing bundle"
        );

        std::fs::remove_file(&assets).unwrap();
        std::fs::rename(saved_assets, assets).unwrap();
    }

    let output = build().env("FIXTURE_INVALID_EXPORT", "1").output().unwrap();
    assert!(!output.status.success());
    assert_eq!(
        std::fs::read(root.join("plugin.dokime")).unwrap(),
        bytes,
        "refusal must preserve the existing bundle"
    );
    let output = cli(root)
        .args(["build", "--target", "wasm32-wasip2"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("native host target"));
    std::fs::write(root.join("Cargo.toml"), manifest.replace("1.2.3", "1.2.4")).unwrap();
    success(build().output().unwrap());
    let updated_bytes = std::fs::read(root.join("plugin.dokime")).unwrap();
    let updated = Bundle::inspect(&updated_bytes, Limits::default()).unwrap();
    assert_eq!(updated.manifest().identity.version.to_string(), "1.2.4");
    let declaration = updated.declaration(&executable.path).unwrap();
    assert!(matches!(
        declaration.carrier,
        harmos::plugin::inventory::CarrierSection::Sidecar { .. }
    ));
    assert_eq!(declaration.version.to_string(), "1.2.4");
    // Bundle identity comes from Cargo; declaration presentation comes from
    // the plugin implementation's documentation.
    assert_eq!(
        declaration.description,
        "A plugin whose bundle is built and inspected, never run."
    );
    std::fs::create_dir(root.join("src/bin")).unwrap();
    std::fs::write(root.join("src/bin/other.rs"), "fn main() {}\n").unwrap();
    let output = build().output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("select one native binary"));

    std::fs::write(
        root.join("src/bin/other.rs"),
        include_str!("../fixtures/plugin.rs").replace("independent-owner", "other-owner"),
    )
    .unwrap();
    let multiple = || {
        let mut command = build();
        command.args(["--bin", "independent-plugin", "--bin", "other"]);
        command
    };
    success(multiple().output().unwrap());
    let multiple_bytes = std::fs::read(root.join("plugin.dokime")).unwrap();
    let multiple_bundle = Bundle::inspect(&multiple_bytes, Limits::default()).unwrap();
    let mut owners = Vec::new();
    for file in &multiple_bundle.manifest().files {
        if file.executable.is_none() {
            continue;
        }
        let declaration = multiple_bundle.declaration(&file.path).unwrap();
        assert!(matches!(
            declaration.carrier,
            harmos::plugin::inventory::CarrierSection::Sidecar { .. }
        ));
        owners.push(declaration.id.clone());
        assert_eq!(declaration.version.to_string(), "1.2.4");
        assert!(!file.executable.as_ref().unwrap().registrations.is_empty());
    }
    owners.sort();
    assert_eq!(owners, ["independent-owner", "other-owner"]);
    // MSVC stores debug information beside each executable; other hosts keep
    // it in the binary. Both bundles contain two executables and one asset.
    let symbols = multiple_bundle
        .manifest()
        .files
        .iter()
        .filter(|file| file.path.ends_with(".pdb"))
        .count();
    assert_eq!(
        symbols,
        if cfg!(all(windows, target_env = "msvc")) {
            2
        } else {
            0
        }
    );
    assert_eq!(multiple_bundle.manifest().files.len(), 3 + symbols);
    assert_eq!(
        multiple_bundle.file("assets/calibration.bin").unwrap(),
        &[0, 1, 255]
    );

    let duplicate = multiple().args(["--bin", "other"]).output().unwrap();
    assert!(!duplicate.status.success());
    assert!(String::from_utf8_lossy(&duplicate.stderr).contains("selected twice"));
    let missing = build().args(["--bin", "missing"]).output().unwrap();
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("has no binary missing"));
    std::fs::write(root.join("src/bin/other.rs"), "fn main() {}\n").unwrap();
    let invalid_member = multiple().output().unwrap();
    assert!(!invalid_member.status.success());
    assert!(
        String::from_utf8_lossy(&invalid_member.stderr).contains("no complete native declaration")
    );
    assert_eq!(
        std::fs::read(root.join("plugin.dokime")).unwrap(),
        multiple_bytes,
        "a bad selected member must not replace the previously complete bundle"
    );
}

pub(super) fn bundle_contract() {
    help_and_parse_errors_need_no_cargo_workspace();
    #[cfg(unix)]
    cargo_deadlines_identify_the_operation();
    native_package_builds_and_inspects_without_lifecycle();
}
