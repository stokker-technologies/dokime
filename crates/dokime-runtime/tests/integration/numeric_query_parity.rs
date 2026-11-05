/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Exact Numeric Query Parity Across Projected Batches

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::query_telemetry::Fixture;
use dokime_domain::model::query::{
    page::SampleRow,
    predicate::{Operator, Predicate, Scalar},
    reduction::{BucketRequest, Output, StatisticValue},
};
use dokime_operations::query::AggregateSamples;

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
async fn numeric_query_parity() {
    let mut fixture = Fixture::new(|_| {}).await;
    let big = 9_007_199_254_740_992u64;
    fixture
        .publish(&[
            (big, 0, Some(1e16), "a"),
            (big + 1, 1, Some(1.0), "a"),
            (u64::MAX, 2, Some(-1e16), "a"),
            (0, 3, Some(3.0), "a"),
            (1, 4, Some(f64::from_bits(1)), "a"),
        ])
        .await;
    let request = AggregateSamples {
        execution_id: fixture.execution.to_string(),
        stream: "query.readings".into(),
        versions: Vec::new(),
        r#where: Vec::new(),
        between: None,
        at: None,
        outputs: vec![
            Output {
                alias: "sum".into(),
                field: "value".into(),
                aggregate: harmos::Aggregate::Sum,
            },
            Output {
                alias: "median".into(),
                field: "value".into(),
                aggregate: harmos::Aggregate::Median,
            },
        ],
    };
    let result = fixture.runtime.query_aggregate_samples(request.clone(), usize::MAX).await.unwrap();
    assert_eq!(result.values[0].value, Some(StatisticValue::Number(3.0)));
    assert_eq!(result.values[1].value, Some(StatisticValue::Number(1.0)));
    let mut bucket = fixture.request();
    bucket.fields = vec!["value".into()];
    bucket.bucket = Some(BucketRequest {
        every_nanos: 10,
        aggregate: harmos::Aggregate::Sum,
    });
    let result = fixture.page(bucket.clone()).await;
    let SampleRow::Bucket(sum) = &result.rows[0] else { panic!("bucket output") };
    assert_eq!(sum.fields[0].value, Some(StatisticValue::Number(4.0)));
    let predicate = Predicate {
        field: "counter".into(),
        operator: Operator::Gt,
        values: vec![Scalar::Number(big as f64)],
    };
    let mut filtered = request;
    filtered.r#where = vec![predicate.clone()];
    let result = fixture.runtime.query_aggregate_samples(filtered, usize::MAX).await.unwrap();
    assert_eq!(result.selected_rows, 2);
    bucket.r#where = vec![predicate];
    let result = fixture.page(bucket).await;
    let SampleRow::Bucket(sum) = &result.rows[0] else { panic!("bucket output") };
    assert_eq!(sum.sample_count, 2);
    fixture.owner.shutdown().await.unwrap();
    drop(fixture);

    let mut fixture = Fixture::new(|config| {
        config.local_storage.telemetry_rows = std::num::NonZeroUsize::new(64).unwrap();
        config.query.working_bytes = 1024 * 1024;
    })
    .await;
    for start in (0..16384).step_by(64) {
        let rows = (start..start + 64).map(|index| (index as u64, index as i64, Some(index as f64), "a")).collect::<Vec<_>>();
        fixture.publish(&rows).await;
    }
    fixture.publish(&[(16384, 16384, Some(16384.0), "a")]).await;
    let result = fixture
        .runtime
        .query_aggregate_samples(
            AggregateSamples {
                execution_id: fixture.execution.to_string(),
                stream: "query.readings".into(),
                versions: Vec::new(),
                r#where: Vec::new(),
                between: None,
                at: None,
                outputs: [harmos::Aggregate::Median, harmos::Aggregate::Percentile(95), harmos::Aggregate::Sum, harmos::Aggregate::Mean]
                    .into_iter()
                    .enumerate()
                    .map(|(index, aggregate)| Output {
                        alias: format!("s{index}"),
                        field: "value".into(),
                        aggregate,
                    })
                    .collect(),
            },
            usize::MAX,
        )
        .await
        .unwrap();
    assert_eq!(result.selected_rows, 16385);
    assert!(result.complete);
    assert_eq!(result.values.len(), 4);
    assert_eq!(result.quality_counts[0].finite, 16385);
    for (value, expected) in result.values.iter().zip([8192.0f64, 15564.8, 134225920.0, 8192.0]) {
        let Some(StatisticValue::Number(actual)) = value.value else { panic!("numeric percentile") };
        assert_eq!(actual.to_bits(), expected.to_bits());
    }
    fixture.owner.shutdown().await.unwrap();
}
