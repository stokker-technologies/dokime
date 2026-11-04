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
use dokime_domain::model;
use dokime_runtime::{RuntimeHandle, runtime::ExecutionView};
use {dokime_domain::model::execution::saved::ExecutionCheckpoint, dokime_runtime::TestExecutionId};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn record_page(
    rows: harmos::storage::ReadBatch<Vec<harmos::storage::StoredRecord<dokime_runtime::storage::records::RecordRow>>>,
) -> dokime_runtime::storage::Result<harmos::storage::ReadBatch<Vec<model::records::evidence::RecordEntry>>> {
    use dokime_runtime::storage::Error;
    rows.try_map(|rows| {
        let mut records = Vec::new();
        let allocation = records.try_reserve_exact(rows.len());
        let allocation_failed = Error::Invalid("fixture record allocation failed");
        allocation.map_err(|_| allocation_failed)?;
        for row in rows {
            let record = row.payload.entry()?;
            records.push(record);
        }
        Ok(records)
    })
}

pub async fn checkpoint(handle: &RuntimeHandle, id: &TestExecutionId) -> ExecutionCheckpoint {
    let resident = handle.watch_state().latest().state().executions.contains_key(id);
    if resident {
        handle.wait_execution_report(id).await.unwrap();
    }
    handle.execution_saved_checkpoint(id.clone()).await.unwrap()
}

pub async fn open(handle: &RuntimeHandle, checkpoint: &ExecutionCheckpoint) -> ExecutionView {
    ExecutionView::open_checkpoint(handle.storage(), checkpoint).await.unwrap().unwrap()
}
pub async fn records(handle: &RuntimeHandle, checkpoint: &ExecutionCheckpoint) -> Vec<model::records::evidence::RecordEntry> {
    let mut records = Vec::new();
    let mut after = None;
    loop {
        let page = handle
            .execution_checkpoint_records(
                checkpoint.clone(),
                dokime_runtime::runtime::query::checkpoint_records::ExecutionRecordQuery { after, limit: Some(128) },
            )
            .await
            .unwrap();
        assert!(page.issues.is_empty() && !page.issues_truncated);
        records.extend(page.records.iter().map(|row| row.record.clone()));
        let Some(next) = &page.next else {
            break;
        };
        after = Some(next.clone());
    }
    records
}
pub async fn samples(handle: &RuntimeHandle, checkpoint: &ExecutionCheckpoint) -> Vec<dokime_runtime::telemetry::TelemetrySample> {
    let evidence = handle.execution_checkpoint_evidence(checkpoint.clone()).await.unwrap();
    let mut samples = Vec::new();
    for table in evidence.telemetry.iter() {
        let mut after = None;
        loop {
            let page = handle
                .execution_checkpoint_telemetry(
                    checkpoint.clone(),
                    &table.contract.stream,
                    dokime_runtime::runtime::query::checkpoint_telemetry::ExecutionTelemetryQuery { after, limit: Some(128) },
                )
                .await
                .unwrap();
            assert!(page.issues.is_empty() && !page.issues_truncated);
            samples.extend(page.rows.iter().map(|row| row.sample.clone()));
            let Some(next) = &page.next else {
                break;
            };
            after = Some(next.clone());
        }
    }
    samples
}
pub async fn report(handle: &RuntimeHandle, id: &TestExecutionId) -> String {
    let checkpoint = checkpoint(handle, id).await;
    let view = open(handle, &checkpoint).await;
    let file = &view.execution().saved.as_ref().unwrap().report;
    let bytes = dokime_runtime::storage::read_file_bytes(handle.storage(), file).await.unwrap();
    String::from_utf8(bytes.as_slice().to_vec()).unwrap()
}

/* === Private Functions === */

/* === Tests === */
