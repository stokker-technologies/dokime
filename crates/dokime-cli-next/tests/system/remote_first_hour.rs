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
use prost::Message;
use tokio::io::{AsyncBufReadExt, BufReader};

// Local Imports
use dokime_protocol::v1;

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
            let credentials = super::credentials::CredentialStore::start(directory.path()).await?;
            let mut host = super::command()
                .envs(credentials.environment.iter().cloned())
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

            // Prepare a Manual Cycle From the Same Frozen Cases
            let channel = tonic::transport::Endpoint::from_shared(endpoint.clone())?.connect().await?;
            let mut executions = v1::execution_client::ExecutionClient::new(channel);
            let frozen = v1::PreviewSelectionReply::decode(std::fs::read(&cases)?.as_slice())?;
            let manual = executions
                .create_cycle(v1::ExecutionCreateCycleRequest {
                    origins: frozen.cases.iter().map(|case| case.origin.clone().unwrap()).collect(),
                    continue_automatically: false,
                    ..Default::default()
                })
                .await?
                .into_inner();

            // Append Another Explicit Binding From the Command Line
            let appended = "019a0000-0000-7000-8000-000000000002";
            let request = v1::ExecutionAppendExecutionsRequest {
                cycle_id: manual.cycle_id.clone(),
                executions: vec![v1::ExecutionBinding {
                    execution_id: appended.into(),
                    case_id: frozen.cases[0].case_id.clone(),
                }],
            };
            let path = directory.path().join("append.pb");
            std::fs::write(&path, request.encode_to_vec())?;
            let reply: serde_json::Value = serde_json::from_str(
                &super::run(&["--endpoint", &endpoint, "cycle", "append", "--request", path.to_str().unwrap(), "--json"]).await?,
            )?;
            assert_eq!(reply["execution_ids"][0], appended);
            let queued: serde_json::Value =
                serde_json::from_str(&super::run(&["--endpoint", &endpoint, "execution", "show", appended, "--json"]).await?)?;
            assert!(queued["controls"]["can_start"].as_bool().unwrap());

            // Start the Appended Execution and Complete Its Prompt
            super::run(&["--endpoint", &endpoint, "execution", "start", appended]).await?;
            super::run(&["--endpoint", &endpoint, "execution", "prompt", "next", appended, "--json"]).await?;
            super::run(&[
                "--endpoint",
                &endpoint,
                "execution",
                "prompt",
                "answer",
                appended,
                "--action",
                if language == "rust" { "yes" } else { "0" },
                "--accepted",
            ])
            .await?;
            super::run(&["--endpoint", &endpoint, "execution", "wait", appended]).await?;
            let completed: serde_json::Value =
                serde_json::from_str(&super::run(&["--endpoint", &endpoint, "execution", "show", appended, "--json"]).await?)?;
            assert_eq!(completed["outcome"]["evaluated"]["status"], "RESULT_STATUS_PASSED");

            // Stop the Other Execution at Its Prompt Boundary
            let stopped = &manual.execution_ids[0];
            super::run(&["--endpoint", &endpoint, "execution", "start", stopped]).await?;
            super::run(&["--endpoint", &endpoint, "execution", "prompt", "next", stopped, "--json"]).await?;
            super::run(&["--endpoint", &endpoint, "execution", "stop", stopped]).await?;
            super::run(&[
                "--endpoint",
                &endpoint,
                "execution",
                "prompt",
                "answer",
                stopped,
                "--action",
                if language == "rust" { "yes" } else { "0" },
                "--accepted",
            ])
            .await?;
            super::run(&["--endpoint", &endpoint, "execution", "wait", stopped]).await?;
            let stopped_detail: serde_json::Value =
                serde_json::from_str(&super::run(&["--endpoint", &endpoint, "execution", "show", stopped, "--json"]).await?)?;
            assert_eq!(stopped_detail["outcome"]["not_evaluated"]["termination"], "TERMINATION_CANCELLED");

            // Read a Rendered Report Through the Served Data Operation
            let report = directory.path().join("completed.html");
            super::run(&["--endpoint", &endpoint, "execution", "report", appended, "--out", report.to_str().unwrap()]).await?;
            let html = std::fs::read_to_string(&report)?;
            assert!(html.contains("<!doctype html>"));

            // Seal the Completed Cycle and Rerun Its Frozen Origins
            super::run(&["--endpoint", &endpoint, "cycle", "seal", &manual.cycle_id]).await?;
            assert!(executions.get_cycle(v1::ExecutionGetCycleRequest { cycle_id: manual.cycle_id.clone() }).await?.into_inner().sealed);
            let rerun: serde_json::Value =
                serde_json::from_str(&super::run(&["--endpoint", &endpoint, "cycle", "rerun", &manual.cycle_id, "--json"]).await?)?;
            let rerun_id = rerun["cycle_id"].as_str().ok_or("Rerun Cycle: Missing Cycle")?;
            assert_ne!(rerun_id, manual.cycle_id);
            let rerun_executions = executions
                .list_executions(v1::ExecutionListExecutionsRequest {
                    cycle_id: rerun_id.into(),
                    ..Default::default()
                })
                .await?
                .into_inner();
            assert_eq!(rerun_executions.executions.len(), 2);
            for execution in rerun_executions.executions {
                let id = execution.execution_id;
                super::run(&["--endpoint", &endpoint, "execution", "prompt", "next", &id, "--json"]).await?;
                super::run(&[
                    "--endpoint",
                    &endpoint,
                    "execution",
                    "prompt",
                    "answer",
                    &id,
                    "--action",
                    if language == "rust" { "yes" } else { "0" },
                    "--accepted",
                ])
                .await?;
                super::run(&["--endpoint", &endpoint, "execution", "wait", &id]).await?;
                let detail: serde_json::Value =
                    serde_json::from_str(&super::run(&["--endpoint", &endpoint, "execution", "show", &id, "--json"]).await?)?;
                assert_eq!(detail["outcome"]["evaluated"]["status"], "RESULT_STATUS_PASSED");
            }

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
