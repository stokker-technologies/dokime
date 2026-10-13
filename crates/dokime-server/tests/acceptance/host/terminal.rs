/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
#[path = "../../support/shutdown.rs"]
mod shutdown;

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::process::Command;
// Third Party Imports

// Local Imports
use shutdown::measure_shutdown;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
async fn ready_then_shutdown() {
    measure_shutdown(false, true, false, None, std::time::Duration::ZERO).await;
}

fn browser_origins_are_validated_before_runtime_start() {
    let root = tempfile::tempdir().unwrap();
    let application = root.path().join("application");
    for origins in [
        vec!["*".to_owned()],
        vec!["null".to_owned()],
        vec!["https://frontend.example/path".to_owned()],
        vec!["https://user@frontend.example".to_owned()],
        vec![
            "tauri://localhost".to_owned(),
            "tauri://localhost".to_owned(),
        ],
        (0..33)
            .map(|index| format!("https://frontend{index}.example"))
            .collect(),
    ] {
        let mut command = Command::new(super::executable());
        command.args(["--color", "never", "--application-root"]);
        command.arg(&application);
        for origin in origins {
            command.arg("--allowed-origin").arg(origin);
        }
        let output = command.output().unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!application.exists());
        assert!(String::from_utf8_lossy(&output.stderr).contains("origin"));
    }
}

fn version_identity() {
    let output = Command::new(super::executable())
        .args(["--color", "always", "version", "--json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(value["dev_protocol"], 4);
    assert!(
        value["sdk_revision"].is_null()
            || value["sdk_revision"]
                .as_str()
                .is_some_and(|value| value.len() == 40)
    );
    let output = Command::new(super::executable())
        .args(["version"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        format!("{}\n", env!("CARGO_PKG_VERSION"))
    );
}

fn stderr_color_policy() {
    for (mode, no_color, colored) in [
        ("auto", false, false),
        ("auto", true, false),
        ("always", false, true),
        ("never", false, false),
    ] {
        let mut command = Command::new(super::executable());
        command.args([
            "--color",
            mode,
            "--message-limit-mib",
            &usize::MAX.to_string(),
        ]);
        command.env_remove("NO_COLOR").env_remove("CLICOLOR_FORCE");
        if no_color {
            command.env("NO_COLOR", "1");
        }
        let output = command.output().unwrap();
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("message limit must be between 1 and 16 MiB"));
        assert_eq!(stderr.contains('\u{1b}'), colored);
    }
}

fn control_requires_stdin() {
    let output = Command::new(super::executable())
        .args(["--dev-control"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("--shutdown-stdin"));
}

fn authentication_file_is_read_bounded_and_validated() {
    let root = tempfile::tempdir().unwrap();
    let auth = root.path().join("auth.json");
    for (bytes, expected) in [
        (br#"{"grants":[]}"#.to_vec(), "requires 1 to 128 grants"),
        (b"invalid-json".to_vec(), "invalid authentication file"),
        (
            vec![b' '; 1024 * 1024 + 1],
            "input file exceeds 1048576 bytes",
        ),
    ] {
        std::fs::write(&auth, bytes).unwrap();
        let output = Command::new(super::executable())
            .args(["--auth-file"])
            .arg(&auth)
            .output()
            .unwrap();
        assert!(!output.status.success());
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(expected),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    std::fs::write(&auth, br#"{"grants":[{"subject":"operator","role":"admin","token":"01234567890123456789012345678901"}]}"#).unwrap();
    let output = Command::new(super::executable())
        .args(["--auth-file"])
        .arg(&auth)
        .arg("--env-file")
        .arg(root.path().join("missing.env"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("cannot load environment file"),
        "valid authentication file must pass before environment loading: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

async fn environment_file_precedence_and_missing_file_errors() {
    let root = tempfile::tempdir().unwrap();
    let env = root.path().join("settings.env");
    for exported in [false, true] {
        let scenario = root.path().join(if exported { "exported" } else { "file" });
        std::fs::create_dir(&scenario).unwrap();
        let from_file = scenario.join("from-file");
        let existing = scenario.join("already-exported");
        std::fs::write(
            &env,
            format!("DOKIME_APPLICATION_ROOT='{}'\n", from_file.display()),
        )
        .unwrap();
        let mut command = Command::new(super::executable());
        command
            .args(["--env-file"])
            .arg(&env)
            .args([
                "--shutdown-stdin",
                "--port",
                "0",
                "--no-seed-validation-artifact",
            ])
            .current_dir(&scenario)
            .env_remove("DOKIME_APPLICATION_ROOT")
            .env_remove("DOKIME_OUTPUT_ROOT");
        if exported {
            command.env("DOKIME_APPLICATION_ROOT", &existing);
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let (selected, unselected) = if exported {
            (&existing, &from_file)
        } else {
            (&from_file, &existing)
        };
        assert!(selected.join("settings.json").is_file());
        assert!(!unselected.exists());
    }
    let output = Command::new(super::executable())
        .args(["--env-file"])
        .arg(root.path().join("missing.env"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot load environment file"));
    let malformed = root.path().join("malformed");
    std::fs::create_dir(&malformed).unwrap();
    std::fs::write(
        malformed.join(".env"),
        "DOKIME_APPLICATION_ROOT='unterminated\n",
    )
    .unwrap();
    let output = Command::new(super::executable())
        .current_dir(&malformed)
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot load project .env file"));
}

#[cfg(unix)]
async fn ready_then_sigint_shutdown() {
    measure_shutdown(true, true, false, None, std::time::Duration::ZERO).await;
}

#[tokio::test]
async fn terminal_contract() {
    ready_then_shutdown().await;
    version_identity();
    stderr_color_policy();
    browser_origins_are_validated_before_runtime_start();
    control_requires_stdin();
    authentication_file_is_read_bounded_and_validated();
    environment_file_precedence_and_missing_file_errors().await;
    #[cfg(unix)]
    ready_then_sigint_shutdown().await;
}
