/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
#[path = "../support/shutdown.rs"]
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
            "serve",
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
        .args(["serve", "--dev-control"])
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
            .args(["serve", "--auth-file"])
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
        .args(["serve", "--auth-file"])
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
    use clap::Parser;
    if let Some(file) = std::env::var_os("DOKIME_TEST_ENV_FILE") {
        let expected = std::env::var("DOKIME_TEST_EXPECTED").unwrap();
        let root = tempfile::tempdir().unwrap();
        let invalid = root.path().join("not-a-directory");
        std::fs::write(&invalid, "file").unwrap();
        let cli = dokime_cli::Cli::parse_from([
            std::ffi::OsString::from("dokime"),
            "serve".into(),
            "--env-file".into(),
            file,
            "--output-root".into(),
            invalid.into_os_string(),
            "--application-root".into(),
            root.path().join("application").into_os_string(),
        ]);
        assert!(cli.run().await.is_err());
        assert_eq!(
            std::env::var("DOKIME_SERVE_ENV_FILE_TEST").unwrap(),
            expected
        );
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let env = root.path().join("settings.env");
    std::fs::write(&env, "DOKIME_SERVE_ENV_FILE_TEST=from-file\n").unwrap();
    for (exported, expected) in [
        (None, "from-file"),
        (Some("already-exported"), "already-exported"),
    ] {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args(["--exact", "terminal::terminal_contract", "--nocapture"])
            .env("DOKIME_TEST_ENV_FILE", &env)
            .env("DOKIME_TEST_EXPECTED", expected)
            .env_remove("DOKIME_SERVE_ENV_FILE_TEST");
        if let Some(value) = exported {
            command.env("DOKIME_SERVE_ENV_FILE_TEST", value);
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let output = Command::new(super::executable())
        .args(["serve", "--env-file"])
        .arg(root.path().join("missing.env"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("cannot load environment file"));
}

#[cfg(unix)]
async fn ready_then_sigint_shutdown() {
    measure_shutdown(true, true, false, None, std::time::Duration::ZERO).await;
}

#[tokio::test]
async fn terminal_contract() {
    if std::env::var_os("DOKIME_TEST_ENV_FILE").is_some() {
        environment_file_precedence_and_missing_file_errors().await;
        return;
    }
    ready_then_shutdown().await;
    version_identity();
    stderr_color_policy();
    control_requires_stdin();
    authentication_file_is_read_bounded_and_validated();
    environment_file_precedence_and_missing_file_errors().await;
    #[cfg(unix)]
    ready_then_sigint_shutdown().await;
}
