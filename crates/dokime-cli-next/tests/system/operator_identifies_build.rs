/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Identify the Command Build Alongside the Running Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

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

#[tokio::test]
async fn operator_identifies_build() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::HANG_GUARD, async {
        // Identify the Client Before Connecting to the Host It Will Control
        let version: serde_json::Value = serde_json::from_str(&super::run(&["version", "--json"]).await?)?;
        let brief = super::run(&["--version"]).await?;
        assert!(brief.contains(version["version"].as_str().unwrap()));
        let revision = version["revision"].as_str().ok_or("Build Did Not Retain Its Source Revision")?;
        let commit = revision.strip_suffix("-dirty").unwrap_or(revision);
        let object = format!("{commit}^{{commit}}");
        let status = tokio::process::Command::from(dokime_runtime_next::process::command("git"))
            .current_dir(super::root()).args(["cat-file", "-e", &object]).status().await?;
        assert!(status.success());
        assert_eq!(version["target_os"], std::env::consts::OS);
        assert_eq!(version["target_arch"], std::env::consts::ARCH);

        // Read the Running Host's Identity Alongside the Local Build
        let host = super::HostProcess::start().await?;
        let status: serde_json::Value = serde_json::from_str(&super::run(&["--endpoint", &host.endpoint, "status", "--json"]).await?)?;
        assert!(status["status"]["identity"].is_object());
        host.close().await?;
        Ok(())
    }).await?
}

