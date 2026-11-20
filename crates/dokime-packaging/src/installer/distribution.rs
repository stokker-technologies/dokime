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

/* === Imports === */

// Standard Library Imports
use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::io::Cursor;
use std::str;
use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
};

// Third Party Imports
use semver::Version;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use zip::ZipArchive;

// Local Imports
use super::Error;
use super::manifest::Artifact;

/* === Constants === */

pub(super) const ARCHIVE_MAX_BYTES: usize = 2 * 1024 * 1024 * 1024;
pub(super) const ARCHIVE_MAX_ENTRIES: usize = 10_000;
pub(super) const EXPANDED_MAX_BYTES: u64 = 4 * 1024 * 1024 * 1024;

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Distribution {
    pub schema: u32,
    pub version: String,
    pub target: String,
    pub source_revision: String,
    pub channel: String,
    pub plugins: Vec<String>,

    #[serde(skip)]
    pub identity: Identity,

    #[serde(skip)]
    pub source: PathBuf,

    pub files: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Identity {
    pub identifier: String,
    pub slug: String,
    pub updater_public_key: String,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Identity {
    pub(super) fn compiled() -> Result<Self, Error> {
        let config: Value = serde_json::from_str(include_str!("../../../dokime-gui/src-tauri/tauri.conf.json"))?;
        let identifier =
            config["identifier"].as_str().ok_or_else(|| Error::Invalid("Rebuild the CLI With Its Product Identifier".into()))?;
        let mut identity = Self::new(identifier.to_owned()).map_err(Error::Invalid)?;
        identity.updater_public_key = config["plugins"]["updater"]["pubkey"]
            .as_str()
            .filter(|key| !key.is_empty())
            .ok_or_else(|| Error::Invalid("Rebuild the CLI With Its Updater Verification Key".into()))?
            .to_owned();
        Ok(identity)
    }

    pub fn new(identifier: String) -> Result<Self, String> {
        let valid = identifier.len() <= 253
            && identifier.contains('.')
            && identifier.split('.').all(|label| {
                let bytes = label.as_bytes();
                bytes.first().is_some_and(u8::is_ascii_alphanumeric)
                    && bytes.last().is_some_and(u8::is_ascii_alphanumeric)
                    && label.len() <= 63
                    && bytes.iter().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
            });
        if !valid {
            return Err("Set installer.identifier to a Lowercase Reverse-DNS Identifier".into());
        }
        Ok(Self {
            slug: identifier.replace('.', "-"),
            identifier,
            updater_public_key: String::new(),
        })
    }
    pub fn slug(&self) -> String {
        self.slug.clone()
    }
}

pub fn digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn checksums(bytes: &[u8]) -> Result<BTreeMap<String, String>, Error> {
    let mut sums = BTreeMap::new();
    for line in str::from_utf8(bytes).map_err(|_| Error::Invalid("Download UTF-8 SHA256SUMS".into()))?.lines() {
        let (hash, name) = line.split_once("  ").ok_or_else(|| Error::Invalid("Download Valid SHA256SUMS Lines".into()))?;
        if hash.len() != 64
            || !hash.bytes().all(|byte| byte.is_ascii_hexdigit())
            || !filename(name)
            || sums.insert(name.into(), hash.to_ascii_lowercase()).is_some()
        {
            return Err(Error::Invalid("Download SHA256SUMS With Unique Filenames and SHA-256 Digests".into()));
        }
    }
    Ok(sums)
}

pub fn filename(name: &str) -> bool {
    !name.is_empty() && !matches!(name, "." | "..") && name.bytes().all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
}

pub fn verify(bytes: &[u8], expected: &str) -> Result<(), Error> {
    if digest(bytes).eq_ignore_ascii_case(expected) {
        return Ok(());
    }
    Err(Error::Invalid("Download the Artifact Again: Release SHA-256 Verification Failed".into()))
}

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
    let mut distribution: Distribution = serde_json::from_slice(&fs::read(directory.join("distribution.json"))?)?;
    distribution.source = directory.canonicalize()?;
    Version::parse(&distribution.version).map_err(|error| Error::Invalid(error.to_string()))?;
    if distribution.schema != 1
        || (expected.version() != super::release::NIGHTLY && distribution.version != expected.version())
        || distribution.target != expected.target
    {
        return Err(Error::Invalid("distribution schema, version or target differs from the pinned Dokime artifact".into()));
    }
    let suffix = if expected.target.contains("windows") { ".exe" } else { "" };
    let required = [
        format!("bin/dokime{suffix}"),
        format!("bin/dokime-gui{suffix}"),
        "LICENSE".into(),
        "NOTICE".into(),
        "THIRD-PARTY-NOTICES".into(),
        "organization/organization.toml".into(),
    ];
    if required.iter().any(|name| !distribution.files.contains_key(name)) {
        return Err(Error::Invalid("distribution is missing required executable, license or organization entries".into()));
    }
    for (name, expected) in &distribution.files {
        let path = directory.join(relative(name)?);
        if !path.canonicalize()?.starts_with(directory.canonicalize()?) {
            return Err(Error::Invalid(format!("distribution entry escapes its root: {name}")));
        }
        if fs::symlink_metadata(&path)?.file_type().is_symlink() || !path.is_file() {
            return Err(Error::Invalid(format!("distribution entry is not a regular file: {name}")));
        }
        if &digest(&fs::read(path)?) != expected {
            return Err(Error::DistributionDigest { member: name.clone() });
        }
    }
    let mut pending = vec![directory.join("organization")];
    while let Some(folder) = pending.pop() {
        for entry in fs::read_dir(folder)? {
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
    distribution.identity = Identity::compiled()?;
    Ok(distribution)
}

pub fn extract(bytes: &[u8], destination: &Path, maximum_member_bytes: u64) -> Result<(), Error> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|e| Error::Invalid(e.to_string()))?;
    if archive.has_duplicate_names() || archive.len() > ARCHIVE_MAX_ENTRIES {
        return Err(Error::Invalid("release distribution has too many entries".into()));
    }
    let mut expanded = 0u64;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|e| Error::Invalid(e.to_string()))?;
        if entry.size() > maximum_member_bytes {
            return Err(Error::Invalid("Reduce Archive Members Below the Allowed File Size".into()));
        }
        expanded = expanded.saturating_add(entry.size());
        if expanded > EXPANDED_MAX_BYTES {
            return Err(Error::Invalid("expanded release distribution exceeds 4 GiB".into()));
        }
        let entry_name = entry.name().map_err(|error| Error::Invalid(error.to_string()))?;
        let name = entry_name.trim_end_matches('/');
        let path = destination.join(relative(name)?);
        let mode = entry.unix_mode().unwrap_or(0) & 0o170000;
        let ordinary = mode == 0 || mode == 0o100000 || (mode == 0o040000 && entry.is_dir());
        if !ordinary {
            return Err(Error::Invalid("release distribution contains a link or special file".into()));
        }
        if entry.is_dir() {
            fs::create_dir_all(path)?;
            continue;
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = OpenOptions::new().write(true).create_new(true).open(&path)?;
        io::copy(&mut entry, &mut file)?;
    }
    Ok(())
}

/* === Private Functions === */

/* === Tests === */

#[cfg(test)]
mod tests {
    use std::io::Write;

    use zip::ZipWriter;
    use zip::write::SimpleFileOptions;

    use super::*;

    #[test]
    fn kit_integrity_refuses_mismatch_missing_files_and_unindexed_appearance() -> Result<(), Error> {
        let source = tempfile::tempdir()?;
        super::super::fixtures::kit(source.path())?;
        let manifest = super::super::fixtures::manifest();
        validate(source.path(), &manifest.dokime)?;
        let original = fs::read(source.path().join("LICENSE"))?;
        fs::write(source.path().join("LICENSE"), b"tampered")?;
        assert!(matches!(validate(source.path(), &manifest.dokime), Err(Error::DistributionDigest { .. })));
        fs::write(source.path().join("LICENSE"), original)?;
        fs::write(source.path().join("organization/unindexed.css"), b"unindexed")?;
        assert!(matches!(validate(source.path(), &manifest.dokime), Err(Error::UnindexedDistribution { .. })));
        fs::remove_file(source.path().join("organization/unindexed.css"))?;
        let path = source.path().join("distribution.json");
        let metadata: Value = serde_json::from_slice(&fs::read(&path)?)?;
        for (field, value) in
            [("schema", serde_json::json!(2)), ("version", serde_json::json!("2.0.0")), ("target", serde_json::json!("unsupported"))]
        {
            let mut invalid = metadata.clone();
            invalid[field] = value;
            fs::write(&path, serde_json::to_vec(&invalid)?)?;
            assert!(validate(source.path(), &manifest.dokime).is_err());
        }
        let mut invalid = metadata.clone();
        invalid["files"].as_object_mut().unwrap().remove("LICENSE");
        fs::write(&path, serde_json::to_vec(&invalid)?)?;
        assert!(validate(source.path(), &manifest.dokime).is_err());
        Ok(())
    }

    #[test]
    fn release_checksums_refuse_ambiguous_or_untrusted_bytes() {
        let bytes = b"verified bytes";
        let checksum = digest(bytes);
        let line = format!("{checksum}  example-pack.zip\n");
        assert_eq!(checksums(line.as_bytes()).unwrap()["example-pack.zip"], checksum);
        verify(bytes, &checksum).unwrap();
        assert!(verify(b"tampered", &checksum).is_err());
        for value in ["bad  example-pack.zip".to_owned(), format!("{checksum}  ../example-pack.zip"), format!("{line}{line}")] {
            assert!(checksums(value.as_bytes()).is_err());
        }
    }

    #[test]
    fn extraction_enforces_member_limits_before_writing() -> Result<(), Error> {
        let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
        archive.start_file("example-pack.txt", SimpleFileOptions::default()).unwrap();
        archive.write_all(b"abc")?;
        let bytes = archive.finish().unwrap().into_inner();
        let directory = tempfile::tempdir()?;
        assert!(extract(&bytes, directory.path(), 2).is_err());
        assert!(!directory.path().join("example-pack.txt").exists());
        extract(&bytes, directory.path(), 3)?;
        assert_eq!(fs::read(directory.path().join("example-pack.txt"))?, b"abc");
        Ok(())
    }

    #[test]
    fn extraction_accepts_typeless_members_and_refuses_special_files() -> Result<(), Error> {
        let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
        archive.add_directory("examples/", SimpleFileOptions::default()).unwrap();
        archive.start_file("examples/NOTICE", SimpleFileOptions::default()).unwrap();
        archive.write_all(b"notice")?;
        let bytes = archive.finish().unwrap().into_inner();
        let members: Vec<_> = bytes.windows(4).enumerate().filter_map(|(index, bytes)| (bytes == b"PK\x01\x02").then_some(index)).collect();
        assert_eq!(members.len(), 2);

        // Model Unix ZIP Writers That Store Permissions Without File-Type Bits
        let mut typeless = bytes.clone();
        for member in &members {
            typeless[member + 5] = 3;
            typeless[member + 38..member + 42].copy_from_slice(&(0o600u32 << 16).to_le_bytes());
        }
        let directory = tempfile::tempdir()?;
        extract(&typeless, directory.path(), 64)?;
        assert!(directory.path().join("examples").is_dir());
        assert_eq!(fs::read(directory.path().join("examples/NOTICE"))?, b"notice");

        // Explicit Links, Devices, Sockets and Other Special Types Stay Refused
        for mode in [0o120777u32, 0o020600, 0o060600, 0o010600, 0o140600] {
            let mut special = bytes.clone();
            let member = members[1];
            special[member + 5] = 3;
            special[member + 38..member + 42].copy_from_slice(&(mode << 16).to_le_bytes());
            let directory = tempfile::tempdir()?;
            let result = extract(&special, directory.path(), 64);
            assert!(matches!(result, Err(Error::Invalid(_))));
            assert!(!directory.path().join("examples/NOTICE").exists());
        }
        Ok(())
    }

    #[test]
    fn confined_paths() {
        for path in ["../escape", "/absolute", "C:/drive", "a\\b", ""] {
            assert!(relative(path).is_err());
        }
        assert!(relative("bin/dokime").is_ok());
    }
}
