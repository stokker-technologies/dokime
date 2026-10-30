/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Cleanup Before A Log Resident Opens Its Source Retains Every Byte.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn early_log_stop() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!("../../../../report/fixtures/execution-report-input.json")).unwrap();
    let mut cycle: dokime_domain::model::execution::result::CycleView = serde_json::from_value(fixture["input"]["cycle"].clone()).unwrap();
    cycle.test_executions[0].saved = None;
    let execution = &cycle.test_executions[0];
    let root = tempfile::tempdir().unwrap();
    let config = crate::RuntimeConfig::new(root.path(), cycle.metadata.clone());
    let mut owner = crate::Runtime::start(config).await.unwrap();
    let runtime = owner.handle();
    let storage = &runtime.execution_storage;
    storage.acquire(&cycle, execution).await.unwrap();
    let path = "logs/early.log";
    storage.reserve_output(&execution.test_execution_id, path).await.unwrap();
    let source = root.path().join("source.log");
    let bytes: Vec<u8> = (0..128 * 1024 + 19).map(|index| (index % 251) as u8).collect();
    std::fs::write(&source, &bytes).unwrap();
    let mut tail = LogTail {
        poll_interval: storage.storage.settings().filesystem.poll_interval,
        storage: Arc::clone(storage),
        source: source.clone(),
        execution: execution.test_execution_id.clone(),
        path: path.into(),
        offset: 0,
        opening: None,
        file: None,
        pending: None,
        read_bytes: 64 * 1024,
        source_failure: None,
        final_end: None,
        drained: false,
    };
    // Deterministically Enter Cleanup Without Polling Run Or Opening The Source.
    tail.cleanup().await.unwrap();
    assert!(tail.drained);
    assert_eq!(tail.offset, bytes.len() as u64);
    std::fs::remove_file(source).unwrap();
    tail.cleanup().await.unwrap();
    let saved = storage.native_read(&execution.test_execution_id, path, 0, bytes.len()).await.unwrap().unwrap();
    assert_eq!(saved.bytes, bytes);
    assert_eq!(saved.stream_state, crate::runtime::query::ExecutionFileStreamState::Finalized);
    drop(saved);
    drop(tail);
    drop(runtime);
    owner.shutdown().await.unwrap();
}

/* === Private Functions === */

/* === Tests === */
