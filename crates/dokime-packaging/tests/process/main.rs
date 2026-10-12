/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
#[cfg(unix)]
pub(crate) mod tests {
    use super::*;

    pub(crate) async fn no_deadline_completion() {
        let output = run(
            {
                let mut command = Command::new("sh");
                command.args(["-c", "sleep 0.1; printf complete"]);
                command
            },
            "Cargo compilation",
            None,
            32,
            32,
        )
        .await
        .unwrap();
        assert_eq!(output.stdout, b"complete");
    }

    pub(crate) async fn status_output_bounded() {
        let output = run(
            {
                let mut command = Command::new("sh");
                command.args(["-c", "printf good; printf note >&2"]);
                command
            },
            "fixture",
            Some(Duration::from_secs(2)),
            4,
            4,
        )
        .await
        .unwrap();
        assert_eq!(output.stdout, b"good");
        assert_eq!(output.stderr, b"note");
        for script in ["printf overflow", "printf overflow >&2"] {
            assert!(matches!(
                run(
                    {
                        let mut command = Command::new("sh");
                        command.args(["-c", script]);
                        command
                    },
                    "fixture",
                    Some(Duration::from_secs(2)),
                    4,
                    4
                )
                .await,
                Err(Error::Output(_, 4))
            ));
        }
        assert!(
            matches!(run({ let mut command = Command::new("sh"); command.args(["-c", "printf refused >&2; exit 7"]); command }, "fixture", Some(Duration::from_secs(2)), 32, 32).await, Err(Error::Process { status, stderr, .. }) if status.code() == Some(7) && stderr == "refused")
        );
    }

    pub(crate) async fn timeout_kills_child() {
        let directory = tempfile::tempdir().unwrap();
        let pid = directory.path().join("pid");
        let result = run(
            {
                let mut command = Command::new("sh");
                command
                    .args(["-c", "echo $$ > \"$1\"; while :; do :; done", "fixture"])
                    .arg(&pid);
                command
            },
            "catalog export",
            Some(Duration::from_millis(250)),
            32,
            32,
        )
        .await;
        assert!(matches!(result, Err(Error::Timeout(name)) if name == "catalog export"));
        let pid = std::fs::read_to_string(pid).unwrap();
        let status = tokio::process::Command::new("kill")
            .args(["-0", pid.trim()])
            .stderr(Stdio::null())
            .status()
            .await
            .unwrap();
        assert!(!status.success(), "timed-out direct child is still present");
    }
}

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::process::Command;

#[cfg(unix)]
use std::process::Stdio;
use std::time::Duration;
// Third Party Imports

// Local Imports
use dokime_packaging::{Error, process::run};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

pub(crate) async fn deadline_stops_descendants() {
    let directory = tempfile::tempdir().unwrap();
    let heartbeat = directory.path().join("heartbeat");
    let executable = std::env::current_exe().unwrap();
    let cargo = directory
        .path()
        .join(if cfg!(windows) { "cargo.cmd" } else { "cargo" });
    let launcher = if cfg!(windows) {
        "@echo off\r\nset \"DOKIME_PROCESS_TEST_MODE=parent\"\r\n\"%DOKIME_PROCESS_TEST_EXE%\" --exact helper --nocapture\r\n"
    } else {
        "#!/bin/sh\nexport DOKIME_PROCESS_TEST_MODE=parent\nexec \"$DOKIME_PROCESS_TEST_EXE\" --exact helper --nocapture\n"
    };
    std::fs::write(&cargo, launcher).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&cargo, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let mut command = Command::new(&executable);
    command
        .args(["--exact", "helper", "--nocapture"])
        .env("DOKIME_PROCESS_TEST_MODE", "build")
        .env("DOKIME_PROCESS_TEST_EXE", &executable)
        .env("DOKIME_PROCESS_TEST_HEARTBEAT", &heartbeat)
        .env("CARGO", &cargo);
    run(
        command,
        "packaging deadline fixture",
        Some(Duration::from_secs(10)),
        64 * 1024,
        64 * 1024,
    )
    .await
    .unwrap();
    let before = std::fs::read(&heartbeat).expect("descendant started");
    assert!(!before.is_empty());
    tokio::time::sleep(Duration::from_millis(150)).await;
    assert_eq!(
        std::fs::read(&heartbeat).unwrap(),
        before,
        "compiler descendant survived timeout"
    );
}

#[tokio::test]
async fn helper() {
    let Ok(mode) = std::env::var("DOKIME_PROCESS_TEST_MODE") else {
        return;
    };
    if mode == "build" {
        let result = dokime_packaging::build::run(dokime_packaging::build::Options {
            cargo_timeout: Some(Duration::from_secs(2)),
            ..Default::default()
        })
        .await;
        assert!(
            matches!(result, Err(Error::Timeout(operation)) if operation == "Resolving package metadata")
        );
    } else if mode == "parent" {
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "helper", "--nocapture"])
            .env("DOKIME_PROCESS_TEST_MODE", "descendant")
            .spawn()
            .unwrap();
        child.wait().unwrap();
    } else {
        use std::io::Write;
        let mut heartbeat = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(std::env::var_os("DOKIME_PROCESS_TEST_HEARTBEAT").unwrap())
            .unwrap();
        for _ in 0..3000 {
            heartbeat.write_all(b".").unwrap();
            heartbeat.flush().unwrap();
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

/* === Tests === */

fn fresh_runtime(
    multithread: bool,
    threads: Option<usize>,
    paused: bool,
) -> tokio::runtime::Runtime {
    let _ = multithread;
    let mut builder = tokio::runtime::Builder::new_current_thread();
    builder.enable_all();
    if let Some(threads) = threads {
        builder.worker_threads(threads);
    }
    let _ = paused;
    builder.build().expect("scenario runtime")
}

#[test]
fn domain_contract() {
    #[cfg(unix)]
    {
        eprintln!("Scenario: tests::no_deadline_completion");
        run_scenario(tests::no_deadline_completion, false, None, false);
    }
    #[cfg(unix)]
    {
        eprintln!("Scenario: tests::status_output_bounded");
        run_scenario(tests::status_output_bounded, false, None, false);
    }
    #[cfg(unix)]
    {
        eprintln!("Scenario: tests::timeout_kills_child");
        run_scenario(tests::timeout_kills_child, false, None, false);
    }
    {
        eprintln!("Scenario: deadline_stops_descendants");
        run_scenario(deadline_stops_descendants, false, None, false);
    }
}

#[inline(never)]
fn run_scenario<F: std::future::Future>(
    factory: impl FnOnce() -> F,
    multithread: bool,
    threads: Option<usize>,
    paused: bool,
) -> F::Output {
    fresh_runtime(multithread, threads, paused).block_on(Box::pin(factory()))
}
