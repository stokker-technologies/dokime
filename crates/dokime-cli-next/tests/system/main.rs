/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Command-Line System Stories

/* === Modules === */

#[path = "../../../dokime-runtime-next/tests/support/credentials.rs"]
mod credentials;
mod first_hour;
mod operator_calls_operation;
mod operator_diagnoses_storage;
mod operator_discovers_operations;
mod operator_identifies_build;
mod operator_inspects_contract;
mod operator_pairs_with_host;
mod remote_first_hour;
mod scaffold;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::PathBuf;

// Third Party Imports

// Local Imports

/* === Constants === */

const HANG_GUARD: std::time::Duration = std::time::Duration::from_secs(600);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct HostProcess {
    child: tokio::process::Child,
    endpoint: String,
    _credentials: credentials::CredentialStore,
    _root: credentials::Root,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl HostProcess {
    async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        use tokio::io::{AsyncBufReadExt, BufReader};
        let root = credentials::tempdir()?;
        let credentials = credentials::CredentialStore::start(root.path()).await?;
        let mut child = command()
            .envs(credentials.environment.iter().cloned())
            .args(["--endpoint", "127.0.0.1:0", "server", "--controlled-clock", "--storage"])
            .arg(root.path().join("storage"))
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .spawn()?;
        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
        let endpoint = lines.next_line().await?.ok_or("Start Host: Missing Listener Announcement")?;
        Ok(Self {
            child,
            endpoint,
            _credentials: credentials,
            _root: root,
        })
    }
    async fn close(mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.stop().await
    }
    async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        drop(self.child.stdin.take());
        assert!(self.child.wait().await?.success());
        Ok(())
    }
}

/* === Public Functions === */

/* === Private Functions === */

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixtures() -> PathBuf {
    let native = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
    let profile = std::env::var("TEST_PROFILE").unwrap_or_else(|_| "dev".into());
    root().join("target/build").join(profile).join(native).join("fixtures/cli")
}

fn command() -> tokio::process::Command {
    let executable = std::env::var_os("NEXTEST_BIN_EXE_dokime-next")
        .or_else(|| std::env::var_os("CARGO_BIN_EXE_dokime-next"))
        .map(PathBuf::from)
        .unwrap_or_else(|| root().join("target/debug").join(format!("dokime-next{}", std::env::consts::EXE_SUFFIX)));
    let mut command = tokio::process::Command::from(dokime_runtime_next::process::command(executable));
    command.kill_on_drop(true);
    command
}

async fn run(args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = command().args(args).output().await?;
    if !output.status.success() {
        return Err(format!("Run dokime {args:?}: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

/* === Tests === */
