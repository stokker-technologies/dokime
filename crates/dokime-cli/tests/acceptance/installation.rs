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

fn server() -> PathBuf {
    let platform = if cfg!(windows) { "windows" } else { "linux" };
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).ancestors().find(|path| path.join("Cargo.lock").is_file()).unwrap();
    let profile = if cfg!(debug_assertions) { "dev" } else { "release" };
    let target = format!("{platform}-{}", std::env::consts::ARCH);
    let server =
        root.join("target/build").join(profile).join(target).join("bin").join(format!("dokime-server{}", std::env::consts::EXE_SUFFIX));
    assert!(server.is_file(), "run mise run build to produce {}", server.display());
    server
}

fn dokime(directory: &Path) -> Command {
    let executable = directory.join(format!("dokime{}", std::env::consts::EXE_SUFFIX));
    let mut command = Command::new(executable);
    command.env_remove("DOKIME_HOME");
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

// the server beside the CLI answers its identity through serve.

fn server_beside_cli() {
    let cli = cli_executable();
    let cli = Path::new(&cli);
    let installed = installation(&[cli, &server()]);

    let output = dokime(installed.path()).args(["serve", "--", "version", "--json"]).output().unwrap();
    let identity: serde_json::Value = serde_json::from_str(&succeeded(output)).unwrap();

    assert_eq!(identity["version"].as_str(), Some(env!("CARGO_PKG_VERSION")));
    assert!(identity["dev_protocol"].is_u64());
}

// DOKIME_HOME supplies the server when none is installed beside the CLI.

fn home_supplies_server() {
    let cli = cli_executable();
    let cli = Path::new(&cli);
    let installed = installation(&[cli]);
    let home = installation(&[&server()]);

    let output = dokime(installed.path()).env("DOKIME_HOME", home.path()).args(["serve", "--", "version", "--json"]).output().unwrap();
    succeeded(output);
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

    let missing = installed.path().join("missing");
    let refusal = refused(dokime(installed.path()).arg("serve").arg("--server").arg(&missing).output().unwrap());
    assert!(refusal.contains("is not a file"), "{refusal}");
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

#[test]
fn installation_contract() {
    {
        eprintln!("Scenario: server_beside_cli");
        server_beside_cli();
    }
    {
        eprintln!("Scenario: home_supplies_server");
        home_supplies_server();
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

// The built executable: nextest names it at run time, so an archive extracted on another runner finds it.
fn cli_executable() -> String {
    std::env::var("NEXTEST_BIN_EXE_dokime")
        .ok()
        .or_else(|| option_env!("CARGO_BIN_EXE_dokime").map(str::to_owned))
        .expect("workspace executable")
}
