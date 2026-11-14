/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Bundle Any Native Plugin Executable Without Compiling

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

#[cfg(test)]
mod tests {
    use super::{digest, inspect};
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT: AtomicU64 = AtomicU64::new(0);

    struct Bundle(PathBuf);

    impl Bundle {
        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for Bundle {
        fn drop(&mut self) {
            std::fs::remove_file(&self.0).expect("Remove Owned Bundle Fixture");
        }
    }

    fn bundle(files: &[(&str, &[u8])], manifest: &serde_json::Value) -> Result<Bundle, Box<dyn std::error::Error + Send + Sync>> {
        let (fixture, mut file) = loop {
            let next = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("dokime-bundle-{}-{next}.tar", std::process::id()));
            match std::fs::OpenOptions::new().write(true).create_new(true).open(&path) {
                Ok(file) => break (Bundle(path), file),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error.into()),
            }
        };
        let mut archive = tar::Builder::new(&mut file);
        let bytes = serde_json::to_vec(manifest)?;
        for (name, bytes) in std::iter::once(("manifest.json", bytes.as_slice())).chain(files.iter().copied()) {
            let mut header = tar::Header::new_ustar();
            header.set_size(bytes.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            archive.append_data(&mut header, name, bytes)?;
        }
        archive.finish()?;
        drop(archive);
        file.flush()?;
        drop(file);
        Ok(fixture)
    }

    #[test]
    fn inspect_bundle_preserves_inventory_and_refuses_tampered_content() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Inspect the Declared Content Without Extracting It
        let bytes = b"plugin executable";
        let manifest = serde_json::json!({"schema": 1, "identity": {"name": "example"},
            "files": [{"path": "bin/plugin", "size": bytes.len(), "sha256": digest(bytes)}]});
        let file = bundle(&[("bin/plugin", bytes)], &manifest)?;
        assert_eq!(inspect(file.path())?, manifest);

        // Refuse Different Bytes at the Same Declared Path
        let file = bundle(&[("bin/plugin", b"changed executable")], &manifest)?;
        assert!(inspect(file.path()).is_err());

        // Refuse an Unlisted File While Preserving the Input Archive
        let file = bundle(&[("bin/plugin", bytes), ("extra", b"unlisted")], &manifest)?;
        let before = std::fs::read(file.path())?;
        assert!(inspect(file.path()).is_err());
        assert_eq!(std::fs::read(file.path())?, before);
        Ok(())
    }

    #[test]
    fn inspect_bundle_refuses_duplicate_members_and_missing_inventory() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Refuse Ambiguous Duplicate Archive Members
        let bytes = b"plugin executable";
        let manifest = serde_json::json!({"schema": 1,
            "files": [{"path": "bin/plugin", "size": bytes.len(), "sha256": digest(bytes)}]});
        let file = bundle(&[("bin/plugin", bytes), ("bin/plugin", bytes)], &manifest)?;
        assert!(inspect(file.path()).is_err());

        // Refuse an Inventory That Names Absent Content
        let file = bundle(&[], &manifest)?;
        assert!(inspect(file.path()).is_err());
        Ok(())
    }
}
