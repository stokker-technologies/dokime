/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Package an Installer From a Local Kit

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use serde_json::json;
use std::{error::Error, fs, process::Command, time::Duration};

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
async fn author_packages_organization_installer() -> Result<(), Box<dyn Error>> {
    tokio::time::timeout(Duration::from_secs(600), async {
        // Discover Installer Packaging Without Connecting to a Host.
        let binary = env!("CARGO_BIN_EXE_dokime");
        let help = Command::new(binary).args(["installer", "package", "--help"]).output()?;
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
        let refused = Command::new(binary)
            .args(["installer", "package", "--manifest"])
            .arg(&manifest)
            .arg("--artifacts")
            .arg(&kit)
            .arg("--output")
            .arg(&output)
            .output()?;
        assert!(!refused.status.success());
        assert!(!output.exists());
        let error = String::from_utf8(refused.stderr)?;
        assert!(error.contains("required executable, license or organization"), "{error}");
        // Refuse Implicit Signing Before Attempting a Download.
        let refused = Command::new(binary)
            .args(["installer", "package", "--manifest"])
            .arg(&manifest)
            .arg("--sign")
            .arg("--output")
            .arg(&output)
            .output()?;
        assert!(!refused.status.success());
        assert!(String::from_utf8(refused.stderr)?.contains("explicit signing configuration"));
        assert!(!output.exists());
        Ok(())
    })
    .await?
}
