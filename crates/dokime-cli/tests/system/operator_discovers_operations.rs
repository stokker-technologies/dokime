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

#[test]
fn operator_discovers_operations() -> Result<(), Box<dyn std::error::Error>> {
    super::credentials::run(Box::pin(async {
        tokio::time::timeout(super::HANG_GUARD, async {
            // Start a Host and Discover Its Admitted Operations
            let host = super::HostProcess::start().await?;
            let output = super::run(&["--endpoint", &host.endpoint, "operation", "list", "--json"]).await?;
            let listed: serde_json::Value = serde_json::from_str(&output)?;

            // Compare the Complete Discovery With the Client Catalog
            let channel = tonic::transport::Endpoint::from_shared(host.endpoint.clone())?.connect().await?;
            let client = dokime_client_internal::Client::from_channel(channel, super::client_settings())?;
            let reply = client.operations.list().await?;
            let actual = listed["operations"].as_array().ok_or("List Operations: Missing Operations")?;
            assert_eq!(actual.len(), reply.len());
            assert!(!actual.is_empty());
            for (actual, expected) in actual.iter().zip(reply) {
                assert_eq!(actual["contract"]["id"], expected.contract.unwrap().id);
            }

            // Close the Host Before Removing Its Protected Credentials
            drop(client);
            host.close().await?;
            Ok(())
        })
        .await?
    }))
}
