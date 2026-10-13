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
use sha2::{Digest, Sha256};

// Local Imports
use super::*;
use dokime_contract::operations::content as wire;

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

    // Resume The Original Live File After Its Viewer Disconnects.
    let path = "follow.bin";
    let target = handle
        .execution_file_target(&id, path.to_owned(), false, true)
        .await
        .unwrap()
        .unwrap();
    std::fs::write(&target.absolute_path, b"prefix").unwrap();
    let maximum = std::num::NonZeroUsize::new(3).unwrap();
    let mut follower = handle
        .follow_execution_file(&id, path, None, 0)
        .await
        .unwrap()
        .unwrap();
    let identity = follower.metadata().identity.clone().unwrap();
    let first = follower.next_frame(maximum).await.unwrap().unwrap();
    let Some(wire::ExecutionFileFollowFramePayload::Chunk(chunk)) = &first.payload else {
        panic!("live file starts with its retained prefix");
    };
    assert_eq!(chunk.offset, 0);
    assert_eq!(chunk.data, b"pre");
    drop(first);
    drop(follower);
    let expected = b"prefix-appended";
    {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&target.absolute_path)
            .unwrap();
        std::io::Write::write_all(&mut file, b"-appended").unwrap();
    }
    let mut other = identity.clone();
    other.relative_path = "other.bin".to_owned();
    assert!(
        handle
            .follow_execution_file(&id, path, Some(&other), 3)
            .await
            .is_err()
    );
    let mut follower = handle
        .follow_execution_file(&id, path, Some(&identity), 3)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(follower.metadata().identity.as_ref(), Some(&identity));
    let mut retained = b"pre".to_vec();
    tokio::time::timeout(Duration::from_secs(5), async {
        while retained.len() < expected.len() {
            let frame = follower.next_frame(maximum).await.unwrap().unwrap();
            let Some(wire::ExecutionFileFollowFramePayload::Chunk(chunk)) = &frame.payload else {
                panic!("live file remains producing until its original owner settles");
            };
            assert_eq!(chunk.offset, retained.len() as u64);
            retained.extend_from_slice(&chunk.data);
        }
    })
    .await
    .unwrap();
    assert_eq!(retained, expected);
    executor.release();
    let completed = wait_for_cycle_lifecycle(&handle, &cycle_id, CycleStatus::Completed).await;
    let frame = tokio::time::timeout(Duration::from_secs(5), follower.next_frame(maximum))
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    let Some(wire::ExecutionFileFollowFramePayload::Terminal(terminal)) = &frame.payload else {
        panic!("the original production closes with a sealed reference");
    };
    assert_eq!(terminal.phase, wire::ExecutionFilePhase::Finished);
    let sealed = terminal.sealed.as_ref().unwrap();
    assert_eq!(sealed.identity.as_ref(), Some(&identity));
    assert_eq!(sealed.length, expected.len() as u64);
    assert_eq!(
        sealed.sha256,
        Sha256::digest(expected)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
    assert!(follower.next_frame(maximum).await.unwrap().is_none());
    drop(frame);
    drop(follower);
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
    let follower = handle
        .follow_execution_file(&id, path, Some(&identity), expected.len() as u64)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(follower.metadata().identity.as_ref(), Some(&identity));
    assert_eq!(
        follower.metadata().phase,
        wire::ExecutionFilePhase::Finished
    );
    drop(follower);
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
