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
        execution,
        cycle,
    } = Gated::open_bounded(4096).await;
    let runtime = owner.handle();
    execution
        .unary::<contracts::execution::RunCycle>(authenticated(run(&cycle)))
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(5), gate.started.notified())
        .await
        .unwrap();
    let executions = execution
        .unary::<contracts::execution::ListExecutions>(authenticated(
            contracts::execution::ListExecutionsRequest {
                cycle_id: cycle.clone(),
                ..Default::default()
            },
        ))
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
    let content = WireClient::connect(server.local_endpoint()).await.unwrap();
    let list = |limit| contracts::content::ListExecutionFilesRequest {
        execution_id: id.to_string(),
        limit,
        ..Default::default()
    };
    assert_eq!(
        content
            .unary::<contracts::content::ListExecutionFiles>(list(1))
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    let page = content
        .unary::<contracts::content::ListExecutionFiles>(authenticated(list(1)))
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
            .unary::<contracts::content::ListExecutionFiles>(authenticated(
                contracts::content::ListExecutionFilesRequest {
                    after_path: path,
                    cursor: page.cursor.clone(),
                    ..list(1)
                },
            ))
            .await
            .unwrap()
            .into_inner();
        paths.extend(next.files.iter().map(|file| file.relative_path.clone()));
        after = next.next_after_path;
    }
    assert!(paths.contains(&"files/a.bin".to_owned()) && paths.contains(&"files/b.bin".to_owned()));
    assert!(paths.windows(2).all(|pair| pair[0] < pair[1]));
    let read = |path: &str, offset, max_bytes| contracts::content::ReadExecutionFileRequest {
        execution_id: id.to_string(),
        relative_path: path.into(),
        offset,
        max_bytes,
    };
    let window = content
        .unary::<contracts::content::ReadExecutionFile>(authenticated(read(
            "files/a.bin",
            37,
            65_536,
        )))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(window.offset, 37);
    assert_eq!(window.observed_size_bytes, bytes.len() as u64);
    assert_eq!(
        window.state,
        contracts::content::ExecutionFileState::Streaming
    );
    assert_eq!(window.data, bytes[37..37 + window.data.len()]);
    assert!(window.data.len() < 4096);
    assert_eq!(
        content
            .unary::<contracts::content::ReadExecutionFile>(authenticated(read(
                "files/a.bin",
                0,
                65_537
            )))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument
    );
    assert_eq!(
        content
            .unary::<contracts::content::ReadExecutionFile>(authenticated(read(
                "../../secret",
                0,
                20
            )))
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    assert_eq!(
        content
            .unary::<contracts::content::ReadExecutionFile>(authenticated(read(
                &first.absolute_path,
                0,
                20
            )))
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    let mut download = content
        .download::<contracts::content::DownloadExecutionFile>(authenticated(
            contracts::content::DownloadExecutionFileRequest {
                execution_id: id.to_string(),
                relative_path: "files/a.bin".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let metadata = match download.message().await.unwrap().unwrap().payload.unwrap() {
        contracts::content::ExecutionFileDownloadFramePayload::Metadata(metadata) => metadata,
        payload => panic!("expected execution file metadata, got {payload:?}"),
    };
    assert_eq!(metadata.relative_path, "files/a.bin");
    // `.bin` has no registered media type, so the runtime preserves its
    // intentionally conservative unknown classification while the bytes are
    // still transferred verbatim.
    assert_eq!(
        metadata.kind,
        contracts::content::ExecutionFileKind::Unknown
    );
    assert_eq!(
        metadata.state,
        contracts::content::ExecutionFileState::Streaming
    );
    assert_eq!(metadata.observed_size_bytes, bytes.len() as u64);
    let mut downloaded = Vec::new();
    let completion = loop {
        let frame = download.message().await.unwrap().unwrap();
        match frame.payload.unwrap() {
            contracts::content::ExecutionFileDownloadFramePayload::Chunk(chunk) => {
                assert_eq!(chunk.offset, downloaded.len() as u64);
                assert!(!chunk.data.is_empty());
                assert!(chunk.data.len() <= 64 * 1024);
                downloaded.extend(chunk.data);
            }
            contracts::content::ExecutionFileDownloadFramePayload::Complete(completion) => {
                break completion;
            }
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
            .download::<contracts::content::DownloadExecutionFile>(
                contracts::content::DownloadExecutionFileRequest {
                    execution_id: id.to_string(),
                    relative_path: "files/a.bin".into(),
                }
            )
            .await
            .unwrap_err()
            .code(),
        Code::Unauthenticated
    );
    assert_eq!(
        content
            .download::<contracts::content::DownloadExecutionFile>(authenticated(
                contracts::content::DownloadExecutionFileRequest {
                    execution_id: id.to_string(),
                    relative_path: "../../secret".into(),
                }
            ))
            .await
            .unwrap_err()
            .code(),
        Code::NotFound
    );
    assert_eq!(
        content
            .download::<contracts::content::DownloadExecutionFile>(authenticated(
                contracts::content::DownloadExecutionFileRequest {
                    execution_id: id.to_string(),
                    relative_path: first.absolute_path.clone(),
                }
            ))
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
            .download::<contracts::content::DownloadExecutionFile>(authenticated(
                contracts::content::DownloadExecutionFileRequest {
                    execution_id: id.to_string(),
                    relative_path: "files/a.bin".into(),
                }
            ))
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
    let cancel_content = WireClient::new(stream);
    let mut canceled = cancel_content
        .download::<contracts::content::DownloadExecutionFile>(authenticated(
            contracts::content::DownloadExecutionFileRequest {
                execution_id: id.to_string(),
                relative_path: "files/a.bin".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    cancel_server.shutdown().await.unwrap();
    let mut saw_completion = false;
    tokio::time::timeout(Duration::from_secs(2), async {
        while let Ok(Some(frame)) = canceled.message().await {
            saw_completion |= matches!(
                frame.payload,
                Some(contracts::content::ExecutionFileDownloadFramePayload::Complete(_))
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
        .unary::<contracts::content::ReadExecutionFile>(authenticated(read(
            "files/a.bin",
            u64::MAX,
            20,
        )))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(end.offset, bytes.len() as u64);
    assert!(end.data.is_empty());
    std::fs::write(&first.absolute_path, b"final rewrite").unwrap();
    assert_eq!(
        content
            .unary::<contracts::content::ListExecutionFiles>(authenticated(
                contracts::content::ListExecutionFilesRequest {
                    after_path: page.next_after_path.unwrap(),
                    cursor: page.cursor,
                    ..list(1)
                }
            ))
            .await
            .unwrap_err()
            .code(),
        Code::Aborted
    );
    gate.release.notify_one();
    let final_window = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            let window = content
                .unary::<contracts::content::ReadExecutionFile>(authenticated(read(
                    "files/a.bin",
                    0,
                    128,
                )))
                .await
                .unwrap()
                .into_inner();
            if window.state == contracts::content::ExecutionFileState::Finalized {
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
        .download::<contracts::content::DownloadExecutionFile>(authenticated(
            contracts::content::DownloadExecutionFileRequest {
                execution_id: id.to_string(),
                relative_path: "files/a.bin".into(),
            },
        ))
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
        contracts::content::ExecutionFileDownloadFramePayload::Metadata(metadata) => metadata,
        payload => panic!("expected finalized metadata, got {payload:?}"),
    };
    assert_eq!(
        finalized_metadata.state,
        contracts::content::ExecutionFileState::Finalized
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
        contracts::content::ExecutionFileDownloadFramePayload::Chunk(chunk) => chunk,
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
        contracts::content::ExecutionFileDownloadFramePayload::Complete(completion) => completion,
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
    // Resume A Finished File From A Committed Prefix.
    use contracts::content::{ExecutionFileFollowFramePayload as FollowFrame, ExecutionFilePhase};
    let follow = |identity, offset| contracts::content::FollowExecutionFileRequest {
        execution_id: id.to_string(),
        relative_path: "files/a.bin".into(),
        identity,
        offset,
        max_bytes: 4,
    };
    let mut partial = content
        .download::<contracts::content::FollowExecutionFile>(authenticated(follow(None, 0)))
        .await
        .unwrap()
        .into_inner();
    let FollowFrame::Metadata(metadata) =
        partial.message().await.unwrap().unwrap().payload.unwrap()
    else {
        panic!("follow starts with original identity metadata");
    };
    let identity = metadata.identity.expect("original identity");
    assert_eq!(identity.execution_id, id.to_string());
    assert_eq!(identity.relative_path, "files/a.bin");
    assert!(identity.history.is_some());
    assert_eq!(metadata.phase, ExecutionFilePhase::Finished);
    assert_eq!(metadata.offset, 0);
    assert_eq!(metadata.available_length, 13);
    let FollowFrame::Chunk(prefix) = partial.message().await.unwrap().unwrap().payload.unwrap()
    else {
        panic!("follow yields a prefix before completion");
    };
    assert_eq!(prefix.offset, 0);
    assert_eq!(prefix.data, b"fina");
    let mut retained = prefix.data;
    let committed = retained.len() as u64;
    drop(partial);

    assert_eq!(
        content
            .download::<contracts::content::FollowExecutionFile>(authenticated(follow(
                None, committed
            )))
            .await
            .unwrap_err()
            .code(),
        Code::InvalidArgument,
    );
    let mut remapped = identity.clone();
    remapped.relative_path = "files/b.bin".into();
    assert_eq!(
        content
            .download::<contracts::content::FollowExecutionFile>(authenticated(follow(
                Some(remapped),
                committed
            )))
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition,
    );
    let mut stale = identity.clone();
    stale.file_id.push_str("-stale");
    assert_eq!(
        content
            .download::<contracts::content::FollowExecutionFile>(authenticated(follow(
                Some(stale),
                committed
            )))
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition,
    );
    assert_eq!(
        content
            .download::<contracts::content::FollowExecutionFile>(authenticated(follow(
                Some(identity.clone()),
                14
            )))
            .await
            .unwrap_err()
            .code(),
        Code::FailedPrecondition,
    );

    let mut resumed = content
        .download::<contracts::content::FollowExecutionFile>(authenticated(follow(
            Some(identity.clone()),
            committed,
        )))
        .await
        .unwrap()
        .into_inner();
    let FollowFrame::Metadata(metadata) =
        resumed.message().await.unwrap().unwrap().payload.unwrap()
    else {
        panic!("resumed stream starts with metadata");
    };
    assert_eq!(metadata.identity.as_ref(), Some(&identity));
    assert_eq!(metadata.offset, committed);
    assert_eq!(metadata.available_length, 13);
    let terminal = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match resumed
                .message()
                .await
                .unwrap()
                .expect("terminal frame before EOF")
                .payload
                .unwrap()
            {
                FollowFrame::Chunk(chunk) => {
                    assert_eq!(chunk.offset, retained.len() as u64);
                    assert!(!chunk.data.is_empty() && chunk.data.len() <= 4);
                    retained.extend(chunk.data);
                }
                FollowFrame::Terminal(terminal) => break terminal,
                FollowFrame::Metadata(_) => panic!("metadata must not repeat"),
            }
        }
    })
    .await
    .unwrap();
    assert_eq!(retained, b"final rewrite");
    assert_eq!(terminal.phase, ExecutionFilePhase::Finished);
    assert_eq!(terminal.length, retained.len() as u64);
    let sealed = terminal.sealed.expect("finished output retains its seal");
    assert_eq!(sealed.identity.as_ref(), Some(&identity));
    assert_eq!(
        sealed
            .position
            .as_ref()
            .expect("original storage position")
            .history,
        identity.history
    );
    assert_eq!(sealed.length, retained.len() as u64);
    assert_eq!(
        sealed.sha256,
        Sha256::digest(&retained)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>()
    );
    assert_eq!(sealed.sha256, finalized_completion.sha256);
    assert!(
        resumed.message().await.unwrap().is_none(),
        "terminal requires final OK status"
    );

    let empty = content
        .unary::<contracts::content::ReadExecutionFile>(authenticated(read("files/b.bin", 0, 128)))
        .await
        .unwrap()
        .into_inner();
    assert!(empty.data.is_empty());
    producer_generation_follow(&runtime).await;
    server.shutdown().await.unwrap();
    drop(runtime);
    owner.shutdown().await.unwrap();
    for change in ["truncate", "delete", "rewrite"] {
        live_follow_refusal(change).await;
    }
}

async fn producer_generation_follow(runtime: &dokime_app::RuntimeHandle) {
    use harmos::storage::{FileMetadata, FilePhase, StagedFile};
    let storage = runtime.storage();
    let metadata = FileMetadata {
        id: harmos::Identifier::new(format!("follow-generation-{}", uuid::Uuid::now_v7())).unwrap(),
        scope: None,
        media_type: "application/octet-stream".into(),
        expected_length: Some(13),
        expected_sha256: Some(Sha256::digest(b"prefix-suffix").into()),
    };
    let mut producer = storage.produce_file(metadata).await.unwrap();
    producer.write_all(b"prefix-").await.unwrap();
    let original = producer.session().expect("resumable production").clone();
    let identity = StagedFile {
        history: original.history.clone(),
        id: original.id.clone(),
    };
    let mut follower = storage.follow_file(&identity, 0).await.unwrap();
    let maximum = std::num::NonZeroUsize::new(64).unwrap();
    let prefix = follower.next_chunk(maximum).await.unwrap().unwrap();
    assert_eq!(prefix.as_bytes(), b"prefix-");
    let mut retained = prefix.as_bytes().to_vec();
    drop(prefix);
    assert_eq!(follower.offset(), 7);
    drop(producer);

    // Restart The Producer Without Replacing The Follower's Original Identity.
    let mut resumed = storage.resume_file(&original).await.unwrap();
    let generation = resumed.session().unwrap();
    assert_eq!(generation.history, identity.history);
    assert_eq!(generation.id, identity.id);
    assert_eq!(generation.generation, original.generation + 1);
    assert!(
        storage.resume_file(&original).await.is_err(),
        "stale producer generation is fenced"
    );
    resumed.write_all(b"suffix").await.unwrap();
    let sealed = resumed.finish().await.unwrap();
    let suffix = tokio::time::timeout(Duration::from_secs(5), follower.next_chunk(maximum))
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    assert_eq!(suffix.as_bytes(), b"suffix");
    retained.extend_from_slice(suffix.as_bytes());
    drop(suffix);
    assert!(follower.next_chunk(maximum).await.unwrap().is_none());
    assert_eq!(follower.progress().phase, FilePhase::Finished);
    assert_eq!(follower.offset(), 13);
    assert_eq!(sealed.position.history, identity.history);
    assert_eq!(sealed.metadata.id, identity.id);
    assert_eq!(sealed.length, retained.len() as u64);
    assert_eq!(sealed.sha256, <[u8; 32]>::from(Sha256::digest(&retained)));
    assert_eq!(retained, b"prefix-suffix");
}

async fn live_follow_refusal(change: &str) {
    use contracts::content::{ExecutionFileFollowFramePayload as Frame, ExecutionFilePhase};
    let Gated {
        root: _root,
        gate,
        mut owner,
        server,
        execution,
        cycle,
    } = Gated::open().await;
    let runtime = owner.handle();
    execution
        .unary::<contracts::execution::RunCycle>(authenticated(run(&cycle)))
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(5), gate.started.notified())
        .await
        .unwrap();
    let executions = execution
        .unary::<contracts::execution::ListExecutions>(authenticated(
            contracts::execution::ListExecutionsRequest {
                cycle_id: cycle,
                ..Default::default()
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let id =
        dokime_app::core::TestExecutionId::from_uuid(executions.executions[0].execution_id.clone())
            .unwrap();
    let target = runtime
        .execution_file_target(&id, "follow.bin".into(), false, true)
        .await
        .unwrap()
        .unwrap();
    let original = b"original retained prefix";
    std::fs::write(&target.absolute_path, original).unwrap();
    let request = |identity, offset| contracts::content::FollowExecutionFileRequest {
        execution_id: id.to_string(),
        relative_path: "follow.bin".into(),
        identity,
        offset,
        max_bytes: 4,
    };
    let content = WireClient::connect(server.local_endpoint()).await.unwrap();
    let mut first = content
        .download::<contracts::content::FollowExecutionFile>(authenticated(request(None, 0)))
        .await
        .unwrap()
        .into_inner();
    let Frame::Metadata(metadata) = first.message().await.unwrap().unwrap().payload.unwrap() else {
        panic!("live follow starts with metadata");
    };
    assert_eq!(metadata.phase, ExecutionFilePhase::Producing);
    let identity = metadata.identity.unwrap();
    let Frame::Chunk(chunk) = first.message().await.unwrap().unwrap().payload.unwrap() else {
        panic!("live follow yields retained bytes");
    };
    assert_eq!(chunk.offset, 0);
    assert_eq!(chunk.data, original[..4]);
    drop(first);
    drop(content);
    server.shutdown().await.unwrap();

    // A New Listener Must Reuse The Runtime's Original Production.
    let server = Server::start(
        runtime.clone(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Read)])
            .unwrap(),
    )
    .await
    .unwrap();
    let content = WireClient::connect(server.local_endpoint()).await.unwrap();
    let mut resumed = content
        .download::<contracts::content::FollowExecutionFile>(authenticated(request(
            Some(identity.clone()),
            4,
        )))
        .await
        .unwrap()
        .into_inner();
    let Frame::Metadata(metadata) = resumed.message().await.unwrap().unwrap().payload.unwrap()
    else {
        panic!("resumed live follow starts with metadata");
    };
    assert_eq!(metadata.identity.as_ref(), Some(&identity));
    assert_eq!(metadata.offset, 4);
    let Frame::Chunk(chunk) = resumed.message().await.unwrap().unwrap().payload.unwrap() else {
        panic!("resumed live follow continues the retained prefix");
    };
    assert_eq!(chunk.offset, 4);
    assert_eq!(chunk.data, original[4..8]);
    drop(resumed);

    match change {
        "truncate" => std::fs::OpenOptions::new()
            .write(true)
            .open(&target.absolute_path)
            .unwrap()
            .set_len(2)
            .unwrap(),
        "delete" => std::fs::remove_file(&target.absolute_path).unwrap(),
        "rewrite" => {
            // Growing The Rewritten Source Makes Change Detection Independent Of Timestamp Resolution.
            std::fs::write(
                &target.absolute_path,
                b"changed retained prefix with extra bytes",
            )
            .unwrap();
        }
        _ => unreachable!(),
    }
    let refusal = content
        .download::<contracts::content::FollowExecutionFile>(authenticated(request(
            Some(identity.clone()),
            8,
        )))
        .await
        .unwrap_err();
    assert_eq!(
        refusal.code(),
        Code::FailedPrecondition,
        "{change}: {refusal}"
    );
    gate.release.notify_one();

    // Failed Capture Retains Its Original Bytes Without Claiming A Finished Seal.
    let mut abandoned = tokio::time::timeout(Duration::from_secs(10), async {
        loop {
            match content
                .download::<contracts::content::FollowExecutionFile>(authenticated(request(
                    Some(identity.clone()),
                    0,
                )))
                .await
            {
                Ok(stream) => break stream.into_inner(),
                Err(error) => {
                    assert_eq!(error.code(), Code::FailedPrecondition, "{change}: {error}");
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
        }
    })
    .await
    .expect("failed source is captured as an interrupted original output");
    let Frame::Metadata(metadata) = abandoned.message().await.unwrap().unwrap().payload.unwrap()
    else {
        panic!("abandoned follow starts with metadata");
    };
    assert_eq!(metadata.identity.as_ref(), Some(&identity));
    assert_eq!(metadata.phase, ExecutionFilePhase::Abandoned);
    let mut retained = Vec::new();
    let terminal = tokio::time::timeout(Duration::from_secs(5), async {
        loop {
            match abandoned.message().await.unwrap().unwrap().payload.unwrap() {
                Frame::Chunk(chunk) => {
                    assert_eq!(chunk.offset, retained.len() as u64);
                    retained.extend(chunk.data);
                }
                Frame::Terminal(terminal) => break terminal,
                Frame::Metadata(_) => panic!("metadata must not repeat"),
            }
        }
    })
    .await
    .unwrap();
    assert_eq!(retained, original);
    assert_eq!(terminal.phase, ExecutionFilePhase::Abandoned);
    assert_eq!(terminal.length, original.len() as u64);
    assert!(terminal.sealed.is_none());
    assert!(abandoned.message().await.unwrap().is_none());
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
        execution,
        cycle,
    } = Gated::open().await;
    let runtime = owner.handle();
    execution
        .unary::<contracts::execution::RunCycle>(authenticated(run(&cycle)))
        .await
        .unwrap();
    tokio::time::timeout(Duration::from_secs(5), gate.started.notified())
        .await
        .unwrap();
    let executions = execution
        .unary::<contracts::execution::ListExecutions>(authenticated(
            contracts::execution::ListExecutionsRequest {
                cycle_id: cycle.clone(),
                ..Default::default()
            },
        ))
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
    let content = WireClient::connect(server.local_endpoint()).await.unwrap();
    let mut download = content
        .download::<contracts::content::DownloadExecutionFile>(authenticated(
            contracts::content::DownloadExecutionFileRequest {
                execution_id: id.to_string(),
                relative_path: "large.bin".into(),
            },
        ))
        .await
        .unwrap()
        .into_inner();
    let metadata = match download.message().await.unwrap().unwrap().payload.unwrap() {
        contracts::content::ExecutionFileDownloadFramePayload::Metadata(metadata) => metadata,
        payload => panic!("expected execution file metadata, got {payload:?}"),
    };
    assert_eq!(metadata.observed_size_bytes, size_bytes);
    let mut downloaded = 0_u64;
    let mut digest = Sha256::new();
    let completion = loop {
        let frame = download.message().await.unwrap().unwrap();
        match frame.payload.unwrap() {
            contracts::content::ExecutionFileDownloadFramePayload::Chunk(chunk) => {
                assert_eq!(chunk.offset, downloaded);
                assert!(!chunk.data.is_empty());
                assert!(chunk.data.len() as u64 <= 1024 * 1024);
                downloaded += chunk.data.len() as u64;
                digest.update(&chunk.data);
            }
            contracts::content::ExecutionFileDownloadFramePayload::Complete(completion) => {
                break completion;
            }
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
