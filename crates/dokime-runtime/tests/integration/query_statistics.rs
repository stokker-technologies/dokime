/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Exact Fixed-Cut Telemetry Statistics

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use crate::query_telemetry::Fixture;
use dokime_domain::model::query::{
    error::Reason,
    predicate::{Operator, Predicate, Scalar},
    reduction::{Output, StatisticValue},
    schema::FieldType,
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
async fn exact_query_statistics() {
    use harmos::Aggregate;
    let mut fixture = Fixture::new(|_| {}).await;
    fixture
        .publish(&[(0, 0, Some(0.0), "a"), (1, 1, Some(10.0), "a"), (2, 2, Some(20.0), "a"), (3, 3, Some(30.0), "a"), (4, 4, None, "a")])
        .await;
    let first = fixture.page(fixture.request()).await;
    let mut outputs = vec![Output {
        alias: "rows".into(),
        field: String::new(),
        aggregate: Aggregate::Count,
    }];
    for (alias, aggregate) in [
        ("count", Aggregate::Count),
        ("sum", Aggregate::Sum),
        ("mean", Aggregate::Mean),
        ("min", Aggregate::Min),
        ("max", Aggregate::Max),
        ("median", Aggregate::Median),
        ("p95", Aggregate::Percentile(95)),
        ("std", Aggregate::Std),
        ("rms", Aggregate::Rms),
    ] {
        outputs.push(Output {
            alias: alias.into(),
            field: "value".into(),
            aggregate,
        });
    }
    let request = AggregateSamples {
        execution_id: fixture.execution.to_string(),
        stream: "query.readings".into(),
        versions: Vec::new(),
        r#where: Vec::new(),
        between: None,
        outputs,
        at: Fixture::at(first.snapshot.as_ref().unwrap()),
    };
    fixture.publish(&[(5, 2, Some(1000.0), "late")]).await;
    let result = fixture.runtime.query_aggregate_samples(request.clone(), usize::MAX).await.unwrap();
    assert!(result.complete);
    assert_eq!(result.selected_rows, 5);
    assert_eq!(result.values[0].value, Some(StatisticValue::Count(5)));
    assert_eq!(result.values[1].value, Some(StatisticValue::Count(4)));
    for (statistic, expected) in result.values[2..].iter().zip([60.0, 15.0, 0.0, 30.0, 15.0, 28.5, 125.0f64.sqrt(), 350.0f64.sqrt()]) {
        let Some(StatisticValue::Number(actual)) = statistic.value else { panic!("numeric statistic") };
        assert!((actual - expected).abs() < 1e-12, "{}: {actual} != {expected}", statistic.alias);
    }
    assert_eq!(result.quality_counts[0].finite, 4);
    assert_eq!(result.quality_counts[0].null, 1);
    assert_eq!(result.schema.as_ref().unwrap().fields[0].value_type, FieldType::Uint64);
    assert_eq!(result.schema.as_ref().unwrap().fields[2].value_type, FieldType::Float64);
    let mut filtered = request.clone();
    filtered.r#where = vec![Predicate {
        field: "value".into(),
        operator: Operator::Gte,
        values: vec![Scalar::Number(20.0)],
    }];
    let selected = fixture.runtime.query_aggregate_samples(filtered, usize::MAX).await.unwrap();
    assert_eq!(selected.selected_rows, 2);
    assert_eq!(selected.values[6].value, Some(StatisticValue::Number(25.0)));
    let mut empty = request.clone();
    empty.r#where = vec![Predicate {
        field: "counter".into(),
        operator: Operator::Gt,
        values: vec![Scalar::Unsigned(100)],
    }];
    let empty = fixture.runtime.query_aggregate_samples(empty, usize::MAX).await.unwrap();
    assert_eq!(empty.values[1].value, Some(StatisticValue::Count(0)));
    assert!(empty.values[2..].iter().all(|value| value.value.is_none()));
    let mut fresh = request.clone();
    fresh.at = None;
    assert_eq!(fixture.runtime.query_aggregate_samples(fresh, usize::MAX).await.unwrap().selected_rows, 6);
    fixture.owner.shutdown().await.unwrap();

    let mut restricted = Fixture::new(|config| config.query.scanned_rows = 7).await;
    restricted
        .publish(&[(0, 0, Some(0.0), "a"), (1, 1, Some(10.0), "a"), (2, 2, Some(20.0), "a"), (3, 3, Some(30.0), "a"), (4, 4, None, "a")])
        .await;
    let mut exhausted = request;
    exhausted.execution_id = restricted.execution.to_string();
    exhausted.at = None;
    assert_eq!(restricted.runtime.query_aggregate_samples(exhausted, usize::MAX).await.unwrap_err().reason, Reason::WorkLimit);
    restricted.owner.shutdown().await.unwrap();
}
