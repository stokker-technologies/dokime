/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Export Remote Access and Reconnect to the Same Host

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
async fn operator_pairs_with_host() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::HANG_GUARD, async {
        // Export Access, Enabling the Host's Saved Listener Automatically
        let host = super::HostProcess::start().await?;
        let path = host._root.path().join("operator.dokime-remote");
        let output = super::run(&[
            "--endpoint",
            &host.endpoint,
            "access",
            "pairing",
            "export",
            "--label",
            "Operator",
            "--out",
            path.to_str().unwrap(),
            "--json",
        ])
        .await?;
        let exported: serde_json::Value = serde_json::from_str(&output)?;
        let original = std::fs::read(&path)?;
        let mut pairing: serde_json::Value = serde_json::from_slice(&original)?;
        let key = pairing["access_key"].as_str().unwrap().to_owned();
        assert!(!output.contains(&key));

        // Reconnect With Pinned Trust and Verify Both Library and Product Reads
        let status: serde_json::Value =
            serde_json::from_str(&super::run(&["--remote", path.to_str().unwrap(), "status", "--json"]).await?)?;
        assert_eq!(status["status"]["identity"]["host_id"], pairing["backend_identifier"]);
        super::run(&["--remote", path.to_str().unwrap(), "plugin", "list", "--json"]).await?;
        super::run(&["--remote", path.to_str().unwrap(), "catalog", "list", "--json"]).await?;
        super::run(&["--remote", path.to_str().unwrap(), "execution", "list", "--json"]).await?;
        let call: serde_json::Value = serde_json::from_str(
            &super::run(&["--remote", path.to_str().unwrap(), "operation", "call", "harmos.v1.Status/Get", "--json", "{}"]).await?,
        )?;
        assert_eq!(call["status"]["identity"], status["status"]["identity"]);
        let environment = super::command().env("DOKIME_REMOTE", &path).args(["status", "--json"]).output().await?;
        assert!(environment.status.success());

        // Continue Past an Endpoint With Different TLS Trust
        let other = super::HostProcess::start().await?;
        let other_path = other._root.path().join("other.dokime-remote");
        super::run(&[
            "--endpoint",
            &other.endpoint,
            "access",
            "pairing",
            "export",
            "--label",
            "Other",
            "--out",
            other_path.to_str().unwrap(),
        ])
        .await?;
        let other_pairing: serde_json::Value = serde_json::from_slice(&std::fs::read(&other_path)?)?;
        pairing["endpoints"].as_array_mut().unwrap().insert(0, other_pairing["endpoints"][0].clone());
        std::fs::write(&path, serde_json::to_vec(&pairing)?)?;
        let fallback: serde_json::Value =
            serde_json::from_str(&super::run(&["--remote", path.to_str().unwrap(), "status", "--json"]).await?)?;
        assert_eq!(fallback["status"]["identity"], status["status"]["identity"]);
        other.close().await?;
        pairing = serde_json::from_slice(&original)?;

        // Refuse the Wrong Backend Without Leaking the Pairing Key
        pairing["backend_identifier"] = "another-backend".into();
        std::fs::write(&path, serde_json::to_vec(&pairing)?)?;
        let refused = super::command().args(["--remote", path.to_str().unwrap(), "status", "--json"]).output().await?;
        assert_eq!(refused.status.code(), Some(3));
        assert!(!String::from_utf8(refused.stderr)?.contains(&key));
        std::fs::write(&path, &original)?;

        // Existing Secret Files Are Never Replaced by Another Export
        let refused = super::command()
            .args(["--endpoint", &host.endpoint, "access", "pairing", "export", "--label", "Replacement", "--out", path.to_str().unwrap()])
            .output()
            .await?;
        assert_eq!(refused.status.code(), Some(2));
        assert_eq!(std::fs::read(&path)?, original);

        // Revoke This Pairing Through Access and Observe Its Next Connection Fail
        let request = serde_json::json!({"name": exported["pairing_id"]}).to_string();
        super::run(&["--endpoint", &host.endpoint, "operation", "call", "harmos.v1.access.Pairings/Delete", "--json", &request]).await?;
        let refused = super::command().args(["--remote", path.to_str().unwrap(), "status", "--json"]).output().await?;
        assert_eq!(refused.status.code(), Some(3));
        assert!(!String::from_utf8(refused.stderr)?.contains(&key));

        // Remove Every Protected Credential Through the Host Removal Operation
        host.close().await?;
        Ok(())
    })
    .await?
}
