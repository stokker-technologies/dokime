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
use harmos_runtime::bundle::{Bundle as PluginBundle, Limits};

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn inspect(path: &Path) -> crate::Result<serde_json::Value> {
    let limits = Limits::default();
    let bytes = read(path, limits.archive_bytes as u64)?;
    let bundle = PluginBundle::inspect(&bytes, limits)?;
    Ok(serde_json::to_value(bundle.manifest())?)
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

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::inspect;
    use harmos_runtime::bundle::Manifest;
    use sha2::{Digest, Sha256};
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    fn digest(bytes: &[u8]) -> String {
        Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
    }

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
        let manifest: Manifest = serde_json::from_value(manifest.clone())?;
        let bytes = serde_json::to_vec(&manifest)?;
        for (name, bytes) in std::iter::once(("manifest.json", bytes.as_slice())).chain(files.iter().copied()) {
            let mut header = tar::Header::new_ustar();
            header.set_entry_type(tar::EntryType::Regular);
            header.set_uid(0);
            header.set_gid(0);
            header.set_mtime(0);
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
        let manifest = serde_json::json!({"schema": 1, "identity": {"name": "example", "version": "1.0.0", "description": "", "package": {"authors": [], "license": null, "repository": null, "homepage": null, "documentation": null, "keywords": [], "categories": []}},
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
        let manifest = serde_json::json!({"schema": 1, "identity": {"name": "example", "version": "1.0.0", "description": ""},
            "files": [{"path": "bin/plugin", "size": bytes.len(), "sha256": digest(bytes)}]});
        let file = bundle(&[("bin/plugin", bytes), ("bin/plugin", bytes)], &manifest)?;
        assert!(inspect(file.path()).is_err());

        // Refuse an Inventory That Names Absent Content
        let file = bundle(&[], &manifest)?;
        assert!(inspect(file.path()).is_err());
        Ok(())
    }
}
