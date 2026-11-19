/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Package an Installer from a Local Kit

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{error::Error, fs, time::Duration};

// Third Party Imports
use serde_json::json;
use tokio::process::Command;

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

/* === Tests === */

#[tokio::test]
async fn author_packages_organization_installer() -> Result<(), Box<dyn Error>> {
    tokio::time::timeout(Duration::from_secs(600), async {
        // Discover Installer Packaging Without Connecting to a Host.
        let binary = env!("CARGO_BIN_EXE_dokime");
        let help = Command::from(process::command(binary))
            .env("DOKIME_AUTHENTICODE_PROVIDER", "none")
            .kill_on_drop(true)
            .args(["installer", "package", "--help"])
            .output()
            .await?;
        assert!(help.status.success());
        // Author a Neutral Manifest and Supply a Local Kit; Never Consult Previous Releases.
        let root = tempfile::tempdir()?;
        let manifest = root.path().join("installer.toml");
        let output = root.path().join("output");
        let kit = root.path().join("kit");
        fs::create_dir_all(&kit)?;
        let format = if cfg!(target_os = "windows") { "nsis" } else { "deb" };
        let text =
            format!("[installer]\nversion='1.2.3'\nidentifier='com.example.testhub'\nformats=['{format}']\n[dokime]\nversion='nightly'\n");
        fs::write(&manifest, text)?;
        // Refuse an Incomplete Kit Before Native Tool Invocation or Output Publication.
        fs::write(
            kit.join("distribution.json"),
            serde_json::to_vec(&json!({"schema":1,"version":"1.2.3","target":dokime_packaging::bundle::native_target(),"files":{}}))?,
        )?;
        let refused = Command::from(process::command(binary))
            .env("DOKIME_AUTHENTICODE_PROVIDER", "none")
            .kill_on_drop(true)
            .args(["installer", "package", "--manifest"])
            .arg(&manifest)
            .arg("--artifacts")
            .arg(&kit)
            .arg("--output")
            .arg(&output)
            .output()
            .await?;
        assert_eq!(refused.status.code(), Some(2));
        assert!(!output.exists());
        let error = String::from_utf8(refused.stderr)?;
        assert!(error.contains("distribution"), "{error}");
        // Refuse Implicit Signing Before Attempting a Download.
        let refused = Command::from(process::command(binary))
            .env("DOKIME_AUTHENTICODE_PROVIDER", "none")
            .kill_on_drop(true)
            .args(["installer", "package", "--manifest"])
            .arg(&manifest)
            .arg("--sign")
            .arg("--output")
            .arg(&output)
            .output()
            .await?;
        assert_eq!(refused.status.code(), Some(2));
        assert!(String::from_utf8(refused.stderr)?.contains("--sign"));
        assert!(!output.exists());
        Ok(())
    })
    .await?
}
