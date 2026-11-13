/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Command-Line System Stories

/* === Modules === */

#[path = "../../../dokime-runtime-next/tests/support/credentials.rs"]
mod credentials;
mod first_hour;
mod remote_first_hour;
mod scaffold;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::path::PathBuf;

// Third Party Imports

// Local Imports

/* === Constants === */

const HANG_GUARD: std::time::Duration = std::time::Duration::from_secs(600);

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn fixtures() -> PathBuf {
    let native = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
    let profile = std::env::var("TEST_PROFILE").unwrap_or_else(|_| "dev".into());
    root().join("target/build").join(profile).join(native).join("fixtures/cli")
}

fn command() -> tokio::process::Command {
    let executable = std::env::var_os("NEXTEST_BIN_EXE_dokime-next")
        .or_else(|| std::env::var_os("CARGO_BIN_EXE_dokime-next"))
        .map(PathBuf::from)
        .unwrap_or_else(|| root().join("target/debug").join(format!("dokime-next{}", std::env::consts::EXE_SUFFIX)));
    let mut command = tokio::process::Command::from(dokime_runtime_next::process::command(executable));
    command.kill_on_drop(true);
    command
}

async fn run(args: &[&str]) -> Result<String, Box<dyn std::error::Error>> {
    let output = command().args(args).output().await?;
    if !output.status.success() {
        return Err(format!("Run dokime {args:?}: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    Ok(String::from_utf8(output.stdout)?)
}

/* === Tests === */
