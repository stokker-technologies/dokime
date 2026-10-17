/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */
// Standard Library Imports
use std::path::Path;
// Third Party Imports
use serde_json::Value;
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
fn hub_log(root: &Path) -> std::path::PathBuf {
    root.join("logs").join("dokime.jsonl")
}

/* === Tests === */
#[test]
fn serve_writes_records() {
    let root = tempfile::tempdir().unwrap();

    if !isolated("serve_writes_records", None) {
        return;
    }

    let guards = dokime_server::host::install_logging(root.path(), false).unwrap();
    tracing::info!(target: "dokime_runtime::hub", phase = "storage", "backend reached storage");
    tracing::warn!(target: "dokime_runtime::hub", attempts = 3, "plugin handshake retried");
    // Flushes the non-blocking worker's queue; the file is only whole after.
    drop(guards);

    let contents = std::fs::read_to_string(hub_log(root.path())).unwrap();
    let records: Vec<Value> = contents
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str(line).expect("every hub log line is one JSON object"))
        .collect();
    assert_eq!(records.len(), 2, "{contents}");

    let expected = [
        ("INFO", "backend reached storage", "phase", "\"storage\""),
        ("WARN", "plugin handshake retried", "attempts", "3"),
    ];
    for (record, (level, message, field, value)) in records.iter().zip(expected) {
        assert_eq!(record["level"], level, "{record}");
        assert_eq!(record["target"], "dokime_runtime::hub", "{record}");
        // Nested, not flattened: this is where the reader looks.
        assert_eq!(record["fields"]["message"], message, "{record}");
        assert_eq!(record["fields"][field].to_string(), value, "{record}");
        // The reader's own parser, so a shape it would reject fails here.
        let timestamp = record["timestamp"].as_str().expect("an RFC 3339 timestamp");
        time::OffsetDateTime::parse(timestamp, &time::format_description::well_known::Rfc3339)
            .unwrap_or_else(|error| panic!("{timestamp} is not RFC 3339: {error}"));
    }
}

// Configure global subscribers and environment only in a child process.
fn isolated(name: &str, profile: Option<&str>) -> bool {
    if std::env::var("DOKIME_LOG_TEST").as_deref() == Ok(name) {
        return true;
    }
    let test = format!("{}::{name}", module_path!().split_once("::").unwrap().1);
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", &test, "--nocapture"])
        .env("DOKIME_LOG_TEST", name)
        .env("RUST_LOG", "info")
        .env_remove("DOKIME_LOG_FORMAT")
        .env_remove("DOKIME_DEV_PROFILE");
    if let Some(profile) = profile {
        command.env("DOKIME_DEV_PROFILE", profile);
    }
    let output = command.output().unwrap();
    assert!(
        output.status.success() && String::from_utf8_lossy(&output.stdout).contains("1 passed"),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    false
}

#[test]
fn reopened_log_rotates() {
    if !isolated("reopened_log_rotates", None) {
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let path = hub_log(root.path());
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let file = std::fs::File::create(&path).unwrap();
    file.set_len(32 * 1024 * 1024).unwrap();
    drop(file);
    let guards = dokime_server::host::install_logging(root.path(), false).unwrap();
    tracing::info!("first record");
    tracing::info!("second record");
    drop(guards);
    let contents = std::fs::read_to_string(&path).unwrap();
    let records: Vec<serde_json::Value> = contents
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["fields"]["message"], "first record");
    assert_eq!(records[1]["fields"]["message"], "second record");
    assert_eq!(
        std::fs::metadata(path.with_file_name("dokime.jsonl.1"))
            .unwrap()
            .len(),
        32 * 1024 * 1024
    );
}

#[test]
fn plain_rolling_file() {
    if !isolated("plain_rolling_file", None) {
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let guards = dokime_server::host::install_logging(root.path(), false).unwrap();
    tracing::info!("boot phase reached storage");
    drop(guards);
    let files: Vec<_> = std::fs::read_dir(root.path().join("logs"))
        .unwrap()
        .map(Result::unwrap)
        .filter(|entry| entry.file_name() != "dokime.jsonl")
        .collect();
    assert_eq!(files.len(), 1, "exactly one plain rolling file");
    assert!(
        std::fs::read_to_string(files[0].path())
            .unwrap()
            .contains("boot phase reached storage")
    );
    assert!(!root.path().join("diagnostics/profiles").exists());
}

#[cfg(debug_assertions)]
#[test]
fn profiling_prunes_runs() {
    if !isolated("profiling_prunes_runs", Some("yes")) {
        return;
    }
    let root = tempfile::tempdir().unwrap();
    let profiles = root.path().join("diagnostics/profiles");
    for stamp in 1..=5 {
        std::fs::create_dir_all(profiles.join(format!("dev-{stamp}"))).unwrap();
    }
    std::fs::create_dir_all(profiles.join("keep-me")).unwrap();
    let guards = dokime_server::host::install_logging(root.path(), false).unwrap();
    let span = tracing::info_span!("profile_fixture");
    {
        let _entered = span.enter();
        tracing::info!("profile activity");
    }
    drop(span);
    drop(guards);
    let runs: Vec<_> = std::fs::read_dir(&profiles)
        .unwrap()
        .map(Result::unwrap)
        .filter(|entry| entry.file_name().to_string_lossy().starts_with("dev-"))
        .collect();
    assert_eq!(runs.len(), 3);
    assert!(profiles.join("dev-4").is_dir());
    assert!(profiles.join("dev-5").is_dir());
    assert!(profiles.join("keep-me").is_dir());
    let current = runs
        .iter()
        .find(|entry| entry.path().join("server.trace.json").exists())
        .unwrap();
    assert!(current.path().join("server.folded").is_file());
    let _: serde_json::Value =
        serde_json::from_slice(&std::fs::read(current.path().join("server.trace.json")).unwrap())
            .unwrap();
}
