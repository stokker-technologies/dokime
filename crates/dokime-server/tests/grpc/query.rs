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
use super::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
async fn client(root: &std::path::Path) -> (impl Sized, WireClient) {
    let owner = runtime(root).await;
    let config = Config::default()
        .with_bearer_auth(vec![grant(Role::Read)])
        .unwrap();
    let server = Server::start(owner.handle(), config).await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();

    ((owner, server), client)
}

pub(crate) async fn aggregate_answers_asked() {
    let root = tempfile::tempdir().unwrap();
    let (_held, client) = client(root.path()).await;

    let reply = client
        .unary::<contracts::query::Aggregate>(authenticated(contracts::query::AggregateRequest {
            source: contracts::query::QuerySource::Executions,
            filters: Vec::new(),
            group_by: vec!["status".to_owned()],
            aggregates: vec![contracts::query::QueryAggregate {
                function: contracts::query::AggregateFunction::Count,
                column: String::new(),
                alias: "total".to_owned(),
            }],
            limit: 0,
        }))
        .await
        .unwrap()
        .into_inner();

    // A runtime that has executed nothing still answers the contract's shape.
    assert_eq!(reply.columns, ["status", "total"]);
    assert!(reply.rows.is_empty());
    assert!(reply.complete);
}

pub(crate) async fn unknown_column_exist() {
    let root = tempfile::tempdir().unwrap();
    let (_held, client) = client(root.path()).await;

    let status = client
        .unary::<contracts::query::Aggregate>(authenticated(contracts::query::AggregateRequest {
            source: contracts::query::QuerySource::Steps,
            filters: Vec::new(),
            group_by: vec!["step_id".to_owned()],
            aggregates: vec![contracts::query::QueryAggregate {
                function: contracts::query::AggregateFunction::Mean,
                column: "duration_ms".to_owned(),
                alias: "mean".to_owned(),
            }],
            limit: 0,
        }))
        .await
        .unwrap_err();

    assert_eq!(status.code(), tonic::Code::InvalidArgument);
    assert!(status.message().contains("unknown column step_id"));
    assert!(status.message().contains("template_identifier"));
}

pub(crate) async fn unspecified_source_read() {
    let root = tempfile::tempdir().unwrap();
    let (_held, client) = client(root.path()).await;

    let status = client
        .unary::<contracts::query::Aggregate>(authenticated(contracts::query::AggregateRequest {
            source: contracts::query::QuerySource::Unspecified,
            filters: vec![],
            group_by: vec![],
            aggregates: vec![],
            limit: 0,
        }))
        .await
        .unwrap_err();

    assert_eq!(status.code(), tonic::Code::InvalidArgument);
    assert_eq!(status.message(), "aggregate source is unspecified");
}

/* === Tests === */

pub(crate) async fn detailed_status_directory() {
    use prost::Message;

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .expect("the integration environment declares its home directory");
    assert!(!home.is_empty());
    let root = tempfile::tempdir().unwrap();
    let (held, client) = client(root.path()).await;
    let status = client
        .unary::<contracts::query::Aggregate>(authenticated(contracts::query::AggregateRequest {
            source: contracts::query::QuerySource::Steps,
            group_by: vec![format!("{home}/evidence")],
            aggregates: vec![contracts::query::QueryAggregate {
                function: contracts::query::AggregateFunction::Count,
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
    let detail = envelope
        .details
        .first()
        .expect("the mapper includes a detailed error");
    assert_eq!(
        detail.type_url,
        "type.googleapis.com/dokime.v1.CapturedRecordError"
    );
    let detail = <contracts::evidence::CapturedRecordError as harmos::Protobuf>::decode(
        detail.value.as_slice(),
    )
    .unwrap();
    assert!(detail.message.contains("evidence"));
    assert!(
        !detail.message.contains(&home),
        "a wire diagnostic is redacted: {}",
        detail.message
    );
    drop(client);
    drop(held);
}
