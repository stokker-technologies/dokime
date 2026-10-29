/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Segmented Log Volume Fixture

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
pub async fn logs(days: u64, per_day: u64) {
    use dokime_domain::model::execution::result::CycleView;

    let fixture: serde_json::Value = serde_json::from_str(include_str!("../../../../report/fixtures/execution-report-input.json")).unwrap();
    let mut cycle: CycleView = serde_json::from_value(fixture["input"]["cycle"].clone()).unwrap();
    cycle.test_executions[0].saved = None;
    let execution = &cycle.test_executions[0];
    let root = tempfile::tempdir().unwrap();
    let config = crate::RuntimeConfig::new(root.path(), cycle.metadata.clone());
    let mut owner = crate::Runtime::start(config).await.unwrap();
    let runtime = owner.handle();
    let storage = &runtime.execution_storage;
    storage.acquire(&cycle, execution).await.unwrap();

    let mut log = StreamLog::default();
    let path = segment_path("workflow", 0);
    storage.reserve_output(&execution.test_execution_id, &path).await.unwrap();
    let mut writer = StreamLogWriter {
        storage,
        execution: &execution.test_execution_id,
        path,
        capacity: 64 * 1024,
        buffer: None,
        log: &mut log,
        workflow: "workflow",
    };
    runtime.inject_log_write_faults(writer.execution, 2, std::io::ErrorKind::Interrupted);
    for day in 0..days {
        for _ in 0..per_day {
            writer.buffer = Some(vec![day as u8; writer.capacity]);
            writer.flush().await.unwrap();
            assert!(writer.buffer.is_none());
            assert!(writer.log.segment_bytes <= SEGMENT_BYTES);
        }
        assert_eq!(writer.log.total_bytes, (day + 1) * per_day * writer.capacity as u64);
    }
    let total = days * per_day * writer.capacity as u64;
    assert_eq!(writer.log.segment as u64, (total - 1) / SEGMENT_BYTES);
    assert_eq!(writer.log.segment_bytes, (total - 1) % SEGMENT_BYTES + 1);
    let acknowledged = storage.log_prefix(writer.execution, &writer.path).await.unwrap();
    runtime.inject_log_write_faults(writer.execution, 1, std::io::ErrorKind::StorageFull);
    writer.buffer = Some(vec![9]);
    let before = writer.log.total_bytes;
    assert!(writer.flush().await.is_err());
    assert_eq!(writer.log.total_bytes, before);
    assert_eq!(storage.log_prefix(writer.execution, &writer.path).await.unwrap(), acknowledged);
    writer.buffer = Some(vec![9]);
    writer.flush().await.unwrap();
    assert_eq!(storage.log_prefix(writer.execution, &writer.path).await.unwrap(), acknowledged + 1);

    // Advance Only The Synthetic Volume Counter To Exercise The Exact Limit.
    writer.log.total_bytes = EXECUTION_BYTES - 1;
    writer.buffer = Some(vec![10]);
    writer.flush().await.unwrap();
    assert_eq!(writer.log.total_bytes, EXECUTION_BYTES);
    writer.buffer = Some(vec![11]);
    assert!(writer.flush().await.is_err());
    assert_eq!(writer.log.total_bytes, EXECUTION_BYTES);
    assert_eq!(storage.log_prefix(writer.execution, &writer.path).await.unwrap(), acknowledged + 2);

    drop(runtime);
    owner.shutdown().await.unwrap();
}
/* === Private Functions === */

/* === Tests === */
