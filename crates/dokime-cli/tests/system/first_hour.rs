/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// A New User Runs the Starters Through the Command Line

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::env::consts::EXE_SUFFIX;
use std::error::Error;
use std::fs;
use std::process::Stdio;

// Third Party Imports
use harmos_runtime::bundle::{Bundle, Facility, Limits};
use prost_reflect::{DynamicMessage, SerializeOptions};
use serde_json::{Deserializer, Serializer, Value};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::time;

// Local Imports
use dokime_runtime::process;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

/// Serve the Native Starter Fixture at an Ephemeral Address and Return Its URL.
async fn serve_fixture() -> Result<(tokio::process::Child, String), Box<dyn Error>> {
    let executable = super::fixtures().join(format!("rust{EXE_SUFFIX}"));
    let mut plugin = Command::from(process::command(executable))
        .env_remove("HARMOS_ADDRESS")
        .env("DOKIME_PLUGIN_ADDRESS", "127.0.0.1:0")
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .kill_on_drop(true)
        .spawn()?;
    let mut lines = BufReader::new(plugin.stdout.take().unwrap()).lines();
    let announcement = lines.next_line().await?.ok_or("Register Plugin: Read Its Bound Endpoint")?;
    let url =
        format!("http://{}", announcement.strip_prefix("harmos listening ").ok_or("Register Plugin: Read Its Listener Announcement")?);
    tokio::spawn(async move { while let Ok(Some(_)) = lines.next_line().await {} });
    Ok((plugin, url))
}

/* === Tests === */

#[test]
fn unmanaged_plugin_keeps_one_entry_across_restarts_until_removed() -> Result<(), Box<dyn Error>> {
    super::credentials::run(Box::pin(async {
        time::timeout(super::HANG_GUARD, async {
            // Start the Candidate Plugin at an Ephemeral Address.
            let (mut plugin, url) = serve_fixture().await?;

            // Register the Remote Address and List It as an Unmanaged Ready Entry.
            let host = super::HostProcess::start().await?;
            let result: Result<(), Box<dyn Error>> = async {
                let registered: Value =
                    serde_json::from_str(&super::run(&["--endpoint", &host.endpoint, "plugin", "register", &url, "--json"]).await?)?;
                assert_eq!(registered["entry"]["placement"]["remote"]["url"], url);
                assert_eq!(registered["entry"]["state"], "PLUGIN_STATE_READY");
                let name = registered["entry"]["name"].as_str().ok_or("Register Plugin: Return Its Entry Name")?.to_owned();
                let listed = super::run(&["--endpoint", &host.endpoint, "plugin", "list"]).await?;
                assert_eq!(
                    listed.lines().map(|line| line.split('\t').collect::<Vec<_>>()).collect::<Vec<_>>(),
                    [[name.as_str(), "Unmanaged", "Ready"]]
                );

                // Restarting the Plugin at a New Address Reuses Its One Entry.
                plugin.kill().await?;
                let (restarted, moved) = serve_fixture().await?;
                plugin = restarted;
                let again: Value =
                    serde_json::from_str(&super::run(&["--endpoint", &host.endpoint, "plugin", "register", &moved, "--json"]).await?)?;
                assert_eq!(again["entry"]["name"], name.as_str());
                assert_eq!(again["entry"]["placement"]["remote"]["url"], moved);
                assert_eq!(again["entry"]["state"], "PLUGIN_STATE_READY");
                let listed: Value = serde_json::from_str(&super::run(&["--endpoint", &host.endpoint, "plugin", "list", "--json"]).await?)?;
                assert_eq!(listed["entries"].as_array().map(Vec::len), Some(1));

                // Only an Explicit Remove Deletes the Entry, and the User-Owned Process Keeps Serving.
                super::run(&["--endpoint", &host.endpoint, "plugin", "remove", &name]).await?;
                let listed: Value = serde_json::from_str(&super::run(&["--endpoint", &host.endpoint, "plugin", "list", "--json"]).await?)?;
                assert!(listed.get("entries").is_none_or(|entries| entries.as_array().unwrap().is_empty()));
                super::run(&["plugin", "inspect", "--url", &moved, "--json"]).await?;
                Ok(())
            }
            .await;

            // Close Only the Host and Plugin Started by This Story.
            let closed = host.close().await;
            plugin.kill().await?;
            closed?;
            result
        })
        .await?
    }))
}

#[test]
fn managed_installation_preserves_resolved_bundle_version() -> Result<(), Box<dyn Error>> {
    super::credentials::run(Box::pin(async {
        time::timeout(super::HANG_GUARD, async {
            // Bundle a Native Starter with an Explicit Version Override.
            let directory = super::credentials::tempdir()?;
            let project = directory.path().join("plugin");
            fs::create_dir(&project)?;
            fs::copy(super::fixtures().join(format!("rust{EXE_SUFFIX}")), project.join(format!("plugin{EXE_SUFFIX}")))?;
            fs::write(project.join("dokime.toml"), "[plugin]\nversion = '8.7.6'\n[scripts]\nexecutable = 'plugin'\n")?;
            let output = directory.path().join("overridden.dokime");
            super::run(&["plugin", "bundle", project.to_str().unwrap(), "--out", output.to_str().unwrap()]).await?;

            // Verify Both Frozen Versions Before Installing the Bundle.
            let bytes = fs::read(&output)?;
            let bundle = Bundle::inspect(&bytes, Limits::default())?;
            let version = bundle.manifest().identity.version.to_string();
            assert_eq!(version, "8.7.6");
            let descriptor = dokime_protocol::descriptor_pool().get_message_by_name("harmos.plugin.v1.InstanceGetResponse").unwrap();
            let description =
                DynamicMessage::decode(descriptor, bundle.file("description.pb").ok_or("Bundle Plugin: Freeze Its Description")?)?;
            assert_eq!(description.get_field_by_name("version").unwrap().as_ref(), &prost_reflect::Value::String(version.clone()));

            // Read the Installed Version After the Host Reports Readiness.
            let host = super::HostProcess::start().await?;
            let installed = super::run(&["--endpoint", &host.endpoint, "plugin", "install", output.to_str().unwrap(), "--json"]).await;
            host.close().await?;
            let installed: Value = serde_json::from_str(&installed?)?;
            assert_eq!(installed["entry"]["description"]["version"], version);
            Ok(())
        })
        .await?
    }))
}

#[test]
fn first_hour_from_rust_bundle_to_python_remote() -> Result<(), Box<dyn Error>> {
    super::credentials::run(Box::pin(async {
        time::timeout(super::HANG_GUARD, async {
            // Create a Rust Project From the Embedded SDK Starter
            let directory = super::credentials::tempdir()?;
            let project = directory.path().join("first-rust");
            super::run(&["new", project.to_str().unwrap(), "--lang", "rust", "--from", super::root().to_str().unwrap()]).await?;
            assert!(project.join("src/bin/client.rs").is_file());

            // Bundle the Executable Built From Those Exact Starter Sources
            let executable = super::fixtures().join(format!("rust{}", EXE_SUFFIX));
            let prepared = directory.path().join("prepared-plugin");
            fs::create_dir(&prepared)?;
            fs::copy(&executable, prepared.join(format!("plugin{EXE_SUFFIX}")))?;
            fs::write(prepared.join("dokime.toml"), "[plugin]\n[scripts]\nexecutable = 'plugin'\n")?;
            let bundle = directory.path().join("first.dokime");
            super::run(&["plugin", "bundle", prepared.to_str().unwrap(), "--out", bundle.to_str().unwrap()]).await?;
            super::run(&["plugin", "inspect", "--bundle", bundle.to_str().unwrap(), "--json"]).await?;

            // Preserve the Plugin Identity and Frozen Declarations in the Native Bundle
            let bytes = fs::read(&bundle)?;
            let packed = Bundle::inspect(&bytes, Limits::default())?;
            let descriptor = dokime_protocol::descriptor_pool().get_message_by_name("harmos.plugin.v1.InstanceGetResponse").unwrap();
            let description =
                DynamicMessage::decode(descriptor, packed.file("description.pb").ok_or("Bundle Plugin: Missing Declarations")?)?;
            let mut json = Vec::new();
            let mut serializer = Serializer::new(&mut json);
            description.serialize_with_options(&mut serializer, &SerializeOptions::new().use_proto_field_name(true))?;
            let description: Value = serde_json::from_slice(&json)?;
            assert_eq!(packed.manifest().identity.name, description["id"]);
            assert_eq!(packed.manifest().identity.version.to_string(), description["version"]);
            assert!(description["declarations"].is_object());
            assert!(packed.manifest().launch.is_none());
            let entrypoint = packed.manifest().files.iter().find(|file| file.executable.is_some()).unwrap();
            assert_eq!(entrypoint.executable.as_ref().unwrap().facility, Facility::Sidecar);
            assert_eq!(packed.file(&entrypoint.path), Some(fs::read(&executable)?.as_slice()));

            // Start a Host with an Explicit Endpoint and Controlled Clock
            let credentials = super::credentials::CredentialStore::start(directory.path()).await?;
            let storage = directory.path().join("storage");
            let mut host = super::command()
                .envs(credentials.environment.iter().cloned())
                .args(["--endpoint", "127.0.0.1:0", "server", "--controlled-clock", "--storage"])
                .arg(&storage)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()?;
            let mut report_errors = Vec::new();
            let mut remote_plugin = None;
            let result: Result<(), Box<dyn Error>> = async {
                let mut host_lines = BufReader::new(host.stdout.take().unwrap()).lines();
                let endpoint = host_lines.next_line().await?.ok_or("Start Host: Missing Listener Announcement")?;

                // Add the Bundle as a Managed Entry Through the Host Service
                let installation: Value = serde_json::from_str(
                    &super::run(&["--endpoint", &endpoint, "plugin", "install", bundle.to_str().unwrap(), "--json"]).await?,
                )?;
                assert_eq!(installation["entry"]["description"]["declarations"], description["declarations"]);
                let entry = installation["entry"]["name"].as_str().ok_or("Install Plugin: Missing Managed Entry")?;
                let suites: Value = serde_json::from_str(&super::run(&["--endpoint", &endpoint, "catalog", "list", "--json"]).await?)?;
                let suite = suites["values"][0]["metadata"]["identifier"].as_str().ok_or("List Suites: Missing Suite Identity")?;
                // Preview Without Registration, Then Recover the Same Registered Reply
                let plan =
                    serde_json::json!({"plan":{"selections":[{"suite_id":suite,"template_id":"capture.confirmation"}],"max_cases":1}});
                let input = directory.path().join("plan.json");
                fs::write(&input, serde_json::to_vec(&plan)?)?;
                let preview: Value = serde_json::from_str(
                    &super::run(&["--endpoint", &endpoint, "plan", "preview", "--input-file", input.to_str().unwrap(), "--json"]).await?,
                )?;
                assert_eq!(preview["cases"].as_array().unwrap().len(), 1);
                let mut plan = plan;
                plan["request_id"] = "first-hour-selection".into();
                fs::write(&input, serde_json::to_vec(&plan)?)?;
                let registered =
                    super::run(&["--endpoint", &endpoint, "plan", "register", "--input-file", input.to_str().unwrap(), "--json"]).await?;
                let replayed =
                    super::run(&["--endpoint", &endpoint, "plan", "register", "--input-file", input.to_str().unwrap(), "--json"]).await?;
                assert_eq!(serde_json::from_str::<Value>(&registered)?, serde_json::from_str::<Value>(&replayed)?);
                plan["plan"]["max_cases"] = 2.into();
                fs::write(&input, serde_json::to_vec(&plan)?)?;
                assert!(
                    super::run(&["--endpoint", &endpoint, "plan", "register", "--input-file", input.to_str().unwrap(), "--json"])
                        .await
                        .is_err()
                );

                let cases = directory.path().join("rust-cases.pb");
                super::run(&[
                    "--endpoint",
                    &endpoint,
                    "plan",
                    "materialize",
                    "--suite",
                    suite,
                    "--template",
                    "capture.confirmation",
                    "--out",
                    cases.to_str().unwrap(),
                ])
                .await?;
                let cycle: Value = serde_json::from_str(
                    &super::run(&["--endpoint", &endpoint, "cycle", "create", "--cases", cases.to_str().unwrap(), "--json"]).await?,
                )?;
                let cycle_id = cycle["cycle"]["cycle_id"].as_str().ok_or("Start Cycle: Missing Cycle")?;
                let executions: Value = serde_json::from_str(
                    &super::run(&["--endpoint", &endpoint, "execution", "list", "--cycle", cycle_id, "--json"]).await?,
                )?;
                let execution = executions["executions"][0]["summary"]["execution_id"].as_str().ok_or("Start Cycle: Missing Execution")?;
                super::run(&["--endpoint", &endpoint, "execution", "prompt", "next", execution, "--json"]).await?;
                super::run(&["--endpoint", &endpoint, "execution", "prompt", "answer", execution, "--action", "0", "--accepted"]).await?;
                super::run(&["--endpoint", &endpoint, "execution", "wait", execution]).await?;
                let report = directory.path().join("rust.html");
                if let Err(error) =
                    super::run(&["--endpoint", &endpoint, "execution", "report", execution, "--out", report.to_str().unwrap()]).await
                {
                    assert!(!report.exists());
                    report_errors.push(error.to_string());
                } else {
                    assert!(!fs::read(&report)?.is_empty());
                }

                // Read Frozen Descriptors Through the Digest-Cached SDK Path.
                let output =
                    super::run(&["--endpoint", &endpoint, "execution", "show", execution, execution, "--schemas", "--json"]).await?;
                let responses = Deserializer::from_str(&output).into_iter::<Value>().collect::<Result<Vec<_>, _>>()?;
                assert_eq!(responses.len(), 4);
                assert!(!responses[1]["file"].as_array().unwrap().is_empty());
                assert_eq!(responses[1], responses[3]);

                // Download the Completed Cycle Without Publishing a Report
                let report = directory.path().join("cycle.html");
                let cycle_id = cycle["cycle"]["cycle_id"].as_str().ok_or("Start Cycle: Missing Cycle")?;
                if let Err(error) =
                    super::run(&["--endpoint", &endpoint, "cycle", "report", cycle_id, "--out", report.to_str().unwrap()]).await
                {
                    assert!(!report.exists());
                    report_errors.push(error.to_string());
                } else {
                    let html = fs::read_to_string(&report)?;
                    assert!(html.contains("<html"));
                    assert!(
                        super::run(&["--endpoint", &endpoint, "cycle", "report", cycle_id, "--out", report.to_str().unwrap()])
                            .await
                            .is_err()
                    );
                    assert_eq!(fs::read_to_string(&report)?, html);
                }

                // Remove the Rust Entry Before Attaching the Same Example Identity Remotely
                super::run(&["--endpoint", &endpoint, "plugin", "remove", entry]).await?;

                // Create and Serve the Pure Python Starter Remotely
                let project = directory.path().join("first-python");
                super::run(&["new", project.to_str().unwrap(), "--lang", "python", "--from", super::root().to_str().unwrap()]).await?;
                remote_plugin = Some(
                    super::command()
                        .envs(credentials.environment.iter().cloned())
                        .args(["dev", "--plugin", project.to_str().unwrap(), "--no-desktop"])
                        .stdin(Stdio::piped())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::inherit())
                        .spawn()?,
                );
                let plugin = remote_plugin.as_mut().unwrap();
                let mut lines = BufReader::new(plugin.stdout.take().unwrap()).lines();
                let endpoint = lines.next_line().await?.ok_or("Develop Python: Read the Host Endpoint")?;
                lines.next_line().await?.ok_or("Develop Python: Read the Storage Root")?;
                let suites: Value = serde_json::from_str(&super::run(&["--endpoint", &endpoint, "catalog", "list", "--json"]).await?)?;
                let suite = suites["values"].as_array().unwrap().last().unwrap()["metadata"]["identifier"].as_str().unwrap();
                let cases = directory.path().join("python-cases.pb");
                let parameters = directory.path().join("parameters.json");
                fs::write(&parameters, b"{}")?;
                super::run(&[
                    "--endpoint",
                    &endpoint,
                    "plan",
                    "materialize",
                    "--suite",
                    suite,
                    "--template",
                    "capture.confirmation",
                    "--out",
                    cases.to_str().unwrap(),
                    "--parameters",
                    parameters.to_str().unwrap(),
                ])
                .await?;
                let cycle: Value = serde_json::from_str(
                    &super::run(&["--endpoint", &endpoint, "cycle", "create", "--cases", cases.to_str().unwrap(), "--json"]).await?,
                )?;
                let cycle_id = cycle["cycle"]["cycle_id"].as_str().ok_or("Start Cycle: Missing Cycle")?;
                let executions: Value = serde_json::from_str(
                    &super::run(&["--endpoint", &endpoint, "execution", "list", "--cycle", cycle_id, "--json"]).await?,
                )?;
                let execution = executions["executions"][0]["summary"]["execution_id"].as_str().unwrap();
                super::run(&["--endpoint", &endpoint, "execution", "prompt", "next", execution, "--json"]).await?;
                super::run(&["--endpoint", &endpoint, "execution", "prompt", "answer", execution, "--action", "0", "--accepted"]).await?;
                super::run(&["--endpoint", &endpoint, "execution", "wait", execution]).await?;
                let report = directory.path().join("python.html");
                if let Err(error) =
                    super::run(&["--endpoint", &endpoint, "execution", "report", execution, "--out", report.to_str().unwrap()]).await
                {
                    assert!(!report.exists());
                    report_errors.push(error.to_string());
                } else {
                    assert!(!fs::read(&report)?.is_empty());
                }

                Ok(())
            }
            .await;
            // Close the Host Before Discarding Its Credentials, Including on Failure.
            drop(host.stdin.take());
            let host_status = host.wait().await?;
            if let Some(mut plugin) = remote_plugin {
                drop(plugin.stdin.take());
                assert!(plugin.wait().await?.success());
            }
            result?;
            assert!(host_status.success());
            if !report_errors.is_empty() {
                return Err(report_errors.join("\n").into());
            }
            Ok(())
        })
        .await?
    }))
}
