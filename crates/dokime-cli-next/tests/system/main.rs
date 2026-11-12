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

mod first_hour;
mod remote_first_hour;
mod scaffold;

/* === Re-Exports === */

/* === Imports === */

use std::path::PathBuf;

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
    root().join("target/build/dev").join(native).join("fixtures/cli")
}
fn command() -> tokio::process::Command {
    let executable = std::env::var_os("NEXTEST_BIN_EXE_dokime-next")
        .or_else(|| std::env::var_os("CARGO_BIN_EXE_dokime-next"))
        .unwrap_or_else(|| env!("CARGO_BIN_EXE_dokime-next").into());
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
