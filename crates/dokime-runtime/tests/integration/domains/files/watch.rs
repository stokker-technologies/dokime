/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Stream, Mirror and Snapshot watches over a local tree that grows, rotates
// and gains files while the watch runs.

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::time::SystemTime;

// Third Party Imports

// Local Imports
use super::*;
use dokime_domain::model;
use dokime_operations::files::FileWatch;
use dokime_runtime::artifacts::ArtifactStatus;
use dokime_runtime::domains::evidence::RecordSelection;
use dokime_runtime::domains::files::{CAPPED, GapReason, INVALID, Phase, UNSUPPORTED, UNTRUSTED};
use dokime_runtime::runtime::Error;
use sha2::Digest;
use {
    dokime_operations::files::WatchFile, model::files::wire::FileWatchPolicy, model::files::wire::FileWatchSourcePayload,
    model::files::wire::FtpWatchSource, model::files::wire::LineField, model::files::wire::LineFieldKind, model::files::wire::LineMarker,
    model::files::wire::LineMatcher, model::files::wire::WatchedArtifact,
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
// A Host-Routed Watch Carries A Ticket In Its Call Context; Its Readiness Request Repeats It.
fn ticketed(bench: &Bench) -> (harmos::operations::OperationManager<dokime_runtime::app::Lanes>, u64) {
    let ticket = dokime_runtime::execution::WatchTicket::issue();
    let mut context = harmos::operations::CallContext::default();
    context.insert_portable(ticket.clone()).unwrap();
    (bench.handle.operations().with_context(context), ticket.ticket)
}

fn gaps(events: &[WatchEvent]) -> Vec<(String, u64, GapReason)> {
    events
        .iter()
        .filter_map(|event| match event {
            WatchEvent::Gap { path, offset, reason } => Some((path.clone(), *offset, *reason)),
            _ => None,
        })
        .collect()
}

fn phases(events: &[WatchEvent], wanted: Phase) -> Vec<(String, u64)> {
    events
        .iter()
        .filter_map(|event| match event {
            WatchEvent::Progress { path, offset, phase } if *phase == wanted => Some((path.clone(), *offset)),
            _ => None,
        })
        .collect()
}

fn code(error: &Error) -> &str {
    error.code().unwrap_or_default()
}

/* === Tests === */

#[dokime_tracing::trace_test]
#[tokio::test]
async fn many_file_stop() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    for index in 0..64 {
        write(&tree.path().join(format!("nested/{index}.log")), b"baseline\n");
    }
    let (operations, ticket) = ticketed(&bench);
    let watch = operations
        .submit::<WatchFile>(WatchFile {
            recursive: true,
            rules: vec![include("**/*.log")],
            maximum_files: Some(64),
            execution_id: Some(bench.execution.clone()),
            source: Some(FileWatchSource { payload: Some(local(tree.path())) }),
            output: "many".into(),
            artifact: None,
            policy: FileWatchPolicy::Mirror,
            selection: model::files::wire::FileWatchSelection::All,
            wait_for_source_seconds: None,
            poll_interval_ms: Some(60_000),
            maximum_seconds: None,
            maximum_bytes: None,
            matchers: Vec::new(),
            markers: Vec::new(),
        })
        .unwrap();
    operations
        .operation::<dokime_operations::files::AwaitFileWatchArmed>(dokime_operations::files::AwaitFileWatchArmed { instance_id: ticket })
        .await
        .unwrap();
    for index in 0..64 {
        append(&tree.path().join(format!("nested/{index}.log")), b"changed\n");
    }
    let started = std::time::Instant::now();
    let outcome = watch.cancel().await.unwrap().into_owned();
    eprintln!("many_file_stop: cancellation and sealing {:?}", started.elapsed());
    assert_eq!(outcome.artifacts.len(), 64);
    let execution = TestExecutionId::from_uuid(bench.execution.clone()).unwrap();
    bench.handle.stop_execution(execution.clone(), CommandContext::default()).await.unwrap();
    let artifacts = bench.handle.artifacts().await.unwrap();
    for index in 0..64 {
        let filename = format!("many/nested/{index}.log");
        let artifact = artifacts.iter().find(|record| record.filename == filename).unwrap();
        assert_eq!(artifact.scope.test_execution_id.as_ref(), Some(&execution));
        assert_eq!(artifact.status, ArtifactStatus::Complete);
        let content = bench.handle.read_artifact_content(&artifact.artifact_id, 1024).await.unwrap().unwrap();
        assert_eq!(content.bytes, b"baseline\nchanged\n");
    }
    bench.stop().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn failed_initial_admission_never_reports_armed() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    write(&tree.path().join("one.log"), b"one\n");
    write(&tree.path().join("two.log"), b"two\n");
    let request = FileWatch {
        maximum_files: Some(1),
        ..request(&bench.execution, local(tree.path()), vec![include("*.log")], FileWatchPolicy::Stream)
    };
    let mut watch = bench.handle.open_file_watch(&request, None).await.unwrap();
    let mut events = Vec::new();
    assert!(watch.poll(&mut events).await.is_err());
    assert!(!events.iter().any(|event| matches!(event, WatchEvent::Armed { .. })));
    seal(&mut watch).await;

    let single = WatchFile {
        recursive: false,
        rules: Vec::new(),
        maximum_files: None,
        execution_id: Some(bench.execution.clone()),
        source: Some(FileWatchSource {
            payload: Some(local(&tree.path().join("one.log"))),
        }),
        output: "failed".into(),
        artifact: None,
        policy: FileWatchPolicy::Mirror,
        selection: model::files::wire::FileWatchSelection::All,
        wait_for_source_seconds: None,
        poll_interval_ms: None,
        maximum_seconds: None,
        maximum_bytes: Some(1),
        matchers: Vec::new(),
        markers: Vec::new(),
    };
    let (operations, _) = ticketed(&bench);
    let instance = operations.submit::<WatchFile>(single).unwrap();
    assert!(instance.join().await.is_err());
    bench.stop().await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn source_wait_does_not_consume_the_capture_budget() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    let absent = tree.path().join("absent");
    let request = FileWatch {
        maximum_seconds: Some(2),
        wait_for_source_seconds: Some(10),
        ..request(&bench.execution, local(&absent), vec![include("*.log")], FileWatchPolicy::Stream)
    };
    let mut watch = bench.handle.open_file_watch(&request, None).await.unwrap();
    assert!(poll(&mut watch).await.is_empty());
    assert!(watch.remaining().is_none());
    tokio::time::pause();
    tokio::time::advance(std::time::Duration::from_secs(6)).await;
    write(&absent.join("appeared.log"), b"baseline\n");
    let armed = poll(&mut watch).await;
    assert!(armed.iter().any(|event| matches!(event, WatchEvent::Armed { .. })));
    assert_eq!(watch.remaining(), Some(std::time::Duration::from_secs(2)));
    tokio::time::advance(std::time::Duration::from_secs(1)).await;
    assert_eq!(watch.remaining(), Some(std::time::Duration::from_secs(1)));
    tokio::time::resume();
    seal(&mut watch).await;
    bench.stop().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn empty_baseline_arms_before_the_producer() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    let request = FileWatch {
        wait_for_source_seconds: Some(30),
        ..request(&bench.execution, local(tree.path()), vec![include("*.log")], FileWatchPolicy::Stream)
    };
    let mut watch = bench.handle.open_file_watch(&request, None).await.unwrap();
    let first = poll(&mut watch).await;
    assert!(matches!(first.as_slice(), [WatchEvent::Armed { baseline }] if baseline.is_empty()));

    write(&tree.path().join("device.log"), b"first reading\n");
    let captured = seal(&mut watch).await;
    assert_artifact(&sealed(&captured)["device.log"], "device.log", b"first reading\n");
    bench.stop().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn latest_selects_by_mtime_then_path_and_follows_rotation() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    for (path, seconds) in [("old.log", 1), ("a.log", 2), ("b.log", 2)] {
        let path = tree.path().join(path);
        write(&path, b"baseline\n");
        let file = std::fs::File::options().write(true).open(path).unwrap();
        file.set_modified(SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(seconds)).unwrap();
    }
    let request = FileWatch {
        selection: model::files::wire::FileWatchSelection::Latest,
        ..request(&bench.execution, local(tree.path()), vec![include("*.log")], FileWatchPolicy::Stream)
    };
    let mut watch = bench.handle.open_file_watch(&request, None).await.unwrap();
    let first = poll(&mut watch).await;
    assert_eq!(phases(&first, Phase::Baseline), [("a.log".to_owned(), 0)]);
    append(&tree.path().join("a.log"), b"one\n");
    poll(&mut watch).await;
    write(&tree.path().join("later.log"), b"do not switch\n");
    std::fs::rename(tree.path().join("a.log"), tree.path().join("a.rotated")).unwrap();
    write(&tree.path().join("a.log"), b"two\n");
    let rotation = poll(&mut watch).await;
    assert_artifact(&sealed(&rotation)["a.log"], "a.log", b"one\n");
    let final_pass = seal(&mut watch).await;
    assert_artifact(&sealed(&final_pass)["a.log"], "a.segment2.log", b"two\n");
    assert_eq!(watch.sealed().len(), 2);
    bench.stop().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn new_ignores_baseline_and_captures_first_discoveries_from_zero() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    write(&tree.path().join("existing.log"), b"before\n");
    let request = FileWatch {
        selection: model::files::wire::FileWatchSelection::New,
        ..request(&bench.execution, local(tree.path()), vec![include("*.log")], FileWatchPolicy::Stream)
    };
    let mut watch = bench.handle.open_file_watch(&request, None).await.unwrap();
    let first = poll(&mut watch).await;
    assert!(matches!(first.as_slice(), [WatchEvent::Armed { baseline }] if baseline == &["existing.log"]));
    append(&tree.path().join("existing.log"), b"ignored\n");
    write(&tree.path().join("new.log"), b"capture from zero\n");
    let final_pass = seal(&mut watch).await;
    let artifacts = sealed(&final_pass);
    assert_eq!(artifacts.len(), 1);
    assert_artifact(&artifacts["new.log"], "new.log", b"capture from zero\n");
    bench.stop().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn new_with_empty_accessible_root_arms_despite_opt_in_source_wait() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    let request = FileWatch {
        selection: model::files::wire::FileWatchSelection::New,
        wait_for_source_seconds: Some(30),
        ..request(&bench.execution, local(tree.path()), vec![include("*.log")], FileWatchPolicy::Stream)
    };
    let mut watch = bench.handle.open_file_watch(&request, None).await.unwrap();
    let armed = poll(&mut watch).await;
    assert!(matches!(armed.as_slice(), [WatchEvent::Armed { baseline }] if baseline.is_empty()));
    write(&tree.path().join("new.log"), b"first bytes\n");
    let final_pass = seal(&mut watch).await;
    assert_artifact(&sealed(&final_pass)["new.log"], "new.log", b"first bytes\n");
    bench.stop().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn latest_empty_baseline_is_fixed_unless_wait_was_requested() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    let request = FileWatch {
        selection: model::files::wire::FileWatchSelection::Latest,
        ..request(&bench.execution, local(tree.path()), vec![include("*.log")], FileWatchPolicy::Stream)
    };
    let mut empty = bench.handle.open_file_watch(&request, None).await.unwrap();
    let armed = poll(&mut empty).await;
    assert!(matches!(armed.as_slice(), [WatchEvent::Armed { baseline }] if baseline.is_empty()));
    let mut waiting = bench
        .handle
        .open_file_watch(
            &FileWatch {
                wait_for_source_seconds: Some(30),
                ..request
            },
            None,
        )
        .await
        .unwrap();
    assert!(poll(&mut waiting).await.is_empty());
    write(&tree.path().join("appeared.log"), b"baseline\n");
    let armed = poll(&mut waiting).await;
    assert!(armed.iter().any(|event| matches!(event, WatchEvent::Armed { baseline } if baseline == &["appeared.log"])));
    append(&tree.path().join("appeared.log"), b"after armed\n");
    seal(&mut empty).await;
    assert!(empty.sealed().is_empty(), "Latest does not switch from its empty baseline to a later arrival");
    let final_pass = seal(&mut waiting).await;
    assert_artifact(&sealed(&final_pass)["appeared.log"], "appeared.log", b"after armed\n");
    bench.stop().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn source_wait_rejects_invalid_paths_and_permission_failures() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    let file = tree.path().join("source.log");
    write(&file, b"retained\n");
    for root in [std::path::Path::new("relative"), file.as_path()] {
        let request = FileWatch {
            wait_for_source_seconds: Some(30),
            ..request(&bench.execution, local(root), vec![include("*.log")], FileWatchPolicy::Stream)
        };
        let error = bench.handle.open_file_watch(&request, None).await.err().unwrap();
        assert_eq!(code(&error), INVALID);
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let request = FileWatch {
            wait_for_source_seconds: Some(30),
            ..request(&bench.execution, local(tree.path()), vec![include("*.log")], FileWatchPolicy::Stream)
        };
        let mut watch = bench.handle.open_file_watch(&request, None).await.unwrap();
        let metadata = std::fs::metadata(&file).unwrap();
        let permissions = metadata.permissions();
        std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o000)).unwrap();
        let inaccessible = std::fs::File::open(&file).is_err();

        let mut events = Vec::new();
        let result = watch.poll(&mut events).await;
        std::fs::set_permissions(&file, permissions).unwrap();
        assert!(inaccessible, "the permission proof requires an unprivileged test process");
        assert_eq!(code(&result.unwrap_err()), INVALID);
        assert!(!events.iter().any(|event| matches!(event, WatchEvent::Armed { .. })));
        seal(&mut watch).await;
    }

    bench.stop().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn vanished_paths_release_slots_before_new_paths_are_admitted() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    write(&tree.path().join("before.log"), b"baseline\n");
    let request = FileWatch {
        maximum_files: Some(1),
        ..request(&bench.execution, local(tree.path()), vec![include("*.log")], FileWatchPolicy::Stream)
    };
    let mut watch = bench.handle.open_file_watch(&request, None).await.unwrap();
    poll(&mut watch).await;
    append(&tree.path().join("before.log"), b"before\n");
    poll(&mut watch).await;
    std::fs::remove_file(tree.path().join("before.log")).unwrap();
    write(&tree.path().join("after.log"), b"after\n");
    let pass = poll(&mut watch).await;
    assert_artifact(&sealed(&pass)["before.log"], "before.log", b"before\n");
    let final_pass = seal(&mut watch).await;
    assert_artifact(&sealed(&final_pass)["after.log"], "after.log", b"after\n");
    bench.stop().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn single_file_retains_renamed_and_copy_truncated_segments_with_one_slot() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    let log = tree.path().join("device.log");
    write(&log, b"before\n");
    let request = WatchFile {
        recursive: false,
        rules: Vec::new(),
        maximum_files: None,
        execution_id: Some(bench.execution.clone()),
        source: Some(FileWatchSource { payload: Some(local(&log)) }),
        output: OUTPUT.to_owned(),
        artifact: Some(format!("{OUTPUT}/named.log")),
        policy: FileWatchPolicy::Stream,
        selection: model::files::wire::FileWatchSelection::All,
        wait_for_source_seconds: None,
        poll_interval_ms: None,
        maximum_seconds: None,
        maximum_bytes: None,
        matchers: Vec::new(),
        markers: Vec::new(),
    };
    let mut watch = bench.handle.open_watch_file(&request, None).await.unwrap();
    poll(&mut watch).await;
    append(&log, b"one\n");
    poll(&mut watch).await;
    for segment in 2..=4 {
        std::fs::rename(&log, tree.path().join(format!("rotated-{segment}"))).unwrap();
        write(&log, format!("segment {segment}\n").as_bytes());
        poll(&mut watch).await;
    }
    write(&log, b"x\n");
    let truncated = poll(&mut watch).await;
    assert!(gaps(&truncated).iter().any(|(_, _, reason)| *reason == GapReason::Truncated));
    seal(&mut watch).await;
    let artifacts = watch.sealed();
    assert_eq!(artifacts.len(), 5);
    let names = artifacts.iter().map(|record| record.filename.as_str()).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(names.len(), 5);
    for (index, record) in artifacts.iter().enumerate() {
        let bytes = bench.handle.read_artifact_content(&record.artifact_id, 100).await.unwrap().unwrap().bytes;
        let expected = match index {
            0 => b"one\n".to_vec(),
            4 => b"x\n".to_vec(),
            _ => format!("segment {}\n", index + 1).into_bytes(),
        };
        assert_eq!(bytes, expected);
    }
    bench.stop().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn missing_source_waits_only_when_requested_and_can_seal_before_arming() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    let absent = tree.path().join("absent");
    let base = request(&bench.execution, local(&absent), vec![include("*.log")], FileWatchPolicy::Stream);
    assert!(bench.handle.open_file_watch(&base, None).await.is_err());
    let request = FileWatch {
        wait_for_source_seconds: Some(30),
        ..base
    };
    let mut watch = bench.handle.open_file_watch(&request, None).await.unwrap();
    assert!(poll(&mut watch).await.is_empty());
    assert!(seal(&mut watch).await.is_empty());
    assert!(watch.is_complete());

    let mut watch = bench.handle.open_file_watch(&request, None).await.unwrap();
    assert!(poll(&mut watch).await.is_empty());
    write(&absent.join("appeared.log"), b"baseline\n");
    let armed = poll(&mut watch).await;
    assert!(armed.iter().any(|event| matches!(event, WatchEvent::Armed { baseline } if baseline == &["appeared.log"])));
    append(&absent.join("appeared.log"), b"captured\n");
    let final_pass = seal(&mut watch).await;
    assert_artifact(&sealed(&final_pass)["appeared.log"], "appeared.log", b"captured\n");
    bench.stop().await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn late_source_cannot_arm_after_the_wait_deadline() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    let absent = tree.path().join("absent");
    let request = FileWatch {
        wait_for_source_seconds: Some(1),
        ..request(&bench.execution, local(&absent), vec![include("*.log")], FileWatchPolicy::Stream)
    };
    let mut watch = bench.handle.open_file_watch(&request, None).await.unwrap();
    assert!(poll(&mut watch).await.is_empty());
    tokio::time::pause();
    tokio::time::advance(std::time::Duration::from_secs(2)).await;
    write(&absent.join("late.log"), b"too late\n");
    let mut events = Vec::new();
    let error = watch.poll(&mut events).await.unwrap_err();
    assert!(error.to_string().contains("deadline"));
    assert!(events.is_empty());
    seal(&mut watch).await;
    tokio::time::resume();
    bench.stop().await;
}

#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn cancelled_source_wait() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    let request = WatchFile {
        recursive: false,
        rules: Vec::new(),
        maximum_files: None,
        execution_id: Some(bench.execution.clone()),
        source: Some(FileWatchSource {
            payload: Some(local(&tree.path().join("missing/device.log"))),
        }),
        output: OUTPUT.to_owned(),
        artifact: None,
        policy: FileWatchPolicy::Stream,
        selection: model::files::wire::FileWatchSelection::All,
        wait_for_source_seconds: Some(30),
        poll_interval_ms: None,
        maximum_seconds: None,
        maximum_bytes: None,
        matchers: Vec::new(),
        markers: Vec::new(),
    };
    let (operations, _) = ticketed(&bench);
    let watch = operations.submit::<WatchFile>(request).unwrap();
    watch.ready().await.unwrap();
    watch.request_cancel();
    let failure = watch.join().await.expect_err("the missing-source watch settles as cancelled");
    let cancelled = matches!(&failure, harmos::Error::OperationRefused(harmos::Refusal::Cancelled))
        || failure.failure().is_some_and(|cause| cause.code() == harmos::operations::FailureKind::Cancelled);
    assert!(cancelled, "the missing-source watch retains a cancellation outcome: {failure:?}");
    bench.stop().await;
}
// Stream captures only bytes written after the start, follows rotation and
// truncation with gaps, picks up new files, and seals vanished ones.
#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn stream_local_tree() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    write(&tree.path().join("logs/charger.log"), b"before the watch\n");
    write(&tree.path().join("logs/debug/trace.log"), b"excluded\n");
    write(&tree.path().join("logs/debug/bms.log"), b"re-included\n");
    write(&tree.path().join("notes.txt"), b"unmatched\n");
    let mut watch = bench
        .handle
        .open_file_watch(
            &request(
                &bench.execution,
                local(tree.path()),
                vec![include("**/*.log"), exclude("**/debug/**"), include("**/debug/bms.log")],
                FileWatchPolicy::Stream,
            ),
            None,
        )
        .await
        .unwrap();

    // Files Present at the Start Baseline at Their Length
    let first = poll(&mut watch).await;
    assert_eq!(phases(&first, Phase::Baseline), vec![("logs/charger.log".to_owned(), 0), ("logs/debug/bms.log".to_owned(), 0)]);
    assert!(phases(&first, Phase::Capturing).is_empty());

    // Appended Bytes Are Captured; A New File From Its First Byte
    append(&tree.path().join("logs/charger.log"), b"line 1\n");
    write(&tree.path().join("logs/bench/inverter.log"), b"born\n");
    let grown = poll(&mut watch).await;
    assert_eq!(phases(&grown, Phase::Capturing), vec![("logs/bench/inverter.log".to_owned(), 5), ("logs/charger.log".to_owned(), 7)]);

    // Truncation Restarts At The First Byte; Rotation Seals The Segment
    std::fs::rename(tree.path().join("logs/charger.log"), tree.path().join("logs/charger.log.1")).unwrap();
    write(&tree.path().join("logs/charger.log"), b"rotated\n");
    write(&tree.path().join("logs/bench/inverter.log"), b"x\n");
    let rotated = poll(&mut watch).await;
    assert_eq!(
        gaps(&rotated),
        vec![("logs/bench/inverter.log".to_owned(), 5, GapReason::Truncated), ("logs/charger.log".to_owned(), 7, GapReason::Replaced)]
    );
    let segment = sealed(&rotated);
    assert_artifact(&segment["logs/charger.log"], "logs/charger.log", b"line 1\n");

    // A Vanished File Is Sealed with a Gap While the Watch Continues
    append(&tree.path().join("logs/debug/bms.log"), b"cell 3\n");
    poll(&mut watch).await;
    std::fs::remove_file(tree.path().join("logs/debug/bms.log")).unwrap();
    let vanished = poll(&mut watch).await;
    assert_eq!(gaps(&vanished), vec![("logs/debug/bms.log".to_owned(), 7, GapReason::Vanished)]);
    let early = sealed(&vanished);
    assert_artifact(&early["logs/debug/bms.log"], "logs/debug/bms.log", b"cell 3\n");

    // Sealing Captures the Last Bytes of Every Remaining File
    append(&tree.path().join("logs/charger.log"), b"tail\n");
    let last = seal(&mut watch).await;
    let sealed = sealed(&last);
    assert_eq!(sealed.len(), 2);
    assert_artifact(&sealed["logs/charger.log"], "logs/charger.segment2.log", b"rotated\ntail\n");
    assert_artifact(&sealed["logs/bench/inverter.log"], "logs/bench/inverter.segment2.log", b"x\n");
    assert_eq!(sealed["logs/charger.log"].content_type.as_deref(), Some("text/plain"));
    assert_eq!(sealed["logs/charger.log"].scope.test_execution_id.as_ref().map(ToString::to_string), Some(bench.execution.clone()));
    assert!(watch.is_complete());
    assert_eq!(watch.sealed().len(), 5);
    assert!(seal(&mut watch).await.is_empty(), "sealing is idempotent");

    drop(watch);
    bench.stop().await;
}

// Mirror keeps each output equal to its source, refetching on a size or
// modification-time change.
#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn mirror_local_tree() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    let report = tree.path().join("report.csv");
    write(&report, b"a,b\n1,2\n");
    let mut watch = bench
        .handle
        .open_file_watch(&request(&bench.execution, local(tree.path()), vec![include("**/*.csv")], FileWatchPolicy::Mirror), None)
        .await
        .unwrap();

    // The Whole File Is Mirrored at Discovery, Then Only on Change
    let first = poll(&mut watch).await;
    assert_eq!(phases(&first, Phase::Synced), vec![("report.csv".to_owned(), 8)]);
    assert!(phases(&poll(&mut watch).await, Phase::Synced).is_empty());

    // A Rewrite of Equal Length Is Detected by Its Modification Time
    write(&report, b"a,b\n3,4\n");
    let later = SystemTime::now() + Duration::from_secs(5);
    std::fs::File::options().write(true).open(&report).unwrap().set_modified(later).unwrap();
    write(&tree.path().join("runs/second.csv"), b"x\n");
    let changed = poll(&mut watch).await;
    assert_eq!(phases(&changed, Phase::Synced), vec![("report.csv".to_owned(), 8), ("runs/second.csv".to_owned(), 2)]);

    // A Shrunk Source Retains Its Previous Segment Before Mirroring a Fresh One
    write(&report, b"a\n");
    let shrunk = poll(&mut watch).await;
    assert_eq!(gaps(&shrunk), [("report.csv".to_owned(), 8, GapReason::Truncated)]);
    assert_artifact(&sealed(&shrunk)["report.csv"], "report.csv", b"a,b\n3,4\n");

    let sealed = sealed(&seal(&mut watch).await);
    assert_artifact(&sealed["report.csv"], "report.segment2.csv", b"a\n");
    assert_artifact(&sealed["runs/second.csv"], "runs/second.csv", b"x\n");

    drop(watch);
    bench.stop().await;
}

// A snapshot copies each file once: at the start, or at the stop.
#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn snapshot_local_tree() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    let log = tree.path().join("bench.log");
    write(&log, b"start\n");

    // At Start: Copied And Sealed By The First Pass
    let mut start = bench
        .handle
        .open_file_watch(&request(&bench.execution, local(tree.path()), vec![include("*.log")], FileWatchPolicy::SnapshotAtStart), None)
        .await
        .unwrap();
    let taken = sealed(&poll(&mut start).await);
    assert!(start.is_complete());
    assert_artifact(&taken["bench.log"], "bench.log", b"start\n");

    // At Stop: Nothing Until Sealed, Then The Tree As It Is
    let mut stop = bench
        .handle
        .open_file_watch(&request(&bench.execution, local(tree.path()), vec![include("*.log")], FileWatchPolicy::SnapshotAtStop), None)
        .await
        .unwrap();
    assert!(matches!(poll(&mut stop).await.as_slice(), [WatchEvent::Armed { .. }]));
    append(&log, b"end\n");
    write(&tree.path().join("late.log"), b"late\n");
    assert!(poll(&mut stop).await.is_empty());
    let taken = sealed(&seal(&mut stop).await);
    assert_artifact(&taken["bench.log"], "bench.2.log", b"start\nend\n");
    assert_artifact(&taken["late.log"], "late.log", b"late\n");

    drop((start, stop));
    bench.stop().await;
}

// Caps and malformed requests are refused explicitly; a capped watch still
// seals what it captured.
#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn watch_refusals() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    write(&tree.path().join("a.log"), b"0123456789");
    write(&tree.path().join("b.log"), b"0123456789");
    let base = || request(&bench.execution, local(tree.path()), vec![include("*.log")], FileWatchPolicy::Mirror);

    // The File Cap Refuses the Second File, After Sealing the First
    let mut capped = bench.handle.open_file_watch(&FileWatch { maximum_files: Some(1), ..base() }, None).await.unwrap();
    let mut events = Vec::new();
    let refused = capped.poll(&mut events).await.unwrap_err();
    assert_eq!(code(&refused), CAPPED, "{refused}");
    let sealed = sealed(&seal(&mut capped).await);
    assert_artifact(&sealed["a.log"], "a.log", b"0123456789");

    // The Byte Cap Refuses a File That Would Exceed It
    let mut small = bench.handle.open_file_watch(&FileWatch { maximum_bytes: Some(15), ..base() }, None).await.unwrap();
    let refused = small.poll(&mut Vec::new()).await.unwrap_err();
    assert_eq!(code(&refused), CAPPED, "{refused}");
    seal(&mut small).await;

    // Requests Are Refused Before a Source Opens
    let refusals = [
        FileWatch { execution_id: None, ..base() },
        FileWatch { rules: vec![exclude("*.log")], ..base() },
        FileWatch { output: "../escape".to_owned(), ..base() },
        FileWatch {
            policy: FileWatchPolicy::Unspecified,
            ..base()
        },
        FileWatch { poll_interval_ms: Some(1), ..base() },
        FileWatch { maximum_files: Some(257), ..base() },
        FileWatch { source: None, ..base() },
    ];
    for refused in refusals {
        let error = bench.handle.open_file_watch(&refused, None).await.err().unwrap();
        assert_eq!(code(&error), INVALID, "{error}");
    }

    // The Context Execution Must Agree with an Explicit One
    let error = bench.handle.open_file_watch(&base(), Some("00000000-0000-7000-8000-000000000000")).await.err().unwrap();
    assert_eq!(code(&error), INVALID, "{error}");
    let scoped = bench.handle.open_file_watch(&FileWatch { execution_id: None, ..base() }, Some(&bench.execution)).await;
    assert!(scoped.is_ok(), "the calling step's execution scopes the watch");
    drop(scoped);

    // FTP Sources Refuse Untrusted Plain FTP, SFTP and Inline Passwords
    let ftp = |url: &str| FileWatch {
        source: Some(model::files::wire::FileWatchSource {
            payload: Some(FileWatchSourcePayload::Ftp(FtpWatchSource { url: url.to_owned(), secret_id: None })),
        }),
        ..base()
    };
    for (url, expected) in [
        ("ftp://127.0.0.1:1/logs", UNTRUSTED),
        ("sftp://127.0.0.1/logs", UNSUPPORTED),
        ("ftps://bench:secret@127.0.0.1/logs", INVALID),
        ("http://127.0.0.1/logs", INVALID),
    ] {
        let error = bench.handle.open_file_watch(&ftp(url), None).await.err().unwrap();
        assert_eq!(code(&error), expected, "{url}: {error}");
    }

    bench.stop().await;
}

// A failed refetch keeps the last whole copy: the output never holds a
// partial or emptied file.
#[cfg(unix)]
#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn mirror_keeps_last() {
    use std::os::unix::fs::PermissionsExt;
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    let report = tree.path().join("report.csv");
    write(&report, b"v1\n");
    let mut watch = bench
        .handle
        .open_file_watch(&request(&bench.execution, local(tree.path()), vec![include("*.csv")], FileWatchPolicy::Mirror), None)
        .await
        .unwrap();
    assert_eq!(phases(&poll(&mut watch).await, Phase::Synced), vec![("report.csv".to_owned(), 3)]);

    // The Source Changes but Cannot Be Read
    write(&report, b"version two, longer\n");
    std::fs::set_permissions(&report, std::fs::Permissions::from_mode(0o000)).unwrap();
    let unreadable = std::fs::File::open(&report).is_err();
    let mut refetch = Vec::new();
    let result = watch.poll(&mut refetch).await;
    let sealed = sealed(&seal(&mut watch).await);
    std::fs::set_permissions(&report, std::fs::Permissions::from_mode(0o644)).unwrap();
    if unreadable {
        assert!(result.is_err(), "permission errors fail immediately");
        assert!(phases(&refetch, Phase::Synced).is_empty());
        assert_artifact(&sealed["report.csv"], "report.csv", b"v1\n");
    } else {
        eprintln!("Skipped the unreadable refetch: this user reads any file");
    }

    drop(watch);
    bench.stop().await;
}

// Closing the execution seals its running watch into it first.
#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn execution_seals_watch() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    let log = tree.path().join("run.log");
    write(&log, b"");
    let mut watch = bench
        .handle
        .open_file_watch(&request(&bench.execution, local(tree.path()), vec![include("*.log")], FileWatchPolicy::Stream), None)
        .await
        .unwrap();
    poll(&mut watch).await;
    append(&log, b"captured before the execution closed\n");

    // Poll Once for the Appended Line, Then Drive the Watch Until the Execution Asks It to Seal
    poll(&mut watch).await;
    let driver = tokio::spawn(async move {
        watch.seal_requested().await;
        let mut events = Vec::new();
        watch.seal(&mut events).await.unwrap();
        sealed(&events)
    });
    bench.executor.release();
    let sealed = driver.await.unwrap();
    assert_artifact(&sealed["run.log"], "run.log", b"captured before the execution closed\n");
    let stored = bench
        .handle
        .artifacts()
        .await
        .unwrap()
        .iter()
        .find(|record| record.filename == format!("{OUTPUT}/run.log"))
        .cloned()
        .expect("the watch artifact belongs to the execution");
    assert_eq!(stored.status, ArtifactStatus::Complete);

    bench.stop().await;
}

// The step's watch over one file, submitted as an operation: its matcher
// publishes rows to the declared stream, its marker files a record, and its
// artifact holds only what was written after the watch started.
#[dokime_tracing::trace_test]
#[tokio::test(flavor = "multi_thread")]
async fn watch_file_matchers() {
    const APPENDED: &[u8] = b"elapsed=1.5 locked=true\nWARN W42\nelapsed=2.5 locked=false\n";
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    let log = tree.path().join("device.log");
    write(&log, b"elapsed=0.5 locked=false\n");
    let request = WatchFile {
        recursive: false,
        rules: Vec::new(),
        maximum_files: None,
        execution_id: Some(bench.execution.clone()),
        source: Some(FileWatchSource { payload: Some(local(&log)) }),
        output: OUTPUT.to_owned(),
        artifact: Some(format!("{OUTPUT}/device.log")),
        policy: FileWatchPolicy::Stream,
        selection: model::files::wire::FileWatchSelection::All,
        wait_for_source_seconds: None,
        poll_interval_ms: None,
        maximum_seconds: None,
        maximum_bytes: None,
        matchers: vec![LineMatcher {
            storage_schema: crate::service_support::contracts::minimal_tracking_contract().storage_schema.unwrap(),
            pattern: r"elapsed=(?P<elapsed_s>[\d.]+) locked=(?P<target_locked>\w+)".to_owned(),
            stream: "tracking".to_owned(),
            fields: vec![
                LineField {
                    name: "elapsed_s".to_owned(),
                    kind: LineFieldKind::Number,
                },
                LineField {
                    name: "target_locked".to_owned(),
                    kind: LineFieldKind::Boolean,
                },
            ],
        }],
        markers: vec![LineMarker::new(r"WARN (?P<warning_code>\w+)", "tracking.fixture_warning")],
    };
    let (operations, ticket) = ticketed(&bench);
    let watch = operations.submit::<WatchFile>(request).unwrap();

    // Lines Written After the Start Are Captured and Matched
    bench
        .handle
        .operations()
        .operation::<dokime_operations::files::AwaitFileWatchArmed>(dokime_operations::files::AwaitFileWatchArmed { instance_id: ticket })
        .await
        .unwrap();
    let mut records = bench.handle.subscribe_records();
    append(&log, APPENDED);
    crate::service_support::waits::wait_for_subscription_record(&mut records, "tracking.fixture_warning").await;
    let outcome = watch.cancel().await.unwrap().into_owned();
    assert_eq!(
        outcome.artifacts,
        vec![WatchedArtifact {
            artifact: format!("{OUTPUT}/device.log"),
            size_bytes: APPENDED.len() as u64
        }]
    );
    assert!(outcome.gaps.is_empty());

    // Rows Reach the Declared Stream in the Execution's Scope
    let execution = dokime_runtime::TestExecutionId::from_uuid(bench.execution.clone()).unwrap();
    let rows = bench
        .handle
        .collect_telemetry(&TelemetrySelection {
            streams: vec![identifier("tracking").into()],
            scope: ScopeSelection {
                test_execution_id: Some(execution.clone()),
                ..Default::default()
            },
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(
        rows.iter().map(|row| (row.values.get("elapsed_s").cloned(), row.values.get("target_locked").cloned())).collect::<Vec<_>>(),
        vec![(Some(Value::Number(1.5)), Some(Value::Bool(true))), (Some(Value::Number(2.5)), Some(Value::Bool(false))),]
    );

    // The Marker Is Recorded
    let markers = bench
        .handle
        .selected_records(RecordSelection {
            record_types: vec![identifier("tracking.fixture_warning")],
            ..Default::default()
        })
        .await
        .unwrap();
    assert_eq!(markers.len(), 1);
    assert_eq!(markers[0].record.details.get("warning_code"), Some(&Value::Text("W42".to_owned())));
    assert_eq!(markers[0].record.message.as_deref(), Some("WARN W42"));

    // The Artifact Holds Only the Appended Bytes
    let record = bench
        .handle
        .artifacts()
        .await
        .unwrap()
        .iter()
        .find(|record| record.filename == format!("{OUTPUT}/device.log"))
        .cloned()
        .expect("the watch sealed its artifact");
    assert_artifact(&record, "device.log", APPENDED);
    let content = bench.handle.read_artifact_content(&record.artifact_id, APPENDED.len()).await.unwrap().unwrap();
    assert_eq!(content.bytes, APPENDED);
    bench.stop().await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn recursive_step_watches() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    write(&tree.path().join("Logs/existing.log"), b"before\n");
    write(&tree.path().join("Logs/old/day.log"), b"old\n");
    let request = WatchFile {
        recursive: true,
        rules: vec![include("Logs/**/*.log")],
        maximum_files: Some(4),
        execution_id: Some(bench.execution.clone()),
        source: Some(FileWatchSource { payload: Some(local(tree.path())) }),
        output: "logs".into(),
        artifact: None,
        policy: FileWatchPolicy::Stream,
        selection: model::files::wire::FileWatchSelection::All,
        wait_for_source_seconds: None,
        poll_interval_ms: Some(50),
        maximum_seconds: None,
        maximum_bytes: None,
        matchers: Vec::new(),
        markers: Vec::new(),
    };
    let mut watch = bench.handle.open_watch_file(&request, None).await.unwrap();
    let armed = poll(&mut watch).await;
    assert!(armed.iter().any(|event| matches!(event, WatchEvent::Armed { baseline } if baseline.len() == 2)));
    append(&tree.path().join("Logs/existing.log"), b"after\n");
    append(&tree.path().join("Logs/old/day.log"), b"tail\n");
    write(&tree.path().join("Logs/next-day/new.log"), b"new\n");
    write(&tree.path().join("Logs/next-day/ignored.zip"), b"ignored");
    let records = sealed(&seal(&mut watch).await);
    assert_eq!(records.len(), 3);
    for (path, bytes) in
        [("Logs/existing.log", b"after\n".as_slice()), ("Logs/old/day.log", b"tail\n"), ("Logs/next-day/new.log", b"new\n")]
    {
        let record = &records[path];
        assert_eq!(record.filename, format!("logs/{path}"));
        let digest = sha2::Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
        assert_eq!(record.sha256.as_deref(), Some(digest.as_str()));
    }
    write(&tree.path().join("bundle.zip"), b"old");
    write(&tree.path().join("ignored.diag"), b"diagnostic");
    let archives = WatchFile {
        rules: vec![include("**/*.zip")],
        policy: FileWatchPolicy::Mirror,
        ..request.clone()
    };
    let mut watch = bench.handle.open_watch_file(&archives, None).await.unwrap();
    poll(&mut watch).await;
    write(&tree.path().join("bundle.zip"), b"updated archive");
    write(&tree.path().join("new/deep/archive.zip"), b"new archive");
    write(&tree.path().join("new/deep/ignored.log"), b"ignored");
    poll(&mut watch).await;
    let records = sealed(&seal(&mut watch).await);
    assert_eq!(records.len(), 3);
    assert_eq!(records["bundle.zip"].filename, "logs/bundle.zip");
    assert_eq!(records["bundle.zip"].size_bytes, 15);
    assert_eq!(records["new/deep/archive.zip"].filename, "logs/new/deep/archive.zip");
    for extension in ["diag", "hpl30"] {
        let name = format!("capture.{extension}");
        write(&tree.path().join(&name), b"snapshot");
        let snapshot = WatchFile {
            recursive: false,
            rules: Vec::new(),
            source: Some(FileWatchSource {
                payload: Some(local(&tree.path().join(&name))),
            }),
            policy: FileWatchPolicy::SnapshotAtStart,
            output: String::new(),
            ..request.clone()
        };
        let mut watch = bench.handle.open_watch_file(&snapshot, None).await.unwrap();
        let records = sealed(&poll(&mut watch).await);
        assert_eq!(records[&name].filename, name);
        assert_eq!(records[&name].size_bytes, 8);
        assert_eq!(records[&name].content_type.as_deref(), Some("application/octet-stream"));
        for unsafe_name in ["../escape.diag", "/absolute.hpl30", "C:/escape.diag", "logs/CON.diag", "NUL.hpl30"] {
            let invalid = WatchFile {
                artifact: Some(unsafe_name.into()),
                ..snapshot.clone()
            };
            assert!(bench.handle.open_watch_file(&invalid, None).await.is_err(), "{unsafe_name}");
        }
    }
    let capped = WatchFile {
        maximum_files: Some(1),
        ..request.clone()
    };
    let mut watch = bench.handle.open_watch_file(&capped, None).await.unwrap();
    assert!(watch.poll(&mut Vec::new()).await.is_err());
    seal(&mut watch).await;
    let capped = WatchFile {
        maximum_bytes: Some(20),
        rules: vec![include("**/*.zip")],
        policy: FileWatchPolicy::Mirror,
        ..request.clone()
    };
    let mut watch = bench.handle.open_watch_file(&capped, None).await.unwrap();
    assert!(watch.poll(&mut Vec::new()).await.is_err());
    seal(&mut watch).await;
    // Nonrecursive directory discovery ignores deeper files even with a recursive glob.
    let top = WatchFile {
        recursive: false,
        rules: vec![include("**/*.zip")],
        policy: FileWatchPolicy::SnapshotAtStart,
        ..request.clone()
    };
    let mut watch = bench.handle.open_watch_file(&top, None).await.unwrap();
    let records = sealed(&poll(&mut watch).await);
    assert_eq!(records.keys().map(String::as_str).collect::<Vec<_>>(), ["bundle.zip"]);
    write(&tree.path().join("Logs/status"), b"no extension");
    let all = WatchFile {
        rules: Vec::new(),
        policy: FileWatchPolicy::SnapshotAtStart,
        maximum_files: None,
        ..request
    };
    let mut watch = bench.handle.open_watch_file(&all, None).await.unwrap();
    let records = sealed(&poll(&mut watch).await);
    assert!(records.contains_key("capture.hpl30"));
    assert_eq!(records["Logs/status"].filename, "logs/Logs/status");
    assert_eq!(records["Logs/status"].size_bytes, 12);
    bench.stop().await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn live_artifact_tree() {
    use dokime_domain::model::content::wire::{ExecutionFileFollowFramePayload, ExecutionFilePhase};
    use dokime_runtime::ExecutionFileStreamState;
    use dokime_runtime::artifacts::ArtifactUploadInput;
    use std::num::NonZeroUsize;

    let bench = Bench::start(|config, _| config).await;
    let execution = TestExecutionId::from_uuid(bench.execution.clone()).unwrap();
    let tree = tempdir().unwrap();
    let existing = tree.path().join("Logs/existing.log");
    write(&existing, b"before\n");
    write(&tree.path().join("Logs/untouched.log"), b"untouched\n");
    write(&tree.path().join("Logs/empty.log"), b"");
    let request = WatchFile {
        execution_id: Some(bench.execution.clone()),
        source: Some(FileWatchSource { payload: Some(local(tree.path())) }),
        recursive: true,
        rules: Vec::new(),
        maximum_files: None,
        output: OUTPUT.into(),
        artifact: None,
        policy: FileWatchPolicy::Stream,
        selection: model::files::wire::FileWatchSelection::All,
        wait_for_source_seconds: None,
        poll_interval_ms: None,
        maximum_seconds: None,
        maximum_bytes: None,
        matchers: Vec::new(),
        markers: Vec::new(),
    };
    let mut untouched = bench.handle.open_watch_file(&request, None).await.unwrap();
    poll(&mut untouched).await;
    assert!(sealed(&seal(&mut untouched).await).is_empty());
    assert!(untouched.sealed().is_empty());
    assert!(!bench.handle.execution_files(&execution).await.unwrap().iter().any(|file| file.relative_path.starts_with("artifacts/")));

    let mut watch = bench.handle.open_watch_file(&request, None).await.unwrap();
    poll(&mut watch).await;
    append(&existing, b"first");
    write(&tree.path().join("Logs/new/deep.log"), b"born");
    write(&tree.path().join("Logs/new/empty.log"), b"");
    poll(&mut watch).await;
    let path = "artifacts/capture/Logs/existing.log";
    let files = bench.handle.execution_files(&execution).await.unwrap();
    let live = files.iter().find(|file| file.relative_path == path).unwrap();
    assert_eq!(live.size_bytes, 5);
    assert_eq!(live.stream_state, ExecutionFileStreamState::Streaming);
    let artifact_id = live.artifact_id.clone();
    let window = bench.handle.execution_file_content(&execution, path.into(), 0, 3).await.unwrap().unwrap();
    assert_eq!(window.bytes, b"fir");
    drop(window);
    let mut follower = bench.handle.follow_execution_file(&execution, path, None, 0).await.unwrap().unwrap();
    let identity = follower.metadata().identity.clone();
    assert_eq!(follower.metadata().phase, ExecutionFilePhase::Producing);
    let maximum = NonZeroUsize::new(3).unwrap();
    let retained = follower.next_frame(maximum).await.unwrap().unwrap();
    let Some(ExecutionFileFollowFramePayload::Chunk(first)) = &retained.payload else { panic!("first committed bytes") };
    assert_eq!(first.data, b"fir");
    // A Viewer Can Retain A Read While Capture Continues Independently.
    append(&existing, b"more");
    poll(&mut watch).await;
    let files = bench.handle.execution_files(&execution).await.unwrap();
    let grown = files.iter().find(|file| file.relative_path == path).unwrap();
    assert_eq!(grown.size_bytes, 9);
    assert_eq!(grown.artifact_id, artifact_id);
    let mut bytes = first.data.clone();
    drop(retained);
    while bytes.len() < 9 {
        let frame = follower.next_frame(maximum).await.unwrap().unwrap();
        let Some(ExecutionFileFollowFramePayload::Chunk(chunk)) = &frame.payload else { panic!("growing bytes") };
        bytes.extend_from_slice(&chunk.data);
    }
    assert_eq!(bytes, b"firstmore");
    let records = sealed(&seal(&mut watch).await);
    assert_eq!(records.len(), 3);
    assert_artifact(&records["Logs/existing.log"], "Logs/existing.log", b"firstmore");
    assert_artifact(&records["Logs/new/deep.log"], "Logs/new/deep.log", b"born");
    assert_artifact(&records["Logs/new/empty.log"], "Logs/new/empty.log", b"");
    let terminal = follower.next_frame(maximum).await.unwrap().unwrap();
    let Some(ExecutionFileFollowFramePayload::Terminal(terminal)) = &terminal.payload else { panic!("finished file") };
    assert_eq!(terminal.phase, ExecutionFilePhase::Finished);
    assert_eq!(terminal.length, 9);
    assert_eq!(terminal.sealed.as_ref().unwrap().identity, identity);
    assert!(follower.next_frame(maximum).await.unwrap().is_none());
    drop(follower);
    let files = bench.handle.execution_files(&execution).await.unwrap();
    let finished = files.iter().find(|file| file.relative_path == path).unwrap();
    assert_eq!(finished.stream_state, ExecutionFileStreamState::Finalized);
    assert_eq!(finished.artifact_id, artifact_id);
    let mut late = bench.handle.follow_execution_file(&execution, path, None, 0).await.unwrap().unwrap();
    assert_eq!(late.metadata().phase, ExecutionFilePhase::Finished);
    assert!(late.next_frame(maximum).await.unwrap().is_some());
    drop(late);

    let scope = records["Logs/existing.log"].scope.clone();
    let mut uploads = Vec::new();
    for name in ["Logs/existing.log", "Logs/existing.log", "Logs/existing.2.log", "Logs/existing.log"] {
        let upload = bench
            .handle
            .begin_artifact_upload(ArtifactUploadInput {
                scope: scope.clone(),
                filename: name.into(),
                content_type: None,
                size_bytes: 1,
                sha256: None,
            })
            .await
            .unwrap();
        upload.send(b"x".to_vec()).await.unwrap();
        uploads.push(upload);
    }
    let files = bench.handle.execution_files(&execution).await.unwrap();
    let uploaded = files.iter().find(|file| file.relative_path == "artifacts/Logs/existing.log").unwrap();
    assert_eq!(uploaded.content_type.as_deref(), Some("application/octet-stream"));
    let window = bench.handle.execution_file_content(&execution, uploaded.relative_path.clone(), 0, 1).await.unwrap().unwrap();
    assert_eq!(window.content_type.as_deref(), Some("application/octet-stream"));
    drop(window);
    let followed = bench.handle.follow_execution_file(&execution, &uploaded.relative_path, None, 0).await.unwrap().unwrap();
    assert_eq!(followed.metadata().content_type, "application/octet-stream");
    drop(followed);
    let mut ids = std::collections::BTreeSet::new();
    for (upload, name) in uploads.iter().zip(["Logs/existing.log", "Logs/existing.2.log", "Logs/existing.2.2.log", "Logs/existing.3.log"]) {
        let record = upload.finish().await.unwrap();
        assert_eq!(record.filename, name);
        assert!(ids.insert(record.artifact_id.clone()));
        let path = format!("artifacts/{name}");
        let file = bench.handle.execution_file_content(&execution, path, 0, 8).await.unwrap().unwrap();
        assert_eq!(file.bytes, b"x");
    }
    let files = bench.handle.execution_files(&execution).await.unwrap();
    assert!(files.iter().filter(|file| file.relative_path.starts_with("artifacts/")).all(|file| !file.relative_path.contains("uploads/")));
    drop(uploads);
    bench.executor.release();
    let checkpoint = crate::service_support::native::checkpoint(&bench.handle, &execution).await;
    let saved = bench.handle.execution_files(&execution).await.unwrap();
    assert_eq!(saved.iter().filter(|file| file.relative_path.starts_with("artifacts/")).count(), 7);
    assert!(!saved.iter().any(|file| file.relative_path.contains("untouched") || file.relative_path.ends_with("Logs/empty.log")));
    let mut reader = bench.handle.checkpoint_artifact(checkpoint, &records["Logs/existing.log"].artifact_id).await.unwrap();
    let mut bytes = [0; 9];
    assert_eq!(reader.read(&mut bytes).await.unwrap(), 9);
    assert_eq!(&bytes, b"firstmore");
    drop(reader);
    bench.stop().await;
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn live_media_upload() {
    use dokime_runtime::ExecutionFileStreamState;
    use dokime_runtime::artifacts::ArtifactUploadInput;

    let bench = Bench::start(|config, _| config).await;
    let execution = TestExecutionId::from_uuid(bench.execution.clone()).unwrap();
    let scope = bench.handle.watch_state().latest().state().executions[&execution].scope.clone();
    // A Valid One-Pixel PNG Is Saved While The Execution Remains Blocked.
    let png: &[u8] = &[
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 4, 0, 0, 0, 181, 28, 12, 2, 0, 0, 0, 11,
        73, 68, 65, 84, 120, 218, 99, 100, 248, 15, 0, 1, 5, 1, 1, 39, 24, 227, 102, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
    ];
    let stem = "capture-".repeat(28);
    let names = [format!("{stem}.png"), format!("{stem}.2.png"), format!("{stem}.2.2.png"), format!("{stem}.3.png")];
    let mut uploads = Vec::new();
    for requested in [&names[0], &names[0], &names[1], &names[0]] {
        let upload = bench
            .handle
            .begin_artifact_upload(ArtifactUploadInput {
                scope: scope.clone(),
                filename: requested.clone(),
                content_type: Some("image/png".into()),
                size_bytes: png.len() as u64,
                sha256: None,
            })
            .await
            .unwrap();
        upload.send(png.to_vec()).await.unwrap();
        uploads.push(upload);
    }
    for (upload, name) in uploads.iter().zip(&names) {
        let record = upload.finish().await.unwrap();
        assert_eq!(&record.filename, name);
        let path = format!("media/{name}");
        let files = bench.handle.execution_files(&execution).await.unwrap();
        let file = files.iter().find(|file| file.relative_path == path).unwrap();
        assert_eq!(file.artifact_id, record.artifact_id.to_string());
        assert_eq!(file.content_type.as_deref(), Some("image/png"));
        assert_eq!(file.stream_state, ExecutionFileStreamState::Finalized);
        let content = bench.handle.execution_file_content(&execution, path, 0, png.len()).await.unwrap().unwrap();
        assert_eq!(content.content_type.as_deref(), Some("image/png"));
        assert_eq!(content.bytes, png);
        assert!(bench.handle.artifacts().await.unwrap().iter().any(|entry| entry.artifact_id == record.artifact_id));
    }
    assert!(bench.handle.watch_state().latest().state().executions[&execution].saved.is_none());
    drop(uploads);
    bench.executor.release();
    crate::service_support::native::checkpoint(&bench.handle, &execution).await;
    let files = bench.handle.execution_files(&execution).await.unwrap();
    assert_eq!(files.iter().filter(|file| file.relative_path.starts_with("media/")).count(), 4);
    let mut report = bench.handle.open_execution_report(&execution).await.unwrap();
    let html = crate::service_support::reports::opened_bytes(&mut report).await;
    let evidence = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/tests/media");
    std::fs::create_dir_all(&evidence).unwrap();
    std::fs::write(evidence.join("report.html"), &html).unwrap();
    let payload = crate::service_support::reports::parse(std::str::from_utf8(&html).unwrap());
    let artifacts = payload["snapshot"]["artifacts"].as_array().unwrap();
    assert_eq!(artifacts.len(), 4);
    for artifact in artifacts {
        assert!(artifact["path"].as_str().unwrap().starts_with("media/"));
        let path = artifact["path"].as_str().unwrap();
        let href = artifact["href"].as_str().unwrap();
        assert_eq!(href, format!("outputs/{path}"));
        let content = bench.handle.execution_file_content(&execution, path.to_owned(), 0, png.len()).await.unwrap().unwrap();
        assert_eq!(content.bytes, png);
        let destination = evidence.join(href);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::write(destination, &content.bytes).unwrap();
        assert_eq!(artifact["downloadHref"], href);
    }
    bench.stop().await;
}

#[cfg(feature = "testing")]
#[dokime_tracing::trace_test]
#[tokio::test]
async fn artifact_live_failure() {
    use dokime_domain::model::content::wire::{ExecutionFileFollowFramePayload, ExecutionFilePhase};
    use dokime_runtime::artifacts::ArtifactUploadInput;
    use std::num::NonZeroUsize;

    let bench = Bench::start(|config, _| config).await;
    let execution = TestExecutionId::from_uuid(bench.execution.clone()).unwrap();
    let scope = bench.handle.watch_state().latest().state().executions[&execution].scope.clone();
    bench.handle.inject_artifact_capture_failure(execution.clone(), "failed.log".into(), std::io::ErrorKind::PermissionDenied);
    let upload = bench
        .handle
        .begin_artifact_upload(ArtifactUploadInput {
            scope,
            filename: "failed.log".into(),
            content_type: None,
            size_bytes: 1,
            sha256: None,
        })
        .await
        .unwrap();
    upload.send(b"x".to_vec()).await.unwrap();
    let mut follower = bench.handle.follow_execution_file(&execution, "artifacts/failed.log", None, 0).await.unwrap().unwrap();
    let maximum = NonZeroUsize::new(1).unwrap();
    assert!(matches!(&follower.next_frame(maximum).await.unwrap().unwrap().payload, Some(ExecutionFileFollowFramePayload::Chunk(_))));
    assert_eq!(upload.finish().await.unwrap().status, ArtifactStatus::Failed);
    let frame = follower.next_frame(maximum).await.unwrap().unwrap();
    let Some(ExecutionFileFollowFramePayload::Terminal(terminal)) = &frame.payload else { panic!("failed capture terminates") };
    assert_eq!(terminal.phase, ExecutionFilePhase::Abandoned);
    assert!(terminal.sealed.is_none());
    assert!(follower.next_frame(maximum).await.unwrap().is_none());
    drop((follower, upload));
    bench.stop().await;
}

#[cfg(feature = "testing")]
#[tokio::test]
async fn capture_failure_isolated_during_stop() {
    use dokime_runtime::telemetry::{SamplingScope, TelemetryBatch, TelemetrySample};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tracing_subscriber::{Layer, layer::SubscriberExt};

    struct LeaseWarnings(Arc<AtomicUsize>);
    impl<S: tracing::Subscriber> Layer<S> for LeaseWarnings {
        fn on_event(&self, event: &tracing::Event<'_>, _: tracing_subscriber::layer::Context<'_, S>) {
            if event.metadata().level() == &tracing::Level::WARN && event.metadata().target().ends_with("telemetry::leases") {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }
    }
    let warnings = Arc::new(AtomicUsize::new(0));
    let subscriber = tracing_subscriber::registry().with(LeaseWarnings(warnings.clone()));
    let _subscriber = tracing::subscriber::set_default(subscriber);
    let stream = identifier("fixture.capture");
    let contract = || {
        let mut contract = minimal_tracking_contract();
        contract.stream = stream.clone();
        contract.metadata = metadata("fixture.capture", "Capture");
        contract.interval = Some(dokime_runtime::DurationMillis(1000));
        contract
    };
    let bench = Bench::start(|config, _| {
        config.with_installed_plugins([dokime_runtime::InstalledPlugin {
            source: "fixture.capture".into(),
            enabled: true,
            telemetry_contracts: vec![contract()],
            ..Default::default()
        }])
    })
    .await;
    let execution = dokime_runtime::TestExecutionId::from_uuid(bench.execution.clone()).unwrap();
    let cycle_id = bench.handle.watch_state().latest().state().executions[&execution].test_cycle_id.clone();
    let cycle = cycle_snapshot(&bench.handle, &cycle_id).await;
    let scope = cycle.test_executions[0].scope.clone();
    let held = bench
        .handle
        .lease_streams(
            SamplingScope {
                test_cycle_id: scope.test_cycle_id.as_ref().map(ToString::to_string),
                test_case_id: scope.test_case_id.as_ref().map(ToString::to_string),
                test_execution_id: Some(execution.to_string()),
            },
            std::slice::from_ref(&stream),
            "fixture.reader",
        )
        .await
        .unwrap();
    let viewed = bench
        .handle
        .lease_streams(
            SamplingScope {
                test_cycle_id: None,
                test_case_id: None,
                test_execution_id: None,
            },
            std::slice::from_ref(&stream),
            "fixture.viewer",
        )
        .await
        .unwrap();

    let tree = tempdir().unwrap();
    write(&tree.path().join("bad/app.log"), b"unavailable\n");
    write(&tree.path().join("good/app.log"), b"retained\n");
    write(&tree.path().join("example.diag"), b"snapshot\n");
    bench.handle.inject_artifact_capture_failure(execution.clone(), format!("{OUTPUT}/bad/app.log"), std::io::ErrorKind::PermissionDenied);
    let mut mirror = bench
        .handle
        .open_file_watch(&request(&bench.execution, local(tree.path()), vec![include("**/*.log")], FileWatchPolicy::Mirror), None)
        .await
        .unwrap();
    poll(&mut mirror).await;
    let mut streaming = request(&bench.execution, local(tree.path()), vec![include("good/*.log")], FileWatchPolicy::Stream);
    streaming.output = "stream".into();
    let mut streaming = bench.handle.open_file_watch(&streaming, None).await.unwrap();
    poll(&mut streaming).await;
    append(&tree.path().join("good/app.log"), b"tail\n");
    let mut snapshot = request(&bench.execution, local(tree.path()), vec![include("*.diag")], FileWatchPolicy::SnapshotAtStart);
    snapshot.output = "prepare".into();
    let mut snapshot = bench.handle.open_file_watch(&snapshot, None).await.unwrap();
    poll(&mut snapshot).await;
    let captures = tokio::spawn(async move {
        let seal_mirror = async {
            mirror.seal_requested().await;
            seal(&mut mirror).await
        };
        let seal_stream = async {
            streaming.seal_requested().await;
            seal(&mut streaming).await
        };
        tokio::join!(seal_mirror, seal_stream)
    });
    let publish = |index| {
        let mut sample = TelemetrySample::new(
            dokime_runtime::model::identity::new_id(),
            stream.clone(),
            index,
            dokime_runtime::model::time::now_utc(),
            "fixture.capture",
        );
        sample.values.insert("elapsed_s".into(), dokime_runtime::Value::Number(index as f64));
        sample.values.insert("target_locked".into(), dokime_runtime::Value::Bool(true));
        let sample = typed_minimal_tracking_sample(sample);
        TelemetryBatch::new(dokime_runtime::model::identity::new_id(), stream.clone(), vec![sample])
    };
    bench.handle.publish_telemetry_batch(publish(0)).await.unwrap();
    bench.handle.stop_execution(execution.clone(), CommandContext::default()).await.unwrap();
    captures.await.unwrap();
    let idle = wait_for_cycle_idle(&bench.handle, &cycle.test_cycle_id, ResultStatus::Cancelled).await;
    assert_eq!(idle.lifecycle, CycleStatus::Ready);
    assert_eq!(idle.test_executions[0].status, ExecutionStatus::Cancelled);
    assert!(idle.test_executions[0].saved.is_some());
    let request = bench
        .handle
        .watch_state()
        .latest()
        .state()
        .run_requests
        .values()
        .find(|request| request.selected_execution_ids.contains(&execution))
        .unwrap()
        .summary
        .request_id
        .clone();
    assert!(bench.handle.run_request(&request).await.unwrap().unwrap().summary.phase.terminal());
    for lease in &held {
        assert!(!bench.handle.stream_lease_is_active(lease).await);
    }
    for index in 1..4 {
        bench.handle.publish_telemetry_batch(publish(index)).await.unwrap();
    }
    for lease in &viewed {
        assert!(bench.handle.stream_lease_is_active(lease).await);
    }
    assert_eq!(warnings.load(Ordering::SeqCst), 0);
    assert!(
        bench
            .handle
            .lease_streams(
                SamplingScope {
                    test_cycle_id: None,
                    test_case_id: None,
                    test_execution_id: Some(execution.to_string()),
                },
                std::slice::from_ref(&stream),
                "fixture.reader"
            )
            .await
            .is_err()
    );

    let records = bench.handle.artifacts().await.unwrap();
    let failed = records.iter().find(|record| record.filename == format!("{OUTPUT}/bad/app.log")).unwrap();
    assert_eq!(failed.status, ArtifactStatus::Failed);
    assert!(failed.error.is_some());
    let durable = dokime_runtime::storage::objects::execution::read(
        bench.handle.storage(),
        &execution,
        dokime_runtime::harmos::storage::ObjectVersion::Latest,
    )
    .await
    .unwrap()
    .unwrap();
    let saved = durable.value.saved.as_ref().unwrap();
    saved.validate(&durable.value).unwrap();
    let failed_output = saved
        .source
        .outputs
        .iter()
        .find(|output| {
            matches!(&output.content,
        dokime_domain::model::execution::saved::ExecutionOutputContent::Artifact { record, .. } if record.artifact_id == failed.artifact_id)
        })
        .unwrap();
    assert!(failed_output.file.is_none());
    assert!(bench.handle.read_artifact_content(&failed.artifact_id, 1024).await.is_err());
    for attach_content in [false, true] {
        let mut invalid = failed_output.clone();
        if attach_content {
            invalid.file = saved.source.outputs.iter().find_map(|output| output.file.clone());
        } else if let dokime_domain::model::execution::saved::ExecutionOutputContent::Artifact { record, .. } = &mut invalid.content {
            record.error = None;
        }
        assert!(dokime_domain::model::execution::saved::validate_outputs(&execution, &saved.dependencies_saved, &[invalid]).is_err());
    }
    let report = crate::service_support::reports::payload(bench.handle.storage(), &saved.report).await;
    let artifact =
        report["snapshot"]["artifacts"].as_array().unwrap().iter().find(|row| row["artifact_id"] == failed.artifact_id.as_str()).unwrap();
    assert_eq!(artifact["description"].as_str(), failed.error.as_deref());
    assert_eq!(artifact["href"], "");
    for (filename, bytes) in [
        ("capture/good/app.log", b"retained\ntail\n".as_slice()),
        ("stream/good/app.log", b"tail\n".as_slice()),
        ("prepare/example.diag", b"snapshot\n".as_slice()),
    ] {
        let artifact = records.iter().find(|record| record.filename == filename).unwrap();
        assert_eq!(artifact.status, ArtifactStatus::Complete);
        let content = bench.handle.read_artifact_content(&artifact.artifact_id, 1024).await.unwrap().unwrap();
        assert_eq!(content.bytes, bytes);
    }
    bench.handle.stop_execution(execution.clone(), CommandContext::default()).await.unwrap();
    assert_eq!(bench.handle.artifacts().await.unwrap(), records);
    assert_eq!(warnings.load(Ordering::SeqCst), 0);
    let failed = failed.clone();
    let Bench { _root: root, mut owner, handle, .. } = bench;
    handle.release_stream_leases(viewed).await.unwrap();
    drop((snapshot, handle));
    owner.shutdown().await.unwrap();
    let (mut owner, handle) = started(config(root.path())).await;
    assert_eq!(handle.artifacts().await.unwrap().iter().find(|record| record.artifact_id == failed.artifact_id), Some(&failed));
    assert!(handle.open_execution_report(&execution).await.is_ok());
    owner.shutdown().await.unwrap();
}
