/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// One Process-Ownership Story Covers Output, Refusal, Deadline And Descendant Cleanup.

use std::path::Path;
use std::time::Duration;

use dokime_packaging::{Error, process::run};

// A bounded wait for a process that has already been signalled to disappear;
// the operating system exposes no portable exit notification for a process
// this story does not own.
const GONE_POLL: Duration = Duration::from_millis(5);

#[cfg(unix)]
fn shell(script: &str) -> std::process::Command {
    let mut command = dokime::domain::process::command("sh");
    command.args(["-c", script]);
    command
}

fn alive(pid: &str) -> bool {
    #[cfg(unix)]
    let alive = dokime::domain::process::command("kill").args(["-0", pid]).stderr(std::process::Stdio::null()).status().unwrap().success();
    #[cfg(windows)]
    let alive = {
        let listed = dokime::domain::process::command("tasklist").args(["/FI", &format!("PID eq {pid}"), "/NH"]).output().unwrap();
        String::from_utf8_lossy(&listed.stdout).contains(pid)
    };
    alive
}

async fn wait_gone(pid: &str) {
    while alive(pid) {
        tokio::time::sleep(GONE_POLL).await;
    }
}

fn deadline() -> Duration {
    let policy: toml::Value = toml::from_str(include_str!("../../../../stkr.toml")).unwrap();
    let seconds = policy["conventions"]["testing"]["deadlines"]["acceptance"].as_integer().unwrap();
    Duration::from_secs(seconds.try_into().unwrap())
}

// A Running Blocking Task Inhibits Tokio Auto-Advance Until The Real Child Is Ready.
fn ready(pid: std::path::PathBuf) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn_blocking(move || {
        while !pid.is_file() {
            std::thread::sleep(GONE_POLL);
        }
    })
}

#[cfg(unix)]
async fn no_deadline_completion() {
    let output = run(shell("printf complete"), "Cargo compilation", None, 32, 32).await.unwrap();

    assert_eq!(output.stdout, b"complete");
}

#[cfg(unix)]
async fn status_output_bounded() {
    let output = run(shell("printf good; printf note >&2"), "fixture", None, 4, 4).await.unwrap();
    assert_eq!(output.stdout, b"good");
    assert_eq!(output.stderr, b"note");

    for script in ["printf overflow", "printf overflow >&2"] {
        let overflowed = run(shell(script), "fixture", None, 4, 4).await;
        assert!(matches!(overflowed, Err(Error::Output(_, 4))));
    }

    let refused = run(shell("printf refused >&2; exit 7"), "fixture", None, 32, 32).await;
    assert!(matches!(
        refused,
        Err(Error::Process { status, stderr, .. }) if status.code() == Some(7) && stderr == "refused"
    ));
}

// The deadline must outlast the child writing its pid, which is the only way
// this story can name the process it expects to be gone.
#[cfg(unix)]
async fn timeout_kills_child(directory: &Path) {
    let pid = directory.join("pid");
    let mut command = shell("echo $$ > \"$1\"; while :; do :; done");
    command.arg("fixture").arg(&pid);

    tokio::time::pause();
    let ready = ready(pid.clone());
    let result = run(command, "catalog export", Some(deadline()), 32, 32).await;
    ready.await.unwrap();
    tokio::time::resume();

    assert!(matches!(result, Err(Error::Timeout(_))));
    let pid = std::fs::read_to_string(pid).unwrap();
    assert!(!alive(pid.trim()), "timed-out direct child is still present");
}

// The cargo deadline must outlast the fake cargo starting its compiler
// descendant, which reports readiness by writing its pid.
async fn deadline_stops_descendants(directory: &Path) {
    // Fake Cargo That Runs A Compiler Descendant
    let pid = directory.join("descendant.pid");
    let executable = std::env::current_exe().unwrap();
    let cargo = directory.join(if cfg!(windows) { "cargo.cmd" } else { "cargo" });
    let launcher = if cfg!(windows) {
        "@echo off\r\nset \"DOKIME_PROCESS_TEST_MODE=parent\"\r\n\"%DOKIME_PROCESS_TEST_EXE%\" --exact process::process --nocapture\r\n"
    } else {
        "#!/bin/sh\nexport DOKIME_PROCESS_TEST_MODE=parent\nexec \"$DOKIME_PROCESS_TEST_EXE\" --exact process::process --nocapture\n"
    };
    std::fs::write(&cargo, launcher).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&cargo, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    // Packaging Build Under The Deadline
    let mut command = dokime::domain::process::command(&executable);
    command
        .args(["--exact", "process::process", "--nocapture"])
        .env("DOKIME_PROCESS_TEST_MODE", "build")
        .env("DOKIME_PROCESS_TEST_EXE", &executable)
        .env("DOKIME_PROCESS_TEST_PID", &pid)
        .env("CARGO", &cargo);
    run(command, "packaging deadline fixture", None, 64 * 1024, 64 * 1024).await.unwrap();

    let pid = std::fs::read_to_string(&pid).expect("descendant started");
    wait_gone(pid.trim()).await;
}

async fn leader_exit_keeps_descendants_owned(directory: &Path) {
    let pid = directory.join("leader.pid");
    let mut command = tokio::process::Command::from(dokime::domain::process::command(std::env::current_exe().unwrap()));
    command
        .args(["--exact", "process::process", "--nocapture"])
        .env("DOKIME_PROCESS_TEST_MODE", "leader")
        .env("DOKIME_PROCESS_TEST_PID", &pid);
    let mut child = dokime_packaging::process::spawn_session(command).unwrap();
    assert!(child.wait_leader().await.unwrap().success());

    let pid = std::fs::read_to_string(&pid).unwrap();
    assert!(alive(pid.trim()));
    child.kill().await.unwrap();
    wait_gone(pid.trim()).await;
}

// Fixture modes for `deadline_stops_descendants`; a no-op when run directly.
async fn fixture(mode: String) {
    match mode.as_str() {
        "leader" => {
            let pid = std::env::var_os("DOKIME_PROCESS_TEST_PID").unwrap();
            let mut child = dokime::domain::process::command(std::env::current_exe().unwrap())
                .args(["--exact", "process::process", "--nocapture"])
                .env("DOKIME_PROCESS_TEST_MODE", "descendant")
                .spawn()
                .unwrap();
            // Reap Early Exits While The Leader Lives; Its Exit Leaves The Session Owner Responsible.
            let reap = move || child.wait().unwrap();
            std::thread::spawn(reap);
            while !Path::new(&pid).is_file() {
                std::thread::sleep(GONE_POLL);
            }
        }
        "build" => {
            tokio::time::pause();
            let pid = std::env::var_os("DOKIME_PROCESS_TEST_PID").unwrap();
            let ready = ready(pid.into());
            let result = dokime_packaging::build::run(dokime_packaging::build::Options {
                cargo_timeout: Some(deadline()),
                ..Default::default()
            })
            .await;
            ready.await.unwrap();
            assert!(matches!(result, Err(Error::Timeout(_))));
        }
        "parent" => {
            let mut child = dokime::domain::process::command(std::env::current_exe().unwrap())
                .args(["--exact", "process::process", "--nocapture"])
                .env("DOKIME_PROCESS_TEST_MODE", "descendant")
                .spawn()
                .unwrap();
            child.wait().unwrap();
        }
        _ => {
            // The compiler stand-in: report readiness, then run until killed.
            let pid = std::env::var_os("DOKIME_PROCESS_TEST_PID").unwrap();
            std::fs::write(Path::new(&pid), std::process::id().to_string()).unwrap();
            loop {
                std::thread::park();
            }
        }
    }
}

#[tokio::test]
async fn process() {
    if let Ok(mode) = std::env::var("DOKIME_PROCESS_TEST_MODE") {
        fixture(mode).await;
        return;
    }
    let root = tempfile::tempdir().unwrap();
    #[cfg(unix)]
    {
        no_deadline_completion().await;
        status_output_bounded().await;
        timeout_kills_child(root.path()).await;
    }
    deadline_stops_descendants(root.path()).await;
    leader_exit_keeps_descendants_owned(root.path()).await;
}
