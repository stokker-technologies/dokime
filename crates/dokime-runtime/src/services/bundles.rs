/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Native Plugin Target

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

use harmos_runtime::bundle::Manifest;
use std::{env::consts::EXE_SUFFIX, path::Path};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn native_target() -> &'static str {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86_64", "windows") => "x86_64-pc-windows-msvc",
        ("aarch64", "windows") => "aarch64-pc-windows-msvc",
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu",
        _ => "unsupported",
    }
}

pub(crate) fn from_executable(executable: &Path) -> crate::Result<Vec<u8>> {
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(executable)?;
    let filename = format!("bin/plugin{}", EXE_SUFFIX);
    let digest = |bytes: &[u8]| Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect::<String>();
    let compatibility = br#"{"schema":1,"requires_dokime":">=0.1.0"}"#;
    let manifest: Manifest = serde_json::from_value(serde_json::json!({ "schema": 1,
        "identity": { "name": "native-plugin", "version": "0.1.0", "description": "Native Plugin" },
        "files": [
            { "path": filename, "size": bytes.len(), "sha256": digest(&bytes), "executable": {
                "facility": "sidecar", "target": native_target(), "registrations": []
            } },
            { "path": "dokime.json", "size": compatibility.len(), "sha256": digest(compatibility) }
        ]
    }))
    .map_err(|source| crate::Error::Invalid(format!("Create Plugin Bundle: {source}")))?;
    let manifest = serde_json::to_vec(&manifest).map_err(|source| crate::Error::Invalid(format!("Create Plugin Bundle: {source}")))?;
    let mut archive = tar::Builder::new(Vec::new());
    for (path, content) in
        [("manifest.json", manifest.as_slice()), (filename.as_str(), bytes.as_slice()), ("dokime.json", compatibility.as_slice())]
    {
        let mut header = tar::Header::new_ustar();
        header.set_entry_type(tar::EntryType::Regular);
        header.set_uid(0);
        header.set_gid(0);
        header.set_mtime(0);
        header.set_size(content.len() as u64);
        header.set_mode(if path == filename { 0o755 } else { 0o644 });
        header.set_cksum();
        archive.append_data(&mut header, path, content)?;
    }
    Ok(archive.into_inner()?)
}

/* === Private Functions === */

/* === Tests === */
