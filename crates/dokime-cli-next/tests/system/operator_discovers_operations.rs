/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Discover Admitted Operations Through a Running Host

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports


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
async fn operator_discovers_operations() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::HANG_GUARD, async {
        // Start a Host and Discover Its Admitted Operations
        let host = super::HostProcess::start().await?;
        let output = super::run(&["--endpoint", &host.endpoint, "operation", "list", "--json"]).await?;
        let listed: serde_json::Value = serde_json::from_str(&output)?;

        // Compare the Complete Discovery With the Client Catalog
        let channel = tonic::transport::Endpoint::from_shared(host.endpoint.clone())?.connect().await?;
        let settings = dokime::client::ClientSettings {
            transfer: dokime::client::StorageTransferSettings { chunk_bytes: Some(1024) },
            retained_bytes: std::num::NonZeroUsize::new(64 * 1024 * 1024).unwrap(),
            message_bytes: std::num::NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
            input_items: std::num::NonZeroUsize::new(8).unwrap(),
            input_bytes: std::num::NonZeroUsize::new(4 * 1024 * 1024).unwrap(),
        };
        let client = dokime::Client::from_channel(channel, settings)?;
        let reply = client.catalog(dokime::client::CatalogEntriesListRequest {
            kind: dokime::client::CatalogKind::Operations as i32,
            ..Default::default()
        }).await?.into_inner();
        let actual = listed["operations"].as_array().ok_or("List Operations: Missing Operations")?;
        assert_eq!(actual.len(), reply.operations.len());
        assert!(!actual.is_empty());
        for (actual, expected) in actual.iter().zip(reply.operations) {
            assert_eq!(actual["contract"]["id"], expected.contract.unwrap().id);
        }

        // Close the Host Before Removing Its Protected Credentials
        drop(client);
        host.close().await?;
        Ok(())
    }).await?
}

