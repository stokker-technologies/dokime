/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Start the Host From an Explicit Environment File

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::process::Stdio;

// Third Party Imports
use tokio::io::{AsyncBufReadExt, BufReader};

// Local Imports

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

#[test]
fn operator_starts_host_from_environment_file() -> Result<(), Box<dyn std::error::Error>> {
    super::credentials::run(Box::pin(async {
        tokio::time::timeout(super::HANG_GUARD, async {
            let directory = super::credentials::tempdir()?;
            let credentials = super::credentials::CredentialStore::start(directory.path()).await?;
            let storage = directory.path().join("storage");
            let server = |file: Option<&std::path::Path>, logging: Option<&str>| {
                let mut command = super::command();
                command
                    .envs(credentials.environment.iter().cloned())
                    .env_remove("RUST_LOG")
                    .current_dir(directory.path())
                    .args(["--endpoint", "127.0.0.1:0", "server", "--controlled-clock", "--storage"])
                    .arg(&storage)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped());
                if let Some(file) = file {
                    command.arg("--env-file").arg(file);
                }
                if let Some(logging) = logging {
                    command.env("RUST_LOG", logging);
                }
                command
            };

            // Refuse a Missing File and Variables the Host Never Reads Before Opening Storage
            let missing = server(Some(&directory.path().join("missing.env")), None).output().await?;
            assert_eq!(missing.status.code(), Some(2));
            let unknown = directory.path().join("unknown.env");
            std::fs::write(
                &unknown,
                "# Retired Access Key
DOKIME_KEY=retired
",
            )?;
            let refused = server(Some(&unknown), None).output().await?;
            assert_eq!(refused.status.code(), Some(2));
            assert!(!storage.exists());

            // Apply the File's Logging Directives Before the Host Starts
            let invalid = directory.path().join("invalid.env");
            std::fs::write(
                &invalid,
                "RUST_LOG=\"dokime=loudest\"
",
            )?;
            let applied = server(Some(&invalid), None).output().await?;
            assert_eq!(applied.status.code(), Some(2));
            assert!(!storage.exists());

            // Prefer the Process Environment, and Never Search for an Implicit .env File
            std::fs::write(
                directory.path().join(".env"),
                "DOKIME_KEY=retired
",
            )?;
            for (file, logging) in [(Some(invalid.as_path()), Some("info")), (None, None)] {
                let mut host = server(file, logging).spawn()?;
                let mut lines = BufReader::new(host.stdout.take().unwrap()).lines();
                let endpoint = lines.next_line().await?.ok_or("Start Host: Missing Listener Announcement")?;
                let status: serde_json::Value = serde_json::from_str(&super::run(&["--endpoint", &endpoint, "status", "--json"]).await?)?;
                assert!(status["status"]["identity"].is_object());
                drop(host.stdin.take());
                assert!(host.wait().await?.success());
            }
            Ok(())
        })
        .await?
    }))
}
