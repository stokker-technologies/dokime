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

// Local Imports
use super::support::{
    TestResult,
    server::{self, FakeServer},
};
use dokime::{
    Client, ClientSettings,
    client::{CatalogEntriesListRequest, CatalogKind},
    protocol::client::v1 as client,
    v1,
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

#[tokio::test]
async fn discover_bind_call_and_follow_a_typed_backend() -> TestResult {
    // Connect with Explicit Consumer-Owned Bounds.
    let server = FakeServer::start().await?;
    let settings = ClientSettings {
        transfer: harmos_protocol::storage::StorageTransferSettings { chunk_bytes: Some(1024) },
        message_bytes: NonZeroUsize::new(1_048_576).unwrap(),
        retained_bytes: NonZeroUsize::new(16 * 1_048_576).unwrap(),
        input_items: NonZeroUsize::new(8).unwrap(),
        input_bytes: NonZeroUsize::new(1_048_576).unwrap(),
    };
    let client = Client::from_channel(server.channel.clone(), settings)?;
    let overview = client.method::<client::backend::OverviewGetRequest, client::backend::OverviewGetResponse>(server::contract())?;

    // Authentication Failures Keep Their gRPC Status.
    let error = client.call(&overview, client::backend::OverviewGetRequest::default()).await.unwrap_err();
    assert_eq!(error.code(), tonic::Code::Unauthenticated);
    let client = client.authorization("Bearer story".parse()?);

    // Discover Operations and Bind the Authoritative Generated Types.
    let page = client
        .catalog(CatalogEntriesListRequest {
            kind: CatalogKind::Operations as i32,
            ..Default::default()
        })
        .await?
        .into_inner();
    assert_eq!(page.operations.len(), 1);
    let contract = page.operations[0].contract.clone().ok_or("Missing Discovered Contract")?;
    let overview = client.method::<client::backend::OverviewGetRequest, client::backend::OverviewGetResponse>(contract.clone())?;
    assert!(client.method::<client::backend::OverviewGetRequest, client::ExecutionsGetResponse>(contract).is_err());

    // Read Current State Without Native Host Types or a Second Codec.
    let response = client.call(&overview, client::backend::OverviewGetRequest::default()).await?.into_inner();
    assert_eq!(response.overview.ok_or("Missing Overview")?.session_id, "bench");

    // Follow the Ordinary Resource Watch with Caller Metadata and Consumer Bounds.
    let mut request = tonic::Request::new(client::backend::OverviewWatchRequest::default());
    request.metadata_mut().insert("authorization", "Bearer story".parse()?);
    let mut watching = client.backend.overview.watch(request).await?;
    let opening = watching.next().await?.ok_or("Missing Opening Snapshot")?;
    assert!(opening.opening);
    assert_eq!(opening.overview.ok_or("Missing Watched Overview")?.session_id, "bench");
    assert!(watching.next().await?.is_none());
    drop(watching);

    // Refuse a Contract Whose Advertised Schema Fingerprint Was Altered.
    let mut contract = server::contract();
    contract.fingerprint[0] ^= 1;
    assert!(client.method::<client::backend::OverviewGetRequest, client::backend::OverviewGetResponse>(contract).is_err());

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
            transfer: harmos_protocol::storage::StorageTransferSettings { chunk_bytes: Some(256) },
            message_bytes: NonZeroUsize::new(1024).unwrap(),
            ..settings
        },
    )?
    .authorization("Bearer story".parse()?);
    assert!(bounded.call(&overview, client::backend::OverviewGetRequest::default()).await.is_err());

    // Admit a Cycle and Traverse Its Execution Pages at the Write's Durable Cut.
    let host = dokime::client::HostClient::new(server.channel.clone(), settings)?;
    let cases = vec![
        client::PlannedCase {
            origin: Some(v1::Origin::default()),
            ..Default::default()
        };
        2
    ];
    let cycle = host.cycles.start(cases).await?;
    assert_eq!(cycle.executions, [format!("{}/0", cycle.identifier), format!("{}/1", cycle.identifier)]);
    assert!(host.cycles.start(vec![client::PlannedCase::default()]).await.is_err());
    drop(cycle);
    host.close();

    // Release the Connection and Stop the Owned In-Process Server.
    drop(bounded);
    drop(client);
    server.close().await?;
    Ok(())
}

/* === Tests === */
