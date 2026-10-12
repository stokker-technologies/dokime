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
use super::*;

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

pub(crate) async fn execution_save_sealing() {
    let root = tempdir().unwrap();
    let (mut owner, handle, cycle_id, executor) = blocked_cycle(root.path()).await;
    let cycle = wait_for_cycle_lifecycle(&handle, &cycle_id, CycleStatus::Running).await;
    let id = cycle.test_executions[0].test_execution_id.clone();
    handle.storage().flush().await.unwrap();
    let view = handle
        .storage()
        .view_with_limits(dokime_app::config::StorageReadSettings::default().execution)
        .await
        .unwrap();
    let running = dokime::storage::objects::execution::read_in(
        handle.storage(),
        &view,
        &id,
        harmos::storage::ObjectVersion::Latest,
        dokime_app::config::StorageReadSettings::default().decoded_bytes,
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(running.value.status, ExecutionStatus::Pending);
    assert!(running.value.saved.is_none());
    let snapshot = running.value.snapshot.clone();
    drop(running);
    drop(view);
    executor.release();
    let completed = wait_for_cycle_lifecycle(&handle, &cycle_id, CycleStatus::Completed).await;
    let cycle_seal = completed.saved.as_ref().expect("completed cycle is sealed");
    cycle_seal.validate_view(&completed).unwrap();
    assert_eq!(cycle_seal.executions.len(), 1);
    assert_eq!(cycle_seal.executions[0].execution_id, id);
    let cycle_report = cycle_seal.report.clone();
    let checkpoint = crate::support::native::checkpoint(&handle, &id).await;
    let terminal = crate::support::native::open(&handle, &checkpoint).await;
    assert_eq!(terminal.execution().status, ExecutionStatus::Passed);
    assert_eq!(terminal.execution().snapshot, snapshot);
    assert!(terminal.execution().saved.is_some());
    let html = crate::support::native::report(&handle, &id).await;
    assert!(html.contains("hub-report-data"));
    assert!(html.contains(id.as_str()));
    drop(terminal);
    drop(handle);
    owner.shutdown().await.unwrap();
    let (mut reopened, handle) = booted(root.path()).await;
    assert!(
        handle
            .cycles()
            .await
            .iter()
            .all(|cycle| cycle.test_cycle_id != cycle_id),
        "sealed cycle stays in saved history"
    );
    let view = handle
        .storage()
        .view_with_limits(dokime_app::config::StorageReadSettings::default().execution)
        .await
        .unwrap();
    let saved_cycle = dokime::storage::objects::cycle::read_in(
        handle.storage(),
        &view,
        &cycle_id,
        harmos::storage::ObjectVersion::Latest,
        dokime_app::config::StorageReadSettings::default().decoded_bytes,
    )
    .await
    .unwrap()
    .unwrap();
    let seal = saved_cycle.value.saved.as_ref().unwrap();
    seal.validate(&saved_cycle.value).unwrap();
    assert_eq!(seal.report, cycle_report);
    let mut reader = view.read_file(&seal.report).await.unwrap();
    let mut bytes = Vec::new();
    while let Some(chunk) = reader
        .next_chunk(std::num::NonZeroUsize::new(64 * 1024).unwrap())
        .await
        .unwrap()
    {
        bytes.extend_from_slice(chunk.as_bytes());
    }
    assert!(String::from_utf8(bytes).unwrap().contains(id.as_str()));
    drop(reader);
    drop(saved_cycle);
    drop(view);
    let restored = crate::support::native::open(&handle, &checkpoint).await;
    assert_eq!(restored.execution().snapshot, snapshot);
    assert_eq!(crate::support::native::report(&handle, &id).await, html);
    drop(restored);
    drop(handle);
    reopened.shutdown().await.unwrap();
}

pub(crate) async fn terminal_capture_repaired() {
    let root = tempdir().unwrap();
    let (mut owner, handle, cycle_id, executor) = blocked_cycle(root.path()).await;
    std::fs::write(
        root.path().join("brand.toml"),
        "[app]\nlabel = \"Test\"\nslug = \"test\"\nidentifier = \"com.test.hub\"\n",
    )
    .unwrap();
    let mut brand = dokime_app::capabilities::installed_plugins::load_brand(root.path()).unwrap();
    brand.customization.display_name = Some("x".repeat(4 * 1024 * 1024));
    handle.set_brand(Some(brand));
    let id = cycle_snapshot(&handle, &cycle_id).await.test_executions[0]
        .test_execution_id
        .clone();
    executor.release();
    tokio::time::timeout(Duration::from_secs(5), async {
        while cycle_snapshot(&handle, &cycle_id).await.test_executions[0]
            .finished_at
            .is_none()
        {
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert!(
        handle
            .execution_saved_checkpoint(id.clone(), |checkpoint, _| Ok(checkpoint.clone()))
            .await
            .is_err()
    );
    handle.set_brand(None);
    let checkpoint = crate::support::native::checkpoint(&handle, &id).await;
    assert_eq!(
        crate::support::native::open(&handle, &checkpoint)
            .await
            .execution()
            .status,
        ExecutionStatus::Passed
    );
    drop(handle);
    owner.shutdown().await.unwrap();
}
