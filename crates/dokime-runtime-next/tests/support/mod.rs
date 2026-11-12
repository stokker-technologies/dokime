/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execution Host

/* === Modules === */

pub mod thermal;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{num::NonZeroUsize, path::PathBuf, sync::Arc, time::Duration};

// Third Party Imports

// Local Imports
use dokime_runtime_next::{ControlledClock, HostSettings, PlacementSettings};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct PythonPlugin {
    child: tokio::process::Child,
    pub url: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl PythonPlugin {
    pub async fn start() -> Result<Self, Box<dyn std::error::Error>> {
        use std::process::Stdio;
        use tokio::io::{AsyncBufReadExt, BufReader};
        let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../sdk/python/examples/thermal/main.py");
        let mut child = tokio::process::Command::from(dokime_runtime_next::process::command(python()))
            .arg(script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()?;
        let mut lines = BufReader::new(child.stdout.take().unwrap()).lines();
        let url = lines.next_line().await?.ok_or("Start Python Thermal Plugin: No Readiness Event")?;
        Ok(Self { child, url })
    }
    pub async fn close(mut self) -> Result<(), Box<dyn std::error::Error>> {
        drop(self.child.stdin.take());
        assert!(self.child.wait().await?.success());
        Ok(())
    }
}

/* === Public Functions === */

pub fn settings() -> HostSettings {
    HostSettings {
        placement: PlacementSettings {
            connect_timeout: Duration::from_secs(30),
            reconnect_delay: Duration::from_secs(1),
            stop_grace: Duration::from_secs(30),
            stream_window: NonZeroUsize::new(8).unwrap(),
            message_bytes: NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        },
        clock: Arc::new(ControlledClock::new(0)),
    }
}
pub fn thermal_fixture() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = std::env::var_os("DOKIME_THERMAL_PLUGIN").map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/debug")
            .join(format!("dokime-rust-thermal{}", std::env::consts::EXE_SUFFIX))
    });
    if !path.is_file() {
        return Err("Build the Shared Thermal Example Before Running the Host Stories".into());
    }
    Ok(path)
}

pub fn host_fixture() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let executable = std::env::var_os("NEXTEST_BIN_EXE_dokime-host-next")
        .or_else(|| option_env!("CARGO_BIN_EXE_dokime-host-next").map(Into::into))
        .ok_or("Build the New Host Binary Before Running Its Client Stories")?;
    Ok(PathBuf::from(executable))
}

pub fn language_fixture(language: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let variable = format!("DOKIME_{}_THERMAL_PLUGIN", language.to_uppercase());
    let default = match language {
        "csharp" if cfg!(windows) => "sdk/csharp/tests/SystemPlugin/bin/Release/net10.0/dokime-system-story.exe",
        "csharp" => "sdk/csharp/tests/SystemPlugin/bin/Release/net10.0/dokime-system-story",
        "cpp" if cfg!(windows) => "target/sdk/cpp/tests/Debug/dokime-system-suite.exe",
        "cpp" => "target/sdk/cpp/tests/dokime-system-suite",
        _ => return Err("Select a Compiled SDK Fixture".into()),
    };
    let path = std::env::var_os(variable).map(PathBuf::from).unwrap_or_else(|| repository.join(default));
    if !path.is_file() {
        return Err(format!("Build the {language} Thermal Fixture Through Its SDK Mise Task: {}", path.display()).into());
    }
    Ok(path)
}

pub fn python() -> PathBuf {
    std::env::var_os("DOKIME_PYTHON").map(PathBuf::from).unwrap_or_else(|| {
        let path = if cfg!(windows) { ".venv/Scripts/python.exe" } else { ".venv/bin/python" };
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(path)
    })
}

/* === Private Functions === */

/* === Tests === */
