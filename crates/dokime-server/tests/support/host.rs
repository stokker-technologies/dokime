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
use std::{path::Path, process::Stdio};
// Third Party Imports
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{Child, ChildStdout, Command};
// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
/// The built `dokime-server` host under dev control, killed on drop.
pub struct Host {
    child: Child,
    endpoint: String,
    replies: Lines<BufReader<ChildStdout>>,
    _root: Option<TempDir>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Host {
    /// Launches the host on a fresh storage root it owns.
    pub async fn launch(plugin: Option<&Path>) -> Self {
        let root = tempfile::tempdir().unwrap();
        let mut host = Self::launch_in(root.path(), plugin).await;
        host._root = Some(root);
        host
    }

    /// Launches the host on `root`, optionally with one plugin bundle, and
    /// returns once its `ready` line names the listening endpoint. A later
    /// launch on the same root sees what this one stored.
    pub async fn launch_in(root: &Path, plugin: Option<&Path>) -> Self {
        let mut command = Command::new(super::artifacts::host_binary());
        command
            .args(["--dev-control", "--shutdown-stdin", "--no-seed-validation-artifact", "--port", "0", "--output-root"])
            .arg(root.join("data"))
            .arg("--application-root")
            .arg(root.join("application"))
            .arg("--plugins-root")
            .arg(root.join("plugins"))
            .env("RUST_LOG", "off")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .kill_on_drop(true);
        if let Some(plugin) = plugin {
            command.arg("--plugin").arg(plugin);
        }
        let mut child = command.spawn().unwrap();

        // Readiness Is The Host's `ready` Line After One Line Per Startup Phase
        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
        let mut phases = Vec::new();
        let ready = loop {
            let line = lines.next_line().await.unwrap().expect("the backend exited before reporting readiness");
            let event: serde_json::Value =
                serde_json::from_str(&line).unwrap_or_else(|error| panic!("unparseable readiness line {line:?}: {error}"));
            if event["event"] != "phase" {
                break event;
            }
            phases.push(event["name"].as_str().unwrap_or_default().to_owned());
        };
        // The dev-control startup phases `serve` reaches, in order, before it
        // starts the gRPC listener and reports ready.
        assert_eq!(phases, vec!["storage", "plugins", "requests"]);
        assert_eq!(ready["event"], "ready");
        assert_eq!(ready["protocol"], 3);
        assert_eq!(ready["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(ready["dev_protocol"], 4);
        assert_eq!(ready["transport"], "grpc");
        let endpoint = ready["endpoint"].as_str().unwrap().to_owned();
        Self { child, endpoint, replies: lines, _root: None }
    }

    /// Sends one dev-control command line and returns the host's reply line.
    pub async fn command(&mut self, request: serde_json::Value) -> serde_json::Value {
        let stdin = self.child.stdin.as_mut().unwrap();
        stdin.write_all(format!("{request}\n").as_bytes()).await.unwrap();
        stdin.flush().await.unwrap();
        let line = self.replies.next_line().await.unwrap().expect("the host replies before exiting");
        serde_json::from_str(&line).unwrap()
    }

    /// Connects an unauthenticated local client to the host.
    pub async fn connect(&self) -> dokime_client::Client {
        dokime_client::Client::connect(dokime_client::Target::local(&self.endpoint).unwrap(), None, None).await.unwrap()
    }

    /// Stops the host by closing stdin or, with `signal`, by SIGINT, and
    /// asserts a clean exit. Process exit is the event; the lane deadline
    /// bounds a hang.
    pub async fn stop(mut self, signal: bool) {
        if signal {
            let pid = self.child.id().unwrap().to_string();
            let status = Command::new("kill").args(["-INT", &pid]).status().await.unwrap();
            assert!(status.success());
        } else {
            drop(self.child.stdin.take());
        }
        let status = self.child.wait().await.unwrap();
        assert!(status.success(), "the host exited with {status}");
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
