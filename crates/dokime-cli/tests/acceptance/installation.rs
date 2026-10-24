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
use std::path::Path;
use std::process::Command;

// Third Party Imports

// Local Imports

/* === Constants === */

const ENDPOINT: &str = "http://127.0.0.1:8765";

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn installation(programs: &[&Path]) -> tempfile::TempDir {
    let directory = tempfile::Builder::new().prefix("dokime-installation-").tempdir().unwrap();
    for program in programs {
        let program = program.canonicalize().expect("the program under test exists");
        let destination = directory.path().join(program.file_name().unwrap());
        if let Err(error) = std::fs::hard_link(&program, &destination) {
            assert_eq!(error.kind(), std::io::ErrorKind::CrossesDevices, "installation fixture link: {error}");
            std::fs::copy(&program, destination).unwrap();
        }
    }
    directory
}

fn dokime(directory: &Path) -> Command {
    let executable = directory.join(format!("dokime{}", std::env::consts::EXE_SUFFIX));
    let mut command = Command::new(executable);
    #[cfg(windows)]
    command.creation_flags(0x0800_0000);
    // The story's own directory is the whole installation: a Dokime installed on the host is never found.
    command.env("DOKIME_HOME", directory);
    command
}

#[cfg(unix)]
fn stub_gui(path: &Path, arguments: &Path) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(path, format!("#!/bin/sh\nprintf '%s' \"$*\" > {arguments:?}\n")).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
}

fn succeeded(output: std::process::Output) -> String {
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8(output.stdout).unwrap()
}

fn refused(output: std::process::Output) -> String {
    assert!(!output.status.success());
    String::from_utf8(output.stderr).unwrap()
}

/* === Tests === */

// The CLI hosts its server without a companion executable or installation lookup.
fn embedded_server_identity() {
    let cli = cli_executable();
    let installed = installation(&[Path::new(&cli)]);
    let output = dokime(installed.path()).args(["server", "version", "--json"]).output().unwrap();
    let identity: serde_json::Value = serde_json::from_str(&succeeded(output)).unwrap();
    assert_eq!(identity["version"].as_str(), Some(env!("CARGO_PKG_VERSION")));
    assert!(identity["dev_protocol"].is_u64());
}

// a missing program names the installer and the explicit overrides.

fn missing_programs_refused() {
    let cli = cli_executable();
    let cli = Path::new(&cli);
    let installed = installation(&[cli]);

    let refusal = refused(dokime(installed.path()).args(["gui", "--endpoint", ENDPOINT]).output().unwrap());
    for expected in ["dokime-gui", "desktop installer", "DOKIME_HOME"] {
        assert!(refusal.contains(expected), "{refusal}");
    }
}

// the GUI receives the endpoint and the arguments after --.

#[cfg(unix)]
fn gui_receives_endpoint() {
    let cli = cli_executable();
    let cli = Path::new(&cli);
    let installed = installation(&[cli]);
    let arguments = installed.path().join("arguments");

    // Installed Beside The CLI
    stub_gui(&installed.path().join("dokime-gui"), &arguments);
    let output = dokime(installed.path()).args(["gui", "--endpoint", ENDPOINT, "--", "--shutdown-stdin"]).output().unwrap();
    succeeded(output);
    assert_eq!(std::fs::read_to_string(&arguments).unwrap(), format!("--endpoint {ENDPOINT} --shutdown-stdin"));

    // Selected Explicitly
    let explicit = installed.path().join("custom-gui");
    std::fs::rename(installed.path().join("dokime-gui"), &explicit).unwrap();
    let output = dokime(installed.path()).args(["gui", "--endpoint", ENDPOINT, "--gui"]).arg(&explicit).output().unwrap();
    succeeded(output);
    assert_eq!(std::fs::read_to_string(&arguments).unwrap(), format!("--endpoint {ENDPOINT}"));
}

// The built executable: nextest names it at run time, so an archive extracted on another runner finds it.
fn cli_executable() -> String {
    std::env::var("NEXTEST_BIN_EXE_dokime")
        .ok()
        .or_else(|| option_env!("CARGO_BIN_EXE_dokime").map(str::to_owned))
        .expect("workspace executable")
}

/* === Tests === */

#[test]
fn installation_contract() {
    {
        eprintln!("Scenario: embedded_server_identity");
        embedded_server_identity();
    }
    {
        eprintln!("Scenario: missing_programs_refused");
        missing_programs_refused();
    }
    #[cfg(unix)]
    {
        eprintln!("Scenario: gui_receives_endpoint");
        gui_receives_endpoint();
    }
}
