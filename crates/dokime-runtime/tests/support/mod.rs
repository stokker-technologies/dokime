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

// Third Party Imports

// Local Imports
use dokime::Client;
use dokime::ClientSettings;
use dokime::Metadata;
use dokime::Value;
use dokime::client::PlannedCase;
use dokime_protocol::host::PromptsCreateRequest;
use dokime_protocol::host::PromptsCreateResponse;
use dokime_runtime::Host;
use dokime_runtime::Result as HostResult;
use dokime_runtime::services::ClientSettings as ServicesClientSettings;
use harmos_protocol::model::StorageTransferSettings;
use std::collections::HashMap;
use std::env::consts::EXE_SUFFIX;
use std::error::Error;
use std::path::PathBuf;
use tokio::process::Child;
use tokio::process::Command;

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
    child: Child,
    pub url: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl PythonPlugin {
    pub async fn start() -> Result<Self, Box<dyn Error>> {
        use std::process::Stdio;
        use tokio::io::AsyncBufReadExt;
        use tokio::io::BufReader;
        let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../sdk/python/examples/thermal/main.py");
        let mut child = Command::from(dokime_runtime::process::command(python()))
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
    pub async fn close(mut self) -> Result<(), Box<dyn Error>> {
        drop(self.child.stdin.take());
        assert!(self.child.wait().await?.success());
        Ok(())
    }
}

/* === Public Functions === */

pub async fn materialize(
    client: &Client, suite: impl Into<String>, template: impl Into<String>, parameters: HashMap<String, Vec<Value>>,
) -> Result<Vec<PlannedCase>, Box<dyn Error>> {
    use dokime::client::{RegisteredSelection, RegisteredSelectionParametersEntry, SelectionPlan, SelectionValues};

    // Preview the Complete Selection Without Registering Cases
    let parameters = parameters
        .into_iter()
        .map(|(key, values)| RegisteredSelectionParametersEntry {
            key,
            value: Some(SelectionValues { values }),
        })
        .collect();
    let selection = RegisteredSelection {
        suite_id: suite.into(),
        template_id: template.into(),
        parameters,
        ..Default::default()
    };
    let plan = SelectionPlan {
        selections: vec![selection],
        ..Default::default()
    };
    Ok(client.cases.create(plan, true).await?.cases)
}

pub async fn admit(client: &Client, cases: Vec<PlannedCase>) -> Result<Vec<String>, Box<dyn Error>> {
    // Admit Frozen Origins and Read the Executions at the Admission Receipt
    use dokime::client::ExecutionsListOptions;
    use harmos_protocol::model::ReadCut;
    let origins = cases.into_iter().map(|case| case.origin.ok_or("Admit Story Cases: Missing Frozen Origin")).collect::<Result<_, _>>()?;
    let metadata = Metadata {
        label: "Story Cycle".into(),
        ..Default::default()
    };
    let admitted = client.cycles.create(origins, metadata, true, String::new()).await?;
    let parent = admitted.cycle.ok_or("Admit Story Cases: Missing Cycle")?.cycle_id;
    let at = admitted.receipt.and_then(|receipt| receipt.durable_at).map(|position| ReadCut {
        positions: vec![position],
        revisions: Vec::new(),
    });
    let options = ExecutionsListOptions { at, ..Default::default() };
    let executions = client.executions.list_with(parent, Default::default(), Default::default(), Default::default(), options).await?;
    executions
        .executions
        .into_iter()
        .map(|execution| Ok(execution.summary.ok_or("Admit Story Cases: Missing Execution Summary")?.execution_id))
        .collect()
}

pub fn thermal_fixture() -> Result<PathBuf, Box<dyn Error>> {
    let path = std::env::var_os("DOKIME_THERMAL_PLUGIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| fixture_root().join(format!("dokime-rust-thermal{}", EXE_SUFFIX)));
    if !path.is_file() {
        return Err("Build the Shared Thermal Example Before Running the Host Stories".into());
    }
    Ok(path)
}
pub fn execution_fixture() -> Result<PathBuf, Box<dyn Error>> {
    let path = fixture_root().join(format!("dokime-rust-execution{}", EXE_SUFFIX));
    if !path.is_file() {
        return Err("Build the Execution Example with the SDK Task Before Running the Host Stories".into());
    }
    Ok(path)
}
pub fn secrets_fixture() -> Result<PathBuf, Box<dyn Error>> {
    let path = fixture_root().join(format!("dokime-rust-secrets{}", EXE_SUFFIX));
    if !path.is_file() {
        return Err("Build the Secret Fixture Through mise run build:host-fixtures".into());
    }
    Ok(path)
}
pub fn parameters(value: f64, fail_attempts: u32, evaluate: bool) -> HashMap<String, Value> {
    use dokime::Value;
    use dokime_protocol::model::value::Kind;
    HashMap::from([
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
pub fn answer(host: &Host, execution: &str, prompt: PromptsCreateRequest) -> HostResult<()> {
    host.answer(
        execution,
        PromptsCreateResponse {
            prompt: prompt.prompt,
            accepted: true,
            action: "continue".into(),
            ..Default::default()
        },
    )
}

pub fn host_fixture() -> Result<PathBuf, Box<dyn Error>> {
    let executable = std::env::var_os("NEXTEST_BIN_EXE_dokime-host")
        .or_else(|| std::env::var_os("CARGO_BIN_EXE_dokime-host"))
        .ok_or("Build the New Host Binary Before Running Its Client Stories")?;
    Ok(PathBuf::from(executable))
}

pub fn language_fixture(language: &str) -> Result<PathBuf, Box<dyn Error>> {
    let variable = format!("DOKIME_{}_THERMAL_PLUGIN", language.to_uppercase());
    let name = match language {
        "csharp" => "csharp/dokime-system-story",
        "cpp" => "dokime-system-suite",
        _ => return Err("Select a Compiled SDK Fixture".into()),
    };
    let path = std::env::var_os(variable).map(PathBuf::from).unwrap_or_else(|| fixture_root().join(format!("{name}{}", EXE_SUFFIX)));
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

pub fn client_settings() -> ClientSettings {
    let bounds = settings();
    ClientSettings {
        transfer: StorageTransferSettings { chunk_bytes: Some(1024) },
        retained_bytes: bounds.placement.message_bytes,
        message_bytes: bounds.placement.message_bytes,
        input_items: bounds.placement.stream_window,
        input_bytes: bounds.placement.message_bytes,
    }
}

pub fn service_settings() -> ServicesClientSettings {
    let bounds = settings();
    ServicesClientSettings {
        read_timeout: bounds.placement.connect_timeout,
        mutation_timeout: bounds.placement.connect_timeout,
        watch_timeout: SYSTEM_HANG_GUARD,
        max_cases: 1000,
        page_items: 1000,
        message_bytes: bounds.placement.message_bytes.get(),
        stream_buffer: bounds.placement.stream_window.get(),
    }
}

pub fn content_fixture() -> Result<PathBuf, Box<dyn Error>> {
    let executable = std::env::var_os("DOKIME_CONTENT_PLUGIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| fixture_root().join(format!("content_fixture{}", EXE_SUFFIX)));
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
