/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Verified Managed Plugin Bundle Fixtures

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports

// Third Party Imports

// Local Imports
use dokime::Client;
use dokime::FileReference;
use std::env::consts::EXE_SUFFIX;
use std::env::consts::OS;
use std::error::Error;
use tempfile::NamedTempFile;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn bundle(executable: &[u8], name: &str, version: &str) -> Result<Vec<u8>, Box<dyn Error>> {
    use sha2::Digest;
    let target = match OS {
        "windows" => "x86_64-pc-windows-msvc",
        _ => "x86_64-unknown-linux-gnu",
    };
    let filename = format!("bin/fixture{}", EXE_SUFFIX);
    let compatibility = br#"{"schema":1,"requires_dokime":">=0.1.0"}"#;
    let manifest = serde_json::to_vec(
        &serde_json::json!({ "schema": 1, "identity": { "name": name, "version": version, "description": "Thermal Fixture" }, "files": [
        { "path": filename, "size": executable.len(), "sha256": sha2::Sha256::digest(executable).iter().map(|byte| format!("{byte:02x}")).collect::<String>(), "executable": { "facility": "sidecar", "target": target } },
        { "path": "dokime.json", "size": compatibility.len(), "sha256": sha2::Sha256::digest(compatibility).iter().map(|byte| format!("{byte:02x}")).collect::<String>() }
    ] }),
    )?;
    let mut archive = tar::Builder::new(Vec::new());
    for (path, bytes) in
        [("manifest.json", manifest.as_slice()), (filename.as_str(), executable), ("dokime.json", compatibility.as_slice())]
    {
        let mut header = tar::Header::new_ustar();
        header.set_size(bytes.len() as u64);
        header.set_mode(0o700);
        header.set_cksum();
        archive.append_data(&mut header, path, bytes)?;
    }
    Ok(archive.into_inner()?)
}
pub async fn upload(client: &Client, bytes: Vec<u8>) -> Result<FileReference, Box<dyn Error>> {
    use std::io::Write;
    // Stage Fixture Bytes for the SDK's Bounded Upload Journey
    let mut file = NamedTempFile::new()?;
    file.write_all(&bytes)?;
    Ok(client.files.upload(file.path()).await?)
}

/* === Private Functions === */

/* === Tests === */
