/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */


// Inspect the Contract Before Invoking an Operation

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
async fn operator_inspects_contract() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::HANG_GUARD, async {
        // Discover a Public Operation Before Inspecting Its Contract
        let host = super::HostProcess::start().await?;
        let listed: serde_json::Value = serde_json::from_str(&super::run(&[
            "--endpoint", &host.endpoint, "operation", "list", "--json"
        ]).await?)?;
        let operations = listed["operations"].as_array().unwrap();
        let selected = operations.iter().find(|entry| entry["contract"]["id"] == "harmos.v1.Status/Get").unwrap();
        let id = selected["contract"]["id"].as_str().unwrap();

        // Read Identity, Visibility, Shapes and Policy From That Same Contract
        let shown: serde_json::Value = serde_json::from_str(&super::run(&[
            "--endpoint", &host.endpoint, "operation", "show", id, "--json"
        ]).await?)?;
        assert_eq!(shown["contract"], selected["contract"]);
        assert_eq!(shown["request_shape"]["message"], selected["contract"]["protobuf"]["request"]);
        assert_eq!(shown["response_shape"]["message"], selected["contract"]["protobuf"]["response"]);
        assert!(!shown["response_shape"]["fields"].as_array().unwrap().is_empty());

        // A Missing Contract Fails Without Returning Another Operation
        let refused = super::command().args(["--endpoint", &host.endpoint, "operation", "show", "absent.v1.Operation/Get", "--json"]).output().await?;
        assert_eq!(refused.status.code(), Some(3));
        assert!(refused.stdout.is_empty());
        assert!(String::from_utf8(refused.stderr)?.contains("absent.v1.Operation/Get"));
        host.close().await?;
        Ok(())
    }).await?
}

