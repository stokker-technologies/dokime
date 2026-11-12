/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Client gRPC Services

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::{Path, PathBuf},
};

// Third Party Imports
use serde::Deserialize;
use sha2::{Digest, Sha256};

// Local Imports
use crate::{Error, Result};
use dokime_protocol::v1;

/* === Constants === */

use dokime_operations_next::client::plugins_install::BUNDLE_BYTES;
const BUNDLE_FILES: usize = 4096;
const MANIFEST_BYTES: usize = 16777216;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Deserialize)]
struct Manifest {
    schema: u32,
    identity: Identity,
    files: Vec<File>,
}
#[derive(Deserialize)]
struct Identity {
    name: String,
    version: String,
    description: String,
}
#[derive(Deserialize)]
struct File {
    path: String,
    size: u64,
    sha256: String,
    executable: Option<Executable>,
}
#[derive(Deserialize)]
struct Executable {
    facility: String,
    target: String,
}
#[derive(Deserialize)]
struct Compatibility {
    schema: u32,
    requires_dokime: semver::VersionReq,
}
pub(super) struct Bundle {
    pub metadata: v1::PluginMetadata,
    pub executable: PathBuf,
    pub files: BTreeMap<String, Vec<u8>>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Bundle {
    pub fn read(bytes: &[u8]) -> Result<Self> {
        if bytes.is_empty() || bytes.len() > BUNDLE_BYTES {
            return Err(invalid("Choose a Nonempty Bundle Within the Archive Limit"));
        }
        let mut archive = tar::Archive::new(bytes);
        let mut entries = archive.entries()?.raw(true);
        let mut first = entries.next().ok_or_else(|| invalid("Put manifest.json First in the Bundle"))??;
        if first.path_bytes().as_ref() != b"manifest.json"
            || !first.header().entry_type().is_file()
            || first.size() as usize > MANIFEST_BYTES
        {
            return Err(invalid("Put a Bounded Regular manifest.json First in the Bundle"));
        }
        let mut json = Vec::new();
        first.read_to_end(&mut json)?;
        let manifest: Manifest = serde_json::from_slice(&json).map_err(|_| invalid("Rebuild the Bundle with a Valid Manifest"))?;
        if manifest.schema != 1
            || manifest.identity.name.trim().is_empty()
            || manifest.identity.name.len() > 128
            || manifest.files.is_empty()
            || manifest.files.len() > BUNDLE_FILES
        {
            return Err(invalid("Rebuild a v1 Bundle with a Package Name and Bounded File Inventory"));
        }
        semver::Version::parse(&manifest.identity.version).map_err(|_| invalid("Supply a Semantic Package Version"))?;
        let mut files = BTreeMap::new();
        let mut normalized = BTreeSet::new();
        let mut total = 0usize;
        for entry in entries {
            let mut entry = entry?;
            if !entry.header().entry_type().is_file() || entry.size() > BUNDLE_BYTES as u64 {
                return Err(invalid("Use Bounded Regular Bundle Files; Remove Links and Special Files"));
            }
            let name = std::str::from_utf8(entry.path_bytes().as_ref()).map_err(|_| invalid("Use UTF-8 Bundle Paths"))?.to_owned();
            portable(&name)?;
            if !normalized.insert(name.to_ascii_lowercase()) {
                return Err(invalid("Remove Duplicate or Case-Colliding Bundle Paths"));
            }
            total = total.checked_add(entry.size() as usize).ok_or_else(|| invalid("Reduce the Bundle Size"))?;
            if total > BUNDLE_BYTES || files.len() >= BUNDLE_FILES {
                return Err(invalid("Reduce the Bundle File Count and Expanded Size"));
            }
            let declared =
                manifest.files.iter().find(|file| file.path == name).ok_or_else(|| invalid("List Every Bundle File in the Manifest"))?;
            if declared.size != entry.size() {
                return Err(invalid("Rebuild the Bundle with Matching File Sizes"));
            }
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes)?;
            if digest(&bytes) != declared.sha256 {
                return Err(invalid("Rebuild the Bundle with Matching SHA-256 Digests"));
            }
            files.insert(name, bytes);
        }
        let mut declared = BTreeSet::new();
        for file in &manifest.files {
            portable(&file.path)?;
            if !declared.insert(&file.path) || !files.contains_key(&file.path) {
                return Err(invalid("List Each Existing Bundle File Exactly Once"));
            }
        }
        let compatibility =
            files.get("dokime.json").ok_or_else(|| invalid("Include dokime.json with the Supported Dokime Version Requirement"))?;
        let compatibility: Compatibility =
            serde_json::from_slice(compatibility).map_err(|_| invalid("Supply a Valid dokime.json Compatibility Declaration"))?;
        if compatibility.schema != 1
            || !compatibility
                .requires_dokime
                .matches(&semver::Version::parse(env!("CARGO_PKG_VERSION")).map_err(|_| invalid("Use a Matching Dokime Product Version"))?)
        {
            return Err(invalid("Install a Bundle Compatible with This Dokime Version"));
        }
        let target = native_target();
        let executables: Vec<_> = manifest
            .files
            .iter()
            .filter(|file| {
                file.executable.as_ref().is_some_and(|executable| executable.facility == "sidecar" && executable.target == target)
            })
            .collect();
        if executables.len() != 1 {
            return Err(invalid("Package Exactly One Managed gRPC Executable for This Platform"));
        }
        let executable = PathBuf::from(&executables[0].path);
        Ok(Self {
            metadata: v1::PluginMetadata {
                identity: Some(v1::PluginIdentity {
                    name: manifest.identity.name,
                    version: manifest.identity.version,
                    description: manifest.identity.description,
                }),
                owners: Vec::new(),
                size_bytes: bytes.len() as u64,
            },
            executable,
            files,
        })
    }
    pub fn extract(&self, root: &Path) -> Result<()> {
        for (name, bytes) in &self.files {
            let path = root.join(name);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, bytes)?;
        }
        super::bundle_permissions::executable(&root.join(&self.executable))?;
        Ok(())
    }
}

/* === Public Functions === */

pub(super) fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

/* === Private Functions === */

fn invalid(message: &str) -> Error {
    Error::Invalid(format!("Install Plugin: {message}"))
}
pub(super) fn native_target() -> &'static str {
    match (std::env::consts::ARCH, std::env::consts::OS) {
        ("x86_64", "windows") => "x86_64-pc-windows-msvc",
        ("aarch64", "windows") => "aarch64-pc-windows-msvc",
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("aarch64", "linux") => "aarch64-unknown-linux-gnu",
        _ => "unsupported",
    }
}
fn portable(name: &str) -> Result<()> {
    if name.is_empty() || name.len() > 100 || name.eq_ignore_ascii_case("manifest.json") {
        return Err(invalid("Use Portable Relative Bundle Paths of 1 to 100 Bytes"));
    }
    for part in name.split('/') {
        let stem = part.split('.').next().unwrap_or_default().to_ascii_uppercase();
        if part.is_empty()
            || matches!(part, "." | "..")
            || part.ends_with('.')
            || !part.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
            || matches!(
                stem.as_str(),
                "CON"
                    | "PRN"
                    | "AUX"
                    | "NUL"
                    | "COM1"
                    | "COM2"
                    | "COM3"
                    | "COM4"
                    | "COM5"
                    | "COM6"
                    | "COM7"
                    | "COM8"
                    | "COM9"
                    | "LPT1"
                    | "LPT2"
                    | "LPT3"
                    | "LPT4"
                    | "LPT5"
                    | "LPT6"
                    | "LPT7"
                    | "LPT8"
                    | "LPT9"
            )
        {
            return Err(invalid("Remove Unsafe or Nonportable Bundle Paths"));
        }
    }
    Ok(())
}

/* === Tests === */
