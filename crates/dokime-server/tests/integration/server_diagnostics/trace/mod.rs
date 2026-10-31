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
use tracing_subscriber::prelude::*;
// Local Imports
use super::*;
use dokime::domain::model;
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

fn emit(journal: std::sync::Arc<Journal>, execution_id: &str, count: usize) {
    let subscriber = tracing_subscriber::registry().with(Capture::new(journal));
    tracing::subscriber::with_default(subscriber, || {
        for index in 0..count {
            tracing::info!(execution_id, api_token = "trace-fake-secret", index, "fixture trace");
        }
    });
}

fn request(execution_id: &str, after: u64, follow: bool) -> contracts::backend::WatchTrace {
    contracts::backend::WatchTrace {
        epoch: None,
        scope: Some(model::backend::wire::TraceScope::Execution(execution_id.to_owned())),
        min_level: model::backend::wire::LogLevel::Info,
        after_sequence: after,
        max_events: 2,
        max_bytes: 32768,
        follow,
    }
}

/* === Tests === */

pub(super) async fn roundtrip(environment: &support::TestEnvironment, client: &WireClient, journal: &std::sync::Arc<Journal>) {
    let execution = uuid::Uuid::now_v7().to_string();
    let mut unavailable = client.download::<contracts::backend::WatchTrace>(request(&execution, 0, true)).await.unwrap().into_inner();
    assert_eq!(unavailable.message().await.unwrap_err().code(), Code::Unavailable);
    emit(journal.clone(), &execution, 3);
    let mut stream = client.download::<contracts::backend::WatchTrace>(request(&execution, 0, false)).await.unwrap().into_inner();
    let first = stream.message().await.unwrap().unwrap();
    assert_eq!(first.epoch, environment.runtime.process_epoch().to_string());
    assert_eq!(first.events.len(), 2);
    assert_eq!(first.dropped, 0);
    assert!(first.events.windows(2).all(|events| events[0].sequence < events[1].sequence));
    assert!(first.events.iter().all(|event| event.execution_id.as_deref() == Some(execution.as_str())));
    assert!(first.events[0].fields.iter().any(|field| field.key == "api_token" && field.value == "<redacted>"));
    let second = stream.message().await.unwrap().unwrap();
    assert_eq!(second.events.len(), 1);
    assert!(second.next_sequence > first.next_sequence);
    // Exhausting The Stream Also Verifies Its Final OK Status.
    assert!(stream.message().await.unwrap().is_none());

    let mut resumed = client
        .download::<contracts::backend::WatchTrace>(contracts::backend::WatchTrace {
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

    for (epoch, after) in
        [(None, first.next_sequence), (Some(uuid::Uuid::now_v7().to_string()), first.next_sequence), (Some(first.epoch.clone()), u64::MAX)]
    {
        let mut invalid = client
            .download::<contracts::backend::WatchTrace>(contracts::backend::WatchTrace {
                epoch,
                ..request(&execution, after, false)
            })
            .await
            .unwrap()
            .into_inner();
        assert_eq!(invalid.message().await.unwrap_err().code(), Code::OutOfRange);
    }

    emit(journal.clone(), &execution, 1);
    let mut stream = client
        .download::<contracts::backend::WatchTrace>(contracts::backend::WatchTrace {
            epoch: Some(first.epoch.clone()),
            ..request(&execution, second.next_sequence, true)
        })
        .await
        .unwrap()
        .into_inner();
    let initial = stream.message().await.unwrap().unwrap();
    assert_eq!(initial.events.len(), 1);
    emit(journal.clone(), &execution, 1);
    let live = stream.message().await.unwrap().unwrap();
    assert_eq!(live.events.len(), 1);
    assert!(live.next_sequence > initial.next_sequence);
    drop(stream);
    let mut reconnect = client
        .download::<contracts::backend::WatchTrace>(contracts::backend::WatchTrace {
            epoch: Some(first.epoch.clone()),
            ..request(&execution, initial.next_sequence, false)
        })
        .await
        .unwrap()
        .into_inner();
    let recovered = reconnect.message().await.unwrap().unwrap();
    assert_eq!(recovered.events, live.events);
    assert!(reconnect.message().await.unwrap().is_none());
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn scope_history_tail() {
    use model::backend::wire::{LogLevel, TraceScope};

    let journal = Journal::new(dokime_server::trace::Limits::default());
    let subscriber = tracing::Dispatch::new(tracing_subscriber::registry().with(dokime_server::trace::layer(journal.clone())));
    let (environment, server, client) = diagnostics::fixture(journal.clone()).await;
    for selection in 0..3 {
        let cycle_id = uuid::Uuid::now_v7().to_string();
        let execution_id = uuid::Uuid::now_v7().to_string();
        let scope = match selection {
            0 => TraceScope::Instance,
            1 => TraceScope::Cycle(cycle_id.clone()),
            _ => TraceScope::Execution(execution_id.clone()),
        };
        let watch = contracts::backend::WatchTrace {
            scope: Some(scope),
            ..request(&execution_id, 0, true)
        };
        let mut stream = client.download::<contracts::backend::WatchTrace>(watch.clone()).await.unwrap().into_inner();
        let empty = stream.message().await.unwrap().unwrap();
        assert!(empty.events.is_empty());

        tracing::dispatcher::with_default(&subscriber, || {
            tracing::debug!(target: "dokime_server", %cycle_id, %execution_id, "filtered");
        });
        let filtered = stream.message().await.unwrap().unwrap();
        assert!(filtered.events.is_empty());
        assert!(filtered.next_sequence > empty.next_sequence);
        tracing::dispatcher::with_default(&subscriber, || {
            tracing::info!(%cycle_id, %execution_id, "history");
        });
        let history = stream.message().await.unwrap().unwrap();
        assert_eq!(history.events.len(), 1);
        assert_eq!(history.events[0].level, LogLevel::Info);
        assert_eq!(history.events[0].cycle_id.as_deref(), Some(cycle_id.as_str()));
        assert_eq!(history.events[0].execution_id.as_deref(), Some(execution_id.as_str()));
        drop(stream);

        let mut replay = client.download::<contracts::backend::WatchTrace>(watch).await.unwrap().into_inner();
        let retained = replay.message().await.unwrap().unwrap();
        assert_eq!(retained.events, history.events);
        assert_eq!(retained.next_sequence, history.next_sequence);
        tracing::dispatcher::with_default(&subscriber, || {
            tracing::warn!(%cycle_id, %execution_id, "live");
        });
        let live = replay.message().await.unwrap().unwrap();
        assert_eq!(live.events.len(), 1);
        assert_eq!(live.events[0].level, LogLevel::Warn);
        assert!(live.next_sequence > retained.next_sequence);
        drop(replay);
    }
    server.shutdown().await.unwrap();
    drop(environment);
}
