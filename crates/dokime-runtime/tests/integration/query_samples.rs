/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Finite Arrow Snapshot Pages

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::query_telemetry::Fixture;
use dokime_domain::model::{
    identity::wire::Timestamp,
    query::{
        error::Reason,
        page::SampleRow,
        predicate::{Operator, Predicate, Scalar},
        selection::Window,
    },
    values::Value,
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

/* === Tests === */

#[tokio::test]
async fn finite_arrow_pages() {
    let mut fixture = Fixture::new(|_| {}).await;
    let big = (1u64 << 63) + 7;
    fixture
        .publish(&[
            (big, 999_999_999, Some(1.0), "alpha"),
            (big + 1, 1_000_000_000, Some(2.0), "beta"),
            (big + 2, 3_000_000_000, None, "alpha"),
        ])
        .await;
    let mut request = fixture.request();
    request.between = Some(Window {
        start: None,
        end: Some(Timestamp { seconds: 1, nanos: 0 }),
    });
    eprintln!("query first page");
    let page = fixture.page(request.clone()).await;
    eprintln!("query first page returned");
    assert!(page.complete);
    let SampleRow::Raw(sample) = &page.rows[0] else { panic!("raw sample") };
    assert_eq!(
        sample.values.as_ref().unwrap().to_core().unwrap(),
        Value::Object(std::collections::BTreeMap::from([("counter".into(), Value::Unsigned(big))]))
    );
    assert_eq!(sample.scope.as_ref().unwrap().execution_id.as_deref(), Some(fixture.execution.as_str()));
    let snapshot = page.snapshot.unwrap();
    fixture.publish(&[(big + 3, 500_000_000, Some(9.0), "late")]).await;
    request.at = Fixture::at(&snapshot);
    let old = fixture.page(request.clone()).await;
    assert_eq!(old.rows, page.rows);
    request.between = Some(Window {
        start: Some(Timestamp { seconds: 1, nanos: 0 }),
        end: None,
    });
    let first = fixture.page(request.clone()).await;
    assert!(!first.complete);
    request.page.after = first.next;
    request.page.rows = Some(3);
    let last = fixture.page(request.clone()).await;
    assert!(last.complete);
    assert_eq!(last.rows.len(), 1);
    request.r#where = vec![Predicate {
        field: "counter".into(),
        operator: Operator::Eq,
        values: vec![Scalar::Unsigned(big)],
    }];
    let failure = fixture.runtime.query_list_samples(request, usize::MAX).await.unwrap_err();
    assert_eq!(failure.reason, Reason::InvalidContinuation);
    let mut projected = fixture.request();
    projected.r#where = vec![Predicate {
        field: "counter".into(),
        operator: Operator::Eq,
        values: vec![Scalar::Unsigned(big + 1)],
    }];
    projected.fields = vec!["value".into()];
    let projected = fixture.page(projected).await;
    assert!(projected.complete);
    assert_eq!(projected.rows.len(), 1);
    let SampleRow::Raw(sample) = &projected.rows[0] else { panic!("raw sample") };
    assert_eq!(
        sample.values.as_ref().unwrap().to_core().unwrap(),
        Value::Object(std::collections::BTreeMap::from([("value".into(), Value::Number(2.0))]))
    );
    let mut tiny = fixture.request();
    tiny.page.bytes = Some(32);
    assert_eq!(fixture.runtime.query_list_samples(tiny, usize::MAX).await.unwrap_err().reason, Reason::ResourceLimit);
    let history = fixture
        .runtime
        .query_list_executions(
            dokime_operations::query::ListExecutions {
                selection: dokime_domain::model::query::selection::ExecutionSelection {
                    template_ids: Vec::new(),
                    cycle_ids: Vec::new(),
                    statuses: Vec::new(),
                    started_at: None,
                    archive: None,
                },
                page: dokime_domain::model::query::page::PageRequest { rows: Some(10), bytes: None, after: None },
                at: None,
            },
            usize::MAX,
        )
        .await
        .unwrap();
    assert_eq!(history.rows.len(), 1);
    let mut related = fixture.request();
    related.page.rows = Some(10);
    related.at = Fixture::at(history.snapshot.as_ref().unwrap());
    assert_eq!(fixture.page(related).await.rows.len(), 4);
    eprintln!("query shutdown");
    fixture.owner.shutdown().await.unwrap();
    eprintln!("query shutdown complete");

    let mut sparse = Fixture::new(|config| {
        config.query.page_rows = 1;
        config.query.max_page_rows = 1;
    })
    .await;
    sparse.publish(&[(0, 0, Some(5.0), "a"), (1, 1, Some(6.0), "a"), (2, 2, Some(7.0), "a")]).await;
    let mut request = sparse.request();
    request.r#where = vec![Predicate {
        field: "counter".into(),
        operator: Operator::Eq,
        values: vec![Scalar::Unsigned(2)],
    }];
    let first = sparse.page(request.clone()).await;
    assert!(first.rows.is_empty());
    assert!(!first.complete);
    request.at = Fixture::at(first.snapshot.as_ref().unwrap());
    let mut next = first.next;
    let mut seen = std::collections::BTreeSet::new();
    let mut rows = Vec::new();
    while let Some(token) = next {
        assert!(seen.insert(token.clone()), "continuation must advance");
        request.page.after = Some(token);
        let page = sparse.page(request.clone()).await;
        rows.extend(page.rows);
        if page.complete {
            assert!(page.next.is_none());
            next = None;
        } else {
            assert!(page.next.is_some());
            next = page.next;
        }
    }
    assert_eq!(rows.len(), 1);
    sparse.owner.shutdown().await.unwrap();

    // A Newer Unscoped Stream Version Cannot Enlarge An Older Execution's Contract.
    let root = fixture._root.path().to_owned();
    let execution_id = fixture.execution.to_string();
    drop(fixture.runtime);
    drop(fixture.owner);
    let configuration = crate::service_support::config(root).with_installed_plugins([dokime_runtime::InstalledPlugin {
        source: "query_fixture".into(),
        version: "next".into(),
        description: "Query Fixture".into(),
        enabled: true,
        telemetry_contracts: vec![crate::query_telemetry::contract(2)],
        ..Default::default()
    }]);
    let (mut owner, runtime) = crate::service_support::started(configuration).await;
    let stream = dokime_runtime::Identifier::new("query.readings").unwrap();
    let mut sample = dokime_runtime::telemetry::TelemetrySample::new(
        dokime_runtime::model::identity::new_id(),
        stream.clone(),
        0,
        dokime_runtime::Timestamp::from_datetime(time::OffsetDateTime::UNIX_EPOCH),
        "upgrade",
    )
    .typed(&crate::query_telemetry::Reading { counter: 999, value: Some(1.0) });
    sample.schema_version = 2;
    sample.values.insert("counter".into(), Value::Unsigned(999));
    sample.values.insert("value".into(), Value::Number(1.0));
    let receipt = runtime
        .publish_telemetry_batch(dokime_runtime::telemetry::TelemetryBatch::new(
            dokime_runtime::model::identity::new_id(),
            stream,
            vec![sample],
        ))
        .await
        .unwrap();
    assert_eq!(receipt.accepted, 1);
    drop(receipt);
    let mut request = dokime_operations::query::ListSamples {
        execution_id,
        stream: "query.readings".into(),
        versions: Vec::new(),
        fields: vec!["counter".into()],
        r#where: Vec::new(),
        between: None,
        bucket: None,
        page: dokime_domain::model::query::page::PageRequest { rows: Some(10), bytes: None, after: None },
        at: None,
    };
    let retained = runtime.query_list_samples(request.clone(), usize::MAX).await.unwrap();
    assert!(retained.complete);
    assert_eq!(retained.rows.len(), 4);
    assert_eq!(retained.schema.unwrap().versions, vec![1]);
    request.versions = vec![2];
    assert_eq!(runtime.query_list_samples(request, usize::MAX).await.unwrap_err().reason, Reason::IncompatibleSchema);
    owner.shutdown().await.unwrap();
}
