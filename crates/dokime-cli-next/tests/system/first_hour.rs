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
async fn first_hour_from_rust_bundle_to_python_remote() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::HANG_GUARD, async {
        // Create a Rust Project From the Embedded SDK Starter
        let directory = tempfile::tempdir()?;
        let project = directory.path().join("first-rust");
        super::run(&["new", project.to_str().unwrap(), "--lang", "rust"]).await?;
        let built_project = super::fixtures().join("rust");
        assert_eq!(std::fs::read(project.join("src/main.rs"))?, std::fs::read(built_project.join("src/main.rs"))?);

        // Bundle the Executable Built From Those Exact Starter Sources
        let executable = super::fixtures().join(format!("dokime-starter{}", std::env::consts::EXE_SUFFIX));
        let bundle = directory.path().join("first.dokime");
        super::run(&["plugin", "bundle", "--executable", executable.to_str().unwrap(), "--out", bundle.to_str().unwrap()]).await?;
        super::run(&["plugin", "inspect", "--bundle", bundle.to_str().unwrap(), "--json"]).await?;

        // Start a Host With an Explicit Endpoint and Controlled Clock
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
        let mut host_lines = BufReader::new(host.stdout.take().unwrap()).lines();
        let endpoint = host_lines.next_line().await?.ok_or("Start Host: Missing Listener Announcement")?;

        // Add the Bundle as a Managed Entry Through the Host Service
        let installation: serde_json::Value =
            serde_json::from_str(&super::run(&["--endpoint", &endpoint, "plugin", "install", bundle.to_str().unwrap(), "--json"]).await?)?;
        let entry = installation["plugin"]["entry"].as_str().ok_or("Install Plugin: Missing Managed Entry")?;
        let suites: serde_json::Value = serde_json::from_str(&super::run(&["--endpoint", &endpoint, "catalog", "list", "--json"]).await?)?;
        let suite = suites["suites"][0]["suite_id"].as_str().ok_or("List Suites: Missing Suite Identity")?;
        // Preview Without Registration, Then Recover the Same Registered Reply
        let plan = serde_json::json!({"plan":{"selections":[{"suite_id":suite,"template_id":"example.capture"}],"max_cases":1}});
        let input = directory.path().join("plan.json");
        std::fs::write(&input, serde_json::to_vec(&plan)?)?;
        let preview: serde_json::Value = serde_json::from_str(
            &super::run(&["--endpoint", &endpoint, "plan", "preview", "--input-file", input.to_str().unwrap(), "--json"]).await?,
        )?;
        assert_eq!(preview["cases"].as_array().unwrap().len(), 1);
        let mut plan = plan;
        plan["idempotency_key"] = "first-hour-selection".into();
        std::fs::write(&input, serde_json::to_vec(&plan)?)?;
        let registered =
            super::run(&["--endpoint", &endpoint, "plan", "register", "--input-file", input.to_str().unwrap(), "--json"]).await?;
        let replayed =
            super::run(&["--endpoint", &endpoint, "plan", "register", "--input-file", input.to_str().unwrap(), "--json"]).await?;
        assert_eq!(serde_json::from_str::<serde_json::Value>(&registered)?, serde_json::from_str::<serde_json::Value>(&replayed)?);
        plan["plan"]["max_cases"] = 2.into();
        std::fs::write(&input, serde_json::to_vec(&plan)?)?;
        assert!(
            super::run(&["--endpoint", &endpoint, "plan", "register", "--input-file", input.to_str().unwrap(), "--json"]).await.is_err()
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
            "example.capture",
            "--out",
            cases.to_str().unwrap(),
        ])
        .await?;
        let cycle: serde_json::Value = serde_json::from_str(
            &super::run(&["--endpoint", &endpoint, "cycle", "create", "--cases", cases.to_str().unwrap(), "--json"]).await?,
        )?;
        let execution = cycle["execution_ids"][0].as_str().ok_or("Start Cycle: Missing Execution")?;
        super::run(&["--endpoint", &endpoint, "execution", "prompt", "next", execution, "--json"]).await?;
        super::run(&["--endpoint", &endpoint, "execution", "prompt", "answer", execution, "--action", "yes", "--accepted"]).await?;
        super::run(&["--endpoint", &endpoint, "execution", "wait", execution]).await?;
        let report = directory.path().join("rust.html");
        super::run(&["--endpoint", &endpoint, "execution", "report", execution, "--out", report.to_str().unwrap()]).await?;
        assert!(!std::fs::read(&report)?.is_empty());

        // Download the Completed Cycle Without Publishing a Report
        let report = directory.path().join("cycle.html");
        let cycle_id = cycle["cycle_id"].as_str().ok_or("Start Cycle: Missing Cycle")?;
        super::run(&["--endpoint", &endpoint, "cycle", "report", cycle_id, "--out", report.to_str().unwrap()]).await?;
        let html = std::fs::read_to_string(&report)?;
        assert!(html.contains("<html"));
        assert!(super::run(&["--endpoint", &endpoint, "cycle", "report", cycle_id, "--out", report.to_str().unwrap()]).await.is_err());
        assert_eq!(std::fs::read_to_string(&report)?, html);

        // Remove the Rust Entry Before Attaching the Same Example Identity Remotely
        super::run(&["--endpoint", &endpoint, "plugin", "remove", entry]).await?;

        // Create and Serve the Pure Python Starter Remotely
        let project = directory.path().join("first-python");
        super::run(&["new", project.to_str().unwrap(), "--lang", "python"]).await?;
        let python = std::env::var_os("DOKIME_PYTHON")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| super::root().join(if cfg!(windows) { ".venv/Scripts/python.exe" } else { ".venv/bin/python" }));
        let mut plugin = tokio::process::Command::from(dokime_runtime_next::process::command(python))
            .arg(project.join("main.py"))
            .args(["--address", "127.0.0.1:0"])
            .current_dir(&project)
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()?;
        let mut lines = BufReader::new(plugin.stdout.take().unwrap()).lines();
        let url = lines.next_line().await?.ok_or("Serve Python: Missing Listener Announcement")?;
        super::run(&["--endpoint", &endpoint, "plugin", "add", &url]).await?;
        let suites: serde_json::Value = serde_json::from_str(&super::run(&["--endpoint", &endpoint, "catalog", "list", "--json"]).await?)?;
        let suite = suites["suites"].as_array().unwrap().last().unwrap()["suite_id"].as_str().unwrap();
        let cases = directory.path().join("python-cases.pb");
        let parameters = directory.path().join("parameters.json");
        std::fs::write(&parameters, br#"{"limit":[{"number":10}]}"#)?;
        super::run(&[
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
            "--parameters",
            parameters.to_str().unwrap(),
        ])
        .await?;
        let cycle: serde_json::Value = serde_json::from_str(
            &super::run(&["--endpoint", &endpoint, "cycle", "create", "--cases", cases.to_str().unwrap(), "--json"]).await?,
        )?;
        let execution = cycle["execution_ids"][0].as_str().unwrap();
        super::run(&["--endpoint", &endpoint, "execution", "prompt", "next", execution, "--json"]).await?;
        super::run(&["--endpoint", &endpoint, "execution", "prompt", "answer", execution, "--action", "0", "--accepted"]).await?;
        super::run(&["--endpoint", &endpoint, "execution", "wait", execution]).await?;
        let report = directory.path().join("python.html");
        super::run(&["--endpoint", &endpoint, "execution", "report", execution, "--out", report.to_str().unwrap()]).await?;
        assert!(!std::fs::read(&report)?.is_empty());

        // Close the Host and the Owned Remote Process
        drop(host.stdin.take());
        assert!(host.wait().await?.success());
        plugin.kill().await?;
        plugin.wait().await?;
        Ok(())
    })
    .await?
}

/* === Tests === */
