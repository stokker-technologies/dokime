/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Execute Remote Examples Against The Built OBC Plugin

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{collections::BTreeMap, io::Read, path::Path};

// Third Party Imports
use serde_json::Value;
use sha2::{Digest, Sha256};
use tokio::process::Command;

// Local Imports
use crate::support::artifacts;
use dokime_client::{Client, dynamic::Event};
use dokime_domain::model::{
    execution::result::CyclePhase,
    identity::{PromptId, TestCycleId},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub async fn scripts(client: &Client, endpoint: &str, root: &Path) {
    std::fs::create_dir_all(root).unwrap();
    let extensions: &[&str] = if cfg!(windows) { &["ps1", "bat", "py"] } else { &["sh", "py"] };
    let workflows = extensions.iter().flat_map(|extension| ["device", "test", "prompt"].map(|workflow| (*extension, workflow)));
    for (extension, workflow) in workflows {
        let directory = root.join(format!("{extension} {workflow}"));
        let stdout = execute(extension, workflow, endpoint, &directory).await;
        if workflow == "device" {
            device(client, extension, &stdout, &directory).await;
            continue;
        }
        evidence(client, extension, workflow, &stdout, &directory).await;
    }
}

/* === Private Functions === */
async fn execute(extension: &str, workflow: &str, endpoint: &str, directory: &Path) -> String {
    let workspace = artifacts::workspace_root();
    let family = if extension == "py" { "python" } else { "cli" };
    let script = workspace.join(format!("packages/dokime/examples/remote/{family}/control-{workflow}.{extension}"));
    let binary = artifacts::host_binary().with_file_name(if cfg!(windows) { "dokime.exe" } else { "dokime" });
    assert!(binary.is_file(), "the Build archive must contain {}", binary.display());
    let mut command = launcher(extension, &script);
    let mut paths = vec![binary.parent().unwrap().to_owned()];
    let inherited = std::env::var_os("PATH").unwrap_or_default();
    paths.extend(std::env::split_paths(&inherited));

    command
        .args(["--endpoint", endpoint])
        .env("PATH", std::env::join_paths(paths).unwrap())
        .env("DOKIME_EXECUTABLE", binary)
        .env("DOKIME_OUTPUT_DIRECTORY", directory)
        .env("DOKIME_RESPONSE_DIRECTORY", directory.join("responses"))
        .current_dir(directory.parent().unwrap())
        .kill_on_drop(true);
    for name in ["DOKIME_KEY", "DOKIME_KEY_FILE", "DOKIME_ENV_FILE", "DOKIME_CA_CERTIFICATE", "DOKIME_BACKEND"] {
        command.env_remove(name);
    }
    if extension == "py" {
        let flag = if workflow == "device" { "--response-directory" } else { "--directory" };
        let destination = if workflow == "device" { directory.join("responses") } else { directory.to_owned() };
        command.arg(flag).arg(destination);
    }

    let output = command.output().await.unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{} failed with {}\n{stdout}\n{stderr}", script.display(), output.status);
    stdout
}

fn launcher(extension: &str, script: &Path) -> Command {
    match extension {
        "py" => {
            let interpreter = if cfg!(windows) { ".venv/Scripts/python.exe" } else { ".venv/bin/python" };
            let executable = artifacts::workspace_root().join(interpreter);
            let mut command = Command::new(executable);
            command.arg(script);
            command.env_remove("PYTHONPATH");
            command
        }
        "sh" => {
            let mut command = Command::new("sh");
            command.arg(script);
            command
        }
        "ps1" => {
            let mut command = Command::new("pwsh");
            command.args(["-NoProfile", "-File"]).arg(script);
            command
        }
        "bat" => Command::new(script),
        _ => unreachable!(),
    }
}

async fn device(client: &Client, extension: &str, stdout: &str, directory: &Path) {
    device_responses(extension, stdout, directory);
    let response = client.call_operation("obc.device.read", None, b"{}", Vec::new()).await.unwrap();
    let Some(Event::Completion(state)) = response.message().await.unwrap() else { panic!("device read has one completion") };
    device_state(&state, false);
    assert!(response.message().await.unwrap().is_none());
}

fn device_responses(extension: &str, stdout: &str, directory: &Path) {
    if extension == "py" {
        for (prefix, enabled) in [("Power On: ", true), ("Power Off: ", false), ("After: ", false)] {
            let text = stdout.lines().find_map(|line| line.strip_prefix(prefix)).unwrap();
            let state = serde_json::from_str(text).unwrap();
            device_state(&state, enabled);
        }
        for (name, enabled) in [("power-on", true), ("running", true), ("power-off", false), ("after", false)] {
            let path = directory.join(format!("responses/{name}.json"));
            let bytes = std::fs::read(path).unwrap();
            let state = serde_json::from_slice(&bytes).unwrap();
            device_state(&state, enabled);
        }
        return;
    }
    for (name, enabled) in [("power-on", true), ("running", true), ("power-off", false), ("after", false)] {
        let path = directory.join(format!("responses/{name}.json"));
        let bytes = std::fs::read(path).unwrap();
        let events: Vec<Value> = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["event"], "completion");
        device_state(&events[0]["value"], enabled);
    }
}

fn device_state(state: &Value, enabled: bool) {
    let fields = state.as_object().unwrap();
    let powered = fields.get("enabled").map(|value| value.as_bool().unwrap()).unwrap_or(false);
    assert_eq!(powered, enabled);
    let field = state.get("voltage_rms_v").or_else(|| state.get("voltageRmsV"));
    let voltage = field.map(|value| value.as_f64().unwrap()).unwrap_or(0.0);
    assert_eq!(voltage, if enabled { 230.0 } else { 0.0 });
}

async fn evidence(client: &Client, extension: &str, workflow: &str, stdout: &str, directory: &Path) {
    let cycles = identities(stdout, "Cycle: ");
    assert_eq!(cycles.len(), 1);
    let cycle_id = &cycles[0];
    let id = TestCycleId::from_uuid(cycle_id).unwrap();
    let cycle = client.cycle(id);
    let snapshot = cycle.snapshot().await.unwrap();
    assert_eq!(snapshot.phase, CyclePhase::Sealed);
    assert!(snapshot.sealed);

    let mut executions = identities(stdout, "Execution: ");
    let expected = if workflow == "test" { 2 } else { 1 };
    assert_eq!(executions.len(), expected);
    let members = cycle.executions().await.unwrap();
    let mut actual: Vec<_> = members.iter().map(|member| member.id().to_string()).collect();
    actual.sort();
    executions.sort();
    assert_eq!(actual, executions);

    let path = directory.join("cycle.zip");
    let samples = archive(&path, cycle_id, &executions, true);
    for (index, id) in identities(stdout, "Execution: ").iter().enumerate() {
        let name = if workflow == "test" { format!("execution-{}.zip", index + 1) } else { "execution.zip".to_owned() };
        let path = directory.join(name);
        let selected = std::slice::from_ref(id);
        let captured = archive(&path, cycle_id, selected, false);
        assert_eq!(captured[id], samples[id]);
    }

    if workflow == "prompt" {
        let prompts = identities(stdout, "Prompt: ");
        assert_eq!(prompts.len(), 1);
        let id = PromptId::from_uuid(&prompts[0]).unwrap();
        let prompt = members[0].prompt(&id).await.unwrap();
        assert_eq!(prompt.response.unwrap().action, "yes");
        return;
    }
    if extension == "py" {
        drained(stdout, expected, &samples);
    }
}

fn drained(stdout: &str, expected: usize, samples: &BTreeMap<String, u64>) {
    let summaries: Vec<Value> =
        stdout.lines().filter_map(|line| line.strip_prefix("Drained Evidence: ")).map(|text| serde_json::from_str(text).unwrap()).collect();
    assert_eq!(summaries.len(), expected);
    for summary in summaries {
        let id = summary["execution_id"].as_str().unwrap();
        assert_eq!(summary["status"], "passed");
        assert_eq!(summary["samples_by_stream"]["obc.pulse"].as_u64().unwrap(), samples[id]);
        assert_eq!(summary["records_by_type"]["obc.run.started"], 1);
        assert_eq!(summary["records_by_type"]["obc.run.completed"], 1);
    }
}

fn identities(stdout: &str, prefix: &str) -> Vec<String> {
    stdout
        .lines()
        .filter_map(|line| line.strip_prefix(prefix))
        .map(|line| {
            let id = line.split_whitespace().next().unwrap();
            let uuid = uuid::Uuid::parse_str(id).unwrap();
            uuid.to_string()
        })
        .collect()
}

fn archive(path: &Path, cycle: &str, executions: &[String], sealed: bool) -> BTreeMap<String, u64> {
    let file = std::fs::File::open(path).unwrap();
    let mut zip = zip::ZipArchive::new(file).unwrap();
    let entry = zip.by_name("manifest.json").unwrap();
    let manifest: Value = serde_json::from_reader(entry).unwrap();
    assert_eq!(manifest["schema_version"], 1);
    assert_eq!(manifest["complete"], true);
    assert_eq!(manifest["cycle_id"], cycle);
    assert_eq!(manifest["executions"].as_array().unwrap().len(), executions.len());
    assert!(!manifest.to_string().contains("PRIVATE KEY"));

    for payload in manifest["payloads"].as_array().unwrap() {
        let mut entry = zip.by_name(payload["path"].as_str().unwrap()).unwrap();
        assert_eq!(entry.size(), payload["size_bytes"].as_u64().unwrap());
        let actual = checksum(&mut entry);
        assert_eq!(actual, payload["sha256"].as_str().unwrap());
    }

    let root = tempfile::tempdir().unwrap();
    let database_path = root.path().join("evidence.sqlite");
    let mut output = std::fs::File::create_new(&database_path).unwrap();
    let mut entry = zip.by_name("evidence.sqlite").unwrap();
    std::io::copy(&mut entry, &mut output).unwrap();
    drop(output);
    let database = rusqlite::Connection::open_with_flags(&database_path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let phase: String = database.query_row("SELECT phase FROM cycles WHERE cycle_id=?1", [cycle], |row| row.get(0)).unwrap();
    assert_eq!(phase, if sealed { "sealed" } else { "active" });

    let mut statement = database.prepare("SELECT execution_id FROM executions ORDER BY execution_id").unwrap();
    let actual: Vec<String> = statement.query_map([], |row| row.get(0)).unwrap().map(Result::unwrap).collect();
    assert_eq!(actual, executions);

    executions.iter().map(|id| (id.clone(), captured(&database, cycle, id))).collect()
}

fn captured(database: &rusqlite::Connection, cycle: &str, id: &str) -> u64 {
    let status: String =
        database.query_row("SELECT status FROM executions WHERE execution_id=?1 AND cycle_id=?2", [id, cycle], |row| row.get(0)).unwrap();
    assert_eq!(status, "passed");

    let (table, count): (String, i64) = database
        .query_row("SELECT table_name,sample_count FROM streams WHERE execution_id=?1 AND stream_id='obc.pulse'", [id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .unwrap();
    assert!(count > 0);
    assert!(table.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'));
    let query = format!("SELECT count(*) FROM {table}");
    let physical: i64 = database.query_row(&query, [], |row| row.get(0)).unwrap();
    assert_eq!(physical, count);
    for kind in ["obc.run.started", "obc.run.completed"] {
        let records: i64 = database
            .query_row("SELECT count(*) FROM records WHERE execution_id=?1 AND record_type=?2", [id, kind], |row| row.get(0))
            .unwrap();
        assert_eq!(records, 1);
    }
    u64::try_from(count).unwrap()
}

fn checksum(reader: &mut impl Read) -> String {
    let mut digest = Sha256::new();
    let mut bytes = [0_u8; 65_536];
    loop {
        let read = reader.read(&mut bytes).unwrap();
        if read == 0 {
            break;
        }
        digest.update(&bytes[..read]);
    }
    digest.finalize().iter().map(|byte| format!("{byte:02x}")).collect()
}

/* === Tests === */
