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
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
// Third Party Imports

// Local Imports
use super::{Options, metadata, select_package};
use crate::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */
pub struct Project {
    options: Options,
    metadata: cargo_metadata::Metadata,
    package_name: String,
    sdk_version: String,
    sdk_source: Option<String>,
    watch_paths: Vec<PathBuf>,
    target_directory: PathBuf,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */
impl Project {
    pub async fn resolve(options: Options) -> Result<Self, Error> {
        let metadata = metadata(&options, true).await?;
        let selected = select_package(&metadata, options.package.as_deref())?;
        let package_name = selected.name.clone();
        let resolve = metadata.resolve.as_ref().ok_or_else(|| Error::Invalid("Cargo metadata omitted the dependency graph".into()))?;
        let mut pending = vec![selected.id.clone()];
        let mut seen = BTreeSet::new();
        while let Some(id) = pending.pop() {
            if !seen.insert(id.clone()) {
                continue;
            }
            if let Some(node) = resolve.nodes.iter().find(|node| node.id == id) {
                pending.extend(node.deps.iter().map(|dependency| dependency.pkg.clone()));
            }
        }
        let sdks = metadata.packages.iter().filter(|package| package.name == "dokime" && seen.contains(&package.id)).collect::<Vec<_>>();
        if sdks.len() != 1 {
            return Err(Error::Invalid("selected plugin must resolve exactly one Dokime SDK dependency".into()));
        }
        let sdk_version = sdks[0].version.to_string();
        let sdk_source = sdks[0].source.as_ref().map(|source| source.repr.clone());
        let mut watch_paths = metadata
            .packages
            .iter()
            .filter(|package| package.source.is_none() && seen.contains(&package.id))
            .map(|package| package.manifest_path.parent().expect("manifest parent").as_std_path().to_owned())
            .collect::<Vec<_>>();
        for file in ["Cargo.toml", "Cargo.lock", ".cargo/config.toml", ".cargo/config", "rust-toolchain.toml", "rust-toolchain"] {
            watch_paths.push(metadata.workspace_root.join(file).into_std_path_buf());
        }
        watch_paths.sort();
        watch_paths.dedup();
        let target_directory = options
            .target_dir
            .clone()
            .map(std::path::absolute)
            .transpose()?
            .unwrap_or_else(|| metadata.target_directory.clone().into_std_path_buf());
        Ok(Self { options, metadata, package_name: package_name.to_string(), sdk_version, sdk_source, watch_paths, target_directory })
    }

    pub fn package_name(&self) -> &str {
        &self.package_name
    }
    pub fn sdk_version(&self) -> &str {
        &self.sdk_version
    }
    pub fn sdk_source(&self) -> Option<&str> {
        self.sdk_source.as_deref()
    }
    pub fn watch_paths(&self) -> &[PathBuf] {
        &self.watch_paths
    }
    pub fn target_directory(&self) -> &Path {
        &self.target_directory
    }

    pub async fn build(self) -> Result<PathBuf, Error> {
        super::build_resolved(self.options, self.metadata).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
