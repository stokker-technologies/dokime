/* ***
 *
 * dokime-rust-workspace
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Stage Verified gRPC Plugin Bundles

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::io;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::Read,
    path::{Path, PathBuf},
};

// Third Party Imports
use flate2::read::GzDecoder;
use harmos_runtime::bundle::{Bundle, Facility, Identity, Limits};

// Local Imports
use super::{Error, distribution};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub(super) struct Staged {
    pub path: PathBuf,
    pub identity: Identity,
    pub source: PathBuf,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn unpack(bytes: &[u8], name: &str, destination: &Path) -> Result<Vec<PathBuf>, Error> {
    fs::create_dir_all(destination)?;
    if name.ends_with(".zip") {
        distribution::extract(bytes, destination, Limits::default().archive_bytes as u64)?;
    } else if name.ends_with(".tar.gz") {
        let mut archive = tar::Archive::new(GzDecoder::new(bytes));
        let mut count = 0;
        let mut expanded = 0u64;
        for entry in archive.entries()? {
            let mut entry = entry?;
            count += 1;
            if entry.size() > Limits::default().archive_bytes as u64 {
                return Err(Error::Invalid("Reduce Plugin Archive Members Below the Bundle Size Limit".into()));
            }
            expanded = expanded.saturating_add(entry.size());
            if count > distribution::ARCHIVE_MAX_ENTRIES || expanded > distribution::EXPANDED_MAX_BYTES {
                return Err(Error::Invalid("Reduce the Plugin Archive Inventory".into()));
            }
            let name = entry.path()?.to_str().ok_or_else(|| Error::Invalid("Use UTF-8 Plugin Archive Paths".into()))?.to_owned();
            let path = destination.join(distribution::relative(name.trim_end_matches('/'))?);
            if entry.header().entry_type().is_dir() {
                fs::create_dir_all(path)?;
                continue;
            }
            if !entry.header().entry_type().is_file() {
                return Err(Error::Invalid("Remove Links and Special Files From the Plugin Archive".into()));
            }
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut file = fs::OpenOptions::new().write(true).create_new(true).open(path)?;
            io::copy(&mut entry, &mut file)?;
        }
    } else {
        return Err(Error::Invalid("Supply .dokime, .zip or .tar.gz Plugin Inputs".into()));
    }
    let mut paths = Vec::new();
    let mut pending = vec![destination.to_owned()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory)? {
            let path = entry?.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|value| value == "dokime") {
                paths.push(path);
            }
        }
    }
    if paths.is_empty() {
        return Err(Error::Invalid("Include at Least One .dokime Bundle in the Plugin Archive".into()));
    }
    paths.sort();
    Ok(paths)
}

pub(super) fn select(paths: &[PathBuf], names: &[String]) -> Result<Vec<PathBuf>, Error> {
    let mut packages = BTreeMap::new();
    for path in paths {
        let bytes = bounded(path, Limits::default().archive_bytes as u64)?;
        let bundle = Bundle::inspect(&bytes, Limits::default())?;
        let name = bundle.manifest().identity.name.clone();
        if packages.insert(name.clone(), path.clone()).is_some() {
            return Err(Error::Invalid(format!("Publish Each Plugin Package Once: {name}")));
        }
    }
    names
        .iter()
        .map(|name| {
            packages
                .get(name)
                .cloned()
                .ok_or_else(|| Error::Invalid(format!("Select a Plugin Name Declared by a Bundle; No {name} in the Plugin Archive")))
        })
        .collect()
}

pub(super) fn stage(paths: &[PathBuf], scratch: &Path, target: &str, version: &str) -> Result<BTreeMap<String, Staged>, Error> {
    let mut bundles = Vec::new();
    for (index, path) in paths.iter().enumerate() {
        if path.extension().is_some_and(|value| value == "dokime") {
            bundles.push((path.clone(), path.clone()));
            continue;
        }
        let bytes = bounded(path, distribution::ARCHIVE_MAX_BYTES as u64)?;
        let name =
            path.file_name().and_then(|value| value.to_str()).ok_or_else(|| Error::Invalid("Use UTF-8 Plugin Archive Names".into()))?;
        let unpacked = unpack(&bytes, name, &scratch.join("external-plugins").join(index.to_string()))?;
        bundles.extend(unpacked.into_iter().map(|bundle| (bundle, path.clone())));
    }
    let mut names = BTreeSet::new();
    let mut packages = BTreeSet::new();
    let mut verified = Vec::new();
    for (path, source) in bundles {
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| Error::Invalid("Use UTF-8 Plugin Filenames".into()))?
            .to_owned();
        distribution::relative(&name)?;
        if !names.insert(name.to_ascii_lowercase()) {
            return Err(Error::Invalid(format!("Select Each Installer Plugin Filename Once: {name}")));
        }
        let bytes = bounded(&path, Limits::default().archive_bytes as u64)?;
        let bundle = Bundle::inspect(&bytes, Limits::default())?;
        if bundle
            .manifest()
            .files
            .iter()
            .filter_map(|file| file.executable.as_ref())
            .any(|executable| executable.facility != Facility::Sidecar || executable.target != target)
        {
            return Err(Error::Invalid(format!("Rebuild Plugin {name} for {target}")));
        }
        if let Some(metadata) = bundle.file("dokime.json") {
            let metadata: serde_json::Value = serde_json::from_slice(metadata)?;
            let requirement = match metadata.get("requires_dokime") {
                None | Some(serde_json::Value::Null) => None,
                Some(serde_json::Value::String(value)) => Some(value.as_str()),
                _ => return Err(Error::Invalid(format!("Rebundle {name} With a Valid requires_dokime Requirement"))),
            };
            crate::project::check_compatibility(requirement, version)
                .map_err(|error| Error::Invalid(format!("Rebundle {name} for Dokime {version}: {error}")))?;
        }
        let package = &bundle.manifest().identity.name;
        if !packages.insert(package.clone()) {
            return Err(Error::Invalid(format!("Select Plugin Package {package} Only Once")));
        }
        let identity = bundle.manifest().identity.clone();
        verified.push((name, bytes, identity, source));
    }
    let destination = scratch.join("plugins");
    fs::create_dir_all(&destination)?;
    let mut staged = BTreeMap::new();
    for (name, bytes, identity, source) in verified {
        let path = destination.join(&name);
        fs::write(&path, bytes)?;
        staged.insert(name, Staged { path, identity, source });
    }
    Ok(staged)
}

/* === Private Functions === */

fn bounded(path: &Path, maximum: u64) -> Result<Vec<u8>, Error> {
    let kind = fs::symlink_metadata(path)?;
    if !kind.is_file() || kind.file_type().is_symlink() || kind.len() > maximum {
        return Err(Error::Invalid(format!("Supply a Bounded Regular Plugin File: {}", path.display())));
    }
    let mut bytes = Vec::new();
    fs::File::open(path)?.take(maximum + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > maximum {
        return Err(Error::Invalid("Reduce Plugin Input Size".into()));
    }
    Ok(bytes)
}

/* === Tests === */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle;
    use harmos_runtime::bundle::Identity;
    use std::io::{Cursor, Write};
    use std::slice;
    use zip::{ZipWriter, write::SimpleFileOptions};

    #[test]
    fn stage_verified_grpc_bundle_and_refuse_duplicates_corruption_target_and_version() -> crate::Result<()> {
        let root = tempfile::tempdir()?;
        let executable = root.path().join("example-pack");
        fs::write(&executable, b"executable fixture")?;
        let identity = Identity {
            name: "example-pack".into(),
            version: "1.2.3".parse()?,
            description: String::new(),
            package: Default::default(),
        };
        let bytes = bundle::create(&executable, &[], identity, None)?;
        let path = root.path().join("example-pack.dokime");
        fs::write(&path, &bytes)?;
        let selected = select(slice::from_ref(&path), &["example-pack".into()])?;
        assert_eq!(selected.as_slice(), slice::from_ref(&path));
        assert!(select(slice::from_ref(&path), &["missing-example-pack".into()]).is_err());
        let staged = tempfile::tempdir()?;
        let result = stage(slice::from_ref(&path), staged.path(), bundle::native_target(), env!("CARGO_PKG_VERSION"))?;
        assert_eq!(fs::read(&result["example-pack.dokime"].path)?, bytes);
        assert_eq!(result["example-pack.dokime"].source, path);
        let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
        archive.start_file("example-pack.dokime", SimpleFileOptions::default())?;
        archive.write_all(&bytes)?;
        let archive = archive.finish()?.into_inner();
        let archive_path = root.path().join("bench.zip");
        fs::write(&archive_path, archive)?;
        let output = tempfile::tempdir()?;
        let result = stage(slice::from_ref(&archive_path), output.path(), bundle::native_target(), env!("CARGO_PKG_VERSION"))?;
        assert_eq!(result["example-pack.dokime"].source, archive_path);
        assert_eq!(fs::read(&result["example-pack.dokime"].path)?, bytes);
        assert!(stage(&[path.clone(), path.clone()], staged.path(), bundle::native_target(), env!("CARGO_PKG_VERSION")).is_err());
        assert!(stage(slice::from_ref(&path), staged.path(), "unsupported", env!("CARGO_PKG_VERSION")).is_err());
        assert!(stage(slice::from_ref(&path), staged.path(), bundle::native_target(), "99.0.0").is_err());
        let second = root.path().join("second.dokime");
        fs::write(&second, &bytes)?;
        assert!(select(&[path.clone(), second.clone()], &["example-pack".into()]).is_err());
        assert!(stage(&[path.clone(), second], staged.path(), bundle::native_target(), env!("CARGO_PKG_VERSION")).is_err());
        fs::write(&path, b"corrupt bundle")?;
        assert!(stage(&[path], staged.path(), bundle::native_target(), env!("CARGO_PKG_VERSION")).is_err());
        Ok(())
    }
}
