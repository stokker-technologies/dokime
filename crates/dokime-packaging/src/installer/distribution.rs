/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

/* === Re-Exports === */

pub use dokime::distribution::digest;

/* === Imports === */

// Standard Library Imports
use std::{
    collections::BTreeMap,
    path::{Component, Path},
};

// Third Party Imports
use serde::{Deserialize, Serialize};

// Local Imports
use super::manifest::Artifact;
use crate::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Distribution {
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

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub fn relative(value: &str) -> Result<&Path, Error> {
    let path = Path::new(value);
    if value.is_empty()
        || value.split('/').any(|part| part.is_empty() || part == "." || part == "..")
        || value.contains('\\')
        || value.contains(':')
        || !path.components().all(|part| matches!(part, Component::Normal(_)))
    {
        return Err(Error::Invalid(format!("unsafe distribution path: {value}")));
    }
    Ok(path)
}

pub fn validate(directory: &Path, expected: &Artifact) -> Result<Distribution, Error> {
    let distribution: Distribution = serde_json::from_slice(&std::fs::read(directory.join("distribution.json"))?)?;
    semver::Version::parse(&distribution.version).map_err(|error| Error::Invalid(error.to_string()))?;
    if distribution.schema != 1
        || (expected.version() != "dev-channel" && distribution.version != expected.version())
        || distribution.target != expected.target
    {
        return Err(Error::Invalid("distribution schema, version or target differs from the pinned Dokime artifact".into()));
    }
    let identity = &distribution.identity;
    if dokime::paths::DistributionIdentity::new(identity.identifier.clone()).is_err()
        || identity.updater_public_key.is_empty()
        || !safe_identity(&identity.slug)
    {
        return Err(Error::Invalid("distribution compiled identity is invalid".into()));
    }
    let suffix = if expected.target.contains("windows") { ".exe" } else { "" };
    let required =
        [format!("bin/dokime{suffix}"), format!("bin/dokime-gui{suffix}"), "LICENSE".into(), "organization/organization.toml".into()];
    if required.iter().any(|name| !distribution.files.contains_key(name)) {
        return Err(Error::Invalid("distribution is missing required executable, license or organization entries".into()));
    }
    for (name, expected) in &distribution.files {
        let path = directory.join(relative(name)?);
        if !path.canonicalize()?.starts_with(directory.canonicalize()?) {
            return Err(Error::Invalid(format!("distribution entry escapes its root: {name}")));
        }
        if std::fs::symlink_metadata(&path)?.file_type().is_symlink() || !path.is_file() {
            return Err(Error::Invalid(format!("distribution entry is not a regular file: {name}")));
        }
        if &digest(&std::fs::read(path)?) != expected {
            return Err(Error::DistributionDigest { member: name.clone() });
        }
    }
    let mut pending = vec![directory.join("organization")];
    while let Some(folder) = pending.pop() {
        for entry in std::fs::read_dir(folder)? {
            let entry = entry?;
            let path = entry.path();
            let kind = entry.file_type()?;
            if kind.is_symlink() || !path.canonicalize()?.starts_with(directory.canonicalize()?) {
                return Err(Error::Invalid("distribution organization contains a link or escaped member".into()));
            }
            if kind.is_dir() {
                pending.push(path);
                continue;
            }
            let name = path
                .strip_prefix(directory)
                .map_err(|_| Error::Invalid("distribution member escapes its root".into()))?
                .to_string_lossy()
                .replace('\\', "/");
            if !kind.is_file() || !distribution.files.contains_key(&name) {
                return Err(Error::UnindexedDistribution { member: name });
            }
        }
    }
    Ok(distribution)
}

pub fn extract(bytes: &[u8], destination: &Path) -> Result<(), Error> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|e| Error::Invalid(e.to_string()))?;
    if archive.has_duplicate_names() || archive.len() > dokime::distribution::ARCHIVE_MAX_ENTRIES {
        return Err(Error::Invalid("release distribution has too many entries".into()));
    }
    let mut expanded = 0u64;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|e| Error::Invalid(e.to_string()))?;
        expanded = expanded.saturating_add(entry.size());
        if expanded > 4 * 1024 * 1024 * 1024 {
            return Err(Error::Invalid("expanded release distribution exceeds 4 GiB".into()));
        }
        let entry_name = entry.name().map_err(|error| Error::Invalid(error.to_string()))?;
        let name = entry_name.trim_end_matches('/');
        let path = destination.join(relative(name)?);
        let mode = entry.unix_mode().unwrap_or(0o100644) & 0o170000;
        let ordinary = mode == 0o100000 || (mode == 0o040000 && entry.is_dir());
        if !ordinary {
            return Err(Error::Invalid("release distribution contains a link or special file".into()));
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

/* === Private Functions === */

pub(super) fn safe_identity(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}

/* === Tests === */

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
}
