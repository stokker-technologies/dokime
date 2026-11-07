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
use std::{collections::BTreeSet, path::PathBuf};

// Third Party Imports
use cargo_metadata::Package;
use harmos_packaging::{Build, config::BuildSettings};

// Local Imports
use super::{Options, bundle, metadata, select_package, settings};
use crate::Error;

/* === Constants === */

/* === Macros === */

/* === Type Aliases === */

/* === Structs === */

// A plugin package resolved once for a development session.
pub struct Project {
    pub package: Package,
    pub settings: BuildSettings,
    pub sdk_version: semver::Version,
    pub sdk_source: Option<String>,
}

/* === Enums === */

/* === Traits === */

/* === Implementations === */

impl Project {
    pub async fn resolve(options: &Options) -> Result<Self, Error> {
        let metadata = metadata(options, true).await?;
        let selected = select_package(&metadata, options.package.as_deref())?;
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
        let [sdk] = sdks.as_slice() else {
            return Err(Error::Invalid("selected plugin must resolve exactly one Dokime SDK dependency".into()));
        };
        Ok(Self {
            settings: settings(options, selected)?,
            sdk_version: sdk.version.clone(),
            sdk_source: sdk.source.as_ref().map(|source| source.repr.clone()),
            package: selected.clone(),
        })
    }

    pub async fn package(&self, build: Build, output: PathBuf) -> Result<PathBuf, Error> {
        bundle(self.package.clone(), build, output, None).await
    }
}

/* === Public Functions === */

/* === Private Functions === */

/* === Tests === */
