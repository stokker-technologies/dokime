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

        // Use Final Product Identities and Replace Whole Metadata with a Revision.
        let created: serde_json::Value = serde_json::from_str(
            &super::run(&[
                "--endpoint",
                &host.endpoint,
                "operation",
                "call",
                "dokime.client.v1.Cycles/Create",
                "--json",
                r#"{"request_id":"metadata-cycle"}"#,
            ])
            .await?,
        )?;
        let name = created["cycle"]["cycle_id"].as_str().unwrap();
        let input = serde_json::json!({"name": name}).to_string();
        let current: serde_json::Value = serde_json::from_str(
            &super::run(&["--endpoint", &host.endpoint, "operation", "call", "dokime.client.v1.Cycles/Get", "--json", &input]).await?,
        )?;
        let mut value = current["value"].clone();
        value["label"] = "Recorded Metadata".into();
        let request =
            serde_json::json!({"name": name, "value": value, "expected_revision": current["revision"], "request_id":"metadata-set"});
        let input = request.to_string();
        let updated: serde_json::Value = serde_json::from_str(
            &super::run(&["--endpoint", &host.endpoint, "operation", "call", "dokime.client.v1.Cycles/Set", "--json", &input]).await?,
        )?;
        assert_eq!(updated["value"], value);
        let replayed: serde_json::Value = serde_json::from_str(
            &super::run(&["--endpoint", &host.endpoint, "operation", "call", "dokime.client.v1.Cycles/Set", "--json", &input]).await?,
        )?;
        assert_eq!(replayed, updated);
        let mut stale = request;
        stale["request_id"] = "metadata-stale".into();
        stale["value"]["label"] = "Stale Write".into();
        let input = stale.to_string();
        let refused = super::command()
            .args(["--endpoint", &host.endpoint, "operation", "call", "dokime.client.v1.Cycles/Set", "--json", &input])
            .output()
            .await?;
        assert!(!refused.status.success());
        let retained: serde_json::Value = serde_json::from_str(
            &super::run(&[
                "--endpoint",
                &host.endpoint,
                "operation",
                "call",
                "dokime.client.v1.Cycles/Get",
                "--json",
                &serde_json::json!({"name":name}).to_string(),
            ])
            .await?,
        )?;
        assert_eq!(retained["value"], value);

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
