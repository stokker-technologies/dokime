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
use crate::service_support::prelude::*;

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
async fn artifact_close_boundary() {
    use dokime_runtime::artifacts::{ArtifactStatus, ArtifactUploadInput};

    let root = tempdir().unwrap();
    let (mut runtime, handle, cycle_id, provider) = blocked_cycle(root.path()).await;
    let cycle = cycle_snapshot(&handle, &cycle_id).await;
    let execution = &cycle.test_executions[0];
    let mut records = Vec::new();
    for name in ["first.bin", "second.bin"] {
        let upload = handle
            .begin_artifact_upload(ArtifactUploadInput {
                scope: execution.scope.clone(),
                filename: name.into(),
                content_type: None,
                size_bytes: 3,
                sha256: None,
            })
            .await
            .unwrap();
        upload.send(b"abc").await.unwrap();
        let record = upload.finish().await.unwrap();
        assert_eq!(record.status, ArtifactStatus::Complete);
        assert_eq!(upload.finish().await.unwrap(), record);
        records.push(record);
    }
    let listed = handle.artifacts().await.unwrap();
    for record in &records {
        assert!(listed.contains(record));
        let content = handle.read_artifact_content(&record.artifact_id, 3).await.unwrap().unwrap();
        assert_eq!(content.bytes, b"abc");
    }

    provider.release();
    let checkpoint = crate::service_support::native::checkpoint(&handle, &execution.test_execution_id).await;
    for record in &records {
        let mut reader = handle.checkpoint_artifact(checkpoint.clone(), &record.artifact_id).await.unwrap();
        let mut bytes = [0; 3];
        assert_eq!(reader.read(&mut bytes).await.unwrap(), 3);
        assert_eq!(&bytes, b"abc");
    }
    drop(handle);
    runtime.shutdown().await.unwrap();
}

#[tokio::test]
async fn concurrent_log_bytes() {
    use std::io::Write;

    let root = tempdir().unwrap();
    let (mut runtime, handle, cycle_id, provider) = blocked_cycle(root.path()).await;
    let id = cycle_snapshot(&handle, &cycle_id).await.test_executions[0].test_execution_id.clone();
    let source = root.path().join("source.log");
    std::fs::write(&source, b"start\n").unwrap();
    let tail = handle.start_execution_log_tail(id.clone(), "logs/tail.log", &source).await.unwrap();

    // TODO(source-hook): the execution log tail publishes no copy progress; a tail event would replace this poll.
    loop {
        if handle.execution_file_content(&id, "logs/tail.log".into(), 0, 1024).await.unwrap().is_some_and(|value| value.bytes == b"start\n")
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    let suffix = vec![b'x'; 128 * 1024 + 19];
    std::fs::OpenOptions::new().append(true).open(&source).unwrap().write_all(&suffix).unwrap();
    let (first, second) = tokio::join!(handle.stop_execution_log_tail(&tail), handle.stop_execution_log_tail(&tail),);
    first.unwrap();
    second.unwrap();

    let drained = handle.execution_file_snapshot(&id, "logs/tail.log".into()).await.unwrap().unwrap();
    assert_eq!(&drained.bytes[..6], b"start\n");
    assert_eq!(&drained.bytes[6..], suffix);
    assert_eq!(drained.stream_state, dokime_runtime::runtime::query::ExecutionFileStreamState::Finalized);

    provider.release();
    crate::service_support::native::checkpoint(&handle, &id).await;
    let saved = handle.execution_file_snapshot(&id, "logs/tail.log".into()).await.unwrap().unwrap();
    assert_eq!(&saved.bytes[..6], b"start\n");
    assert_eq!(&saved.bytes[6..], suffix);
    assert_eq!(saved.stream_state, drained.stream_state);

    drop(handle);
    runtime.shutdown().await.unwrap();
}
