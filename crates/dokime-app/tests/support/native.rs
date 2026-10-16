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
use std::{num::NonZeroUsize, time::Duration};
// Third Party Imports
// Local Imports
use dokime::{core::TestExecutionId, execution::saved::ExecutionCheckpoint};
use dokime_app::{RuntimeHandle, runtime::ExecutionView};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
pub async fn checkpoint(handle: &RuntimeHandle, id: &TestExecutionId) -> ExecutionCheckpoint {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    loop {
        match handle.execution_saved_checkpoint(id.clone()).await {
            Ok(checkpoint) => return checkpoint,
            Err(error) if tokio::time::Instant::now() >= deadline => {
                panic!("saved seal did not appear: {error}")
            }
            Err(_) => tokio::time::sleep(Duration::from_millis(10)).await,
        }
    }
}
pub async fn open(handle: &RuntimeHandle, checkpoint: &ExecutionCheckpoint) -> ExecutionView {
    ExecutionView::open_checkpoint(handle.storage(), checkpoint)
        .await
        .unwrap()
        .unwrap()
}
pub async fn records(
    handle: &RuntimeHandle,
    checkpoint: &ExecutionCheckpoint,
) -> Vec<dokime::records::RecordEntry> {
    let mut records = Vec::new();
    let mut after = None;
    loop {
        let page = handle
            .execution_checkpoint_records(
                checkpoint.clone(),
                dokime_app::runtime::query::checkpoint_records::ExecutionRecordQuery {
                    after,
                    limit: Some(128),
                },
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
pub async fn samples(
    handle: &RuntimeHandle,
    checkpoint: &ExecutionCheckpoint,
) -> Vec<dokime::telemetry::TelemetrySample> {
    let evidence = handle
        .execution_checkpoint_evidence(checkpoint.clone())
        .await
        .unwrap();
    let mut samples = Vec::new();
    for table in evidence.telemetry.iter() {
        let mut after = None;
        loop {
            let page = handle
                .execution_checkpoint_telemetry(
                    checkpoint.clone(),
                    &table.contract.stream,
                    dokime_app::runtime::query::checkpoint_telemetry::ExecutionTelemetryQuery {
                        after,
                        limit: Some(128),
                    },
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
    let bytes = dokime::storage::read_file_bytes(handle.storage(), file)
        .await
        .unwrap();
    String::from_utf8(bytes).unwrap()
}

/* === Private Functions === */

/* === Tests === */
