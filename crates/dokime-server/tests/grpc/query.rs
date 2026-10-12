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
use dokime_protocol::v1::query_client::QueryClient;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */
async fn client(root: &std::path::Path) -> (impl Sized, QueryClient<tonic::transport::Channel>) {
    let owner = runtime(root).await;
    let config = Config::default()
        .with_bearer_auth(vec![grant(Role::Read)])
        .unwrap();
    let server = Server::start(owner.handle(), config).await.unwrap();
    let client = QueryClient::connect(server.local_endpoint()).await.unwrap();

    ((owner, server), client)
}

pub(crate) async fn aggregate_answers_asked() {
    let root = tempfile::tempdir().unwrap();
    let (_held, mut client) = client(root.path()).await;

    let reply = client
        .aggregate(authenticated(v1::AggregateRequest {
            source: v1::QuerySource::Executions as i32,
            filters: Vec::new(),
            group_by: vec!["status".to_owned()],
            aggregates: vec![v1::QueryAggregate {
                function: v1::AggregateFunction::Count as i32,
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
    let (_held, mut client) = client(root.path()).await;

    let status = client
        .aggregate(authenticated(v1::AggregateRequest {
            source: v1::QuerySource::Steps as i32,
            filters: Vec::new(),
            group_by: vec!["step_id".to_owned()],
            aggregates: vec![v1::QueryAggregate {
                function: v1::AggregateFunction::Mean as i32,
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
    let (_held, mut client) = client(root.path()).await;

    let status = client
        .aggregate(authenticated(v1::AggregateRequest::default()))
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
    let (held, mut client) = client(root.path()).await;
    let status = client
        .aggregate(authenticated(v1::AggregateRequest {
            source: v1::QuerySource::Steps as i32,
            group_by: vec![format!("{home}/evidence")],
            aggregates: vec![v1::QueryAggregate {
                function: v1::AggregateFunction::Count as i32,
                column: String::new(),
                alias: "total".into(),
            }],
            ..Default::default()
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
    let detail = v1::CapturedRecordError::decode(detail.value.as_slice()).unwrap();
    assert!(detail.message.contains("evidence"));
    assert!(
        !detail.message.contains(&home),
        "a wire diagnostic is redacted: {}",
        detail.message
    );
    drop(client);
    drop(held);
}
