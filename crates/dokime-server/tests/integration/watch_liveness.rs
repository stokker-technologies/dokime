/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Quiet Plot And Log Watches Survive Their Configured Idle Period

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::io::Write as _;

// Third Party Imports

// Local Imports
use crate::support::{
    self,
    plots::{publish, request, served_points},
    wire::runtime,
};
use dokime::{
    Identifier,
    client::{Client, Target, contracts},
    domain::model,
};
use dokime_server::grpc::Config;

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
async fn watch_liveness() {
    let root = tempfile::tempdir().unwrap();
    let prepared = dokime_server::grpc::Service::prepare(Config::local()).unwrap();
    let mut owner = runtime(root.path(), &prepared).await;
    let handle = owner.handle();
    support::register_fixture_progress_and_idle_execution(&handle, support::fixture_suite(&["passed"])).await;
    let id = Identifier::new("fixture.progress").unwrap();
    publish(&handle, &id, 0, 10).await;
    let log = root.path().join("application/logs/dokime.jsonl");
    std::fs::create_dir_all(log.parent().unwrap()).unwrap();
    let record = br#"{"timestamp":"2026-09-30T14:22:33.000001Z","level":"ERROR","target":"fixture","fields":{"message":"fixture"}}
"#;
    std::fs::write(&log, record).unwrap();
    let server = prepared.ready().await.unwrap().listen().await.unwrap();
    let client = Client::connect(Target::local(server.local_endpoint()).unwrap(), None, None).await.unwrap();
    let plots = client.stream(contracts::plots::Watch { requests: vec![request(10_000)] }).await.unwrap();
    let opened = loop {
        let frame = plots.message().await.unwrap().unwrap();
        if frame.kind != model::plots::wire::PlotStreamKind::Unspecified {
            break frame;
        }
    };
    assert_eq!(opened.kind, model::plots::wire::PlotStreamKind::Snapshot);
    assert_eq!(served_points(&opened), 10);
    let logs = client
        .stream(contracts::backend::WatchLogs {
            min_level: model::backend::wire::LogLevel::Error,
            replay_lines: 500,
        })
        .await
        .unwrap();
    assert_eq!(logs.message().await.unwrap().unwrap().level, model::backend::wire::LogLevel::Error);

    // A Blocking Read Inhibits Tokio Auto-Advance While Real Network Events Settle.
    let (release, wait) = std::sync::mpsc::channel::<()>();
    let (entered, ready) = tokio::sync::oneshot::channel();
    let held = tokio::task::spawn_blocking(move || {
        let _ = entered.send(());
        let _ = wait.recv();
    });
    ready.await.unwrap();
    tokio::time::pause();
    let idle = std::time::Duration::from_millis(client.settings().request_timeout_ms);
    let heartbeat = dokime_server::grpc::WATCH_HEARTBEAT;
    assert!(heartbeat < idle);
    // Cross Each Heartbeat Deadline, Including Timer Rounding, Without Crossing The Client Idle Deadline.
    let step = heartbeat + (idle - heartbeat) / 2;
    assert!(heartbeat < step && step < idle);
    let beats = usize::try_from(idle.as_nanos() / step.as_nanos()).unwrap() + 1;
    for _ in 0..beats {
        tokio::time::advance(step).await;
        let frame = plots.message().await.unwrap().unwrap();
        assert_eq!(frame.kind, model::plots::wire::PlotStreamKind::Unspecified);
        assert!(frame.envelope.unwrap().payload.is_none());
        let frame = logs.message().await.unwrap().unwrap();
        assert_eq!(frame.level, model::backend::wire::LogLevel::Unspecified);
        assert!(frame.time.is_none() && frame.message.is_empty());
    }
    tokio::time::resume();
    drop(release);
    held.await.unwrap();

    // Both Live Paths Still Deliver Actual Data After The Simulated Quiet Period.
    publish(&handle, &id, 10, 1).await;
    let updated = loop {
        let frame = plots.message().await.unwrap().unwrap();
        if frame.kind != model::plots::wire::PlotStreamKind::Unspecified {
            break frame;
        }
    };
    assert_ne!(updated.kind, model::plots::wire::PlotStreamKind::Error);
    assert_eq!(served_points(&updated), 11);
    std::fs::OpenOptions::new().append(true).open(&log).unwrap().write_all(record).unwrap();
    let updated = loop {
        let frame = logs.message().await.unwrap().unwrap();
        if frame.level != model::backend::wire::LogLevel::Unspecified {
            break frame;
        }
    };
    assert_eq!(updated.level, model::backend::wire::LogLevel::Error);
    plots.close().await;
    logs.close().await;
    client.close_and_wait().await;
    server.shutdown().await.unwrap();
    drop(handle);
    owner.shutdown().await.unwrap();
}
