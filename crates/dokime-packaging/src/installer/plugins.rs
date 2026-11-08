/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

// Verify and Stage Installer Plugin Bundles

/* === Modules === */

/* === Re-Exports === */

/* === Imports === */

// Standard Library Imports
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

// Third Party Imports
use harmos_packaging::bundle::{Bundle, Facility, Limits};

// Local Imports
use crate::{
    Error,
    bundle::{COMPATIBILITY_FILE, Compatibility},
};

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub(super) fn stage(paths: &[PathBuf], scratch: &Path, target: &str, version: &str) -> Result<BTreeMap<String, PathBuf>, Error> {
    let mut selected = BTreeMap::new();
    let external = scratch.join("external-plugins");
    let mut bundles = Vec::new();
    for (index, path) in paths.iter().enumerate() {
        if path.extension().is_some_and(|extension| extension == "dokime") {
            bundles.push(path.clone());
            continue;
        }
        let name = path.file_name().and_then(|name| name.to_str()).ok_or_else(|| Error::Invalid("invalid plugin archive name".into()))?;
        let metadata = std::fs::symlink_metadata(path)?;
        if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > dokime::distribution::ARCHIVE_MAX_BYTES as u64 {
            return Err(Error::Invalid("external plugin archive must be a bounded regular file".into()));
        }
        let archive = std::fs::read(path)?;
        let members = dokime::distribution::member_names(&archive, name).map_err(Error::Invalid)?;
        let directory = external.join(index.to_string());
        std::fs::create_dir_all(&directory)?;
        for member in members {
            let bytes = dokime::distribution::member_bytes(&archive, name, &member).map_err(Error::Invalid)?;
            let destination = directory.join(member);
            if destination.exists() {
                return Err(Error::Invalid("duplicate external plugin archive member".into()));
            }
            std::fs::write(&destination, bytes)?;
            bundles.push(destination);
        }
    }
    for path in &bundles {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| Error::Invalid(format!("invalid plugin bundle path: {}", path.display())))?;
        if selected.insert(name.to_owned(), path.clone()).is_some() {
            return Err(Error::Invalid(format!("duplicate installer plugin filename: {name}")));
        }
    }

    let mut names = BTreeSet::new();
    let mut packages = BTreeSet::new();
    let mut verified = Vec::new();
    for (name, path) in selected {
        super::distribution::relative(&name)?;
        let folded = name.to_lowercase();
        if !names.insert(folded) {
            return Err(Error::Invalid(format!("duplicate installer plugin filename: {name}")));
        }
        let inspected = verify(&path, target, version);
        let (bytes, package) = inspected.map_err(|error| Error::Invalid(format!("plugin bundle {}: {error}", path.display())))?;
        if !packages.insert(package.clone()) {
            return Err(Error::Invalid(format!("plugin bundle {}: duplicate package {package}", path.display())));
        }
        verified.push((name, bytes));
    }

    let mut staged = BTreeMap::new();
    if verified.is_empty() {
        return Ok(staged);
    }
    let directory = scratch.join("plugins");
    std::fs::create_dir(&directory)?;
    for (name, bytes) in verified {
        let destination = directory.join(&name);
        std::fs::write(&destination, bytes)?;
        staged.insert(name, destination);
    }
    Ok(staged)
}

/* === Private Functions === */

fn verify(path: &Path, target: &str, version: &str) -> Result<(Vec<u8>, String), Error> {
    if path.extension().is_none_or(|extension| extension != "dokime") {
        return Err(Error::Invalid("installer plugins must be .dokime bundles".into()));
    }
    let bytes = crate::bundle::read(path)?;
    let bundle = Bundle::inspect_with_settings(&bytes, Limits::default(), &dokime::config::schema_defaults())?;
    let executables: Vec<_> = bundle.manifest().files.iter().filter_map(|file| file.executable.as_ref()).collect();
    if executables.is_empty()
        || executables.iter().any(|executable| executable.target != target || executable.facility != Facility::Sidecar)
    {
        return Err(Error::Invalid(format!("expected only native sidecars for target {target}")));
    }
    let compatibility = bundle.file(COMPATIBILITY_FILE).ok_or_else(|| Error::Invalid("missing Dokime compatibility metadata".into()))?;
    let compatibility: Compatibility = serde_json::from_slice(compatibility)?;
    let version = version.parse().map_err(|error: semver::Error| Error::Invalid(error.to_string()))?;
    if compatibility.schema != 1 || !compatibility.requires_dokime.matches(&version) {
        return Err(Error::Invalid(format!("requires Dokime {}, installer has {version}", compatibility.requires_dokime)));
    }
    Ok((bytes, bundle.manifest().identity.name.clone()))
}

/* === Tests === */
