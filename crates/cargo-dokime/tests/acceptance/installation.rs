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
use std::{path::Path, process::Command};
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
fn installation() -> tempfile::TempDir {
    let backend = std::path::PathBuf::from(
        std::env::var_os("DOKIME_TEST_HOST").expect("mise selects the exact prepared server"),
    )
    .canonicalize()
    .expect("prepared server exists");
    let directory = tempfile::Builder::new()
        .prefix("dokime-installation-")
        .tempdir_in(super::support::target_directory())
        .unwrap();
    std::fs::hard_link(
        &backend,
        directory.path().join(backend.file_name().unwrap()),
    )
    .unwrap();
    directory
}

#[cfg(unix)]
fn stub_gui(directory: &Path, arguments: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let path = directory.join("dokime-gui");
    std::fs::write(
        &path,
        format!("#!/bin/sh\nprintf '%s' \"$*\" > {arguments:?}\n"),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

fn cargo_dokime(home: &Path) -> Command {
    let mut command = Command::new(super::support::tool());
    command.env("DOKIME_HOME", home);
    command
}

/* === Tests === */
fn dokime_home_supplies_backend_identity_without_override() {
    let installation = installation();
    let output = cargo_dokime(installation.path())
        .args(["version", "--json"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let identity: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(identity["dev_protocol"], 4);
    assert_eq!(
        identity["version"].as_str().unwrap(),
        env!("CARGO_PKG_VERSION"),
        "the discovered host shares this tool's compatibility series"
    );
}

fn missing_gui_names_installer_and_both_overrides() {
    let installation = installation();
    let output = cargo_dokime(installation.path())
        .args(["gui", "--endpoint", "http://127.0.0.1:8765"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let refusal = String::from_utf8_lossy(&output.stderr);
    for expected in ["desktop installer", "--dokime", "--gui"] {
        assert!(refusal.contains(expected), "{refusal}");
    }
}

#[cfg(unix)]
fn the_gui_beside_the_discovered_backend_receives_the_endpoint() {
    let installation = installation();
    let arguments = installation.path().join("arguments");
    stub_gui(installation.path(), &arguments);
    let output = cargo_dokime(installation.path())
        .args(["gui", "--endpoint", "http://127.0.0.1:8765"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(&arguments).unwrap(),
        "--endpoint http://127.0.0.1:8765"
    );
}

fn explicit_backend_paths_search_hits_and_missing_paths() {
    let installed = installation();
    let backend = installed
        .path()
        .join(format!("dokime-server{}", std::env::consts::EXE_SUFFIX));
    for selected in [&backend, Path::new("dokime-server")] {
        let output = cargo_dokime(installed.path())
            .env("PATH", installed.path())
            .args(["version", "--json", "--dokime"])
            .arg(selected)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let missing = installed.path().join("missing");
    let output = cargo_dokime(installed.path())
        .args(["version", "--json", "--dokime"])
        .arg(&missing)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("not a file"));
    let output = cargo_dokime(installed.path())
        .args(["gui", "--endpoint", "http://127.0.0.1:8765", "--gui"])
        .arg(&missing)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("not a regular file"));
}

#[cfg(unix)]
fn explicit_gui_and_backend_shims_preserve_selected_paths() {
    let installed = installation();
    let shim = installed.path().join("dokime-shim");
    std::os::unix::fs::symlink(installed.path().join("dokime-server"), &shim).unwrap();
    let output = cargo_dokime(installed.path())
        .args(["version", "--json", "--dokime"])
        .arg(&shim)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let arguments = installed.path().join("arguments");
    stub_gui(installed.path(), &arguments);
    let explicit = installed.path().join("custom-gui");
    std::fs::rename(installed.path().join("dokime-gui"), &explicit).unwrap();
    let output = Command::new(super::support::tool())
        .args(["gui", "--endpoint", "http://127.0.0.1:8765", "--gui"])
        .arg(&explicit)
        .env_remove("DOKIME_HOME")
        .env("PATH", "")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(arguments).unwrap(),
        "--endpoint http://127.0.0.1:8765"
    );
}

pub(super) fn installation_contract() {
    dokime_home_supplies_backend_identity_without_override();
    missing_gui_names_installer_and_both_overrides();
    #[cfg(unix)]
    the_gui_beside_the_discovered_backend_receives_the_endpoint();
    explicit_backend_paths_search_hits_and_missing_paths();
    #[cfg(unix)]
    explicit_gui_and_backend_shims_preserve_selected_paths();
}
