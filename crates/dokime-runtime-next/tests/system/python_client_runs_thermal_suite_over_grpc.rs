/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Python Client Runs a Thermal Suite over gRPC

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use super::client_runs_thermal_suite_over_grpc::HostProcess;
use dokime_runtime_next::PluginProcess;
use std::path::PathBuf;
use tokio::process::Command;

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
async fn python_client_runs_thermal_suite_over_grpc() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::support::SYSTEM_HANG_GUARD, async {
        // Prepare the Installed Pure SDK and Launch the Real Plugin and Host
        let root = tempfile::tempdir()?;
        let credential_root = tempfile::tempdir()?;
        let credentials = super::support::CredentialStore::start(credential_root.path()).await?;
        let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let python = std::env::var_os("DOKIME_PYTHON").map(PathBuf::from).unwrap_or_else(|| {
            let directory = if std::env::consts::EXE_SUFFIX.is_empty() { ".venv/bin" } else { ".venv/Scripts" };
            repository.join(directory).join(format!("python{}", std::env::consts::EXE_SUFFIX))
        });
        let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/system/client_thermal.py");
        let expected = root.path().join("expected.bin");
        let settings = super::support::settings();
        let executable = super::support::thermal_fixture()?;
        let mut plugin = PluginProcess::start(&executable, true, &settings.placement).await?;
        let process = HostProcess::start_with_environment(&root.path().join("store"), &credentials.environment).await?;

        // Drive the Complete Journey with the Pure Python Package's Generated Clients
        let output = Command::from(dokime_runtime_next::process::command(&python))
            .arg(&script)
            .args(["run", &process.url, plugin.endpoint()])
            .arg(&expected)
            .kill_on_drop(true)
            .output()
            .await?;
        assert!(output.status.success(), "{}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        process.close().await?;

        // Reconnect After a Real Process Restart and Compare All Stored Evidence
        let process = HostProcess::start_with_environment(&root.path().join("store"), &credentials.environment).await?;
        let output = Command::from(dokime_runtime_next::process::command(&python))
            .arg(&script)
            .args(["read", &process.url, plugin.endpoint()])
            .arg(&expected)
            .kill_on_drop(true)
            .output()
            .await?;
        assert!(output.status.success(), "{}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        process.close().await?;
        plugin.close().await?;
        Ok(())
    })
    .await?
}
