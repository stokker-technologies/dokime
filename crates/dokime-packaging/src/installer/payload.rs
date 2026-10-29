/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

use std::{
    collections::BTreeMap,
    path::{Component, Path},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::manifest::Artifact;
use crate::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Payload {
    pub schema: u32,
    pub version: String,
    pub target: String,
    pub identity: Identity,
    pub files: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub identifier: String,
    pub slug: String,
    pub updater_public_key: String,
}

pub fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn relative(value: &str) -> Result<&Path, Error> {
    let path = Path::new(value);
    if value.is_empty()
        || value.split('/').any(|part| part.is_empty() || part == "." || part == "..")
        || value.contains('\\')
        || value.contains(':')
        || !path.components().all(|part| matches!(part, Component::Normal(_)))
    {
        return Err(Error::Invalid(format!("unsafe payload path: {value}")));
    }
    Ok(path)
}

pub fn validate(directory: &Path, expected: &Artifact) -> Result<Payload, Error> {
    let payload: Payload = serde_json::from_slice(&std::fs::read(directory.join("payload.json"))?)?;
    if payload.schema != 1 || payload.version != expected.version || payload.target != expected.target {
        return Err(Error::Invalid("payload schema, version or target differs from the pinned Dokime artifact".into()));
    }
    let identity = &payload.identity;
    if dokime::paths::DistributionIdentity::new(identity.identifier.clone()).is_err()
        || identity.updater_public_key.is_empty()
        || !safe_identity(&identity.slug)
    {
        return Err(Error::Invalid("payload compiled identity is invalid".into()));
    }
    let suffix = if expected.target.contains("windows") { ".exe" } else { "" };
    let required =
        [format!("bin/dokime{suffix}"), format!("bin/dokime-gui{suffix}"), "LICENSE".into(), "organization/organization.toml".into()];
    if required.iter().any(|name| !payload.files.contains_key(name)) {
        return Err(Error::Invalid("payload is missing required executable, license or organization entries".into()));
    }
    for (name, expected) in &payload.files {
        let path = directory.join(relative(name)?);
        if !path.canonicalize()?.starts_with(directory.canonicalize()?) {
            return Err(Error::Invalid(format!("payload entry escapes its root: {name}")));
        }
        if std::fs::symlink_metadata(&path)?.file_type().is_symlink() || !path.is_file() {
            return Err(Error::Invalid(format!("payload entry is not a regular file: {name}")));
        }
        if &digest(&std::fs::read(path)?) != expected {
            return Err(Error::PayloadDigest { member: name.clone() });
        }
    }
    let mut pending = vec![directory.join("organization")];
    while let Some(folder) = pending.pop() {
        for entry in std::fs::read_dir(folder)? {
            let entry = entry?;
            let path = entry.path();
            let kind = entry.file_type()?;
            if kind.is_symlink() || !path.canonicalize()?.starts_with(directory.canonicalize()?) {
                return Err(Error::Invalid("payload organization contains a link or escaped member".into()));
            }
            if kind.is_dir() {
                pending.push(path);
                continue;
            }
            let name = path
                .strip_prefix(directory)
                .map_err(|_| Error::Invalid("payload member escapes its root".into()))?
                .to_string_lossy()
                .replace('\\', "/");
            if !kind.is_file() || !payload.files.contains_key(&name) {
                return Err(Error::UnindexedPayload { member: name });
            }
        }
    }
    Ok(payload)
}

pub async fn download(artifact: &Artifact, destination: &Path) -> Result<(), Error> {
    let url =
        artifact.release_url.as_deref().ok_or_else(|| Error::Invalid("supply --artifacts or a pinned release_url and sha256".into()))?;
    let expected = artifact.sha256.as_deref().ok_or_else(|| Error::Invalid("release downloads require an expected SHA-256".into()))?;
    if !url.starts_with("https://") || expected.len() != 64 || !expected.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err(Error::Invalid("release download requires HTTPS and a SHA-256 digest".into()));
    }
    let _ = rustls::crypto::ring::default_provider().install_default();
    let response = reqwest::get(url).await.map_err(|_| Error::Invalid("release payload download failed".into()))?;
    let response = response.error_for_status().map_err(|_| Error::Invalid("release payload server refused the exact release".into()))?;
    let mut response = response;
    let mut bytes = Vec::new();
    const MAX_PAYLOAD: usize = 2 * 1024 * 1024 * 1024;
    while let Some(chunk) = response.chunk().await.map_err(|_| Error::Invalid("release payload download interrupted".into()))? {
        if bytes.len().saturating_add(chunk.len()) > MAX_PAYLOAD {
            return Err(Error::Invalid("release payload exceeds 2 GiB".into()));
        }
        bytes.extend_from_slice(&chunk);
    }
    if digest(&bytes) != expected.to_ascii_lowercase() {
        return Err(Error::Invalid("release payload SHA-256 mismatch".into()));
    }
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| Error::Invalid(e.to_string()))?;
    if archive.len() > 10000 {
        return Err(Error::Invalid("release payload has too many entries".into()));
    }
    let mut expanded = 0u64;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|e| Error::Invalid(e.to_string()))?;
        expanded = expanded.saturating_add(entry.size());
        if expanded > 4 * 1024 * 1024 * 1024 {
            return Err(Error::Invalid("expanded release payload exceeds 4 GiB".into()));
        }
        let entry_name = entry.name().map_err(|error| Error::Invalid(error.to_string()))?;
        let name = entry_name.trim_end_matches('/');
        let path = destination.join(relative(name)?);
        if entry.unix_mode().is_some_and(|mode| mode & 0o170000 == 0o120000) {
            return Err(Error::Invalid("release payload contains a symbolic link".into()));
        }
        if entry.is_dir() {
            std::fs::create_dir_all(path)?;
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&path)?;
        std::io::copy(&mut entry, &mut file)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = entry.unix_mode() {
                std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode & 0o777))?;
            }
        }
    }
    Ok(())
}

pub(super) fn safe_identity(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn confined_paths() {
        for path in ["../escape", "/absolute", "C:/drive", "a\\b", ""] {
            assert!(relative(path).is_err());
        }
        assert!(relative("bin/dokime").is_ok());
    }
    #[test]
    fn sha256_vector() {
        assert_eq!(digest(b"abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    }
}
