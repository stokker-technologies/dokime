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
use std::{env::consts, error::Error};

// Third Party Imports
use serde_json::Value;
use tokio::{process::Command, time};

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
fn operator_identifies_build() -> Result<(), Box<dyn Error>> {
    super::credentials::run(Box::pin(async {
        time::timeout(super::HANG_GUARD, async {
            // Identify the Client Before Connecting to the Host It Will Control
            let version: Value = serde_json::from_str(&super::run(&["--version", "--json"]).await?)?;
            let brief = super::run(&["--version"]).await?;
            let short_flag = super::run(&["-V"]).await?;
            assert_eq!(short_flag, brief);
            let revision = version["revision"].as_str().ok_or("Build Did Not Retain Its Source Revision")?;
            let commit = revision.strip_suffix("-dirty").unwrap_or(revision);
            let short = &commit[..commit.len().min(8)];
            let dirty = if revision.ends_with("-dirty") { "-dirty" } else { "" };
            let channel = version["channel"].as_str().unwrap_or("local");
            let package = version["version"].as_str().unwrap();
            assert_eq!(brief.trim(), format!("{package} ({short}{dirty}, {channel})"));
            let object = format!("{commit}^{{commit}}");
            let status = Command::from(dokime_runtime::process::command("git"))
                .current_dir(super::root())
                .args(["cat-file", "-e", &object])
                .status()
                .await?;
            assert!(status.success());
            assert_eq!(version["target_os"], consts::OS);
            assert_eq!(version["target_arch"], consts::ARCH);

            // Read the Running Host's Identity Alongside the Local Build
            let host = super::HostProcess::start().await?;
            let status: Value = serde_json::from_str(&super::run(&["--endpoint", &host.endpoint, "status", "--json"]).await?)?;
            assert!(status["status"]["identity"].is_object());
            host.close().await?;
            Ok(())
        })
        .await?
    }))
}
