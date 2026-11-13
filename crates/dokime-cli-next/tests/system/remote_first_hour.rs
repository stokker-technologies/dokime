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
use dokime_protocol::client;

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
        let mut failures = Vec::new();
        for language in ["rust", "python"] {
            // Generate the SDK's Starter and Start the New Host
            let directory = super::credentials::tempdir()?;
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
            let mut remote_plugin = None;
            let result: Result<(), Box<dyn std::error::Error>> = async {
                let mut lines = BufReader::new(host.stdout.take().unwrap()).lines();
                let endpoint = lines.next_line().await?.ok_or("Start Host: Missing Listener Announcement")?;

                // Serve the New Plugin and Add Its Remote Entry
                remote_plugin = Some(if language == "rust" {
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
                });
                let plugin = remote_plugin.as_mut().unwrap();
                let mut lines = BufReader::new(plugin.stdout.take().unwrap()).lines();
                let url = lines.next_line().await?.ok_or("Serve Starter: Missing Listener Announcement")?;
                super::run(&["--endpoint", &endpoint, "plugin", "inspect", "--url", &url, "--json"]).await?;
                super::run(&["--endpoint", &endpoint, "plugin", "add", &url]).await?;
                super::run(&["--endpoint", &endpoint, "plugin", "list", "--json"]).await?;

                // Materialize the Frozen Case and Start a Cycle
                let suites: serde_json::Value =
                    serde_json::from_str(&super::run(&["--endpoint", &endpoint, "catalog", "list", "--json"]).await?)?;
                let suite = suites["values"][0]["metadata"]["identifier"].as_str().ok_or("List Suites: Missing Suite Identity")?;
                let cases = directory.path().join("cases.pb");
                let parameters = directory.path().join("parameters.json");
                std::fs::write(&parameters, b"{}")?;
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
                let cycle_id = cycle["cycle"]["cycle_id"].as_str().ok_or("Start Cycle: Missing Cycle")?;
                let executions: serde_json::Value = serde_json::from_str(
                    &super::run(&["--endpoint", &endpoint, "execution", "list", "--cycle", cycle_id, "--json"]).await?,
                )?;
                let execution = executions["executions"][0]["summary"]["execution_id"].as_str().ok_or("Start Cycle: Missing Execution")?;

                // Await the Prompt Event and Answer It From the Command Line
                super::run(&["--endpoint", &endpoint, "execution", "prompt", "next", execution, "--json"]).await?;
                super::run(&["--endpoint", &endpoint, "execution", "prompt", "answer", execution, "--action", "yes", "--accepted"]).await?;
                super::run(&["--endpoint", &endpoint, "execution", "watch", execution]).await?;
                let detail: serde_json::Value =
                    serde_json::from_str(&super::run(&["--endpoint", &endpoint, "execution", "show", execution, "--json"]).await?)?;
                assert!(!detail["execution"]["seal"].as_str().unwrap_or_default().is_empty());
                assert_eq!(detail["execution"]["outcome"]["evaluated"]["status"], "RESULT_STATUS_PASSED");
                let report = directory.path().join("report.html");
                if let Err(error) =
                    super::run(&["--endpoint", &endpoint, "execution", "report", execution, "--out", report.to_str().unwrap()]).await
                {
                    assert!(!report.exists());
                    failures.push(error.to_string());
                } else {
                    assert!(std::fs::read_to_string(report)?.contains(execution));
                }

                // Prepare a Manual Cycle From the Same Frozen Cases
                let channel = tonic::transport::Endpoint::from_shared(endpoint.clone())?.connect().await?;
                let sdk = dokime::Client::from_channel(channel, super::client_settings())?;
                let frozen = client::CasesCreateResponse::decode(std::fs::read(&cases)?.as_slice())?;
                let manual_reply = sdk
                    .cycles
                    .create(tonic::Request::new(client::CyclesCreateRequest {
                        origins: frozen.cases.iter().map(|case| case.origin.clone().unwrap()).collect(),
                        continue_automatically: false,
                        request_id: "manual".into(),
                        ..Default::default()
                    }))
                    .await?
                    .into_inner();
                let manual = manual_reply.cycle.unwrap();
                let manual_executions = sdk
                    .executions
                    .list(tonic::Request::new(client::ExecutionsListRequest {
                        parent: manual.cycle_id.clone(),
                        ..Default::default()
                    }))
                    .await?
                    .into_inner();

                // Append Another Explicit Binding From the Command Line
                let appended = "019a0000-0000-7000-8000-000000000002";
                let request = client::ExecutionsCreateRequest {
                    parent: manual.cycle_id.clone(),
                    request_id: "append".into(),
                    executions: vec![client::ExecutionBinding {
                        execution_id: appended.into(),
                        case_id: frozen.cases[0].case_id.clone(),
                    }],
                };
                let path = directory.path().join("append.pb");
                std::fs::write(&path, request.encode_to_vec())?;
                let reply: serde_json::Value = serde_json::from_str(
                    &super::run(&["--endpoint", &endpoint, "cycle", "append", "--request", path.to_str().unwrap(), "--json"]).await?,
                )?;
                assert_eq!(reply["executions"][0]["summary"]["execution_id"], appended);
                let queued: serde_json::Value =
                    serde_json::from_str(&super::run(&["--endpoint", &endpoint, "execution", "show", appended, "--json"]).await?)?;
                assert!(queued["execution"]["controls"]["can_start"].as_bool().unwrap());

                // Continue a One-Row Page at Its Returned Cut.
                let output =
                    super::run(&["--endpoint", &endpoint, "execution", "list", "--cycle", &manual.cycle_id, "--page-size", "1", "--json"])
                        .await?;
                let pages = serde_json::Deserializer::from_str(&output).into_iter::<serde_json::Value>().collect::<Result<Vec<_>, _>>()?;
                assert_eq!(pages.len(), 2);
                assert_eq!(pages[0]["resolved_at"], pages[1]["resolved_at"]);
                assert_ne!(pages[0]["executions"][0]["summary"]["execution_id"], pages[1]["executions"][0]["summary"]["execution_id"]);

                // Start the Appended Execution and Complete Its Prompt
                super::run(&["--endpoint", &endpoint, "execution", "start", appended]).await?;
                super::run(&["--endpoint", &endpoint, "execution", "prompt", "next", appended, "--json"]).await?;
                super::run(&["--endpoint", &endpoint, "execution", "prompt", "answer", appended, "--action", "yes", "--accepted"]).await?;
                super::run(&["--endpoint", &endpoint, "execution", "wait", appended]).await?;
                let completed: serde_json::Value =
                    serde_json::from_str(&super::run(&["--endpoint", &endpoint, "execution", "show", appended, "--json"]).await?)?;
                assert_eq!(completed["execution"]["outcome"]["evaluated"]["status"], "RESULT_STATUS_PASSED");

                // Stop the Other Execution at Its Prompt Boundary
                let stopped = &manual_executions.executions[0].summary.as_ref().unwrap().execution_id;
                super::run(&["--endpoint", &endpoint, "execution", "start", stopped]).await?;
                super::run(&["--endpoint", &endpoint, "execution", "prompt", "next", stopped, "--json"]).await?;
                super::run(&["--endpoint", &endpoint, "execution", "stop", stopped]).await?;
                super::run(&["--endpoint", &endpoint, "execution", "prompt", "answer", stopped, "--action", "yes", "--accepted"]).await?;
                super::run(&["--endpoint", &endpoint, "execution", "wait", stopped]).await?;
                let stopped_detail: serde_json::Value =
                    serde_json::from_str(&super::run(&["--endpoint", &endpoint, "execution", "show", stopped, "--json"]).await?)?;
                assert_eq!(stopped_detail["execution"]["outcome"]["not_evaluated"]["termination"], "TERMINATION_CANCELLED");

                // Read a Rendered Report Through the Served Data Operation
                let report = directory.path().join("completed.html");
                if let Err(error) =
                    super::run(&["--endpoint", &endpoint, "execution", "report", appended, "--out", report.to_str().unwrap()]).await
                {
                    assert!(!report.exists());
                    failures.push(error.to_string());
                } else {
                    let html = std::fs::read_to_string(&report)?;
                    assert!(html.contains("<!doctype html>"));
                }

                // Seal the Completed Cycle and Rerun Its Frozen Origins
                super::run(&["--endpoint", &endpoint, "cycle", "seal", &manual.cycle_id]).await?;
                assert!(
                    sdk.cycles
                        .get(tonic::Request::new(client::CyclesGetRequest { name: manual.cycle_id.clone(), at: None }))
                        .await?
                        .into_inner()
                        .cycle
                        .unwrap()
                        .sealed
                );
                let rerun: serde_json::Value =
                    serde_json::from_str(&super::run(&["--endpoint", &endpoint, "cycle", "rerun", &manual.cycle_id, "--json"]).await?)?;
                let rerun_id = rerun["cycle"]["cycle_id"].as_str().ok_or("Rerun Cycle: Missing Cycle")?;
                assert_ne!(rerun_id, manual.cycle_id);
                let rerun_executions = sdk
                    .executions
                    .list(tonic::Request::new(client::ExecutionsListRequest {
                        parent: rerun_id.into(),
                        ..Default::default()
                    }))
                    .await?
                    .into_inner();
                assert_eq!(rerun_executions.executions.len(), 2);
                for execution in rerun_executions.executions {
                    let id = execution.summary.unwrap().execution_id;
                    super::run(&["--endpoint", &endpoint, "execution", "prompt", "next", &id, "--json"]).await?;
                    super::run(&["--endpoint", &endpoint, "execution", "prompt", "answer", &id, "--action", "yes", "--accepted"]).await?;
                    super::run(&["--endpoint", &endpoint, "execution", "wait", &id]).await?;
                    let detail: serde_json::Value =
                        serde_json::from_str(&super::run(&["--endpoint", &endpoint, "execution", "show", &id, "--json"]).await?)?;
                    assert_eq!(detail["execution"]["outcome"]["evaluated"]["status"], "RESULT_STATUS_PASSED");
                }

                Ok(())
            }
            .await;
            // Close the Host Before Discarding Its Credentials, Including on Failure.
            drop(host.stdin.take());
            let host_status = host.wait().await?;
            if let Some(mut plugin) = remote_plugin {
                if language == "rust" {
                    drop(plugin.stdin.take());
                    assert!(plugin.wait().await?.success());
                } else {
                    plugin.kill().await?;
                    plugin.wait().await?;
                }
            }
            if let Err(error) = result {
                failures.push(error.to_string());
            }
            if !host_status.success() {
                failures.push(format!("Close Host: {host_status}"));
            }
        }
        if !failures.is_empty() {
            return Err(failures.join("\n").into());
        }
        Ok(())
    })
    .await?
}

/* === Tests === */
