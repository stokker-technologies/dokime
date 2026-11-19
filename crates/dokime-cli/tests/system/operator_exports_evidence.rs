/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Export Self-Contained Evidence and Script Confirmed Identities

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::env::consts::EXE_SUFFIX;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Stdio;

// Third Party Imports
use serde_json::Value;
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

fn members(archive: &[u8], names: &[&str]) -> bool {
    archive.starts_with(b"PK") && names.iter().all(|name| archive.windows(name.len()).any(|window| window == name.as_bytes()))
}

/* === Tests === */

#[test]
fn operator_exports_evidence() -> Result<(), Box<dyn Error>> {
    super::credentials::run(Box::pin(async {
        tokio::time::timeout(super::HANG_GUARD, async {
            let directory = super::credentials::tempdir()?;

            // Develop the Boundary Fixture and Materialize Its Case
            let fixture = super::host_fixtures().join(format!("dokime-rust-boundary{}", EXE_SUFFIX));
            assert!(fixture.is_file(), "Build Host Fixtures Through mise run build:host-fixtures -- --language rust");
            let prepared = directory.path().join("prepared-plugin");
            fs::create_dir(&prepared)?;
            fs::copy(&fixture, prepared.join(format!("plugin{EXE_SUFFIX}")))?;
            fs::write(prepared.join("dokime.toml"), "[plugin]\n[scripts]\nexecutable = 'plugin'\n")?;
            let bundle = directory.path().join("boundary.dokime");
            super::run(&["plugin", "bundle", prepared.to_str().unwrap(), "--out", bundle.to_str().unwrap()]).await?;
            let credentials = super::credentials::CredentialStore::start(directory.path()).await?;
            let mut plugin = super::command()
                .envs(credentials.environment.iter().cloned())
                .args(["dev", "--no-desktop", "--plugin"])
                .arg(bundle)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()?;
            let mut lines = BufReader::new(plugin.stdout.take().unwrap()).lines();
            let endpoint = lines.next_line().await?.ok_or("Develop Fixture: Missing Host Endpoint")?;
            let data = lines.next_line().await?.ok_or("Develop Fixture: Missing Data Root")?;
            let data = PathBuf::from(data.strip_prefix("Data Root: ").ok_or("Read the Development Data Root")?);
            let suites: Value = serde_json::from_str(&super::run(&["--endpoint", &endpoint, "catalog", "list", "--json"]).await?)?;
            let suite = suites["values"][0]["metadata"]["identifier"].as_str().ok_or("List Suites: Missing Suite Identity")?;
            let cases = directory.path().join("cases.pb");
            let arguments = ["--endpoint", &endpoint, "plan", "materialize", "--suite", suite, "--template", "execution.boundary"];
            super::run(&[arguments.as_slice(), &["--out", cases.to_str().unwrap()]].concat()).await?;

            // Read the Confirmed Cycle Identity From the File Rather Than Standard Output
            let ids = directory.path().join("ids.txt");
            fs::write(
                &ids, "stale
",
            )?;
            let cycle: Value = serde_json::from_str(
                &super::run(&[
                    "--endpoint",
                    &endpoint,
                    "cycle",
                    "create",
                    "--cases",
                    cases.to_str().unwrap(),
                    "--json",
                    "--id-file",
                    ids.to_str().unwrap(),
                ])
                .await?,
            )?;
            let cycle_id = cycle["cycle"]["cycle_id"].as_str().ok_or("Start Cycle: Missing Cycle")?;
            assert_eq!(
                fs::read_to_string(&ids)?,
                format!(
                    "{cycle_id}
"
                )
            );
            let executions: Value =
                serde_json::from_str(&super::run(&["--endpoint", &endpoint, "execution", "list", "--cycle", cycle_id, "--json"]).await?)?;
            let execution = executions["executions"][0]["summary"]["execution_id"].as_str().ok_or("Start Cycle: Missing Execution")?;
            super::run(&["--endpoint", &endpoint, "execution", "wait", execution]).await?;
            super::run(&["--endpoint", &endpoint, "cycle", "seal", cycle_id]).await?;

            // Export the Execution's Self-Contained Evidence ZIP and Refuse to Replace It
            let exported = directory.path().join("execution.zip");
            let path =
                super::run(&["--endpoint", &endpoint, "execution", "export", execution, "--out", exported.to_str().unwrap()]).await?;
            assert_eq!(path.trim(), exported.to_str().unwrap());
            let archive = fs::read(&exported)?;
            assert!(members(&archive, &["manifest.json", "evidence.pb", "report.html"]));
            let repeated = super::command()
                .args(["--endpoint", &endpoint, "execution", "export", execution, "--out", exported.to_str().unwrap()])
                .output()
                .await?;
            assert_eq!(repeated.status.code(), Some(2));
            assert_eq!(fs::read(&exported)?, archive);

            // Export the Cycle ZIP With Its Execution Member
            let exported = directory.path().join("cycle.zip");
            super::run(&["--endpoint", &endpoint, "cycle", "export", cycle_id, "--out", exported.to_str().unwrap()]).await?;
            let member = format!("executions/{execution}.zip");
            let retained = fs::read(&exported)?;
            assert!(members(&retained, &[member.as_str(), "manifest.json", "report.html", "evidence.pb"]));
            let second = directory.path().join("cycle-again.zip");
            super::run(&["--endpoint", &endpoint, "cycle", "export", cycle_id, "--out", second.to_str().unwrap()]).await?;
            assert_eq!(fs::read(second)?, retained);

            // Download the Retained Cycle Report Twice and an Execution Report.
            let report = directory.path().join("cycle.html");
            let again = directory.path().join("cycle-again.html");
            super::run(&["--endpoint", &endpoint, "cycle", "report", cycle_id, "--out", report.to_str().unwrap()]).await?;
            super::run(&["--endpoint", &endpoint, "cycle", "report", cycle_id, "--out", again.to_str().unwrap()]).await?;
            assert_eq!(fs::read(report)?, fs::read(again)?);
            let report = directory.path().join("execution.html");
            super::run(&["--endpoint", &endpoint, "execution", "report", execution, "--out", report.to_str().unwrap()]).await?;
            assert!(!fs::read(report)?.is_empty());

            drop(plugin.stdin.take());
            assert!(plugin.wait().await?.success());
            assert!(!data.exists());
            Ok(())
        })
        .await?
    }))
}
