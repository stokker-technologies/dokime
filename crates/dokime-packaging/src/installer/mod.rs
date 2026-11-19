/* ***
 *
 * Dokime
 *
 * SPDX-License-Identifier: Apache-2.0
 *
 * ***
 */

/* === Modules === */

mod artwork;
mod distribution;
mod error;
mod license;
mod manifest;
mod native;
mod pack;
mod plugins;
mod policy;
mod release;
mod signing;

#[cfg(target_os = "windows")]
mod windows;

#[cfg(unix)]
mod unix;

#[cfg(test)]
mod fixtures;

/* === Re-Exports === */

pub use error::Error;

#[cfg(target_os = "windows")]
pub(super) use windows::{scratch, stage_executable};

#[cfg(all(test, target_os = "windows"))]
pub(super) use windows::executable;

#[cfg(unix)]
pub(super) use unix::{scratch, stage_executable};

pub use manifest::{Format, Manifest};
#[cfg(all(test, unix))]
pub(super) use unix::executable;

/* === Imports === */

// Standard Library Imports
use std::fs;
use std::path::{Path, PathBuf};
use std::{ffi::OsStr, process::Command};

// Third Party Imports

// Local Imports

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct PackageOptions {
    /// Constructs Tool Commands with the Product's Process Policy.
    pub command: fn(&OsStr) -> Command,

    pub artifacts: Option<PathBuf>,
    pub plugins: Vec<PathBuf>,
    pub output: PathBuf,
    pub sign: bool,
    pub formats: Vec<Format>,
    pub target: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

/* === Public Functions === */

pub async fn package(path: &Path, options: &PackageOptions) -> Result<Vec<PathBuf>, Error> {
    let mut manifest = Manifest::read(path)?;
    if !options.formats.is_empty() {
        manifest.installer.formats = options.formats.clone();
    }
    if let Some(target) = &options.target {
        manifest.dokime.target = target.clone();
    }
    manifest.validate()?;
    native::validate_signing(&manifest, options.sign)?;
    let scratch = scratch()?;
    let downloaded = scratch.path().join("distribution");
    let names: Vec<_> = manifest.plugin.iter().filter_map(|plugin| plugin.name.clone()).collect();
    let release = if options.artifacts.is_none() { Some(release::Release::resolve(manifest.dokime.version()).await?) } else { None };
    let source = if let Some(source) = &options.artifacts {
        source.canonicalize()?
    } else {
        fs::create_dir_all(&downloaded)?;
        let release = release.as_ref().ok_or_else(|| Error::Invalid("missing resolved release".into()))?;
        let version = release.distribution(manifest.dokime.version(), &manifest.dokime.target, &downloaded).await?;
        manifest.dokime.version = Some(version);
        downloaded
    };
    let distribution = distribution::validate(&source, &manifest.dokime)?;
    manifest.dokime.version = Some(distribution.version.clone());
    native::validate_signing(&manifest, options.sign)?;
    let mut selected = options.plugins.clone();
    if options.artifacts.is_some() && !names.is_empty() {
        let directory = source.join("plugins");
        let mut paths = Vec::new();
        for entry in
            fs::read_dir(&directory).map_err(|error| Error::Invalid(format!("Add Requested .dokime Bundles to {directory:?}: {error}")))?
        {
            let path = entry?.path();
            if path.extension().is_some_and(|extension| extension == "dokime") {
                paths.push(path);
            }
        }
        selected.extend(plugins::select(&paths, &names)?);
    }
    selected.extend(manifest.plugin_paths(&source.join("organization"))?);
    if let Some(release) = release {
        let paths = release.plugins(&names, &distribution.target, &distribution.version, &scratch.path().join("release-plugins")).await?;
        selected.extend(paths);
    }
    let plugins = plugins::stage(&selected, scratch.path(), &distribution.target, &distribution.version)?;
    let staged = scratch.path().join("installers");
    fs::create_dir_all(&staged)?;
    let outputs = native::assemble(&manifest, &distribution, &source, &plugins, scratch.path(), &staged, options)?;
    fs::create_dir_all(&options.output)?;
    let output = options.output.canonicalize()?;
    let mut published = Vec::new();
    for path in outputs {
        let destination = output.join(path.file_name().ok_or_else(|| Error::Invalid("invalid installer output".into()))?);
        fs::copy(&path, &destination)?;
        let signature = path.with_extension("exe.sig");
        if signature.is_file() {
            fs::copy(&signature, destination.with_extension("exe.sig"))?;
        } else if destination.extension().is_some_and(|extension| extension == "exe") {
            let stale = destination.with_extension("exe.sig");
            if stale.exists() {
                fs::remove_file(stale)?;
            }
            let latest = output.join("release.json");
            if latest.is_file() {
                let value: serde_json::Value = serde_json::from_slice(&fs::read(&latest)?)?;
                let named = value
                    .pointer("/platforms/windows-x86_64/url")
                    .and_then(serde_json::Value::as_str)
                    .and_then(|url| url.rsplit('/').next());
                if named == destination.file_name().and_then(|name| name.to_str()) {
                    fs::remove_file(latest)?;
                }
            }
        }
        published.push(destination);
    }
    if staged.join("release.json").is_file() {
        fs::copy(staged.join("release.json"), output.join("release.json"))?;
    }
    scratch.close()?;
    Ok(published)
}

/* === Private Functions === */

/* === Tests === */
