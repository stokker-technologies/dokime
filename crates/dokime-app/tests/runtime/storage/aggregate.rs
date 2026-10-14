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
use crate::support::prelude::*;
use dokime_app::runtime::query::aggregate::{AggregateColumn, AggregateQuery, AggregateSource};
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
    let mut runtime =
        Runtime::with_execution_fixtures_without_builtins(config(root.path()), std::iter::empty())
            .await
            .unwrap();
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
    let message = error.to_string();
    assert!(message.contains("unknown column template_id"), "{message}");
    assert!(message.contains("case_identifier"), "{message}");
    runtime.shutdown().await.unwrap();
}
