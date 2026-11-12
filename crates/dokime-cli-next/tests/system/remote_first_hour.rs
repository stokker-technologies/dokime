/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Run Both SDK Starters Through the Remote Command Surface

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

#[tokio::test]
async fn remote_first_hour_runs_rust_and_python() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::HANG_GUARD, async {
        for language in ["rust", "python"] {
            // Generate the SDK's Starter and Start the New Host
            let directory = tempfile::tempdir()?;
            let project = directory.path().join(language);
            super::run(&["new", project.to_str().unwrap(), "--lang", language]).await?;
            let storage = directory.path().join("storage");
            let mut host = super::command()
                .args(["--endpoint", "127.0.0.1:0", "server", "--controlled-clock", "--storage"])
                .arg(storage)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()?;
            let mut lines = BufReader::new(host.stdout.take().unwrap()).lines();
            let endpoint = lines.next_line().await?.ok_or("Start Host: Missing Listener Announcement")?;

            // Serve the New Plugin and Add Its Remote Entry
            let mut plugin = if language == "rust" {
                let executable = super::fixtures().join(format!("dokime-starter{}", std::env::consts::EXE_SUFFIX));
                super::command()
                    .args(["plugin", "dev", "--executable"])
                    .arg(executable)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::inherit())
                    .spawn()?
            } else {
                let python = std::env::var_os("DOKIME_PYTHON")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(|| super::root().join(if cfg!(windows) { ".venv/Scripts/python.exe" } else { ".venv/bin/python" }));
                tokio::process::Command::from(dokime_runtime_next::process::command(python))
                    .arg(project.join("main.py"))
                    .args(["--address", "127.0.0.1:0"])
                    .current_dir(&project)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::inherit())
                    .kill_on_drop(true)
                    .spawn()?
            };
            let mut lines = BufReader::new(plugin.stdout.take().unwrap()).lines();
            let url = lines.next_line().await?.ok_or("Serve Starter: Missing Listener Announcement")?;
            super::run(&["--endpoint", &endpoint, "plugin", "inspect", "--url", &url, "--json"]).await?;
            super::run(&["--endpoint", &endpoint, "plugin", "add", &url]).await?;
            super::run(&["--endpoint", &endpoint, "plugin", "list", "--json"]).await?;

            // Materialize the Frozen Case and Start a Cycle
            let suites: serde_json::Value =
                serde_json::from_str(&super::run(&["--endpoint", &endpoint, "catalog", "list", "--json"]).await?)?;
            let suite = suites["suites"][0]["suite_id"].as_str().ok_or("List Suites: Missing Suite Identity")?;
            let cases = directory.path().join("cases.pb");
            let parameters = directory.path().join("parameters.json");
            std::fs::write(&parameters, br#"{"limit":[{"number":10}]}"#)?;
            let mut arguments = vec![
                "--endpoint",
                &endpoint,
                "plan",
                "materialize",
                "--suite",
                suite,
                "--template",
                "example.capture",
                "--out",
                cases.to_str().unwrap(),
            ];
            if language == "python" {
                arguments.extend(["--parameters", parameters.to_str().unwrap()]);
            }
            super::run(&arguments).await?;

            let cycle: serde_json::Value = serde_json::from_str(
                &super::run(&["--endpoint", &endpoint, "cycle", "create", "--cases", cases.to_str().unwrap(), "--json"]).await?,
            )?;
            let execution = cycle["execution_ids"][0].as_str().ok_or("Start Cycle: Missing Execution")?;

            // Await the Prompt Event and Answer It From the Command Line
            super::run(&["--endpoint", &endpoint, "execution", "prompt", "next", execution, "--json"]).await?;
            super::run(&[
                "--endpoint",
                &endpoint,
                "execution",
                "prompt",
                "answer",
                execution,
                "--action",
                if language == "rust" { "yes" } else { "0" },
                "--accepted",
            ])
            .await?;
            super::run(&["--endpoint", &endpoint, "execution", "watch", execution]).await?;
            let detail: serde_json::Value =
                serde_json::from_str(&super::run(&["--endpoint", &endpoint, "execution", "show", execution, "--json"]).await?)?;
            assert!(!detail["seal"].as_str().unwrap_or_default().is_empty());
            assert_eq!(detail["outcome"]["evaluated"]["status"], "RESULT_STATUS_PASSED");
            let report = directory.path().join("report.html");
            super::run(&["--endpoint", &endpoint, "execution", "report", execution, "--out", report.to_str().unwrap()]).await?;
            assert!(std::fs::read_to_string(report)?.contains(execution));

            // Close the Host and Its Independently Owned Remote Process
            drop(host.stdin.take());
            assert!(host.wait().await?.success());
            if language == "rust" {
                drop(plugin.stdin.take());
                assert!(plugin.wait().await?.success());
            } else {
                plugin.kill().await?;
                plugin.wait().await?;
            }
        }
        Ok(())
    })
    .await?
}

/* === Tests === */
