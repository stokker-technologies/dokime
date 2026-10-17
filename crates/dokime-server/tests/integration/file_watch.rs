/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// File watches against a running execution over the listener: a bounded
// watch streams progress and seals, and a cancelled watch still seals.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::Arc;

// Third Party Imports

// Local Imports
use dokime_domain::model;
use super::content::UploadGate;
use super::*;
use {
    model::files::wire::FileRule,
    model::files::wire::FileRuleAction,
    model::files::wire::FileWatchFrame,
    model::files::wire::FileWatchFramePayload,
    model::files::wire::FileWatchPhase,
    model::files::wire::FileWatchPolicy,
    contracts::files::FileWatch,
    model::files::wire::FileWatchSource,
    model::files::wire::FileWatchSourcePayload,
    model::files::wire::LocalWatchSource,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */
#[tokio::test]
async fn file_watch_artifacts() {
    let root = tempfile::tempdir().unwrap();
    let gate = Arc::new(UploadGate::default());
    let mut owner = Runtime::with_execution_fixture(
        crate::support::budgets::runtime_config(
            root.path(),
            support::metadata("grpc.files", "Files"),
        ),
        "fixture",
        gate.clone(),
    )
    .await
    .unwrap();
    support::register_fixture_execution(&owner.handle(), support::fixture_suite(&["passed"])).await;
    let server = Server::start(
        owner.handle(),
        Config::local()
            .with_bearer_auth(vec![grant(Role::Control)])
            .unwrap()
            .with_max_message_bytes(1024 * 1024),
    )
    .await
    .unwrap();
    let client = connect(&server).await;
    let scope = super::media::running_execution(&owner, &client, &gate).await;
    let execution = scope.execution_id.clone().unwrap();
    let tree = tempfile::tempdir().unwrap();
    let log = tree.path().join("logs/charger.log");
    std::fs::create_dir_all(log.parent().unwrap()).unwrap();
    std::fs::write(&log, b"before the watch\n").unwrap();
    let request = |output: &str, maximum_seconds: Option<u32>| FileWatch {
        execution_id: Some(execution.clone()),
        source: Some(FileWatchSource {
            payload: Some(FileWatchSourcePayload::Local(LocalWatchSource {
                root: tree.path().to_str().unwrap().to_owned(),
            })),
        }),
        rules: vec![FileRule {
            glob: "**/*.log".into(),
            action: FileRuleAction::Include,
        }],
        output: output.to_owned(),
        policy: FileWatchPolicy::Stream,
        poll_interval_ms: Some(50),
        maximum_seconds,
        maximum_files: None,
        maximum_bytes: None,
    };

    // A Watch Without An Execution Is Refused Before Any Source Opens
    let unscoped = watch(
        &client,
        FileWatch {
            execution_id: None,
            ..request("capture", Some(1))
        },
    )
    .await
    .unwrap_err();
    assert_eq!(unscoped.code(), Code::InvalidArgument, "{unscoped:?}");

    // A Bounded Watch Reports Progress, Seals, Then Completes
    let mut frames = watch(&client, request("capture", Some(2))).await.unwrap();
    let baseline = next(&mut frames).await;
    assert!(
        matches!(
            &baseline.payload,
            Some(FileWatchFramePayload::Progress(progress))
                if progress.path == "logs/charger.log" && progress.phase == FileWatchPhase::Baseline
        ),
        "{baseline:?}"
    );
    append(&log, b"line 1\n");
    let mut captured = false;
    let mut sealed = None;
    let completed = loop {
        match next(&mut frames).await.payload {
            Some(FileWatchFramePayload::Progress(progress)) => {
                captured |= progress.phase == FileWatchPhase::Capturing;
            }
            Some(FileWatchFramePayload::Sealed(frame)) => sealed = frame.artifact,
            Some(FileWatchFramePayload::Completed(completed)) => break completed,
            other => panic!("unexpected watch frame {other:?}"),
        }
    };
    assert!(captured, "appended bytes are reported before sealing");
    let sealed = sealed.expect("the watched file is sealed");
    assert_eq!(sealed.filename, "capture/logs/charger.log");
    assert_eq!(sealed.size_bytes, 7);
    assert_eq!(sealed.state, model::content::wire::ArtifactState::Complete);
    assert_eq!(completed.artifacts, vec![sealed]);
    assert!(frames.message().await.unwrap().is_none());

    // A Cancelled Watch Still Seals What It Captured
    let mut frames = watch(&client, request("cancelled", None)).await.unwrap();
    next(&mut frames).await;
    append(&log, b"line 2\n");
    loop {
        if let Some(FileWatchFramePayload::Progress(progress)) = next(&mut frames).await.payload
            && progress.phase == FileWatchPhase::Capturing
        {
            break;
        }
    }
    drop(frames);
    let stored = support::on_state_change(&owner.handle(), async || {
        let found = owner
            .handle()
            .artifacts()
            .await
            .expect("artifact query succeeds")
            .iter()
            .find(|record| record.filename == "cancelled/logs/charger.log")
            .cloned();
        if let Some(record) = found {
            return Some(record);
        }
        None
    })
    .await;
    assert_eq!(
        stored.status,
        dokime_runtime::artifacts::ArtifactStatus::Complete
    );
    assert_eq!(stored.size_bytes, 7);

    gate.release.notify_one();
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

/* === Private Functions === */
async fn watch(
    client: &WireClient,
    request: FileWatch,
) -> Result<WireStream<FileWatchFrame>, tonic::Status> {
    let mut frames = client
        .download::<contracts::files::FileWatch>(authenticated(request))
        .await?
        .into_inner();

    // A Refusal Before Watching Arrives As The First Frame
    use futures_util::StreamExt;
    let first = frames.message().await?;
    let frames = futures_util::stream::iter(first.map(Ok)).chain(frames.frames);
    Ok(WireStream {
        frames: Box::pin(frames),
    })
}

async fn next(frames: &mut WireStream<FileWatchFrame>) -> FileWatchFrame {
    frames
        .message()
        .await
        .unwrap()
        .expect("the watch stream continues")
}

fn append(path: &std::path::Path, bytes: &[u8]) {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new().append(true).open(path).unwrap();
    file.write_all(bytes).unwrap();
}

/* === Tests === */
