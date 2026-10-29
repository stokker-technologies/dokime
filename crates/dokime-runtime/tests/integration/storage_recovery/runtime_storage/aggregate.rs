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

// Local Imports
use crate::service_support::prelude::*;
use dokime_runtime::runtime::query::aggregate::{AggregateColumn, AggregateQuery, AggregateSource};
use harmos::Aggregate;

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

pub(crate) async fn unknown_column_exist() {
    let root = tempdir().unwrap();
    let mut runtime = Runtime::with_execution_fixtures_without_builtins(config(root.path()), std::iter::empty()).await.unwrap();
    let error = runtime
        .handle()
        .aggregate(AggregateQuery {
            source: AggregateSource::Executions,
            filters: Vec::new(),
            group_by: vec!["template_id".into()],
            aggregates: vec![AggregateColumn {
                aggregate: Aggregate::Count,
                column: String::new(),
                alias: "total".into(),
            }],
            limit: 0,
        })
        .await
        .unwrap_err();
    assert!(matches!(
        error,
        dokime_runtime::runtime::Error::Admission(dokime_domain::model::diagnostics::admission::AdmissionFailure::Validation { ref code, .. }) if code == "aggregate_query_invalid"
    ));
    runtime.shutdown().await.unwrap();
}

pub(crate) async fn aggregate_width_is_enforced() {
    let root = tempdir().unwrap();
    let settings = config(root.path());

    let mut runtime = Runtime::with_execution_fixtures_without_builtins(settings, std::iter::empty()).await.unwrap();
    let mut query = AggregateQuery {
        source: AggregateSource::Executions,
        filters: Vec::new(),
        group_by: Vec::new(),
        aggregates: (0..65)
            .map(|index| AggregateColumn {
                aggregate: Aggregate::Count,
                column: String::new(),
                alias: format!("count_{index}"),
            })
            .collect(),
        limit: 0,
    };
    let error = runtime.handle().aggregate(query.clone()).await.unwrap_err();
    assert!(matches!(
        error,
        dokime_runtime::runtime::Error::Admission(dokime_domain::model::diagnostics::admission::AdmissionFailure::Validation { ref code, .. }) if code == "aggregate_query_invalid"
    ));

    query.aggregates = vec![AggregateColumn {
        aggregate: Aggregate::Count,
        column: String::new(),
        alias: "total".into(),
    }];
    let outcome = runtime.handle().aggregate(query).await.unwrap();
    assert_eq!(outcome.columns, ["total"]);
    runtime.shutdown().await.unwrap();
}
