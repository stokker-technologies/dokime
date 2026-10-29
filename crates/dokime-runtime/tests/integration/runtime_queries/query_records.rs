/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use model::execution::OperatorNoteRequest;
use {
    model::records::product::ProductCalibrationDetails, model::records::product::ProductCalibrationKind,
    model::records::product::ProductCalibrationPhase,
};

use crate::service_support::suites::ExecutionFixtureStart;
use dokime_domain::model;
use dokime_runtime::domains::evidence::RecordSelection;
use dokime_runtime::{Identifier, Metadata};
use dokime_runtime::{Runtime, RuntimeConfig, RuntimeHandle};

fn runtime_config(root: &std::path::Path) -> RuntimeConfig {
    crate::service_support::budgets::runtime_config(
        root,
        Metadata::new(Identifier::new("fixture.records").unwrap(), "Record Query Fixture", None),
    )
}

async fn runtime(root: &std::path::Path) -> Runtime {
    Runtime::start(runtime_config(root)).await.unwrap()
}

async fn publish(handle: &RuntimeHandle) {
    handle
        .publish_product_record(
            ProductCalibrationDetails::new("fixture-device", ProductCalibrationKind::Field, ProductCalibrationPhase::Completed)
                .into_draft("fixture.product"),
        )
        .await
        .unwrap();
}

async fn poison_capture(handle: &RuntimeHandle) {
    // Damage only this fixture's primary store. Projection refusal is healthy;
    // a physical persistence failure must instead close admission and readers.
    // Save everything already admitted first, so a background save of earlier
    // groups never meets the damage before the publication below does.
    handle.storage().flush().await.unwrap();
    let primary = handle.storage_root().join("local.sqlite");
    let connection = rusqlite::Connection::open(primary).unwrap();
    connection.execute_batch("DROP TABLE storage_data_entries").unwrap();
    publish(handle).await;
    // Admission is buffered. Force the damaged primary through its save barrier
    // before asserting that the manager's failure reaches every reader.
    let saved = handle.storage().flush().await;
    assert!(saved.is_err());
    assert!(handle.storage().status().failure.is_some());
    let captured = handle.captured_records();
    assert!(captured.next(0, captured.frontier()).await.is_err());
}

pub(crate) async fn oversized_record_changes() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    publish(&handle).await;
    let reader = handle.captured_records();
    let count = handle.watch_state().latest().state().record_count;
    let before = reader.next(0, reader.frontier()).await.unwrap().unwrap();
    let mut draft = ProductCalibrationDetails::new("fixture-device", ProductCalibrationKind::Field, ProductCalibrationPhase::Completed)
        .into_draft("fixture.product");
    draft.message = Some("x".repeat(1024 * 1024));
    assert!(handle.publish_product_record(draft).await.is_err());
    // The public API records a separate health warning for rejected producers.
    // That diagnostic may advance the journal; the refused product fact may not
    // change evidence visibility or the product record count.
    assert_eq!(handle.watch_state().latest().state().record_count, count);
    assert_eq!(reader.next(0, reader.frontier()).await.unwrap().as_deref(), Some(&*before));
    assert_eq!(handle.selected_records(RecordSelection::default()).await.unwrap().len(), 1);
    publish(&handle).await;
    assert_eq!(handle.selected_records(RecordSelection::default()).await.unwrap().len(), 2);
    drop(reader);
    drop(handle);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn published_record_reads() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    publish(&handle).await;
    let reader = handle.captured_records();
    let record = reader.next(0, reader.frontier()).await.unwrap().unwrap();
    assert_eq!(record.record.record_type.as_str(), "calibration.field.completed");
    drop(record);
    drop(reader);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn live_reader_cursor() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    let mut subscription = handle.subscribe_records();
    publish(&handle).await;
    let captured = handle.captured_records();
    let expected = captured.next(0, captured.frontier()).await.unwrap().unwrap().sequence;
    drop(captured);
    let record = subscription.next().await.unwrap().unwrap();
    drop(subscription);
    assert_eq!(record.sequence, expected);
    drop(record);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn filtered_record_view() {
    use dokime_runtime::runtime::query::{RuntimeResolution, RuntimeViewPayload, RuntimeViewQuery, RuntimeViewTopic, RuntimeWindow};
    use dokime_runtime::{RecordScope, SessionId};
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    let selected = RecordScope {
        session_id: Some(SessionId::from_seed("record-view.selected")),
        ..RecordScope::default()
    };
    for scope in [
        selected.clone(),
        RecordScope {
            session_id: Some(SessionId::from_seed("record-view.other")),
            ..RecordScope::default()
        },
    ] {
        let mut draft = ProductCalibrationDetails::new("fixture.tool", ProductCalibrationKind::Field, ProductCalibrationPhase::Completed)
            .into_draft("fixture.product");
        draft.scope = Some(scope);
        handle.publish_product_record(draft).await.unwrap();
    }
    let query = RuntimeViewQuery {
        scope: Some(selected.clone()),
        topic: RuntimeViewTopic::Records,
        window: RuntimeWindow::Full,
        resolution: RuntimeResolution::Raw,
        cursor: None,
    };
    let first = handle.query_runtime_view(query.clone()).await.unwrap();
    let first_cursor = first.cursor;
    let RuntimeViewPayload::Records { frame: first } = first.payload else {
        panic!("expected saved record frame");
    };
    assert_eq!(first.records.len(), 1);
    assert_eq!(first.records[0].scope.as_ref(), Some(&selected));
    let first_id = first.records[0].record_id.clone();
    let stable = handle.query_runtime_view(query.clone()).await.unwrap();
    let RuntimeViewPayload::Records { frame: stable } = stable.payload else {
        panic!("expected saved record frame");
    };
    assert_eq!(stable.records.len(), 1);
    assert_eq!(stable.records[0].record_id, first_id);

    let mut draft = ProductCalibrationDetails::new("fixture.tool", ProductCalibrationKind::Field, ProductCalibrationPhase::Completed)
        .into_draft("fixture.product");
    draft.scope = Some(selected.clone());
    handle.publish_product_record(draft).await.unwrap();
    let mut delta = query.clone();
    delta.window = RuntimeWindow::Since { cursor: first_cursor };
    let delta = handle.query_runtime_view(delta).await.unwrap();
    let RuntimeViewPayload::Records { frame: delta } = delta.payload else {
        panic!("expected next explicit record cut");
    };
    assert_eq!(delta.records.len(), 1);
    assert_ne!(delta.records[0].record_id, first_id);
    assert_eq!(delta.records[0].scope.as_ref(), Some(&selected));
    assert!(delta.records[0].sequence > first_cursor.0);
    let full = handle.query_runtime_view(query).await.unwrap();
    let RuntimeViewPayload::Records { frame: full } = full.payload else {
        panic!("expected full saved record frame");
    };
    assert_eq!(full.records.len(), 2);
    drop(handle);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn fixed_replay_append() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    publish(&handle).await;
    handle.storage().flush().await.unwrap();
    publish(&handle).await;
    let reader = handle.captured_records();
    let through = reader.frontier();
    let first = reader.next(0, through).await.unwrap().unwrap();
    publish(&handle).await;
    assert_eq!(&*reader.next(0, through).await.unwrap().unwrap(), &*first);
    let second = reader.next(first.sequence, through).await.unwrap().unwrap();
    assert!(second.sequence > first.sequence && second.sequence <= through);
    assert!(reader.next(second.sequence, through).await.unwrap().is_none());
    let frontier = reader.wait_after(through).await.unwrap().unwrap();
    assert!(frontier > through);
    assert!(reader.next(through, frontier).await.unwrap().unwrap().sequence > through);
    drop(handle);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn independent_readers_pages() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    for _ in 0..260 {
        publish(&handle).await;
    }
    handle.storage().flush().await.unwrap();
    let old = handle.captured_records();
    let through = old.frontier();
    let first = old.next(0, through).await.unwrap().unwrap();

    for _ in 0..3 {
        publish(&handle).await;
    }
    let fresh = handle.captured_records();
    let current = fresh.frontier();
    assert_eq!(&*fresh.next(0, current).await.unwrap().unwrap(), &*first);
    let appended = fresh.next(through, current).await.unwrap().unwrap();
    assert!(appended.sequence > through);

    let mut after = first.sequence;
    let mut count = 1;
    while let Some(record) = old.next(after, through).await.unwrap() {
        assert!(record.sequence > after && record.sequence <= through);
        after = record.sequence;
        count += 1;
    }
    assert_eq!(count, 260);
    assert_eq!(&*old.next(0, through).await.unwrap().unwrap(), &*first);
    assert_eq!(&*old.next(through, current).await.unwrap().unwrap(), &*appended);
    assert_eq!(&*old.next(through, current).await.unwrap().unwrap(), &*appended);

    let second = fresh.next(appended.sequence, current).await.unwrap().unwrap();
    let third = fresh.next(second.sequence, current).await.unwrap().unwrap();
    assert!(fresh.next(third.sequence, current).await.unwrap().is_none());
    assert_eq!(&*fresh.next(0, current).await.unwrap().unwrap(), &*first);
    drop(fresh);
    drop(old);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn slow_dropped_create() {
    let root = tempfile::tempdir().unwrap();
    let config = runtime_config(root.path());
    let mut owner = Runtime::start(config).await.unwrap();
    let handle = owner.handle();
    let slow = handle.captured_records();
    for _ in 0..2000 {
        publish(&handle).await;
    }
    let frontier = handle.captured_records().frontier();
    let first = slow.next(0, frontier).await.unwrap().unwrap();
    assert_eq!(slow.wait_after(0).await.unwrap(), Some(frontier));
    assert!(first.sequence <= frontier);
    // Nothing Follows The Frontier Yet, So The Wait Is Still Pending When First Polled.
    let pending = {
        let mut waiting = std::pin::pin!(slow.wait_after(frontier));
        std::future::poll_fn(|context| std::task::Poll::Ready(std::future::Future::poll(waiting.as_mut(), context).is_pending())).await
    };
    assert!(pending);
    publish(&handle).await;
    drop(slow);
    let reader = handle.captured_records();
    assert!(reader.next(frontier, reader.frontier()).await.unwrap().unwrap().sequence > frontier);
    let selected = handle.selected_records(RecordSelection::default()).await.unwrap();
    assert_eq!(selected.len(), 2001);
    drop(handle);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn capture_failure_reader() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    publish(&handle).await;
    let reader = handle.captured_records();
    poison_capture(&handle).await;
    assert!(reader.next(0, reader.frontier()).await.is_err());
    assert!(reader.wait_after(reader.frontier()).await.is_err());
    drop(handle);
    assert!(owner.shutdown().await.is_err());
}

pub(crate) async fn captured_records_failure() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    let note = |message: &str| OperatorNoteRequest {
        scope: None,
        author: None,
        message: message.into(),
    };
    handle.add_operator_note(note("first")).await.unwrap();
    let captured = handle.captured_records();
    let through = captured.frontier();
    let first = captured.next(0, through).await.unwrap().unwrap();
    handle.add_operator_note(note("later")).await.unwrap();
    assert_eq!(&*captured.next(0, through).await.unwrap().unwrap(), &*first);
    assert!(captured.next(first.sequence, through).await.unwrap().is_none());
    assert!(first.sequence <= through);
    assert_eq!(handle.selected_records(RecordSelection::default()).await.unwrap().len(), 2);
    // Transaction-only groups evict the Harmos in-memory tail without adding
    // product facts; retained payloads must come from shared primary capture.
    for _ in 0..1030 {
        handle.set_suites("query.fixture", dokime_runtime::definition::SuiteSet::default()).await.unwrap();
    }
    let current = handle.captured_records().frontier();
    assert_eq!(&*captured.next(0, current).await.unwrap().unwrap(), &*first);
    assert!(captured.wait_after(through).await.unwrap().unwrap() >= current);
    assert_eq!(&*captured.next(0, through).await.unwrap().unwrap(), &*first);
    poison_capture(&handle).await;
    assert!(captured.next(0, through).await.is_err());
    assert!(handle.selected_records(RecordSelection::default()).await.is_err());
    assert!(handle.evidence_snapshot().await.is_err());
    drop(handle);
    assert!(owner.shutdown().await.is_err());
}

pub(crate) async fn capture_failure_provider() {
    use dokime_runtime::domains::{ExecutionProviderTarget, TestSuiteDefinition};
    use dokime_runtime::{ExecutionFixture, ExecutionFixtureContext, ExecutionFixtureFuture};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    #[derive(Default)]
    struct BlockingProvider {
        entered: tokio::sync::Notify,
        cancelled: AtomicBool,
    }
    struct Running<'a>(&'a AtomicBool);
    impl Drop for Running<'_> {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }
    impl ExecutionFixture for BlockingProvider {
        fn run_cycle<'a>(
            &'a self, context: ExecutionFixtureContext, _cycle: &'a dokime_domain::model::execution::result::CycleView,
            _target: &'a ExecutionProviderTarget,
        ) -> ExecutionFixtureFuture<'a> {
            Box::pin(async move {
                let _running = Running(&self.cancelled);
                self.entered.notify_one();
                context.cancelled().await;
                Err(dokime_runtime::transport::ProviderExecutionFailure::new("cancel", "fixture cancelled"))
            })
        }
    }
    let root = tempfile::tempdir().unwrap();
    let metadata = Metadata::new(Identifier::new("fixture.shutdown").unwrap(), "Shutdown", None);
    let config = crate::service_support::budgets::runtime_config(root.path(), metadata.clone());
    let provider = Arc::new(BlockingProvider::default());
    let mut owner = Runtime::with_execution_fixture(config, "testdb", provider.clone()).await.unwrap();
    let handle = owner.handle();
    let suite: dokime_runtime::definition::TestSuite = serde_json::from_value(serde_json::json!({
        "metadata": metadata,
        "test_case_templates": [{
            "test_case_template_id": dokime_runtime::model::identity::new_id::<dokime_runtime::TemplateId>(),
            "metadata": metadata,
            "parameters": []
        }]
    }))
    .unwrap();
    handle
        .set_suites(
            "provider:testdb",
            dokime_runtime::definition::SuiteSet::new([TestSuiteDefinition {
                suite_id: "provider:testdb:shutdown".into(),
                tags: vec![],
                workflow: Some("execute_cycle".into()),
                suite: suite.clone(),
                record_contracts: vec![],
                telemetry_contracts: vec![],
            }]),
        )
        .await
        .unwrap();
    let cycle = crate::service_support::suites::start_cases(
        &handle,
        metadata,
        crate::service_support::suites::materialize_cases("provider:testdb:shutdown", &suite),
    )
    .await;
    handle.start_next_execution(cycle, dokime_runtime::runtime::command::CommandContext::default()).await;
    provider.entered.notified().await;

    let reader = handle.captured_records();
    poison_capture(&handle).await;
    assert!(reader.next(0, reader.frontier()).await.is_err());
    assert!(!handle.watch_state().latest().state().shutting_down);
    drop(handle);
    let error = owner.shutdown().await.unwrap_err();
    assert!(error.to_string().contains("capture"), "{error}");
    assert!(provider.cancelled.load(Ordering::SeqCst));
}

pub(crate) async fn primary_capture_boots() {
    use harmos::storage::StorageBackend;
    use std::sync::Arc;

    let root = tempfile::tempdir().unwrap();
    let mut expected = Vec::new();
    let mut primary = None;
    for _ in 0..2 {
        let config = crate::service_support::budgets::runtime_config(
            root.path(),
            Metadata::new(Identifier::new("fixture.records").unwrap(), "Record Query Fixture", None),
        );
        primary = Some(config.storage_root.clone());
        let mut owner = Runtime::start(config).await.unwrap();
        let handle = owner.handle();
        publish(&handle).await;
        let reader = handle.captured_records();
        let record = reader.next(0, reader.frontier()).await.unwrap().unwrap();
        expected.push(record.record_id.to_string());
        assert_eq!(
            handle.selected_records(RecordSelection::default()).await.unwrap().len(),
            expected.len(),
            "managed recovery retains record visibility across process sessions",
        );
        assert!(reader.next(record.sequence, reader.frontier()).await.unwrap().is_none());
        drop(reader);
        drop(handle);
        owner.shutdown().await.unwrap();
    }
    let backend = Arc::new(dokime_runtime::storage::Backend::primary(primary.unwrap()).await.unwrap());
    let saved = backend.cursor_saved().await.unwrap();
    let mut actual = Vec::new();
    let mut epochs = Vec::new();
    crate::service_support::backend::for_each_record(backend.as_ref(), &saved, dokime_runtime::storage::records::schema(), |row| {
        let record = dokime_runtime::storage::records::RecordRow::decode_entry(&row.payload).unwrap();
        assert_eq!(row.item.as_str(), record.record_id.as_str());
        actual.push(record.record_id.to_string());
        epochs.push(row.scope.as_ref().expect("captured record retains boot scope").to_string());
    })
    .await;
    assert_eq!(actual, expected);
    assert_ne!(epochs[0], epochs[1]);
    backend.close().await.unwrap();
}

pub(crate) async fn completed_record_capacity() {
    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    publish(&handle).await;
    let mut readers = Vec::new();
    for _ in 0..40 {
        let reader = handle.captured_records();
        let mut after = 0;
        while let Some(record) = reader.next(after, reader.frontier()).await.unwrap() {
            after = record.sequence;
        }
        readers.push(reader);
    }
    // Completed Readers Observe Later Publications.
    publish(&handle).await;
    let reader = handle.captured_records();
    assert!(reader.next(0, reader.frontier()).await.unwrap().is_some());
    drop(reader);
    drop(readers);
    drop(handle);
    owner.shutdown().await.unwrap();
}

pub(crate) async fn native_entry_independent() {
    use dokime_runtime::TestCycleId;
    use dokime_runtime::storage::objects::cycle::{self, Cycle};
    use harmos::storage::ObjectVersion;
    use {dokime_domain::model::execution::result::CycleStatus, dokime_domain::model::execution::result::TestCycle};

    let root = tempfile::tempdir().unwrap();
    let mut owner = runtime(root.path()).await;
    let handle = owner.handle();
    let now = dokime_runtime::model::time::now_utc();
    let cycle = TestCycle {
        saved: None,
        test_cycle_id: dokime_runtime::model::identity::new_id::<TestCycleId>(),
        session_id: None,
        metadata: Metadata::new(
            Identifier::new("fixture.large-cycle").unwrap(),
            "Native Payload Within Its Entry Limit",
            Some("x".repeat(300_000)),
        ),
        lifecycle: CycleStatus::Ready,
        phase: Default::default(),
        created_at: now.clone(),
        started_at: None,
        updated_at: now,
        execution_ids: vec![],
        records: Default::default(),
        continue_automatically: false,
        result: None,
    };
    let native = Cycle::try_from(&cycle).unwrap();
    assert!(harmos::Message::encoded_size(&native) < 1024 * 1024);
    let receipt = handle.storage().write_object(&native).await.unwrap();
    handle.storage().flush_until(&receipt.position).await.unwrap();
    let restored = cycle::read(handle.storage(), &cycle.test_cycle_id, ObjectVersion::Latest).await.unwrap().unwrap();
    assert_eq!(restored.value, cycle);
    drop(restored);
    drop(handle);
    owner.shutdown().await.unwrap();
}
