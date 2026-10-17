/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */
mod journal;
/* === Re-Exports === */
/* === Imports === */
// Standard Library Imports
// Third Party Imports
use tracing_subscriber::prelude::*;
// Local Imports
use super::*;
use dokime_server::trace::{Capture, Journal};

/* === Constants === */
/* === Macros === */
/* === Type Aliases === */
/* === Structs === */
/* === Enums === */
/* === Traits === */
/* === Implementations === */
/* === Public Functions === */
/* === Private Functions === */
fn emit(execution_id: &str, count: usize) {
    let subscriber = tracing_subscriber::registry().with(Capture::new(Journal::global().clone()));
    tracing::subscriber::with_default(subscriber, || {
        for index in 0..count {
            tracing::info!(
                execution_id,
                api_token = "trace-fake-secret",
                index,
                "fixture trace"
            );
        }
    });
}

fn request(execution_id: &str, after: u64, follow: bool) -> contracts::backend::WatchTraceRequest {
    contracts::backend::WatchTraceRequest {
        epoch: None,
        scope: Some(contracts::backend::TraceScope::Execution(
            execution_id.to_owned(),
        )),
        min_level: contracts::backend::LogLevel::Info,
        after_sequence: after,
        max_events: 2,
        max_bytes: 32768,
        follow,
    }
}

/* === Tests === */
#[tokio::test]
async fn trace_watch_replays() {
    let environment = support::TestEnvironment::with_installed_plugins(Vec::new()).await;
    let server = Server::start(environment.runtime.clone(), Config::default())
        .await
        .unwrap();
    let client = connect(&server).await;
    let execution = uuid::Uuid::now_v7().to_string();
    emit(&execution, 3);
    let mut stream = client
        .download::<contracts::backend::WatchTrace>(request(&execution, 0, false))
        .await
        .unwrap()
        .into_inner();
    let first = stream.message().await
        .unwrap()
        .unwrap();
    assert_eq!(first.epoch, environment.runtime.process_epoch().to_string());
    assert_eq!(first.events.len(), 2);
    assert_eq!(first.dropped, 0);
    assert!(
        first
            .events
            .windows(2)
            .all(|events| events[0].sequence < events[1].sequence)
    );
    assert!(
        first
            .events
            .iter()
            .all(|event| event.execution_id.as_deref() == Some(execution.as_str()))
    );
    assert!(
        first.events[0]
            .fields
            .iter()
            .any(|field| field.key == "api_token" && field.value == "<redacted>")
    );
    let second = stream.message().await.unwrap().unwrap();
    assert_eq!(second.events.len(), 1);
    assert!(second.next_sequence > first.next_sequence);
    // Exhausting The Stream Also Verifies Its Final OK Status.
    assert!(stream.message().await.unwrap().is_none());

    let mut resumed = client
        .download::<contracts::backend::WatchTrace>(contracts::backend::WatchTraceRequest {
            epoch: Some(first.epoch.clone()),
            ..request(&execution, first.next_sequence, false)
        })
        .await
        .unwrap()
        .into_inner();
    let resumed_page = resumed.message().await.unwrap().unwrap();
    assert_eq!(resumed_page.events.len(), 1);
    assert_eq!(resumed_page.events[0].sequence, second.events[0].sequence);
    assert!(resumed.message().await.unwrap().is_none());

    for (epoch, after) in [
        (None, first.next_sequence),
        (Some(uuid::Uuid::now_v7().to_string()), first.next_sequence),
        (Some(first.epoch.clone()), u64::MAX),
    ] {
        let mut invalid = client
            .download::<contracts::backend::WatchTrace>(contracts::backend::WatchTraceRequest {
                epoch,
                ..request(&execution, after, false)
            })
            .await
            .unwrap()
            .into_inner();
        assert_eq!(
            invalid.message().await.unwrap_err().code(),
            Code::OutOfRange
        );
    }
    server.shutdown().await.unwrap();
}

#[tokio::test]
async fn trace_watch_follows() {
    let environment = support::TestEnvironment::with_installed_plugins(Vec::new()).await;
    let server = Server::start(environment.runtime.clone(), Config::default())
        .await
        .unwrap();
    let client = connect(&server).await;
    let execution = uuid::Uuid::now_v7().to_string();
    emit(&execution, 1);
    let mut stream = client
        .download::<contracts::backend::WatchTrace>(request(&execution, 0, true))
        .await
        .unwrap()
        .into_inner();
    let initial = stream.message().await.unwrap().unwrap();
    assert_eq!(initial.events.len(), 1);
    emit(&execution, 1);
    let live = stream.message().await
        .unwrap()
        .unwrap();
    assert_eq!(live.events.len(), 1);
    assert!(live.next_sequence > initial.next_sequence);
    drop(stream);
    server.shutdown().await
        .unwrap();
}
