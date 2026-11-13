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

pub mod bundles;

mod credentials;
mod settings;
pub mod thermal;

/* === Re-Exports === */

pub use credentials::{CredentialStore, tempdir};
pub use settings::{SYSTEM_HANG_GUARD, settings};

/* === Imports === */

// Standard Library Imports
use std::path::PathBuf;

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Clone, PartialEq, prost::Message)]
pub struct Reading {
    #[prost(double, tag = "1")]
    pub value: f64,
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct Attempt {
    #[prost(double, tag = "1")]
    pub value: f64,

    #[prost(uint32, tag = "2")]
    pub attempt: u32,

    #[prost(bool, tag = "3")]
    pub scope_refused: bool,
}

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

pub fn thermal_fixture() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = std::env::var_os("DOKIME_THERMAL_PLUGIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| fixture_root().join(format!("dokime-rust-thermal{}", std::env::consts::EXE_SUFFIX)));
    if !path.is_file() {
        return Err("Build the Shared Thermal Example Before Running the Host Stories".into());
    }
    Ok(path)
}
pub fn execution_fixture() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let path = fixture_root().join(format!("dokime-rust-execution{}", std::env::consts::EXE_SUFFIX));
    if !path.is_file() {
        return Err("Build the Execution Example with the SDK Task Before Running the Host Stories".into());
    }
    Ok(path)
}
pub fn parameters(value: f64, fail_attempts: u32, evaluate: bool) -> std::collections::HashMap<String, dokime_protocol::v1::Value> {
    use dokime_protocol::v1::{Value, value::Kind};
    std::collections::HashMap::from([
        ("value".into(), Value { kind: Some(Kind::Number(value)) }),
        (
            "fail_attempts".into(),
            Value {
                kind: Some(Kind::UnsignedInteger(fail_attempts as u64)),
            },
        ),
        ("evaluate".into(), Value { kind: Some(Kind::Boolean(evaluate)) }),
    ])
}
pub fn answer(
    host: &dokime_runtime_next::Host, execution: &str, prompt: dokime_protocol::context::PromptsCreateRequest,
) -> dokime_runtime_next::Result<()> {
    host.answer(
        execution,
        dokime_protocol::context::PromptsCreateResponse {
            prompt: prompt.prompt,
            accepted: true,
            action: "continue".into(),
            ..Default::default()
        },
    )
}

pub fn host_fixture() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let executable = std::env::var_os("NEXTEST_BIN_EXE_dokime-host-next")
        .or_else(|| std::env::var_os("CARGO_BIN_EXE_dokime-host-next"))
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
    let path = std::env::var_os(variable).map(PathBuf::from).unwrap_or_else(|| {
        if std::env::var_os("TEST_PROFILE").is_some() {
            let name = if language == "csharp" { "csharp/dokime-system-story" } else { "dokime-system-suite" };
            fixture_root().join(format!("{name}{}", std::env::consts::EXE_SUFFIX))
        } else {
            repository.join(default)
        }
    });
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

pub fn client_settings() -> dokime::client::ClientSettings {
    let bounds = settings();
    dokime::client::ClientSettings {
        transfer: harmos_protocol::storage::StorageTransferSettings { chunk_bytes: Some(1024) },
        retained_bytes: bounds.placement.message_bytes,
        message_bytes: bounds.placement.message_bytes,
        input_items: bounds.placement.stream_window,
        input_bytes: bounds.placement.message_bytes,
    }
}

pub fn content_fixture() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let folder = match std::env::var("TEST_PROFILE") {
        Ok(profile) => repository
            .join("target/build")
            .join(profile)
            .join(format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH))
            .join("fixtures/host"),
        Err(_) => repository.join("target/debug/examples"),
    };
    let executable = std::env::var_os("DOKIME_CONTENT_PLUGIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| folder.join(format!("content_fixture{}", std::env::consts::EXE_SUFFIX)));
    if !executable.is_file() {
        return Err("Build the Content Fixture Through mise run build:host-fixtures".into());
    }
    Ok(executable)
}

/* === Private Functions === */

fn fixture_root() -> PathBuf {
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let profile = std::env::var("TEST_PROFILE").unwrap_or_else(|_| "dev".into());
    let target = if cfg!(windows) { "windows-x86_64" } else { "linux-x86_64" };
    repository.join("target/build").join(profile).join(target).join("fixtures/host")
}

/* === Tests === */
