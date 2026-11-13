/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bundle Any Native harmos.v1 Executable Without Compiling

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{io::Read, path::Path};

// Third Party Imports
use sha2::Digest;

// Local Imports

/* === Constants === */

const ARCHIVE_BYTES: u64 = 272 * 1024 * 1024;
const FILE_BYTES: u64 = 128 * 1024 * 1024;
const MANIFEST_BYTES: u64 = 16 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn inspect(path: &Path) -> crate::Result<serde_json::Value> {
    let bytes = read(path, ARCHIVE_BYTES)?;
    let mut archive = tar::Archive::new(bytes.as_slice());
    let mut contents = std::collections::BTreeMap::new();
    for entry in archive.entries()? {
        let entry = entry?;
        if !entry.header().entry_type().is_file() || contents.len() >= 4096 {
            return Err("Inspect Bundle: Supply a Bounded Archive of Regular Files".into());
        }
        let path = entry.path()?.into_owned();
        if path.components().any(|part| !matches!(part, std::path::Component::Normal(_)))
            || path.to_str().is_none_or(|value| value.contains('\\'))
        {
            return Err("Inspect Bundle: Invalid Archive Path".into());
        }
        let limit = if path == Path::new("manifest.json") { MANIFEST_BYTES } else { FILE_BYTES };
        if entry.size() > limit {
            return Err("Inspect Bundle: File Exceeds the Byte Bound".into());
        }
        let mut value = Vec::new();
        entry.take(limit + 1).read_to_end(&mut value)?;
        if value.len() as u64 > limit || contents.insert(path, value).is_some() {
            return Err("Inspect Bundle: Duplicate or Oversized File".into());
        }
    }
    let bytes = contents.remove(Path::new("manifest.json")).ok_or("Inspect Bundle: Missing manifest.json")?;
    let manifest: serde_json::Value = serde_json::from_slice(&bytes)?;
    if manifest["schema"] != 1 {
        return Err("Inspect Bundle: Unsupported Manifest Schema".into());
    }
    let files = manifest["files"].as_array().ok_or("Inspect Bundle: Missing File Inventory")?;
    for file in files {
        let path = file["path"].as_str().ok_or("Inspect Bundle: Missing File Path")?;
        let bytes = contents.remove(Path::new(path)).ok_or("Inspect Bundle: Missing or Duplicate Inventory File")?;
        if file["size"].as_u64() != Some(bytes.len() as u64) || file["sha256"].as_str() != Some(digest(&bytes).as_str()) {
            return Err("Inspect Bundle: File Size or SHA-256 Does Not Match".into());
        }
    }
    if !contents.is_empty() {
        return Err("Inspect Bundle: Unlisted Archive File".into());
    }
    Ok(manifest)
}

/* === Private Functions === */

fn read(path: &Path, limit: u64) -> crate::Result<Vec<u8>> {
    let file = std::fs::File::open(path)?;
    if file.metadata()?.len() > limit {
        return Err("Read Plugin Input: File Exceeds the Byte Bound".into());
    }
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err("Read Plugin Input: File Grew Beyond the Byte Bound".into());
    }
    Ok(bytes)
}

fn digest(bytes: &[u8]) -> String {
    sha2::Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

/* === Tests === */
