/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Invoke a Typed Public Operation and Recover From Refusals

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
async fn operator_calls_operation() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::HANG_GUARD, async {
        // Discover the Public Status Contract and Invoke Its Typed Empty Request
        let host = super::HostProcess::start().await?;
        let list: serde_json::Value =
            serde_json::from_str(&super::run(&["--endpoint", &host.endpoint, "operation", "list", "--json"]).await?)?;
        let id = "harmos.v1.Status/Get";
        assert!(list["operations"].as_array().unwrap().iter().any(|entry| entry["contract"]["id"] == id));
        let reply: serde_json::Value =
            serde_json::from_str(&super::run(&["--endpoint", &host.endpoint, "operation", "call", id, "--json", "{}"]).await?)?;
        let direct: serde_json::Value = serde_json::from_str(&super::run(&["--endpoint", &host.endpoint, "status", "--json"]).await?)?;
        assert_eq!(reply["status"]["identity"], direct["status"]["identity"]);
        assert_eq!(reply["status"]["available"], true);

        // Correct Invalid Typed Input and Retry From a JSON File
        let invalid = super::command()
            .args(["--endpoint", &host.endpoint, "operation", "call", id, "--json", r#"{"unknown_field":true}"#])
            .output()
            .await?;
        assert_eq!(invalid.status.code(), Some(2));
        assert!(String::from_utf8(invalid.stderr)?.contains(id));
        let path = host._root.path().join("request.json");
        std::fs::write(&path, b"{}")?;
        let reply: serde_json::Value = serde_json::from_str(
            &super::run(&["--endpoint", &host.endpoint, "operation", "call", id, "--input-file", path.to_str().unwrap()]).await?,
        )?;
        assert_eq!(reply["status"]["identity"], direct["status"]["identity"]);

        // An Unadmitted Action Is Named and Refused Without a Response
        let absent = "fixture.v1.Absent/Execute";
        let refused = super::command().args(["--endpoint", &host.endpoint, "operation", "call", absent, "--json", "{}"]).output().await?;
        assert_eq!(refused.status.code(), Some(3));
        assert!(refused.stdout.is_empty());
        assert!(String::from_utf8(refused.stderr)?.contains(absent));
        host.close().await?;
        Ok(())
    })
    .await?
}
