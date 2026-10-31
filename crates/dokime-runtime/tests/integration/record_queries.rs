/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Finite Record Query Lifecycle

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::BTreeMap;

// Third Party Imports
use harmos::{Message, operations::OperationDeclaration};

// Local Imports
use dokime_domain::model::{
    execution::RecordScope as NativeScope,
    identity::wire::Timestamp,
    identity::{Identifier, Metadata, SessionId, TestExecutionId},
    query::{
        error::Reason,
        predicate::{Operator, Predicate, Scalar},
        selection::{CycleScope, RecordScope},
    },
    records::{RecordContract, RecordField},
    values::{Value, ValueType},
};
use dokime_operations::query::ListRecords;
use dokime_runtime::{config, runtime::query::snapshots::Scope, testing::RecordQueryFixture as Fixture};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

fn contract() -> RecordContract {
    let mut contract = RecordContract::declared("example.measurement", "Measurement");
    let mut field = RecordField::new(Metadata::new(Identifier::new("counter.value").unwrap(), "Counter", None), ValueType::Integer);
    field.required = false;
    field.nullable = true;
    field.default = Some(Value::Unsigned(99));
    contract.fields.push(field);
    contract
}

fn term(field: &str, operator: Operator, value: Scalar) -> Predicate {
    Predicate {
        field: field.into(),
        operator,
        values: vec![value],
    }
}

async fn restart_selection() {
    let mut f = Fixture::new(16).await;
    for position in 10..13 {
        f.insert(position, &f.epoch, f.entry(position)).await;
    }
    let mut queued = f.entry(13);
    queued.sequence = 100;
    f.insert(13, &f.epoch, queued).await;
    let cut = f.snapshot(Scope::Execution(f.execution.to_string()), 13, Some(20), vec![contract()]).await;
    let mut request = f.request();
    request.between = Some(dokime_domain::model::query::selection::Window {
        start: Some(Timestamp { seconds: 0, nanos: 10 }),
        end: Some(Timestamp { seconds: 0, nanos: 12 }),
    });
    request.r#where = vec![
        term("@severity", Operator::Eq, Scalar::Severity(dokime_domain::model::identity::wire::Severity::Warn)),
        term("@received_at", Operator::Gte, Scalar::Datetime(Timestamp { seconds: 0, nanos: 0 })),
        term("@source", Operator::Eq, Scalar::Text("observer.fixture".into())),
        term("@schema_version", Operator::Eq, Scalar::Unsigned(1)),
    ];
    let first = f.read(&request, &cut, None).await.unwrap();
    assert_eq!(first.rows.len(), 2);
    f.close().await;
    f.reopen().await;
    f.epoch = SessionId::from_seed("restarted-query-process");
    let repeated = f.read(&request, &cut, None).await.unwrap();
    assert!(first == repeated);
    request.between = None;
    let all = f.read(&request, &cut, None).await.unwrap();
    assert_eq!(all.rows.len(), 3);
    f.close().await;
}

async fn sparse_pages() {
    let f = Fixture::new(2).await;
    for position in 10..15 {
        f.insert(position, &f.epoch, f.entry(position)).await;
    }
    let snapshot = f.snapshot(Scope::Execution(f.execution.to_string()), 14, Some(14), vec![contract()]).await;
    let mut request = f.request();
    request.r#where = vec![term("counter.value", Operator::Gte, Scalar::Unsigned(14))];
    let first = f.read(&request, &snapshot, None).await.unwrap();
    assert!(first.rows.is_empty() && !first.complete && first.next.is_some());
    let repeated = f.read(&request, &snapshot, None).await.unwrap();
    assert!(repeated == first);
    let second = f.read(&request, &snapshot, first.next.as_deref()).await.unwrap();
    assert!(second.rows.is_empty() && !second.complete && second.next != first.next);
    request.page.rows = Some(1);
    let last = f.read(&request, &snapshot, second.next.as_deref()).await.unwrap();
    assert!(last.complete && last.next.is_none());
    assert_eq!(last.rows.iter().map(|row| row.record_id.as_str()).collect::<Vec<_>>(), [f.entry(14).record_id.as_str()]);
    request.r#where.clear();
    assert_eq!(f.read(&request, &snapshot, first.next.as_deref()).await.err().unwrap().reason, Reason::InvalidContinuation);
    f.close().await;
}

async fn capture_attribution() {
    let f = Fixture::new(8).await;
    let mut entry = f.entry(10);
    let producer = TestExecutionId::from_seed("query-record-producer");
    entry.scope = Some(NativeScope {
        test_execution_id: Some(producer.clone()),
        ..Default::default()
    });
    entry.captured_scopes = vec![f.native_scope(&f.execution), f.native_scope(&f.second)];
    f.insert(10, &f.epoch, entry.clone()).await;
    let cut = f.snapshot(Scope::Cycle(f.cycle.to_string()), 10, Some(10), vec![contract()]).await;
    let mut request = f.request();
    for scope in [
        RecordScope::Execution(f.execution.to_string()),
        RecordScope::Execution(f.second.to_string()),
        RecordScope::Cycle(CycleScope {
            cycle_id: f.cycle.to_string(),
            execution_id: None,
        }),
        RecordScope::Cycle(CycleScope {
            cycle_id: f.cycle.to_string(),
            execution_id: Some(f.execution.to_string()),
        }),
    ] {
        request.scope = scope;
        let scope = match &request.scope {
            RecordScope::Execution(id) => Scope::Execution(id.clone()),
            RecordScope::Cycle(scope) => Scope::Cycle(scope.cycle_id.clone()),
        };
        let snapshot = f.snapshot(scope, 10, Some(10), vec![contract()]).await;
        let page = f.read(&request, &snapshot, None).await.unwrap();
        assert!(page.complete && page.rows.len() == 1);
        let row = &page.rows[0];
        assert_eq!(row.record_id, entry.record_id.to_string());
        assert_eq!(row.scope.as_ref().unwrap().execution_id.as_deref(), Some(producer.as_str()));
        assert!(row.scope.as_ref().unwrap().cycle_id.is_none());
        assert_eq!(row.source, "observer.fixture");
        assert!(row.severity == dokime_domain::model::identity::wire::Severity::Warn);
        assert_eq!(row.timestamp.unwrap().nanos, 10);
        assert_eq!(row.received_at, row.timestamp);
        assert_eq!(row.sequence, 10);
    }
    request.scope = RecordScope::Cycle(CycleScope {
        cycle_id: f.cycle.to_string(),
        execution_id: Some(producer.to_string()),
    });
    assert_eq!(f.read(&request, &cut, None).await.err().unwrap().reason, Reason::ScopeNotFound);
    f.close().await;
}

async fn fixed_frontiers() {
    let f = Fixture::new(16).await;
    f.insert(10, &f.epoch, f.entry(10)).await;
    let mut queued = f.entry(11);
    queued.sequence = 101;
    f.insert(11, &f.epoch, queued).await;
    let old_epoch = SessionId::from_seed("previous-query-process");
    let mut old = f.entry(12);
    old.sequence = 102;
    f.insert(12, &old_epoch, old).await;
    let new_member = TestExecutionId::from_seed("new-query-member");
    let mut later = f.entry(13);
    later.scope = Some(f.native_scope(&new_member));
    f.insert(13, &f.epoch, later).await;
    let cut = f.snapshot(Scope::Cycle(f.cycle.to_string()), 13, Some(20), vec![contract()]).await;
    f.insert(14, &f.epoch, f.entry(14)).await;
    f.append_member(new_member).await;
    let mut request = f.request();
    request.scope = RecordScope::Cycle(CycleScope {
        cycle_id: f.cycle.to_string(),
        execution_id: None,
    });
    let page = f.read(&request, &cut, None).await.unwrap();
    assert!(page.complete);
    assert_eq!(
        page.rows.iter().map(|row| row.record_id.as_str()).collect::<Vec<_>>(),
        [f.entry(10).record_id.as_str(), f.entry(12).record_id.as_str()]
    );
    // A Checkpoint Has Only Its Original Physical Boundary, Independent Of Journal Sequence.
    let checkpoint = f.snapshot(Scope::Execution(f.execution.to_string()), 11, None, vec![contract()]).await;
    let page = f.read(&f.request(), &checkpoint, None).await.unwrap();
    assert_eq!(page.rows.len(), 2);
    f.close().await;
}

async fn captured_projection() {
    let f = Fixture::new(16).await;
    let mut missing = f.entry(10);
    missing.details.clear();
    let mut null = f.entry(11);
    null.details.insert("counter.value".into(), Value::Null);
    let mut large = f.entry(12);
    large.schema_version = 2;
    large.details.insert("counter.value".into(), Value::Unsigned(u64::MAX));
    large.details.insert("undeclared".into(), Value::Text("extra".into()));
    for (position, row) in [(10, missing), (11, null), (12, large)] {
        f.insert(position, &f.epoch, row).await;
    }
    let mut second = contract();
    second.schema_version = 2;
    second.fields.push(RecordField::new(Metadata::new(Identifier::new("optional").unwrap(), "Optional", None), ValueType::Text));
    let cut = f.snapshot(Scope::Execution(f.execution.to_string()), 12, Some(12), vec![contract(), second]).await;
    let mut request = f.request();
    let all = f.read(&request, &cut, None).await.unwrap();
    assert_eq!(all.rows.len(), 3);
    assert!(all.schema.as_ref().unwrap().fields.iter().find(|field| field.id == "optional").unwrap().nullable);
    let details: Vec<_> = all.rows.iter().map(|row| row.details.as_ref().unwrap().to_core().unwrap()).collect();
    assert!(details[0] == Value::Object(BTreeMap::new()));
    assert!(details[1] == Value::Object(BTreeMap::from([("counter.value".into(), Value::Null)])));
    assert!(details[2] == Value::Object(BTreeMap::from([("counter.value".into(), Value::Unsigned(u64::MAX))])));
    request.r#where = vec![term("counter.value", Operator::Exists, Scalar::Boolean(false))];
    assert_eq!(f.read(&request, &cut, None).await.unwrap().rows.len(), 2);
    request.r#where = vec![term("counter.value", Operator::Eq, Scalar::Unsigned(u64::MAX))];
    let page = f.read(&request, &cut, None).await.unwrap();
    assert_eq!(page.rows.len(), 1);
    assert_eq!(page.rows[0].schema_version, 2);
    request.versions = vec![1];
    assert!(f.read(&request, &cut, None).await.unwrap().rows.is_empty());
    request.record_type = "example.unknown".into();
    assert_eq!(f.read(&request, &cut, None).await.err().unwrap().reason, Reason::ContractNotFound);
    let mut changed = contract();
    changed.schema_version = 2;
    changed.fields[0].value_type = ValueType::Text;
    let incompatible = f.snapshot(Scope::Execution(f.execution.to_string()), 12, Some(12), vec![contract(), changed]).await;
    request = f.request();
    let failure = f.read(&request, &incompatible, None).await.err().unwrap();
    assert_eq!(failure.reason, Reason::IncompatibleSchema);
    assert_eq!(failure.available_versions, vec![1, 2]);
    request.versions = vec![1];
    assert_eq!(f.read(&request, &incompatible, None).await.unwrap().rows.len(), 2);
    f.close().await;
}

async fn budget_failures() {
    let deferred = Fixture::new(2).await;
    deferred.insert(10, &deferred.epoch, deferred.entry(10)).await;
    let mut oversized = deferred.entry(11);
    oversized.message = Some("x".repeat(20_000));
    deferred.insert(11, &deferred.epoch, oversized).await;
    let cut = deferred.snapshot(Scope::Execution(deferred.execution.to_string()), 11, Some(11), vec![contract()]).await;
    let mut request = deferred.request();
    request.page.rows = Some(1);
    request.page.bytes = Some(8192);
    let first = deferred.read(&request, &cut, None).await.unwrap();
    assert!(first.rows.len() == 1 && !first.complete);
    assert_eq!(deferred.read(&request, &cut, first.next.as_deref()).await.err().unwrap().reason, Reason::ResourceLimit);
    deferred.close().await;

    let mut f = Fixture::new(2).await;
    let mut row = f.entry(10);
    row.message = Some("x".repeat(20_000));
    f.insert(10, &f.epoch, row).await;
    f.insert(11, &f.epoch, f.entry(11)).await;
    f.insert(12, &f.epoch, f.entry(12)).await;
    let cut = f.snapshot(Scope::Execution(f.execution.to_string()), 12, Some(12), vec![contract()]).await;
    let mut request = f.request();
    request.page.bytes = Some(8192);
    assert_eq!(f.read(&request, &cut, None).await.err().unwrap().reason, Reason::ResourceLimit);
    request.page.bytes = Some(16);
    assert_eq!(f.read(&request, &cut, None).await.err().unwrap().reason, Reason::ResourceLimit);
    request.page.bytes = None;
    let mut settings = config::query_defaults();
    settings.scanned_rows = 1;
    f.config = settings.compose(&config::storage_defaults(), &config::schema_defaults()).unwrap();
    request.r#where = vec![term("counter.value", Operator::Gt, Scalar::Unsigned(100))];
    assert_eq!(f.read(&request, &cut, None).await.err().unwrap().limit.unwrap().name, "scanned_rows");
    settings.scanned_bytes = 1;
    f.config = settings.compose(&config::storage_defaults(), &config::schema_defaults()).unwrap();
    assert_eq!(f.read(&request, &cut, None).await.err().unwrap().reason, Reason::ResourceLimit);
    settings.scanned_bytes = config::query_defaults().scanned_bytes;
    settings.working_bytes = 8192;
    f.config = settings.compose(&config::storage_defaults(), &config::schema_defaults()).unwrap();
    assert_eq!(f.read(&request, &cut, None).await.err().unwrap().reason, Reason::ResourceLimit);
    f.close().await;
}

async fn byte_pages() {
    let f = Fixture::new(16).await;
    for position in 10..14 {
        let mut row = f.entry(position);
        row.message = Some("x".repeat(2000));
        f.insert(position, &f.epoch, row).await;
    }
    let cut = f.snapshot(Scope::Execution(f.execution.to_string()), 13, Some(13), vec![contract()]).await;
    let mut request = f.request();
    request.page.bytes = Some(8192);
    let mut next = None;
    let mut ids = Vec::new();
    loop {
        let page = f.read(&request, &cut, next.as_deref()).await.unwrap();
        assert!(page.encoded_size() <= 8192);
        assert!(serde_json::to_vec(&page).unwrap().len() <= 8192);
        ids.extend(page.rows.into_iter().map(|row| row.record_id));
        next = page.next;
        if page.complete {
            assert!(next.is_none());
            break;
        }
        assert!(next.is_some());
    }
    assert_eq!(ids, (10..14).map(|position| f.entry(position).record_id.to_string()).collect::<Vec<_>>());
    let empty = f.snapshot(Scope::Execution(f.execution.to_string()), 3, Some(3), vec![contract()]).await;
    let page = f.read(&f.request(), &empty, None).await.unwrap();
    assert!(page.complete && page.rows.is_empty() && page.schema.is_some());
    request.r#where = vec![term("missing", Operator::Eq, Scalar::Unsigned(1))];
    assert_eq!(f.read(&request, &empty, None).await.err().unwrap().reason, Reason::UnknownField);
    f.close().await;
}

async fn unavailable_evidence() {
    let f = Fixture::new(1).await;
    f.insert(10, &f.epoch, f.entry(10)).await;
    f.insert(11, &f.epoch, f.entry(11)).await;
    let cut = f.snapshot(Scope::Execution(f.execution.to_string()), 11, Some(11), vec![contract()]).await;
    let request = f.request();
    let first = f.read(&request, &cut, None).await.unwrap();
    assert!(!first.complete && first.next.is_some());
    f.corrupt_record(11).await;
    let failure = f.read(&request, &cut, first.next.as_deref()).await.err().unwrap();
    assert_eq!(failure.reason, Reason::Corruption);
    assert_eq!(failure.operation, ListRecords::ID);
    f.invalidate().await;
    assert_eq!(f.read(&request, &cut, first.next.as_deref()).await.err().unwrap().reason, Reason::SnapshotUnavailable);
    f.close().await;
}

/* === Tests === */

#[dokime_tracing::trace_test]
#[tokio::test]
async fn record_queries() {
    restart_selection().await;
    sparse_pages().await;
    capture_attribution().await;
    fixed_frontiers().await;
    captured_projection().await;
    budget_failures().await;
    byte_pages().await;
    unavailable_evidence().await;
}
