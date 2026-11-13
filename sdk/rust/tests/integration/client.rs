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
    let health = client.method::<v1::GetBackendHealth, v1::BackendHealth>(server::contract(false))?;

    // Authentication Failures Keep Their gRPC Status.
    let error = client.call(&health, v1::GetBackendHealth {}).await.unwrap_err();
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
    assert_eq!(page.operations.len(), 2);
    let contract = page.operations[0].contract.clone().ok_or("Missing Discovered Contract")?;
    let health = client.method::<v1::GetBackendHealth, v1::BackendHealth>(contract.clone())?;
    assert!(client.method::<v1::GetBackendHealth, v1::BackendIdentity>(contract).is_err());

    // Read Current State Without Native Host Types or a Second Codec.
    let response = client.call(&health, v1::GetBackendHealth {}).await?.into_inner();
    assert_eq!(response.backend_id, "bench");
    assert_eq!(response.observed_at, Some(prost_types::Timestamp { seconds: 100, nanos: 0 }));

    // Follow Managed Output and Join Its Independently Typed Final Result.
    let contract = page.operations[1].contract.clone().ok_or("Missing Watch Contract")?;
    let watch = client.method::<v1::WatchBackendLogs, v1::BackendWatchLogsCompletion>(contract)?.with_output::<v1::LogFrame>()?;
    let mut running = client
        .start(
            &watch,
            v1::WatchBackendLogs {
                min_level: v1::LogLevel::Info as i32,
                replay_lines: 2,
            },
        )
        .await?;
    assert_eq!(running.next().await?.ok_or("Missing Log Item")?.target, "sdk.story");
    assert!(running.next().await?.is_none());
    running.join().await?;

    // Refuse a Contract Whose Advertised Schema Fingerprint Was Altered.
    let mut contract = server::contract(false);
    contract.fingerprint[0] ^= 1;
    assert!(client.method::<v1::GetBackendHealth, v1::BackendHealth>(contract).is_err());

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
    assert!(bounded.call(&health, v1::GetBackendHealth {}).await.is_err());

    // Release the Connection and Stop the Owned In-Process Server.
    drop(bounded);
    drop(client);
    server.close().await?;
    Ok(())
}

/* === Tests === */
