/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

#[allow(dead_code)]
#[path = "../../../src/grpc/logs.rs"]
mod implementation;
#[allow(dead_code)]
#[path = "../../../src/grpc/redaction.rs"]
mod redaction;

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports

use dokime::domain::model::backend::wire::LogLevel;
use dokime_server::grpc::WATCH_HEARTBEAT;
use implementation::testing::LogProbe;
use std::io::Write as _;

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

fn line(level: &str, message: &str) -> String {
    format!(r#"{{"level":"{level}","target":"dokime::fixture","fields":{{"message":"{message}"}}}}"#)
}

#[tokio::test]
async fn tail_reads_only_appended_bytes() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("dokime.jsonl");
    let first = format!("{}\n{}\n", line("INFO", "started"), line("WARN", "slow"));
    std::fs::write(&path, &first).unwrap();
    let tail = LogProbe::new(path.clone());
    assert_eq!(tail.caught_up().await, first.len() as u64);
    let mut total = first.len() as u64;
    for index in 0..32 {
        let appended = format!("{}\n", line("ERROR", &format!("failed {index}")));
        std::fs::OpenOptions::new().append(true).open(&path).unwrap().write_all(appended.as_bytes()).unwrap();
        total += appended.len() as u64;
        // Each Poll Reads Exactly The Appended Record, Never The File Again.
        assert_eq!(tail.caught_up().await, total);
    }
    assert_eq!(tail.shared().0, 34);
}

#[tokio::test]
async fn subscribers_share_one_parsed_record() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("dokime.jsonl");
    std::fs::write(&path, format!("{}\n{}\n", line("INFO", "started"), line("WARN", "slow"))).unwrap();
    let tail = LogProbe::new(path.clone());
    let warn = tail.subscribe(LogLevel::Warn).await;
    let all = tail.subscribe(LogLevel::Trace).await;
    assert_eq!(warn.len(), 1);
    assert_eq!(all.len(), 2);
    assert!(warn.same_record(0, &all, 1), "one parse serves every subscriber");
    // The Second Subscriber Found The Reader Caught Up, So The File Was Read Once.
    let read = std::fs::metadata(&path).unwrap().len();
    assert_eq!(tail.reader().await.0, read);
}

#[tokio::test]
async fn partial_lines_resume_without_rereading_or_duplicate_records() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("dokime.jsonl");
    let record = line("INFO", "completed");
    let split = record.len() / 2;
    tokio::fs::write(&path, &record[..split]).await.unwrap();
    let tail = LogProbe::new(path.clone());
    assert_eq!(tail.caught_up().await, split as u64);
    assert_eq!(tail.shared().0, 0);

    let mut file = tokio::fs::OpenOptions::new().append(true).open(&path).await.unwrap();
    let appended = format!("{}\n", &record[split..]);
    tokio::io::AsyncWriteExt::write_all(&mut file, appended.as_bytes()).await.unwrap();
    file.sync_all().await.unwrap();
    assert_eq!(tail.caught_up().await, record.len() as u64 + 1);
    assert_eq!(tail.caught_up().await, record.len() as u64 + 1);
    assert_eq!(tail.shared().0, 1);
    assert!(tail.reader().await.1);
}

#[tokio::test]
async fn subscriber_replays_only_records_from_the_truncated_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("dokime.jsonl");
    tokio::fs::write(&path, format!("{}\n", line("INFO", "old record"))).await.unwrap();
    let tail = LogProbe::new(path.clone());
    tail.caught_up().await;
    tokio::fs::write(&path, format!("{}\n", line("INFO", "new"))).await.unwrap();

    let subscription = tail.subscribe(LogLevel::Trace).await;
    assert_eq!(subscription.len(), 1);
    assert_eq!(subscription.message(0), "new");
}
