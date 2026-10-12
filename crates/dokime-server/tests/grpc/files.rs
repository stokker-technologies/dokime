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
use dokime_protocol::v1::content_client::ContentClient;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

pub(crate) async fn live_file_rewrites() {
    let Gated {
        root: _root,
        gate,
        mut owner,
        server,
        mut execution,
        cycle,
    } = Gated::open_bounded(4096).await;
    let runtime = owner.handle();
    execution
        .run_cycle(authenticated(run(&cycle)))
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(5), gate.started.notified())
        .await
        .unwrap();
    let executions = execution
        .list_executions(authenticated(v1::ListExecutionsRequest {
            cycle_id: cycle.clone(),
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner();
    let id =
        dokime_app::core::TestExecutionId::from_uuid(executions.executions[0].execution_id.clone())
            .unwrap();
    let first = runtime
        .execution_file_target(&id, "files/a.bin".into(), false, true)
        .await
        .unwrap()
        .unwrap();
    let second = runtime
        .execution_file_target(&id, "files/b.bin".into(), false, false)
        .await
        .unwrap()
        .unwrap();
    let bytes = (0..90_000).map(|n| (n % 251) as u8).collect::<Vec<_>>();
    std::fs::write(&first.absolute_path, &bytes).unwrap();
    std::fs::write(&second.absolute_path, []).unwrap();
    let mut content = ContentClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let list = |limit| v1::ListExecutionFilesRequest {
        execution_id: id.to_string(),
        limit,
        ..Default::default()
    };
    assert_eq!(
        content
            .list_execution_files(list(1))
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    let page = content
        .list_execution_files(authenticated(list(1)))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(page.files.len(), 1);
    let mut paths = page
        .files
        .iter()
        .map(|file| file.relative_path.clone())
        .collect::<Vec<_>>();
    let mut after = page.next_after_path.clone();
    while let Some(path) = after {
        let next = content
            .list_execution_files(authenticated(v1::ListExecutionFilesRequest {
                after_path: path,
                cursor: page.cursor.clone(),
                ..list(1)
            }))
            .await
            .unwrap()
            .into_inner();
        paths.extend(next.files.iter().map(|file| file.relative_path.clone()));
        after = next.next_after_path;
    }
    assert!(paths.contains(&"files/a.bin".to_owned()) && paths.contains(&"files/b.bin".to_owned()));
    assert!(paths.windows(2).all(|pair| pair[0] < pair[1]));
    let read = |path: &str, offset, max_bytes| v1::ReadExecutionFileRequest {
        execution_id: id.to_string(),
        relative_path: path.into(),
        offset,
        max_bytes,
    };
    let window = content
        .read_execution_file(authenticated(read("files/a.bin", 37, 65_536)))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(window.offset, 37);
    assert_eq!(window.observed_size_bytes, bytes.len() as u64);
    assert_eq!(window.state, v1::ExecutionFileState::Streaming as i32);
    assert_eq!(window.data, bytes[37..37 + window.data.len()]);
    assert!(window.data.len() < 4096);
    assert_eq!(
        content
            .read_execution_file(authenticated(read("files/a.bin", 0, 65_537)))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    assert_eq!(
        content
            .read_execution_file(authenticated(read("../../secret", 0, 20)))
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    assert_eq!(
        content
            .read_execution_file(authenticated(read(&first.absolute_path, 0, 20)))
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    let mut download = content
        .download_execution_file(authenticated(v1::DownloadExecutionFileRequest {
            execution_id: id.to_string(),
            relative_path: "files/a.bin".into(),
        }))
        .await
        .unwrap()
        .into_inner();
    let metadata = match download.message().await.unwrap().unwrap().payload.unwrap() {
        v1::execution_file_download_frame::Payload::Metadata(metadata) => metadata,
        payload => panic!("expected execution file metadata, got {payload:?}"),
    };
    assert_eq!(metadata.relative_path, "files/a.bin");
    // `.bin` has no registered media type, so the runtime preserves its
    // intentionally conservative unknown classification while the bytes are
    // still transferred verbatim.
    assert_eq!(metadata.kind, v1::ExecutionFileKind::Unknown as i32);
    assert_eq!(metadata.state, v1::ExecutionFileState::Streaming as i32);
    assert_eq!(metadata.observed_size_bytes, bytes.len() as u64);
    let mut downloaded = Vec::new();
    let completion = loop {
        let frame = download.message().await.unwrap().unwrap();
        match frame.payload.unwrap() {
            v1::execution_file_download_frame::Payload::Chunk(chunk) => {
                assert_eq!(chunk.offset, downloaded.len() as u64);
                assert!(!chunk.data.is_empty());
                assert!(chunk.data.len() <= 64 * 1024);
                downloaded.extend(chunk.data);
            }
            v1::execution_file_download_frame::Payload::Complete(completion) => break completion,
            payload => panic!("expected execution file chunk or completion, got {payload:?}"),
        }
    };
    assert_eq!(downloaded, bytes);
    assert_eq!(completion.size_bytes, downloaded.len() as u64);
    assert_eq!(
        completion.sha256,
        Sha256::digest(&downloaded)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
    assert!(download.message().await.unwrap().is_none());
    assert_eq!(
        content
            .download_execution_file(v1::DownloadExecutionFileRequest {
                execution_id: id.to_string(),
                relative_path: "files/a.bin".into(),
            })
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    assert_eq!(
        content
            .download_execution_file(authenticated(v1::DownloadExecutionFileRequest {
                execution_id: id.to_string(),
                relative_path: "../../secret".into(),
            }))
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    assert_eq!(
        content
            .download_execution_file(authenticated(v1::DownloadExecutionFileRequest {
                execution_id: id.to_string(),
                relative_path: first.absolute_path.clone(),
            }))
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    let oversize = std::fs::OpenOptions::new()
        .write(true)
        .open(&first.absolute_path)
        .unwrap();
    oversize.set_len(2 * 1024 * 1024 * 1024 + 1).unwrap();
    assert_eq!(
        content
            .download_execution_file(authenticated(v1::DownloadExecutionFileRequest {
                execution_id: id.to_string(),
                relative_path: "files/a.bin".into(),
            }))
            .await
            .unwrap_err()
            .code(),
        Code::ResourceExhausted
    );
    oversize.set_len(bytes.len() as u64).unwrap();
    let cancel_server = Server::start(
        runtime.clone(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Read)])
            .unwrap()
            .with_max_message_bytes(4096),
    )
    .await
    .unwrap();
    let stream = Endpoint::from_shared(cancel_server.local_endpoint())
        .unwrap()
        .initial_connection_window_size(1)
        .initial_stream_window_size(1)
        .connect()
        .await
        .unwrap();
    let mut cancel_content = ContentClient::new(stream);
    let mut canceled = cancel_content
        .download_execution_file(authenticated(v1::DownloadExecutionFileRequest {
            execution_id: id.to_string(),
            relative_path: "files/a.bin".into(),
        }))
        .await
        .unwrap()
        .into_inner();
    cancel_server.shutdown().await.unwrap();
    let mut saw_completion = false;
    tokio::time::timeout(Duration::from_secs(2), async {
        while let Ok(Some(frame)) = canceled.message().await {
            saw_completion |= matches!(
                frame.payload,
                Some(v1::execution_file_download_frame::Payload::Complete(_))
            );
        }
    })
    .await
    .unwrap();
    assert!(
        !saw_completion,
        "listener shutdown must stop before completion"
    );
    let end = content
        .read_execution_file(authenticated(read("files/a.bin", u64::MAX, 20)))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(end.offset, bytes.len() as u64);
    assert!(end.data.is_empty());
    std::fs::write(&first.absolute_path, b"final rewrite").unwrap();
    assert_eq!(
        content
            .list_execution_files(authenticated(v1::ListExecutionFilesRequest {
                after_path: page.next_after_path.unwrap(),
                cursor: page.cursor,
                ..list(1)
            }))
            .await
            .unwrap_err()
            .code(),
        Code::Aborted
    );
    gate.release.notify_one();
    let final_window = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let window = content
                .read_execution_file(authenticated(read("files/a.bin", 0, 128)))
                .await
                .unwrap()
                .into_inner();
            if window.state == v1::ExecutionFileState::Finalized as i32 {
                break window;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap();
    assert_eq!(final_window.data, b"final rewrite");
    assert_eq!(final_window.observed_size_bytes, 13);
    let mut finalized_download = content
        .download_execution_file(authenticated(v1::DownloadExecutionFileRequest {
            execution_id: id.to_string(),
            relative_path: "files/a.bin".into(),
        }))
        .await
        .unwrap()
        .into_inner();
    let finalized_metadata = finalized_download
        .message()
        .await
        .unwrap()
        .unwrap()
        .payload
        .unwrap();
    let finalized_metadata = match finalized_metadata {
        v1::execution_file_download_frame::Payload::Metadata(metadata) => metadata,
        payload => panic!("expected finalized metadata, got {payload:?}"),
    };
    assert_eq!(
        finalized_metadata.state,
        v1::ExecutionFileState::Finalized as i32
    );
    assert_eq!(finalized_metadata.observed_size_bytes, 13);
    let finalized_chunk = finalized_download
        .message()
        .await
        .unwrap()
        .unwrap()
        .payload
        .unwrap();
    let finalized_chunk = match finalized_chunk {
        v1::execution_file_download_frame::Payload::Chunk(chunk) => chunk,
        payload => panic!("expected finalized file chunk, got {payload:?}"),
    };
    assert_eq!(finalized_chunk.offset, 0);
    assert_eq!(finalized_chunk.data, b"final rewrite");
    let finalized_completion = finalized_download
        .message()
        .await
        .unwrap()
        .unwrap()
        .payload
        .unwrap();
    let finalized_completion = match finalized_completion {
        v1::execution_file_download_frame::Payload::Complete(completion) => completion,
        payload => panic!("expected finalized completion, got {payload:?}"),
    };
    assert_eq!(finalized_completion.size_bytes, 13);
    assert_eq!(
        finalized_completion.sha256,
        Sha256::digest(b"final rewrite")
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
    assert!(finalized_download.message().await.unwrap().is_none());
    let empty = content
        .read_execution_file(authenticated(read("files/b.bin", 0, 128)))
        .await
        .unwrap()
        .into_inner();
    assert!(empty.data.is_empty());
    server.shutdown().await.unwrap();
    drop(runtime);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn execution_file_buffer() {
    // The boundary is the storage read memory budget: a snapshot of this file
    // cannot be decoded within `decoded_bytes`, so the transfer must stream it
    // in bounded chunks. Twice the budget crosses it with margin. The whole
    // transfer runs inside one query computation (5 s by default), and the
    // earlier 300 MiB workload needed 3.5 to 6.4 s under machine load, so a
    // larger file measures the machine instead of the boundary.
    let size_bytes = 2 * dokime_app::config::StorageReadSettings::default()
        .decoded_bytes
        .get() as u64;

    let Gated {
        root: _root,
        gate,
        mut owner,
        server,
        mut execution,
        cycle,
    } = Gated::open().await;
    let runtime = owner.handle();
    execution
        .run_cycle(authenticated(run(&cycle)))
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(5), gate.started.notified())
        .await
        .unwrap();
    let executions = execution
        .list_executions(authenticated(v1::ListExecutionsRequest {
            cycle_id: cycle.clone(),
            ..Default::default()
        }))
        .await
        .unwrap()
        .into_inner();
    let id =
        dokime_app::core::TestExecutionId::from_uuid(executions.executions[0].execution_id.clone())
            .unwrap();
    let target = runtime
        .execution_file_target(&id, "large.bin".into(), false, true)
        .await
        .unwrap()
        .unwrap();
    std::fs::OpenOptions::new()
        .write(true)
        .open(&target.absolute_path)
        .unwrap()
        .set_len(size_bytes)
        .unwrap();
    let mut content = ContentClient::connect(server.local_endpoint())
        .await
        .unwrap();
    let mut download = content
        .download_execution_file(authenticated(v1::DownloadExecutionFileRequest {
            execution_id: id.to_string(),
            relative_path: "large.bin".into(),
        }))
        .await
        .unwrap()
        .into_inner();
    let metadata = match download.message().await.unwrap().unwrap().payload.unwrap() {
        v1::execution_file_download_frame::Payload::Metadata(metadata) => metadata,
        payload => panic!("expected execution file metadata, got {payload:?}"),
    };
    assert_eq!(metadata.observed_size_bytes, size_bytes);
    let mut downloaded = 0_u64;
    let mut digest = Sha256::new();
    let completion = loop {
        let frame = download.message().await.unwrap().unwrap();
        match frame.payload.unwrap() {
            v1::execution_file_download_frame::Payload::Chunk(chunk) => {
                assert_eq!(chunk.offset, downloaded);
                assert!(!chunk.data.is_empty());
                assert!(chunk.data.len() as u64 <= 1024 * 1024);
                downloaded += chunk.data.len() as u64;
                digest.update(&chunk.data);
            }
            v1::execution_file_download_frame::Payload::Complete(completion) => break completion,
            payload => panic!("expected execution file chunk or completion, got {payload:?}"),
        }
    };
    assert_eq!(downloaded, size_bytes);
    assert_eq!(completion.size_bytes, size_bytes);
    assert_eq!(
        completion.sha256,
        digest
            .finalize()
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
    assert!(download.message().await.unwrap().is_none());
    server.shutdown().await.unwrap();
    gate.release.notify_one();
    drop(runtime);
    owner.shutdown().await.unwrap();
}

/* === Tests === */
