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
use std::{
    path::{Path, PathBuf},
    process::Stdio,
};
// Third Party Imports
use tempfile::TempDir;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{Child, ChildStdout, Command};
// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct Host {
    child: Child,
    endpoint: String,
    replies: Lines<BufReader<ChildStdout>>,
    stderr: PathBuf,
    _root: Option<TempDir>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Host {
    pub async fn launch(plugin: Option<&Path>) -> Self {
        let root = tempfile::tempdir().unwrap();
        let mut host = Self::launch_in(root.path(), plugin).await;
        host._root = Some(root);
        host
    }

    pub async fn launch_in(root: &Path, plugin: Option<&Path>) -> Self {
        std::fs::create_dir_all(root).unwrap();
        let stderr = root.join("host.stderr.log");
        let diagnostics = std::fs::File::create(&stderr).unwrap();
        let mut command = Command::new(super::artifacts::host_binary());
        command
            .args(["--dev-control", "--shutdown-stdin", "--port", "0", "--output-root"])
            .arg(root.join("data"))
            .arg("--application-root")
            .arg(root.join("application"))
            .arg("--plugins-root")
            .arg(root.join("plugins"))
            .env("RUST_LOG", "off")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(diagnostics)
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
        Self { child, endpoint, replies: lines, stderr, _root: None }
    }

    pub async fn command(&mut self, request: serde_json::Value) -> serde_json::Value {
        let stdin = self.child.stdin.as_mut().unwrap();
        stdin.write_all(format!("{request}\n").as_bytes()).await.unwrap();
        stdin.flush().await.unwrap();
        let line = self.replies.next_line().await.unwrap().expect("the host replies before exiting");
        serde_json::from_str(&line).unwrap()
    }

    pub async fn connect(&self) -> dokime_client::Client {
        dokime_client::Client::connect(dokime_client::Target::local(&self.endpoint).unwrap(), None, None).await.unwrap()
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub async fn stop(mut self, signal: bool) {
        if signal {
            let pid = self.child.id().unwrap().to_string();
            let status = Command::new("kill").args(["-INT", &pid]).status().await.unwrap();
            assert!(status.success());
        } else {
            drop(self.child.stdin.take());
        }
        let status = self.child.wait().await.unwrap();
        let diagnostics = std::fs::read_to_string(&self.stderr).unwrap();
        assert!(status.success(), "the host exited with {status}:\n{diagnostics}");
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
