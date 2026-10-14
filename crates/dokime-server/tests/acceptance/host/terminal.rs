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
    for (web, origins) in [
        (false, vec!["tauri://localhost".to_owned()]),
        (true, vec!["*".to_owned()]),
        (true, vec!["null".to_owned()]),
        (true, vec!["https://frontend.example/path".to_owned()]),
        (true, vec!["https://user@frontend.example".to_owned()]),
        (
            true,
            vec![
                "tauri://localhost".to_owned(),
                "tauri://localhost".to_owned(),
            ],
        ),
        (
            true,
            (0..33)
                .map(|index| format!("https://frontend{index}.example"))
                .collect(),
        ),
    ] {
        let mut command = Command::new(super::executable());
        command.args(["--color", "never", "--application-root"]);
        command.arg(&application);
        if web {
            command.arg("--grpc-web");
        }
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

fn owned_launch(application: &std::path::Path) -> std::process::Command {
    // The Arguments The Desktop Shell Passes Its Owned Backend
    let mut command = Command::new(super::executable());
    command
        .args([
            "--color",
            "never",
            "--dev-control",
            "--shutdown-stdin",
            "--port",
            "0",
            "--application-root",
        ])
        .arg(application)
        .env_remove("DOKIME_OUTPUT_ROOT")
        .env("RUST_LOG", "off");
    command
}

fn same_root_relaunches_ready() {
    use std::io::{BufRead, BufReader};
    use std::process::Stdio;
    let root = tempfile::tempdir().unwrap();
    let application = root.path().join("application");
    for launch in 0..3 {
        // Await The Readiness Line On The Default Data Root
        let mut child = owned_launch(&application)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut events = BufReader::new(child.stdout.take().unwrap());
        let ready = loop {
            let mut line = String::new();
            if events.read_line(&mut line).unwrap() == 0 {
                break None;
            }
            let event: serde_json::Value = serde_json::from_str(&line).unwrap();
            if event["event"] != "phase" {
                break Some(event);
            }
        };

        // Drain Through The Owned Shutdown Path
        drop(child.stdin.take());
        let output = child.wait_with_output().unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(
            ready.as_ref().map(|event| &event["event"]),
            Some(&serde_json::json!("ready")),
            "launch {launch} did not report readiness: {stderr}"
        );
        assert!(
            output.status.success(),
            "launch {launch} did not drain cleanly: {stderr}"
        );
    }
    assert!(application.join("data").join("local.sqlite").is_file());
}

async fn incompatible_storage_is_refused_by_name() {
    use dokime::storage::sqlite::{self, rusqlite};
    let root = tempfile::tempdir().unwrap();
    let application = root.path().join("application");
    let data = application.join("data");
    std::fs::create_dir_all(&data).unwrap();

    // Write A Root From Another Baseline
    sqlite::initialize_sqlite().unwrap();
    let connection = rusqlite::Connection::open(data.join("local.sqlite")).unwrap();
    connection
        .execute_batch("PRAGMA user_version=7; CREATE TABLE foreign_evidence(payload BLOB);")
        .unwrap();
    drop(connection);
    let original = std::fs::read(data.join("local.sqlite")).unwrap();

    // The Process Exits With Its Own Code And A Remedy On Stderr Only
    let output = owned_launch(&application).output().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(4), "{stderr}");
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("\"ready\""),
        "a refused root never reports readiness"
    );
    for expected in [
        data.display().to_string(),
        "schema version 7".to_owned(),
        "this build reads version 1".to_owned(),
        "Move or delete that directory".to_owned(),
        "DOKIME_OUTPUT_ROOT".to_owned(),
        "settings.json data_root".to_owned(),
    ] {
        assert!(
            stderr.contains(&expected),
            "missing {expected:?} in {stderr}"
        );
    }
    assert!(!stderr.contains("backtrace"), "{stderr}");
    assert_eq!(std::fs::read(data.join("local.sqlite")).unwrap(), original);

    // The Same Refusal Is Typed In Process
    let mut config = dokime_app::config::Config::local(&data);
    config.application_root = Some(application.clone());
    config.seed_plugins = false;
    let error = dokime_app::app::launch(config).await.err().unwrap();
    let error = dokime_server::host::Error::launch(error, data.clone());
    assert!(
        matches!(&error, dokime_server::host::Error::IncompatibleStorage { root, .. } if root == &data),
        "{error}"
    );
    assert_eq!(error.exit_code(), std::process::ExitCode::from(4));
    assert_eq!(
        error.to_string().lines().count(),
        3,
        "one sentence per line"
    );

    // A Root Owned By Another Runtime Is Not Reported As Incompatible
    let fresh = root.path().join("owned");
    let owner = sqlite::State::open(&fresh).await.unwrap();
    let mut config = dokime_app::config::Config::local(&fresh);
    config.application_root = Some(application);
    config.seed_plugins = false;
    let error = dokime_app::app::launch(config).await.err().unwrap();
    let error = dokime_server::host::Error::launch(error, fresh);
    assert!(
        matches!(&error, dokime_server::host::Error::Runtime(_)),
        "{error}"
    );
    assert_eq!(error.exit_code(), std::process::ExitCode::from(3));
    owner.close().await.unwrap();
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
    same_root_relaunches_ready();
    incompatible_storage_is_refused_by_name().await;
    #[cfg(unix)]
    ready_then_sigint_shutdown().await;
}
