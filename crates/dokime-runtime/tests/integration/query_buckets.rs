/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Epoch Bucket Boundaries And Bounded Pages

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
        reduction::{BucketRequest, StatisticValue},
        selection::Window,
    },
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
async fn epoch_query_buckets() {
    let mut fixture = Fixture::new(|_| {}).await;
    fixture
        .publish(&[
            (2, 3_000_000_000, None, "a"),
            (0, 999_999_999, Some(10.0), "a"),
            (1, 1_000_000_000, Some(20.0), "a"),
            (4, 1_000_000_000, Some(100.0), "b"),
            (3, 500_000_000, Some(0.0), "a"),
        ])
        .await;
    let mut request = fixture.request();
    request.fields = vec!["value".into()];
    request.bucket = Some(BucketRequest {
        every_nanos: 1_000_000_000,
        aggregate: harmos::Aggregate::Mean,
    });
    let mut rows = Vec::new();
    let mut snapshot = None;
    loop {
        let page = fixture.page(request.clone()).await;
        if snapshot.is_none() {
            snapshot = page.snapshot;
        }
        rows.extend(page.rows);
        if page.complete {
            assert!(page.next.is_none());
            break;
        }
        request.page.after = page.next;
        request.at = Fixture::at(snapshot.as_ref().unwrap());
        request.page.rows = Some(2);
    }
    let buckets: Vec<_> = rows
        .iter()
        .map(|row| {
            let SampleRow::Bucket(bucket) = row else { panic!("typed bucket") };
            bucket
        })
        .collect();
    assert_eq!(
        buckets.iter().map(|bucket| (bucket.start.unwrap().seconds, bucket.source.as_str())).collect::<Vec<_>>(),
        vec![(0, "a"), (1, "a"), (1, "b"), (3, "a")]
    );
    assert_eq!(buckets.iter().map(|bucket| bucket.sample_count).collect::<Vec<_>>(), vec![2, 1, 1, 1]);
    assert_eq!(buckets[0].fields[0].value, Some(StatisticValue::Number(5.0)));
    assert_eq!(buckets[1].fields[0].value, Some(StatisticValue::Number(20.0)));
    assert_eq!(buckets[2].fields[0].value, Some(StatisticValue::Number(100.0)));
    assert_eq!(buckets[3].fields[0].count, 0);
    assert!(buckets[3].fields[0].value.is_none());
    assert_eq!(buckets[0].end.unwrap().seconds, 1);
    let mut clipped = fixture.request();
    clipped.fields = vec!["value".into()];
    clipped.page.rows = Some(10);
    clipped.at = Fixture::at(snapshot.as_ref().unwrap());
    clipped.bucket = Some(BucketRequest {
        every_nanos: 1_000_000_000,
        aggregate: harmos::Aggregate::Mean,
    });
    clipped.between = Some(Window {
        start: Some(Timestamp { seconds: 0, nanos: 750_000_000 }),
        end: Some(Timestamp { seconds: 3, nanos: 0 }),
    });
    let page = fixture.page(clipped.clone()).await;
    assert!(page.complete);
    assert_eq!(page.rows.len(), 3);
    let SampleRow::Bucket(first) = &page.rows[0] else { panic!("typed bucket") };
    assert_eq!(first.start, Some(Timestamp { seconds: 0, nanos: 0 }));
    assert_eq!(first.fields[0].value, Some(StatisticValue::Number(10.0)));
    clipped.bucket.as_mut().unwrap().aggregate = harmos::Aggregate::Count;
    clipped.between = None;
    let page = fixture.page(clipped).await;
    let SampleRow::Bucket(last) = &page.rows[3] else { panic!("typed bucket") };
    assert_eq!(last.fields[0].value, Some(StatisticValue::Count(0)));
    fixture.owner.shutdown().await.unwrap();

    let mut restricted = Fixture::new(|config| {
        config.query.buckets = 1;
        config.query.partitions = 1;
    })
    .await;
    restricted.publish(&[(0, 0, Some(0.0), "a"), (1, 1_000_000_000, Some(10.0), "a")]).await;
    let mut request = restricted.request();
    request.fields = vec!["value".into()];
    request.bucket = Some(BucketRequest {
        every_nanos: 1_000_000_000,
        aggregate: harmos::Aggregate::Mean,
    });
    assert_eq!(restricted.runtime.query_list_samples(request.clone(), usize::MAX).await.unwrap_err().reason, Reason::BucketLimit);
    restricted.publish(&[(2, 500_000_000, Some(20.0), "b")]).await;
    request.between = Some(Window {
        start: None,
        end: Some(Timestamp { seconds: 1, nanos: 0 }),
    });
    assert_eq!(restricted.runtime.query_list_samples(request, usize::MAX).await.unwrap_err().reason, Reason::ResourceLimit);
    restricted.owner.shutdown().await.unwrap();
}
