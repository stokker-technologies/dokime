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
use dokime_domain::model;
use super::*;
use dokime_runtime::artifacts::ArtifactStatus;
use dokime_runtime::runtime::Error;
use dokime_runtime::domains::evidence::RecordSelection;
use dokime_runtime::domains::files::{CAPPED, GapReason, INVALID, Phase, UNSUPPORTED, UNTRUSTED};
use {
    model::files::wire::FileWatchPolicy,
    model::files::wire::FileWatchSourcePayload,
    model::files::wire::FtpWatchSource,
    model::files::wire::LineField,
    model::files::wire::LineFieldKind,
    model::files::wire::LineMarker,
    model::files::wire::LineMatcher,
    dokime_domain::operations::files::WatchFile,
    model::files::wire::WatchedArtifact,
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
fn gaps(events: &[WatchEvent]) -> Vec<(String, u64, GapReason)> {
    events
        .iter()
        .filter_map(|event| match event {
            WatchEvent::Gap {
                path,
                offset,
                reason,
            } => Some((path.clone(), *offset, *reason)),
            _ => None,
        })
        .collect()
}

fn phases(events: &[WatchEvent], wanted: Phase) -> Vec<(String, u64)> {
    events
        .iter()
        .filter_map(|event| match event {
            WatchEvent::Progress {
                path,
                offset,
                phase,
            } if *phase == wanted => Some((path.clone(), *offset)),
            _ => None,
        })
        .collect()
}

fn code(error: &Error) -> &str {
    error.code().unwrap_or_default()
}

/* === Tests === */
// Stream captures only bytes written after the start, follows rotation and
// truncation with gaps, picks up new files, and seals vanished ones.
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
                vec![
                    include("**/*.log"),
                    exclude("**/debug/**"),
                    include("**/debug/bms.log"),
                ],
                FileWatchPolicy::Stream,
            ),
            None,
        )
        .await
        .unwrap();

    // Files Present At The Start Baseline At Their Length
    let first = poll(&mut watch).await;
    assert_eq!(
        phases(&first, Phase::Baseline),
        vec![
            ("logs/charger.log".to_owned(), 0),
            ("logs/debug/bms.log".to_owned(), 0)
        ]
    );
    assert!(phases(&first, Phase::Capturing).is_empty());

    // Appended Bytes Are Captured; A New File From Its First Byte
    append(&tree.path().join("logs/charger.log"), b"line 1\n");
    write(&tree.path().join("logs/bench/inverter.log"), b"born\n");
    let grown = poll(&mut watch).await;
    assert_eq!(
        phases(&grown, Phase::Capturing),
        vec![
            ("logs/bench/inverter.log".to_owned(), 5),
            ("logs/charger.log".to_owned(), 7)
        ]
    );

    // Truncation Restarts At The First Byte; Rotation Seals The Segment
    std::fs::rename(
        tree.path().join("logs/charger.log"),
        tree.path().join("logs/charger.log.1"),
    )
    .unwrap();
    write(&tree.path().join("logs/charger.log"), b"rotated\n");
    write(&tree.path().join("logs/bench/inverter.log"), b"x\n");
    let rotated = poll(&mut watch).await;
    assert_eq!(
        gaps(&rotated),
        vec![
            (
                "logs/bench/inverter.log".to_owned(),
                5,
                GapReason::Truncated
            ),
            ("logs/charger.log".to_owned(), 7, GapReason::Replaced)
        ]
    );
    let segment = sealed(&rotated);
    assert_artifact(
        &segment["logs/charger.log"],
        "logs/charger.log",
        b"line 1\n",
    );

    // A Vanished File Is Sealed With A Gap While The Watch Continues
    append(&tree.path().join("logs/debug/bms.log"), b"cell 3\n");
    poll(&mut watch).await;
    std::fs::remove_file(tree.path().join("logs/debug/bms.log")).unwrap();
    let vanished = poll(&mut watch).await;
    assert_eq!(
        gaps(&vanished),
        vec![("logs/debug/bms.log".to_owned(), 7, GapReason::Vanished)]
    );
    let early = sealed(&vanished);
    assert_artifact(
        &early["logs/debug/bms.log"],
        "logs/debug/bms.log",
        b"cell 3\n",
    );

    // Sealing Captures The Last Bytes Of Every Remaining File
    append(&tree.path().join("logs/charger.log"), b"tail\n");
    let last = seal(&mut watch).await;
    let sealed = sealed(&last);
    assert_eq!(sealed.len(), 2);
    assert_artifact(
        &sealed["logs/charger.log"],
        "logs/charger.log",
        b"rotated\ntail\n",
    );
    assert_artifact(
        &sealed["logs/bench/inverter.log"],
        "logs/bench/inverter.log",
        b"born\nx\n",
    );
    assert_eq!(
        sealed["logs/charger.log"].content_type.as_deref(),
        Some("text/plain")
    );
    assert_eq!(
        sealed["logs/charger.log"]
            .scope
            .test_execution_id
            .as_ref()
            .map(ToString::to_string),
        Some(bench.execution.clone())
    );
    assert!(watch.is_complete());
    assert_eq!(watch.sealed().len(), 4);
    assert!(seal(&mut watch).await.is_empty(), "sealing is idempotent");

    drop(watch);
    bench.stop().await;
}

// Mirror keeps each output equal to its source, refetching on a size or
// modification-time change.
#[tokio::test(flavor = "multi_thread")]
async fn mirror_local_tree() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    let report = tree.path().join("report.csv");
    write(&report, b"a,b\n1,2\n");
    let mut watch = bench
        .handle
        .open_file_watch(
            &request(
                &bench.execution,
                local(tree.path()),
                vec![include("**/*.csv")],
                FileWatchPolicy::Mirror,
            ),
            None,
        )
        .await
        .unwrap();

    // The Whole File Is Mirrored At Discovery, Then Only On Change
    let first = poll(&mut watch).await;
    assert_eq!(
        phases(&first, Phase::Synced),
        vec![("report.csv".to_owned(), 8)]
    );
    assert!(phases(&poll(&mut watch).await, Phase::Synced).is_empty());

    // A Rewrite Of Equal Length Is Detected By Its Modification Time
    write(&report, b"a,b\n3,4\n");
    let later = SystemTime::now() + Duration::from_secs(5);
    std::fs::File::options()
        .write(true)
        .open(&report)
        .unwrap()
        .set_modified(later)
        .unwrap();
    write(&tree.path().join("runs/second.csv"), b"x\n");
    let changed = poll(&mut watch).await;
    assert_eq!(
        phases(&changed, Phase::Synced),
        vec![
            ("report.csv".to_owned(), 8),
            ("runs/second.csv".to_owned(), 2)
        ]
    );

    // A Shrunk Source Is Mirrored Whole, Without A Gap
    write(&report, b"a\n");
    let shrunk = poll(&mut watch).await;
    assert!(gaps(&shrunk).is_empty());

    let sealed = sealed(&seal(&mut watch).await);
    assert_artifact(&sealed["report.csv"], "report.csv", b"a\n");
    assert_artifact(&sealed["runs/second.csv"], "runs/second.csv", b"x\n");

    drop(watch);
    bench.stop().await;
}

// A snapshot copies each file once: at the start, or at the stop.
#[tokio::test(flavor = "multi_thread")]
async fn snapshot_local_tree() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    let log = tree.path().join("bench.log");
    write(&log, b"start\n");

    // At Start: Copied And Sealed By The First Pass
    let mut start = bench
        .handle
        .open_file_watch(
            &request(
                &bench.execution,
                local(tree.path()),
                vec![include("*.log")],
                FileWatchPolicy::SnapshotAtStart,
            ),
            None,
        )
        .await
        .unwrap();
    let taken = sealed(&poll(&mut start).await);
    assert!(start.is_complete());
    assert_artifact(&taken["bench.log"], "bench.log", b"start\n");

    // At Stop: Nothing Until Sealed, Then The Tree As It Is
    let mut stop = bench
        .handle
        .open_file_watch(
            &request(
                &bench.execution,
                local(tree.path()),
                vec![include("*.log")],
                FileWatchPolicy::SnapshotAtStop,
            ),
            None,
        )
        .await
        .unwrap();
    assert!(poll(&mut stop).await.is_empty());
    append(&log, b"end\n");
    write(&tree.path().join("late.log"), b"late\n");
    assert!(poll(&mut stop).await.is_empty());
    let taken = sealed(&seal(&mut stop).await);
    assert_artifact(&taken["bench.log"], "bench.log", b"start\nend\n");
    assert_artifact(&taken["late.log"], "late.log", b"late\n");

    drop((start, stop));
    bench.stop().await;
}

// Caps and malformed requests are refused explicitly; a capped watch still
// seals what it captured.
#[tokio::test(flavor = "multi_thread")]
async fn watch_refusals() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    write(&tree.path().join("a.log"), b"0123456789");
    write(&tree.path().join("b.log"), b"0123456789");
    let base = || {
        request(
            &bench.execution,
            local(tree.path()),
            vec![include("*.log")],
            FileWatchPolicy::Mirror,
        )
    };

    // The File Cap Refuses The Second File, After Sealing The First
    let mut capped = bench
        .handle
        .open_file_watch(
            &FileWatch {
                maximum_files: Some(1),
                ..base()
            },
            None,
        )
        .await
        .unwrap();
    let mut events = Vec::new();
    let refused = capped.poll(&mut events).await.unwrap_err();
    assert_eq!(code(&refused), CAPPED, "{refused}");
    let sealed = sealed(&seal(&mut capped).await);
    assert_artifact(&sealed["a.log"], "a.log", b"0123456789");

    // The Byte Cap Refuses A File That Would Exceed It
    let mut small = bench
        .handle
        .open_file_watch(
            &FileWatch {
                maximum_bytes: Some(15),
                ..base()
            },
            None,
        )
        .await
        .unwrap();
    let refused = small.poll(&mut Vec::new()).await.unwrap_err();
    assert_eq!(code(&refused), CAPPED, "{refused}");
    seal(&mut small).await;

    // Requests Are Refused Before A Source Opens
    let refusals = [
        FileWatch {
            execution_id: None,
            ..base()
        },
        FileWatch {
            rules: vec![exclude("*.log")],
            ..base()
        },
        FileWatch {
            output: "../escape".to_owned(),
            ..base()
        },
        FileWatch {
            policy: FileWatchPolicy::Unspecified,
            ..base()
        },
        FileWatch {
            poll_interval_ms: Some(1),
            ..base()
        },
        FileWatch {
            maximum_files: Some(257),
            ..base()
        },
        FileWatch {
            source: None,
            ..base()
        },
    ];
    for refused in refusals {
        let error = bench
            .handle
            .open_file_watch(&refused, None)
            .await
            .err()
            .unwrap();
        assert_eq!(code(&error), INVALID, "{error}");
    }

    // The Context Execution Must Agree With An Explicit One
    let error = bench
        .handle
        .open_file_watch(&base(), Some("00000000-0000-7000-8000-000000000000"))
        .await
        .err()
        .unwrap();
    assert_eq!(code(&error), INVALID, "{error}");
    let scoped = bench
        .handle
        .open_file_watch(
            &FileWatch {
                execution_id: None,
                ..base()
            },
            Some(&bench.execution),
        )
        .await;
    assert!(
        scoped.is_ok(),
        "the calling step's execution scopes the watch"
    );
    drop(scoped);

    // FTP Sources Refuse Untrusted Plain FTP, SFTP And Inline Passwords
    let ftp = |url: &str| FileWatch {
        source: Some(model::files::wire::FileWatchSource {
            payload: Some(FileWatchSourcePayload::Ftp(FtpWatchSource {
                url: url.to_owned(),
                credential_id: None,
            })),
        }),
        ..base()
    };
    for (url, expected) in [
        ("ftp://127.0.0.1:1/logs", UNTRUSTED),
        ("sftp://127.0.0.1/logs", UNSUPPORTED),
        ("ftps://bench:secret@127.0.0.1/logs", INVALID),
        ("http://127.0.0.1/logs", INVALID),
    ] {
        let error = bench
            .handle
            .open_file_watch(&ftp(url), None)
            .await
            .err()
            .unwrap();
        assert_eq!(code(&error), expected, "{url}: {error}");
    }

    bench.stop().await;
}

// A failed refetch keeps the last whole copy: the output never holds a
// partial or emptied file.
#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn mirror_keeps_last() {
    use std::os::unix::fs::PermissionsExt;
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    let report = tree.path().join("report.csv");
    write(&report, b"v1\n");
    let mut watch = bench
        .handle
        .open_file_watch(
            &request(
                &bench.execution,
                local(tree.path()),
                vec![include("*.csv")],
                FileWatchPolicy::Mirror,
            ),
            None,
        )
        .await
        .unwrap();
    assert_eq!(
        phases(&poll(&mut watch).await, Phase::Synced),
        vec![("report.csv".to_owned(), 3)]
    );

    // The Source Changes But Cannot Be Read
    write(&report, b"version two, longer\n");
    std::fs::set_permissions(&report, std::fs::Permissions::from_mode(0o000)).unwrap();
    let unreadable = std::fs::File::open(&report).is_err();
    let refetch = poll(&mut watch).await;
    let sealed = sealed(&seal(&mut watch).await);
    std::fs::set_permissions(&report, std::fs::Permissions::from_mode(0o644)).unwrap();
    if unreadable {
        assert!(phases(&refetch, Phase::Synced).is_empty());
        assert_artifact(&sealed["report.csv"], "report.csv", b"v1\n");
    } else {
        eprintln!("Skipped the unreadable refetch: this user reads any file");
    }

    drop(watch);
    bench.stop().await;
}

// Closing the execution seals its running watch into it first.
#[tokio::test(flavor = "multi_thread")]
async fn execution_seals_watch() {
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    let log = tree.path().join("run.log");
    write(&log, b"");
    let mut watch = bench
        .handle
        .open_file_watch(
            &request(
                &bench.execution,
                local(tree.path()),
                vec![include("*.log")],
                FileWatchPolicy::Stream,
            ),
            None,
        )
        .await
        .unwrap();
    poll(&mut watch).await;
    append(&log, b"captured before the execution closed\n");

    // Poll Once For The Appended Line, Then Drive The Watch Until The Execution Asks It To Seal
    poll(&mut watch).await;
    let driver = tokio::spawn(async move {
        watch.seal_requested().await;
        let mut events = Vec::new();
        watch.seal(&mut events).await.unwrap();
        sealed(&events)
    });
    bench.executor.release();
    let sealed = driver.await.unwrap();
    assert_artifact(
        &sealed["run.log"],
        "run.log",
        b"captured before the execution closed\n",
    );
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
#[tokio::test(flavor = "multi_thread")]
async fn watch_file_matchers() {
    const APPENDED: &[u8] = b"elapsed=1.5 locked=true\nWARN W42\nelapsed=2.5 locked=false\n";
    let bench = Bench::start(|config, _| config).await;
    let tree = tempdir().unwrap();
    let log = tree.path().join("device.log");
    write(&log, b"elapsed=0.5 locked=false\n");
    let request = WatchFile {
        execution_id: Some(bench.execution.clone()),
        source: Some(FileWatchSource {
            payload: Some(local(&log)),
        }),
        output: OUTPUT.to_owned(),
        artifact: Some(format!("{OUTPUT}/device.log")),
        policy: FileWatchPolicy::Stream,
        poll_interval_ms: Some(50),
        maximum_seconds: Some(2),
        maximum_bytes: None,
        matchers: vec![LineMatcher {
            storage_schema: crate::service_support::contracts::minimal_tracking_contract()
                .storage_schema
                .unwrap(),
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
        markers: vec![LineMarker::new(
            r"WARN (?P<warning_code>\w+)",
            "tracking.fixture_warning",
        )],
    };
    let watch = bench
        .handle
        .operations()
        .submit_as::<WatchFile>(request)
        .unwrap();

    // Lines Written After The Start Are Captured And Matched
    // TODO(source-hook): the WatchFile operation signals no readiness once it tails the file; this sleep stands in for it.
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;
    append(&log, APPENDED);
    let outcome = watch.join().await.unwrap().into_owned();
    assert_eq!(
        outcome.artifacts,
        vec![WatchedArtifact {
            artifact: format!("{OUTPUT}/device.log"),
            size_bytes: APPENDED.len() as u64,
        }]
    );
    assert!(outcome.gaps.is_empty());

    // Rows Reach The Declared Stream In The Execution's Scope
    let execution = dokime_runtime::TestExecutionId::from_uuid(bench.execution.clone()).unwrap();
    let rows = bench
        .handle
        .collect_telemetry(&TelemetrySelection {
            streams: vec![identifier("tracking")],
            scope: ScopeSelection {
                test_execution_id: Some(execution.clone()),
                ..Default::default()
            },
            ..Default::default()
        })
        .await;
    assert_eq!(
        rows.iter()
            .map(|row| (
                row.values.get("elapsed_s").cloned(),
                row.values.get("target_locked").cloned()
            ))
            .collect::<Vec<_>>(),
        vec![
            (Some(Value::Number(1.5)), Some(Value::Bool(true))),
            (Some(Value::Number(2.5)), Some(Value::Bool(false))),
        ]
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
    assert_eq!(
        markers[0].record.details.get("warning_code"),
        Some(&Value::Text("W42".to_owned()))
    );
    assert_eq!(markers[0].record.message.as_deref(), Some("WARN W42"));

    // The Artifact Holds Only The Appended Bytes
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
    let content = bench
        .handle
        .read_artifact_content(&record.artifact_id, APPENDED.len())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(content.bytes, APPENDED);
    bench.stop().await;
}
