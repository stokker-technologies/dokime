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
use std::env::consts::EXE_SUFFIX;
use std::error::Error;
use std::fs;
use std::process::Stdio;

// Third Party Imports
use prost::Message;
use serde_json::{Deserializer, Value};
use tokio::io::{AsyncBufReadExt, BufReader};
use tonic::transport::Endpoint;

// Local Imports
use dokime::Client;
use dokime::Metadata;
use dokime::client::{CyclesCreateOptions, ExecutionBinding};
use dokime_protocol::host::{CasesCreateResponse, ExecutionsCreateRequest};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

#[test]
fn remote_first_hour_runs_rust_and_python() -> Result<(), Box<dyn Error>> {
    super::credentials::run(Box::pin(async {
        tokio::time::timeout(super::HANG_GUARD, async {
            let mut failures = Vec::new();
            for language in ["rust", "python"] {
                // Generate the SDK's Starter and Start the New Host
                let directory = super::credentials::tempdir()?;
                let project = directory.path().join(language);
                super::run(&["new", project.to_str().unwrap(), "--lang", language]).await?;
                // Select the Development Input
                let input = if language == "rust" {
                    let executable = super::fixtures().join(format!("dokime-example-test{}", EXE_SUFFIX));
                    let prepared = directory.path().join("prepared-plugin");
                    fs::create_dir(&prepared)?;
                    fs::copy(&executable, prepared.join(format!("plugin{EXE_SUFFIX}")))?;
                    fs::write(prepared.join("dokime.toml"), "executable = 'plugin'\n")?;
                    let bundle = directory.path().join("starter.dokime");
                    super::run(&["plugin", "bundle", prepared.to_str().unwrap(), "--out", bundle.to_str().unwrap()]).await?;
                    super::run(&["plugin", "inspect", "--bundle", bundle.to_str().unwrap(), "--json"]).await?;
                    bundle
                } else {
                    // Resolve the SDK Candidate Exported by This Worktree.
                    let input = project.join("test");
                    let profile = std::env::var("TEST_PROFILE").unwrap_or_else(|_| "dev".into());
                    let wheel = super::root()
                        .join("target/build")
                        .join(profile)
                        .join("any/python")
                        .join(format!("dokime_sdk-{}-py3-none-any.whl", env!("CARGO_PKG_VERSION")));
                    let source = serde_json::to_string(&wheel.to_string_lossy())?;
                    let manifest = input.join("pyproject.toml");
                    let contents = fs::read_to_string(&manifest)?;
                    fs::write(manifest, format!("{contents}\n[tool.uv.sources]\ndokime-sdk = {{ path = {source} }}\n"))?;
                    input
                };

                // Start One Disposable Application with Its Plugin
                let credentials = super::credentials::CredentialStore::start(directory.path()).await?;
                let mut host = super::command()
                    .envs(credentials.environment.iter().cloned())
                    .args(["dev", "--plugin"])
                    .arg(input)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::inherit())
                    .spawn()?;
                let result: Result<(), Box<dyn Error>> = async {
                    let mut lines = BufReader::new(host.stdout.take().unwrap()).lines();
                    let endpoint = lines.next_line().await?.ok_or("Develop Starter: Missing Host Endpoint")?;
                    lines.next_line().await?.ok_or("Develop Starter: Missing Data Root")?;
                    let entries: Value = serde_json::from_str(&super::run(&["--endpoint", &endpoint, "plugin", "list", "--json"]).await?)?;
                    if language == "python" {
                        let url = entries["entries"][0]["placement"]["remote"]["url"].as_str().ok_or("Read the Python Plugin Endpoint")?;
                        super::run(&["plugin", "inspect", "--url", url, "--json"]).await?;
                    }

                    // Materialize the Frozen Case and Start a Cycle
                    let suites: Value = serde_json::from_str(&super::run(&["--endpoint", &endpoint, "catalog", "list", "--json"]).await?)?;
                    let suite = suites["values"][0]["metadata"]["identifier"].as_str().ok_or("List Suites: Missing Suite Identity")?;
                    let cases = directory.path().join("cases.pb");
                    let parameters = directory.path().join("parameters.json");
                    fs::write(&parameters, b"{}")?;
                    let mut arguments = vec![
                        "--endpoint",
                        &endpoint,
                        "plan",
                        "materialize",
                        "--suite",
                        suite,
                        "--template",
                        "capture",
                        "--out",
                        cases.to_str().unwrap(),
                    ];
                    if language == "python" {
                        arguments.extend(["--parameters", parameters.to_str().unwrap()]);
                    }
                    super::run(&arguments).await?;

                    let cycle: Value = serde_json::from_str(
                        &super::run(&["--endpoint", &endpoint, "cycle", "create", "--cases", cases.to_str().unwrap(), "--json"]).await?,
                    )?;
                    let cycle_id = cycle["cycle"]["cycle_id"].as_str().ok_or("Start Cycle: Missing Cycle")?;
                    let executions: Value = serde_json::from_str(
                        &super::run(&["--endpoint", &endpoint, "execution", "list", "--cycle", cycle_id, "--json"]).await?,
                    )?;
                    let execution =
                        executions["executions"][0]["summary"]["execution_id"].as_str().ok_or("Start Cycle: Missing Execution")?;

                    // Await the Prompt Event and Answer It From the Command Line
                    super::run(&["--endpoint", &endpoint, "execution", "prompt", "next", execution, "--json"]).await?;
                    super::run(&["--endpoint", &endpoint, "execution", "prompt", "answer", execution, "--action", "0", "--accepted"])
                        .await?;
                    super::run(&["--endpoint", &endpoint, "execution", "wait", execution]).await?;
                    let detail: Value =
                        serde_json::from_str(&super::run(&["--endpoint", &endpoint, "execution", "show", execution, "--json"]).await?)?;
                    assert!(!detail["execution"]["seal"].as_str().unwrap_or_default().is_empty());
                    assert_eq!(detail["execution"]["outcome"]["evaluated"]["status"], "RESULT_STATUS_PASSED");
                    let report = directory.path().join("report.html");
                    if let Err(error) =
                        super::run(&["--endpoint", &endpoint, "execution", "report", execution, "--out", report.to_str().unwrap()]).await
                    {
                        assert!(!report.exists());
                        failures.push(error.to_string());
                    }

                    // Prepare a Manual Cycle From the Same Frozen Cases
                    let channel = Endpoint::from_shared(endpoint.clone())?.connect().await?;
                    let sdk = Client::from_channel(channel, super::client_settings())?;
                    let frozen = CasesCreateResponse::decode(fs::read(&cases)?.as_slice())?;
                    let origins = frozen.cases.iter().map(|case| case.origin.clone().unwrap()).collect();
                    let manual_reply = sdk
                        .cycles
                        .create_with(
                            origins,
                            Metadata {
                                label: "Manual Cycle".into(),
                                ..Default::default()
                            },
                            false,
                            "",
                            CyclesCreateOptions {
                                request_id: Some("manual".into()),
                                ..Default::default()
                            },
                        )
                        .await?;
                    let manual = manual_reply.cycle.unwrap();
                    let manual_executions =
                        sdk.executions.list(manual.cycle_id.clone(), Default::default(), Default::default(), Default::default()).await?;

                    // Append Another Explicit Binding From the Command Line
                    let appended = "019a0000-0000-7000-8000-000000000002";
                    let request = ExecutionsCreateRequest {
                        parent: manual.cycle_id.clone(),
                        request_id: "append".into(),
                        executions: vec![ExecutionBinding {
                            execution_id: appended.into(),
                            case_id: frozen.cases[0].case_id.clone(),
                        }],
                    };
                    let path = directory.path().join("append.pb");
                    fs::write(&path, request.encode_to_vec())?;
                    let reply: Value = serde_json::from_str(
                        &super::run(&["--endpoint", &endpoint, "cycle", "append", "--request", path.to_str().unwrap(), "--json"]).await?,
                    )?;
                    assert_eq!(reply["executions"][0]["summary"]["execution_id"], appended);
                    let queued: Value =
                        serde_json::from_str(&super::run(&["--endpoint", &endpoint, "execution", "show", appended, "--json"]).await?)?;
                    assert!(queued["execution"]["controls"]["can_start"].as_bool().unwrap());

                    // Continue a One-Row Page at Its Returned Cut.
                    let output = super::run(&[
                        "--endpoint",
                        &endpoint,
                        "execution",
                        "list",
                        "--cycle",
                        &manual.cycle_id,
                        "--page-size",
                        "1",
                        "--json",
                    ])
                    .await?;
                    let pages = Deserializer::from_str(&output).into_iter::<Value>().collect::<Result<Vec<_>, _>>()?;
                    assert_eq!(pages.len(), 2);
                    assert_eq!(pages[0]["resolved_at"], pages[1]["resolved_at"]);
                    assert_ne!(pages[0]["executions"][0]["summary"]["execution_id"], pages[1]["executions"][0]["summary"]["execution_id"]);

                    // Start the Appended Execution and Complete Its Prompt
                    super::run(&["--endpoint", &endpoint, "execution", "start", appended]).await?;
                    super::run(&["--endpoint", &endpoint, "execution", "prompt", "next", appended, "--json"]).await?;
                    super::run(&["--endpoint", &endpoint, "execution", "prompt", "answer", appended, "--action", "0", "--accepted"])
                        .await?;
                    super::run(&["--endpoint", &endpoint, "execution", "wait", appended]).await?;
                    let completed: Value =
                        serde_json::from_str(&super::run(&["--endpoint", &endpoint, "execution", "show", appended, "--json"]).await?)?;
                    assert_eq!(completed["execution"]["outcome"]["evaluated"]["status"], "RESULT_STATUS_PASSED");

                    // Stop the Other Execution at Its Prompt Boundary
                    let stopped = &manual_executions.executions[0].summary.as_ref().unwrap().execution_id;
                    super::run(&["--endpoint", &endpoint, "execution", "start", stopped]).await?;
                    super::run(&["--endpoint", &endpoint, "execution", "prompt", "next", stopped, "--json"]).await?;
                    super::run(&["--endpoint", &endpoint, "execution", "stop", stopped]).await?;
                    super::run(&["--endpoint", &endpoint, "execution", "prompt", "answer", stopped, "--action", "0", "--accepted"]).await?;
                    super::run(&["--endpoint", &endpoint, "execution", "wait", stopped]).await?;
                    let stopped_detail: Value =
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
                        let html = fs::read_to_string(&report)?;
                        assert!(html.contains("<!doctype html>"));
                    }

                    // Seal the Completed Cycle and Rerun Its Frozen Origins
                    super::run(&["--endpoint", &endpoint, "cycle", "seal", &manual.cycle_id]).await?;
                    assert!(sdk.cycles.get(manual.cycle_id.clone()).await?.cycle.unwrap().sealed);
                    let rerun: Value =
                        serde_json::from_str(&super::run(&["--endpoint", &endpoint, "cycle", "rerun", &manual.cycle_id, "--json"]).await?)?;
                    let rerun_id = rerun["cycle"]["cycle_id"].as_str().ok_or("Rerun Cycle: Missing Cycle")?;
                    assert_ne!(rerun_id, manual.cycle_id);
                    let rerun_executions =
                        sdk.executions.list(rerun_id, Default::default(), Default::default(), Default::default()).await?;
                    assert_eq!(rerun_executions.executions.len(), 2);
                    for execution in rerun_executions.executions {
                        let id = execution.summary.unwrap().execution_id;
                        super::run(&["--endpoint", &endpoint, "execution", "prompt", "next", &id, "--json"]).await?;
                        super::run(&["--endpoint", &endpoint, "execution", "prompt", "answer", &id, "--action", "0", "--accepted"]).await?;
                        super::run(&["--endpoint", &endpoint, "execution", "wait", &id]).await?;
                        let detail: Value =
                            serde_json::from_str(&super::run(&["--endpoint", &endpoint, "execution", "show", &id, "--json"]).await?)?;
                        assert_eq!(detail["execution"]["outcome"]["evaluated"]["status"], "RESULT_STATUS_PASSED");
                    }

                    Ok(())
                }
                .await;
                // Close the Host Before Discarding Its Credentials, Including on Failure.
                drop(host.stdin.take());
                let host_status = host.wait().await?;
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
    }))
}

/* === Tests === */
