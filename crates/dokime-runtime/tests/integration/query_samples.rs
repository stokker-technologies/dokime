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
    eprintln!("query shutdown");
    fixture.owner.shutdown().await.unwrap();
    eprintln!("query shutdown complete");
}
