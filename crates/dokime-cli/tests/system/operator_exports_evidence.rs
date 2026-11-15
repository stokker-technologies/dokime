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

fn members(archive: &[u8], names: &[&str]) -> bool {
    archive.starts_with(b"PK") && names.iter().all(|name| archive.windows(name.len()).any(|window| window == name.as_bytes()))
}

/* === Tests === */

#[test]
fn operator_exports_evidence() -> Result<(), Box<dyn std::error::Error>> {
    super::credentials::run(Box::pin(async {
        tokio::time::timeout(super::HANG_GUARD, async {
            let host = super::HostProcess::start().await?;
            let endpoint = host.endpoint.clone();
            let directory = super::credentials::tempdir()?;

            // Serve the Boundary Fixture and Materialize Its Case
            let fixture = super::host_fixtures().join(format!("dokime-rust-boundary{}", std::env::consts::EXE_SUFFIX));
            assert!(fixture.is_file(), "Build Host Fixtures Through mise run build:host-fixtures -- --language rust");
            let mut plugin = super::command()
                .args(["plugin", "dev", "--executable"])
                .arg(fixture)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()?;
            let mut lines = BufReader::new(plugin.stdout.take().unwrap()).lines();
            let url = lines.next_line().await?.ok_or("Serve Fixture: Missing Listener Announcement")?;
            super::run(&["--endpoint", &endpoint, "plugin", "add", &url]).await?;
            let suites: serde_json::Value =
                serde_json::from_str(&super::run(&["--endpoint", &endpoint, "catalog", "list", "--json"]).await?)?;
            let suite = suites["values"][0]["metadata"]["identifier"].as_str().ok_or("List Suites: Missing Suite Identity")?;
            let cases = directory.path().join("cases.pb");
            let arguments = ["--endpoint", &endpoint, "plan", "materialize", "--suite", suite, "--template", "execution.boundary"];
            super::run(&[arguments.as_slice(), &["--out", cases.to_str().unwrap()]].concat()).await?;

            // Read the Confirmed Cycle Identity From the File Rather Than Standard Output
            let ids = directory.path().join("ids.txt");
            std::fs::write(
                &ids, "stale
",
            )?;
            let cycle: serde_json::Value = serde_json::from_str(
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
                std::fs::read_to_string(&ids)?,
                format!(
                    "{cycle_id}
"
                )
            );
            let executions: serde_json::Value =
                serde_json::from_str(&super::run(&["--endpoint", &endpoint, "execution", "list", "--cycle", cycle_id, "--json"]).await?)?;
            let execution = executions["executions"][0]["summary"]["execution_id"].as_str().ok_or("Start Cycle: Missing Execution")?;
            super::run(&["--endpoint", &endpoint, "execution", "wait", execution]).await?;
            super::run(&["--endpoint", &endpoint, "cycle", "seal", cycle_id]).await?;

            // Export the Execution's Self-Contained Evidence ZIP and Refuse to Replace It
            let exported = directory.path().join("execution.zip");
            let path =
                super::run(&["--endpoint", &endpoint, "execution", "export", execution, "--out", exported.to_str().unwrap()]).await?;
            assert_eq!(path.trim(), exported.to_str().unwrap());
            let archive = std::fs::read(&exported)?;
            assert!(members(&archive, &["manifest.json", "evidence.pb"]));
            let repeated = super::command()
                .args(["--endpoint", &endpoint, "execution", "export", execution, "--out", exported.to_str().unwrap()])
                .output()
                .await?;
            assert_eq!(repeated.status.code(), Some(2));
            assert_eq!(std::fs::read(&exported)?, archive);

            // Export the Cycle ZIP With Its Execution Member
            let exported = directory.path().join("cycle.zip");
            super::run(&["--endpoint", &endpoint, "cycle", "export", cycle_id, "--out", exported.to_str().unwrap()]).await?;
            let member = format!("executions/{execution}.zip");
            assert!(members(&std::fs::read(&exported)?, &[member.as_str(), "manifest.json"]));

            drop(plugin.stdin.take());
            plugin.wait().await?;
            host.close().await?;
            Ok(())
        })
        .await?
    }))
}
