/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::num::NonZeroUsize;

// Third Party Imports
use tonic::{Code, metadata::MetadataMap};

// Local Imports
use super::support::{
    TestResult,
    server::{self, FakeServer},
};
use dokime::client::{
    BackendOverviewWatchOptions, CallOptions, CyclesCreateOptions, ExecutionFilter, ExecutionOrder, ExecutionSelection,
    ExecutionsListOptions, ExecutionsQuery, OperationsListOptions,
};
use dokime::{Client, ClientSettings, Metadata, Origin, PageRequest, ReadCut, StorageTransferSettings};
use dokime_protocol::host::ExecutionsGetResponse;
use dokime_protocol::host::backend::{OverviewGetRequest, OverviewGetResponse};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

/* === Private Functions === */

#[tokio::test]
async fn discover_bind_call_and_follow_a_typed_backend() -> TestResult {
    // Connect with Explicit Consumer-Owned Bounds.
    let server = FakeServer::start().await?;
    let settings = ClientSettings {
        transfer: StorageTransferSettings { chunk_bytes: Some(1024) },
        message_bytes: NonZeroUsize::new(1_048_576).unwrap(),
        retained_bytes: NonZeroUsize::new(16 * 1_048_576).unwrap(),
        input_items: NonZeroUsize::new(8).unwrap(),
        input_bytes: NonZeroUsize::new(1_048_576).unwrap(),
    };
    let client = Client::from_channel(server.channel.clone(), settings)?;
    let overview = client.method::<OverviewGetRequest, OverviewGetResponse>(server::contract())?;

    // Authentication Failures Keep Their gRPC Status.
    let error = client.call(&overview, OverviewGetRequest::default()).await.unwrap_err();
    assert_eq!(error.code(), Code::Unauthenticated);
    let mut metadata = MetadataMap::new();
    metadata.insert("authorization", "Bearer story".parse()?);
    let options = || CallOptions {
        metadata: metadata.clone(),
        ..Default::default()
    };

    // Discover Operations and Bind the Authoritative Generated Types.
    let operations = client.operations.list_with(OperationsListOptions { context: options() }).await?;
    assert_eq!(operations.len(), 1);
    let contract = operations[0].contract.clone().ok_or("Missing Discovered Contract")?;
    let overview = client.method::<OverviewGetRequest, OverviewGetResponse>(contract.clone())?;
    assert!(client.method::<OverviewGetRequest, ExecutionsGetResponse>(contract).is_err());

    // Read Current State Without Native Host Types or a Second Codec.
    let response = client.call_with(&overview, OverviewGetRequest::default(), options()).await?;
    assert_eq!(response.overview.ok_or("Missing Overview")?.session_id, "bench");

    // Follow the Ordinary Resource Watch with Caller Metadata and Consumer Bounds.
    let mut watching = client.backend.overview.watch_with(BackendOverviewWatchOptions { context: options(), ..Default::default() }).await?;
    let opening = watching.next().await?.ok_or("Missing Opening Snapshot")?;
    assert!(opening.opening);
    assert_eq!(opening.overview.ok_or("Missing Watched Overview")?.session_id, "bench");
    assert!(watching.next().await?.is_none());
    drop(watching);

    // Refuse a Contract Whose Advertised Schema Fingerprint Was Altered.
    let mut contract = server::contract();
    contract.fingerprint[0] ^= 1;
    assert!(client.method::<OverviewGetRequest, OverviewGetResponse>(contract).is_err());

    // Refuse a Message Ceiling That Cannot Carry a Bounded Transfer.
    let bounded = Client::from_channel(
        server.channel.clone(),
        ClientSettings {
            message_bytes: NonZeroUsize::new(1).unwrap(),
            ..settings
        },
    );
    assert!(bounded.is_err());

    // Enforce a Valid Consumer Response Bound Before Decoding the Reply.
    let bounded = Client::from_channel(
        server.channel.clone(),
        ClientSettings {
            transfer: StorageTransferSettings { chunk_bytes: Some(256) },
            message_bytes: NonZeroUsize::new(1024).unwrap(),
            ..settings
        },
    )?;
    assert!(bounded.call_with(&overview, OverviewGetRequest::default(), options()).await.is_err());

    // Admit a Cycle and Traverse Its Execution Pages at the Write's Durable Cut.
    let origins = vec![Origin::default(); 2];
    let created = client
        .cycles
        .create_with(
            origins,
            Metadata::default(),
            true,
            "",
            CyclesCreateOptions {
                request_id: Some("cycle".into()),
                ..Default::default()
            },
        )
        .await?;
    let cycle = created.cycle.ok_or("Missing Cycle")?;
    let at = created.receipt.and_then(|receipt| receipt.durable_at).ok_or("Missing Durable Position")?;
    let cut = ReadCut {
        positions: vec![at],
        revisions: Vec::new(),
    };
    let mut executions = Vec::new();
    let mut page_token = String::new();
    loop {
        let page = client
            .executions
            .list_with(
                cycle.cycle_id.clone(),
                ExecutionSelection::default(),
                ExecutionFilter::default(),
                ExecutionOrder::default(),
                ExecutionsListOptions {
                    at: Some(cut.clone()),
                    page: Some(PageRequest {
                        page_size: 1,
                        page_token,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .await?;
        executions.extend(page.executions.into_iter().filter_map(|execution| execution.summary).map(|summary| summary.execution_id));
        if page.next_page_token.is_empty() {
            break;
        }
        page_token = page.next_page_token;
    }
    assert_eq!(executions, ["cycle/0", "cycle/1"]);

    // Bind Every Page to This Client and Retain the First Read Cut.
    let executions = client
        .executions()
        .list_with(ExecutionsQuery {
            cycle: "cycle".into(),
            page_size: 1,
            ..Default::default()
        })
        .await?;
    assert_eq!(executions.iter().map(|execution| execution.identifier()).collect::<Vec<_>>(), ["cycle/0", "cycle/1"]);
    assert_eq!(executions[0].cycle.identifier(), "cycle");
    assert_eq!(executions[0].case.identifier(), "case/0");

    // Release the Connection and Stop the Owned In-Process Server.
    drop(bounded);
    drop(client);
    server.close().await?;
    Ok(())
}

/* === Tests === */
