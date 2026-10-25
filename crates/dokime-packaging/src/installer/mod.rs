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
mod inspect;
mod manifest;
mod native;
mod payload;
mod signing;

#[cfg(windows)]
mod windows;

/* === Re-Exports === */

pub use inspect::{Inspection, inspect};
pub use manifest::{Format, Manifest};

/* === Imports === */

// Standard Library Imports
use std::path::{Path, PathBuf};

// Third Party Imports

// Local Imports
use crate::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

pub struct PackageOptions {
    pub artifacts: Option<PathBuf>,
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
    let scratch = tempfile::tempdir()?;
    let downloaded = scratch.path().join("payload");
    let source = if let Some(source) = &options.artifacts {
        source.canonicalize()?
    } else {
        std::fs::create_dir_all(&downloaded)?;
        payload::download(&manifest.dokime, &downloaded).await?;
        downloaded
    };
    let payload = payload::validate(&source, &manifest.dokime)?;
    std::fs::create_dir_all(&options.output)?;
    let output = options.output.canonicalize()?;
    let staged = scratch.path().join("installers");
    std::fs::create_dir_all(&staged)?;
    let outputs = native::assemble(&manifest, &payload, &source, scratch.path(), &staged, options.sign)?;
    let mut published = Vec::new();
    for path in outputs {
        let destination = output.join(path.file_name().ok_or_else(|| Error::Invalid("invalid installer output".into()))?);
        std::fs::copy(&path, &destination)?;
        let signature = path.with_extension("exe.sig");
        if signature.is_file() {
            std::fs::copy(&signature, destination.with_extension("exe.sig"))?;
        } else if destination.extension().is_some_and(|extension| extension == "exe") {
            let stale = destination.with_extension("exe.sig");
            if stale.exists() {
                std::fs::remove_file(stale)?;
            }
            let latest = output.join("latest.json");
            if latest.is_file() {
                let value: serde_json::Value = serde_json::from_slice(&std::fs::read(&latest)?)?;
                let named = value
                    .pointer("/platforms/windows-x86_64/url")
                    .and_then(serde_json::Value::as_str)
                    .and_then(|url| url.rsplit('/').next());
                if named == destination.file_name().and_then(|name| name.to_str()) {
                    std::fs::remove_file(latest)?;
                }
            }
        }
        published.push(destination);
    }
    if staged.join("latest.json").is_file() {
        std::fs::copy(staged.join("latest.json"), output.join("latest.json"))?;
    }
    scratch.close()?;
    Ok(published)
}

/* === Private Functions === */

/* === Tests === */
