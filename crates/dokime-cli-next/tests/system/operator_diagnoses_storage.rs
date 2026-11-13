/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Probe Scratch Storage and Preserve Existing Roots

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

fn files(root: &std::path::Path) -> std::io::Result<std::collections::BTreeMap<std::path::PathBuf, Vec<u8>>> {
    let mut files = std::collections::BTreeMap::new();
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            files.insert(entry.file_name().into(), std::fs::read(entry.path())?);
        }
    }
    Ok(files)
}

/* === Tests === */

#[tokio::test]
async fn operator_diagnoses_storage() -> Result<(), Box<dyn std::error::Error>> {
    tokio::time::timeout(super::HANG_GUARD, async {
        // Probe the Same Store a User Has Just Served
        let mut host = super::HostProcess::start().await?;
        let storage = host._root.path().join("storage");
        let path = storage.to_str().unwrap();
        let occupied = super::command().args(["doctor", "storage", "--storage", path, "--json"]).output().await?;
        assert_eq!(occupied.status.code(), Some(3));
        host.stop().await?;
        let before = files(&storage)?;
        let report: serde_json::Value = serde_json::from_str(&super::run(&["doctor", "storage", "--storage", path, "--json"]).await?)?;
        assert_eq!(report["compatible"], true);
        assert!(report["checks"].as_array().unwrap().len() > 1);
        assert_eq!(files(&storage)?, before);

        // Refuse Corrupt Storage While Retaining Every Original Byte
        let incompatible = host._root.path().join("incompatible");
        std::fs::create_dir(&incompatible)?;
        std::fs::write(incompatible.join("local.lock"), b"")?;
        std::fs::write(incompatible.join("execution.sqlite3"), b"incompatible database")?;
        let before = files(&incompatible)?;
        let refused = super::command().args(["doctor", "storage", "--storage", incompatible.to_str().unwrap(), "--json"]).output().await?;
        assert_eq!(refused.status.code(), Some(3));
        let report: serde_json::Value = serde_json::from_slice(&refused.stdout)?;
        assert_eq!(report["compatible"], false);
        assert!(report["recovery"].is_string());
        assert_eq!(files(&incompatible)?, before);

        // Diagnose Scratch Storage Without Creating a Requested Missing Root
        let missing = host._root.path().join("missing");
        let refused = super::command().args(["doctor", "storage", "--storage", missing.to_str().unwrap(), "--json"]).output().await?;
        assert!(!refused.status.success());
        assert!(!missing.exists());
        let report: serde_json::Value = serde_json::from_str(&super::run(&["doctor", "storage", "--json"]).await?)?;
        assert_eq!(report["compatible"], true);
        Ok(())
    })
    .await?
}
