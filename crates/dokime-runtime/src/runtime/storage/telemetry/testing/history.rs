/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Pending Admission, Window Reads And Follow On One Saved History

use super::{Reading, StorageConfig, TestApp};
use crate::runtime::storage::telemetry::{ReadOptions, collect_page, prepare_staged, visit_saved};
use crate::storage::{Manager, telemetry};
use harmos::{Runtime, storage::StorageBackends};
use std::sync::Arc;

pub async fn telemetry(days: u64, per_day: u64) {
    use std::num::NonZeroUsize;
    let root = tempfile::tempdir().unwrap();
    let backend = Arc::new(crate::storage::Backend::primary(root.path().to_owned()).await.unwrap());
    let local = backend.local_state();
    let state = Arc::new(harmos::storage::state::file::FileBackend::new(
        root.path().to_owned(),
        harmos::storage::StorageBackend::history(backend.as_ref()).clone(),
        crate::config::storage_state_defaults(),
    ));
    let mut settings = crate::config::storage_defaults();
    settings.request.max_pending_write_batches = NonZeroUsize::new(2).unwrap();
    settings.request.max_pending_write_bytes = NonZeroUsize::new(128 * 1024).unwrap();
    let capacity = settings.request.max_pending_write_bytes.get();
    let storage = Manager::new(StorageBackends { primary: backend, state }, settings).unwrap();
    let runtime = Runtime::<TestApp>::launch((), StorageConfig { storage: Some(storage.clone()) }).await.unwrap();
    let contract = crate::telemetry::TelemetryContract::declared("fixture.pending", "Pending").typed::<Reading>();
    let codec = telemetry::codec(&contract, &Default::default()).unwrap();
    storage.write_schema(codec.schema().clone(), Arc::clone(codec.graph())).await.unwrap();
    storage.flush().await.unwrap();
    let epoch = dokime_domain::model::identity::SessionId::from_seed("pending.epoch");
    let row = |minute: u64| {
        let sample = crate::telemetry::TelemetrySample::new(
            dokime_domain::model::identity::TelemetrySampleId::from_seed(format!("pending:{minute}")),
            contract.stream.clone(),
            minute,
            dokime_domain::model::time::Timestamp::from_datetime(time::OffsetDateTime::UNIX_EPOCH + time::Duration::minutes(minute as i64)),
            "fixture.pending",
        )
        .typed(&Reading { value: "bounded".into() });
        prepare_staged(&epoch, &sample, None, minute, &codec).unwrap().into_input()
    };
    assert!(per_day > 100);
    let streams = vec![contract.stream.clone().into()];
    let selection = crate::evidence_runtime::TelemetrySelection {
        streams: streams.clone(),
        ..Default::default()
    };
    let paged = crate::evidence_runtime::TelemetrySelection {
        streams: streams.clone(),
        limit: Some(100),
        ..Default::default()
    };
    let whole = crate::evidence_runtime::TelemetrySelection {
        streams,
        limit: Some(per_day as usize),
        ..Default::default()
    };
    let (mut peak_bytes, mut peak_batches) = (0, 0);
    let (mut settled, mut window_work, mut follow_work) = (0, None, None);
    for day in 0..days {
        // Hold The Actual SQLite Writer; Admission Must Respect Its Finite Queue.
        let gate = local.clone();
        let (entered, ready) = tokio::sync::oneshot::channel();
        let (release, resume) = std::sync::mpsc::channel();
        let held = tokio::spawn(async move {
            gate.call(move |_| {
                let _ = entered.send(());
                let _ = resume.recv();
                Ok(())
            })
            .await
            .unwrap();
        });
        ready.await.unwrap();
        let before = storage.status();
        let first = day * per_day;
        storage.write_stream(vec![row(first)]).await.unwrap();
        storage.write_stream(vec![row(first + 1)]).await.unwrap();
        let full = storage.status();
        assert_eq!(full.pending_batches, 2);
        assert_eq!(full.accepted.sequence, before.accepted.sequence + 2);
        assert_eq!(full.saved, before.saved);
        assert!(full.pending_bytes > 0 && full.pending_bytes <= capacity);
        peak_bytes = peak_bytes.max(full.pending_bytes);
        peak_batches = peak_batches.max(full.pending_batches);

        let mut blocked = std::pin::pin!(storage.write_stream(vec![row(first + 2)]));
        std::future::poll_fn(|context| {
            assert!(std::future::Future::poll(blocked.as_mut(), context).is_pending());
            std::task::Poll::Ready(())
        })
        .await;
        assert_eq!(storage.status().accepted.sequence, full.accepted.sequence);
        release.send(()).unwrap();
        held.await.unwrap();
        blocked.await.unwrap();
        let drained = storage.flush().await.unwrap();
        assert_eq!(drained.sequence, before.accepted.sequence + 3);

        // Grow Saved History Without Retaining It In The Pending Queue.
        for start in (3..per_day).step_by(16) {
            let batch = (start..(start + 16).min(per_day)).map(|offset| row(first + offset)).collect();
            storage.write_stream(batch).await.unwrap();
            let current = storage.status();
            peak_bytes = peak_bytes.max(current.pending_bytes);
            peak_batches = peak_batches.max(current.pending_batches);
            assert!(peak_bytes <= capacity);
            assert!(peak_batches <= 2);
        }
        let saved = storage.flush().await.unwrap();
        assert_eq!(saved.sequence, before.accepted.sequence + per_day);
        assert_eq!(storage.status().pending_bytes, 0);
        assert_eq!(storage.status().pending_batches, 0);

        // The Same History Supports A Fixed Window With Constant Anchor And Row Work.
        let before = local.read_counters();
        let mut rows = Vec::new();
        let options = ReadOptions {
            window: Some(("fixture.pending", 300.0)),
            ..Default::default()
        };
        visit_saved(&storage, &saved, &selection, &Default::default(), options, &local, |sample| {
            rows.push(sample);
            Ok(())
        })
        .await
        .unwrap();
        let after = local.read_counters();
        let scanned = after.data_scan_rows - before.data_scan_rows;
        assert_eq!(rows.len(), 6);
        assert_eq!(after.telemetry_anchor_rows - before.telemetry_anchor_rows, 1);
        assert!(scanned <= 16);
        if let Some(previous) = window_work {
            assert_eq!(scanned, previous);
        }
        window_work = Some(scanned);

        // Follow Stops At The Limited Page, Then Resumes Only The Unsettled Day.
        let options = ReadOptions { after: settled, ..Default::default() };
        let page = collect_page(&storage, &saved, &paged, &Default::default(), options, &local).await.unwrap();
        assert!(page.limited);
        assert_eq!(page.samples.len(), 100);
        assert_eq!(page.samples.first().map(|sample| sample.sequence), Some(first));
        let last = *page.positions.last().unwrap();
        assert!(page.positions.windows(2).all(|pair| pair[0] < pair[1]));
        let before = local.read_counters();
        let options = ReadOptions { after: last, ..Default::default() };
        let rest = collect_page(&storage, &saved, &whole, &Default::default(), options, &local).await.unwrap();
        let scanned = local.read_counters().data_scan_rows - before.data_scan_rows;
        assert!(!rest.limited);
        assert_eq!(rest.samples.len(), per_day as usize - 100);
        assert_eq!(rest.samples.first().map(|sample| sample.sequence), Some(first + 100));
        assert!(scanned <= per_day);
        if let Some(previous) = follow_work {
            assert_eq!(scanned, previous);
        }
        follow_work = Some(scanned);
        settled = saved.sequence;
    }
    assert!(peak_bytes > 0 && peak_bytes <= capacity);
    assert_eq!(peak_batches, 2);
    // Read The Same Durable History Back In Bounded Pages After Pressure Has Settled.
    let selection = crate::evidence_runtime::TelemetrySelection {
        streams: vec![contract.stream.clone().into()],
        limit: Some(64),
        ..Default::default()
    };
    let at = storage.flush().await.unwrap();
    let (mut after, mut read) = (0, 0);
    loop {
        let options = ReadOptions { after, ..Default::default() };
        let page = collect_page(&storage, &at, &selection, &Default::default(), options, &local).await.unwrap();
        assert!(page.samples.len() <= 64);
        if page.samples.is_empty() {
            break;
        }
        for sample in page.samples.iter() {
            assert_eq!(sample.sequence, read);
            read += 1;
        }
        after = *page.positions.last().unwrap();
    }
    assert_eq!(read, days * per_day);
    runtime.shutdown().await.unwrap();
}
