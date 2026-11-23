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

mod settings;
pub mod thermal;

/* === Re-Exports === */

pub use dokime_test_support::credentials::{CredentialStore, namespace, run, tempdir};
pub use settings::{OPERATION_TIMEOUT, SYSTEM_HANG_GUARD, settings};

/* === Imports === */

// Standard Library Imports
use std::collections::HashMap;
use std::env::{
    self,
    consts::{EXE_SUFFIX, OS},
};
use std::error::Error;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::process::Stdio;

// Third Party Imports
use dokime_protocol::host::{PluginEntry, PluginState, ReportState};
use harmos_protocol::model::StorageTransferSettings;
use harmos_runtime::plugin::grpc::Launches;
use sha2::{Digest, Sha256};
use tokio::process::Child;
use tokio::process::Command;

// Local Imports
use dokime::Value;
use dokime::{FileMetadata, FileReference, Metadata};
use dokime_client_internal::{
    Client, ClientSettings, PlannedCase, Plugins, PluginsWatchOptions, RegisteredSelection, RegisteredSelectionParametersEntry,
    SelectionPlan, SelectionValues,
};
use dokime_protocol::host::PromptsCreateRequest;
use dokime_protocol::host::PromptsCreateResponse;
use dokime_protocol::model::value::Kind;
use dokime_runtime::Host;
use dokime_runtime::{process, services};

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
        let script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../sdk/tests/fixtures/python/thermal/plugin.py");
        let launches = Launches::new();
        let mut launch = launches.reserve("python thermal fixture").await?;
        let child = Command::from(process::command(python()))
            .arg(script)
            .envs(launch.environment())
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()?;
        let address = launch.announced().await?;
        launches.shutdown().await;
        let url = format!("http://{address}");
        Ok(Self { child, url })
    }
    pub async fn close(mut self) -> Result<(), Box<dyn Error>> {
        self.child.kill().await?;
        self.child.wait().await?;
        Ok(())
    }
}

/* === Public Functions === */

pub async fn plugin_state(plugins: &Plugins, mut entry: PluginEntry, expected: PluginState) -> Result<PluginEntry, Box<dyn Error>> {
    let mut watching = plugins
        .watch_with(PluginsWatchOptions {
            name: Some(entry.name.clone()),
            ..Default::default()
        })
        .await?;
    loop {
        if entry.state == expected as i32 {
            return Ok(entry);
        }
        if entry.failure.is_some() && expected == PluginState::Ready {
            return Err(format!("Await Plugin {} Ready: {:?}", entry.name, entry.failure).into());
        }
        watching.next().await?.ok_or("Await Plugin State: Watch Closed Before the Requested Condition")?;
        entry = plugins.get(entry.name).await?.entry.ok_or("Await Plugin State: Entry Removed")?;
    }
}

/// Await the Report a Seal Freezes in the Background.
pub async fn report_frozen(client: &Client, execution: &str) -> Result<(), Box<dyn Error>> {
    let mut watching = client.executions.watch("", Default::default(), Default::default(), Default::default()).await?;
    while let Some(frame) = watching.next().await? {
        if frame.executions.iter().any(|detail| {
            detail.summary.as_ref().is_some_and(|summary| summary.execution_id == execution)
                && detail.report.as_ref().is_some_and(|report| report.state == ReportState::Ready as i32)
        }) {
            return Ok(());
        }
    }
    Err("Await Frozen Report: Watch Closed Before the Report Was Ready".into())
}

pub async fn materialize(
    client: &Client, suite: impl Into<String>, template: impl Into<String>, parameters: HashMap<String, Vec<Value>>,
) -> Result<Vec<PlannedCase>, Box<dyn Error>> {
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
    // Admit Frozen Origins and Read the Executions the Durable Admission Created
    let origins = cases.into_iter().map(|case| case.origin.ok_or("Admit Story Cases: Missing Frozen Origin")).collect::<Result<_, _>>()?;
    let metadata = Metadata {
        label: "Story Cycle".into(),
        ..Default::default()
    };
    let admitted = client.cycles.create(origins, metadata, true, String::new()).await?;
    let parent = admitted.cycle.ok_or("Admit Story Cases: Missing Cycle")?.cycle_id;
    let executions = client.executions.list(parent, Default::default(), Default::default(), Default::default()).await?;
    executions
        .executions
        .into_iter()
        .map(|execution| Ok(execution.summary.ok_or("Admit Story Cases: Missing Execution Summary")?.execution_id))
        .collect()
}

pub fn thermal_fixture() -> Result<PathBuf, Box<dyn Error>> {
    let path = env::var_os("DOKIME_THERMAL_PLUGIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| fixture_root().join(format!("dokime-rust-thermal{}", EXE_SUFFIX)));
    if !path.is_file() {
        return Err("Build the Shared Thermal Example Before Running the Host Stories".into());
    }
    Ok(path)
}
pub fn obc_bundle() -> Result<PathBuf, Box<dyn Error>> {
    let path = fixture_root().with_file_name("desktop").join("plugin-obc.dokime");
    if !path.is_file() {
        return Err("Bundle the OBC Plugin Through mise run build:desktop-fixtures".into());
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
pub fn ping_fixture() -> Result<PathBuf, Box<dyn Error>> {
    let path = fixture_root().join(format!("dokime-rust-plugin{EXE_SUFFIX}"));
    if !path.is_file() {
        return Err("Build the Ping Fixture Through mise run build:host-fixtures -- --language rust --fixture dokime-rust-plugin".into());
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
pub fn capture_fixture() -> Result<PathBuf, Box<dyn Error>> {
    let path = fixture_root().join(format!("dokime-rust-capture{}", EXE_SUFFIX));
    if !path.is_file() {
        return Err("Build the Capture Fixture Through mise run build:host-fixtures".into());
    }
    Ok(path)
}
pub fn boundary_fixture() -> Result<PathBuf, Box<dyn Error>> {
    let path = fixture_root().join(format!("dokime-rust-boundary{}", EXE_SUFFIX));
    if !path.is_file() {
        return Err("Build the Boundary Fixture Through mise run build:host-fixtures".into());
    }
    Ok(path)
}
pub fn parameters(value: f64, fail_attempts: u32, evaluate: bool) -> HashMap<String, Value> {
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
pub fn answer(host: &Host, execution: &str, prompt: PromptsCreateRequest) -> dokime_runtime::Result<()> {
    host.answer(
        execution,
        PromptsCreateResponse {
            prompt: prompt.prompt,
            accepted: true,
            action: "0".into(),
            ..Default::default()
        },
    )
}

pub fn host_fixture() -> Result<PathBuf, Box<dyn Error>> {
    let executable = env::var_os("NEXTEST_BIN_EXE_dokime-host")
        .or_else(|| env::var_os("CARGO_BIN_EXE_dokime-host"))
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
    let path = env::var_os(variable).map(PathBuf::from).unwrap_or_else(|| fixture_root().join(format!("{name}{}", EXE_SUFFIX)));
    if !path.is_file() {
        return Err(format!("Build the {language} Thermal Fixture Through Its SDK Mise Task: {}", path.display()).into());
    }
    Ok(path)
}

pub fn python() -> PathBuf {
    env::var_os("DOKIME_PYTHON").map(PathBuf::from).unwrap_or_else(|| {
        let path = if OS == "windows" { ".venv/Scripts/python.exe" } else { ".venv/bin/python" };
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").join(path)
    })
}

pub fn client_settings() -> ClientSettings {
    let bounds = settings();
    ClientSettings {
        transfer: StorageTransferSettings { chunk_bytes: Some(1024) },
        // Decoded Replies Outgrow Their Wire Bytes, So Retain as Much as the Shipped Clients.
        retained_bytes: NonZeroUsize::new(64 * 1024 * 1024).unwrap(),
        message_bytes: bounds.placement.message_bytes,
        input_items: bounds.placement.stream_window,
        input_bytes: bounds.placement.message_bytes,
    }
}

pub fn service_settings() -> services::ClientSettings {
    let bounds = settings();
    services::ClientSettings {
        read_timeout: OPERATION_TIMEOUT,
        mutation_timeout: OPERATION_TIMEOUT,
        watch_timeout: SYSTEM_HANG_GUARD,
        max_cases: 1000,
        page_items: 1000,
        message_bytes: bounds.placement.message_bytes.get(),
        stream_buffer: bounds.placement.stream_window.get(),
    }
}

pub fn content_fixture() -> Result<PathBuf, Box<dyn Error>> {
    let executable = env::var_os("DOKIME_CONTENT_PLUGIN")
        .map(PathBuf::from)
        .unwrap_or_else(|| fixture_root().join(format!("content_fixture{}", EXE_SUFFIX)));
    if !executable.is_file() {
        return Err("Build the Content Fixture Through mise run build:host-fixtures".into());
    }
    Ok(executable)
}

pub async fn native_file(client: &Client, id: &str, execution: &str, bytes: &[u8]) -> Result<FileReference, Box<dyn Error>> {
    let created = client
        .files
        .uploads
        .create(FileMetadata {
            id: id.into(),
            scope: if execution.is_empty() { None } else { Some(format!("execution/{execution}")) },
            media_type: "application/octet-stream".into(),
            expected_length: Some(bytes.len() as u64),
            expected_sha256: Some(Sha256::digest(bytes).to_vec()),
        })
        .await?;
    let upload = created.upload.ok_or("Create Story File: Missing Upload")?;
    for (index, chunk) in bytes.chunks(created.chunk_bytes as usize).enumerate() {
        client.files.uploads.append(upload.clone(), (index * created.chunk_bytes as usize) as u64, chunk.to_vec()).await?;
    }
    Ok(client
        .files
        .uploads
        .publish(upload, bytes.len() as u64, Sha256::digest(bytes).to_vec())
        .await?
        .upload
        .ok_or("Publish Story File: Missing Reference")?)
}

/* === Private Functions === */

fn fixture_root() -> PathBuf {
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let profile = env::var("TEST_PROFILE").unwrap_or_else(|_| "dev".into());
    let target = if OS == "windows" { "windows-x86_64" } else { "linux-x86_64" };
    repository.join("target/build").join(profile).join(target).join("fixtures/host")
}

/* === Tests === */
