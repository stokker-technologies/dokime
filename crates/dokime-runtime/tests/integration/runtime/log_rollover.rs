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

// Third Party Imports

// Local Imports
use dokime_runtime::testing::LogWriter;

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
async fn execution_log_names_rollover_and_capture_limit() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../src/report/fixtures/execution-report-input.json")).unwrap();
    let mut cycle: dokime_domain::model::execution::result::CycleView = serde_json::from_value(fixture["input"]["cycle"].clone()).unwrap();
    cycle.test_executions[0].saved = None;
    let execution = &cycle.test_executions[0];
    let root = tempfile::tempdir().unwrap();
    let mut owner = dokime_runtime::Runtime::start(dokime_runtime::RuntimeConfig::new(root.path(), cycle.metadata.clone())).await.unwrap();
    let runtime = owner.handle();
    let storage = dokime_runtime::testing::ExecutionOutputs::new(&runtime);
    storage.acquire(&cycle, execution).await.unwrap();
    let id = &execution.test_execution_id;
    let path = "artifacts/execution.log".to_owned();
    storage.reserve_output(id, &path).await.unwrap();
    let mut writer = LogWriter::new(&runtime, id.clone(), path);
    writer.set_segment_bytes(LogWriter::segment_limit() - 1);
    writer.set_total_bytes(LogWriter::segment_limit() - 1);
    writer.flush(vec![1, 2, 3]).await.unwrap();
    let entries = storage.native_entries(id).await.unwrap();
    assert!(entries.iter().any(|entry| entry.relative_path == "artifacts/execution.log" && entry.size_bytes == 1));
    assert_eq!(writer.path(), "artifacts/execution.2.log");
    assert_eq!(storage.log_prefix(id, writer.path()).await.unwrap(), 2);
    assert_eq!(writer.segment_bytes(), 2);
    storage.native_target(id.clone(), "artifacts/execution.3.log".into(), true, false).await.unwrap();
    writer.set_segment_bytes(LogWriter::segment_limit());
    writer.flush(vec![4]).await.unwrap();
    assert_eq!(writer.path(), "artifacts/execution.4.log");
    assert_eq!(storage.log_prefix(id, writer.path()).await.unwrap(), 1);
    writer.set_total_bytes(LogWriter::execution_limit() - 1);
    writer.flush(vec![5]).await.unwrap();
    assert!(writer.flush(vec![6]).await.is_err());
    assert_eq!(storage.log_prefix(id, writer.path()).await.unwrap(), 2);
    drop(writer);
    drop(storage);
    drop(runtime);
    owner.shutdown().await.unwrap();
}
