/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Finite Query Transport Contracts

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

// Third Party Imports
use dokime::harmos::operations::{
    CallContext, OperationDeclaration,
    authorization::{Authorization, Authorizer, Provider},
};
use tonic_types::StatusExt;

// Local Imports
use super::*;
use dokime::domain::model::query::{
    error::{Failure, Reason},
    page::PageRequest,
    reduction::Output,
    selection::{CycleSelection, ExecutionSelection, RecordScope},
};
use dokime::operations::query::*;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

struct ReadAccess(AtomicBool);

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Authorizer for ReadAccess {
    fn authorize(&self, _: &CallContext, provider: Provider<'_>) -> dokime::harmos::Result<Option<Authorization>> {
        Ok(self.0.load(Ordering::SeqCst).then(|| Authorization::Full(provider.contract.clone())))
    }
}

/* === Public Functions === */

/* === Private Functions === */

pub(crate) fn cycles() -> ListCycles {
    ListCycles {
        selection: CycleSelection {
            cycle_ids: vec![],
            phases: vec![],
            statuses: vec![],
            created_at: None,
            archive: None,
        },
        page: PageRequest { rows: Some(1), bytes: None, after: None },
        at: None,
    }
}

fn detail(status: &tonic::Status) -> Failure {
    let details = status.get_error_details();
    let info = details.error_info().unwrap();
    assert_eq!(info.domain, "dokime.query");
    serde_json::from_str(&info.metadata["query"]).unwrap()
}

/* === Tests === */

#[dokime_tracing::trace_test]
#[tokio::test]
async fn finite_query_transports_and_cli_pages() {
    let root = tempfile::tempdir().unwrap();
    let prepared = dokime_server::grpc::Service::prepare(keyed()).unwrap();
    let mut owner = runtime(root.path(), &prepared).await;
    let server = prepared.ready().await.unwrap().listen().await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let handle = owner.handle();
    for label in ["First", "Second"] {
        client
            .unary::<contracts::execution::CreateCycle>(authenticated(contracts::execution::CreateCycle {
                metadata: Some(dokime::domain::model::identity::wire::Metadata {
                    identifier: label.to_lowercase(),
                    label: label.into(),
                    ..Default::default()
                }),
                continue_automatically: false,
                context: None,
            }))
            .await
            .unwrap();
    }
    let first = client.unary::<ListCycles>(authenticated(cycles())).await.unwrap().into_inner();
    assert_eq!(first.rows.len(), 1);
    assert_eq!(first.rows[0].status, dokime::domain::model::execution::wire::CycleStatus::Ready);
    assert_eq!(first.rows[0].running_count, 0);
    assert!(!first.complete);
    let mut next = cycles();
    next.page.after = first.next.clone();
    let native = handle.operations().call::<ListCycles>(next.clone()).await.unwrap();
    let second = client.unary::<ListCycles>(authenticated(next.clone())).await.unwrap().into_inner();
    assert_eq!(native.rows, second.rows);
    assert_ne!(first.rows[0].cycle_id, second.rows[0].cycle_id);
    assert!(second.complete);
    assert_eq!(first.snapshot, second.snapshot);
    let mut filtered = cycles();
    filtered.selection.statuses = vec![dokime::domain::model::execution::wire::CycleStatus::Running];
    assert!(client.unary::<ListCycles>(authenticated(filtered.clone())).await.unwrap().into_inner().rows.is_empty());
    filtered.selection.statuses.push(dokime::domain::model::execution::wire::CycleStatus::Ready);
    filtered.at =
        first.snapshot.as_ref().map(|snapshot| dokime::domain::model::query::snapshot::SnapshotRef::Token(snapshot.token.clone()));
    let selected = client.unary::<ListCycles>(authenticated(filtered.clone())).await.unwrap().into_inner();
    let native = handle.operations().call::<ListCycles>(filtered).await.unwrap();
    assert_eq!(native.rows, selected.rows);
    assert_eq!(selected.rows, first.rows);
    let mut invalid = cycles();
    invalid.selection.statuses = vec![dokime::domain::model::execution::wire::CycleStatus::Unspecified];
    let error = client.unary::<ListCycles>(authenticated(invalid)).await.unwrap_err();
    assert_eq!(error.code(), Code::InvalidArgument);
    assert_eq!(detail(&error).reason, Reason::InvalidSelection);
    let mut capped = cycles();
    capped.page.bytes = Some(u64::MAX);
    let capped = client.unary::<ListCycles>(authenticated(capped)).await.unwrap().into_inner();
    let maximum = handle.query_config().limits().max_page_bytes.min(keyed().max_message_bytes() as u64);
    assert_eq!(capped.limits.unwrap().bytes, maximum);

    assert_eq!(client.unary::<ListCycles>(cycles()).await.unwrap_err().code(), Code::Unauthenticated);

    // The Real Generic CLI Reads And Publishes Both Canonical Page Requests.
    use clap::Parser as _;
    let cli_config = Config::remote("127.0.0.1:0".parse().unwrap(), None, None).unwrap();
    let cli_server = prepared.ready().await.unwrap().listen_with(cli_config).await.unwrap();
    for (name, request) in [("first", cycles()), ("next", next)] {
        let input = root.path().join(format!("{name}-request.json"));
        let output = root.path().join(format!("{name}-page.json"));
        // The CLI Uses Protobuf JSON, Including Uint32 Wrappers.
        let json = if name == "first" {
            serde_json::json!({"selection":{},"page":{"rows":1}})
        } else {
            serde_json::json!({"selection":{},"page":{"rows":1,"after":request.page.after}})
        };
        std::fs::write(&input, json.to_string()).unwrap();
        let endpoint = cli_server.local_endpoint();
        let args = dokime_cli::Cli::try_parse_from([
            "dokime",
            "operation",
            "call",
            ListCycles::ID,
            "--endpoint",
            &endpoint,
            "--input",
            input.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ])
        .unwrap();
        args.run(dokime::config::Config::local(root.path())).await.unwrap();
        let events: serde_json::Value = serde_json::from_slice(&std::fs::read(output).unwrap()).unwrap();
        assert_eq!(events[0]["event"], "completion");
        assert_eq!(events[0]["value"]["rows"].as_array().unwrap().len(), 1);
        assert_eq!(events[0]["value"]["complete"].as_bool().unwrap_or(false), name == "next");
    }
    cli_server.shutdown().await.unwrap();
    drop(client);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn finite_query_failures_and_cancellation() {
    let root = tempfile::tempdir().unwrap();
    let prepared = dokime_server::grpc::Service::prepare(keyed()).unwrap();
    let mut owner = runtime(root.path(), &prepared).await;
    let server = prepared.ready().await.unwrap().listen().await.unwrap();
    let client = WireClient::connect(server.local_endpoint()).await.unwrap();
    let handle = owner.handle();
    let page = PageRequest { rows: None, bytes: None, after: None };
    let missing = uuid::Uuid::now_v7().to_string();
    let records = ListRecords {
        scope: RecordScope::Execution(missing.clone()),
        record_type: "fixture.record".into(),
        versions: vec![],
        r#where: vec![],
        between: None,
        page: page.clone(),
        at: None,
    };
    let status = client.unary::<ListRecords>(authenticated(records)).await.unwrap_err();
    assert_eq!(status.code(), Code::NotFound);
    assert_eq!(detail(&status).reason, Reason::ScopeNotFound);
    let samples = ListSamples {
        execution_id: missing.clone(),
        stream: "fixture.stream".into(),
        versions: vec![],
        fields: vec![],
        r#where: vec![],
        between: None,
        bucket: None,
        page: page.clone(),
        at: None,
    };
    let status = client.unary::<ListSamples>(authenticated(samples)).await.unwrap_err();
    assert_eq!(status.code(), Code::NotFound);
    assert_eq!(detail(&status).operation, ListSamples::ID);
    let aggregate = AggregateSamples {
        execution_id: missing,
        stream: "fixture.stream".into(),
        versions: vec![],
        r#where: vec![],
        between: None,
        outputs: vec![Output {
            alias: "count".into(),
            field: String::new(),
            aggregate: dokime::harmos::Aggregate::Count,
        }],
        at: None,
    };
    let status = client.unary::<AggregateSamples>(authenticated(aggregate)).await.unwrap_err();
    assert_eq!(status.code(), Code::NotFound);
    assert_eq!(detail(&status).operation, AggregateSamples::ID);
    let executions = ListExecutions {
        selection: ExecutionSelection {
            template_ids: vec![],
            cycle_ids: vec![],
            statuses: vec![],
            started_at: None,
            archive: None,
        },
        page,
        at: None,
    };
    assert!(client.unary::<ListExecutions>(authenticated(executions)).await.unwrap().into_inner().complete);
    let mut invalid = cycles();
    invalid.page.rows = Some(0);
    let status = client.unary::<ListCycles>(authenticated(invalid)).await.unwrap_err();
    assert_eq!(status.code(), Code::InvalidArgument);
    assert_eq!(detail(&status).reason, Reason::InvalidSelection);
    let mut unavailable = cycles();
    unavailable.page.after = Some("invalid".into());
    let status = client.unary::<ListCycles>(authenticated(unavailable)).await.unwrap_err();
    assert_eq!(status.code(), Code::InvalidArgument);
    assert_eq!(detail(&status).reason, Reason::InvalidContinuation);

    // Trusted Revocation Context Cancels Work Before Capture And Leaves Admission Reusable.
    let revoked = tokio_util::sync::CancellationToken::new();
    revoked.cancel();
    let mut context = CallContext::default();
    context
        .insert(dokime::runtime::query::Caller {
            message_bytes: 65536,
            revoked: Some(revoked),
        })
        .unwrap();
    let error = handle.operations().with_context(context).call::<ListCycles>(cycles()).await.unwrap_err();
    let dokime::harmos::Error::OperationFailed { source, .. } = error else { panic!("expected an operation failure") };
    assert_eq!(source.downcast_ref::<tonic::Status>().unwrap().code(), Code::PermissionDenied);
    let cancelled = handle.operations().submit::<ListCycles>(cycles()).unwrap();
    cancelled.request_cancel();
    assert!(cancelled.await.is_err());
    assert!(handle.operations().call::<ListCycles>(cycles()).await.unwrap().complete);
    assert!(client.unary::<ListCycles>(authenticated(cycles())).await.unwrap().into_inner().rows.is_empty());
    drop(client);
    server.shutdown().await.unwrap();
    owner.shutdown().await.unwrap();
}

#[dokime_tracing::trace_test]
#[tokio::test]
async fn finite_query_continuation_rechecks_authorization() {
    let root = tempfile::tempdir().unwrap();
    let metadata = Metadata::new(Identifier::new("query.access").unwrap(), "Query Access", None);
    let config = crate::support::budgets::runtime_config(root.path(), metadata.clone());
    let mut owner = dokime::Runtime::start(config).await.unwrap();
    let handle = owner.handle();
    let access = Arc::new(ReadAccess(AtomicBool::new(true)));
    handle.operations().register_authorizer(access.clone()).unwrap();
    for _ in 0..2 {
        handle
            .create_cycle(
                dokime::transactions::CycleDefinition {
                    metadata: metadata.clone(),
                    continue_automatically: false,
                },
                Default::default(),
            )
            .await
            .unwrap();
    }
    let listener = handle.grpc().attach(dokime::harmos::grpc::Settings::default()).await.unwrap();
    let client = WireClient::connect(format!("http://{}", listener.local_addr())).await.unwrap();
    let first = client.unary::<ListCycles>(cycles()).await.unwrap().into_inner();
    let mut next = cycles();
    next.page.after = Some(first.next.unwrap());
    access.0.store(false, Ordering::SeqCst);
    assert_eq!(client.unary::<ListCycles>(next.clone()).await.unwrap_err().code(), Code::PermissionDenied);
    access.0.store(true, Ordering::SeqCst);
    let last = client.unary::<ListCycles>(next).await.unwrap().into_inner();
    assert!(last.complete);
    assert_eq!(last.rows.len(), 1);
    drop(client);
    listener.detach().await.unwrap();
    owner.shutdown().await.unwrap();
}
