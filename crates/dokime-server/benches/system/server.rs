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
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::{
    env::consts::EXE_SUFFIX,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{Child, ChildStdout, Command, Stdio},
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
    _stdout: BufReader<ChildStdout>,
    _root: TempDir,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Server {
    pub fn start(plugin: Option<&Path>) -> Self {
        let root = tempfile::tempdir().unwrap();
        let mut command = Command::new(artifact(&format!("bin/dokime-server{EXE_SUFFIX}")));
        #[cfg(windows)]
        command.creation_flags(0x0800_0000);
        command
            .args(["--dev-control", "--shutdown-stdin", "--no-seed-validation-artifact", "--port", "0", "--output-root"])
            .arg(root.path().join("data"))
            .arg("--application-root")
            .arg(root.path().join("application"))
            .arg("--plugins-root")
            .arg(root.path().join("plugins"))
            .env("RUST_LOG", "off")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        if let Some(plugin) = plugin {
            command.arg("--plugin").arg(plugin);
        }
        let mut child = command.spawn().unwrap();
        let mut stdout = BufReader::new(child.stdout.take().unwrap());
        // `--dev-control` prints one JSON line per startup phase, then `ready`.
        let endpoint = loop {
            let mut line = String::new();
            assert!(stdout.read_line(&mut line).unwrap() > 0, "the server exited before reporting readiness");
            let event: serde_json::Value = serde_json::from_str(&line).unwrap();
            if event["event"] == "ready" {
                break event["endpoint"].as_str().unwrap().to_owned();
            }
        };
        Self { child, endpoint, _stdout: stdout, _root: root }
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
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
    let path = build_root().join("plugins/obc-plugin.dokime");
    path.is_file().then_some(path)
}

/* === Private Functions === */
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
