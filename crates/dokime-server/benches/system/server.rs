/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Server Process Driver: Built Artifact Paths, Readiness, And Shutdown

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::{
    env::consts::EXE_SUFFIX,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdout, Stdio},
    time::{Duration, Instant},
};

// Third Party Imports
use tempfile::TempDir;

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct Server {
    child: Child,
    endpoint: String,
    stdout: BufReader<ChildStdout>,
    _root: TempDir,
}

pub struct StorageCounters {
    pub committed_transactions: u64,
    pub data_batches: u64,
    pub durable_files: u64,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Server {
    pub fn start(plugin: Option<&Path>) -> Self {
        let root = tempfile::tempdir().unwrap();
        let log = std::env::var("DOKIME_BENCH_LOG").ok();
        let stderr = if log.is_some() { Stdio::inherit() } else { Stdio::null() };
        let filter = log.as_deref().unwrap_or("off");
        let mut command = dokime::domain::process::command(artifact(&format!("bin/dokime{EXE_SUFFIX}")));
        command.arg("server");
        command
            .args(["--dev-control", "--shutdown-stdin", "--no-seed-validation-artifact", "--port", "0", "--output-root"])
            .arg(root.path().join("data"))
            .arg("--application-root")
            .arg(root.path().join("application"))
            .arg("--plugins-root")
            .arg(root.path().join("plugins"))
            .env("RUST_LOG", filter)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(stderr);
        if let Some(plugin) = plugin {
            command.arg("--plugin").arg(plugin);
        }
        let mut child = command.spawn().unwrap();
        let mut stdout = BufReader::new(child.stdout.take().unwrap());
        // `--dev-control` prints one JSON line per startup phase, then `ready`.
        let mut event = startup_event(&mut stdout);
        while event["event"] != "ready" {
            event = startup_event(&mut stdout);
        }
        let endpoint = event["endpoint"].as_str().unwrap().to_owned();
        Self { child, endpoint, stdout, _root: root }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn storage_counters(&mut self) -> StorageCounters {
        let stdin = self.child.stdin.as_mut().unwrap();
        writeln!(stdin, "{}", serde_json::json!({"command": "storage_counters"})).unwrap();
        stdin.flush().unwrap();
        let mut line = String::new();
        assert!(self.stdout.read_line(&mut line).unwrap() > 0, "the server exited before replying");
        let reply: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(reply["event"], "storage_counters");
        StorageCounters {
            committed_transactions: reply["committed_transactions"].as_u64().unwrap(),
            data_batches: reply["data_batches"].as_u64().unwrap(),
            durable_files: reply["durable_files"].as_u64().unwrap(),
        }
    }

    // Development Control Loads The Bundle Into The Running Session.
    pub fn reload(&mut self, bundle: &Path) {
        let stdin = self.child.stdin.as_mut().unwrap();
        writeln!(stdin, "{}", serde_json::json!({"command": "reload", "path": bundle})).unwrap();
        stdin.flush().unwrap();
        let mut line = String::new();
        assert!(self.stdout.read_line(&mut line).unwrap() > 0, "the server exited before replying");
        let reply: serde_json::Value = serde_json::from_str(&line).unwrap();
        assert_eq!(reply["event"], "reloaded", "the development session loads the bench bundle: {reply}");
    }

    pub fn stop(mut self) -> Duration {
        let started = Instant::now();
        drop(self.child.stdin.take());
        let status = self.child.wait().unwrap();
        let elapsed = started.elapsed();
        assert!(status.success(), "the server exited with {status}");
        elapsed
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/* === Public Functions === */
pub fn plugin_bundle() -> Option<PathBuf> {
    let path = build_root().join("fixtures/plugins/obc-plugin.dokime");
    path.is_file().then_some(path)
}

/* === Private Functions === */
fn startup_event(stdout: &mut BufReader<ChildStdout>) -> serde_json::Value {
    let mut line = String::new();
    assert!(stdout.read_line(&mut line).unwrap() > 0, "the server exited before reporting readiness");
    serde_json::from_str(&line).unwrap()
}

fn artifact(relative: &str) -> PathBuf {
    let path = build_root().join(relative);
    assert!(path.is_file(), "missing built artifact {}; run `mise run build`", path.display());
    path
}

fn build_root() -> PathBuf {
    std::env::current_dir()
        .expect("benchmark runner starts in the package")
        .ancestors()
        .find(|directory| directory.join("Cargo.lock").is_file())
        .expect("the workspace root holds Cargo.lock")
        .join("target/build/bench")
        .join(format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH))
}

/* === Tests === */
