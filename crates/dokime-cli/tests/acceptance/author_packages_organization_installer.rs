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
use std::{env, error::Error, fs, path::PathBuf, time::Duration};

// Third Party Imports
use serde_json::json;
use tokio::{process::Command, time};

// Local Imports
use dokime_packaging::bundle;
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
    time::timeout(Duration::from_secs(600), async {
        // Discover Installer Packaging Without Connecting to a Host.
        let binary =
            env::var_os("NEXTEST_BIN_EXE_dokime").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_dokime")));
        let help = Command::from(process::command(&binary))
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
            serde_json::to_vec(&json!({"schema":1,"version":"1.2.3","target":bundle::native_target(),"source_revision":"f7c50e68","channel":"nightly","plugins":[],"files":{}}))?,
        )?;
        let refused = Command::from(process::command(&binary))
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
        assert!(refused.stdout.is_empty());
        assert!(!refused.stderr.contains(&0x1b));
        assert!(!output.exists());
        assert!(!refused.stderr.is_empty());
        // Refuse Implicit Signing Before Attempting a Download.
        let refused = Command::from(process::command(&binary))
            .env("DOKIME_AUTHENTICODE_PROVIDER", "none")
            .kill_on_drop(true)
            .args(["installer", "package", "--manifest"])
            .arg(&manifest)
            .arg("--sign")
            .arg("--json")
            .arg("--output")
            .arg(&output)
            .output()
            .await?;
        assert_eq!(refused.status.code(), Some(2));
        assert!(refused.stdout.is_empty());
        assert!(!refused.stderr.is_empty());
        assert!(!output.exists());
        Ok(())
    })
    .await?
}
