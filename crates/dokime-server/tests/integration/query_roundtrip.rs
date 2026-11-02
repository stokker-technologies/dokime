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
use prost::Message as _;

// Local Imports
use super::*;
use crate::support::wire::WireMessage;
use dokime::domain::model;

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

#[dokime_tracing::trace_test]
#[tokio::test]
async fn query_roundtrip() {
    let root = tempfile::tempdir().unwrap();
    let prepared = dokime_server::grpc::Service::prepare(keyed()).unwrap();
    let mut owner = runtime(root.path(), &prepared).await;
    let server = prepared.ready().await.unwrap().listen().await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();

    let baseline = contracts::query::Aggregate {
        source: model::query::wire::QuerySource::Executions,
        filters: Vec::new(),
        group_by: vec!["status".to_owned()],
        aggregates: vec![model::query::wire::QueryAggregate {
            aggregate: dokime::harmos::Aggregate::Count,
            column: String::new(),
            alias: "total".to_owned(),
        }],
        limit: 0,
    };
    let empty = client.unary::<contracts::query::Aggregate>(authenticated(baseline.clone())).await.unwrap().into_inner();

    // A runtime that has executed nothing still answers the contract's shape.
    assert_eq!(empty.columns, ["status", "total"]);
    assert!(empty.rows.is_empty());
    assert!(empty.complete);

    let status = client
        .unary::<contracts::query::Aggregate>(authenticated(contracts::query::Aggregate {
            source: model::query::wire::QuerySource::Steps,
            filters: Vec::new(),
            group_by: vec!["step_id".to_owned()],
            aggregates: vec![model::query::wire::QueryAggregate {
                aggregate: dokime::harmos::Aggregate::Mean,
                column: "duration_ms".to_owned(),
                alias: "mean".to_owned(),
            }],
            limit: 0,
        }))
        .await
        .unwrap_err();

    assert_eq!(status.code(), tonic::Code::InvalidArgument);
    assert!(status.message().contains("step_id"));
    assert!(status.message().contains("template_identifier"));

    let request = |aggregate| contracts::query::Aggregate {
        source: model::query::wire::QuerySource::Steps,
        filters: Vec::new(),
        group_by: Vec::new(),
        aggregates: vec![model::query::wire::QueryAggregate {
            aggregate,
            column: "duration_ms".to_owned(),
            alias: "statistic".to_owned(),
        }],
        limit: 0,
    };

    // Percentile Ranks Survive The Wire Codec; Invalid Ranks Cannot Poison A Later Valid Query.
    for rank in [101, 255] {
        let status = client
            .unary::<contracts::query::Aggregate>(authenticated(request(dokime::harmos::Aggregate::Percentile(rank))))
            .await
            .unwrap_err();

        assert_eq!(status.code(), tonic::Code::InvalidArgument, "{rank}");
    }

    let reply = client
        .unary::<contracts::query::Aggregate>(authenticated(request(dokime::harmos::Aggregate::Percentile(95))))
        .await
        .unwrap()
        .into_inner();
    assert_eq!(reply.columns, ["statistic"]);
    assert!(reply.complete);

    let status = client
        .unary::<contracts::query::Aggregate>(authenticated(contracts::query::Aggregate {
            source: model::query::wire::QuerySource::Unspecified,
            ..baseline.clone()
        }))
        .await
        .unwrap_err();

    assert_eq!(status.code(), tonic::Code::InvalidArgument);

    let home =
        std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")).expect("the integration environment declares its home directory");
    assert!(!home.is_empty());
    let status = client
        .unary::<contracts::query::Aggregate>(authenticated(contracts::query::Aggregate {
            source: model::query::wire::QuerySource::Steps,
            group_by: vec![format!("{home}/evidence")],
            aggregates: vec![model::query::wire::QueryAggregate {
                aggregate: dokime::harmos::Aggregate::Count,
                column: String::new(),
                alias: "total".into(),
            }],
            filters: vec![],
            limit: 0,
        }))
        .await
        .unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
    let envelope = tonic_types::Status::decode(status.details()).unwrap();
    let detail = envelope.details.first().expect("the mapper includes a detailed error");
    assert_eq!(detail.type_url, "type.googleapis.com/dokime.v1.CapturedRecordError");
    let detail = <model::evidence::wire::CapturedRecordError as WireMessage>::from_wire(detail.value.as_slice()).unwrap();
    assert!(detail.message.contains("evidence"));
    assert!(!detail.message.contains(&home), "a wire diagnostic is redacted: {}", detail.message);
    let recovered = client.unary::<contracts::query::Aggregate>(authenticated(baseline)).await.unwrap().into_inner();
    assert_eq!(recovered.columns, empty.columns);
    assert_eq!(recovered.rows, empty.rows);
    assert!(recovered.complete);
    drop(client);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}
